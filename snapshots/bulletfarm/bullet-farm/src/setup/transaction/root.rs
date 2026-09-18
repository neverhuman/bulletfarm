use std::{
    ffi::OsString,
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{
        AtFlags, Dir, FileType, Mode, OFlags, RenameFlags, fchmod, fstat, fsync, linkat, mkdirat,
        openat, renameat_with, statat, unlinkat,
    },
    io::Errno,
};

use super::super::STAGING_PREFIX;
use crate::coord::CoordError;

const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_CLEANUP_DEPTH: usize = 64;
const MAX_CLEANUP_ENTRIES: usize = 100_000;
const DIRECTORY_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

#[derive(Clone, Debug)]
pub(in crate::setup) struct AdmittedRoot {
    inner: Arc<RootInner>,
}

#[derive(Debug)]
struct RootInner {
    path: PathBuf,
    directory: File,
    identity: Identity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
}

impl AdmittedRoot {
    pub(in crate::setup) fn open(path: &Path) -> Result<Self, CoordError> {
        let descriptor =
            openat(rustix::fs::CWD, path, DIRECTORY_FLAGS, Mode::empty()).map_err(|error| {
                if matches!(error, Errno::LOOP | Errno::NOTDIR) {
                    CoordError::new(
                        "INVALID_CHECKOUT",
                        "setup root must be a non-symlink directory",
                    )
                } else {
                    root_error("cannot open the admitted setup root", error)
                }
            })?;
        let directory = File::from(descriptor);
        let identity = identity_of(&directory)?;
        let path = path.canonicalize().map_err(CoordError::io)?;
        let root = Self {
            inner: Arc::new(RootInner {
                path,
                directory,
                identity,
            }),
        };
        root.ensure_path_identity()?;
        Ok(root)
    }

    pub(in crate::setup) fn path(&self) -> &Path {
        &self.inner.path
    }

    pub(in crate::setup) fn ensure_path_identity(&self) -> Result<(), CoordError> {
        let metadata = fs::symlink_metadata(&self.inner.path).map_err(|error| {
            CoordError::new(
                "SETUP_ROOT_REPLACED",
                format!("the admitted setup root is no longer reachable: {error}"),
            )
        })?;
        let actual = Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        };
        if !metadata.file_type().is_dir() || actual != self.inner.identity {
            return Err(CoordError::new(
                "SETUP_ROOT_REPLACED",
                "the admitted setup-root pathname now names a different object",
            ));
        }
        Ok(())
    }

    pub(in crate::setup) fn create_staging(&self, class: &str) -> Result<Staging, CoordError> {
        admit_component(class, "staging class")?;
        self.ensure_path_identity()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| CoordError::new("CLOCK_BEFORE_EPOCH", error.to_string()))?
            .as_nanos();
        for sequence in 0..64 {
            let name = OsString::from(format!(
                "{STAGING_PREFIX}{class}.{}.{}.{}",
                std::process::id(),
                now,
                sequence
            ));
            match mkdirat(&self.inner.directory, &name, Mode::RWXU) {
                Ok(()) => {
                    let descriptor =
                        openat(&self.inner.directory, &name, DIRECTORY_FLAGS, Mode::empty())
                            .map_err(|error| staging_error("cannot open new staging", error))?;
                    fchmod(&descriptor, Mode::RWXU)
                        .map_err(|error| staging_error("cannot make staging private", error))?;
                    let staging = Staging {
                        root: self.clone(),
                        name,
                        path: self.inner.path.join(format!(
                            "{STAGING_PREFIX}{class}.{}.{}.{}",
                            std::process::id(),
                            now,
                            sequence
                        )),
                        directory: Some(Dir::new(descriptor).map_err(coord_io)?),
                    };
                    staging.ensure_path_identity()?;
                    self.sync()?;
                    return Ok(staging);
                }
                Err(Errno::EXIST) => continue,
                Err(error) => return Err(staging_error("cannot create private staging", error)),
            }
        }
        Err(CoordError::new(
            "STAGING_COLLISION",
            "could not allocate a unique setup staging directory",
        ))
    }

    pub(super) fn read_optional_file(&self, name: &str) -> Result<Option<Vec<u8>>, CoordError> {
        admit_component(name, "family-root file")?;
        let descriptor = match openat(
            &self.inner.directory,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(descriptor) => descriptor,
            Err(Errno::NOENT) => return Ok(None),
            Err(error) => return Err(root_error("cannot open family-root file", error)),
        };
        read_bounded_regular(descriptor, &format!("family-root {name}")).map(Some)
    }

    pub(super) fn read_hub_manifest(&self) -> Result<Vec<u8>, CoordError> {
        self.ensure_path_identity()?;
        let hub = openat(
            &self.inner.directory,
            "bullet-farm",
            DIRECTORY_FLAGS,
            Mode::empty(),
        )
        .map_err(|error| root_error("cannot open the pinned hub checkout", error))?;
        let descriptor = openat(
            &hub,
            "repos.manifest.toml",
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| root_error("cannot open the pinned hub manifest", error))?;
        let bytes = read_bounded_regular(descriptor, "pinned hub repos.manifest.toml")?;
        self.ensure_path_identity()?;
        Ok(bytes)
    }

    fn sync(&self) -> Result<(), CoordError> {
        fsync(&self.inner.directory).map_err(coord_io)
    }
}

