use std::{
    ffi::OsString,
    fs,
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

use bullet_family::coord::{
    Applied, ClaimState, ClaimSummary, CommitReceiptGroupInput, CommitReceiptInput, CoordError,
    CoordStore, GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput, MutationEnvelope,
    ReceiptCorrectionInput, Status,
};

#[path = "support/coord_v2.rs"]
pub mod coord_v2;
use coord_v2::{Harness, claim_input, request_id, strict_json};

fn run_cli(root: &Path, args: &[String]) -> Result<String, CoordError> {
    let mut argv = vec![
        OsString::from("bullet-family"),
        OsString::from("--root"),
        root.as_os_str().to_os_string(),
        OsString::from("coord"),
    ];
    argv.extend(args.iter().map(|value| OsString::from(value.as_str())));
    bullet_family::cli::run(argv, Ok(root.to_path_buf()))
}

fn assert_repository_refused(harness: &Harness, repo: &str) {
    let before = harness.segment_len();
    let mut invented = claim_input("agent-invented", &["src"]);
    invented.repo = repo.to_owned();
    let error = harness
        .store()
        .claim(&harness.mutation(invented))
        .unwrap_err();
    assert_eq!(error.code(), "REPOSITORY_IDENTITY_MISMATCH");
    assert_eq!(harness.segment_len(), before);
    assert!(harness.store().status().unwrap().claims.is_empty());
}

#[test]
fn lifecycle_is_durable_and_handoff_binds_changed_paths() {
    let harness = Harness::new("lifecycle");
    let claimed = harness.claim("agent-a", &["src"]);
    assert_eq!(claimed.receipt.sequence, 2);

    let heartbeat = harness
        .store()
        .heartbeat(&harness.mutation(HeartbeatInput {
            claim_id: claimed.projection.claim_id.clone(),
            agent: "agent-a".to_owned(),
            ttl_seconds: 600,
            note: Some("proof started".to_owned()),
        }))
        .unwrap();
    assert!(heartbeat.projection.expires_unix_ms > claimed.projection.expires_unix_ms);

    let handed_off = harness.handoff(&claimed.projection.claim_id, "agent-a", &["src/main.rs"]);
    assert_eq!(handed_off.projection.state, ClaimState::HandedOff);
    let commit_oid = harness.commit("src/main.rs", "fn main() {}\n");
    let receipted = harness
        .store()
        .receipt(&harness.mutation(CommitReceiptInput {
            claim_id: handed_off.projection.claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid: commit_oid.clone(),
            committed_paths: vec!["src/main.rs".to_owned()],
        }))
        .unwrap();
    assert_eq!(
        receipted.projection.commit_oid.as_deref(),
        Some(commit_oid.as_str())
    );
    assert_eq!(receipted.watermark.last_sequence, 5);

    let reopened = harness.reopen().status().unwrap();
    assert_eq!(reopened.as_of_sequence, receipted.watermark.last_sequence);
    assert_eq!(reopened.claims, vec![receipted.projection]);
}

#[test]
fn overlap_and_out_of_claim_handoff_fail_without_append() {
    let harness = Harness::new("overlap");
    let claimed = harness.claim("agent-a", &["src"]);
    let length = harness.segment_len();

    let overlap = harness
        .store()
        .claim(&harness.mutation(claim_input("agent-b", &["src/main.rs"])))
        .unwrap_err();
    assert_eq!(overlap.code(), "CLAIM_OVERLAP");

    let outside = harness
        .store()
        .handoff(&harness.mutation(HandoffInput {
            claim_id: claimed.projection.claim_id,
            agent: "agent-a".to_owned(),
            proof_command: "cargo test".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["README.md".to_owned()],
            commit_oid: None,
        }))
        .unwrap_err();
    assert_eq!(outside.code(), "PATH_OUTSIDE_CLAIM");
    assert_eq!(harness.segment_len(), length);
}

#[test]
fn concurrent_overlapping_claims_have_one_serialized_winner() {
    let harness = Harness::new("race");
    let root = harness.root().to_path_buf();
    let generation = harness.generation();
    let barrier = Arc::new(Barrier::new(3));
    let handles = ["agent-a", "agent-b"]
        .into_iter()
        .enumerate()
        .map(|(index, agent)| {
            let root = root.clone();
            let generation = generation.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let store = CoordStore::new(root);
                let envelope = MutationEnvelope {
                    request_id: request_id(100 + index as u64),
                    expected_generation_id: generation,
                    command: claim_input(agent, &["src"]),
                };
                barrier.wait();
                store.claim(&envelope)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .filter(|error| error.code() == "CLAIM_OVERLAP")
            .count(),
        1
    );
}

#[test]
fn exact_request_replays_across_restart_and_changed_operation_conflicts() {
    let harness = Harness::new("replay");
    let envelope = harness.mutation(claim_input("agent-a", &["src"]));
    let applied = harness.store().claim(&envelope).unwrap();
    let length = harness.segment_len();

    let replayed = harness.reopen().claim(&envelope).unwrap();
    assert!(replayed.replayed);
    assert_eq!(replayed.receipt, applied.receipt);
    assert_eq!(replayed.watermark, applied.watermark);
    assert_eq!(replayed.projection, applied.projection);
    assert_eq!(harness.segment_len(), length);

    let conflict = MutationEnvelope {
        request_id: envelope.request_id,
        expected_generation_id: envelope.expected_generation_id,
        command: HeartbeatInput {
            claim_id: applied.projection.claim_id,
            agent: "agent-a".to_owned(),
            ttl_seconds: 600,
            note: None,
        },
    };
    assert_eq!(
        harness.reopen().heartbeat(&conflict).unwrap_err().code(),
        "COORD_REQUEST_CONFLICT"
    );
    assert_eq!(harness.segment_len(), length);
}

#[test]
fn absent_public_store_is_creation_free() {
    let root = tempfile::tempdir().unwrap();
    let store = CoordStore::new(root.path().to_path_buf());
    assert_eq!(store.status().unwrap_err().code(), "COORD_NOT_INITIALIZED");
    let mutation = MutationEnvelope {
        request_id: request_id(1),
        expected_generation_id: bullet_family::coord::GenerationId::parse(format!(
            "gen_{}",
            "a".repeat(64)
        ))
        .unwrap(),
        command: claim_input("agent-a", &["src"]),
    };
    assert_eq!(
        store.claim(&mutation).unwrap_err().code(),
        "COORD_NOT_INITIALIZED"
    );
    assert!(!root.path().join(".bullet-family").exists());
}

#[test]
fn cli_init_claim_reconcile_and_status_use_the_closed_v2_contract() {
    let harness = Harness::new("cli");
    let generation = harness.generation().as_str().to_owned();
    let initialized: Status = strict_json(
        &run_cli(
            harness.root(),
            &[
                "init".to_owned(),
                "--operator".to_owned(),
                "coord-test-operator".to_owned(),
                "--policy-sha256".to_owned(),
                format!("sha256:{}", "a".repeat(64)),
                "--replay-contract-sha256".to_owned(),
                format!("sha256:{}", "b".repeat(64)),
                "--bootstrap-commit".to_owned(),
                "c".repeat(40),
                "--bootstrap-path".to_owned(),
                "Cargo.toml".to_owned(),
                "--bootstrap-path".to_owned(),
                "src".to_owned(),
            ],
        )
        .unwrap(),
    );
    assert_eq!(initialized.generation_id, generation);
    assert_eq!(initialized.as_of_sequence, 1);

    let request = format!("req_{:064x}", 900_u64);
    let claim_args = vec![
        "claim".to_owned(),
        "--request-id".to_owned(),
        request,
        "--expected-generation".to_owned(),
        generation.clone(),
        "--agent".to_owned(),
        "agent-cli".to_owned(),
        "--lane".to_owned(),
        "lane-cli".to_owned(),
        "--repo".to_owned(),
        "bullet-farm".to_owned(),
        "--path".to_owned(),
        "src/cli.rs".to_owned(),
    ];
    let applied: Applied<ClaimSummary> =
        strict_json(&run_cli(harness.root(), &claim_args).unwrap());
    assert!(!applied.replayed);
    let replayed: Applied<ClaimSummary> =
        strict_json(&run_cli(harness.root(), &claim_args).unwrap());
    assert!(replayed.replayed);
    assert_eq!(replayed.receipt, applied.receipt);
    assert_eq!(replayed.projection, applied.projection);

    let before = harness.segment_len();
    let missing_request = vec![
        "claim".to_owned(),
        "--expected-generation".to_owned(),
        generation.clone(),
        "--agent".to_owned(),
        "agent-missing".to_owned(),
        "--lane".to_owned(),
        "lane-missing".to_owned(),
        "--repo".to_owned(),
        "bullet-farm".to_owned(),
        "--path".to_owned(),
        "README.md".to_owned(),
    ];
    assert_eq!(
        run_cli(harness.root(), &missing_request)
            .unwrap_err()
            .code(),
        "MISSING_OPTION"
    );
    let mut obsolete_time = claim_args.clone();
    obsolete_time[2] = format!("req_{:064x}", 901_u64);
    obsolete_time.extend(["--now".to_owned(), "1000".to_owned()]);
    assert_eq!(
        run_cli(harness.root(), &obsolete_time).unwrap_err().code(),
        "UNKNOWN_OPTION"
    );
    assert_eq!(harness.segment_len(), before);

    let status: Status = strict_json(
        &run_cli(
            harness.root(),
            &["status".to_owned(), "--json".to_owned(), "--all".to_owned()],
        )
        .unwrap(),
    );
    assert_eq!(status.generation_id, generation);
    assert_eq!(status.claims, vec![applied.projection]);
}

#[test]
fn handoff_cannot_attach_a_commit() {
    let harness = Harness::new("handoff-commit");
    let claimed = harness.claim("agent-a", &["src"]);
    let error = harness
        .store()
        .handoff(&harness.mutation(HandoffInput {
            claim_id: claimed.projection.claim_id,
            agent: "agent-a".to_owned(),
            proof_command: "cargo test".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["src/lib.rs".to_owned()],
            commit_oid: Some("a".repeat(40)),
        }))
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_REQUIRES_RECEIPT");
}

#[test]
fn receipts_refuse_incomplete_scope_and_wrong_git_readback() {
    let subset = Harness::new("receipt-subset");
    let claim_id = subset.claim("agent-a", &["src"]).projection.claim_id;
    subset.handoff(&claim_id, "agent-a", &["src/lib.rs", "src/main.rs"]);
    let error = subset
        .store()
        .receipt(&subset.mutation(CommitReceiptInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid: "b".repeat(40),
            committed_paths: vec!["src/lib.rs".to_owned()],
        }))
        .unwrap_err();
    assert_eq!(error.code(), "COMMITTED_PATH_MISMATCH");

    let readback = Harness::new("receipt-readback");
    let claim_id = readback.claim_and_handoff("agent-a", &["src/lib.rs"]);
    let commit_oid = readback.commit("src/main.rs", "fn main() {}\n");
    let error = readback
        .store()
        .receipt(&readback.mutation(CommitReceiptInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
            committed_paths: vec!["src/lib.rs".to_owned()],
        }))
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");

    for (name, manifest) in [("empty-manifest", ""), ("invalid-manifest", "[[repo]")] {
        let custody = Harness::new(name);
        let outside = Harness::new(&format!("{name}-outside"));
        fs::write(custody.root().join("repos.manifest.toml"), manifest).unwrap();
        std::os::unix::fs::symlink(
            outside.root().join("bullet-farm"),
            custody.root().join("invented-repo"),
        )
        .unwrap();
        assert_repository_refused(&custody, "invented-repo");
    }

    let linked = Harness::new("linked-member");
    let outside = Harness::new("linked-member-outside");
    fs::rename(
        linked.root().join("bullet-farm"),
        linked.root().join("held-member"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        outside.root().join("bullet-farm"),
        linked.root().join("bullet-farm"),
    )
    .unwrap();
    assert_repository_refused(&linked, "bullet-farm");

    let git_link = Harness::new("linked-git-dir");
    let git = git_link.root().join("bullet-farm/.git");
    let held = git_link.root().join("held-git-dir");
    fs::rename(&git, &held).unwrap();
    std::os::unix::fs::symlink(&held, &git).unwrap();
    assert_repository_refused(&git_link, "bullet-farm");

    let alternate = Harness::new("alternate-object-store");
    let info = alternate.root().join("bullet-farm/.git/objects/info");
    fs::create_dir_all(&info).unwrap();
    fs::write(info.join("alternates"), b"/outside/objects\n").unwrap();
    assert_repository_refused(&alternate, "bullet-farm");
}

#[test]
fn receipt_correction_is_append_only_and_binds_previous_oid() {
    let harness = Harness::new("receipt-correction");
    let claim_id = harness.claim_and_handoff("agent-a", &["src/lib.rs"]);
    let first = harness.commit("src/lib.rs", "pub const VALUE: u8 = 1;\n");
    harness
        .store()
        .receipt(&harness.mutation(CommitReceiptInput {
            claim_id: claim_id.clone(),
            orchestrator: "orchestrator".to_owned(),
            commit_oid: first.clone(),
            committed_paths: vec!["src/lib.rs".to_owned()],
        }))
        .unwrap();
    let second = harness.commit("src/lib.rs", "pub const VALUE: u8 = 2;\n");
    let corrected = harness
        .store()
        .correct_receipt(&harness.mutation(ReceiptCorrectionInput {
            claim_id: claim_id.clone(),
            orchestrator: "orchestrator".to_owned(),
            previous_commit_oid: first,
            commit_oid: second.clone(),
            committed_paths: vec!["src/lib.rs".to_owned()],
            reason: "expanded OID was transcribed incorrectly".to_owned(),
        }))
        .unwrap();
    assert_eq!(
        corrected.projection.commit_oid.as_deref(),
        Some(second.as_str())
    );

    let error = harness
        .store()
        .correct_receipt(&harness.mutation(ReceiptCorrectionInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            previous_commit_oid: "a".repeat(40),
            commit_oid: second,
            committed_paths: vec!["src/lib.rs".to_owned()],
            reason: "does not bind current receipt".to_owned(),
        }))
        .unwrap_err();
    assert_eq!(error.code(), "RECEIPT_CORRECTION_MISMATCH");
}

