//! Signed-tag verification and tagged-object hashing for family locks.

mod command;

use std::{collections::BTreeSet, path::Path, time::Duration};

use serde::Deserialize;

use super::schema::{LockedFile, LockedMember, validate_repository_path};
use crate::{coord::CoordError, process::Limits};

const GENERATED_ZONES: &str = "agent/generated-zones.toml";
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 16 * 1024 * 1024,
};
const MAX_HASHED_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_GENERATED_ARTIFACTS: usize = 4096;

pub(super) fn run_admitted_git_after_verify(
    repo: &Path,
    args: &[&str],
    limits: Limits,
    label: &str,
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<std::process::Output, CoordError> {
    command::run_labeled_after_verify(repo, args, limits, label, after_verify)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedZones {
    zone: Vec<GeneratedZone>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedZone {
    path: String,
    source: String,
    owner: String,
}

pub(super) fn tag_commit(repo: &Path, tag: &str) -> Result<String, CoordError> {
    let object_type = git(repo, &["cat-file", "-t", &format!("refs/tags/{tag}")])?;
    if object_type.trim() != "tag" {
        return Err(CoordError::new(
            "UNSIGNED_OR_LIGHTWEIGHT_TAG",
            format!("{tag} in {} is not an annotated tag", repo.display()),
        ));
    }
    tagged_oid(repo, &format!("{tag}^{{commit}}"))
}

pub(super) fn tag_tree(repo: &Path, tag: &str) -> Result<String, CoordError> {
    tagged_oid(repo, &format!("{tag}^{{tree}}"))
}

pub(super) fn head_commit(repo: &Path) -> Result<String, CoordError> {
    tagged_oid(repo, "HEAD^{commit}")
}

pub(super) fn head_tree(repo: &Path) -> Result<String, CoordError> {
    tagged_oid(repo, "HEAD^{tree}")
}

fn tagged_oid(repo: &Path, revision: &str) -> Result<String, CoordError> {
    let algorithm = git(repo, &["rev-parse", "--show-object-format"])?;
    let algorithm = algorithm.trim();
    let expected = match algorithm {
        "sha1" => 40,
        "sha256" => 64,
        _ => {
            return Err(CoordError::new(
                "UNSUPPORTED_GIT_OBJECT_FORMAT",
                format!("unsupported Git object format {algorithm:?}"),
            ));
        }
    };
    let oid = git(repo, &["rev-parse", "--verify", revision])?;
    let oid = oid.trim();
    if oid.len() != expected
        || !oid
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CoordError::new(
            "INVALID_GIT_OID",
            format!("{revision} resolved to an invalid {algorithm} OID"),
        ));
    }
    Ok(format!("{algorithm}:{oid}"))
}

pub(super) fn verify_tag(
    repo: &Path,
    tag: &str,
    allowed_signers: &Path,
) -> Result<String, CoordError> {
    let signer_metadata = std::fs::symlink_metadata(allowed_signers).map_err(CoordError::io)?;
    if !signer_metadata.file_type().is_file()
        || signer_metadata.file_type().is_symlink()
        || signer_metadata.len() > 64 * 1024
    {
        return Err(CoordError::new(
            "INVALID_ALLOWED_SIGNERS",
            "allowed-signers must be a regular non-symlink file no larger than 64 KiB",
        ));
    }
    let output = git_output_with_helper(
        repo,
        &["-c", "gpg.format=ssh", "verify-tag", "--raw", tag],
        allowed_signers,
    )?;
    let status = String::from_utf8(output.stderr).map_err(|_| {
        CoordError::new(
            "INVALID_GIT_OUTPUT",
            "Git emitted non-UTF-8 signature status",
        )
    })?;
    signer_identity(&status, tag)
}

pub(super) fn signer_identity(status: &str, tag: &str) -> Result<String, CoordError> {
    let lines: Vec<_> = status
        .lines()
        .filter_map(|line| line.strip_prefix("Good \"git\" signature for "))
        .collect();
    if lines.len() != 1 {
        return Err(CoordError::new(
            "TAG_SIGNER_IDENTITY_MISSING",
            format!("{tag} has no single verified SSH signer status"),
        ));
    }
    let (principal, fingerprint) = lines[0].split_once(" with ED25519 key ").ok_or_else(|| {
        CoordError::new(
            "UNSUPPORTED_TAG_SIGNATURE",
            format!("{tag} is not signed by an Ed25519 SSH identity"),
        )
    })?;
    if principal.is_empty()
        || principal.bytes().any(|byte| byte.is_ascii_whitespace())
        || !fingerprint.starts_with("SHA256:")
        || fingerprint.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'+' | b'/' | b'='))
        })
    {
        return Err(CoordError::new(
            "INVALID_TAG_SIGNER_IDENTITY",
            format!("{tag} returned a malformed signer identity"),
        ));
    }
    Ok(format!("{principal}|ed25519|{fingerprint}"))
}

