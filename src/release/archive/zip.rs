//! Strict stored-ZIP reader for the Windows release package.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use super::{
    ArchivePlan, EntryKind, MAX_ENTRIES, MAX_ENTRY_BYTES, MAX_EXPANDED_BYTES, MAX_EXPANSION_RATIO,
    MIN_RATIO_ALLOWANCE, RawEntry, invalid_archive, limit, materialize_entry,
};
use crate::coord::CoordError;

pub(super) fn scan(mut input: File) -> Result<Vec<RawEntry>, CoordError> {
    admit_raw_layout(&mut input)?;
    let archive_size = input.metadata().map_err(CoordError::io)?.len();
    let mut budget = ScanBudget::new(archive_size);
    let mut archive = open(input)?;
    admit_container(&archive)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| invalid_archive(format!("invalid ZIP entry: {error}")))?;
        let raw = admit_file(&file)?;
        budget.admit(raw.size)?;
        let copied = std::io::copy(&mut file, &mut std::io::sink())
            .map_err(|error| invalid_archive(format!("could not read ZIP entry: {error}")))?;
        if copied != raw.size {
            return Err(invalid_archive(
                "ZIP entry bytes differ from its declared size",
            ));
        }
        entries.push(raw);
    }
    Ok(entries)
}

pub(super) fn materialize(
    mut input: File,
    plan: &ArchivePlan,
    output: &Path,
) -> Result<(), CoordError> {
    admit_raw_layout(&mut input)?;
    let mut archive = open(input)?;
    admit_container(&archive)?;
    if archive.len() != plan.entries().len() {
        return Err(invalid_archive(
            "ZIP archive entry count changed after admission",
        ));
    }
    for (index, expected) in plan.entries().iter().enumerate() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| invalid_archive(format!("invalid ZIP entry: {error}")))?;
        let raw = admit_file(&file)?;
        materialize_entry(&mut file, &raw, expected, output, &plan.executable)?;
    }
    Ok(())
}

fn open(input: File) -> Result<::zip::ZipArchive<File>, CoordError> {
    ::zip::ZipArchive::new(input)
        .map_err(|error| invalid_archive(format!("invalid ZIP archive: {error}")))
}

fn admit_container(archive: &::zip::ZipArchive<File>) -> Result<(), CoordError> {
    if archive.is_empty() || archive.len() > MAX_ENTRIES {
        return Err(limit("ZIP entry count is outside the admitted bound"));
    }
    if archive.offset() != 0 || !archive.comment().is_empty() {
        return Err(invalid_archive(
            "ZIP prefix offsets and archive comments are forbidden",
        ));
    }
    Ok(())
}

fn admit_file(file: &::zip::read::ZipFile<'_>) -> Result<RawEntry, CoordError> {
    if file.encrypted() {
        return Err(invalid_archive("encrypted ZIP entries are forbidden"));
    }
    if file.compression() != ::zip::CompressionMethod::Stored {
        return Err(invalid_archive(
            "only stored ZIP entries are admitted for deterministic extraction",
        ));
    }
    if file.compressed_size() != file.size()
        || !file.comment().is_empty()
        || file.extra_data().is_some_and(|extra| !extra.is_empty())
    {
        return Err(invalid_archive(
            "ZIP entry size, comment, or extra metadata is not canonical",
        ));
    }
    let kind = if file.is_dir() {
        EntryKind::Directory
    } else if file.is_symlink() {
        return Err(invalid_archive("ZIP symbolic links are forbidden"));
    } else {
        EntryKind::File
    };
    if let Some(mode) = file.unix_mode() {
        let file_type = mode & 0o170000;
        let expected_type = match kind {
            EntryKind::Directory => 0o040000,
            EntryKind::File => 0o100000,
        };
        if file_type != 0 && file_type != expected_type {
            return Err(invalid_archive(
                "ZIP device, FIFO, socket, and other special entries are forbidden",
            ));
        }
    }
    Ok(RawEntry {
        name: file.name_raw().to_vec(),
        kind,
        size: file.size(),
    })
}

fn admit_raw_layout(input: &mut File) -> Result<(), CoordError> {
    let length = input.metadata().map_err(CoordError::io)?.len();
    if length < 22 {
        return Err(invalid_archive(
            "ZIP archive is shorter than its end record",
        ));
    }
    input.seek(SeekFrom::End(-22)).map_err(CoordError::io)?;
    let mut eocd = [0_u8; 22];
    input
        .read_exact(&mut eocd)
        .map_err(|error| invalid_archive(format!("invalid ZIP end record: {error}")))?;
    if &eocd[..4] != b"PK\x05\x06"
        || u16_at(&eocd, 4) != 0
        || u16_at(&eocd, 6) != 0
        || u16_at(&eocd, 8) == 0
        || u16_at(&eocd, 8) != u16_at(&eocd, 10)
        || u16_at(&eocd, 8) == u16::MAX
        || u16_at(&eocd, 20) != 0
        || u32_at(&eocd, 12) == u32::MAX
        || u32_at(&eocd, 16) == u32::MAX
    {
        return Err(invalid_archive(
            "ZIP end record must be single-disk, non-ZIP64, comment-free, and exact",
        ));
    }
    let central_offset = u64::from(u32_at(&eocd, 16));
    let central_size = u64::from(u32_at(&eocd, 12));
    let end_offset = length - 22;
    if central_offset.checked_add(central_size) != Some(end_offset) {
        return Err(invalid_archive(
            "ZIP central directory does not end exactly at its end record",
        ));
    }
    let count = usize::from(u16_at(&eocd, 10));
    if count > MAX_ENTRIES {
        return Err(limit("ZIP entry count exceeds its bound"));
    }
    let central = admit_central_directory(input, central_offset, central_size, count)?;
    admit_local_headers(input, central_offset, &central)?;
    input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(())
}

