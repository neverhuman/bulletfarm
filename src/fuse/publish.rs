//! Contained publication of a complete ignored fusion tree.

use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::coord::CoordError;

const OUTPUT: &str = ".fusion";
const STAGING_PREFIX: &str = ".fusion.stage.";
const MARKER: &str = ".bullet-family-fusion-v1";
const MARKER_BYTES: &[u8] = b"bullet-family-fusion-v1\n";
const OWNED_FILES: &[&str] = &[MARKER, "dev.sh", "manifest.toml", "source"];
const MAX_OUTPUT_FILE_BYTES: u64 = 4 * 1024 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn publish(hub: &Path, files: &[(&str, &[u8], bool)]) -> Result<(), CoordError> {
    require_platform()?;
    validate_specification(files)?;
    let output = hub.join(OUTPUT);
    let existed = validate_existing(&output)?;
    let mut staging = Staging::create(hub)?;
    write_file(&staging.path.join(MARKER), MARKER_BYTES, false)?;
    for &(name, bytes, executable) in files {
        validate_name(name)?;
        write_file(&staging.path.join(name), bytes, executable)?;
    }
    sync_directory(&staging.path)?;
    if existed {
        exchange(&staging.path, &output)?;
        staging.owned = false;
        sync_directory(hub)?;
        validate_owned_tree(&staging.path)?;
        staging.owned = true;
        staging.remove()?;
    } else {
        publish_no_replace(&staging.path, &output)?;
        sync_directory(hub)?;
    }
    Ok(())
}

fn validate_existing(path: &Path) -> Result<bool, CoordError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            validate_owned_tree(path)?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(CoordError::io(error)),
    }
}

fn validate_owned_tree(path: &Path) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(conflict(
            ".fusion must be an ordinary non-symlink directory",
        ));
    }
    let expected = OWNED_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(path).map_err(CoordError::io)? {
        let entry = entry.map_err(CoordError::io)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| conflict(".fusion contains a non-UTF-8 entry; it was preserved"))?;
        if !expected.contains(&name) || !actual.insert(name.clone()) {
            return Err(conflict(format!(
                ".fusion contains unowned entry {name:?}; it was preserved"
            )));
        }
        let metadata = fs::symlink_metadata(entry.path()).map_err(CoordError::io)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() > MAX_OUTPUT_FILE_BYTES
        {
            return Err(conflict(format!(
                ".fusion entry {name:?} is unsafe; it was preserved"
            )));
        }
    }
    if actual != expected {
        return Err(conflict(
            ".fusion is incomplete or predates Rust-owned fusion; remove it manually after preserving any work",
        ));
    }
    if fs::read(path.join(MARKER)).map_err(CoordError::io)? != MARKER_BYTES {
        return Err(conflict(
            ".fusion ownership marker is invalid; it was preserved",
        ));
    }
    Ok(())
}

fn validate_specification(files: &[(&str, &[u8], bool)]) -> Result<(), CoordError> {
    let names = files
        .iter()
        .map(|(name, _, _)| *name)
        .collect::<BTreeSet<_>>();
    if names.len() != files.len()
        || names != ["dev.sh", "manifest.toml", "source"].into_iter().collect()
    {
        return Err(CoordError::new(
            "INVALID_FUSION_PATH",
            "fusion output specification must contain exactly dev.sh, manifest.toml, and source",
        ));
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), CoordError> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'.' | b'-' | b'_'))
        || name == MARKER
    {
        return Err(CoordError::new(
            "INVALID_FUSION_PATH",
            format!("invalid fixed fusion path {name:?}"),
        ));
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8], executable: bool) -> Result<(), CoordError> {
    if bytes.len() as u64 > MAX_OUTPUT_FILE_BYTES {
        return Err(CoordError::new(
            "FUSION_OUTPUT_TOO_LARGE",
            format!("{} exceeds 4 MiB", path.display()),
        ));
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(CoordError::io)?;
    file.write_all(bytes)
        .and_then(|()| set_mode(&file, executable))
        .and_then(|()| file.sync_all())
        .map_err(CoordError::io)
}

#[cfg(unix)]
fn set_mode(file: &fs::File, executable: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(if executable {
        0o755
    } else {
        0o644
    }))
}

#[cfg(not(unix))]
fn set_mode(_file: &fs::File, _executable: bool) -> std::io::Result<()> {
    Ok(())
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn publish_no_replace(staged: &Path, output: &Path) -> Result<(), CoordError> {
    use nix::{
        errno::Errno,
        fcntl::{RenameFlags, renameat2},
    };
    renameat2(None, staged, None, output, RenameFlags::RENAME_NOREPLACE).map_err(|error| {
        if error == Errno::EEXIST {
            conflict(".fusion appeared during publication and was preserved")
        } else {
            CoordError::new("FUSION_PUBLICATION_FAILED", error.to_string())
        }
    })
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn exchange(staged: &Path, output: &Path) -> Result<(), CoordError> {
    use nix::fcntl::{RenameFlags, renameat2};
    renameat2(None, staged, None, output, RenameFlags::RENAME_EXCHANGE)
        .map_err(|error| CoordError::new("FUSION_PUBLICATION_FAILED", error.to_string()))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn require_platform() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn require_platform() -> Result<(), CoordError> {
    Err(CoordError::new(
        "UNSUPPORTED_PLATFORM_CONTAINMENT",
        "atomic no-replace/exchange fusion publication is not verified on this platform",
    ))
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn publish_no_replace(_staged: &Path, _output: &Path) -> Result<(), CoordError> {
    require_platform()
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn exchange(_staged: &Path, _output: &Path) -> Result<(), CoordError> {
    require_platform()
}

fn sync_directory(path: &Path) -> Result<(), CoordError> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(CoordError::io)
}

fn conflict(reason: impl Into<String>) -> CoordError {
    CoordError::new("FUSION_DESTINATION_CONFLICT", reason)
}

struct Staging {
    path: PathBuf,
    owned: bool,
}

impl Staging {
    fn create(hub: &Path) -> Result<Self, CoordError> {
        for _ in 0..64 {
            let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = hub.join(format!(
                "{STAGING_PREFIX}{}.{}",
                std::process::id(),
                sequence
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path, owned: true }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(CoordError::io(error)),
            }
        }
        Err(CoordError::new(
            "FUSION_STAGING_COLLISION",
            "could not allocate a unique fusion staging directory",
        ))
    }

    fn remove(&mut self) -> Result<(), CoordError> {
        if self.owned && self.path.exists() {
            remove_owned_tree(&self.path)?;
            self.owned = false;
            if let Some(parent) = self.path.parent() {
                sync_directory(parent)?;
            }
        }
        Ok(())
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if self.owned && validate_existing_staging(&self.path) {
            let _ = remove_owned_tree(&self.path);
        }
    }
}

fn validate_existing_staging(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.file_type().is_dir()
            && !metadata.file_type().is_symlink()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(STAGING_PREFIX))
    })
}

fn remove_owned_tree(path: &Path) -> Result<(), CoordError> {
    if !validate_existing_staging(path) {
        return Err(conflict(
            "fusion staging path changed type during cleanup; it was preserved",
        ));
    }
    for name in OWNED_FILES {
        match fs::remove_file(path.join(name)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CoordError::io(error)),
        }
    }
    fs::remove_dir(path).map_err(CoordError::io)
}