#[derive(Debug)]
pub(in crate::setup) struct Staging {
    root: AdmittedRoot,
    name: OsString,
    path: PathBuf,
    directory: Option<Dir>,
}

impl Staging {
    pub(in crate::setup) fn path(&self) -> &Path {
        &self.path
    }

    pub(in crate::setup) fn ensure_path_identity(&self) -> Result<(), CoordError> {
        self.root.ensure_path_identity()?;
        let directory = self.directory()?;
        let expected = identity_of(directory.fd().map_err(coord_io)?)?;
        let actual = statat(
            &self.root.inner.directory,
            &self.name,
            AtFlags::SYMLINK_NOFOLLOW,
        )
        .map_err(|error| staging_error("cannot inspect staging identity", error))?;
        if !FileType::from_raw_mode(actual.st_mode).is_dir()
            || expected
                != (Identity {
                    device: actual.st_dev,
                    inode: actual.st_ino,
                })
        {
            return Err(CoordError::new(
                "SETUP_STAGING_REPLACED",
                "setup staging was renamed or replaced",
            ));
        }
        Ok(())
    }

    pub(in crate::setup) fn create_private_dir(&self, name: &str) -> Result<(), CoordError> {
        admit_component(name, "staging directory")?;
        self.ensure_path_identity()?;
        let directory = self.directory()?;
        mkdirat(directory.fd().map_err(coord_io)?, name, Mode::RWXU)
            .map_err(|error| staging_error("cannot create private staging child", error))?;
        let child = openat(
            directory.fd().map_err(coord_io)?,
            name,
            DIRECTORY_FLAGS,
            Mode::empty(),
        )
        .map_err(|error| staging_error("cannot open private staging child", error))?;
        fchmod(&child, Mode::RWXU)
            .map_err(|error| staging_error("cannot set private staging child mode", error))?;
        fsync(&child).map_err(coord_io)?;
        fsync(directory.fd().map_err(coord_io)?).map_err(coord_io)
    }

    pub(super) fn publish_member(&self, member: &str) -> Result<(), CoordError> {
        admit_component(member, "family member")?;
        self.ensure_path_identity()?;
        let directory = self.directory()?;
        renameat_with(
            directory.fd().map_err(coord_io)?,
            member,
            &self.root.inner.directory,
            member,
            RenameFlags::NOREPLACE,
        )
        .map_err(|error| {
            if error == Errno::EXIST {
                CoordError::new(
                    "CHECKOUT_CONFLICT",
                    format!("{member} appeared during setup; the existing path was preserved"),
                )
            } else {
                CoordError::new(
                    "CHECKOUT_PUBLICATION_FAILED",
                    format!("cannot publish {member} without replacement: {error}"),
                )
            }
        })?;
        self.root.sync()?;
        self.root.ensure_path_identity()
    }

    pub(super) fn write_manifest(&self, bytes: &[u8]) -> Result<(), CoordError> {
        self.ensure_path_identity()?;
        let directory = self.directory()?;
        let descriptor = openat(
            directory.fd().map_err(coord_io)?,
            "manifest.tmp",
            OFlags::WRONLY
                | OFlags::CREATE
                | OFlags::EXCL
                | OFlags::NOFOLLOW
                | OFlags::NONBLOCK
                | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|error| staging_error("cannot create manifest staging file", error))?;
        let mut file = File::from(descriptor);
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(CoordError::io)
    }

    pub(super) fn link_manifest(&self) -> Result<(), CoordError> {
        let directory = self.directory()?;
        linkat(
            directory.fd().map_err(coord_io)?,
            "manifest.tmp",
            &self.root.inner.directory,
            "repos.manifest.toml",
            AtFlags::empty(),
        )
        .map_err(|error| {
            CoordError::new(
                "FAMILY_MANIFEST_CONFLICT",
                format!("cannot publish without replacement: {error}"),
            )
        })
    }

    pub(super) fn sync_root(&self) -> Result<(), CoordError> {
        self.root.sync()
    }

    pub(super) fn remove_manifest_temporary(&self) -> Result<(), CoordError> {
        let directory = self.directory()?;
        unlinkat(
            directory.fd().map_err(coord_io)?,
            "manifest.tmp",
            AtFlags::empty(),
        )
        .map_err(coord_io)?;
        fsync(directory.fd().map_err(coord_io)?).map_err(coord_io)
    }

    pub(in crate::setup) fn finish(mut self) -> Result<(), CoordError> {
        self.cleanup()?;
        self.root.sync()
    }

    fn directory(&self) -> Result<&Dir, CoordError> {
        self.directory.as_ref().ok_or_else(|| {
            CoordError::new("SETUP_STAGING_CLOSED", "setup staging is already closed")
        })
    }

