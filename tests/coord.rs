use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Barrier},
    thread,
};

use bullet_family::coord::{
    ClaimInput, ClaimState, CommitReceiptGroupInput, CommitReceiptInput, CoordStore,
    GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput, ReceiptCorrectionInput,
};

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-family-{name}-{}-{:?}",
        std::process::id(),
        thread::current().id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    let repo = root.join("bullet-farm");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["config", "user.name", "Coord Test"]);
    git(&repo, &["config", "user.email", "coord@example.invalid"]);
    root
}

fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("UTF-8 git output")
        .trim()
        .to_owned()
}

fn commit(root: &std::path::Path, path: &str, contents: &str) -> String {
    commit_many(root, &[(path, contents)])
}

fn commit_many(root: &std::path::Path, files: &[(&str, &str)]) -> String {
    let repo = root.join("bullet-farm");
    for (path, contents) in files {
        let target = repo.join(path);
        fs::create_dir_all(target.parent().expect("file parent")).unwrap();
        fs::write(target, contents).unwrap();
        git(&repo, &["add", path]);
    }
    git(&repo, &["commit", "--quiet", "-m", "fixture"]);
    git(&repo, &["rev-parse", "HEAD"])
}

fn claim(agent: &str, path: &str) -> ClaimInput {
    ClaimInput {
        agent: agent.to_owned(),
        lane: format!("lane-{agent}"),
        repo: "bullet-farm".to_owned(),
        paths: vec![path.to_owned()],
        ttl_seconds: 600,
    }
}

