use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use rustix::fs::{AtFlags, Mode, OFlags, fchmod, openat, statat, unlinkat};

use super::*;

mod anonymous;

#[cfg(test)]
pub(super) use anonymous::test_kill_after_link;

const GENESIS_INTENT: &str = "genesis-init-intent.json";
const GENESIS_INTENT_STAGE_PREFIX: &str = ".genesis-init-intent.json.next-";

pub(in crate::coord::store::ledger) enum GenesisIntentCandidate {
    Absent,
    Published(Vec<u8>),
    Staged(Vec<u8>),
}

pub(in crate::coord::store::ledger) fn create_generation(
    lock: &CoordLock,
    generation_id: &str,
    manifest: &[u8],
) -> Result<GenerationFiles, CoordError> {
    validate_name(generation_id)?;
    let generations = ensure_dir_at(&lock.directory, "generations", DIR_MODE)?;
    if child_exists(&generations, generation_id)? {
        require_empty_or(&generations, generation_id)?;
        return lock.generation(generation_id, true);
    }
    let staging_name = format!(".next-{generation_id}");
    require_empty_or(&generations, &staging_name)?;
    let staging = ensure_dir_at(&generations, &staging_name, DIR_MODE)?;
    publish_file(&staging, "manifest.json", manifest, 0o400)?;
    ensure_dir_at(&staging, "pending", DIR_MODE)?;
    ensure_empty_at(&staging, "events.jsonl", 0o600)?;
    staging.sync_all().map_err(CoordError::io)?;
    generations.sync_all().map_err(CoordError::io)?;
    lock.generation_named(&staging_name, generation_id, true)
}

pub(in crate::coord::store::ledger) fn publish_generation(
    lock: &CoordLock,
    files: GenerationFiles,
) -> Result<GenerationFiles, CoordError> {
    if files.path_name == files.generation_id {
        return Ok(files);
    }
    files.revalidate(lock, true)?;
    let generations = open_dir_at(&lock.directory, "generations", DIR_MODE)?;
    rename_noreplace(&generations, &files.path_name, &files.generation_id)
        .map_err(CoordError::io)?;
    generations.sync_all().map_err(CoordError::io)?;
    let published = lock.generation(&files.generation_id, true)?;
    if published.identities != files.identities {
        return Err(changed(
            "published generation differs from staged descriptor identity",
        ));
    }
    Ok(published)
}

pub(in crate::coord::store::ledger) fn publish_current(
    lock: &CoordLock,
    bytes: &[u8],
) -> Result<(), CoordError> {
    publish_file(&lock.directory, "CURRENT", bytes, 0o400)?;
    let mut current = open_file_at(&lock.directory, "CURRENT", false, 0o400, None)?;
    exact(&mut current, bytes)
}

pub(in crate::coord::store::ledger) fn publish_genesis_intent(
    lock: &CoordLock,
    intent: &[u8],
) -> Result<(), CoordError> {
    publish_file(&lock.directory, GENESIS_INTENT, intent, 0o400)
}

pub(in crate::coord::store::ledger) fn genesis_intent_candidate(
    lock: &CoordLock,
) -> Result<GenesisIntentCandidate, CoordError> {
    if child_exists(&lock.directory, GENESIS_INTENT)? {
        let mut file = open_file_at(&lock.directory, GENESIS_INTENT, false, 0o400, None)?;
        return read_canonical(&mut file).map(GenesisIntentCandidate::Published);
    }
    let names = intent_stage_names(&lock.directory)?;
    let Some(name) = names.first() else {
        return Ok(GenesisIntentCandidate::Absent);
    };
    if names.len() != 1 {
        return Err(fence_unknown(
            "multiple Genesis initialization intent stages exist",
        ));
    }
    let mut stage = open_stage(&lock.directory, name, 0o400, false)?
        .ok_or_else(|| changed("Genesis initialization intent stage disappeared"))?;
    let bytes = read_canonical(&mut stage).map_err(|_| {
        fence_unknown("partial or unreadable Genesis initialization stage was preserved")
    })?;
    if stage_name(GENESIS_INTENT, &bytes)? != *name {
        return Err(fence_unknown(
            "sealed Genesis initialization intent stage has the wrong digest name",
        ));
    }
    Ok(GenesisIntentCandidate::Staged(bytes))
}

