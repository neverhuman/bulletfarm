use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::coord::CoordError;
use crate::coord::generation::manifest::{CurrentPointer, GenerationId, GenerationManifest};
use fs2::FileExt;

mod fence;
mod io;
mod publish;
use io::*;
pub(super) use publish::{
    GenesisIntentCandidate, create_generation, genesis_intent_candidate, publish_current,
    publish_generation, publish_genesis_intent, published_genesis_intent,
};

pub(super) fn ensure_tombstone(
    lock: &CoordLock,
    generation_id: &str,
    intent: &[u8],
) -> Result<(), CoordError> {
    fence::ensure(lock, generation_id, intent)
}

pub(super) fn preflight_genesis_fence(
    lock: &CoordLock,
    generation_id: &str,
    intent: &[u8],
) -> Result<(), CoordError> {
    fence::preflight(lock, generation_id, intent)
}

pub(super) fn validate_genesis_tombstone(
    lock: &CoordLock,
    generation_id: &str,
    intent: &[u8],
) -> Result<(), CoordError> {
    fence::validate(lock, generation_id, intent)
}

#[cfg(test)]
pub(super) fn test_inventory_retained_directory(directory: &File) -> Result<(), CoordError> {
    inventory_empty_dir(directory)
}

#[cfg(test)]
pub(super) fn test_crash_genesis_fence_after(phase: &'static str) {
    fence::test_crash_after(phase);
}

#[cfg(test)]
pub(super) fn test_kill_genesis_fence_after(phase: &'static str) {
    fence::test_kill_after(phase);
}

#[cfg(test)]
pub(super) fn test_insert_genesis_fence_after(phase: &'static str) {
    fence::test_insert_after(phase);
}

#[cfg(test)]
pub(super) fn test_kill_publish_after_link(name: &'static str) {
    publish::test_kill_after_link(name);
}

