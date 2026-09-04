use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    process::Command,
};

use sha2::{Digest, Sha256};

use super::{GIT_BIN, GIT_LIMITS};
use crate::{
    coord::{
        CoordError,
        model::{RecoveryGitExpectationV1, RecoveryGitObjectFormatV1},
        validate_path, validate_repo_name,
    },
    process::{run_bounded, run_bounded_with_input_file},
};

#[path = "git/derive.rs"]
mod derive;
#[path = "git/manifest.rs"]
mod manifest;
#[path = "git/object_store.rs"]
mod object_store;
pub(crate) use derive::derive_recovery_commit;
use manifest::family_selection;

#[derive(Clone, Debug, Eq, PartialEq)]
struct RepositoryIdentity {
    root_device: u64,
    root_inode: u64,
    git_device: u64,
    git_inode: u64,
    object_store_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommitObservation {
    object_format: RecoveryGitObjectFormatV1,
    commit_oid: String,
    raw_commit_bytes: Vec<u8>,
    raw_commit_sha256: String,
    parent_oid: String,
    parent_tree_oid: String,
    result_tree_oid: String,
    raw_tree_sha256: String,
    leaves: Vec<LeafObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeafObservation {
    path: String,
    old_mode: String,
    new_mode: String,
    old_blob_oid: String,
    new_blob_oid: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
    old_sha256: String,
    new_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiffLeaf {
    path: String,
    old_mode: String,
    new_mode: String,
    old_oid: String,
    new_oid: String,
}

pub(in crate::coord) fn verify_recovery_commit(
    family_root: &Path,
    repo: &str,
    expected: &RecoveryGitExpectationV1,
) -> Result<(), CoordError> {
    validate_repo_name(repo)?;
    let before = family_selection(family_root, repo)?;
    let first = observe(&before.checkout, untag(&expected.commit_oid, "sha1:")?)?;
    verify_expected(&first, expected)?;
    after_first_read();
    let middle = family_selection(family_root, repo)?;
    let second = observe(&middle.checkout, untag(&expected.commit_oid, "sha1:")?)?;
    verify_expected(&second, expected)?;
    let after = family_selection(family_root, repo)?;
    if before != middle || middle != after || first != second {
        return Err(mismatch(
            "repository identity or Git objects changed across exact double read-back",
        ));
    }
    Ok(())
}

#[cfg(not(test))]
fn after_first_read() {}

#[cfg(test)]
thread_local! {
    static AFTER_FIRST_READ: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const {
        std::cell::RefCell::new(None)
    };
}

#[cfg(test)]
fn after_first_read() {
    AFTER_FIRST_READ.with(|hook| {
        if let Some(hook) = hook.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(test)]
fn test_after_first_read(hook: impl FnOnce() + 'static) {
    AFTER_FIRST_READ.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
}

fn observe(root: &Path, commit_oid: &str) -> Result<CommitObservation, CoordError> {
    let object_format = match text(root, &["rev-parse", "--show-object-format"])? {
        value if value == "sha1" => RecoveryGitObjectFormatV1::Sha1,
        _ => return Err(mismatch("repository object format is not SHA-1")),
    };
    let raw_commit_bytes = bytes(root, &["cat-file", "commit", commit_oid])?;
    require_object_hash(root, "commit", &raw_commit_bytes, commit_oid)?;
    let (result_tree_oid, parent_oid) = parse_commit(&raw_commit_bytes)?;
    let raw_parent = bytes(root, &["cat-file", "commit", &parent_oid])?;
    require_object_hash(root, "commit", &raw_parent, &parent_oid)?;
    let parent_tree_oid = parse_tree(&raw_parent)?;
    let raw_tree = bytes(root, &["cat-file", "tree", &result_tree_oid])?;
    require_object_hash(root, "tree", &raw_tree, &result_tree_oid)?;
    let diff = bytes(
        root,
        &[
            "diff-tree",
            "--no-commit-id",
            "--raw",
            "-r",
            "-z",
            "--no-abbrev",
            "--no-renames",
            "--no-ext-diff",
            &parent_oid,
            commit_oid,
        ],
    )?;
    let mut leaves = parse_diff(&diff)?
        .into_iter()
        .map(|leaf| observe_leaf(root, leaf))
        .collect::<Result<Vec<_>, _>>()?;
    leaves.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(CommitObservation {
        object_format,
        commit_oid: tagged("sha1:", commit_oid),
        raw_commit_sha256: sha256(&raw_commit_bytes),
        raw_commit_bytes,
        parent_oid: tagged("sha1:", &parent_oid),
        parent_tree_oid: tagged("sha1:", &parent_tree_oid),
        result_tree_oid: tagged("sha1:", &result_tree_oid),
        raw_tree_sha256: sha256(&raw_tree),
        leaves,
    })
}

fn observe_leaf(root: &Path, leaf: DiffLeaf) -> Result<LeafObservation, CoordError> {
    if !matches!(leaf.old_mode.as_str(), "100644" | "100755")
        || !matches!(leaf.new_mode.as_str(), "100644" | "100755")
    {
        return Err(mismatch(format!(
            "Git leaf {} is not a regular-file modification",
            leaf.path
        )));
    }
    let old_bytes = bytes(root, &["cat-file", "blob", &leaf.old_oid])?;
    let new_bytes = bytes(root, &["cat-file", "blob", &leaf.new_oid])?;
    require_object_hash(root, "blob", &old_bytes, &leaf.old_oid)?;
    require_object_hash(root, "blob", &new_bytes, &leaf.new_oid)?;
    Ok(LeafObservation {
        path: leaf.path,
        old_mode: leaf.old_mode,
        new_mode: leaf.new_mode,
        old_blob_oid: tagged("sha1:", &leaf.old_oid),
        new_blob_oid: tagged("sha1:", &leaf.new_oid),
        old_sha256: sha256(&old_bytes),
        new_sha256: sha256(&new_bytes),
        old_bytes,
        new_bytes,
    })
}

fn verify_expected(
    observed: &CommitObservation,
    expected: &RecoveryGitExpectationV1,
) -> Result<(), CoordError> {
    let expected_leaves = expected
        .leaf_transitions
        .iter()
        .map(|leaf| LeafObservation {
            path: leaf.path.clone(),
            old_mode: leaf.old_mode.clone(),
            new_mode: leaf.new_mode.clone(),
            old_blob_oid: leaf.old_blob_oid.clone(),
            new_blob_oid: leaf.new_blob_oid.clone(),
            old_bytes: leaf.old_bytes.clone(),
            new_bytes: leaf.new_bytes.clone(),
            old_sha256: leaf.old_sha256.clone(),
            new_sha256: leaf.new_sha256.clone(),
        })
        .collect::<Vec<_>>();
    if observed.object_format != expected.object_format
        || observed.commit_oid != expected.commit_oid
        || observed.raw_commit_bytes != expected.raw_commit_bytes
        || observed.raw_commit_sha256 != expected.raw_commit_sha256
        || observed.parent_oid != expected.parent_oid
        || observed.parent_tree_oid != expected.parent_tree_oid
        || observed.result_tree_oid != expected.result_tree_oid
        || observed.raw_tree_sha256 != expected.raw_tree_sha256
        || observed.leaves != expected_leaves
    {
        return Err(mismatch(
            "Git commit, parent, tree, or complete leaf transition differs",
        ));
    }
    Ok(())
}

fn parse_commit(raw: &[u8]) -> Result<(String, String), CoordError> {
    let tree = parse_tree(raw)?;
    let header_end = raw
        .windows(2)
        .position(|window| window == b"\n\n")
        .ok_or_else(|| mismatch("raw commit lacks a header terminator"))?;
    let header = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| mismatch("raw commit header is not UTF-8"))?;
    let parents = header
        .lines()
        .filter_map(|line| line.strip_prefix("parent "))
        .collect::<Vec<_>>();
    if parents.len() != 1 {
        return Err(mismatch(
            "recovery commit must have exactly one tree and one parent",
        ));
    }
    validate_oid(parents[0])?;
    Ok((tree, parents[0].to_owned()))
}

fn parse_tree(raw: &[u8]) -> Result<String, CoordError> {
    let header_end = raw
        .windows(2)
        .position(|window| window == b"\n\n")
        .ok_or_else(|| mismatch("raw commit lacks a header terminator"))?;
    let header = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| mismatch("raw commit header is not UTF-8"))?;
    let trees = header
        .lines()
        .filter_map(|line| line.strip_prefix("tree "))
        .collect::<Vec<_>>();
    if trees.len() != 1 {
        return Err(mismatch("Git commit must have exactly one tree"));
    }
    validate_oid(trees[0])?;
    Ok(trees[0].to_owned())
}

fn parse_diff(raw: &[u8]) -> Result<Vec<DiffLeaf>, CoordError> {
    if raw.is_empty() || raw.last() != Some(&0) {
        return Err(mismatch("raw Git diff is empty or lacks its terminal NUL"));
    }
    let fields = raw[..raw.len() - 1]
        .split(|byte| *byte == 0)
        .collect::<Vec<_>>();
    if fields.len() % 2 != 0 {
        return Err(mismatch("raw Git diff does not contain header/path pairs"));
    }
    let mut leaves = Vec::with_capacity(fields.len() / 2);
    for pair in fields.chunks_exact(2) {
        let header = std::str::from_utf8(pair[0])
            .map_err(|_| mismatch("raw Git diff header is not UTF-8"))?;
        let path =
            std::str::from_utf8(pair[1]).map_err(|_| mismatch("raw Git diff path is not UTF-8"))?;
        let parts = header
            .strip_prefix(':')
            .ok_or_else(|| mismatch("raw Git diff header lacks its colon"))?
            .split(' ')
            .collect::<Vec<_>>();
        if parts.len() != 5 || parts[4] != "M" {
            return Err(mismatch(
                "recovery Git diff contains a non-modification leaf",
            ));
        }
        validate_oid(parts[2])?;
        validate_oid(parts[3])?;
        let normalized = validate_path(path)?;
        if normalized != path {
            return Err(mismatch("raw Git diff path is not canonical"));
        }
        leaves.push(DiffLeaf {
            path: normalized,
            old_mode: parts[0].to_owned(),
            new_mode: parts[1].to_owned(),
            old_oid: parts[2].to_owned(),
            new_oid: parts[3].to_owned(),
        });
    }
    leaves.sort_by(|left, right| left.path.cmp(&right.path));
    if leaves.windows(2).any(|pair| pair[0].path == pair[1].path) {
        return Err(mismatch("raw Git diff contains a duplicate leaf"));
    }
    Ok(leaves)
}

fn repository_identity(root: &Path) -> Result<RepositoryIdentity, CoordError> {
    let root_metadata = fs::symlink_metadata(root).map_err(CoordError::io)?;
    let git_metadata = fs::symlink_metadata(root.join(".git")).map_err(CoordError::io)?;
    if !root_metadata.is_dir()
        || root_metadata.file_type().is_symlink()
        || !git_metadata.is_dir()
        || git_metadata.file_type().is_symlink()
    {
        return Err(mismatch(
            "recovery Git subject must be a primary repository directory",
        ));
    }
    Ok(RepositoryIdentity {
        root_device: root_metadata.dev(),
        root_inode: root_metadata.ino(),
        git_device: git_metadata.dev(),
        git_inode: git_metadata.ino(),
        object_store_blake3: object_store::inventory(root)?,
    })
}

fn bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, CoordError> {
    let output = run_bounded(
        command(root).args(args),
        "recovery Git read-back",
        GIT_LIMITS,
    )?;
    if !output.status.success() {
        return Err(mismatch(format!(
            "Git read-back failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}

fn text(root: &Path, args: &[&str]) -> Result<String, CoordError> {
    let output = bytes(root, args)?;
    let value = std::str::from_utf8(&output)
        .map_err(|_| mismatch("Git text output is not UTF-8"))?
        .strip_suffix('\n')
        .ok_or_else(|| mismatch("Git text output lacks one terminal LF"))?;
    if value.contains('\n') || value.contains('\r') {
        return Err(mismatch("Git text output contains extra lines"));
    }
    Ok(value.to_owned())
}

fn require_object_hash(
    root: &Path,
    object_type: &str,
    raw: &[u8],
    expected: &str,
) -> Result<(), CoordError> {
    let mut input = tempfile::tempfile().map_err(CoordError::io)?;
    input.write_all(raw).map_err(CoordError::io)?;
    input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let output = run_bounded_with_input_file(
        command(root).args(["hash-object", "-t", object_type, "--stdin"]),
        "recovery Git object rehash",
        GIT_LIMITS,
        input,
    )?;
    if !output.output.status.success()
        || output.byte_count != raw.len() as u64
        || std::str::from_utf8(&output.output.stdout)
            .ok()
            .and_then(|value| value.strip_suffix('\n'))
            != Some(expected)
    {
        return Err(mismatch(format!(
            "{object_type} object does not rehash to its expected OID"
        )));
    }
    Ok(())
}

fn command(root: &Path) -> Command {
    let mut command = Command::new(GIT_BIN);
    command
        .arg("-C")
        .arg(root)
        .arg("--no-replace-objects")
        .env_clear()
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_OPTIONAL_LOCKS", "0");
    command
}

fn untag<'a>(value: &'a str, prefix: &str) -> Result<&'a str, CoordError> {
    value
        .strip_prefix(prefix)
        .ok_or_else(|| mismatch("Git expectation contains an untagged OID"))
}

fn validate_oid(value: &str) -> Result<(), CoordError> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(mismatch("Git emitted an invalid SHA-1 OID"))
    }
}

fn tagged(prefix: &str, value: &str) -> String {
    format!("{prefix}{value}")
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_GIT_EVIDENCE_MISMATCH", reason)
}

#[cfg(test)]
#[path = "git/tests.rs"]
mod tests;
