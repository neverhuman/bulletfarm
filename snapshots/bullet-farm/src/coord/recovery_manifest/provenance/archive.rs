use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};

use sha2::{Digest, Sha256};
use tar::{Archive, EntryType};

use super::TreeBlob;
use crate::coord::{CoordError, RecoveryBootstrapProvenanceV1};

pub(in crate::coord::recovery_manifest) const MAX_ARCHIVE_BYTES: usize = 256 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_FILES: usize = 8_192;
pub(in crate::coord::recovery_manifest) const MAX_ENTRIES: usize = 16_384;
const MAX_PATH_BYTES: usize = 512;
const MAX_METADATA_BYTES: u64 = 4 * 1024;

pub(in crate::coord::recovery_manifest) struct ArchiveObservation {
    pub(super) source_files: Vec<(String, u64, String)>,
    pub(super) cargo_lock_sha256: String,
    pub(super) rust_toolchain: Vec<u8>,
}

pub(super) fn inspect(
    bytes: &[u8],
    commit_oid: &str,
    tree: &BTreeMap<String, TreeBlob>,
) -> Result<ArchiveObservation, CoordError> {
    validate_archive_length(bytes.len())?;
    verify_raw_archive(bytes, commit_oid)?;
    inspect_logical_archive(bytes, tree)
}

pub(in crate::coord::recovery_manifest) fn verify_retained_source(
    bytes: &[u8],
    provenance: &RecoveryBootstrapProvenanceV1,
) -> Result<(), CoordError> {
    provenance.validate()?;
    validate_archive_length(bytes.len())?;
    verify_raw_archive(bytes, &provenance.bootstrap_commit_oid)?;
    let mut tree = BTreeMap::new();
    let mut archive = Archive::new(Cursor::new(bytes));
    for item in archive.entries().map_err(tar_error)? {
        let entry = item.map_err(tar_error)?;
        if entry.header().entry_type() != EntryType::Regular {
            continue;
        }
        let path = repository_path(&entry.path_bytes(), false)?;
        let mode = entry.header().mode().map_err(tar_error)? & 0o7777;
        if !matches!(mode, 0o664 | 0o775)
            || tree
                .insert(
                    path,
                    TreeBlob {
                        mode,
                        oid: String::new(),
                    },
                )
                .is_some()
        {
            return Err(invalid("retained source TAR mode or path is invalid"));
        }
    }
    let observed = inspect_logical_archive(bytes, &tree)?;
    let expected = provenance
        .source_files
        .iter()
        .map(|file| (file.path.clone(), file.byte_length, file.sha256.clone()))
        .collect::<Vec<_>>();
    if observed.source_files != expected
        || observed.cargo_lock_sha256 != provenance.cargo_lock_sha256
    {
        return Err(invalid("retained source inventory or Cargo.lock differs"));
    }
    Ok(())
}

pub(in crate::coord::recovery_manifest) fn validate_archive_length(
    length: usize,
) -> Result<(), CoordError> {
    if length == 0 || length > MAX_ARCHIVE_BYTES || !length.is_multiple_of(512) {
        return Err(invalid(
            "Git TAR archive is empty, oversized, or not block aligned",
        ));
    }
    Ok(())
}

