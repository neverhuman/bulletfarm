use std::{
    fs::{self, File},
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Path, PathBuf},
};

use nix::{
    errno::Errno,
    fcntl::{RenameFlags, renameat2},
};
use rustix::fs::{Mode, OFlags, ResolveFlags, fchmod, mkdirat, openat2};

use super::{platform_fs as io, verifier};
use crate::coord::{CoordError, generation::manifest::GenerationManifest};

use super::authority::BaselineSubject;

#[path = "exchange/evidence.rs"]
mod evidence;

const LEGACY: &str = "events.jsonl";
const RETIRED: &str = "retired-v1.non-authoritative";
const SIBLING_PREFIX: &str = ".recovery-tombstone-";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LegacyLocation {
    Fresh,
    TransientSibling,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SiblingState {
    Absent,
    Unsealed,
    Sealed,
}

pub(super) struct PreparedTombstone {
    control: File,
    post_seal: Option<File>,
    pre_exchange: Option<File>,
    post_exchange: Option<File>,
    state: SiblingState,
}

impl PreparedTombstone {
    pub(super) fn retained(&self) -> &File {
        &self.control
    }
}

pub(super) fn sibling_name(generation_id: &str) -> Result<String, CoordError> {
    let Some(hex) = generation_id.strip_prefix("gen_") else {
        return Err(invalid("generation ID has no admitted prefix"));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("generation ID is not full lowercase hexadecimal"));
    }
    Ok(format!("{SIBLING_PREFIX}{generation_id}"))
}

pub(super) fn sibling_path(coord: &Path, generation_id: &str) -> Result<PathBuf, CoordError> {
    Ok(coord.join(sibling_name(generation_id)?))
}

pub(super) fn open_legacy(
    legacy: &Path,
    retired: &Path,
    sibling: &Path,
    owner: u32,
) -> Result<(File, LegacyLocation, SiblingState), CoordError> {
    let legacy_metadata = fs::symlink_metadata(legacy).map_err(CoordError::io)?;
    let kind = legacy_metadata.file_type();
    if kind.is_dir() && legacy_metadata.mode() & 0o7777 == 0 {
        return Err(reconciliation(
            "legacy mode-000 tombstone predates the reopenable recovery protocol",
        ));
    }
    if kind.is_file() {
        if optional_metadata(retired)?.is_some() {
            return Err(changed("retired source exists before legacy exchange"));
        }
        let sibling_state = match optional_metadata(sibling)? {
            None => SiblingState::Absent,
            Some(metadata)
                if metadata.is_dir()
                    && metadata.uid() == owner
                    && metadata.mode() & 0o7777 == 0o700 =>
            {
                let retained = io::open_directory(sibling, owner, 0o700)?;
                io::require_empty_descriptor(&retained)?;
                SiblingState::Unsealed
            }
            Some(metadata)
                if metadata.is_dir()
                    && metadata.uid() == owner
                    && metadata.mode() & 0o7777 == 0o400
                    && metadata.nlink() == 2 =>
            {
                let retained = io::open_directory(sibling, owner, 0o400)?;
                io::require_empty_descriptor(&retained)?;
                SiblingState::Sealed
            }
            Some(metadata) if metadata.is_dir() && metadata.mode() & 0o7777 == 0 => {
                return Err(reconciliation(
                    "mode-000 staged tombstone requires exact incident reconciliation",
                ));
            }
            Some(_) => return Err(changed("staged tombstone sibling is not admitted")),
        };
        return Ok((
            io::open_exact_file(legacy, owner, 0o400, false)?,
            LegacyLocation::Fresh,
            sibling_state,
        ));
    }
    if !kind.is_dir()
        || legacy_metadata.uid() != owner
        || legacy_metadata.mode() & 0o7777 != 0o400
        || legacy_metadata.nlink() != 2
    {
        return Err(invalid("legacy pathname is neither file nor tombstone"));
    }
    let tombstone = io::open_directory(legacy, owner, 0o400)?;
    io::require_empty_descriptor(&tombstone)?;
    match (optional_metadata(retired)?, optional_metadata(sibling)?) {
        (Some(_), None) => Ok((
            io::open_exact_file(retired, owner, 0o400, false)?,
            LegacyLocation::Retired,
            SiblingState::Absent,
        )),
        (None, Some(_)) => Ok((
            io::open_exact_file(sibling, owner, 0o400, false)?,
            LegacyLocation::TransientSibling,
            SiblingState::Absent,
        )),
        (Some(_), Some(_)) => Err(changed("duplicate retired and transient sources exist")),
        (None, None) => Err(changed("sealed tombstone has no exact retired source")),
    }
}

pub(super) fn revalidate_topology(
    legacy: &Path,
    retired: &Path,
    sibling: &Path,
    owner: u32,
    expected: (LegacyLocation, SiblingState, (u64, u64)),
) -> Result<(), CoordError> {
    let (source, location, sibling_state) = open_legacy(legacy, retired, sibling, owner)?;
    if (location, sibling_state, verifier::identity(&source)?) != expected {
        return Err(changed("legacy recovery topology changed during preflight"));
    }
    Ok(())
}

