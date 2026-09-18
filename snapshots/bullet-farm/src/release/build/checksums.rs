//! Checksum manifest generation and mandatory re-read verification.

use std::path::PathBuf;

use serde_json::{Map, Value, json};

use super::{BuildPlan, BundleFile, SUPPORTED_TARGET, archive::ArchiveOutput};
use crate::coord::CoordError;

pub(super) const CHECKSUM_SCHEMA_VERSION: &str = "1";
/// BLAKE3 is the family digest and the only algorithm the committed verifier,
/// family lock, Portal bundle manifest, and coordination ledger accept. No
/// SHA-256 implementation is pinned in this workspace's `Cargo.lock`, so a
/// second algorithm here would be an unpinned new dependency, not evidence.
pub(super) const CHECKSUM_ALGORITHM: &str = "blake3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ChecksumEntry {
    pub(super) path: String,
    pub(super) size: u64,
    pub(super) digest: String,
}

pub(super) struct ChecksumOutput {
    pub(super) relative: String,
    pub(super) digest: String,
    pub(super) entries: Vec<ChecksumEntry>,
}

/// Writes the checksum manifest, then re-opens it, re-parses it, and re-hashes
/// every byte subject it names before the build may continue.
pub(super) fn write(
    plan: &BuildPlan,
    archive: &ArchiveOutput,
    extra: &[BundleFile],
) -> Result<ChecksumOutput, CoordError> {
    let mut bundle = vec![ChecksumEntry {
        path: archive.relative.clone(),
        size: archive.size,
        digest: archive.digest.clone(),
    }];
    for file in extra {
        bundle.push(ChecksumEntry {
            path: file.relative.clone(),
            size: file.size,
            digest: file.digest.clone(),
        });
    }
    bundle.sort_by(|left, right| left.path.cmp(&right.path));

    let mut document = Map::new();
    document.insert(
        "checksum_manifest_schema_version".to_owned(),
        json!(CHECKSUM_SCHEMA_VERSION),
    );
    document.insert("algorithm".to_owned(), json!(CHECKSUM_ALGORITHM));
    document.insert("family".to_owned(), json!("bullet-farm"));
    document.insert("tag".to_owned(), json!(plan.tag));
    document.insert("target".to_owned(), json!(SUPPORTED_TARGET));
    document.insert(
        "archive_entry".to_owned(),
        Value::Array(
            archive
                .entries
                .iter()
                .map(|entry| {
                    json!({
                        "path": entry.path,
                        "kind": if entry.directory { "directory" } else { "file" },
                        "mode": format!("{:04o}", entry.mode),
                        "size": entry.size,
                        "digest": entry.digest,
                    })
                })
                .collect(),
        ),
    );
    document.insert(
        "bundle_file".to_owned(),
        Value::Array(
            bundle
                .iter()
                .map(|entry| {
                    json!({ "path": entry.path, "size": entry.size, "digest": entry.digest })
                })
                .collect(),
        ),
    );
    let relative = format!("{SUPPORTED_TARGET}/{}.checksums.json", plan.stem());
    let path = plan.out.join(&relative);
    let bytes = super::manifest::canonical_bytes(&Value::Object(document))?;
    super::write_new(&path, &bytes)?;
    verify(plan, &path, &bytes, &relative, &bundle, archive)?;
    Ok(ChecksumOutput {
        digest: super::digest_bytes(&bytes),
        relative,
        entries: bundle,
    })
}

/// The re-read half: the file on disk is the file that was written, it does not
/// name itself, and every subject it names still hashes to the recorded digest.
fn verify(
    plan: &BuildPlan,
    path: &PathBuf,
    written: &[u8],
    relative: &str,
    bundle: &[ChecksumEntry],
    archive: &ArchiveOutput,
) -> Result<(), CoordError> {
    let reread = std::fs::read(path).map_err(CoordError::io)?;
    if reread != written {
        return Err(mismatch(
            "the checksum manifest changed between writing and re-reading it",
        ));
    }
    let parsed: Value = bullet_wire::decode_unique_value(&reread)
        .map_err(|error| mismatch(format!("the checksum manifest JSON is ambiguous: {error}")))?;
    let files = parsed
        .get("bundle_file")
        .and_then(Value::as_array)
        .ok_or_else(|| mismatch("the re-read checksum manifest has no bundle_file array"))?;
    if files.len() != bundle.len() {
        return Err(mismatch("the re-read checksum manifest lost a bundle file"));
    }
    for (record, expected) in files.iter().zip(bundle) {
        let named = record
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if named == relative {
            return Err(mismatch(
                "the checksum manifest names itself; a checksum manifest is never its own subject",
            ));
        }
        if named != expected.path
            || record.get("size").and_then(Value::as_u64) != Some(expected.size)
            || record.get("digest").and_then(Value::as_str) != Some(expected.digest.as_str())
        {
            return Err(mismatch(format!(
                "{named} drifted in the checksum manifest"
            )));
        }
        let (digest, size) = super::digest_path(&plan.out.join(named))?;
        if digest != expected.digest || size != expected.size {
            return Err(mismatch(format!(
                "{named} on disk differs from the checksum manifest that was just written"
            )));
        }
    }
    let entries = parsed
        .get("archive_entry")
        .and_then(Value::as_array)
        .ok_or_else(|| mismatch("the re-read checksum manifest has no archive_entry array"))?;
    if entries.len() != archive.entries.len() {
        return Err(mismatch(
            "the re-read checksum manifest lost an archive entry",
        ));
    }
    for (record, expected) in entries.iter().zip(&archive.entries) {
        if record.get("path").and_then(Value::as_str) != Some(expected.path.as_str())
            || record.get("digest").and_then(Value::as_str) != Some(expected.digest.as_str())
            || record.get("size").and_then(Value::as_u64) != Some(expected.size)
        {
            return Err(mismatch(format!(
                "{} drifted in the checksum manifest",
                expected.path
            )));
        }
    }
    Ok(())
}

fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("RELEASE_CHECKSUM_MISMATCH", reason)
}
