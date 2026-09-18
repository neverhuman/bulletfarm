use std::{fs, os::unix::fs::PermissionsExt, process::Command};

use sha2::{Digest, Sha256};

use super::*;
use crate::coord::model::{
    ForensicRecordRefV1, RecoveryForensicArtifactKindV1, RecoveryForensicRecordKindV1,
    RecoveryGitLeafStatusV1, RecoveryGitLeafTransitionV1,
};

struct Fixture {
    family: tempfile::TempDir,
    expected: RecoveryGitExpectationV1,
}

fn fixture() -> Fixture {
    let family = tempfile::tempdir().unwrap();
    let root = family.path().join("bullet-kernel");
    fs::create_dir(&root).unwrap();
    write_manifest(family.path(), "bullet-kernel", &root);
    git(&root, &["init", "--quiet"]);
    git(&root, &["config", "user.name", "fixture"]);
    git(&root, &["config", "user.email", "fixture@example.invalid"]);
    fs::write(root.join("a.txt"), b"old-a\n").unwrap();
    fs::write(root.join("b.txt"), b"old-b\n").unwrap();
    git(&root, &["add", "--", "a.txt", "b.txt"]);
    commit(&root, "parent");
    let parent_oid = text(&root, &["rev-parse", "HEAD"]);
    let parent_tree_oid = text(&root, &["rev-parse", "HEAD^{tree}"]);
    let old_a = text(&root, &["rev-parse", "HEAD:a.txt"]);
    let old_b = text(&root, &["rev-parse", "HEAD:b.txt"]);

    fs::write(root.join("a.txt"), b"new-a\n").unwrap();
    fs::write(root.join("b.txt"), b"new-b\n").unwrap();
    git(&root, &["add", "--", "a.txt", "b.txt"]);
    commit(&root, "subject");
    let commit_oid = text(&root, &["rev-parse", "HEAD"]);
    let result_tree_oid = text(&root, &["rev-parse", "HEAD^{tree}"]);
    let new_a = text(&root, &["rev-parse", "HEAD:a.txt"]);
    let new_b = text(&root, &["rev-parse", "HEAD:b.txt"]);
    let raw_commit = git(&root, &["cat-file", "commit", &commit_oid]);
    let raw_tree = git(&root, &["cat-file", "tree", &result_tree_oid]);
    let expected = RecoveryGitExpectationV1 {
        object_format: RecoveryGitObjectFormatV1::Sha1,
        commit_oid: tag(&commit_oid),
        raw_commit_sha256: sha(&raw_commit),
        raw_commit_bytes: raw_commit,
        parent_oid: tag(&parent_oid),
        parent_tree_oid: tag(&parent_tree_oid),
        parent_receipt_observation: ForensicRecordRefV1 {
            artifact_kind: RecoveryForensicArtifactKindV1::TrustedPrefix,
            artifact_sha256: format!("sha256:{}", "a".repeat(64)),
            record_index: 1,
            byte_start: 1,
            byte_end: 2,
            record_sha256: format!("sha256:{}", "b".repeat(64)),
            expected_record_kind: RecoveryForensicRecordKindV1::CommitReceipt,
        },
        result_tree_oid: tag(&result_tree_oid),
        raw_tree_sha256: sha(&raw_tree),
        leaf_transitions: vec![
            leaf("a.txt", &old_a, &new_a, b"old-a\n", b"new-a\n"),
            leaf("b.txt", &old_b, &new_b, b"old-b\n", b"new-b\n"),
        ],
    };
    Fixture { family, expected }
}

fn leaf(
    path: &str,
    old_oid: &str,
    new_oid: &str,
    old_bytes: &[u8],
    new_bytes: &[u8],
) -> RecoveryGitLeafTransitionV1 {
    RecoveryGitLeafTransitionV1 {
        status: RecoveryGitLeafStatusV1::Modified,
        path: path.to_owned(),
        old_mode: "100644".to_owned(),
        new_mode: "100644".to_owned(),
        old_blob_oid: tag(old_oid),
        new_blob_oid: tag(new_oid),
        old_bytes: old_bytes.to_vec(),
        new_bytes: new_bytes.to_vec(),
        old_sha256: sha(old_bytes),
        new_sha256: sha(new_bytes),
    }
}

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new(GIT_BIN)
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
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn write_manifest(family: &Path, name: &str, path: &Path) {
    fs::write(
        family.join("repos.manifest.toml"),
        format!(
            "[[repo]]\nname = \"{name}\"\npath = {:?}\n",
            path.display().to_string()
        ),
    )
    .unwrap();
}