const DIR_MODE: u32 = 0o700;
const LOCK_MODE: u32 = 0o600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Presence {
    Absent,
    Legacy,
    Retired,
    Current,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LegacyKind {
    Absent,
    Source,
    Tombstone,
}

pub(super) struct CoordLock {
    path: PathBuf,
    directory: File,
    lock: File,
    lock_identity: Identity,
    exclusive: bool,
    current: Option<File>,
    current_identity: Option<Identity>,
}

pub(super) struct Probe {
    presence: Presence,
    directory: Option<File>,
    current: Option<File>,
}

pub(super) struct GenerationFiles {
    generation: File,
    manifest: File,
    pub(super) segment: File,
    pub(super) pending: File,
    generation_id: String,
    path_name: String,
    identities: [Identity; 5],
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Identity(u64, u64);

impl CoordLock {
    pub(super) fn acquire(path: &Path, exclusive: bool) -> Result<Self, CoordError> {
        linux_only()?;
        let directory = open_dir_path(path, DIR_MODE)?;
        let current = open_optional_file(&directory, "CURRENT", 0o400)?;
        Self::from_parts(path, directory, current, exclusive)
    }

    fn from_parts(
        path: &Path,
        directory: File,
        current: Option<File>,
        exclusive: bool,
    ) -> Result<Self, CoordError> {
        let lock = open_file_at(&directory, "LOCK", exclusive, LOCK_MODE, Some(0))?;
        if exclusive {
            lock.lock_exclusive()
        } else {
            FileExt::lock_shared(&lock)
        }
        .map_err(CoordError::io)?;
        let guard = Self {
            path: path.to_owned(),
            directory,
            lock_identity: identity(&lock)?,
            lock,
            exclusive,
            current_identity: current.as_ref().map(identity).transpose()?,
            current,
        };
        guard.revalidate()?;
        Ok(guard)
    }

    pub(super) fn revalidate(&self) -> Result<(), CoordError> {
        validate_file(&self.lock, LOCK_MODE, Some(0))?;
        let directory = open_dir_path(&self.path, DIR_MODE)?;
        let reopened = open_file_at(&directory, "LOCK", self.exclusive, LOCK_MODE, Some(0))?;
        if identity(&directory)? != identity(&self.directory)?
            || identity(&self.lock)? != self.lock_identity
            || identity(&reopened)? != self.lock_identity
        {
            return Err(changed("coordination root or stable LOCK was replaced"));
        }
        if let Some(expected) = self.current_identity {
            let reopened = open_file_at(&directory, "CURRENT", false, 0o400, None)?;
            if identity(&reopened)? != expected {
                return Err(changed("CURRENT changed across stable LOCK acquisition"));
            }
        }
        Ok(())
    }

    pub(super) const fn root(&self) -> &File {
        &self.directory
    }

    pub(super) fn generation(
        &self,
        generation_id: &str,
        writable: bool,
    ) -> Result<GenerationFiles, CoordError> {
        self.generation_named(generation_id, generation_id, writable)
    }

    fn generation_named(
        &self,
        path_name: &str,
        generation_id: &str,
        writable: bool,
    ) -> Result<GenerationFiles, CoordError> {
        validate_name(path_name)?;
        validate_name(generation_id)?;
        let generations = open_dir_at(&self.directory, "generations", DIR_MODE)?;
        let generation = open_dir_at(&generations, path_name, DIR_MODE)?;
        let manifest = open_file_at(&generation, "manifest.json", false, 0o400, None)?;
        let segment = open_file_at(&generation, "events.jsonl", writable, 0o600, None)?;
        let pending = open_dir_at(&generation, "pending", DIR_MODE)?;
        let identities = [
            identity(&generations)?,
            identity(&generation)?,
            identity(&manifest)?,
            identity(&segment)?,
            identity(&pending)?,
        ];
        Ok(GenerationFiles {
            generation,
            manifest,
            segment,
            pending,
            generation_id: generation_id.to_owned(),
            path_name: path_name.to_owned(),
            identities,
        })
    }

    pub(super) fn current(&self) -> Result<Option<CurrentPointer>, CoordError> {
        let mut file = if let Some(retained) = &self.current {
            retained.try_clone().map_err(CoordError::io)?
        } else if let Some(file) = open_optional_file(&self.directory, "CURRENT", 0o400)? {
            file
        } else {
            return Ok(None);
        };
        CurrentPointer::decode_canonical(&read_canonical(&mut file)?).map(Some)
    }

    pub(super) fn presence_without_current(&self) -> Result<Presence, CoordError> {
        Ok(match legacy_kind(&self.directory)? {
            LegacyKind::Absent => Presence::Absent,
            LegacyKind::Source => Presence::Legacy,
            LegacyKind::Tombstone => Presence::Retired,
        })
    }
}

impl Probe {
    pub(super) const fn presence(&self) -> Presence {
        self.presence
    }

    pub(super) fn into_lock(self, path: &Path, exclusive: bool) -> Result<CoordLock, CoordError> {
        let directory = self
            .directory
            .ok_or_else(|| missing("coordination root is absent"))?;
        CoordLock::from_parts(path, directory, self.current, exclusive)
    }
}

impl GenerationFiles {
    pub(super) fn revalidate(&self, lock: &CoordLock, writable: bool) -> Result<(), CoordError> {
        lock.revalidate()?;
        if lock
            .generation_named(&self.path_name, &self.generation_id, writable)?
            .identities
            != self.identities
        {
            return Err(changed("generation descriptor hierarchy was replaced"));
        }
        Ok(())
    }

    pub(super) fn artifact(&self, relative: &str, length: u64) -> Result<File, CoordError> {
        open_file_at(&self.generation, relative, false, 0o400, Some(length))
    }

    pub(super) fn revalidate_artifact(
        &self,
        relative: &str,
        retained: &File,
        length: u64,
    ) -> Result<(), CoordError> {
        let reopened = self.artifact(relative, length)?;
        if identity(&reopened)? != identity(retained)? {
            return Err(changed("recovery artifact pathname was replaced"));
        }
        Ok(())
    }

    pub(super) fn load_manifest(
        &mut self,
        expected: &GenerationId,
    ) -> Result<GenerationManifest, CoordError> {
        let manifest = GenerationManifest::decode_canonical(&read_canonical(&mut self.manifest)?)?;
        if manifest.generation_id() != expected {
            return Err(changed("manifest GenerationId differs from CURRENT"));
        }
        Ok(manifest)
    }
}

pub(super) fn probe(coord: &Path) -> Result<Probe, CoordError> {
    linux_only()?;
    let observed_mode = match fs::symlink_metadata(coord) {
        Ok(_) => current_mode(coord)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Probe {
                presence: Presence::Absent,
                directory: None,
                current: None,
            });
        }
        Err(error) => return Err(CoordError::io(error)),
    };
    if observed_mode != DIR_MODE && observed_mode != 0o775 {
        return Err(invalid(
            "coordination root mode is neither current nor admitted legacy",
        ));
    }
    let Some(directory) = open_dir_path_optional(coord, observed_mode)? else {
        return Ok(Probe {
            presence: Presence::Absent,
            directory: None,
            current: None,
        });
    };
    if child_exists(&directory, "CURRENT")? {
        if observed_mode != DIR_MODE || legacy_kind(&directory)? != LegacyKind::Tombstone {
            return Err(invalid(
                "published CURRENT requires private root and permanent tombstone",
            ));
        }
        let current = open_file_at(&directory, "CURRENT", false, 0o400, None)?;
        return Ok(Probe {
            presence: Presence::Current,
            directory: Some(directory),
            current: Some(current),
        });
    }
    let presence = match legacy_kind(&directory)? {
        LegacyKind::Absent => Presence::Absent,
        LegacyKind::Source => Presence::Legacy,
        LegacyKind::Tombstone => Presence::Retired,
    };
    Ok(Probe {
        presence,
        directory: Some(directory),
        current: None,
    })
}

