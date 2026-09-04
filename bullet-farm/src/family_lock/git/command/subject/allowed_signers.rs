//! Immutable allowed-signers input for SSH tag verification.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, RawFd};

#[cfg(target_os = "linux")]
use nix::fcntl::{OFlag, OpenHow, ResolveFlag, openat2};
#[cfg(target_os = "linux")]
use nix::sys::stat::{SFlag, fstat};
#[cfg(target_os = "linux")]
use nix::unistd::close;

#[cfg(target_os = "linux")]
use super::snapshot_open_subject;
#[cfg(not(target_os = "linux"))]
use super::snapshot_subject;
use super::{Identity, copy_fingerprint, identity};
use crate::coord::CoordError;

const MAX_BYTES: u64 = 64 * 1024;

#[derive(Debug)]
pub(in crate::family_lock::git::command) struct PinnedAllowedSigners {
    path: PathBuf,
    identity: Identity,
    fingerprint: [u8; 32],
    subject: File,
}

impl PinnedAllowedSigners {
    pub(in crate::family_lock::git::command) fn admit(path: &Path) -> Result<Self, CoordError> {
        if !path.is_absolute() {
            return Err(invalid("allowed-signers path must be absolute"));
        }
        let canonical = fs::canonicalize(path)
            .map_err(|error| invalid(format!("{} cannot be resolved: {error}", path.display())))?;
        if canonical != path {
            return Err(invalid(
                "allowed-signers path must be canonical and contain no symlink",
            ));
        }
        let (subject, fingerprint, identity) = snapshot(&canonical).map_err(|_| {
            invalid("allowed-signers must be a regular non-symlink file no larger than 64 KiB")
        })?;
        let pinned = Self {
            path: canonical,
            identity,
            fingerprint,
            subject,
        };
        pinned.verify()?;
        Ok(pinned)
    }

    pub(in crate::family_lock::git::command) fn verify(&self) -> Result<(), CoordError> {
        let mut source = open_source(&self.path).map_err(|_| changed())?;
        let identity = identity(&source).map_err(|_| changed())?;
        let fingerprint = copy_fingerprint("allowed signers", &mut source, None, MAX_BYTES)
            .map_err(|_| changed())?;
        if identity != self.identity || fingerprint != self.fingerprint {
            return Err(changed());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub(in crate::family_lock::git::command) fn subject_path(&self) -> PathBuf {
        descriptor_path(self.subject.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    pub(in crate::family_lock::git::command) fn subject_path(&self) -> PathBuf {
        unreachable!("allowed-signers admission fails without descriptor paths")
    }
}

#[cfg(target_os = "linux")]
fn snapshot(path: &Path) -> Result<(File, [u8; 32], Identity), CoordError> {
    snapshot_open_subject("allowed signers", open_source(path)?, MAX_BYTES, false)
}

#[cfg(not(target_os = "linux"))]
fn snapshot(path: &Path) -> Result<(File, [u8; 32], Identity), CoordError> {
    snapshot_subject("allowed signers", path, MAX_BYTES, false)
}

#[cfg(target_os = "linux")]
fn open_source(path: &Path) -> Result<File, CoordError> {
    let how = OpenHow::new()
        .flags(OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NONBLOCK)
        .resolve(ResolveFlag::RESOLVE_NO_SYMLINKS);
    let descriptor = openat2(nix::libc::AT_FDCWD, path, how).map_err(|error| {
        invalid(format!(
            "cannot open allowed-signers without symlinks: {error}"
        ))
    })?;
    let metadata = match fstat(descriptor) {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = close(descriptor);
            return Err(invalid(format!("cannot inspect allowed-signers: {error}")));
        }
    };
    if SFlag::from_bits_truncate(metadata.st_mode) != SFlag::S_IFREG
        || metadata.st_size < 0
        || metadata.st_size as u64 > MAX_BYTES
    {
        close(descriptor)
            .map_err(|error| invalid(format!("cannot close allowed-signers source: {error}")))?;
        return Err(invalid(
            "allowed-signers must be a regular file no larger than 64 KiB",
        ));
    }
    let source = File::open(descriptor_path(descriptor));
    close(descriptor)
        .map_err(|error| invalid(format!("cannot close allowed-signers source: {error}")))?;
    let source =
        source.map_err(|error| invalid(format!("cannot pin allowed-signers source: {error}")))?;
    Ok(source)
}

#[cfg(not(target_os = "linux"))]
fn open_source(_path: &Path) -> Result<File, CoordError> {
    Err(CoordError::new(
        "UNSUPPORTED_PLATFORM_CONTAINMENT",
        "descriptor-pinned allowed-signers admission is available only on Linux",
    ))
}

#[cfg(target_os = "linux")]
fn descriptor_path(descriptor: RawFd) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{descriptor}"))
}

fn invalid(detail: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_ALLOWED_SIGNERS", detail)
}

fn changed() -> CoordError {
    CoordError::new(
        "ALLOWED_SIGNERS_CHANGED",
        "allowed-signers path, identity, or bytes changed during signature verification",
    )
}
