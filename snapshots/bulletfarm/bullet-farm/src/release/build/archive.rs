//! Deterministic TAR+Zstandard production and extractor re-read.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use super::{
    BuildPlan, SUPPORTED_TARGET, cargo::BuiltBinary, checksums::ChecksumOutput, failed, invalid,
};
use crate::{
    coord::CoordError,
    release::{ReleaseFile, ReleaseManifest, ReleasePackage, SignedReleaseFile},
};

const ARCHIVE_ROOT: &str = "bullet-farm";
const ZSTD_LEVEL: i32 = 19;
const MAX_ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;

/// One archive entry exactly as the committed extractor will read it back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ArchiveEntry {
    pub(super) path: String,
    pub(super) directory: bool,
    pub(super) size: u64,
    pub(super) mode: u32,
    pub(super) digest: String,
}

pub(super) struct ArchiveOutput {
    pub(super) relative: String,
    pub(super) size: u64,
    pub(super) digest: String,
    pub(super) entries: Vec<ArchiveEntry>,
}

pub(super) fn write(
    plan: &BuildPlan,
    binaries: &[BuiltBinary],
) -> Result<ArchiveOutput, CoordError> {
    let hub = plan.member("bullet-farm")?;
    let mut sources = vec![
        (
            format!("{ARCHIVE_ROOT}/LICENSE"),
            hub.path.join("LICENSE"),
            0o644_u32,
        ),
        (
            format!("{ARCHIVE_ROOT}/share/family.lock"),
            hub.path.join("family.lock"),
            0o644,
        ),
    ];
    for binary in binaries {
        sources.push((
            format!("{ARCHIVE_ROOT}/bin/{}", binary.name),
            binary.path.clone(),
            0o755,
        ));
    }
    let mut entries = plan_entries(&sources)?;
    let relative = format!("{SUPPORTED_TARGET}/{}.tar.zst", plan.stem());
    let path = plan.out.join(&relative);
    write_archive(&path, &entries, &sources)?;
    let (digest, size) = super::digest_path(&path)?;
    if size == 0 || size > MAX_ARCHIVE_BYTES {
        return Err(failed(format!(
            "the produced archive is {size} bytes, outside the extractor's admitted bound"
        )));
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(ArchiveOutput {
        relative,
        size,
        digest,
        entries,
    })
}

/// Re-reads the produced archive through the committed extractor and proves
/// every materialized byte matches the checksum manifest that was written for
/// it. This is a read-back of production, not release or install evidence.
pub(super) fn reread(
    plan: &BuildPlan,
    archive: &ArchiveOutput,
    checksums: &ChecksumOutput,
) -> Result<PathBuf, CoordError> {
    let parent = plan.scratch.join("extracted");
    fs::create_dir_all(&parent).map_err(CoordError::io)?;
    let parent = super::admitted_absolute_dir(&parent, "release build re-read parent")?;
    let destination = parent.join(ARCHIVE_ROOT);
    let manifest = readback_manifest(plan, archive);
    crate::release::archive::extract(&plan.out, &manifest, SUPPORTED_TARGET, &destination)?;
    for entry in &archive.entries {
        let relative = entry
            .path
            .strip_prefix(ARCHIVE_ROOT)
            .and_then(|rest| rest.strip_prefix('/'));
        let Some(relative) = relative else {
            continue;
        };
        let materialized = destination.join(relative);
        if entry.directory {
            let metadata = fs::symlink_metadata(&materialized).map_err(CoordError::io)?;
            if !metadata.file_type().is_dir() {
                return Err(mismatch(format!("{} is not a directory", entry.path)));
            }
            continue;
        }
        let (digest, size) = super::digest_path(&materialized)?;
        if size != entry.size || digest != entry.digest {
            return Err(mismatch(format!(
                "{} differs after extraction; the archive is not reproducible",
                entry.path
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let actual_mode = fs::metadata(&materialized)
                .map_err(CoordError::io)?
                .permissions()
                .mode()
                & 0o777;
            if actual_mode != entry.mode {
                return Err(mismatch(format!(
                    "{} has mode {actual_mode:04o} after extraction; expected {:04o}",
                    entry.path, entry.mode
                )));
            }
        }
    }
    if !checksums
        .entries
        .iter()
        .any(|entry| entry.path == archive.relative)
    {
        return Err(mismatch(
            "the checksum manifest does not cover the archive it was generated for",
        ));
    }
    Ok(destination)
}

/// An in-memory descriptor for the extractor's own path and size admission.
/// It is never written to disk and carries no signature: the signature records
/// are deliberately empty because this build signs nothing.
fn readback_manifest(plan: &BuildPlan, archive: &ArchiveOutput) -> ReleaseManifest {
    let hub = plan
        .subjects
        .iter()
        .find(|subject| subject.name == "bullet-farm");
    let file = ReleaseFile {
        path: archive.relative.clone(),
        size: archive.size,
        digest: archive.digest.clone(),
    };
    let absent = |path: &str| SignedReleaseFile {
        file: ReleaseFile {
            path: path.to_owned(),
            size: 0,
            digest: String::new(),
        },
        signature: ReleaseFile {
            path: format!("{path}.sig"),
            size: 0,
            digest: String::new(),
        },
    };
    ReleaseManifest {
        release_manifest_schema_version: crate::release::RELEASE_MANIFEST_SCHEMA_VERSION.to_owned(),
        family_lock_schema_version: plan.lock_schema_version.clone(),
        family: "bullet-farm".to_owned(),
        tag: plan.tag.clone(),
        hub_commit_oid: hub.map_or_else(String::new, |subject| subject.commit_oid.clone()),
        hub_tree_oid: hub.map_or_else(String::new, |subject| subject.tree_oid.clone()),
        release_signing_identity: plan.signing_identity.clone(),
        family_lock: ReleaseFile {
            path: "family.lock".to_owned(),
            size: 0,
            digest: String::new(),
        },
        package: vec![ReleasePackage {
            target: SUPPORTED_TARGET.to_owned(),
            archive: SignedReleaseFile {
                file,
                signature: ReleaseFile {
                    path: format!("{}.sig", archive.relative),
                    size: 0,
                    digest: String::new(),
                },
            },
            checksums: absent(&format!(
                "{SUPPORTED_TARGET}/{}.checksums.json",
                plan.stem()
            )),
            cyclonedx_sbom: absent(&format!("{SUPPORTED_TARGET}/{}.cdx.json", plan.stem())),
            spdx_sbom: absent(&format!("{SUPPORTED_TARGET}/{}.spdx.json", plan.stem())),
            provenance: absent(&format!("{SUPPORTED_TARGET}/{}.intoto.jsonl", plan.stem())),
        }],
    }
}

pub(super) fn plan_entries(
    sources: &[(String, PathBuf, u32)],
) -> Result<Vec<ArchiveEntry>, CoordError> {
    let mut directories = std::collections::BTreeSet::from([ARCHIVE_ROOT.to_owned()]);
    let mut entries = Vec::new();
    for (path, source, mode) in sources {
        let (digest, size) = super::digest_path(source)?;
        if size == 0 {
            return Err(failed(format!("{path} would be an empty archive entry")));
        }
        let mut cursor = path.as_str();
        while let Some((parent, _)) = cursor.rsplit_once('/') {
            directories.insert(parent.to_owned());
            cursor = parent;
        }
        entries.push(ArchiveEntry {
            path: path.clone(),
            directory: false,
            size,
            mode: *mode,
            digest,
        });
    }
    for directory in directories {
        entries.push(ArchiveEntry {
            path: directory,
            directory: true,
            size: 0,
            mode: 0o755,
            digest: super::digest_bytes(&[]),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let mut folded = std::collections::BTreeSet::new();
    for entry in &entries {
        if !folded.insert(entry.path.to_ascii_lowercase()) {
            return Err(invalid(format!(
                "archive path {} repeats under ASCII case folding",
                entry.path
            )));
        }
    }
    if entries.first().map(|entry| entry.path.as_str()) != Some(ARCHIVE_ROOT) {
        return Err(invalid("the archive must begin with its bullet-farm root"));
    }
    Ok(entries)
}

pub(super) fn write_archive(
    path: &Path,
    entries: &[ArchiveEntry],
    sources: &[(String, PathBuf, u32)],
) -> Result<(), CoordError> {
    let output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(CoordError::io)?;
    let encoder = zstd::stream::write::Encoder::new(output, ZSTD_LEVEL).map_err(CoordError::io)?;
    let mut builder = tar::Builder::new(encoder);
    builder.mode(tar::HeaderMode::Deterministic);
    for entry in entries {
        let mut header = tar::Header::new_ustar();
        header.set_path(&entry.path).map_err(CoordError::io)?;
        header.set_entry_type(if entry.directory {
            tar::EntryType::Directory
        } else {
            tar::EntryType::Regular
        });
        header.set_size(entry.size);
        header.set_mode(entry.mode);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_username("").map_err(CoordError::io)?;
        header.set_groupname("").map_err(CoordError::io)?;
        header.set_cksum();
        if entry.directory {
            builder
                .append(&header, std::io::empty())
                .map_err(CoordError::io)?;
            continue;
        }
        let source = sources
            .iter()
            .find(|(archive_path, _, _)| archive_path == &entry.path)
            .ok_or_else(|| failed(format!("{} lost its source file", entry.path)))?;
        let mut file = File::open(&source.1).map_err(CoordError::io)?;
        builder.append(&header, &mut file).map_err(CoordError::io)?;
    }
    let encoder = builder.into_inner().map_err(CoordError::io)?;
    let mut output = encoder.finish().map_err(CoordError::io)?;
    output.flush().map_err(CoordError::io)?;
    output.sync_all().map_err(CoordError::io)
}

fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("RELEASE_CHECKSUM_MISMATCH", reason)
}
