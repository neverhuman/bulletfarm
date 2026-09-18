use std::{fs::File, io::Write};

#[cfg(test)]
use std::cell::Cell;

use rustix::fs::{Mode, OFlags, fchmod, openat};

use super::*;
use crate::coord::anonymous_link::{self, LinkOutcome};

#[cfg(test)]
thread_local! {
    static KILL_AFTER_LINK: Cell<Option<&'static str>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(in crate::coord::store::ledger::fs) fn test_kill_after_link(name: &'static str) {
    KILL_AFTER_LINK.with(|value| value.set(Some(name)));
}

pub(super) fn publish(
    parent: &File,
    name: &str,
    bytes: &[u8],
    mode: u32,
) -> Result<bool, CoordError> {
    let (mut anonymous, before_link) = create(parent, bytes, mode)?;
    match anonymous_link::link(
        &anonymous,
        parent,
        name,
        (before_link.device, before_link.inode),
    ) {
        Ok(LinkOutcome::Linked) => {
            kill_after_link(name)?;
            parent.sync_all().map_err(CoordError::io)?;
            verify_linked(parent, name, &mut anonymous, bytes, mode, before_link)?;
            exact(&mut anonymous, bytes)?;
            Ok(true)
        }
        Ok(LinkOutcome::Exists) => {
            exact(&mut anonymous, bytes)?;
            Ok(false)
        }
        Err(error) => Err(atomic_error("cannot publish anonymous file", error)),
    }
}

fn create(parent: &File, bytes: &[u8], mode: u32) -> Result<(File, FileSnapshot), CoordError> {
    let descriptor = openat(
        parent,
        ".",
        OFlags::TMPFILE | OFlags::RDWR | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|error| atomic_error("cannot create anonymous authority file", error))?;
    let mut file = File::from(descriptor);
    file.write_all(bytes).map_err(CoordError::io)?;
    exact(&mut file, bytes)?;
    file.sync_data().map_err(CoordError::io)?;
    exact(&mut file, bytes)?;
    fchmod(&file, Mode::from_bits_retain(mode))
        .map_err(|error| os_error("cannot seal anonymous authority file", error))?;
    file.sync_all().map_err(CoordError::io)?;
    exact(&mut file, bytes)?;
    let stable = snapshot(&file)?;
    if !stable.valid(mode, 0, bytes.len() as u64) {
        return Err(invalid("anonymous authority metadata is invalid"));
    }
    exact(&mut file, bytes)?;
    if snapshot(&file)? != stable {
        return Err(changed("anonymous authority changed before publication"));
    }
    Ok((file, stable))
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct FileSnapshot {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    links: u64,
    length: u64,
    ctime: (i64, i64),
}

impl FileSnapshot {
    fn valid(self, mode: u32, links: u64, length: u64) -> bool {
        self.owner == owner() && self.mode == mode && self.links == links && self.length == length
    }
}

fn snapshot(file: &File) -> Result<FileSnapshot, CoordError> {
    use std::os::unix::fs::MetadataExt;

    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file() {
        return Err(invalid("authority descriptor is not a file"));
    }
    Ok(FileSnapshot {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner: metadata.uid(),
        mode: metadata.mode() & 0o7777,
        links: metadata.nlink(),
        length: metadata.len(),
        ctime: (metadata.ctime(), metadata.ctime_nsec()),
    })
}

fn verify_linked(
    parent: &File,
    name: &str,
    anonymous: &mut File,
    bytes: &[u8],
    mode: u32,
    before: FileSnapshot,
) -> Result<(), CoordError> {
    let mut published = open_file_at(parent, name, false, mode, Some(bytes.len() as u64))?;
    let admitted = snapshot(&published)?;
    if (admitted.device, admitted.inode) != (before.device, before.inode)
        || !admitted.valid(mode, 1, bytes.len() as u64)
    {
        return Err(changed("published file differs from anonymous inode"));
    }
    exact(&mut published, bytes)?;
    exact(anonymous, bytes)?;
    if snapshot(&published)? != admitted || snapshot(anonymous)? != admitted {
        return Err(changed(
            "published authority changed during exact read-back",
        ));
    }
    Ok(())
}

fn atomic_error(context: &str, error: impl std::fmt::Display) -> CoordError {
    CoordError::new(
        "COORD_ATOMIC_PUBLISH_UNSUPPORTED",
        format!("{context}: {error}"),
    )
}

#[cfg(test)]
fn kill_after_link(name: &str) -> Result<(), CoordError> {
    let kill = KILL_AFTER_LINK.with(|value| {
        if value.get().is_some_and(|target| target == name) {
            value.set(None);
            true
        } else {
            false
        }
    });
    if kill {
        nix::sys::signal::raise(nix::sys::signal::Signal::SIGKILL)
            .map_err(|error| CoordError::io(error.into()))?;
    }
    Ok(())
}

#[cfg(not(test))]
fn kill_after_link(_name: &str) -> Result<(), CoordError> {
    Ok(())
}