pub(super) fn digest_tagged_tree(
    repo: &Path,
    tag: &str,
    prefix: &str,
) -> Result<String, CoordError> {
    let paths = tracked_regular_files(repo, tag, prefix)?;
    if paths.is_empty() {
        return Err(CoordError::new(
            "SCHEMA_BUNDLE_MISSING",
            format!("{tag} has no {prefix} tree"),
        ));
    }
    digest_tagged_files(repo, tag, "bullet.family.schema-bundle.v1", &paths)
}

pub(super) fn digest_dependency_lockfiles(
    repo: &Path,
    tag: &str,
    name: &str,
) -> Result<Vec<LockedFile>, CoordError> {
    let path = if name == "bullet-portal" {
        "package-lock.json"
    } else {
        "Cargo.lock"
    };
    Ok(vec![digest_file(repo, tag, path)?])
}

pub(super) fn digest_generated_artifacts(
    repo: &Path,
    tag: &str,
) -> Result<Vec<LockedFile>, CoordError> {
    let definition = git_bytes(repo, &["show", &format!("{tag}:{GENERATED_ZONES}")])?;
    let definition = std::str::from_utf8(&definition).map_err(|_| {
        CoordError::new(
            "INVALID_GENERATED_ZONES",
            "generated-zone metadata is not UTF-8",
        )
    })?;
    let zones: GeneratedZones = toml::from_str(definition).map_err(|error| {
        CoordError::new(
            "INVALID_GENERATED_ZONES",
            format!("{GENERATED_ZONES}: {error}"),
        )
    })?;
    if zones.zone.len() > 256 {
        return Err(CoordError::new(
            "INVALID_GENERATED_ZONES",
            "generated-zone count exceeds 256",
        ));
    }
    let mut paths = BTreeSet::new();
    for zone in zones.zone {
        if zone.source.is_empty() || zone.owner.is_empty() {
            return Err(CoordError::new(
                "INVALID_GENERATED_ZONES",
                "generated zones require source and owner",
            ));
        }
        if matches!(zone.path.as_str(), ".fusion" | ".fusion/") {
            continue;
        }
        let query = zone.path.strip_suffix('/').unwrap_or(&zone.path);
        validate_repository_path(query)?;
        let expanded = tracked_regular_files(repo, tag, query)?;
        if expanded.is_empty() {
            return Err(CoordError::new(
                "GENERATED_ARTIFACT_MISSING",
                format!("generated zone {} is empty at {tag}", zone.path),
            ));
        }
        for path in expanded {
            if !paths.insert(path.clone()) {
                return Err(CoordError::new(
                    "DUPLICATE_GENERATED_ARTIFACT",
                    format!("generated zones overlap at {path}"),
                ));
            }
        }
    }
    paths
        .into_iter()
        .map(|path| digest_file(repo, tag, &path))
        .collect()
}

pub(super) fn verify_locked_checkout(
    member: &LockedMember,
    repo: &Path,
    allowed_signers: &Path,
) -> Result<(), CoordError> {
    if head_commit(repo)? != member.commit_oid {
        return Err(CoordError::new(
            "LOCKED_COMMIT_MISMATCH",
            format!("{} HEAD does not match its locked commit", member.name),
        ));
    }
    if head_tree(repo)? != member.tree_oid {
        return Err(CoordError::new(
            "LOCKED_TREE_MISMATCH",
            format!("{} tree does not match its locked tree", member.name),
        ));
    }
    if tag_commit(repo, &member.tag)? != member.commit_oid
        || tag_tree(repo, &member.tag)? != member.tree_oid
    {
        return Err(CoordError::new(
            "LOCKED_TAG_SUBJECT_MISMATCH",
            format!("{} tag does not resolve to its locked subject", member.name),
        ));
    }
    if verify_tag(repo, &member.tag, allowed_signers)? != member.release_signing_identity {
        return Err(CoordError::new(
            "LOCKED_SIGNER_MISMATCH",
            format!("{} tag signer does not match the lock", member.name),
        ));
    }
    let lockfiles = digest_dependency_lockfiles(repo, &member.tag, &member.name)?;
    if lockfiles != member.lockfile {
        return Err(CoordError::new(
            "LOCKED_LOCKFILE_MISMATCH",
            format!("{} dependency lockfile digest differs", member.name),
        ));
    }
    let artifacts = digest_generated_artifacts(repo, &member.tag)?;
    if artifacts != member.artifact {
        return Err(CoordError::new(
            "LOCKED_ARTIFACT_MISMATCH",
            format!("{} generated artifact manifest differs", member.name),
        ));
    }
    Ok(())
}

