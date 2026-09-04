use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io::{Read, Seek, SeekFrom},
    os::unix::ffi::OsStringExt,
};

#[cfg(test)]
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

use rustix::fs::{AtFlags, Dir, Mode, OFlags, openat, unlinkat};

use super::{
    AppendReceipt, INTENT_NAME, MAX_FRAME_BYTES, MAX_INTENT_BYTES, SegmentEnvelope,
    SegmentInspection, SegmentPosition, StoredEnvelope,
    validate::{
        PendingIntent, canonical, capacity_error, corrupt, corrupt_pending, digest_record,
        envelope_digest,
    },
    validate_envelope,
};
use crate::coord::CoordError;

mod publish;

#[cfg(test)]
pub(super) use publish::{test_crash_after_link, test_crash_at_offset};

pub(super) fn pending_names(pending_dir: &File) -> Result<Vec<OsString>, CoordError> {
    validate_pending_descriptor(pending_dir)?;
    let mut directory = Dir::read_from(pending_dir).map_err(rustix_error)?;
    let mut names = Vec::new();
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(rustix_error)?;
        let bytes = entry.file_name().to_bytes();
        if bytes != b"." && bytes != b".." {
            names.push(OsString::from_vec(bytes.to_vec()));
        }
    }
    names.sort();
    Ok(names)
}