#[derive(Debug)]
struct CentralEntry {
    name: Vec<u8>,
    crc32: u32,
    size: u32,
    local_offset: u32,
}

fn admit_central_directory(
    input: &mut File,
    offset: u64,
    size: u64,
    count: usize,
) -> Result<Vec<CentralEntry>, CoordError> {
    input
        .seek(SeekFrom::Start(offset))
        .map_err(CoordError::io)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let fixed = read_exact::<46>(input, "ZIP central header")?;
        let name_length = usize::from(u16_at(&fixed, 28));
        if &fixed[..4] != b"PK\x01\x02"
            || u16_at(&fixed, 8) != 0
            || u16_at(&fixed, 10) != 0
            || u16_at(&fixed, 30) != 0
            || u16_at(&fixed, 32) != 0
            || u16_at(&fixed, 34) != 0
            || u32_at(&fixed, 20) == u32::MAX
            || u32_at(&fixed, 24) == u32::MAX
            || u32_at(&fixed, 42) == u32::MAX
            || u32_at(&fixed, 20) != u32_at(&fixed, 24)
            || name_length == 0
            || name_length > super::MAX_PATH_BYTES
        {
            return Err(invalid_archive(
                "ZIP central header uses flags, ZIP64, compression, metadata, or sizes outside the canonical subset",
            ));
        }
        let mut name = vec![0_u8; name_length];
        input
            .read_exact(&mut name)
            .map_err(|error| invalid_archive(format!("invalid ZIP central name: {error}")))?;
        entries.push(CentralEntry {
            name,
            crc32: u32_at(&fixed, 16),
            size: u32_at(&fixed, 24),
            local_offset: u32_at(&fixed, 42),
        });
    }
    let actual_end = input.stream_position().map_err(CoordError::io)?;
    if actual_end != offset + size {
        return Err(invalid_archive(
            "ZIP central directory contains a gap, suffix, or uncounted record",
        ));
    }
    Ok(entries)
}

fn admit_local_headers(
    input: &mut File,
    central_offset: u64,
    central: &[CentralEntry],
) -> Result<(), CoordError> {
    let mut next_offset = 0_u64;
    for expected in central {
        let offset = u64::from(expected.local_offset);
        if offset != next_offset {
            return Err(invalid_archive(
                "ZIP local records must be contiguous and ordered like the central directory",
            ));
        }
        input
            .seek(SeekFrom::Start(offset))
            .map_err(CoordError::io)?;
        let fixed = read_exact::<30>(input, "ZIP local header")?;
        let name_length = usize::from(u16_at(&fixed, 26));
        if &fixed[..4] != b"PK\x03\x04"
            || u16_at(&fixed, 6) != 0
            || u16_at(&fixed, 8) != 0
            || u16_at(&fixed, 28) != 0
            || u32_at(&fixed, 14) != expected.crc32
            || u32_at(&fixed, 18) != expected.size
            || u32_at(&fixed, 22) != expected.size
            || name_length != expected.name.len()
        {
            return Err(invalid_archive(
                "ZIP local header differs from its exact stored central subject",
            ));
        }
        let mut name = vec![0_u8; name_length];
        input
            .read_exact(&mut name)
            .map_err(|error| invalid_archive(format!("invalid ZIP local name: {error}")))?;
        if name != expected.name {
            return Err(invalid_archive("ZIP local and central names differ"));
        }
        next_offset = offset
            .checked_add(30)
            .and_then(|value| value.checked_add(name_length as u64))
            .and_then(|value| value.checked_add(u64::from(expected.size)))
            .ok_or_else(|| limit("ZIP local record offset overflowed"))?;
        if next_offset > central_offset {
            return Err(invalid_archive(
                "ZIP local payload overlaps the central directory",
            ));
        }
    }
    if next_offset != central_offset {
        return Err(invalid_archive(
            "ZIP local records leave an unadmitted gap before the central directory",
        ));
    }
    Ok(())
}

fn read_exact<const SIZE: usize>(input: &mut File, label: &str) -> Result<[u8; SIZE], CoordError> {
    let mut bytes = [0_u8; SIZE];
    input
        .read_exact(&mut bytes)
        .map_err(|error| invalid_archive(format!("invalid {label}: {error}")))?;
    Ok(bytes)
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
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
            .ok_or_else(|| limit("ZIP expanded size overflowed"))?;
        if self.count > MAX_ENTRIES || size > MAX_ENTRY_BYTES || self.expanded > self.limit {
            return Err(limit(
                "ZIP entry count, entry bytes, or compression ratio exceeds its bound",
            ));
        }
        Ok(())
    }
}
