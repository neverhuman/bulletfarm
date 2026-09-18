//! Immutable executable and ordinary-repository subjects for Git verification.

mod allowed_signers;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::{
    io::{Read, Write},
    os::{
        fd::{AsRawFd, RawFd},
        unix::fs::{MetadataExt, PermissionsExt},
    },
};

#[cfg(target_os = "linux")]
use rustix::{
    fs::{Mode, OFlags, openat},
    io::Errno,
};

use crate::coord::CoordError;

pub(super) use self::allowed_signers::PinnedAllowedSigners;

const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_LOCAL_CONFIG_BYTES: u64 = 64 * 1024;

#[derive(Debug)]
pub(super) struct PinnedExecutable {
    path: PathBuf,
    label: &'static str,
    fingerprint: [u8; 32],
    subject: File,
}

impl PinnedExecutable {
    pub(super) fn admit(label: &'static str, path: &Path) -> Result<Self, CoordError> {
        if !path.is_absolute() {
            return Err(tool_error(label, "path must be absolute"));
        }
        let canonical = fs::canonicalize(path).map_err(|error| {
            tool_error(
                label,
                format!("{} cannot be resolved: {error}", path.display()),
            )
        })?;
        if canonical != path {
            return Err(tool_error(
                label,
                format!("use the canonical path {}", canonical.display()),
            ));
        }
        let (subject, fingerprint) = snapshot_executable(label, &canonical)?;
        Ok(Self {
            path: canonical,
            label,
            fingerprint,
            subject,
        })
    }

