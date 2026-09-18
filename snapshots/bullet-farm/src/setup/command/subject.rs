//! Immutable executable subjects for admitted setup tools.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::os::{
    fd::{AsRawFd, RawFd},
    unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
};
#[cfg(target_os = "linux")]
use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[cfg(target_os = "linux")]
use super::MAX_TOOL_BYTES;
use super::tool_error;
use crate::coord::CoordError;

#[derive(Debug)]
pub(super) struct AdmittedFile {
    pub(super) path: PathBuf,
    label: &'static str,
    fingerprint: [u8; 32],
    size_bytes: u64,
    executable: bool,
    subject: File,
    source_identity: SourceIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceIdentity {
    device: u64,
    inode: u64,
}

impl AdmittedFile {
    pub(super) fn admit(
        label: &'static str,
        path: &Path,
        executable: bool,
    ) -> Result<Self, CoordError> {
        if !path.is_absolute() {
            return Err(tool_error(
                "SETUP_TOOL_PATH_NOT_ABSOLUTE",
                label,
                "path must be absolute",
            ));
        }
        let canonical = fs::canonicalize(path).map_err(|error| {
            tool_error(
                "SETUP_TOOL_UNAVAILABLE",
                label,
                format!("{} cannot be resolved: {error}", path.display()),
            )
        })?;
        if canonical != path {
            return Err(tool_error(
                "SETUP_TOOL_PATH_NOT_CANONICAL",
                label,
                format!("use the canonical path {}", canonical.display()),
            ));
        }
        let (subject, fingerprint, size_bytes, source_identity) =
            snapshot_source(label, &canonical, executable)?;
        Ok(Self {
            path: canonical,
            label,
            fingerprint,
            size_bytes,
            executable,
            subject,
            source_identity,
        })
    }

    pub(super) fn verify(&self) -> Result<(), CoordError> {
        let (fingerprint, size_bytes) =
            source_fingerprint(self.label, &self.path, self.executable)?;
        if fingerprint != self.fingerprint || size_bytes != self.size_bytes {
            return Err(tool_error(
                "SETUP_TOOL_CHANGED",
                self.label,
                "file bytes changed after admission",
            ));
        }
        Ok(())
    }

    pub(super) fn canonical_path(&self) -> &Path {
        &self.path
    }

    pub(super) fn digest(&self) -> String {
        format!(
            "blake3:{}",
            blake3::Hash::from_bytes(self.fingerprint).to_hex()
        )
    }

    pub(super) const fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub(super) fn aliases(&self, other: &Self) -> bool {
        self.path == other.path || self.source_identity == other.source_identity
    }