pub(super) fn revalidate_final_topology(
    root: &File,
    recovery: &File,
    sibling: &str,
    tombstone: &File,
    retired_source: &File,
    owner: u32,
) -> Result<(), CoordError> {
    let reopened_tombstone = open_tombstone_at(root, LEGACY, owner)?;
    let reopened_source = open_file_at(recovery, RETIRED, owner)?;
    if verifier::identity(&reopened_tombstone)? != verifier::identity(tombstone)?
        || verifier::identity(&reopened_source)? != verifier::identity(retired_source)?
        || child_present(root, sibling)?
    {
        return Err(CoordError::new(
            "COORD_RECOVERY_FINAL_REBIND_UNKNOWN",
            "final tombstone, retired source, or sibling topology changed",
        ));
    }
    Ok(())
}

pub(super) fn prepare(
    root: &File,
    name: &str,
    owner: u32,
    expected: SiblingState,
) -> Result<PreparedTombstone, CoordError> {
    if expected == SiblingState::Sealed {
        return Ok(PreparedTombstone {
            control: open_tombstone_at(root, name, owner)?,
            post_seal: None,
            pre_exchange: None,
            post_exchange: None,
            state: expected,
        });
    }
    match mkdirat(root, name, Mode::RWXU) {
        Ok(()) if expected == SiblingState::Absent => {}
        Err(rustix::io::Errno::EXIST) if expected == SiblingState::Unsealed => {}
        Ok(()) | Err(rustix::io::Errno::EXIST) => {
            return Err(changed(
                "tombstone sibling presence changed before preparation",
            ));
        }
        Err(error) => return Err(invalid(format!("cannot create tombstone sibling: {error}"))),
    }
    let control = io::open_child_directory(root, name, owner, 0o700)?;
    let post_seal = io::open_child_directory(root, name, owner, 0o700)?;
    let pre_exchange = io::open_child_directory(root, name, owner, 0o700)?;
    let post_exchange = io::open_child_directory(root, name, owner, 0o700)?;
    io::require_empty_descriptor(&control)?;
    let expected_identity = verifier::identity(&control)?;
    if verifier::identity(&post_seal)? != expected_identity
        || verifier::identity(&pre_exchange)? != expected_identity
        || verifier::identity(&post_exchange)? != expected_identity
    {
        return Err(invalid(
            "tombstone sibling identity changed during inventory",
        ));
    }
    root.sync_all().map_err(CoordError::io)?;
    Ok(PreparedTombstone {
        control,
        post_seal: Some(post_seal),
        pre_exchange: Some(pre_exchange),
        post_exchange: Some(post_exchange),
        state: expected,
    })
}