#[test]
fn grouped_receipt_and_correction_bind_every_claim() {
    let harness = Harness::new("group-correction");
    let first = harness.claim_and_handoff("agent-a", &["src/lib.rs"]);
    let second = harness.claim_and_handoff("agent-b", &["README.md"]);
    let claim_ids = vec![second, first];
    let original = harness.commit_many(&[
        ("src/lib.rs", "pub const VALUE: u8 = 1;\n"),
        ("README.md", "first\n"),
    ]);
    let receipted = harness
        .store()
        .receipt_group(&harness.mutation(CommitReceiptGroupInput {
            claim_ids: claim_ids.clone(),
            orchestrator: "orchestrator".to_owned(),
            commit_oid: original.clone(),
        }))
        .unwrap();
    assert_eq!(receipted.projection.len(), 2);

    let replacement = harness.commit_many(&[
        ("src/lib.rs", "pub const VALUE: u8 = 2;\n"),
        ("README.md", "second\n"),
    ]);
    let corrected = harness
        .store()
        .correct_receipt_group(&harness.mutation(GroupReceiptCorrectionInput {
            claim_ids: claim_ids.clone(),
            orchestrator: "orchestrator".to_owned(),
            previous_commit_oid: original,
            commit_oid: replacement.clone(),
            reason: "split a contaminated shared-index commit".to_owned(),
        }))
        .unwrap();
    assert!(
        corrected
            .projection
            .iter()
            .all(|claim| claim.commit_oid.as_deref() == Some(replacement.as_str()))
    );

    let error = harness
        .store()
        .correct_receipt_group(&harness.mutation(GroupReceiptCorrectionInput {
            claim_ids,
            orchestrator: "orchestrator".to_owned(),
            previous_commit_oid: "a".repeat(40),
            commit_oid: replacement,
            reason: "must bind every current receipt".to_owned(),
        }))
        .unwrap_err();
    assert_eq!(error.code(), "RECEIPT_CORRECTION_MISMATCH");
}

#[test]
fn framed_segment_tampering_blocks_public_replay() {
    let harness = Harness::new("tamper");
    harness.claim("agent-a", &["src"]);
    let source = harness.store().status().unwrap().source;
    let mut bytes = fs::read(&source).unwrap();
    let index = bytes.iter().position(|byte| *byte == b'a').unwrap();
    bytes[index] = b'b';
    fs::write(&source, bytes).unwrap();
    assert_eq!(
        harness.store().status().unwrap_err().code(),
        "CORRUPT_COORD_SEGMENT"
    );
}