    pub(super) fn verify(&self) -> Result<(), CoordError> {
        let actual = fingerprint_path(self.label, &self.path, MAX_EXECUTABLE_BYTES, true)?;
        if actual != self.fingerprint {
            return Err(CoordError::new(
                "GIT_TOOL_CHANGED",
                format!("{} bytes changed after admission", self.label),
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub(super) fn execution_path(&self) -> PathBuf {
        descriptor_path(self.subject.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn execution_path(&self) -> PathBuf {
        unreachable!("executable admission fails without sealed descriptor execution")
    }
}

#[derive(Debug)]
pub(super) struct PinnedRepository {
    path: PathBuf,
    work_tree: File,
    git_dir: File,
    work_tree_identity: Identity,
    git_dir_identity: Identity,
    config_identity: Identity,
    config_fingerprint: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
}

impl PinnedRepository {
    #[cfg(target_os = "linux")]
    pub(super) fn admit(path: &Path) -> Result<Self, CoordError> {
        let work_tree = open_directory(rustix::fs::CWD, path, "repository work tree")?;
        let work_tree_identity = identity(&work_tree)?;
        let canonical = fs::canonicalize(path).map_err(CoordError::io)?;
        let canonical_subject =
            open_directory(rustix::fs::CWD, &canonical, "repository work tree")?;
        if identity(&canonical_subject)? != work_tree_identity {
            return Err(repository_changed(
                "repository path changed while its subject was admitted",
            ));
        }
        let git_dir = open_directory(
            &work_tree,
            Path::new(".git"),
            "Git administrative directory",
        )?;
        let git_dir_identity = identity(&git_dir)?;
        let (config_identity, config_fingerprint) = read_config_subject(&git_dir)?;
        make_inheritable(&work_tree, "repository work tree")?;
        make_inheritable(&git_dir, "Git administrative directory")?;
        Ok(Self {
            path: canonical,
            work_tree,
            git_dir,
            work_tree_identity,
            git_dir_identity,
            config_identity,
            config_fingerprint,
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn admit(_path: &Path) -> Result<Self, CoordError> {
        Err(unsupported())
    }

    #[cfg(target_os = "linux")]
    pub(super) fn verify(&self) -> Result<(), CoordError> {
        let work_tree = open_directory(rustix::fs::CWD, &self.path, "repository work tree")?;
        if identity(&work_tree)? != self.work_tree_identity {
            return Err(repository_changed(
                "repository pathname no longer names the admitted work tree",
            ));
        }
        let git_dir = open_directory(
            &work_tree,
            Path::new(".git"),
            "Git administrative directory",
        )?;
        if identity(&git_dir)? != self.git_dir_identity {
            return Err(repository_changed(
                "repository .git no longer names the admitted administrative directory",
            ));
        }
        let (config_identity, config_fingerprint) = read_config_subject(&self.git_dir)?;
        if config_identity != self.config_identity || config_fingerprint != self.config_fingerprint
        {
            return Err(repository_changed(
                "repository-local Git config changed during verification",
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn verify(&self) -> Result<(), CoordError> {
        Err(unsupported())
    }

    #[cfg(target_os = "linux")]
    pub(super) fn work_tree_path(&self) -> PathBuf {
        descriptor_path(self.work_tree.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn work_tree_path(&self) -> PathBuf {
        unreachable!("repository admission fails without descriptor paths")
    }

    #[cfg(target_os = "linux")]
    pub(super) fn git_dir_path(&self) -> PathBuf {
        descriptor_path(self.git_dir.as_raw_fd())
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn git_dir_path(&self) -> PathBuf {
        unreachable!("repository admission fails without descriptor paths")
    }
}

#[cfg(target_os = "linux")]
fn snapshot_executable(label: &'static str, path: &Path) -> Result<(File, [u8; 32]), CoordError> {
    let (subject, fingerprint, _) = snapshot_subject(label, path, MAX_EXECUTABLE_BYTES, true)?;
    Ok((subject, fingerprint))
}

#[cfg(target_os = "linux")]
fn snapshot_subject(
    label: &'static str,
    path: &Path,
    maximum: u64,
    executable: bool,
) -> Result<(File, [u8; 32], Identity), CoordError> {
    let source = open_regular(path, label, maximum, executable)?;
    snapshot_open_subject(label, source, maximum, executable)
}

#[cfg(target_os = "linux")]
fn snapshot_open_subject(
    label: &'static str,
    mut source: File,
    maximum: u64,
    executable: bool,
) -> Result<(File, [u8; 32], Identity), CoordError> {
    use nix::{
        fcntl::{FcntlArg, FdFlag, SealFlag, fcntl},
        sys::{
            memfd::{MemFdCreateFlag, memfd_create},
            stat::{Mode as NixMode, fchmod},
        },
    };

    let source_identity = identity(&source)?;
    let descriptor = memfd_create(
        c"bullet-family-lock-tool",
        MemFdCreateFlag::MFD_ALLOW_SEALING,
    )
    .map_err(|error| tool_error(label, error.to_string()))?;
    let mut writable = File::from(descriptor);
    let fingerprint = copy_fingerprint(label, &mut source, Some(&mut writable), maximum)?;
    writable
        .flush()
        .map_err(|error| tool_error(label, error.to_string()))?;
    let mode = if executable {
        NixMode::S_IRUSR | NixMode::S_IXUSR
    } else {
        NixMode::S_IRUSR
    };
    fchmod(writable.as_raw_fd(), mode).map_err(|error| tool_error(label, error.to_string()))?;
    fcntl(
        writable.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|error| tool_error(label, error.to_string()))?;
    let subject = File::open(descriptor_path(writable.as_raw_fd()))
        .map_err(|error| tool_error(label, error.to_string()))?;
    fcntl(subject.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty()))
        .map_err(|error| tool_error(label, error.to_string()))?;
    Ok((subject, fingerprint, source_identity))
}

#[cfg(not(target_os = "linux"))]
fn snapshot_executable(_label: &'static str, _path: &Path) -> Result<(File, [u8; 32]), CoordError> {
    Err(unsupported())
}

#[cfg(not(target_os = "linux"))]
fn snapshot_subject(
    _label: &'static str,
    _path: &Path,
    _maximum: u64,
    _executable: bool,
) -> Result<(File, [u8; 32], Identity), CoordError> {
    Err(unsupported())
}

#[cfg(target_os = "linux")]
fn fingerprint_path(
    label: &'static str,
    path: &Path,
    maximum: u64,
    executable: bool,
) -> Result<[u8; 32], CoordError> {
    let mut source = open_regular(path, label, maximum, executable)?;
    copy_fingerprint(label, &mut source, None, maximum)
}

#[cfg(not(target_os = "linux"))]
fn fingerprint_path(
    _label: &'static str,
    _path: &Path,
    _maximum: u64,
    _executable: bool,
) -> Result<[u8; 32], CoordError> {
    Err(unsupported())
}

#[cfg(target_os = "linux")]
fn open_regular(
    path: &Path,
    label: &'static str,
    maximum: u64,
    executable: bool,
) -> Result<File, CoordError> {
    use std::os::unix::fs::OpenOptionsExt;

    let source = fs::OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| tool_error(label, error.to_string()))?;
    let metadata = source
        .metadata()
        .map_err(|error| tool_error(label, error.to_string()))?;
    if !metadata.file_type().is_file() || metadata.len() > maximum {
        return Err(tool_error(label, "subject is not a bounded regular file"));
    }
    if executable && metadata.permissions().mode() & 0o111 == 0 {
        return Err(tool_error(label, "subject is not executable"));
    }
    Ok(source)
}

#[cfg(target_os = "linux")]
fn copy_fingerprint(
    label: &'static str,
    source: &mut File,
    mut destination: Option<&mut File>,
    maximum: u64,
) -> Result<[u8; 32], CoordError> {
    let mut hasher = blake3::Hasher::new();
    let mut count = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| tool_error(label, error.to_string()))?;
        if read == 0 {
            return Ok(*hasher.finalize().as_bytes());
        }
        count = count
            .checked_add(read as u64)
            .ok_or_else(|| tool_error(label, "subject size overflowed"))?;
        if count > maximum {
            return Err(tool_error(label, "subject exceeds its admission limit"));
        }
        hasher.update(&buffer[..read]);
        if let Some(destination) = destination.as_mut() {
            destination
                .write_all(&buffer[..read])
                .map_err(|error| tool_error(label, error.to_string()))?;
        }
    }
}

#[cfg(target_os = "linux")]
fn open_directory<Fd: rustix::fd::AsFd>(
    directory: Fd,
    path: &Path,
    label: &str,
) -> Result<File, CoordError> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::NONBLOCK;
    openat(directory, path, flags, Mode::empty())
        .map(File::from)
        .map_err(|error| {
            if matches!(error, Errno::LOOP | Errno::NOTDIR) {
                CoordError::new(
                    "FORBIDDEN_GIT_LAYOUT",
                    format!("{label} must be a non-symlink directory"),
                )
            } else {
                CoordError::new(
                    "FORBIDDEN_GIT_LAYOUT",
                    format!("cannot open {label}: {error}"),
                )
            }
        })
}

#[cfg(target_os = "linux")]
fn read_config_subject(git_dir: &File) -> Result<(Identity, [u8; 32]), CoordError> {
    let descriptor = openat(
        git_dir,
        Path::new("config"),
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| {
        CoordError::new(
            "FORBIDDEN_GIT_LAYOUT",
            format!("cannot open repository-local Git config: {error}"),
        )
    })?;
    let mut config = File::from(descriptor);
    let identity = identity(&config)?;
    let fingerprint = copy_fingerprint(
        "repository-local Git config",
        &mut config,
        None,
        MAX_LOCAL_CONFIG_BYTES,
    )?;
    Ok((identity, fingerprint))
}

#[cfg(target_os = "linux")]
fn identity(file: &File) -> Result<Identity, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    Ok(Identity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(target_os = "linux")]
fn make_inheritable(file: &File, label: &str) -> Result<(), CoordError> {
    use nix::fcntl::{FcntlArg, FdFlag, fcntl};

    fcntl(file.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty())).map_err(|error| {
        CoordError::new(
            "GIT_SUBJECT_PIN_FAILED",
            format!("could not inherit {label} descriptor: {error}"),
        )
    })?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn descriptor_path(descriptor: RawFd) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{descriptor}"))
}

fn tool_error(label: &str, detail: impl AsRef<str>) -> CoordError {
    CoordError::new(
        "GIT_TOOL_UNAVAILABLE",
        format!("{label}: {}", detail.as_ref()),
    )
}

fn repository_changed(detail: &str) -> CoordError {
    CoordError::new("GIT_REPOSITORY_CHANGED", detail)
}

#[cfg(not(target_os = "linux"))]
fn unsupported() -> CoordError {
    CoordError::new(
        "UNSUPPORTED_PLATFORM_CONTAINMENT",
        "descriptor-pinned family-lock Git verification is currently available only on Linux",
    )
}
