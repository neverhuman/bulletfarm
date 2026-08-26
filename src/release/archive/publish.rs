//! Descriptor-pinned, no-replace publication of a fully durable staging tree.

use std::{
    ffi::OsString,
    fs::File,
    path::{Path, PathBuf},
};

#[cfg(all(target_os = "linux", target_env = "gnu"))]
use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
#[cfg(all(target_os = "linux", target_env = "gnu"))]
use std::{fs, fs::OpenOptions, io};

use crate::coord::CoordError;

#[derive(Debug)]
pub(super) struct AdmittedDestination {
    parent_path: PathBuf,
    parent: File,
    name: OsString,
}

impl AdmittedDestination {
    pub(super) fn parent_path(&self) -> &Path {
        &self.parent_path
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) fn admit_destination(path: &Path) -> Result<AdmittedDestination, CoordError> {
    use nix::{errno::Errno, fcntl::AtFlags, sys::stat::fstatat};

    if !path.is_absolute() {
        return Err(invalid_destination(
            "release extraction destination must be absolute",
        ));
    }
    let name = path
        .file_name()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| invalid_destination("release extraction destination has no file name"))?
        .to_owned();
    let parent_path = path
        .parent()
        .ok_or_else(|| invalid_destination("release extraction destination has no parent"))?;
    let metadata = fs::symlink_metadata(parent_path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(invalid_destination(
            "release extraction parent must be a non-symlink directory",
        ));
    }
    let canonical = parent_path.canonicalize().map_err(CoordError::io)?;
    if canonical != parent_path {
        return Err(invalid_destination(
            "release extraction parent and every existing component must be canonical and non-symlinked",
        ));
    }
    let parent = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC)
        .open(&canonical)
        .map_err(CoordError::io)?;
    match fstatat(
        Some(parent.as_raw_fd()),
        name.as_os_str(),
        AtFlags::AT_SYMLINK_NOFOLLOW,
    ) {
        Err(Errno::ENOENT) => {}
        Ok(_) => {
            return Err(CoordError::new(
                "RELEASE_DESTINATION_EXISTS",
                "release extraction never replaces an existing destination",
            ));
        }
        Err(error) => {
            return Err(CoordError::new(
                "RELEASE_DESTINATION_ADMISSION_FAILED",
                format!("could not inspect release destination: {error}"),
            ));
        }
    }
    Ok(AdmittedDestination {
        parent_path: canonical,
        parent,
        name,
    })
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub(super) fn admit_destination(_path: &Path) -> Result<AdmittedDestination, CoordError> {
    Err(CoordError::new(
        "RELEASE_EXTRACTION_PLATFORM_UNSUPPORTED",
        "descriptor-relative no-replace publication is currently supported only on Linux GNU",
    ))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) fn publish_no_replace(
    source: &Path,
    destination: &AdmittedDestination,
) -> Result<(), CoordError> {
    publish_with_sync(source, destination, || destination.parent.sync_all())
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub(super) fn publish_no_replace(
    _source: &Path,
    _destination: &AdmittedDestination,
) -> Result<(), CoordError> {
    Err(CoordError::new(
        "RELEASE_EXTRACTION_PLATFORM_UNSUPPORTED",
        "descriptor-relative no-replace publication is currently supported only on Linux GNU",
    ))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn publish_with_sync(
    source: &Path,
    destination: &AdmittedDestination,
    sync_parent: impl FnOnce() -> io::Result<()>,
) -> Result<(), CoordError> {
    use nix::{
        errno::Errno,
        fcntl::{RenameFlags, renameat2},
    };

    let relative_source = source
        .strip_prefix(&destination.parent_path)
        .map_err(|_| invalid_destination("staging tree is not beneath the destination parent"))?;
    match renameat2(
        Some(destination.parent.as_raw_fd()),
        relative_source,
        Some(destination.parent.as_raw_fd()),
        destination.name.as_os_str(),
        RenameFlags::RENAME_NOREPLACE,
    ) {
        Ok(()) => {}
        Err(Errno::EEXIST) => {
            return Err(CoordError::new(
                "RELEASE_DESTINATION_EXISTS",
                "release extraction lost the no-replace publication race",
            ));
        }
        Err(Errno::ENOSYS | Errno::EINVAL | Errno::EOPNOTSUPP) => {
            return Err(CoordError::new(
                "RELEASE_PUBLICATION_UNSUPPORTED",
                "the destination filesystem cannot guarantee no-replace publication",
            ));
        }
        Err(error) => {
            return Err(CoordError::new(
                "RELEASE_PUBLICATION_FAILED",
                format!("no-replace publication failed before success: {error}"),
            ));
        }
    }
    sync_parent().map_err(|error| {
        CoordError::new(
            "RELEASE_PUBLICATION_UNKNOWN",
            format!("archive was atomically published but parent durability is unknown: {error}"),
        )
    })
}

fn invalid_destination(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_DESTINATION", reason)
}

#[cfg(all(test, target_os = "linux", target_env = "gnu"))]
pub(super) fn publish_with_failed_sync_for_test(
    source: &Path,
    destination: &AdmittedDestination,
) -> Result<(), CoordError> {
    publish_with_sync(source, destination, || {
        Err(io::Error::other("injected parent sync failure"))
    })
}
