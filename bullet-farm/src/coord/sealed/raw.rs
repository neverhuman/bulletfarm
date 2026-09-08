//! Bounded binary artifacts used by credential-free recovery authoring.

use std::path::Path;

use crate::coord::CoordError;

/// Publish exact nonempty bytes once beneath an admitted private parent.
///
/// The shared publisher retains an anonymous descriptor through sync, link,
/// and independent readback, and publishes a single mode-0400 link. The caller
/// supplies the artifact-specific bound; no newline or text transformation is
/// performed here.
pub(crate) fn write_raw(path: &Path, bytes: &[u8], maximum: u64) -> Result<(), CoordError> {
    super::write_bytes(path, bytes, maximum)
}

/// Reconcile exact sealed bytes only after synchronizing their retained file and parent.
pub(crate) fn read_synced_raw(path: &Path, maximum: u64) -> Result<Vec<u8>, CoordError> {
    use rustix::fs::{Mode, OFlags, openat};
    use std::{fs::File, io::Read};

    if maximum == 0 || maximum == u64::MAX {
        return Err(super::invalid("sealed durability read bound is invalid"));
    }
    let parent = super::Parent::open(path, super::ParentAdmission::Sealed)?;
    let descriptor = openat(
        &parent.file,
        &parent.name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| super::invalid(format!("cannot retain sealed document for sync: {error}")))?;
    let mut file = File::from(descriptor);
    let identity = super::Identity::for_file(&file)?;
    identity.validate_file(1, maximum, super::ParentAdmission::Sealed)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() as u64 != identity.length || super::Identity::for_file(&file)? != identity {
        return Err(super::changed(
            "sealed document changed before durability sync",
        ));
    }
    file.sync_all().map_err(CoordError::io)?;
    sync_parent(&parent.file)?;
    parent.verify_published(&mut file, &bytes, maximum, identity)?;
    parent.revalidate_path(path)?;
    Ok(bytes)
}

pub(super) fn sync_parent(parent: &std::fs::File) -> Result<(), CoordError> {
    #[cfg(test)]
    if DIRECTORY_SYNC_FAILED.get() {
        return Err(super::invalid("injected persistent directory sync failure"));
    }
    parent.sync_all().map_err(CoordError::io)
}

#[cfg(test)]
thread_local! {
    static DIRECTORY_SYNC_FAILED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn with_directory_sync_failure<T>(operation: impl FnOnce() -> T) -> T {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            DIRECTORY_SYNC_FAILED.set(self.0);
        }
    }
    let _reset = Reset(DIRECTORY_SYNC_FAILED.replace(true));
    operation()
}