fn digest_file(repo: &Path, revision: &str, path: &str) -> Result<LockedFile, CoordError> {
    validate_repository_path(path)?;
    let bytes = verified_blob(repo, revision, path)?;
    Ok(LockedFile {
        path: path.to_owned(),
        digest: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
    })
}

pub(super) fn verified_blob(
    repo: &Path,
    revision: &str,
    path: &str,
) -> Result<Vec<u8>, CoordError> {
    validate_repository_path(path)?;
    let size = git(repo, &["cat-file", "-s", &format!("{revision}:{path}")])?;
    let size = parse_ascii_u64(size.trim()).ok_or_else(|| {
        CoordError::new("INVALID_GIT_OUTPUT", "Git emitted an invalid object size")
    })?;
    if size > MAX_HASHED_FILE_BYTES {
        return Err(CoordError::new(
            "TAGGED_FILE_TOO_LARGE",
            format!("{path} exceeds the 16 MiB verification limit"),
        ));
    }
    git_bytes(repo, &["show", &format!("{revision}:{path}")])
}

fn parse_ascii_u64(value: &str) -> Option<u64> {
    if value.is_empty() {
        return None;
    }
    value.bytes().try_fold(0_u64, |number, byte| {
        byte.is_ascii_digit()
            .then_some(byte - b'0')
            .and_then(|digit| number.checked_mul(10)?.checked_add(u64::from(digit)))
    })
}

fn digest_tagged_files(
    repo: &Path,
    tag: &str,
    domain: &str,
    paths: &[String],
) -> Result<String, CoordError> {
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, domain.as_bytes());
    for path in paths {
        frame(&mut hasher, path.as_bytes());
        let bytes = verified_blob(repo, tag, path)?;
        frame(&mut hasher, &bytes);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn tracked_regular_files(
    repo: &Path,
    revision: &str,
    prefix: &str,
) -> Result<Vec<String>, CoordError> {
    validate_repository_path(prefix)?;
    let listing = git_bytes(repo, &["ls-tree", "-r", "-z", revision, "--", prefix])?;
    let mut paths = Vec::new();
    for record in listing
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let record = std::str::from_utf8(record)
            .map_err(|_| CoordError::new("INVALID_GIT_OUTPUT", "Git tree path is not UTF-8"))?;
        let (metadata, path) = record
            .split_once('\t')
            .ok_or_else(|| CoordError::new("INVALID_GIT_OUTPUT", "Git tree entry is malformed"))?;
        let mode = metadata.split_whitespace().next().unwrap_or_default();
        if !matches!(mode, "100644" | "100755") {
            return Err(CoordError::new(
                "UNSAFE_TAGGED_ARTIFACT",
                format!("{path} is not a regular tracked file"),
            ));
        }
        validate_repository_path(path)?;
        paths.push(path.to_owned());
    }
    paths.sort();
    if paths.len() > MAX_GENERATED_ARTIFACTS {
        return Err(CoordError::new(
            "TOO_MANY_TAGGED_ARTIFACTS",
            "tagged artifact count exceeds 4096",
        ));
    }
    if paths.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CoordError::new(
            "DUPLICATE_TAGGED_PATH",
            "Git returned duplicate artifact paths",
        ));
    }
    Ok(paths)
}

pub(super) fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn git(repo: &Path, args: &[&str]) -> Result<String, CoordError> {
    String::from_utf8(git_bytes(repo, args)?)
        .map_err(|_| CoordError::new("INVALID_GIT_OUTPUT", "Git emitted non-UTF-8 metadata"))
}

fn git_bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>, CoordError> {
    Ok(git_output(repo, args)?.stdout)
}

fn git_output(repo: &Path, args: &[&str]) -> Result<std::process::Output, CoordError> {
    checked_output(repo, args, None)
}

fn git_output_with_helper(
    repo: &Path,
    args: &[&str],
    allowed_signers: &Path,
) -> Result<std::process::Output, CoordError> {
    checked_output(repo, args, Some(allowed_signers))
}

fn checked_output(
    repo: &Path,
    args: &[&str],
    allowed_signers: Option<&Path>,
) -> Result<std::process::Output, CoordError> {
    let output = command::run(repo, args, allowed_signers, GIT_LIMITS)?;
    if !output.status.success() {
        return Err(CoordError::new(
            "GIT_VERIFICATION_FAILED",
            format!(
                "Git verification failed in {} for {:?}",
                repo.display(),
                args
            ),
        ));
    }
    Ok(output)
}