    fn cleanup(&mut self) -> Result<(), CoordError> {
        let Some(mut directory) = self.directory.take() else {
            return Ok(());
        };
        let mut budget = MAX_CLEANUP_ENTRIES;
        cleanup_directory(&mut directory, 0, &mut budget)?;
        let expected = identity_of(directory.fd().map_err(coord_io)?)?;
        let actual = statat(
            &self.root.inner.directory,
            &self.name,
            AtFlags::SYMLINK_NOFOLLOW,
        )
        .map_err(|error| staging_error("cannot inspect staging before cleanup", error))?;
        if expected
            != (Identity {
                device: actual.st_dev,
                inode: actual.st_ino,
            })
        {
            return Err(CoordError::new(
                "SETUP_STAGING_REPLACED",
                "setup staging changed before cleanup; it was preserved",
            ));
        }
        unlinkat(&self.root.inner.directory, &self.name, AtFlags::REMOVEDIR)
            .map_err(|error| staging_error("cannot remove empty staging", error))?;
        Ok(())
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = self.cleanup();
        let _ = self.root.sync();
    }
}

fn cleanup_directory(
    directory: &mut Dir,
    depth: usize,
    budget: &mut usize,
) -> Result<(), CoordError> {
    if depth > MAX_CLEANUP_DEPTH {
        return Err(CoordError::new(
            "SETUP_CLEANUP_LIMIT",
            "staging cleanup exceeded its maximum depth and was preserved",
        ));
    }
    let mut names = Vec::new();
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(coord_io)?;
        let bytes = entry.file_name().to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        if *budget == 0 {
            return Err(CoordError::new(
                "SETUP_CLEANUP_LIMIT",
                "staging cleanup exceeded its entry limit and was preserved",
            ));
        }
        *budget -= 1;
        use std::os::unix::ffi::OsStringExt;
        names.push(OsString::from_vec(bytes.to_vec()));
    }
    let parent = directory.fd().map_err(coord_io)?;
    for name in names {
        let stat = match statat(parent, &name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(Errno::NOENT) => continue,
            Err(error) => return Err(staging_error("cannot inspect staging entry", error)),
        };
        if FileType::from_raw_mode(stat.st_mode).is_dir() {
            let descriptor = openat(parent, &name, DIRECTORY_FLAGS, Mode::empty())
                .map_err(|error| staging_error("cannot open staging child", error))?;
            let mut child = Dir::new(descriptor).map_err(coord_io)?;
            cleanup_directory(&mut child, depth + 1, budget)?;
            let expected = identity_of(child.fd().map_err(coord_io)?)?;
            let current = statat(parent, &name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| staging_error("cannot recheck staging child", error))?;
            if expected
                != (Identity {
                    device: current.st_dev,
                    inode: current.st_ino,
                })
            {
                return Err(CoordError::new(
                    "SETUP_STAGING_REPLACED",
                    "a staging child changed during cleanup and was preserved",
                ));
            }
            unlinkat(parent, &name, AtFlags::REMOVEDIR)
                .map_err(|error| staging_error("cannot remove staging child", error))?;
        } else {
            unlinkat(parent, &name, AtFlags::empty())
                .map_err(|error| staging_error("cannot remove staging entry", error))?;
        }
    }
    Ok(())
}

fn identity_of(fd: impl AsFd) -> Result<Identity, CoordError> {
    let stat = fstat(fd).map_err(coord_io)?;
    Ok(Identity {
        device: stat.st_dev,
        inode: stat.st_ino,
    })
}

fn read_bounded_regular(descriptor: OwnedFd, label: &str) -> Result<Vec<u8>, CoordError> {
    let stat = fstat(&descriptor).map_err(coord_io)?;
    if !FileType::from_raw_mode(stat.st_mode).is_file() {
        return Err(CoordError::new(
            "FAMILY_MANIFEST_CONFLICT",
            format!("{label} is not a regular file"),
        ));
    }
    let mut file = File::from(descriptor);
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(CoordError::new(
            "FAMILY_MANIFEST_CONFLICT",
            format!("{label} exceeds the bounded admission limit"),
        ));
    }
    Ok(bytes)
}

fn admit_component(value: &str, label: &str) -> Result<(), CoordError> {
    let path = Path::new(value);
    if value.is_empty()
        || value == "."
        || value == ".."
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(CoordError::new(
            "INVALID_SETUP_COMPONENT",
            format!("{label} must be one ordinary path component"),
        ));
    }
    Ok(())
}

fn root_error(context: &str, error: Errno) -> CoordError {
    CoordError::new("SETUP_ROOT_UNAVAILABLE", format!("{context}: {error}"))
}

fn staging_error(context: &str, error: Errno) -> CoordError {
    CoordError::new("SETUP_STAGING_IO", format!("{context}: {error}"))
}

fn coord_io(error: Errno) -> CoordError {
    CoordError::io(error.into())
}
