use std::{ffi::OsString, os::unix::ffi::OsStrExt};

use rustix::{
    fd::AsFd,
    fs::{AtFlags, Dir, FileType, Mode, fsync, fstat, openat, statat, unlinkat},
    io::Errno,
};

use super::{
    DIRECTORY_FLAGS, Identity, Staging, coord_io, identity_of, orphan::OWNER_FILE, staging_error,
};
use crate::coord::CoordError;

const MAX_CLEANUP_DEPTH: usize = 64;
const MAX_CLEANUP_ENTRIES: usize = 100_000;

impl Staging {
    pub(in crate::setup) fn finish(mut self) -> Result<(), CoordError> {
        self.cleanup()?;
        self.root.sync()
    }

    fn cleanup(&mut self) -> Result<(), CoordError> {
        if self.directory.is_none() {
            return Ok(());
        }
        self.ensure_path_identity()?;
        self.verify_owner_marker()?;
        let mut directory = self.directory.take().expect("checked staging descriptor");
        let stage = fstat(directory.fd().map_err(coord_io)?).map_err(coord_io)?;
        let mut budget = MAX_CLEANUP_ENTRIES;
        cleanup_directory(&mut directory, 0, &mut budget, stage.st_dev)?;
        require_only_owner_marker(&mut directory)?;
        unlinkat(
            directory.fd().map_err(coord_io)?,
            OWNER_FILE,
            AtFlags::empty(),
        )
        .map_err(|error| staging_error("cannot remove staging ownership marker", error))?;
        fsync(directory.fd().map_err(coord_io)?).map_err(coord_io)?;

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
    stage_device: u64,
) -> Result<(), CoordError> {
    if depth > MAX_CLEANUP_DEPTH {
        return Err(CoordError::new(
            "SETUP_CLEANUP_LIMIT",
            "staging cleanup exceeded its maximum depth and was preserved",
        ));
    }
    let mut names = list_names(directory, budget)?;
    names.sort_by(|left, right| {
        let left_owner = depth == 0 && left.as_os_str().as_bytes() == OWNER_FILE.as_bytes();
        let right_owner = depth == 0 && right.as_os_str().as_bytes() == OWNER_FILE.as_bytes();
        left_owner
            .cmp(&right_owner)
            .then_with(|| left.as_os_str().as_bytes().cmp(right.as_os_str().as_bytes()))
    });
    let parent = directory.fd().map_err(coord_io)?;
    for name in names {
        if depth == 0 && name.as_os_str().as_bytes() == OWNER_FILE.as_bytes() {
            continue;
        }
        let stat = match statat(parent, &name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(Errno::NOENT) => continue,
            Err(error) => return Err(staging_error("cannot inspect staging entry", error)),
        };
        if stat.st_dev != stage_device || stat.st_uid != rustix::process::geteuid().as_raw() {
            return Err(ambiguous("a staging child has foreign ownership or filesystem identity"));
        }
        let kind = FileType::from_raw_mode(stat.st_mode);
        if kind.is_dir() {
            let descriptor = openat(parent, &name, DIRECTORY_FLAGS, Mode::empty())
                .map_err(|error| staging_error("cannot open staging child", error))?;
            let opened = fstat(&descriptor).map_err(coord_io)?;
            if opened.st_dev != stat.st_dev
                || opened.st_ino != stat.st_ino
                || opened.st_uid != stat.st_uid
            {
                return Err(ambiguous("a staging directory changed during admission"));
            }
            let mut child = Dir::new(descriptor).map_err(coord_io)?;
            cleanup_directory(&mut child, depth + 1, budget, stage_device)?;
            fsync(child.fd().map_err(coord_io)?).map_err(coord_io)?;
            let expected = identity_of(child.fd().map_err(coord_io)?)?;
            let current = statat(parent, &name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| staging_error("cannot recheck staging child", error))?;
            if expected
                != (Identity {
                    device: current.st_dev,
                    inode: current.st_ino,
                })
            {
                return Err(ambiguous("a staging child changed during cleanup"));
            }
            unlinkat(parent, &name, AtFlags::REMOVEDIR)
                .map_err(|error| staging_error("cannot remove staging child", error))?;
        } else if kind.is_file() || kind.is_symlink() {
            unlinkat(parent, &name, AtFlags::empty())
                .map_err(|error| staging_error("cannot remove staging entry", error))?;
        } else {
            return Err(ambiguous("staging contains an unsupported filesystem object"));
        }
    }
    fsync(parent).map_err(coord_io)
}

fn list_names(directory: &mut Dir, budget: &mut usize) -> Result<Vec<OsString>, CoordError> {
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
    Ok(names)
}

fn require_only_owner_marker(directory: &mut Dir) -> Result<(), CoordError> {
    let mut budget = 2;
    let names = list_names(directory, &mut budget)?;
    if names.len() == 1 && names[0].as_os_str().as_bytes() == OWNER_FILE.as_bytes() {
        return Ok(());
    }
    Err(ambiguous(
        "staging changed after bounded cleanup; its ownership marker was preserved",
    ))
}

fn ambiguous(detail: &str) -> CoordError {
    CoordError::new("SETUP_STAGING_ORPHAN_AMBIGUOUS", detail)
}
