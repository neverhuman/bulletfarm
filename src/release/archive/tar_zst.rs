//! Strict TAR+Zstandard reader for deterministic release materialization.

use std::{
    fs::File,
    io::{BufRead, Read},
    path::Path,
};

use super::{
    ArchivePlan, EntryKind, MAX_ENTRIES, MAX_ENTRY_BYTES, MAX_EXPANDED_BYTES, MAX_EXPANSION_RATIO,
    MIN_RATIO_ALLOWANCE, RawEntry, invalid_archive, limit, materialize_entry,
};
use crate::coord::CoordError;

pub(super) fn scan(input: File) -> Result<Vec<RawEntry>, CoordError> {
    let mut budget = ScanBudget::new(input.metadata().map_err(CoordError::io)?.len());
    let mut entries = Vec::new();
    visit(input, |reader, entry| {
        budget.admit(entry.size)?;
        drain_entry(reader, entry.size)?;
        entries.push(entry);
        Ok(())
    })?;
    Ok(entries)
}

pub(super) fn materialize(
    input: File,
    plan: &ArchivePlan,
    output: &Path,
) -> Result<(), CoordError> {
    let mut index = 0_usize;
    visit(input, |reader, entry| {
        let expected = plan
            .entries()
            .get(index)
            .ok_or_else(|| invalid_archive("archive gained an entry after admission"))?;
        materialize_entry(reader, &entry, expected, output, &plan.executable)?;
        index += 1;
        Ok(())
    })?;
    if index != plan.entries().len() {
        return Err(invalid_archive("archive lost an entry after admission"));
    }
    Ok(())
}

fn visit(
    input: File,
    mut visit_entry: impl FnMut(&mut dyn Read, RawEntry) -> Result<(), CoordError>,
) -> Result<(), CoordError> {
    let decoded_limit = input
        .metadata()
        .map_err(CoordError::io)?
        .len()
        .saturating_mul(MAX_EXPANSION_RATIO)
        .clamp(MIN_RATIO_ALLOWANCE, MAX_EXPANDED_BYTES);
    let decoder = zstd::stream::read::Decoder::new(input)
        .map_err(|error| invalid_archive(format!("invalid Zstandard stream: {error}")))?
        .single_frame();
    let mut archive = tar::Archive::new(BoundedReader::new(decoder, decoded_limit));
    {
        let entries = archive
            .entries()
            .map_err(|error| invalid_archive(format!("invalid TAR archive: {error}")))?
            .raw(true);
        for entry in entries {
            let mut entry =
                entry.map_err(|error| invalid_archive(format!("invalid TAR entry: {error}")))?;
            let entry_type = entry.header().entry_type();
            let kind = if entry_type.is_file() {
                EntryKind::File
            } else if entry_type.is_dir() {
                EntryKind::Directory
            } else {
                return Err(invalid_archive(format!(
                    "TAR entry type {entry_type:?} is forbidden"
                )));
            };
            let raw = RawEntry {
                name: entry.path_bytes().into_owned(),
                kind,
                size: entry.size(),
            };
            visit_entry(&mut entry, raw)?;
        }
    }
    let mut bounded = archive.into_inner();
    let mut trailing = [0_u8; 8192];
    loop {
        let count = bounded
            .read(&mut trailing)
            .map_err(|error| invalid_archive(format!("invalid TAR trailer: {error}")))?;
        if count == 0 {
            break;
        }
        if trailing[..count].iter().any(|byte| *byte != 0) {
            return Err(invalid_archive(
                "TAR archive contains nonzero bytes after its end marker",
            ));
        }
    }
    let decoder = bounded.into_inner();
    let mut compressed = decoder.finish();
    if !compressed
        .fill_buf()
        .map_err(|error| invalid_archive(format!("invalid Zstandard trailer: {error}")))?
        .is_empty()
    {
        return Err(invalid_archive(
            "Zstandard archive contains a trailing frame or byte suffix",
        ));
    }
    Ok(())
}

struct BoundedReader<R> {
    inner: R,
    consumed: u64,
    limit: u64,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, limit: u64) -> Self {
        Self {
            inner,
            consumed: 0,
            limit,
        }
    }

    fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.consumed == self.limit {
            let mut extra = [0_u8; 1];
            return match self.inner.read(&mut extra)? {
                0 => Ok(0),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "decompressed TAR exceeds its byte limit",
                )),
            };
        }
        let admitted = usize::try_from((self.limit - self.consumed).min(buffer.len() as u64))
            .expect("buffer-sized read fits usize");
        let count = self.inner.read(&mut buffer[..admitted])?;
        self.consumed += count as u64;
        Ok(count)
    }
}

fn drain_entry(reader: &mut dyn Read, size: u64) -> Result<(), CoordError> {
    let copied = std::io::copy(reader, &mut std::io::sink())
        .map_err(|error| invalid_archive(format!("could not read TAR entry: {error}")))?;
    if copied != size {
        return Err(invalid_archive(
            "TAR entry bytes differ from its declared size",
        ));
    }
    Ok(())
}

struct ScanBudget {
    count: usize,
    expanded: u64,
    limit: u64,
}

impl ScanBudget {
    fn new(archive_size: u64) -> Self {
        Self {
            count: 0,
            expanded: 0,
            limit: archive_size
                .saturating_mul(MAX_EXPANSION_RATIO)
                .clamp(MIN_RATIO_ALLOWANCE, MAX_EXPANDED_BYTES),
        }
    }

    fn admit(&mut self, size: u64) -> Result<(), CoordError> {
        self.count += 1;
        self.expanded = self
            .expanded
            .checked_add(size)
            .ok_or_else(|| limit("TAR expanded size overflowed"))?;
        if self.count > MAX_ENTRIES || size > MAX_ENTRY_BYTES || self.expanded > self.limit {
            return Err(limit(
                "TAR entry count, entry bytes, or compression ratio exceeds its bound",
            ));
        }
        Ok(())
    }
}