pub(super) fn inspect_bytes(
    bytes: &[u8],
    generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentInspection, CoordError> {
    let mut sequence = 1_u64;
    let mut previous_digest = genesis_digest.to_owned();
    let mut offset = 0_u64;
    let mut entries = Vec::new();
    let mut requests = BTreeMap::new();
    for frame in bytes.split_inclusive(|byte| *byte == b'\n') {
        if frame.last() != Some(&b'\n') {
            return Err(corrupt("segment ends with a non-LF partial frame"));
        }
        let canonical = &frame[..frame.len() - 1];
        if canonical.is_empty() || frame.len() > MAX_FRAME_BYTES {
            return Err(corrupt("segment contains an empty or oversized frame"));
        }
        let envelope: SegmentEnvelope = bullet_wire::decode_canonical(canonical)
            .map_err(|error| corrupt(format!("segment frame is not canonical: {error}")))?;
        validate_envelope(&envelope, generation_id)?;
        if envelope.sequence != sequence || envelope.previous_digest != previous_digest {
            return Err(corrupt("segment digest chain is discontinuous"));
        }
        let envelope_digest = envelope_digest(canonical)?;
        let frame_length = u64::try_from(frame.len()).map_err(|_| capacity_error())?;
        let receipt = AppendReceipt {
            sequence,
            envelope_digest: envelope_digest.clone(),
            record_digest: digest_record(&envelope.record)?,
            request_id: envelope.request_id.clone(),
            request_digest: envelope.request_digest.clone(),
            byte_offset: offset,
            frame_length,
        };
        if requests
            .insert(envelope.request_id.clone(), receipt.clone())
            .is_some()
        {
            return Err(corrupt("segment contains a duplicate request ID"));
        }
        entries.push(StoredEnvelope {
            generation_id: envelope.generation_id,
            sequence,
            previous_digest: envelope.previous_digest,
            request_id: envelope.request_id,
            request_digest: envelope.request_digest,
            record: envelope.record,
            receipt,
        });
        previous_digest = envelope_digest;
        sequence = sequence.checked_add(1).ok_or_else(capacity_error)?;
        offset = offset
            .checked_add(frame_length)
            .ok_or_else(capacity_error)?;
    }
    Ok(SegmentInspection {
        position: SegmentPosition {
            next_sequence: sequence,
            previous_digest,
            byte_length: offset,
        },
        entries,
        requests,
    })
}

#[cfg(test)]
pub(super) fn publish_intent(pending_dir: &Path, intent: &PendingIntent) -> Result<(), CoordError> {
    let pending = open_pending(pending_dir)?;
    publish_intent_at(&pending, intent)
}

pub(super) fn publish_intent_at(
    pending_dir: &File,
    intent: &PendingIntent,
) -> Result<(), CoordError> {
    validate_pending_descriptor(pending_dir)?;
    if !pending_names(pending_dir)?.is_empty() {
        return Err(CoordError::new(
            "PENDING_COORD_APPEND",
            "another append intent already requires reconciliation",
        ));
    }
    let bytes = canonical(intent, "pending append intent")?;
    if bytes.len() > MAX_INTENT_BYTES {
        return Err(capacity_error());
    }
    publish::publish(pending_dir, INTENT_NAME, &bytes)
}

pub(super) fn read_intent_at(pending_dir: &File) -> Result<PendingIntent, CoordError> {
    validate_pending_descriptor(pending_dir)?;
    let descriptor = openat(
        pending_dir,
        INTENT_NAME,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| corrupt_pending(format!("cannot open pending intent: {error}")))?;
    let mut file = File::from(descriptor);
    validate_regular_descriptor(&file, 0o600)?;
    decode_intent(&mut file)
}

pub(super) fn remove_intent_at(pending_dir: &File) -> Result<(), CoordError> {
    validate_pending_descriptor(pending_dir)?;
    unlinkat(pending_dir, INTENT_NAME, AtFlags::empty())
        .map_err(|error| corrupt_pending(format!("cannot retire pending intent: {error}")))?;
    pending_dir.sync_all().map_err(CoordError::io)
}

fn decode_intent(file: &mut File) -> Result<PendingIntent, CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let expected_length = file.metadata().map_err(CoordError::io)?.len();
    if expected_length > MAX_INTENT_BYTES as u64 {
        return Err(corrupt_pending("pending intent is oversized"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take((MAX_INTENT_BYTES as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() > MAX_INTENT_BYTES || bytes.len() as u64 != expected_length {
        return Err(corrupt_pending("pending intent changed length while read"));
    }
    let value = bullet_wire::decode_unique_value_bounded(&bytes, MAX_INTENT_BYTES)
        .map_err(|error| corrupt_pending(format!("pending intent is invalid JSON: {error}")))?;
    if canonical(&value, "pending append intent")? != bytes {
        return Err(corrupt_pending("pending intent is not canonical JSON"));
    }
    serde_json::from_value(value)
        .map_err(|error| corrupt_pending(format!("pending intent schema is invalid: {error}")))
}

#[cfg(test)]
pub(super) fn validate_pending_directory(path: &Path) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !metadata.file_type().is_dir() {
        return Err(corrupt_pending("pending path is not a real directory"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.mode() & 0o7777 != 0o700
            || metadata.nlink() < 2
            || metadata.uid() != rustix::process::geteuid().as_raw()
        {
            return Err(corrupt_pending(
                "pending directory owner, mode, or link identity is invalid",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn open_segment(path: &Path, writable: bool) -> Result<File, CoordError> {
    open_regular(path, writable, 0o600)
}

#[cfg(test)]
pub(super) fn open_pending(path: &Path) -> Result<File, CoordError> {
    validate_pending_directory(path)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC);
    }
    let file = options.open(path).map_err(CoordError::io)?;
    validate_pending_descriptor(&file)?;
    Ok(file)
}

pub(super) fn validate_segment_descriptor(file: &File) -> Result<(), CoordError> {
    validate_regular_descriptor(file, 0o600)
}

pub(super) fn validate_pending_descriptor(file: &File) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if !metadata.is_dir()
            || metadata.mode() & 0o7777 != 0o700
            || metadata.nlink() < 2
            || metadata.uid() != rustix::process::geteuid().as_raw()
        {
            return Err(corrupt_pending(
                "pending directory descriptor owner, mode, or link identity is invalid",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn open_regular(
    path: &Path,
    writable: bool,
    expected_mode: u32,
) -> Result<File, CoordError> {
    let before = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !before.file_type().is_file() {
        return Err(corrupt("coordination subject is not a regular file"));
    }
    let mut options = OpenOptions::new();
    options.read(true).write(writable);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC);
    }
    let file = options.open(path).map_err(CoordError::io)?;
    validate_regular(path, &file, expected_mode)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let opened = file.metadata().map_err(CoordError::io)?;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err(corrupt(
                "coordination subject changed identity while it was opened",
            ));
        }
    }
    Ok(file)
}

#[cfg(test)]
pub(super) fn validate_regular(
    path: &Path,
    file: &File,
    expected_mode: u32,
) -> Result<(), CoordError> {
    let path_metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    let file_metadata = file.metadata().map_err(CoordError::io)?;
    if !path_metadata.file_type().is_file() || !file_metadata.file_type().is_file() {
        return Err(corrupt("coordination subject changed file type"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if file_metadata.nlink() != 1
            || file_metadata.mode() & 0o7777 != expected_mode
            || file_metadata.uid() != rustix::process::geteuid().as_raw()
            || path_metadata.uid() != file_metadata.uid()
            || path_metadata.dev() != file_metadata.dev()
            || path_metadata.ino() != file_metadata.ino()
        {
            return Err(corrupt(
                "coordination subject owner, mode, link, or inode identity is invalid",
            ));
        }
    }
    Ok(())
}

fn validate_regular_descriptor(file: &File, expected_mode: u32) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if !metadata.file_type().is_file()
            || metadata.nlink() != 1
            || metadata.mode() & 0o7777 != expected_mode
            || metadata.uid() != rustix::process::geteuid().as_raw()
        {
            return Err(corrupt(
                "coordination descriptor owner, mode, link, or type is invalid",
            ));
        }
    }
    Ok(())
}

pub(super) fn read_bounded(file: &mut File, max: u64) -> Result<Vec<u8>, CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let before = descriptor_identity(file)?;
    let expected_length = file.metadata().map_err(CoordError::io)?.len();
    if expected_length > max {
        return Err(capacity_error());
    }
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = descriptor_identity(file)?;
    if bytes.len() as u64 > max
        || bytes.len() as u64 != expected_length
        || file.metadata().map_err(CoordError::io)?.len() != expected_length
        || before != after
    {
        return Err(capacity_error());
    }
    Ok(bytes)
}

pub(super) fn exact_readback(
    file: &mut File,
    offset: u64,
    expected: &[u8],
) -> Result<(), CoordError> {
    file.seek(SeekFrom::Start(offset)).map_err(CoordError::io)?;
    let mut actual = vec![0_u8; expected.len()];
    file.read_exact(&mut actual).map_err(CoordError::io)?;
    if actual != expected {
        return Err(CoordError::new(
            "PARTIAL_COORD_WRITE",
            "coordination file read-back differs from intended bytes",
        ));
    }
    let end = offset
        .checked_add(expected.len() as u64)
        .ok_or_else(capacity_error)?;
    if file.metadata().map_err(CoordError::io)?.len() != end {
        return Err(CoordError::new(
            "PARTIAL_COORD_WRITE",
            "coordination file contains bytes beyond the intended end",
        ));
    }
    Ok(())
}

fn rustix_error(error: rustix::io::Errno) -> CoordError {
    CoordError::io(std::io::Error::from_raw_os_error(error.raw_os_error()))
}

fn descriptor_identity(file: &File) -> Result<(u64, u64), CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = file.metadata().map_err(CoordError::io)?;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    Err(corrupt("segment descriptor identity is unsupported"))
}
