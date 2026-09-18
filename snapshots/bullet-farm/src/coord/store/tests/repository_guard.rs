use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize},
    },
};

use crate::coord::{
    ClaimInput, CommitReceiptGroupInput, CommitReceiptInput, GenerationId, HandoffInput,
    MutationEnvelope,
};

use super::super::Ledger;
use super::{CoordStore, claim, genesis, request, store};

pub(super) fn run() {
    claim_refuses_manifest_invalidation();
    receipt_refuses_manifest_invalidation();
    group_refuses_repository_swap();
}

fn claim_refuses_manifest_invalidation() {
    let (root, store, generation, segment) = initialized_store();
    let before = fs::metadata(&segment).unwrap().len();
    let manifest = root.path().join("repos.manifest.toml");
    Ledger::test_before_subject_guard(move || fs::write(manifest, "").unwrap());
    assert_eq!(
        store
            .claim(&claim(request(10), generation, "claim-agent"))
            .unwrap_err()
            .code(),
        "REPOSITORY_IDENTITY_MISMATCH"
    );
    assert_eq!(fs::metadata(segment).unwrap().len(), before);
}

fn receipt_refuses_manifest_invalidation() {
    let (root, store, generation, segment) = initialized_store();
    let claimed = store
        .claim(&claim(request(20), generation.clone(), "receipt-agent"))
        .unwrap();
    handoff(
        &store,
        generation.clone(),
        21,
        &claimed.projection.claim_id,
        "receipt-agent",
        &["src/receipt.rs"],
    );
    let commit_oid = commit(root.path(), &[("src/receipt.rs", "receipt\n")]);
    let before = fs::metadata(&segment).unwrap().len();
    let manifest = root.path().join("repos.manifest.toml");
    Ledger::test_before_subject_guard(move || fs::write(manifest, "[[repo]").unwrap());
    assert_eq!(
        store
            .receipt(&MutationEnvelope {
                request_id: request(22),
                expected_generation_id: generation,
                command: CommitReceiptInput {
                    claim_id: claimed.projection.claim_id,
                    orchestrator: "orchestrator".to_owned(),
                    commit_oid,
                    committed_paths: vec!["src/receipt.rs".to_owned()],
                },
            })
            .unwrap_err()
            .code(),
        "REPOSITORY_IDENTITY_MISMATCH"
    );
    assert_eq!(fs::metadata(segment).unwrap().len(), before);
}

fn group_refuses_repository_swap() {
    let (root, store, generation, segment) = initialized_store();
    let first = scoped_claim(&store, generation.clone(), 30, "first", "src/first.rs");
    let second = scoped_claim(&store, generation.clone(), 31, "second", "src/second.rs");
    handoff(
        &store,
        generation.clone(),
        32,
        &first,
        "first",
        &["src/first.rs"],
    );
    handoff(
        &store,
        generation.clone(),
        33,
        &second,
        "second",
        &["src/second.rs"],
    );
    let commit_oid = commit(
        root.path(),
        &[("src/first.rs", "first\n"), ("src/second.rs", "second\n")],
    );
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            "bullet-farm",
            "replacement",
        ],
    );
    let before = fs::metadata(&segment).unwrap().len();
    let canonical = root.path().join("bullet-farm");
    let replacement = root.path().join("replacement");
    let displaced = root.path().join("displaced");
    Ledger::test_before_subject_guard(move || {
        fs::rename(&canonical, displaced).unwrap();
        fs::rename(replacement, canonical).unwrap();
    });
    assert_eq!(
        store
            .receipt_group(&MutationEnvelope {
                request_id: request(34),
                expected_generation_id: generation,
                command: CommitReceiptGroupInput {
                    claim_ids: vec![first, second],
                    orchestrator: "orchestrator".to_owned(),
                    commit_oid,
                },
            })
            .unwrap_err()
            .code(),
        "REPOSITORY_IDENTITY_MISMATCH"
    );
    assert_eq!(fs::metadata(segment).unwrap().len(), before);
}

fn initialized_store() -> (tempfile::TempDir, CoordStore, GenerationId, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("bullet-farm");
    fs::create_dir(&repo).unwrap();
    let manifest = format!(
        "[[repo]]\nname = \"bullet-farm\"\npath = {}\n",
        serde_json::to_string(repo.to_str().unwrap()).unwrap()
    );
    fs::write(root.path().join("repos.manifest.toml"), manifest).unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["config", "user.name", "Coord Test"]);
    git(&repo, &["config", "user.email", "coord@example.invalid"]);
    let store = store(
        &root,
        Arc::new(AtomicU64::new(1_000)),
        Arc::new(AtomicUsize::new(0)),
    );
    let initialized = store.initialize(&genesis()).unwrap();
    let generation = GenerationId::parse(initialized.generation_id).unwrap();
    let segment = root
        .path()
        .join(".bullet-family/coord/generations")
        .join(generation.as_str())
        .join("events.jsonl");
    (root, store, generation, segment)
}

fn scoped_claim(
    store: &CoordStore,
    generation: GenerationId,
    index: u8,
    agent: &str,
    path: &str,
) -> String {
    store
        .claim(&MutationEnvelope {
            request_id: request(index),
            expected_generation_id: generation,
            command: ClaimInput {
                agent: agent.to_owned(),
                lane: format!("lane-{agent}"),
                repo: "bullet-farm".to_owned(),
                paths: vec![path.to_owned()],
                ttl_seconds: 600,
            },
        })
        .unwrap()
        .projection
        .claim_id
}

fn handoff(
    store: &CoordStore,
    generation: GenerationId,
    index: u8,
    claim_id: &str,
    agent: &str,
    paths: &[&str],
) {
    store
        .handoff(&MutationEnvelope {
            request_id: request(index),
            expected_generation_id: generation,
            command: HandoffInput {
                claim_id: claim_id.to_owned(),
                agent: agent.to_owned(),
                proof_command: "cargo test --locked".to_owned(),
                proof_exit_code: 0,
                changed_paths: paths.iter().map(|path| (*path).to_owned()).collect(),
                commit_oid: None,
            },
        })
        .unwrap();
}

fn commit(root: &Path, files: &[(&str, &str)]) -> String {
    let repo = root.join("bullet-farm");
    for (path, contents) in files {
        let target = repo.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, contents).unwrap();
        git(&repo, &["add", path]);
    }
    git(&repo, &["commit", "--quiet", "-m", "fixture"]);
    git(&repo, &["rev-parse", "HEAD"])
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