    #[cfg(target_os = "linux")]
    pub(super) fn execution_path(&self) -> PathBuf {
        descriptor_path(self.subject.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn execution_path(&self) -> PathBuf {
        unreachable!("tool admission refuses platforms without sealed descriptor execution")
    }
}

#[cfg(target_os = "linux")]
fn snapshot_source(
    label: &'static str,
    path: &Path,
    executable: bool,
) -> Result<(File, [u8; 32], u64, SourceIdentity), CoordError> {
    use nix::{
        fcntl::{FcntlArg, FdFlag, SealFlag, fcntl},
        sys::{
            memfd::{MemFdCreateFlag, memfd_create},
            stat::{Mode, fchmod},
        },
    };

    let mut source = open_source(label, path, executable)?;
    let metadata = source
        .metadata()
        .map_err(|error| unavailable(label, path, error))?;
    let source_identity = SourceIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    let descriptor = memfd_create(c"bullet-setup-tool", MemFdCreateFlag::MFD_ALLOW_SEALING)
        .map_err(|error| pin_error(label, error))?;
    let mut writable = File::from(descriptor);
    let (fingerprint, size_bytes) = copy_fingerprint(label, &mut source, Some(&mut writable))?;
    writable.flush().map_err(|error| pin_error(label, error))?;
    fchmod(
        writable.as_raw_fd(),
        if executable {
            Mode::S_IRUSR | Mode::S_IXUSR
        } else {
            Mode::S_IRUSR
        },
    )
    .map_err(|error| pin_error(label, error))?;
    fcntl(
        writable.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|error| pin_error(label, error))?;

    // A writable descriptor makes Linux reject exec with ETXTBSY. Reopen the
    // sealed inode read-only and deliberately inherit it so scripts and the npm
    // companion can still resolve /proc/self/fd/N after the interpreter exec.
    let subject = File::open(descriptor_path(writable.as_raw_fd()))
        .map_err(|error| pin_error(label, error))?;
    fcntl(subject.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty()))
        .map_err(|error| pin_error(label, error))?;
    drop(writable);
    Ok((subject, fingerprint, size_bytes, source_identity))
}

#[cfg(not(target_os = "linux"))]
fn snapshot_source(
    label: &'static str,
    _path: &Path,
    _executable: bool,
) -> Result<(File, [u8; 32], u64, SourceIdentity), CoordError> {
    Err(tool_error(
        "SETUP_TOOL_PIN_UNSUPPORTED",
        label,
        "sealed descriptor execution is currently supported only on Linux",
    ))
}

#[cfg(target_os = "linux")]
fn source_fingerprint(
    label: &'static str,
    path: &Path,
    executable: bool,
) -> Result<([u8; 32], u64), CoordError> {
    let mut source = open_source(label, path, executable)?;
    copy_fingerprint(label, &mut source, None)
}

#[cfg(not(target_os = "linux"))]
fn source_fingerprint(
    label: &'static str,
    _path: &Path,
    _executable: bool,
) -> Result<([u8; 32], u64), CoordError> {
    Err(tool_error(
        "SETUP_TOOL_PIN_UNSUPPORTED",
        label,
        "sealed descriptor execution is currently supported only on Linux",
    ))
}

#[cfg(target_os = "linux")]
fn open_source(label: &'static str, path: &Path, executable: bool) -> Result<File, CoordError> {
    let source = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| unavailable(label, path, error))?;
    let metadata = source
        .metadata()
        .map_err(|error| unavailable(label, path, error))?;
    if !metadata.file_type().is_file() {
        return Err(tool_error(
            "SETUP_TOOL_NOT_REGULAR",
            label,
            "path must be a non-symlink regular file",
        ));
    }
    if metadata.len() > MAX_TOOL_BYTES {
        return Err(tool_error(
            "SETUP_TOOL_TOO_LARGE",
            label,
            "file exceeds the 512 MiB admission limit",
        ));
    }
    if executable && metadata.permissions().mode() & 0o111 == 0 {
        return Err(tool_error(
            "SETUP_TOOL_NOT_EXECUTABLE",
            label,
            "file has no executable mode bit",
        ));
    }
    Ok(source)
}

#[cfg(target_os = "linux")]
fn copy_fingerprint(
    label: &'static str,
    source: &mut File,
    mut destination: Option<&mut File>,
) -> Result<([u8; 32], u64), CoordError> {
    let mut hasher = blake3::Hasher::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|error| tool_error("SETUP_TOOL_UNAVAILABLE", label, error.to_string()))?;
        if count == 0 {
            return Ok((*hasher.finalize().as_bytes(), copied));
        }
        copied = copied.checked_add(count as u64).ok_or_else(|| {
            tool_error(
                "SETUP_TOOL_TOO_LARGE",
                label,
                "file size overflowed the admission limit",
            )
        })?;
        if copied > MAX_TOOL_BYTES {
            return Err(tool_error(
                "SETUP_TOOL_TOO_LARGE",
                label,
                "file exceeds the 512 MiB admission limit",
            ));
        }
        hasher.update(&buffer[..count]);
        if let Some(destination) = destination.as_mut() {
            destination
                .write_all(&buffer[..count])
                .map_err(|error| pin_error(label, error))?;
        }
    }
}

#[cfg(target_os = "linux")]
fn descriptor_path(descriptor: RawFd) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{descriptor}"))
}

#[cfg(target_os = "linux")]
fn unavailable(label: &'static str, path: &Path, error: impl std::fmt::Display) -> CoordError {
    tool_error(
        "SETUP_TOOL_UNAVAILABLE",
        label,
        format!("{} cannot be opened: {error}", path.display()),
    )
}

#[cfg(target_os = "linux")]
fn pin_error(label: &'static str, error: impl std::fmt::Display) -> CoordError {
    tool_error(
        "SETUP_TOOL_PIN_FAILED",
        label,
        format!("could not create immutable executable subject: {error}"),
    )
}
