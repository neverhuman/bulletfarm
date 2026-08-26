//! Fail-closed extraction of one already verified release archive.

mod publish;
mod tar_zst;
mod zip;

use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "linux")]
use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};

use super::{ReleaseFile, ReleaseManifest};
use crate::coord::CoordError;

const ARCHIVE_ROOT: &str = "bullet-farm";
#[cfg(target_os = "linux")]
const MAX_ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_EXPANDED_BYTES: u64 = 1024 * 1024 * 1024;
const MIN_RATIO_ALLOWANCE: u64 = 16 * 1024 * 1024;
const MAX_EXPANSION_RATIO: u64 = 100;
const MAX_ENTRIES: usize = 4096;
const MAX_PATH_BYTES: usize = 512;
const MAX_SEGMENT_BYTES: usize = 255;
pub(super) const PACKAGED_BINARY_NAMES: [&str; 8] = [
    "bullet",
    "bullet-effects",
    "bullet-family",
    "bullet-farmd",
    "bullet-gitd",
    "bullet-mcpd",
    "bullet-runner",
    "bullet-verifier",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EntryKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RawEntry {
    pub(super) name: Vec<u8>,
    pub(super) kind: EntryKind,
    pub(super) size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PlannedEntry {
    pub(super) path: String,
    pub(super) kind: EntryKind,
    pub(super) size: u64,
}

#[derive(Debug)]
pub(super) struct ArchivePlan {
    entries: Vec<PlannedEntry>,
    executable: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArchiveFormat {
    TarZstd,
    Zip,
}

pub(super) fn extract(
    bundle: &Path,
    manifest: &ReleaseManifest,
    target: &str,
    destination: &Path,
) -> Result<(), CoordError> {
    let package = manifest
        .package
        .iter()
        .find(|package| package.target == target)
        .ok_or_else(|| {
            CoordError::new(
                "RELEASE_TARGET_NOT_FOUND",
                "target is not one of the exact signed release packages",
            )
        })?;
    let format = ArchiveFormat::for_path(&package.archive.file.path)?;
    let admitted_destination = publish::admit_destination(destination)?;
    let mut snapshot = snapshot_archive(bundle, &package.archive.file)?;
    let raw = format.scan(&mut snapshot)?;
    let plan = ArchivePlan::admit(raw, target, package.archive.file.size)?;

    let staging = tempfile::Builder::new()
        .prefix(".bullet-release-extract-")
        .tempdir_in(admitted_destination.parent_path())
        .map_err(CoordError::io)?;
    snapshot.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    format.materialize(&mut snapshot, &plan, staging.path())?;
    plan.sync_tree(staging.path())?;
    publish::publish_no_replace(&staging.path().join(ARCHIVE_ROOT), &admitted_destination)
}

impl ArchiveFormat {
    fn for_path(path: &str) -> Result<Self, CoordError> {
        if path.ends_with(".tar.zst") {
            Ok(Self::TarZstd)
        } else if path.ends_with(".zip") {
            Ok(Self::Zip)
        } else {
            Err(invalid_archive("signed archive format is unsupported"))
        }
    }

    fn scan(self, input: &mut File) -> Result<Vec<RawEntry>, CoordError> {
        input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
        match self {
            Self::TarZstd => tar_zst::scan(input.try_clone().map_err(CoordError::io)?),
            Self::Zip => zip::scan(input.try_clone().map_err(CoordError::io)?),
        }
    }

    fn materialize(
        self,
        input: &mut File,
        plan: &ArchivePlan,
        output: &Path,
    ) -> Result<(), CoordError> {
        input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
        match self {
            Self::TarZstd => {
                tar_zst::materialize(input.try_clone().map_err(CoordError::io)?, plan, output)
            }
            Self::Zip => zip::materialize(input.try_clone().map_err(CoordError::io)?, plan, output),
        }
    }
}

impl ArchivePlan {
    fn admit(raw: Vec<RawEntry>, target: &str, archive_size: u64) -> Result<Self, CoordError> {
        if raw.is_empty() || raw.len() > MAX_ENTRIES {
            return Err(limit("archive entry count is outside the admitted bound"));
        }
        let suffix = if target == "x86_64-pc-windows-msvc" {
            ".exe"
        } else {
            ""
        };
        let executable = PACKAGED_BINARY_NAMES
            .iter()
            .map(|name| format!("{ARCHIVE_ROOT}/bin/{name}{suffix}"))
            .collect::<BTreeSet<_>>();
        let ratio_limit = archive_size
            .checked_mul(MAX_EXPANSION_RATIO)
            .unwrap_or(MAX_EXPANDED_BYTES)
            .clamp(MIN_RATIO_ALLOWANCE, MAX_EXPANDED_BYTES);
        let mut paths = BTreeSet::new();
        let mut directories = BTreeSet::new();
        let mut entries = Vec::with_capacity(raw.len());
        let mut expanded = 0_u64;
        let mut previous: Option<String> = None;

        for raw_entry in raw {
            let path = admit_path(&raw_entry.name, raw_entry.kind)?;
            if previous.as_ref().is_some_and(|prior| prior >= &path) {
                return Err(invalid_archive(
                    "archive entries must be uniquely byte-sorted",
                ));
            }
            let folded = path.to_ascii_lowercase();
            if !paths.insert(folded) {
                return Err(invalid_archive(
                    "archive paths repeat or collide under ASCII case folding",
                ));
            }
            if path == ARCHIVE_ROOT {
                if !entries.is_empty() || raw_entry.kind != EntryKind::Directory {
                    return Err(invalid_archive(
                        "archive must begin with its single bullet-farm directory",
                    ));
                }
            } else {
                let parent = path
                    .rsplit_once('/')
                    .map(|(parent, _)| parent)
                    .ok_or_else(|| invalid_archive("archive contains an extra top-level root"))?;
                if !directories.contains(parent) {
                    return Err(invalid_archive(
                        "every archive parent directory must appear before its children",
                    ));
                }
            }
            if raw_entry.kind == EntryKind::Directory {
                if raw_entry.size != 0 {
                    return Err(invalid_archive("archive directory has nonzero content"));
                }
                directories.insert(path.clone());
            } else {
                if raw_entry.size > MAX_ENTRY_BYTES {
                    return Err(limit("archive entry exceeds its byte limit"));
                }
                expanded = expanded
                    .checked_add(raw_entry.size)
                    .ok_or_else(|| limit("archive expanded size overflowed"))?;
                if expanded > ratio_limit {
                    return Err(limit(
                        "archive expanded bytes exceed the absolute or compression-ratio bound",
                    ));
                }
            }
            previous = Some(path.clone());
            entries.push(PlannedEntry {
                path,
                kind: raw_entry.kind,
                size: raw_entry.size,
            });
        }
        for required in &executable {
            if !entries.iter().any(|entry| {
                entry.path == *required && entry.kind == EntryKind::File && entry.size > 0
            }) {
                return Err(invalid_archive(format!(
                    "archive does not contain required packaged executable {required}"
                )));
            }
        }
        if entries.iter().any(|entry| {
            entry.kind == EntryKind::File
                && entry.path.starts_with("bullet-farm/bin/")
                && !executable.contains(&entry.path)
        }) {
            return Err(invalid_archive(
                "archive contains an executable outside the exact packaged binary set",
            ));
        }
        Ok(Self {
            entries,
            executable,
        })
    }

    pub(super) fn entries(&self) -> &[PlannedEntry] {
        &self.entries
    }

    fn sync_tree(&self, output: &Path) -> Result<(), CoordError> {
        for entry in self.entries.iter().rev() {
            if entry.kind == EntryKind::Directory {
                let directory = File::open(output.join(&entry.path)).map_err(CoordError::io)?;
                directory.sync_all().map_err(CoordError::io)?;
            }
        }
        File::open(output)
            .and_then(|directory| directory.sync_all())
            .map_err(CoordError::io)
    }
}

pub(super) fn materialize_entry<R: Read + ?Sized>(
    reader: &mut R,
    actual: &RawEntry,
    expected: &PlannedEntry,
    output: &Path,
    executable: &BTreeSet<String>,
) -> Result<(), CoordError> {
    let path = admit_path(&actual.name, actual.kind)?;
    if path != expected.path || actual.kind != expected.kind || actual.size != expected.size {
        return Err(invalid_archive(
            "archive changed between admission and materialization",
        ));
    }
    let destination = output.join(&expected.path);
    match expected.kind {
        EntryKind::Directory => {
            fs::create_dir(&destination).map_err(CoordError::io)?;
            set_mode(&destination, 0o755)?;
        }
        EntryKind::File => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(CoordError::io)?;
            copy_exact(reader, &mut file, expected.size)?;
            set_mode(
                &destination,
                if executable.contains(&expected.path) {
                    0o755
                } else {
                    0o644
                },
            )?;
            file.sync_all().map_err(CoordError::io)?;
        }
    }
    Ok(())
}

fn copy_exact<R: Read + ?Sized>(
    reader: &mut R,
    output: &mut File,
    size: u64,
) -> Result<(), CoordError> {
    let mut remaining = size;
    let mut buffer = [0_u8; 64 * 1024];
    while remaining > 0 {
        let wanted = usize::try_from(remaining.min(buffer.len() as u64))
            .expect("buffer-sized count fits usize");
        let count = reader
            .read(&mut buffer[..wanted])
            .map_err(|error| invalid_archive(format!("could not read archive entry: {error}")))?;
        if count == 0 {
            return Err(invalid_archive(
                "archive entry ended before its declared size",
            ));
        }
        output.write_all(&buffer[..count]).map_err(CoordError::io)?;
        remaining -= count as u64;
    }
    let mut extra = [0_u8; 1];
    if reader
        .read(&mut extra)
        .map_err(|error| invalid_archive(format!("could not finish archive entry: {error}")))?
        != 0
    {
        return Err(invalid_archive("archive entry exceeds its declared size"));
    }
    Ok(())
}

fn admit_path(raw: &[u8], kind: EntryKind) -> Result<String, CoordError> {
    let raw = if kind == EntryKind::Directory && raw.ends_with(b"/") {
        &raw[..raw.len() - 1]
    } else {
        raw
    };
    if raw.is_empty()
        || raw.len() > MAX_PATH_BYTES
        || !raw.is_ascii()
        || raw.starts_with(b"/")
        || raw.ends_with(b"/")
        || raw.contains(&b'\\')
        || raw.contains(&b':')
        || raw.iter().any(|byte| byte.is_ascii_control())
    {
        return Err(invalid_archive(
            "archive path is not a safe ASCII relative path",
        ));
    }
    let path = std::str::from_utf8(raw).expect("ASCII is UTF-8").to_owned();
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.first().copied() != Some(ARCHIVE_ROOT)
        || segments.iter().any(|segment| invalid_segment(segment))
    {
        return Err(invalid_archive(
            "archive path has an unsafe segment or extra top-level root",
        ));
    }
    Ok(path)
}

fn invalid_segment(segment: &str) -> bool {
    let stem = segment.split('.').next().unwrap_or(segment);
    segment.is_empty()
        || segment.len() > MAX_SEGMENT_BYTES
        || matches!(segment, "." | "..")
        || segment.eq_ignore_ascii_case(".git")
        || segment.ends_with(['.', ' '])
        || matches!(
            stem.to_ascii_uppercase().as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), CoordError> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(CoordError::io)
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), CoordError> {
    Err(CoordError::new(
        "RELEASE_EXTRACTION_PLATFORM_UNSUPPORTED",
        "deterministic release permissions are unsupported on this platform",
    ))
}

#[cfg(target_os = "linux")]
fn snapshot_archive(bundle: &Path, expected: &ReleaseFile) -> Result<File, CoordError> {
    use nix::{
        fcntl::{FcntlArg, SealFlag, fcntl},
        sys::memfd::{MemFdCreateFlag, memfd_create},
    };

    if expected.size > MAX_ARCHIVE_BYTES {
        return Err(limit("signed archive exceeds the extraction byte limit"));
    }
    let path = bundle.join(&expected.path);
    let mut source = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(&path)
        .map_err(CoordError::io)?;
    let metadata = source.metadata().map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.len() != expected.size {
        return Err(invalid_archive(
            "signed archive is not the exact bounded regular file",
        ));
    }
    let descriptor = memfd_create(
        c"bullet-release-archive",
        MemFdCreateFlag::MFD_ALLOW_SEALING | MemFdCreateFlag::MFD_CLOEXEC,
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_ARCHIVE_PIN_FAILED",
            format!("could not create archive snapshot: {error}"),
        )
    })?;
    let mut snapshot = File::from(descriptor);
    let mut hasher = blake3::Hasher::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    while copied <= expected.size {
        let count = source.read(&mut buffer).map_err(CoordError::io)?;
        if count == 0 {
            break;
        }
        copied = copied
            .checked_add(count as u64)
            .ok_or_else(|| limit("archive snapshot size overflowed"))?;
        if copied > expected.size {
            return Err(invalid_archive(
                "signed archive grew while it was being pinned",
            ));
        }
        hasher.update(&buffer[..count]);
        snapshot
            .write_all(&buffer[..count])
            .map_err(CoordError::io)?;
    }
    let digest = format!("blake3:{}", hasher.finalize().to_hex());
    if copied != expected.size || digest != expected.digest {
        return Err(invalid_archive(
            "pinned archive bytes differ from the signed manifest",
        ));
    }
    snapshot.flush().map_err(CoordError::io)?;
    fcntl(
        snapshot.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_ARCHIVE_PIN_FAILED",
            format!("could not seal archive snapshot: {error}"),
        )
    })?;
    snapshot.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(snapshot)
}

#[cfg(not(target_os = "linux"))]
fn snapshot_archive(_bundle: &Path, _expected: &ReleaseFile) -> Result<File, CoordError> {
    Err(CoordError::new(
        "RELEASE_EXTRACTION_PLATFORM_UNSUPPORTED",
        "exact archive snapshots are currently supported only on Linux",
    ))
}

pub(super) fn invalid_archive(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_ARCHIVE", reason)
}

fn limit(reason: impl Into<String>) -> CoordError {
    CoordError::new("RELEASE_ARCHIVE_LIMIT_EXCEEDED", reason)
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
