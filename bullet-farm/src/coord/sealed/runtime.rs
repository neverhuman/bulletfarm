use std::{fs::File, os::unix::fs::MetadataExt, path::Path};

use rustix::fs::{AtFlags, Mode, OFlags, ResolveFlags, open, openat2, statat};

use crate::coord::CoordError;

use super::{Parent, changed, invalid};

const ROOT_RUNTIME_PARENT: &str = "/run/bullet";
const TMPFS_MAGIC: u64 = 0x0102_1994;

impl super::ParentAdmission {
    pub(super) fn file_mode(self) -> Mode {
        match self {
            Self::Sealed | Self::FrozenLegacy => Mode::RUSR,
            Self::RootRuntime => Mode::RUSR | Mode::RGRP | Mode::ROTH,
        }
    }
}

pub(super) fn open_parent(path: &Path) -> Result<File, CoordError> {
    if path != Path::new(ROOT_RUNTIME_PARENT) {
        return Err(invalid("root runtime document has an untrusted parent"));
    }
    let root = File::from(
        open("/", directory_flags(), Mode::empty())
            .map_err(|error| invalid(format!("cannot open runtime filesystem root: {error}")))?,
    );
    let root_identity = DirectoryIdentity::for_file(&root)?;
    let run = File::from(
        openat2(&root, "run", directory_flags(), Mode::empty(), beneath())
            .map_err(|error| invalid(format!("cannot open runtime /run mount: {error}")))?,
    );
    let run_identity = DirectoryIdentity::for_file(&run)?;
    let run_filesystem = rustix::fs::fstatfs(&run)
        .map_err(|error| invalid(format!("cannot identify runtime /run mount: {error}")))?;
    let parent = File::from(
        openat2(
            &run,
            "bullet",
            directory_flags(),
            Mode::empty(),
            beneath() | ResolveFlags::NO_XDEV,
        )
        .map_err(|error| invalid(format!("cannot open runtime trust parent: {error}")))?,
    );
    let parent_identity = DirectoryIdentity::for_file(&parent)?;
    validate_chain(
        root_identity,
        run_identity,
        parent_identity,
        run_filesystem.f_type as u64,
    )?;
    Ok(parent)
}

pub(super) fn open_document(parent: &File, name: &str) -> Result<File, CoordError> {
    let descriptor = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
        beneath() | ResolveFlags::NO_XDEV,
    )
    .map_err(|error| invalid(format!("cannot open root runtime document: {error}")))?;
    Ok(File::from(descriptor))
}

pub(super) fn adopt_exact(
    parent: &Parent,
    path: &Path,
    expected: &[u8],
    maximum: u64,
) -> Result<bool, CoordError> {
    match statat(&parent.file, &parent.name, AtFlags::SYMLINK_NOFOLLOW) {
        Err(rustix::io::Errno::NOENT) => Ok(false),
        Err(error) => Err(invalid(format!(
            "cannot inspect root runtime publication: {error}"
        ))),
        Ok(_) => {
            let (first, first_identity) = parent.read_once(maximum)?;
            let (second, second_identity) = parent.read_once(maximum)?;
            if first_identity != second_identity || !exact_existing_bytes(expected, &first, &second)
            {
                return Err(changed(
                    "existing root runtime publication differs from exact intended bytes",
                ));
            }
            let public =
                statat(&parent.file, &parent.name, AtFlags::SYMLINK_NOFOLLOW).map_err(|error| {
                    changed(format!("cannot restat root runtime publication: {error}"))
                })?;
            if (public.st_dev, public.st_ino) != (second_identity.device, second_identity.inode)
                || public.st_uid != second_identity.owner_uid
                || public.st_gid != second_identity.owner_gid
                || public.st_mode & 0o7777 != second_identity.mode
                || public.st_nlink != second_identity.links
                || u64::try_from(public.st_size).ok() != Some(second_identity.length)
                || public.st_mtime != second_identity.mtime_seconds
                || i64::try_from(public.st_mtime_nsec).ok()
                    != Some(second_identity.mtime_nanoseconds)
                || public.st_ctime != second_identity.ctime_seconds
                || i64::try_from(public.st_ctime_nsec).ok()
                    != Some(second_identity.ctime_nanoseconds)
            {
                return Err(changed(
                    "root runtime publication pathname changed after stable read",
                ));
            }
            parent.revalidate_path(path)?;
            Ok(true)
        }
    }
}

pub(super) fn exact_existing_bytes(expected: &[u8], first: &[u8], second: &[u8]) -> bool {
    first == expected && second == expected
}

#[derive(Clone, Copy)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
}

impl DirectoryIdentity {
    fn for_file(file: &File) -> Result<Self, CoordError> {
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_dir() {
            return Err(invalid("runtime trust ancestor is not a directory"));
        }
        Ok(Self {
            device: value.dev(),
            inode: value.ino(),
            owner_uid: value.uid(),
            owner_gid: value.gid(),
            mode: value.mode() & 0o7777,
        })
    }

    fn has_root_custody(self) -> bool {
        self.device != 0
            && self.inode != 0
            && self.owner_uid == 0
            && self.owner_gid == 0
            && self.mode == 0o755
    }
}

fn validate_chain(
    root: DirectoryIdentity,
    run: DirectoryIdentity,
    parent: DirectoryIdentity,
    run_filesystem: u64,
) -> Result<(), CoordError> {
    if !root.has_root_custody()
        || !run.has_root_custody()
        || !parent.has_root_custody()
        || run_filesystem != TMPFS_MAGIC
        || root.device == run.device
        || run.device != parent.device
    {
        return Err(invalid(
            "runtime trust ancestry or admitted /run mount transition is invalid",
        ));
    }
    Ok(())
}

fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

fn beneath() -> ResolveFlags {
    ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(device: u64) -> DirectoryIdentity {
        DirectoryIdentity {
            device,
            inode: 1,
            owner_uid: 0,
            owner_gid: 0,
            mode: 0o755,
        }
    }

    #[test]
    fn runtime_trust_chain_closes_ancestors_and_mount_transition() {
        let root = identity(1);
        let run = identity(2);
        let parent = identity(2);
        validate_chain(root, run, parent, TMPFS_MAGIC).unwrap();

        for (root, run, parent, filesystem) in [
            (
                DirectoryIdentity {
                    owner_uid: 1,
                    ..root
                },
                run,
                parent,
                TMPFS_MAGIC,
            ),
            (
                root,
                DirectoryIdentity { mode: 0o775, ..run },
                parent,
                TMPFS_MAGIC,
            ),
            (
                root,
                run,
                DirectoryIdentity {
                    owner_gid: 1,
                    ..parent
                },
                TMPFS_MAGIC,
            ),
            (root, run, parent, 0),
            (root, identity(1), identity(1), TMPFS_MAGIC),
            (root, run, identity(3), TMPFS_MAGIC),
        ] {
            assert!(validate_chain(root, run, parent, filesystem).is_err());
        }
    }
}