pub(in crate::coord::store::ledger) fn published_genesis_intent(
    lock: &CoordLock,
) -> Result<Vec<u8>, CoordError> {
    let mut file = open_file_at(&lock.directory, GENESIS_INTENT, false, 0o400, None)
        .map_err(|_| fence_unknown("published Genesis initialization intent is unavailable"))?;
    read_canonical(&mut file)
        .map_err(|_| fence_unknown("published Genesis initialization intent is not readable"))
}

fn intent_stage_names(parent: &File) -> Result<Vec<String>, CoordError> {
    let mut names = Vec::new();
    let mut directory = rustix::fs::Dir::read_from(parent)
        .map_err(|error| os_error("cannot inventory Genesis intent stages", error))?;
    while let Some(entry) = directory.read() {
        let entry =
            entry.map_err(|error| os_error("cannot inventory Genesis intent stages", error))?;
        let bytes = entry.file_name().to_bytes();
        if bytes.starts_with(GENESIS_INTENT_STAGE_PREFIX.as_bytes()) {
            let name = std::str::from_utf8(bytes)
                .map_err(|_| fence_unknown("Genesis intent stage name is not UTF-8"))?;
            validate_name(name)?;
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(names)
}

pub(super) fn publish_file(
    parent: &File,
    final_name: &str,
    bytes: &[u8],
    final_mode: u32,
) -> Result<(), CoordError> {
    validate_name(final_name)?;
    let stage_name = stage_name(final_name, bytes)?;
    if child_exists(parent, final_name)? {
        let mut final_file = open_file_at(parent, final_name, false, final_mode, None)?;
        exact(&mut final_file, bytes)?;
        retire_stage_if_present(parent, &stage_name, bytes, final_mode)?;
        return Ok(());
    }
    if let Some(mut stage) = open_stage(parent, &stage_name, final_mode, true)? {
        complete_stage(&mut stage, bytes, final_mode)?;
        return publish_stage(
            parent,
            final_name,
            &stage_name,
            &mut stage,
            bytes,
            final_mode,
        );
    }
    if anonymous::publish(parent, final_name, bytes, final_mode)? {
        return Ok(());
    }
    let mut final_file = open_file_at(parent, final_name, false, final_mode, None)?;
    exact(&mut final_file, bytes)
}

fn stage_name(final_name: &str, bytes: &[u8]) -> Result<String, CoordError> {
    let digest = bullet_wire::hash_framed_bytes("bullet.coord.staged-file.v2", bytes)
        .map_err(|error| invalid(format!("cannot derive staged-file name: {error}")))?;
    Ok(format!(".{final_name}.next-{}", digest.to_hex()))
}

fn open_stage(
    parent: &File,
    name: &str,
    final_mode: u32,
    writable: bool,
) -> Result<Option<File>, CoordError> {
    let descriptor = match openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(value) => value,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(os_error("cannot admit staged authority file", error)),
    };
    let file = File::from(descriptor);
    let mode = mode_of(&file)?;
    if mode != 0o600 && mode != final_mode {
        return Err(invalid("staged authority file has an invalid mode"));
    }
    validate_stage_file(&file)?;
    if writable && mode == 0o600 {
        let writable = File::from(
            openat(
                parent,
                name,
                OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| os_error("cannot reopen staged authority file", error))?,
        );
        validate_stage_file(&writable)?;
        if identity(&writable)? != identity(&file)? {
            return Err(changed("staged authority pathname was replaced"));
        }
        return Ok(Some(writable));
    }
    Ok(Some(file))
}

fn retire_stage_if_present(
    parent: &File,
    name: &str,
    bytes: &[u8],
    final_mode: u32,
) -> Result<(), CoordError> {
    let Some(mut stage) = open_stage(parent, name, final_mode, false)? else {
        return Ok(());
    };
    if stage_relation(&mut stage, bytes)? != StageRelation::Exact {
        return Err(fence_unknown(
            "incomplete or divergent stage beside final authority was preserved",
        ));
    }
    unlink_retained(parent, name, &stage)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum StageRelation {
    Exact,
    Prefix,
    Divergent,
}

fn stage_relation(file: &mut File, expected: &[u8]) -> Result<StageRelation, CoordError> {
    use std::os::unix::fs::MetadataExt;

    let before = file.metadata().map_err(CoordError::io)?;
    if before.len() > expected.len() as u64 {
        return Ok(StageRelation::Divergent);
    }
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut actual = Vec::new();
    Read::by_ref(file)
        .take(expected.len() as u64 + 1)
        .read_to_end(&mut actual)
        .map_err(CoordError::io)?;
    let after = file.metadata().map_err(CoordError::io)?;
    if (
        before.dev(),
        before.ino(),
        before.len(),
        before.ctime(),
        before.ctime_nsec(),
    ) != (
        after.dev(),
        after.ino(),
        after.len(),
        after.ctime(),
        after.ctime_nsec(),
    ) || actual.len() as u64 != before.len()
    {
        return Err(changed("staged authority changed while read"));
    }
    Ok(if actual == expected {
        StageRelation::Exact
    } else if expected.starts_with(&actual) {
        StageRelation::Prefix
    } else {
        StageRelation::Divergent
    })
}

fn complete_stage(file: &mut File, expected: &[u8], final_mode: u32) -> Result<(), CoordError> {
    match stage_relation(file, expected)? {
        StageRelation::Exact => {}
        StageRelation::Prefix if mode_of(file)? == 0o600 => {
            let offset = file.seek(SeekFrom::End(0)).map_err(CoordError::io)? as usize;
            file.write_all(&expected[offset..])
                .map_err(CoordError::io)?;
            file.sync_data().map_err(CoordError::io)?;
        }
        StageRelation::Prefix | StageRelation::Divergent => {
            return Err(fence_unknown(
                "ambiguous staged authority bytes were preserved",
            ));
        }
    }
    exact(file, expected)?;
    if mode_of(file)? == 0o600 {
        fchmod(&*file, Mode::from_bits_retain(final_mode))
            .map_err(|error| os_error("cannot seal staged authority file", error))?;
        file.sync_all().map_err(CoordError::io)?;
    }
    validate_file(file, final_mode, Some(expected.len() as u64))?;
    exact(file, expected)
}

fn publish_stage(
    parent: &File,
    final_name: &str,
    stage_name: &str,
    stage: &mut File,
    bytes: &[u8],
    final_mode: u32,
) -> Result<(), CoordError> {
    match rename_noreplace(parent, stage_name, final_name) {
        Ok(()) => parent.sync_all().map_err(CoordError::io)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut final_file = open_file_at(parent, final_name, false, final_mode, None)?;
            exact(&mut final_file, bytes)?;
            unlink_retained(parent, stage_name, stage)?;
        }
        Err(error) => return Err(CoordError::io(error)),
    }
    let mut final_file = open_file_at(parent, final_name, false, final_mode, None)?;
    exact(&mut final_file, bytes)
}

fn unlink_retained(parent: &File, name: &str, file: &File) -> Result<(), CoordError> {
    let expected = identity(file)?;
    let stat = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| os_error("cannot revalidate staged file", error))?;
    if expected != Identity(stat.st_dev, stat.st_ino) {
        return Err(changed("staged authority pathname was replaced"));
    }
    unlinkat(parent, name, AtFlags::empty())
        .map_err(|error| os_error("cannot retire staged authority file", error))?;
    parent.sync_all().map_err(CoordError::io)
}

fn validate_stage_file(file: &File) -> Result<(), CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata().map_err(CoordError::io)?;
        if !metadata.is_file() || metadata.uid() != owner() || metadata.nlink() != 1 {
            return Err(invalid("staged file owner, link, or type is invalid"));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    Err(platform())
}

fn mode_of(file: &File) -> Result<u32, CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(file.metadata().map_err(CoordError::io)?.mode() & 0o7777)
    }
    #[cfg(not(unix))]
    Err(platform())
}

fn fence_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_FENCE_UNKNOWN", reason)
}