pub(in crate::coord::recovery_manifest) fn verify_raw_archive(
    bytes: &[u8],
    commit_oid: &str,
) -> Result<(), CoordError> {
    let mut archive = Archive::new(Cursor::new(bytes));
    let entries = archive.entries().map_err(tar_error)?.raw(true);
    let mut next_header = 0_u64;
    let mut global_seen = false;
    let mut pending_path_extension = false;
    let mut entry_count = 0_usize;
    for item in entries {
        increment_entry_count(&mut entry_count)?;
        let mut entry = item.map_err(tar_error)?;
        if entry.raw_header_position() != next_header {
            return Err(invalid("Git TAR contains a gap or overlapping entry"));
        }
        let file_end = entry
            .raw_file_position()
            .checked_add(entry.size())
            .ok_or_else(|| invalid("Git TAR entry range overflowed"))?;
        let padded_end = file_end
            .checked_add(511)
            .ok_or_else(|| invalid("Git TAR padded range overflowed"))?
            & !511;
        let padding = usize::try_from(file_end)
            .ok()
            .zip(usize::try_from(padded_end).ok())
            .and_then(|(start, end)| bytes.get(start..end))
            .ok_or_else(|| invalid("Git TAR padded range is outside the archive"))?;
        if padding.iter().any(|byte| *byte != 0) {
            return Err(invalid("Git TAR entry padding is not zero"));
        }
        next_header = padded_end;
        match entry.header().entry_type() {
            EntryType::XGlobalHeader if !global_seen && entry.raw_header_position() == 0 => {
                if pending_path_extension {
                    return Err(invalid("Git TAR path extension has no file entry"));
                }
                let content = read_metadata(&mut entry)?;
                let records = pax_records(&content)?;
                if records.len() != 1
                    || records[0].0 != "comment"
                    || records[0].1.as_slice() != commit_oid.as_bytes()
                {
                    return Err(invalid(
                        "Git TAR global commit marker does not bind the selected commit",
                    ));
                }
                global_seen = true;
            }
            EntryType::XHeader => {
                if pending_path_extension {
                    return Err(invalid("Git TAR stacks multiple path extensions"));
                }
                let content = read_metadata(&mut entry)?;
                let records = pax_records(&content)?;
                if records.len() != 1 || records[0].0 != "path" {
                    return Err(invalid(
                        "Git TAR local PAX metadata may only carry one path",
                    ));
                }
                repository_path(&records[0].1, records[0].1.ends_with(b"/"))?;
                pending_path_extension = true;
            }
            EntryType::GNULongName => {
                if pending_path_extension {
                    return Err(invalid("Git TAR stacks multiple path extensions"));
                }
                let content = read_metadata(&mut entry)?;
                let path = content
                    .strip_suffix(&[0])
                    .ok_or_else(|| invalid("GNU long path lacks its terminal NUL"))?;
                if path.contains(&0) {
                    return Err(invalid("GNU long path contains an interior NUL"));
                }
                repository_path(path, path.ends_with(b"/"))?;
                pending_path_extension = true;
            }
            EntryType::Regular | EntryType::Directory => {
                let directory = entry.header().entry_type() == EntryType::Directory;
                repository_path(&entry.header().path_bytes(), directory)?;
                pending_path_extension = false;
            }
            _ => {
                return Err(invalid(
                    "Git TAR contains a link, sparse, special, or unknown entry",
                ));
            }
        }
    }
    if !global_seen {
        return Err(invalid("Git TAR omits its global commit marker"));
    }
    if pending_path_extension {
        return Err(invalid("Git TAR path extension has no file entry"));
    }
    let end = usize::try_from(next_header)
        .map_err(|_| invalid("Git TAR range does not fit this host"))?;
    if end > bytes.len() || bytes.len() - end < 1_024 || bytes[end..].iter().any(|byte| *byte != 0)
    {
        return Err(invalid(
            "Git TAR lacks a complete zero trailer or carries trailing data",
        ));
    }
    Ok(())
}

