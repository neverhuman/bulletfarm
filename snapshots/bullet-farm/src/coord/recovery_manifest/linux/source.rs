use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use rustix::fs::{Mode, OFlags, ResolveFlags, open, openat2};

use super::{MAX_ARTIFACT_BYTES, RecoveryFileIdentityV1, changed, invalid};
use crate::coord::CoordError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParentRole {
    Sealed,
    FrozenLegacy,
}

pub(super) struct StableSource {
    pub(super) bytes: Vec<u8>,
    pub(super) identity: RecoveryFileIdentityV1,
}

pub(super) fn read_stable(
    path: &Path,
    label: &'static str,
    role: ParentRole,
) -> Result<StableSource, CoordError> {
    let parent = Parent::open(path, role, label)?;
    let first = parent.read_once(path, label)?;
    let second = parent.read_once(path, label)?;
    if first.identity != second.identity || first.bytes != second.bytes {
        return Err(changed(format!("{label} changed during stable read")));
    }
    parent.revalidate(path, label)?;
    Ok(first)
}

struct Parent {
    file: File,
    name: String,
    identity: ParentIdentity,
    role: ParentRole,
}

impl Parent {
    fn open(path: &Path, role: ParentRole, label: &'static str) -> Result<Self, CoordError> {
        let parent_path = path
            .parent()
            .ok_or_else(|| invalid(format!("{label} has no immediate parent")))?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid(format!("{label} filename must be valid UTF-8")))?
            .to_owned();
        let root = open(
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| invalid(format!("cannot open filesystem root for {label}: {error}")))?;
        let relative = parent_path
            .strip_prefix("/")
            .map_err(|_| invalid(format!("{label} parent is outside filesystem root")))?;
        let relative = if relative.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            relative.to_owned()
        };
        let descriptor = openat2(
            &root,
            &relative,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            resolve(),
        )
        .map_err(|error| invalid(format!("cannot open immediate parent for {label}: {error}")))?;
        let file = File::from(descriptor);
        let identity = ParentIdentity::for_file(&file, label)?;
        identity.validate(role, label)?;
        Ok(Self {
            file,
            name,
            identity,
            role,
        })
    }

    fn read_once(&self, path: &Path, label: &'static str) -> Result<StableSource, CoordError> {
        let descriptor = openat2(
            &self.file,
            &self.name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
            resolve(),
        )
        .map_err(|error| invalid(format!("cannot open {label}: {error}")))?;
        let mut file = File::from(descriptor);
        let before = file_identity(path, &file, label)?;
        if before.byte_length == 0 || before.byte_length > MAX_ARTIFACT_BYTES {
            return Err(invalid(format!(
                "{label} length must be within 1..={MAX_ARTIFACT_BYTES} bytes"
            )));
        }
        let mut bytes = Vec::with_capacity(before.byte_length as usize);
        (&mut file)
            .take(MAX_ARTIFACT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(CoordError::io)?;
        let after = file_identity(path, &file, label)?;
        if before != after || bytes.len() as u64 != before.byte_length {
            return Err(changed(format!("{label} changed while being read")));
        }
        Ok(StableSource {
            bytes,
            identity: before,
        })
    }

    fn revalidate(&self, path: &Path, label: &'static str) -> Result<(), CoordError> {
        let observed = Self::open(path, self.role, label)?;
        if observed.identity != self.identity {
            return Err(changed(format!(
                "{label} immediate parent changed during stable read"
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParentIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
}

impl ParentIdentity {
    fn for_file(file: &File, label: &'static str) -> Result<Self, CoordError> {
        let metadata = file.metadata().map_err(CoordError::io)?;
        if !metadata.is_dir() {
            return Err(invalid(format!("{label} parent is not a directory")));
        }
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner_uid: metadata.uid(),
            owner_gid: metadata.gid(),
            mode: metadata.mode() & 0o7777,
        })
    }

    fn validate(self, role: ParentRole, label: &'static str) -> Result<(), CoordError> {
        let mode_admitted = match role {
            ParentRole::Sealed => self.mode == 0o700,
            ParentRole::FrozenLegacy => matches!(self.mode, 0o700 | 0o775),
        };
        if self.device == 0
            || self.inode == 0
            || self.owner_uid != rustix::process::geteuid().as_raw()
            || !mode_admitted
        {
            return Err(invalid(format!(
                "{label} immediate parent owner or role-specific exact mode is not admitted"
            )));
        }
        Ok(())
    }
}

fn file_identity(
    path: &Path,
    file: &File,
    label: &'static str,
) -> Result<RecoveryFileIdentityV1, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    let identity = RecoveryFileIdentityV1 {
        path: path
            .to_str()
            .ok_or_else(|| invalid(format!("{label} path must be valid UTF-8")))?
            .to_owned(),
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        owner_gid: metadata.gid(),
        mode: metadata.mode(),
        link_count: metadata.nlink(),
        byte_length: metadata.len(),
        mtime_seconds: metadata.mtime(),
        mtime_nanoseconds: metadata.mtime_nsec(),
        ctime_seconds: metadata.ctime(),
        ctime_nanoseconds: metadata.ctime_nsec(),
    };
    if !metadata.is_file()
        || identity.owner_uid != rustix::process::geteuid().as_raw()
        || identity.link_count != 1
        || identity.mode & 0o7777 != 0o400
    {
        return Err(invalid(format!(
            "{label} must be an exact-owner, single-link, mode-0400 regular file"
        )));
    }
    Ok(identity)
}

fn resolve() -> ResolveFlags {
    ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_roles_refuse_wrong_owner_and_modes() {
        let uid = rustix::process::geteuid().as_raw();
        let valid = ParentIdentity {
            device: 1,
            inode: 1,
            owner_uid: uid,
            owner_gid: 1,
            mode: 0o700,
        };
        valid.validate(ParentRole::Sealed, "source").unwrap();
        let wrong_uid = uid ^ 1;
        for (identity, role) in [
            (
                ParentIdentity {
                    owner_uid: wrong_uid,
                    ..valid
                },
                ParentRole::Sealed,
            ),
            (
                ParentIdentity {
                    mode: 0o750,
                    ..valid
                },
                ParentRole::Sealed,
            ),
            (
                ParentIdentity {
                    mode: 0o755,
                    ..valid
                },
                ParentRole::FrozenLegacy,
            ),
            (
                ParentIdentity {
                    mode: 0o777,
                    ..valid
                },
                ParentRole::FrozenLegacy,
            ),
        ] {
            assert!(identity.validate(role, "source").is_err());
        }
        ParentIdentity {
            mode: 0o775,
            ..valid
        }
        .validate(ParentRole::FrozenLegacy, "source")
        .unwrap();
    }
}