#[test]
fn lifecycle_is_durable_and_handoff_binds_changed_paths() {
    let root = test_root("lifecycle");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let heartbeat = store
        .heartbeat(
            &HeartbeatInput {
                claim_id: claimed.claim_id.clone(),
                agent: "agent-a".to_owned(),
                ttl_seconds: 600,
                note: Some("proof started".to_owned()),
            },
            2_000,
        )
        .unwrap();
    assert!(heartbeat.expires_unix_ms > claimed.expires_unix_ms);

    let handed_off = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test --locked".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/main.rs".to_owned()],
                commit_oid: None,
            },
            3_000,
        )
        .unwrap();
    assert_eq!(handed_off.state, ClaimState::HandedOff);
    let commit_oid = commit(&root, "src/main.rs", "fn main() {}\n");
    let receipted = store
        .receipt(
            &CommitReceiptInput {
                claim_id: handed_off.claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid: commit_oid.clone(),
                committed_paths: vec!["src/main.rs".to_owned()],
            },
            4_000,
        )
        .unwrap();
    assert_eq!(receipted.commit_oid.as_deref(), Some(commit_oid.as_str()));
    assert_eq!(
        receipted.commit_orchestrator.as_deref(),
        Some("orchestrator")
    );
    assert_eq!(store.status(5_000).unwrap().claims.len(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn overlap_and_out_of_claim_handoff_fail_closed() {
    let root = test_root("overlap");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let overlap = store.claim(&claim("agent-b", "src/main.rs"), 1_001);
    assert_eq!(overlap.unwrap_err().code(), "CLAIM_OVERLAP");
    let outside = store.handoff(
        &HandoffInput {
            claim_id: claimed.claim_id,
            agent: "agent-a".to_owned(),
            proof_command: "cargo test".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["README.md".to_owned()],
            commit_oid: None,
        },
        2_000,
    );
    assert_eq!(outside.unwrap_err().code(), "PATH_OUTSIDE_CLAIM");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_overlapping_claims_have_one_winner() {
    let root = test_root("race");
    let barrier = Arc::new(Barrier::new(3));
    let handles = ["agent-a", "agent-b"].map(|agent| {
        let barrier = Arc::clone(&barrier);
        let root = root.clone();
        thread::spawn(move || {
            let store = CoordStore::new(root);
            barrier.wait();
            store.claim(&claim(agent, "src"), 1_000)
        })
    });
    barrier.wait();
    let results = handles.map(|handle| handle.join().unwrap());
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .filter(|error| error.code() == "CLAIM_OVERLAP")
            .count(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn expired_claim_no_longer_blocks_but_cannot_be_revived() {
    let root = test_root("expiry");
    let store = CoordStore::new(root.clone());
    let first = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    assert_eq!(
        store.status(601_000).unwrap().claims[0].state,
        ClaimState::Expired
    );
    assert!(store.claim(&claim("agent-b", "src"), 601_000).is_ok());
    let heartbeat = store.heartbeat(
        &HeartbeatInput {
            claim_id: first.claim_id,
            agent: "agent-a".to_owned(),
            ttl_seconds: 600,
            note: None,
        },
        601_001,
    );
    assert_eq!(heartbeat.unwrap_err().code(), "CLAIM_NOT_ACTIVE");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupt_log_blocks_mutation() {
    let root = test_root("corrupt");
    let log = root.join(".bullet-family/coord/events.jsonl");
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(&log, "not-json\n").unwrap();
    let error = CoordStore::new(root.clone())
        .claim(&claim("agent-a", "src"), 1_000)
        .unwrap_err();
    assert_eq!(error.code(), "CORRUPT_COORD_LOG");
    assert_eq!(fs::read_to_string(log).unwrap(), "not-json\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_log_fields_block_reads() {
    let root = test_root("unknown-log-field");
    let log = root.join(".bullet-family/coord/events.jsonl");
    fs::create_dir_all(log.parent().unwrap()).unwrap();
    fs::write(
        &log,
        concat!(
            r#"{"kind":"claim","schema_version":1,"at_unix_ms":1000,"claim_id":"clm_x","agent":"agent-a","lane":"lane-a","repo":"bullet-farm","paths":["src"],"expires_unix_ms":601000,"unexpected":true}"#,
            "\n"
        ),
    )
    .unwrap();
    let error = CoordStore::new(root.clone()).status(2_000).unwrap_err();
    assert_eq!(error.code(), "CORRUPT_COORD_LOG");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn handoff_cannot_attach_a_commit() {
    let root = test_root("handoff-commit");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let error = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: Some("a".repeat(40)),
            },
            2_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_REQUIRES_RECEIPT");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn receipt_rejects_paths_not_present_in_handoff() {
    let root = test_root("receipt-paths");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let handed_off = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let error = store
        .receipt(
            &CommitReceiptInput {
                claim_id: handed_off.claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid: "b".repeat(40),
                committed_paths: vec!["src/main.rs".to_owned()],
            },
            3_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMITTED_PATH_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn receipt_rejects_a_proper_subset_of_handoff_paths() {
    let root = test_root("receipt-subset");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let handed_off = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned(), "src/main.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let error = store
        .receipt(
            &CommitReceiptInput {
                claim_id: handed_off.claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid: "b".repeat(40),
                committed_paths: vec!["src/lib.rs".to_owned()],
            },
            3_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMITTED_PATH_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn receipt_rejects_a_commit_with_different_paths() {
    let root = test_root("receipt-git-paths");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let handed_off = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let commit_oid = commit(&root, "src/main.rs", "fn main() {}\n");
    let error = store
        .receipt(
            &CommitReceiptInput {
                claim_id: handed_off.claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid,
                committed_paths: vec!["src/lib.rs".to_owned()],
            },
            3_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn receipt_correction_is_append_only_and_binds_previous_oid() {
    let root = test_root("receipt-correction");
    let store = CoordStore::new(root.clone());
    let claimed = store.claim(&claim("agent-a", "src"), 1_000).unwrap();
    let handed_off = store
        .handoff(
            &HandoffInput {
                claim_id: claimed.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let first = commit(&root, "src/lib.rs", "pub const VALUE: u8 = 1;\n");
    store
        .receipt(
            &CommitReceiptInput {
                claim_id: handed_off.claim_id.clone(),
                orchestrator: "orchestrator".to_owned(),
                commit_oid: first.clone(),
                committed_paths: vec!["src/lib.rs".to_owned()],
            },
            3_000,
        )
        .unwrap();
    let second = commit(&root, "src/lib.rs", "pub const VALUE: u8 = 2;\n");
    let corrected = store
        .correct_receipt(
            &ReceiptCorrectionInput {
                claim_id: handed_off.claim_id.clone(),
                orchestrator: "orchestrator".to_owned(),
                previous_commit_oid: first,
                commit_oid: second.clone(),
                committed_paths: vec!["src/lib.rs".to_owned()],
                reason: "expanded OID was transcribed incorrectly".to_owned(),
            },
            4_000,
        )
        .unwrap();
    assert_eq!(corrected.commit_oid.as_deref(), Some(second.as_str()));
    let error = store
        .correct_receipt(
            &ReceiptCorrectionInput {
                claim_id: handed_off.claim_id,
                orchestrator: "orchestrator".to_owned(),
                previous_commit_oid: "a".repeat(40),
                commit_oid: second,
                committed_paths: vec!["src/lib.rs".to_owned()],
                reason: "does not bind current receipt".to_owned(),
            },
            5_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "RECEIPT_CORRECTION_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn grouped_receipt_requires_the_exact_union_of_handoffs() {
    let root = test_root("receipt-group");
    let store = CoordStore::new(root.clone());
    let first = store.claim(&claim("agent-a", "src/lib.rs"), 1_000).unwrap();
    let first = store
        .handoff(
            &HandoffInput {
                claim_id: first.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let second = store.claim(&claim("agent-b", "README.md"), 2_001).unwrap();
    let second = store
        .handoff(
            &HandoffInput {
                claim_id: second.claim_id,
                agent: "agent-b".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["README.md".to_owned()],
                commit_oid: None,
            },
            3_000,
        )
        .unwrap();
    let commit_oid = commit_many(
        &root,
        &[
            ("src/lib.rs", "pub const VALUE: u8 = 1;\n"),
            ("README.md", "proof\n"),
        ],
    );
    let receipted = store
        .receipt_group(
            &CommitReceiptGroupInput {
                claim_ids: vec![second.claim_id, first.claim_id],
                orchestrator: "orchestrator".to_owned(),
                commit_oid: commit_oid.clone(),
            },
            4_000,
        )
        .unwrap();
    assert_eq!(receipted.len(), 2);
    assert!(
        receipted
            .iter()
            .all(|claim| claim.commit_oid.as_deref() == Some(commit_oid.as_str()))
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn grouped_receipt_correction_is_exact_and_binds_every_previous_oid() {
    let root = test_root("receipt-group-correction");
    let store = CoordStore::new(root.clone());
    let first = store.claim(&claim("agent-a", "src/lib.rs"), 1_000).unwrap();
    let first = store
        .handoff(
            &HandoffInput {
                claim_id: first.claim_id,
                agent: "agent-a".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["src/lib.rs".to_owned()],
                commit_oid: None,
            },
            2_000,
        )
        .unwrap();
    let second = store.claim(&claim("agent-b", "README.md"), 2_001).unwrap();
    let second = store
        .handoff(
            &HandoffInput {
                claim_id: second.claim_id,
                agent: "agent-b".to_owned(),
                proof_command: "cargo test".to_owned(),
                proof_exit_code: 0,
                changed_paths: vec!["README.md".to_owned()],
                commit_oid: None,
            },
            3_000,
        )
        .unwrap();
    let claim_ids = vec![second.claim_id.clone(), first.claim_id.clone()];
    let original = commit_many(
        &root,
        &[
            ("src/lib.rs", "pub const VALUE: u8 = 1;\n"),
            ("README.md", "first\n"),
        ],
    );
    store
        .receipt_group(
            &CommitReceiptGroupInput {
                claim_ids: claim_ids.clone(),
                orchestrator: "orchestrator".to_owned(),
                commit_oid: original.clone(),
            },
            4_000,
        )
        .unwrap();
    let replacement = commit_many(
        &root,
        &[
            ("src/lib.rs", "pub const VALUE: u8 = 2;\n"),
            ("README.md", "second\n"),
        ],
    );
    let corrected = store
        .correct_receipt_group(
            &GroupReceiptCorrectionInput {
                claim_ids: claim_ids.clone(),
                orchestrator: "orchestrator".to_owned(),
                previous_commit_oid: original,
                commit_oid: replacement.clone(),
                reason: "split a contaminated shared-index commit".to_owned(),
            },
            5_000,
        )
        .unwrap();
    assert_eq!(corrected.len(), 2);
    assert!(corrected.iter().all(|claim| {
        claim.commit_oid.as_deref() == Some(replacement.as_str())
            && claim.commit_recorded_at_unix_ms == Some(5_000)
    }));

    let error = store
        .correct_receipt_group(
            &GroupReceiptCorrectionInput {
                claim_ids: claim_ids.clone(),
                orchestrator: "orchestrator".to_owned(),
                previous_commit_oid: "a".repeat(40),
                commit_oid: replacement.clone(),
                reason: "must bind all current receipts".to_owned(),
            },
            6_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "RECEIPT_CORRECTION_MISMATCH");

    let log = root.join(".bullet-family/coord/events.jsonl");
    let text = fs::read_to_string(&log).unwrap();
    let mut corrupt: serde_json::Value =
        serde_json::from_str(text.lines().last().unwrap()).unwrap();
    corrupt["at_unix_ms"] = serde_json::json!(7_000);
    corrupt["previous_commit_oid"] = serde_json::json!("a".repeat(40));
    let mut file = fs::OpenOptions::new().append(true).open(&log).unwrap();
    use std::io::Write as _;
    writeln!(file, "{}", serde_json::to_string(&corrupt).unwrap()).unwrap();
    let replay_error = store.status(8_000).unwrap_err();
    assert_eq!(replay_error.code(), "CORRUPT_COORD_LOG");
    fs::remove_dir_all(root).unwrap();
}