pub(in crate::coord::recovery_manifest) fn inspect_logical_archive(
    bytes: &[u8],
    tree: &BTreeMap<String, TreeBlob>,
) -> Result<ArchiveObservation, CoordError> {
    let mut archive = Archive::new(Cursor::new(bytes));
    let entries = archive.entries().map_err(tar_error)?;
    let mut seen = BTreeSet::new();
    let mut directories = BTreeSet::new();
    let mut source_files = Vec::new();
    let mut aggregate = 0_u64;
    let mut cargo_lock_sha256 = None;
    let mut rust_toolchain = None;
    let mut entry_count = 0_usize;
    for item in entries {
        increment_entry_count(&mut entry_count)?;
        let mut entry = item.map_err(tar_error)?;
        let entry_type = entry.header().entry_type();
        if entry_type == EntryType::XGlobalHeader {
            continue;
        }
        let is_directory = entry_type == EntryType::Directory;
        if entry_type != EntryType::Regular && !is_directory {
            return Err(invalid("Git TAR logical entry has an unadmitted type"));
        }
        if entry.link_name_bytes().is_some() {
            return Err(invalid("Git TAR logical entry carries a link target"));
        }
        let path = repository_path(&entry.path_bytes(), is_directory)?;
        if !seen.insert(path.clone()) {
            return Err(invalid("Git TAR repeats a logical repository path"));
        }
        if is_directory {
            let mode = entry.header().mode().map_err(tar_error)? & 0o7777;
            if entry.size() != 0 || mode != 0o775 {
                return Err(invalid(
                    "Git TAR directory must have zero content and exact mode 0775",
                ));
            }
            directories.insert(path);
            continue;
        }
        let expected_mode = tree
            .get(&path)
            .ok_or_else(|| invalid("Git TAR contains a file absent from the commit tree"))?;
        let mode = entry.header().mode().map_err(tar_error)? & 0o7777;
        if mode != expected_mode.mode {
            return Err(invalid("Git TAR file mode differs from the commit tree"));
        }
        let length = entry.size();
        if length == 0 {
            return Err(invalid(
                "RecoveryBootstrapProvenanceV1 cannot represent a zero-byte source blob",
            ));
        }
        if length > MAX_FILE_BYTES {
            return Err(invalid("Git TAR source file exceeds 64 MiB"));
        }
        aggregate = aggregate
            .checked_add(length)
            .filter(|total| *total <= MAX_ARCHIVE_BYTES as u64)
            .ok_or_else(|| invalid("Git TAR expanded sources exceed 256 MiB"))?;
        let mut content = Vec::new();
        (&mut entry)
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut content)
            .map_err(tar_error)?;
        if content.len() as u64 != length {
            return Err(invalid("Git TAR source length changed while reading"));
        }
        let sha256 = format!("sha256:{:x}", Sha256::digest(&content));
        if path == "Cargo.lock" {
            cargo_lock_sha256 = Some(sha256.clone());
        }
        if path == "rust-toolchain.toml" {
            rust_toolchain = Some(content.clone());
        }
        source_files.push((path, length, sha256));
        if source_files.len() > MAX_FILES {
            return Err(invalid("Git TAR exceeds 8,192 source files"));
        }
    }
    let archive_paths = source_files
        .iter()
        .map(|source| source.0.as_str())
        .collect::<BTreeSet<_>>();
    let tree_paths = tree.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if archive_paths != tree_paths {
        return Err(invalid(
            "Git TAR file set differs from the complete commit tree (including export-ignore)",
        ));
    }
    if directories != tree_directories(tree) {
        return Err(invalid(
            "Git TAR directory set differs from the commit tree prefixes",
        ));
    }
    source_files.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let cargo_lock_sha256 = cargo_lock_sha256
        .ok_or_else(|| invalid("Git TAR must contain one nonempty top-level Cargo.lock"))?;
    let rust_toolchain =
        rust_toolchain.ok_or_else(|| invalid("Git TAR must contain rust-toolchain.toml"))?;
    Ok(ArchiveObservation {
        source_files,
        cargo_lock_sha256,
        rust_toolchain,
    })
}

fn tree_directories(tree: &BTreeMap<String, TreeBlob>) -> BTreeSet<String> {
    let mut directories = BTreeSet::new();
    for path in tree.keys() {
        let mut prefix = String::new();
        let components = path.split('/').collect::<Vec<_>>();
        for component in &components[..components.len() - 1] {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            directories.insert(prefix.clone());
        }
    }
    directories
}

fn read_metadata<R: Read>(entry: &mut tar::Entry<'_, R>) -> Result<Vec<u8>, CoordError> {
    if entry.size() == 0 || entry.size() > MAX_METADATA_BYTES {
        return Err(invalid("Git TAR metadata entry is empty or oversized"));
    }
    let mut content = Vec::new();
    entry
        .take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut content)
        .map_err(tar_error)?;
    if content.len() as u64 != entry.size() {
        return Err(invalid("Git TAR metadata length changed while reading"));
    }
    Ok(content)
}

pub(in crate::coord::recovery_manifest) fn pax_records(
    bytes: &[u8],
) -> Result<Vec<(String, Vec<u8>)>, CoordError> {
    let mut records = Vec::new();
    let mut keys = BTreeSet::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        let space = remaining
            .iter()
            .position(|byte| *byte == b' ')
            .ok_or_else(|| invalid("Git TAR PAX record lacks its length separator"))?;
        let length_text = std::str::from_utf8(&remaining[..space])
            .map_err(|_| invalid("Git TAR PAX record length is not ASCII"))?;
        let length = usize::try_from(decimal(
            length_text.as_bytes(),
            "Git TAR PAX record length",
        )?)
        .map_err(|_| invalid("Git TAR PAX record length does not fit this host"))?;
        let record = remaining
            .get(..length)
            .filter(|record| record.last() == Some(&b'\n'))
            .ok_or_else(|| invalid("Git TAR PAX record is truncated or lacks its LF"))?;
        let payload = record
            .get(space + 1..length - 1)
            .ok_or_else(|| invalid("Git TAR PAX record length is contradictory"))?;
        let equals = payload
            .iter()
            .position(|byte| *byte == b'=')
            .filter(|equals| *equals > 0)
            .ok_or_else(|| invalid("Git TAR PAX record lacks a nonempty key"))?;
        let key = std::str::from_utf8(&payload[..equals])
            .map_err(|_| invalid("Git TAR PAX key is not UTF-8"))?
            .to_owned();
        if !keys.insert(key.clone()) {
            return Err(invalid("Git TAR PAX metadata repeats a key"));
        }
        records.push((key, payload[equals + 1..].to_vec()));
        offset = offset
            .checked_add(length)
            .ok_or_else(|| invalid("Git TAR PAX offset overflowed"))?;
    }
    if records.is_empty() {
        return Err(invalid("Git TAR PAX metadata is empty"));
    }
    Ok(records)
}

