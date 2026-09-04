use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
};

use rustix::fs::{Mode, OFlags, ResolveFlags, fchmod, mkdirat, openat2};
use sha2::{Digest, Sha256};

use crate::coord::{CoordError, generation::manifest::ArtifactBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Identity {
    device: u64,
    inode: u64,
}

pub(super) fn identity(file: &File) -> Result<Identity, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    Ok(Identity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

pub(super) fn ensure_dir(parent: &File, name: &str, owner: u32) -> Result<File, CoordError> {
    match mkdirat(parent, name, Mode::RWXU) {
        Ok(()) => parent.sync_all().map_err(CoordError::io)?,
        Err(rustix::io::Errno::EXIST) => {}
        Err(error) => return Err(changed(format!("cannot create directory {name}: {error}"))),
    }
    open_dir(parent, name, owner)
}

pub(super) fn open_dir(parent: &File, name: &str, owner: u32) -> Result<File, CoordError> {
    let file = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain directory {name}: {error}")))?;
    validate_dir(&file, name, owner)?;
    Ok(file)
}

pub(super) fn optional_dir(
    parent: &File,
    name: &str,
    owner: u32,
) -> Result<Option<File>, CoordError> {
    match openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    ) {
        Ok(descriptor) => {
            let file = File::from(descriptor);
            validate_dir(&file, name, owner)?;
            Ok(Some(file))
        }
        Err(rustix::io::Errno::NOENT) => Ok(None),
        Err(error) => Err(changed(format!("cannot retain directory {name}: {error}"))),
    }
}

pub(super) fn open_file(
    parent: &File,
    name: &str,
    owner: u32,
    mode: u32,
    length: Option<u64>,
    write: bool,
) -> Result<File, CoordError> {
    let access = if write { OFlags::RDWR } else { OFlags::RDONLY };
    let file = openat2(
        parent,
        name,
        access | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain file {name}: {error}")))?;
    validate_file(&file, name, owner, mode, length)?;
    Ok(file)
}

pub(super) fn optional_file(
    parent: &File,
    name: &str,
    owner: u32,
    mode: u32,
) -> Result<Option<File>, CoordError> {
    match openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    ) {
        Ok(descriptor) => {
            let file = File::from(descriptor);
            validate_file(&file, name, owner, mode, None)?;
            Ok(Some(file))
        }
        Err(rustix::io::Errno::NOENT) => Ok(None),
        Err(error) => Err(changed(format!("cannot retain file {name}: {error}"))),
    }
}

pub(super) fn list_names(directory: &File) -> Result<BTreeSet<String>, CoordError> {
    let mut entries = rustix::fs::Dir::read_from(directory)
        .map_err(|error| changed(format!("cannot inventory directory: {error}")))?;
    let mut names = BTreeSet::new();
    while let Some(entry) = entries.read() {
        let entry = entry.map_err(|error| changed(format!("cannot read directory: {error}")))?;
        let bytes = entry.file_name().to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let name = std::str::from_utf8(bytes)
            .map_err(|_| changed("directory child is not normalized UTF-8"))?;
        if name.contains('/') || name.contains('\\') || name.is_empty() {
            return Err(changed("directory child name is not normalized"));
        }
        if !names.insert(name.to_owned()) {
            return Err(changed("directory inventory contains a duplicate child"));
        }
    }
    Ok(names)
}

pub(super) fn stable_read(file: &mut File, maximum: usize) -> Result<Vec<u8>, CoordError> {
    let before = file.metadata().map_err(CoordError::io)?;
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = file.metadata().map_err(CoordError::io)?;
    if bytes.len() > maximum
        || metadata_identity(&before) != metadata_identity(&after)
        || before.len() != bytes.len() as u64
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
    {
        return Err(changed("retained file changed during bounded read"));
    }
    Ok(bytes)
}

pub(super) fn write_or_verify(
    parent: &File,
    name: &str,
    bytes: &[u8],
    owner: u32,
    mode: u32,
) -> Result<(), CoordError> {
    let (mut file, observed_mode, created) = open_or_create(parent, name, owner)?;
    let observed = stable_read(&mut file, bytes.len())
        .map_err(|_| build_unknown(format!("partial {name} changed during read")))?;
    if observed_mode == mode && !created {
        return (observed == bytes)
            .then_some(())
            .ok_or_else(|| build_unknown(format!("sealed {name} differs from exact bytes")));
    }
    if observed_mode != 0o600 || !bytes.starts_with(&observed) {
        return Err(build_unknown(format!(
            "partial {name} mode or exact prefix differs"
        )));
    }
    file.seek(SeekFrom::End(0)).map_err(CoordError::io)?;
    file.write_all(&bytes[observed.len()..])
        .map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)?;
    fchmod(&file, Mode::from_bits_retain(mode))
        .map_err(|error| build_unknown(format!("cannot seal immutable {name}: {error}")))?;
    file.sync_all().map_err(CoordError::io)?;
    parent.sync_all().map_err(CoordError::io)?;
    let reopened = open_file(parent, name, owner, mode, Some(bytes.len() as u64), false)?;
    if identity(&file)? != identity(&reopened)? {
        return Err(changed(format!("immutable {name} identity changed")));
    }
    let mut reopened = reopened;
    if stable_read(&mut reopened, bytes.len())? != bytes {
        return Err(changed(format!("immutable {name} read-back differs")));
    }
    Ok(())
}