pub(super) fn seal(
    root: &File,
    name: &str,
    prepared: &PreparedTombstone,
    owner: u32,
) -> Result<(), CoordError> {
    if prepared.state == SiblingState::Sealed {
        let sealed = open_tombstone_at(root, name, owner)?;
        return (verifier::identity(&sealed)? == verifier::identity(&prepared.control)?)
            .then_some(())
            .ok_or_else(|| changed("adopted sealed sibling identity changed"));
    }
    let reopened = io::open_child_directory(root, name, owner, 0o700)?;
    io::require_empty_descriptor(&reopened)?;
    let expected = verifier::identity(&prepared.control)?;
    if verifier::identity(&reopened)? != expected {
        return Err(changed("tombstone sibling changed before sealing"));
    }
    fchmod(&prepared.control, Mode::RUSR)
        .map_err(|error| invalid(format!("cannot seal tombstone sibling: {error}")))?;
    prepared.control.sync_all().map_err(CoordError::io)?;
    root.sync_all().map_err(CoordError::io)?;
    io::require_empty_descriptor(
        prepared
            .post_seal
            .as_ref()
            .ok_or_else(|| changed("post-seal inventory descriptor is missing"))?,
    )?;
    let sealed = open_tombstone_at(root, name, owner)?;
    if verifier::identity(&sealed)? != expected {
        return Err(changed("sealed tombstone sibling identity changed"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_or_verify_prepared_observation(
    recovery: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    sibling: &str,
    prepared: &PreparedTombstone,
    owner: u32,
    allow_create: bool,
) -> Result<String, CoordError> {
    evidence::prepared(
        recovery,
        manifest,
        baseline,
        intent_sha256,
        sibling,
        &prepared.control,
        owner,
        allow_create,
    )
}

pub(super) fn verify_prepared_observation(
    recovery: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    sibling: &str,
    tombstone: &File,
    owner: u32,
) -> Result<String, CoordError> {
    evidence::prepared(
        recovery,
        manifest,
        baseline,
        intent_sha256,
        sibling,
        tombstone,
        owner,
        false,
    )
}

pub(super) fn exchange(
    root: &File,
    sibling: &str,
    prepared: &PreparedTombstone,
    legacy: &File,
    owner: u32,
) -> Result<(), CoordError> {
    let tombstone_identity = verifier::identity(&prepared.control)?;
    let legacy_identity = verifier::identity(legacy)?;
    let sealed = open_tombstone_at(root, sibling, owner)?;
    if verifier::identity(&sealed)? != tombstone_identity {
        return Err(changed("sealed sibling changed before exchange"));
    }
    if let Some(descriptor) = prepared.pre_exchange.as_ref() {
        io::require_empty_descriptor(descriptor)?;
    }
    renameat2(
        Some(root.as_raw_fd()),
        LEGACY,
        Some(root.as_raw_fd()),
        sibling,
        RenameFlags::RENAME_EXCHANGE,
    )
    .map_err(|error| CoordError::new("COORD_RECOVERY_EXCHANGE_FAILED", error.to_string()))?;
    root.sync_all().map_err(CoordError::io)?;
    let top = open_tombstone_at(root, LEGACY, owner)?;
    let transient = open_file_at(root, sibling, owner)?;
    if verifier::identity(&top)? != tombstone_identity
        || verifier::identity(&transient)? != legacy_identity
    {
        return Err(changed("same-parent exchange read-back identity differs"));
    }
    if let Some(descriptor) = prepared.post_exchange.as_ref() {
        io::require_empty_descriptor(descriptor)?;
    }
    Ok(())
}

pub(super) fn retire(
    root: &File,
    recovery: &File,
    sibling: &str,
    legacy: &File,
    owner: u32,
) -> Result<(), CoordError> {
    let expected = verifier::identity(legacy)?;
    let transient = open_file_at(root, sibling, owner)?;
    if verifier::identity(&transient)? != expected {
        return Err(changed("transient legacy sibling identity changed"));
    }
    renameat2(
        Some(root.as_raw_fd()),
        sibling,
        Some(recovery.as_raw_fd()),
        RETIRED,
        RenameFlags::RENAME_NOREPLACE,
    )
    .map_err(|error| match error {
        Errno::EEXIST => changed("retired destination already exists"),
        _ => CoordError::new("COORD_RECOVERY_RETIRE_FAILED", error.to_string()),
    })?;
    root.sync_all().map_err(CoordError::io)?;
    recovery.sync_all().map_err(CoordError::io)?;
    let retired = open_file_at(recovery, RETIRED, owner)?;
    if verifier::identity(&retired)? != expected || child_present(root, sibling)? {
        return Err(changed(
            "retired source read-back differs after publication",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_or_verify_retirement_observation(
    root: &File,
    recovery: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    prepared_observation_sha256: &str,
    tombstone_observation_sha256: &str,
    sibling: &str,
    tombstone: &File,
    retired_source: &File,
    owner: u32,
    allow_create: bool,
) -> Result<(), CoordError> {
    if !child_present(recovery, RETIRED)? || child_present(root, sibling)? {
        return Err(changed(
            "retired source or final sibling absence is not exact",
        ));
    }
    evidence::retirement(
        recovery,
        manifest,
        baseline,
        intent_sha256,
        prepared_observation_sha256,
        tombstone_observation_sha256,
        sibling,
        tombstone,
        retired_source,
        owner,
        allow_create,
    )
}

fn open_tombstone_at(parent: &File, name: &str, owner: u32) -> Result<File, CoordError> {
    let file = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain sealed tombstone: {error}")))?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir()
        || metadata.uid() != owner
        || metadata.mode() & 0o7777 != 0o400
        || metadata.nlink() != 2
    {
        return Err(changed(
            "sealed tombstone type, owner, mode, or links differ",
        ));
    }
    io::require_empty_descriptor(&file)?;
    Ok(file)
}

fn open_file_at(parent: &File, name: &str, owner: u32) -> Result<File, CoordError> {
    let file = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain legacy sibling: {error}")))?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.mode() & 0o7777 != 0o400
        || metadata.nlink() != 1
    {
        return Err(changed("legacy sibling type, owner, mode, or links differ"));
    }
    Ok(file)
}

fn child_present(parent: &File, name: &str) -> Result<bool, CoordError> {
    match openat2(
        parent,
        name,
        OFlags::PATH | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    ) {
        Ok(_) => Ok(true),
        Err(rustix::io::Errno::NOENT) => Ok(false),
        Err(error) => Err(changed(format!("cannot probe retired sibling: {error}"))),
    }
}

fn optional_metadata(path: &Path) -> Result<Option<fs::Metadata>, CoordError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(CoordError::io(error)),
    }
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_RECOVERY_SUBJECT_CHANGED", reason)
}

fn reconciliation(reason: impl Into<String>) -> CoordError {
    CoordError::new("TOMBSTONE_LEGACY_RECONCILIATION_UNKNOWN", reason)
}
