use std::{fs, path::Path, process::Command};

use sha2::{Digest, Sha256};

use crate::coord::model::{
    ForensicRecordRefV1, RecoveryForensicArtifactKindV1, RecoveryForensicRecordKindV1,
    RecoveryGitExpectationV1, RecoveryGitLeafStatusV1, RecoveryGitLeafTransitionV1,
    RecoveryGitObjectFormatV1,
};

pub(super) fn git_fixture() -> (tempfile::TempDir, RecoveryGitExpectationV1) {
    let source = tempfile::tempdir().unwrap();
    git(source.path(), &["init", "--quiet"]);
    fs::write(source.path().join("a.txt"), b"old-a\n").unwrap();
    fs::write(source.path().join("b.txt"), b"old-b\n").unwrap();
    git(source.path(), &["add", "--", "a.txt", "b.txt"]);
    commit(source.path(), "parent");
    let parent_oid = text(source.path(), &["rev-parse", "HEAD"]);
    let parent_tree_oid = text(source.path(), &["rev-parse", "HEAD^{tree}"]);
    let old = [
        text(source.path(), &["rev-parse", "HEAD:a.txt"]),
        text(source.path(), &["rev-parse", "HEAD:b.txt"]),
    ];
    fs::write(source.path().join("a.txt"), b"new-a\n").unwrap();
    fs::write(source.path().join("b.txt"), b"new-b\n").unwrap();
    git(source.path(), &["add", "--", "a.txt", "b.txt"]);
    commit(source.path(), "subject");
    let commit_oid = text(source.path(), &["rev-parse", "HEAD"]);
    let result_tree_oid = text(source.path(), &["rev-parse", "HEAD^{tree}"]);
    let new = [
        text(source.path(), &["rev-parse", "HEAD:a.txt"]),
        text(source.path(), &["rev-parse", "HEAD:b.txt"]),
    ];
    let raw_commit = git(source.path(), &["cat-file", "commit", &commit_oid]);
    let raw_tree = git(source.path(), &["cat-file", "tree", &result_tree_oid]);
    (
        source,
        RecoveryGitExpectationV1 {
            object_format: RecoveryGitObjectFormatV1::Sha1,
            commit_oid: oid(&commit_oid),
            raw_commit_sha256: sha(&raw_commit),
            raw_commit_bytes: raw_commit,
            parent_oid: oid(&parent_oid),
            parent_tree_oid: oid(&parent_tree_oid),
            parent_receipt_observation: placeholder_forensic(),
            result_tree_oid: oid(&result_tree_oid),
            raw_tree_sha256: sha(&raw_tree),
            leaf_transitions: vec![
                leaf("a.txt", &old[0], &new[0], b"old-a\n", b"new-a\n"),
                leaf("b.txt", &old[1], &new[1], b"old-b\n", b"new-b\n"),
            ],
        },
    )
}

pub(super) fn clone_repo(source: &Path, target: &Path) {
    let output = Command::new("/usr/bin/git")
        .args(["clone", "--quiet", "--no-local", "--no-hardlinks"])
        .arg(source)
        .arg(target)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(output.status.success());
    fs::write(
        target.parent().unwrap().join("repos.manifest.toml"),
        format!(
            "[[repo]]\nname = \"bullet-kernel\"\npath = {:?}\n",
            target.display().to_string()
        ),
    )
    .unwrap();
}

fn leaf(
    path: &str,
    old: &str,
    new: &str,
    old_bytes: &[u8],
    new_bytes: &[u8],
) -> RecoveryGitLeafTransitionV1 {
    RecoveryGitLeafTransitionV1 {
        status: RecoveryGitLeafStatusV1::Modified,
        path: path.to_owned(),
        old_mode: "100644".to_owned(),
        new_mode: "100644".to_owned(),
        old_blob_oid: oid(old),
        new_blob_oid: oid(new),
        old_bytes: old_bytes.to_vec(),
        new_bytes: new_bytes.to_vec(),
        old_sha256: sha(old_bytes),
        new_sha256: sha(new_bytes),
    }
}

fn placeholder_forensic() -> ForensicRecordRefV1 {
    ForensicRecordRefV1 {
        artifact_kind: RecoveryForensicArtifactKindV1::TrustedPrefix,
        artifact_sha256: sha(b"placeholder artifact"),
        record_index: 1,
        byte_start: 1,
        byte_end: 2,
        record_sha256: sha(b"placeholder record"),
        expected_record_kind: RecoveryForensicRecordKindV1::CommitReceipt,
    }
}

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn text(root: &Path, args: &[&str]) -> String {
    String::from_utf8(git(root, args))
        .unwrap()
        .trim_end_matches('\n')
        .to_owned()
}

fn commit(root: &Path, message: &str) {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(root)
        .args(["commit", "--quiet", "-m", message])
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2001-01-01T00:00:00Z")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_DATE", "2001-01-01T00:00:00Z")
        .output()
        .unwrap();
    assert!(output.status.success());
}

fn oid(value: &str) -> String {
    format!("sha1:{value}")
}

fn sha(value: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(value))
}