fn open_or_create(parent: &File, name: &str, owner: u32) -> Result<(File, u32, bool), CoordError> {
    let flags = OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    match openat2(parent, name, flags, Mode::RUSR | Mode::WUSR, resolve()) {
        Ok(descriptor) => Ok((File::from(descriptor), 0o600, true)),
        Err(rustix::io::Errno::EXIST) => {
            let probe = openat2(
                parent,
                name,
                OFlags::PATH | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
                resolve(),
            )
            .map(File::from)
            .map_err(|error| build_unknown(format!("cannot inspect partial {name}: {error}")))?;
            let metadata = probe.metadata().map_err(CoordError::io)?;
            let mode = metadata.mode() & 0o7777;
            if !metadata.is_file()
                || metadata.uid() != owner
                || metadata.nlink() != 1
                || (mode != 0o600 && mode != 0o400)
            {
                return Err(build_unknown(format!("partial {name} is not admitted")));
            }
            let write = mode == 0o600;
            Ok((
                open_file(parent, name, owner, mode, None, write)?,
                mode,
                false,
            ))
        }
        Err(error) => Err(build_unknown(format!(
            "cannot create partial {name}: {error}"
        ))),
    }
}

pub(super) fn copy_or_verify(
    parent: &File,
    binding: &ArtifactBinding,
    source: &mut File,
    exact_prefix: bool,
    owner: u32,
) -> Result<(), CoordError> {
    source.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let maximum = binding.byte_length.saturating_add(u64::from(!exact_prefix));
    let mut bytes = Vec::new();
    Read::by_ref(source)
        .take(maximum)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let records = bytes.iter().filter(|byte| **byte == b'\n').count() as u64;
    if bytes.len() as u64 != binding.byte_length
        || binding.record_count != Some(records)
        || binding.ends_with_lf != bytes.last().is_some_and(|byte| *byte == b'\n')
        || format!("sha256:{:x}", Sha256::digest(&bytes)) != binding.sha256.as_str()
    {
        return Err(changed("copied artifact differs from manifest binding"));
    }
    let name = artifact_name(binding)?;
    write_or_verify(parent, name, &bytes, owner, 0o400)
}

pub(super) fn artifact_name(binding: &ArtifactBinding) -> Result<&str, CoordError> {
    let name = binding
        .relative_path
        .as_str()
        .strip_prefix("archive/")
        .ok_or_else(|| changed("recovery artifact is outside archive"))?;
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(changed("recovery artifact is not a direct archive child"));
    }
    Ok(name)
}

pub(super) fn revalidate_child(
    parent: &File,
    name: &str,
    retained: &File,
    owner: u32,
    directory: bool,
) -> Result<(), CoordError> {
    let reopened = if directory {
        open_dir(parent, name, owner)?
    } else {
        open_file(parent, name, owner, 0o400, None, false)?
    };
    if identity(retained)? != identity(&reopened)? {
        return Err(changed(format!("retained child {name} changed identity")));
    }
    Ok(())
}

pub(super) fn require_same_device(left: &File, right: &File) -> Result<(), CoordError> {
    if left.metadata().map_err(CoordError::io)?.dev()
        != right.metadata().map_err(CoordError::io)?.dev()
    {
        return Err(changed("generation publication crosses filesystems"));
    }
    Ok(())
}

fn validate_dir(file: &File, name: &str, owner: u32) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir()
        || metadata.uid() != owner
        || metadata.mode() & 0o7777 != 0o700
        || metadata.nlink() < 2
    {
        return Err(changed(format!(
            "directory {name} is not exact owner mode-0700"
        )));
    }
    Ok(())
}

fn validate_file(
    file: &File,
    name: &str,
    owner: u32,
    mode: u32,
    length: Option<u64>,
) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.nlink() != 1
        || metadata.mode() & 0o7777 != mode
        || length.is_some_and(|expected| metadata.len() != expected)
    {
        return Err(changed(format!(
            "file {name} is not the exact admitted subject"
        )));
    }
    Ok(())
}

fn metadata_identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    (metadata.dev(), metadata.ino())
}

fn resolve() -> ResolveFlags {
    ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS
}

fn build_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_GENERATION_BUILD_OUTCOME_UNKNOWN", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_RECOVERY_SUBJECT_CHANGED", reason)
}