pub(in crate::coord::recovery_manifest) fn verify_blob_batch(
    bytes: &[u8],
    tree: &BTreeMap<String, TreeBlob>,
    sources: &[(String, u64, String)],
) -> Result<(), CoordError> {
    if tree.len() != sources.len() {
        return Err(invalid("Git blob batch and archive inventory differ"));
    }
    let mut offset = 0;
    for ((path, blob), (source_path, source_length, source_sha256)) in tree.iter().zip(sources) {
        if path != source_path {
            return Err(invalid("Git blob and archive path order differs"));
        }
        let newline = bytes[offset..]
            .iter()
            .position(|byte| *byte == b'\n')
            .filter(|length| *length <= 200)
            .ok_or_else(|| invalid("Git blob batch header is missing or oversized"))?;
        let header = std::str::from_utf8(&bytes[offset..offset + newline])
            .map_err(|_| invalid("Git blob batch header is not UTF-8"))?;
        let fields = header.split(' ').collect::<Vec<_>>();
        let length = decimal(
            fields
                .get(2)
                .ok_or_else(|| invalid("Git blob batch length is missing"))?
                .as_bytes(),
            "Git blob batch length",
        )?;
        if fields.len() != 3
            || fields[0] != blob.oid
            || fields[1] != "blob"
            || length != *source_length
        {
            return Err(invalid("Git blob batch header differs from tree/archive"));
        }
        let start = offset + newline + 1;
        let length = usize::try_from(length)
            .map_err(|_| invalid("Git blob batch length does not fit this host"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid("Git blob batch range overflowed"))?;
        let content = bytes
            .get(start..end)
            .ok_or_else(|| invalid("Git blob batch content is truncated"))?;
        if bytes.get(end) != Some(&b'\n')
            || format!("sha256:{:x}", Sha256::digest(content)) != *source_sha256
        {
            return Err(invalid("Git blob content differs from the TAR source"));
        }
        offset = end + 1;
    }
    if offset != bytes.len() {
        return Err(invalid("Git blob batch carries trailing output"));
    }
    Ok(())
}

pub(super) fn repository_path(bytes: &[u8], directory: bool) -> Result<String, CoordError> {
    let bytes = if directory {
        bytes
            .strip_suffix(b"/")
            .ok_or_else(|| invalid("Git TAR directory path lacks its terminal slash"))?
    } else {
        if bytes.ends_with(b"/") {
            return Err(invalid("Git TAR file path has a directory suffix"));
        }
        bytes
    };
    if bytes.is_empty() || bytes.len() > MAX_PATH_BYTES {
        return Err(invalid(
            "repository source path is empty or exceeds 512 bytes",
        ));
    }
    let path =
        std::str::from_utf8(bytes).map_err(|_| invalid("repository source path is not UTF-8"))?;
    let path = crate::coord::validate_path(path)
        .map_err(|error| invalid(format!("invalid repository source path: {error}")))?;
    if path == "." {
        return Err(invalid("repository source path cannot name the root"));
    }
    Ok(path)
}

pub(in crate::coord::recovery_manifest) fn increment_entry_count(
    count: &mut usize,
) -> Result<(), CoordError> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| invalid("Git TAR entry count overflowed"))?;
    if *count > MAX_ENTRIES {
        return Err(invalid("Git TAR exceeds 16,384 total entries"));
    }
    Ok(())
}

fn decimal(bytes: &[u8], label: &str) -> Result<u64, CoordError> {
    if bytes.is_empty()
        || !bytes.iter().all(u8::is_ascii_digit)
        || (bytes.len() > 1 && bytes[0] == b'0')
    {
        return Err(invalid(format!("{label} is not canonical ASCII decimal")));
    }
    bytes.iter().try_fold(0_u64, |value, byte| {
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(byte - b'0')))
            .ok_or_else(|| invalid(format!("{label} overflowed")))
    })
}

fn tar_error(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("invalid Git TAR archive: {error}"))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}
