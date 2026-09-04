use std::{fs::File, io::Write};

#[cfg(test)]
use std::cell::Cell;

use rustix::fs::{AtFlags, Mode, OFlags, openat, statat};

use super::*;
use crate::coord::anonymous_link::{self, LinkOutcome};

#[cfg(test)]
thread_local! {
    static CRASH_OFFSET: Cell<Option<usize>> = const { Cell::new(None) };
    static CRASH_AFTER_LINK: Cell<bool> = const { Cell::new(false) };
}

#[cfg(test)]
pub(in crate::coord::generation::segment) fn test_crash_at_offset(offset: usize) {
    CRASH_OFFSET.with(|value| value.set(Some(offset)));
}

#[cfg(test)]
pub(in crate::coord::generation::segment) fn test_crash_after_link() {
    CRASH_AFTER_LINK.with(|value| value.set(true));
}

pub(super) fn publish(parent: &File, name: &str, bytes: &[u8]) -> Result<(), CoordError> {
    validate_pending_descriptor(parent)?;
    let descriptor = openat(
        parent,
        ".",
        OFlags::TMPFILE | OFlags::RDWR | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|error| atomic_error("cannot create anonymous pending intent", error))?;
    let mut anonymous = File::from(descriptor);
    write_or_crash(&mut anonymous, bytes)?;
    exact_readback(&mut anonymous, 0, bytes)?;
    anonymous.sync_data().map_err(CoordError::io)?;
    exact_readback(&mut anonymous, 0, bytes)?;
    let stable = snapshot(&anonymous)?;
    validate_anonymous(stable, bytes.len() as u64)?;
    exact_readback(&mut anonymous, 0, bytes)?;
    if snapshot(&anonymous)? != stable {
        return Err(corrupt_pending(
            "anonymous pending intent changed before publication",
        ));
    }
    match anonymous_link::link(&anonymous, parent, name, (stable.device, stable.inode)) {
        Ok(LinkOutcome::Linked) => {}
        Ok(LinkOutcome::Exists) => {
            return Err(atomic_error(
                "cannot publish anonymous pending intent",
                "target already exists",
            ));
        }
        Err(error) => {
            return Err(atomic_error(
                "cannot publish anonymous pending intent",
                error,
            ));
        }
    }
    crash_after_link()?;
    parent.sync_all().map_err(CoordError::io)?;
    validate_published(parent, name, &mut anonymous, bytes, stable)
}

fn validate_published(
    parent: &File,
    name: &str,
    anonymous: &mut File,
    bytes: &[u8],
    before_link: Metadata,
) -> Result<(), CoordError> {
    let descriptor = openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| corrupt_pending(format!("cannot reopen pending intent: {error}")))?;
    let mut published = File::from(descriptor);
    validate_regular_descriptor(&published, 0o600)?;
    let admitted = snapshot(&published)?;
    if (admitted.device, admitted.inode) != (before_link.device, before_link.inode)
        || admitted.links != 1
        || admitted.length != before_link.length
    {
        return Err(corrupt_pending(
            "published pending intent differs from anonymous inode",
        ));
    }
    exact_readback(&mut published, 0, bytes)?;
    exact_readback(anonymous, 0, bytes)?;
    if snapshot(&published)? != admitted {
        return Err(corrupt_pending(
            "published pending intent changed during read-back",
        ));
    }
    let path = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| corrupt_pending(format!("cannot restat pending intent: {error}")))?;
    if (path.st_dev, path.st_ino) != (admitted.device, admitted.inode)
        || path.st_uid != admitted.owner
        || path.st_nlink != admitted.links
        || path.st_mode & 0o7777 != admitted.mode
        || u64::try_from(path.st_size).ok() != Some(admitted.length)
    {
        return Err(corrupt_pending(
            "pending intent pathname changed after publication",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Metadata {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    links: u64,
    length: u64,
    ctime_seconds: i64,
    ctime_nanoseconds: u64,
}

fn snapshot(file: &File) -> Result<Metadata, CoordError> {
    use std::os::unix::fs::MetadataExt;

    let value = file.metadata().map_err(CoordError::io)?;
    if !value.is_file() {
        return Err(corrupt_pending("pending intent inode is not a file"));
    }
    Ok(Metadata {
        device: value.dev(),
        inode: value.ino(),
        owner: value.uid(),
        mode: value.mode() & 0o7777,
        links: value.nlink(),
        length: value.len(),
        ctime_seconds: value.ctime(),
        ctime_nanoseconds: value.ctime_nsec() as u64,
    })
}

fn validate_anonymous(value: Metadata, length: u64) -> Result<(), CoordError> {
    if value.owner != rustix::process::geteuid().as_raw()
        || value.mode != 0o600
        || value.links != 0
        || value.length != length
    {
        return Err(corrupt_pending(
            "anonymous pending intent owner, mode, link, or length is invalid",
        ));
    }
    Ok(())
}

#[cfg(test)]
fn write_or_crash(file: &mut File, bytes: &[u8]) -> Result<(), CoordError> {
    if let Some(offset) = CRASH_OFFSET.with(Cell::take) {
        if offset > bytes.len() {
            return Err(corrupt_pending("test crash offset exceeds intent length"));
        }
        file.write_all(&bytes[..offset]).map_err(CoordError::io)?;
        file.sync_data().map_err(CoordError::io)?;
        return Err(CoordError::new(
            "COORD_TEST_CRASH",
            "anonymous intent write interrupted before publication",
        ));
    }
    file.write_all(bytes).map_err(CoordError::io)
}

#[cfg(not(test))]
fn write_or_crash(file: &mut File, bytes: &[u8]) -> Result<(), CoordError> {
    file.write_all(bytes).map_err(CoordError::io)
}

#[cfg(test)]
fn crash_after_link() -> Result<(), CoordError> {
    if CRASH_AFTER_LINK.with(Cell::take) {
        Err(CoordError::new(
            "COORD_TEST_CRASH",
            "pending intent linked before parent fsync",
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(test))]
fn crash_after_link() -> Result<(), CoordError> {
    Ok(())
}

fn atomic_error(context: &str, error: impl std::fmt::Display) -> CoordError {
    CoordError::new(
        "COORD_ATOMIC_PUBLISH_UNSUPPORTED",
        format!("{context}: {error}"),
    )
}
