use std::{
    fs::{File, Permissions},
    io::{Read, Write},
    path::{Component, Path},
};

use crate::coord::CoordError;

use super::{CURRENT_FILE, invalid, validate_relative_path};

#[cfg(target_os = "linux")]
pub(super) fn read_relative(
    root: &Path,
    relative: &str,
    maximum: u64,
    mode: u32,
) -> Result<Vec<u8>, CoordError> {
    read_optional_relative(root, relative, maximum, mode)?.ok_or_else(|| {
        invalid(format!(
            "required coordination subject {relative} is absent"
        ))
    })
}

#[cfg(target_os = "linux")]
pub(super) fn read_optional_relative(
    root: &Path,
    relative: &str,
    maximum: u64,
    mode: u32,
) -> Result<Option<Vec<u8>>, CoordError> {
    use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
    use rustix::io::Errno;

    if relative != CURRENT_FILE {
        validate_relative_path(relative)?;
    }
    let directory = open_directory(root)?;
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let resolve = ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS;
    let descriptor = match openat2(&directory, relative, flags, Mode::empty(), resolve) {
        Ok(descriptor) => descriptor,
        Err(Errno::NOENT) => return Ok(None),
        Err(error) => {
            return Err(invalid(format!(
                "cannot open {relative} without symlinks: {error}"
            )));
        }
    };
    let mut file = File::from(descriptor);
    let before = file.metadata().map_err(CoordError::io)?;
    validate_file(&before, maximum, mode, relative)?;
    let expected_identity = identity(&before);
    let mut bytes = Vec::with_capacity(before.len() as usize);
    Read::by_ref(&mut file)
        .take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = file.metadata().map_err(CoordError::io)?;
    let reopened = openat2(&directory, relative, flags, Mode::empty(), resolve)
        .map(File::from)
        .map_err(|error| invalid(format!("cannot reopen {relative}: {error}")))?;
    if bytes.len() as u64 != before.len()
        || identity(&after) != expected_identity
        || identity(&reopened.metadata().map_err(CoordError::io)?) != expected_identity
    {
        return Err(invalid(format!(
            "{relative} changed during its bounded read"
        )));
    }
    Ok(Some(bytes))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn read_relative(
    _root: &Path,
    _relative: &str,
    _maximum: u64,
    _mode: u32,
) -> Result<Vec<u8>, CoordError> {
    Err(unsupported())
}

#[cfg(not(target_os = "linux"))]
pub(super) fn read_optional_relative(
    _root: &Path,
    _relative: &str,
    _maximum: u64,
    _mode: u32,
) -> Result<Option<Vec<u8>>, CoordError> {
    Err(unsupported())
}

#[cfg(target_os = "linux")]
pub(super) fn write_immutable_relative(
    root: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<(), CoordError> {
    use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
    use std::os::unix::fs::PermissionsExt;

    validate_relative_path(relative)?;
    let directory = open_directory(root)?;
    let descriptor = openat2(
        &directory,
        relative,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| invalid(format!("cannot create {relative} exclusively: {error}")))?;
    let mut file = File::from(descriptor);
    file.write_all(bytes).map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)?;
    file.set_permissions(Permissions::from_mode(0o400))
        .map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)?;
    directory.sync_all().map_err(CoordError::io)
}

#[cfg(not(target_os = "linux"))]
pub(super) fn write_immutable_relative(
    _root: &Path,
    _relative: &str,
    _bytes: &[u8],
) -> Result<(), CoordError> {
    Err(unsupported())
}

#[cfg(target_os = "linux")]
fn open_directory(path: &Path) -> Result<File, CoordError> {
    use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
    use std::os::unix::fs::MetadataExt;

    if !path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid(
            "generation directory must be an absolute normalized path",
        ));
    }
    let descriptor = openat2(
        rustix::fs::CWD,
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| invalid(format!("cannot open generation directory safely: {error}")))?;
    let directory = File::from(descriptor);
    let metadata = directory.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != 0o700
    {
        return Err(invalid(
            "generation directory must be current-owner, mode 0700, and non-symlink",
        ));
    }
    Ok(directory)
}

#[cfg(target_os = "linux")]
fn validate_file(
    metadata: &std::fs::Metadata,
    maximum: u64,
    mode: u32,
    label: &str,
) -> Result<(), CoordError> {
    use std::os::unix::fs::MetadataExt;

    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != mode
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(invalid(format!(
            "{label} must be bounded, current-owner, mode {mode:04o}, regular, and single-link"
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn identity(metadata: &std::fs::Metadata) -> (u64, u64, u64, u64, i64, i64, i64, i64) {
    use std::os::unix::fs::MetadataExt;

    (
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.nlink(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

#[cfg(not(target_os = "linux"))]
fn unsupported() -> CoordError {
    CoordError::new(
        "COORD_GENERATION_PLATFORM_UNSUPPORTED",
        "descriptor-safe coordination generation admission is available only on Linux",
    )
}