fn text(root: &Path, args: &[&str]) -> String {
    String::from_utf8(git(root, args))
        .unwrap()
        .trim_end_matches('\n')
        .to_owned()
}

fn commit(root: &Path, message: &str) {
    let output = Command::new(GIT_BIN)
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

fn tag(oid: &str) -> String {
    format!("sha1:{oid}")
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[test]
fn exact_commit_and_complete_leaf_map_pass_two_reads() {
    let fixture = fixture();
    verify_recovery_commit(fixture.family.path(), "bullet-kernel", &fixture.expected).unwrap();
}

#[test]
fn missing_extra_or_changed_leaf_evidence_refuses() {
    let fixture = fixture();
    let mut missing = fixture.expected.clone();
    missing.leaf_transitions.pop();
    assert_eq!(
        verify_recovery_commit(fixture.family.path(), "bullet-kernel", &missing)
            .unwrap_err()
            .code(),
        "RECOVERY_GIT_EVIDENCE_MISMATCH"
    );

    let mut changed = fixture.expected;
    changed.leaf_transitions[0].new_bytes.push(b'x');
    assert_eq!(
        verify_recovery_commit(fixture.family.path(), "bullet-kernel", &changed)
            .unwrap_err()
            .code(),
        "RECOVERY_GIT_EVIDENCE_MISMATCH"
    );
}

#[test]
fn commit_parent_and_tree_substitutions_refuse() {
    let fixture = fixture();
    let mut cases = [
        fixture.expected.clone(),
        fixture.expected.clone(),
        fixture.expected.clone(),
    ];
    cases[0].raw_commit_bytes[0] = b'T';
    cases[1].parent_oid = format!("sha1:{}", "0".repeat(40));
    cases[2].raw_tree_sha256 = format!("sha256:{}", "0".repeat(64));
    for changed in cases {
        assert_eq!(
            verify_recovery_commit(fixture.family.path(), "bullet-kernel", &changed)
                .unwrap_err()
                .code(),
            "RECOVERY_GIT_EVIDENCE_MISMATCH"
        );
    }
}

#[test]
fn non_primary_repository_path_refuses_before_git() {
    let fixture = fixture();
    let linked = fixture.family.path().join("linked");
    std::os::unix::fs::symlink(fixture.family.path().join("bullet-kernel"), &linked).unwrap();
    assert_eq!(
        verify_recovery_commit(fixture.family.path(), "linked", &fixture.expected)
            .unwrap_err()
            .code(),
        "RECOVERY_GIT_EVIDENCE_MISMATCH"
    );
}

#[test]
fn unlisted_duplicate_and_mismatched_manifest_entries_refuse() {
    let fixture = fixture();
    let manifest = fixture.family.path().join("repos.manifest.toml");
    let repo = fixture.family.path().join("bullet-kernel");
    for body in [
        format!(
            "[[repo]]\nname = \"other\"\npath = {:?}\n",
            repo.display().to_string()
        ),
        format!(
            "[[repo]]\nname = \"bullet-kernel\"\npath = {:?}\n[[repo]]\nname = \"bullet-kernel\"\npath = {:?}\n",
            repo.display().to_string(),
            repo.display().to_string()
        ),
        format!(
            "[[repo]]\nname = \"bullet-kernel\"\npath = {:?}\n",
            fixture
                .family
                .path()
                .join("elsewhere")
                .display()
                .to_string()
        ),
    ] {
        fs::write(&manifest, body).unwrap();
        assert_eq!(
            verify_recovery_commit(fixture.family.path(), "bullet-kernel", &fixture.expected)
                .unwrap_err()
                .code(),
            "RECOVERY_GIT_EVIDENCE_MISMATCH"
        );
    }
}

#[test]
fn manifest_symlink_or_replacement_between_reads_refuses() {
    let fixture = fixture();
    let manifest = fixture.family.path().join("repos.manifest.toml");
    let retained = fixture.family.path().join("manifest-retained.toml");
    fs::rename(&manifest, &retained).unwrap();
    std::os::unix::fs::symlink(&retained, &manifest).unwrap();
    assert_eq!(
        verify_recovery_commit(fixture.family.path(), "bullet-kernel", &fixture.expected)
            .unwrap_err()
            .code(),
        "RECOVERY_GIT_EVIDENCE_MISMATCH"
    );

    fs::remove_file(&manifest).unwrap();
    fs::copy(&retained, &manifest).unwrap();
    let replaced = manifest.clone();
    test_after_first_read(move || {
        let bytes = fs::read(&replaced).unwrap();
        fs::remove_file(&replaced).unwrap();
        fs::write(&replaced, bytes).unwrap();
    });
    assert_eq!(
        verify_recovery_commit(fixture.family.path(), "bullet-kernel", &fixture.expected)
            .unwrap_err()
            .code(),
        "RECOVERY_GIT_EVIDENCE_MISMATCH"
    );
}

#[test]
fn checkout_or_object_store_replacement_between_reads_refuses() {
    let checkout_fixture = fixture();
    let root = checkout_fixture.family.path().join("bullet-kernel");
    let replacement = checkout_fixture.family.path().join("replacement");
    clone_repository(&root, &replacement);
    let displaced = checkout_fixture.family.path().join("displaced");
    test_after_first_read({
        let root = root.clone();
        move || {
            fs::rename(&root, &displaced).unwrap();
            fs::rename(&replacement, &root).unwrap();
        }
    });
    assert_mismatch(verify_recovery_commit(
        checkout_fixture.family.path(),
        "bullet-kernel",
        &checkout_fixture.expected,
    ));

    let object_fixture = fixture();
    let root = object_fixture.family.path().join("bullet-kernel");
    test_after_first_read({
        let objects = root.join(".git/objects");
        move || {
            fs::create_dir_all(objects.join("aa")).unwrap();
            fs::write(objects.join("aa/unrelated-object"), b"sealed hostile").unwrap();
        }
    });
    assert_mismatch(verify_recovery_commit(
        object_fixture.family.path(),
        "bullet-kernel",
        &object_fixture.expected,
    ));
}

#[test]
fn alternate_object_store_refuses_without_changing_either_store() {
    let fixture = fixture();
    let root = fixture.family.path().join("bullet-kernel");
    let selected_objects = root.join(".git/objects");
    let alternate_objects = fixture.family.path().join("alternate-objects");
    fs::rename(&selected_objects, &alternate_objects).unwrap();
    fs::create_dir(&selected_objects).unwrap();
    fs::create_dir(selected_objects.join("info")).unwrap();
    fs::write(
        selected_objects.join("info/alternates"),
        format!("{}\n", alternate_objects.display()),
    )
    .unwrap();
    let commit = fixture.expected.commit_oid.strip_prefix("sha1:").unwrap();
    let alternate_commit = alternate_objects.join(&commit[..2]).join(&commit[2..]);
    let retained = fs::read(&alternate_commit).unwrap();

    assert_mismatch(verify_recovery_commit(
        fixture.family.path(),
        "bullet-kernel",
        &fixture.expected,
    ));
    assert_eq!(fs::read(&alternate_commit).unwrap(), retained);
    assert!(
        !selected_objects
            .join(&commit[..2])
            .join(&commit[2..])
            .exists()
    );
}

#[test]
fn promisor_missing_object_never_invokes_fetch_or_changes_objects() {
    let fixture = fixture();
    let root = fixture.family.path().join("bullet-kernel");
    let marker = fixture.family.path().join("lazy-fetch-invoked");
    let helper = fixture.family.path().join("upload-pack-hostile.sh");
    fs::write(
        &helper,
        format!("#!/bin/sh\n/usr/bin/touch {}\nexit 1\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    git(&root, &["config", "core.repositoryFormatVersion", "1"]);
    git(&root, &["config", "extensions.partialClone", "origin"]);
    git(&root, &["config", "remote.origin.promisor", "true"]);
    git(
        &root,
        &["config", "remote.origin.partialCloneFilter", "blob:none"],
    );
    git(
        &root,
        &[
            "config",
            "remote.origin.url",
            fixture
                .family
                .path()
                .join("absent-remote")
                .to_str()
                .unwrap(),
        ],
    );
    git(
        &root,
        &[
            "config",
            "remote.origin.uploadpack",
            helper.to_str().unwrap(),
        ],
    );
    let before = object_store::inventory(&root).unwrap();
    let mut missing = fixture.expected;
    missing.commit_oid = format!("sha1:{}", "0".repeat(40));

    assert_mismatch(verify_recovery_commit(
        fixture.family.path(),
        "bullet-kernel",
        &missing,
    ));
    assert!(!marker.exists());
    assert_eq!(object_store::inventory(&root).unwrap(), before);
}

fn clone_repository(source: &Path, destination: &Path) {
    let output = Command::new(GIT_BIN)
        .args(["clone", "--quiet"])
        .arg(source)
        .arg(destination)
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
}

fn assert_mismatch(result: Result<(), CoordError>) {
    assert_eq!(result.unwrap_err().code(), "RECOVERY_GIT_EVIDENCE_MISMATCH");
}