pub(super) fn ensure_layout(root: &Path, coord: &Path) -> Result<(), CoordError> {
    linux_only()?;
    if !normalized(root) || coord != root.join(".bullet-family/coord") {
        return Err(invalid("coord root is not the normalized family child"));
    }
    let root_fd = open_dir_path(root, current_mode(root)?)?;
    let private = ensure_dir_at(&root_fd, ".bullet-family", DIR_MODE)?;
    let coord_fd = ensure_dir_at(&private, "coord", DIR_MODE)?;
    ensure_empty_at(&coord_fd, "LOCK", LOCK_MODE)
}

fn ensure_dir_at(parent: &File, name: &str, expected_mode: u32) -> Result<File, CoordError> {
    validate_name(name)?;
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{Mode, mkdirat};
        match mkdirat(parent, name, Mode::RWXU) {
            Ok(()) => parent.sync_all().map_err(CoordError::io)?,
            Err(rustix::io::Errno::EXIST) => {}
            Err(error) => return Err(os_error("cannot create directory", error)),
        }
    }
    open_dir_at(parent, name, expected_mode)
}

fn ensure_empty_at(parent: &File, name: &str, file_mode: u32) -> Result<(), CoordError> {
    match open_file_at(parent, name, true, file_mode, Some(0)) {
        Ok(_) => Ok(()),
        Err(error) if error.code() == "COORD_SUBJECT_MISSING" => {
            write_new_at(parent, name, &[], file_mode)
        }
        Err(error) => Err(error),
    }
}

fn write_new_at(parent: &File, name: &str, bytes: &[u8], file_mode: u32) -> Result<(), CoordError> {
    validate_name(name)?;
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{Mode, OFlags, fchmod, openat};
        let descriptor = openat(
            parent,
            name,
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|error| os_error("cannot create coordination file", error))?;
        fchmod(&descriptor, Mode::from_bits_retain(file_mode))
            .map_err(|error| os_error("cannot seal coordination file", error))?;
        let mut file = File::from(descriptor);
        file.write_all(bytes).map_err(CoordError::io)?;
        exact(&mut file, bytes)?;
        file.sync_all().map_err(CoordError::io)?;
        exact(&mut file, bytes)?;
        validate_file(&file, file_mode, Some(bytes.len() as u64))?;
        parent.sync_all().map_err(CoordError::io)
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

fn require_empty_or(directory: &File, expected: &str) -> Result<(), CoordError> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::Dir;
        let mut dir =
            Dir::read_from(directory).map_err(|error| os_error("inventory failed", error))?;
        while let Some(entry) = dir.read() {
            let entry = entry.map_err(|error| os_error("inventory failed", error))?;
            let name = entry.file_name().to_bytes();
            if name != b"." && name != b".." && name != expected.as_bytes() {
                return Err(changed("generations contains another authority subject"));
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

fn legacy_kind(directory: &File) -> Result<LegacyKind, CoordError> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{AtFlags, FileType, statat};
        let stat = match statat(directory, "events.jsonl", AtFlags::SYMLINK_NOFOLLOW) {
            Ok(value) => value,
            Err(rustix::io::Errno::NOENT) => return Ok(LegacyKind::Absent),
            Err(error) => return Err(os_error("cannot inspect legacy authority", error)),
        };
        let kind = FileType::from_raw_mode(stat.st_mode);
        let mode = stat.st_mode & 0o7777;
        let owned = stat.st_uid == owner();
        if kind.is_file() && owned && stat.st_nlink == 1 && (mode == 0o400 || mode == 0o600) {
            Ok(LegacyKind::Source)
        } else if kind.is_dir() && owned && stat.st_nlink >= 2 && matches!(mode, 0o000 | 0o400) {
            Ok(LegacyKind::Tombstone)
        } else {
            Err(CoordError::new(
                "COORD_FENCE_UNKNOWN",
                "legacy authority is neither an admitted source nor an exact sealed tombstone",
            ))
        }
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

fn rename_noreplace(parent: &File, old: &str, new: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        rustix::fs::renameat_with(parent, old, parent, new, rustix::fs::RenameFlags::NOREPLACE)
            .map_err(|error| std::io::Error::from_raw_os_error(error.raw_os_error()))
    }
    #[cfg(not(target_os = "linux"))]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Linux only",
    ))
}

fn os_error(context: &str, error: rustix::io::Errno) -> CoordError {
    invalid(format!("{context}: {error}"))
}
fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_STORAGE", reason)
}
fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}
fn missing(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_MISSING", reason)
}
fn platform() -> CoordError {
    CoordError::new(
        "COORD_PLATFORM_UNSUPPORTED",
        "generation ledger requires Linux",
    )
}
