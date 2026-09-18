use std::{collections::BTreeMap, fs};

use bullet_family::coord::{CommitReceiptGroupInput, CommitReceiptInput, HandoffInput};

#[path = "support/coord_v2.rs"]
pub mod coord_v2;
use coord_v2::Harness;

#[test]
fn directory_handoff_records_only_exact_commit_leaves() {
    let harness = Harness::new("directory");
    let claim_id = harness.claim_and_handoff("paper", &["docs/paper"]);
    let leaves = ["docs/paper/README.md", "docs/paper/paper.tex"];
    let commit_oid = harness.commit_many(&[(leaves[0], "readme\n"), (leaves[1], "paper\n")]);
    let applied = harness
        .store()
        .receipt(&harness.mutation(CommitReceiptInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
            committed_paths: leaves.iter().map(|path| (*path).to_owned()).collect(),
        }))
        .unwrap();
    assert_eq!(applied.projection.changed_paths, vec!["docs/paper"]);
    assert_eq!(
        harness.last_record()["committed_paths"],
        serde_json::json!(leaves)
    );
}

#[test]
fn grouped_directory_handoffs_duplicate_shared_leaves_deterministically() {
    let harness = Harness::new("paper-brand");
    let paper = harness.claim_and_handoff(
        "paper",
        &["docs/README.md", "docs/paper", "docs/spec/paper.md"],
    );
    let competitor =
        harness.claim_and_handoff("competitor", &["docs/assurance/competitor-snapshot.md"]);
    let brand = harness.claim_and_handoff("brand", &["docs/README.md", "docs/brand"]);
    let commit_oid = harness.commit_many(&[
        ("docs/README.md", "index\n"),
        ("docs/paper/README.md", "paper\n"),
        ("docs/paper/paper.tex", "tex\n"),
        ("docs/spec/paper.md", "pointer\n"),
        ("docs/assurance/competitor-snapshot.md", "snapshot\n"),
        ("docs/brand/mascots/README.md", "brand\n"),
    ]);
    harness
        .store()
        .receipt_group(&harness.mutation(CommitReceiptGroupInput {
            claim_ids: vec![paper.clone(), competitor.clone(), brand.clone()],
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
        }))
        .unwrap();

    let receipts = harness.last_record()["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| {
            (
                receipt["claim_id"].as_str().unwrap().to_owned(),
                receipt["committed_paths"].clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        receipts[&competitor],
        serde_json::json!(["docs/assurance/competitor-snapshot.md"])
    );
    assert_eq!(
        receipts[&paper],
        serde_json::json!([
            "docs/README.md",
            "docs/paper/README.md",
            "docs/paper/paper.tex",
            "docs/spec/paper.md"
        ])
    );
    assert_eq!(
        receipts[&brand],
        serde_json::json!(["docs/README.md", "docs/brand/mascots/README.md"])
    );
}

#[test]
fn group_rejects_uncovered_near_prefix_scope_widening_and_empty_commit() {
    for (name, files) in [
        (
            "near-prefix",
            vec![
                ("docs/paper-old/paper.tex", "escape\n"),
                ("README.md", "ok\n"),
            ],
        ),
        (
            "scope-widening",
            vec![("docs/paper/paper.tex", "ok\n"), ("secret.txt", "no\n")],
        ),
    ] {
        let harness = Harness::new(name);
        let paper = harness.claim_and_handoff("paper", &["docs/paper"]);
        let readme = harness.claim_and_handoff("readme", &["README.md"]);
        let commit_oid = harness.commit_many(&files);
        let before = harness.segment_len();
        let error = harness
            .store()
            .receipt_group(&harness.mutation(CommitReceiptGroupInput {
                claim_ids: vec![paper, readme],
                orchestrator: "orchestrator".to_owned(),
                commit_oid,
            }))
            .unwrap_err();
        assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");
        assert_eq!(harness.segment_len(), before);
    }

    let harness = Harness::new("empty");
    let first = harness.claim_and_handoff("first", &["docs/paper"]);
    let second = harness.claim_and_handoff("second", &["README.md"]);
    let commit_oid = harness.empty_commit();
    let error = harness
        .store()
        .receipt_group(&harness.mutation(CommitReceiptGroupInput {
            claim_ids: vec![first, second],
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
        }))
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");
}

#[test]
fn receipt_rejects_a_handoff_scope_covering_no_leaf() {
    let harness = Harness::new("unused-scope");
    let claim = harness.claim("paper", &["docs/paper", "docs/spec/paper.md"]);
    let claim_id = claim.projection.claim_id;
    harness.handoff(&claim_id, "paper", &["docs/paper", "docs/spec/paper.md"]);
    let commit_oid = harness.commit("docs/paper/paper.tex", "paper\n");
    let error = harness
        .store()
        .receipt(&harness.mutation(CommitReceiptInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
            committed_paths: vec!["docs/paper/paper.tex".to_owned()],
        }))
        .unwrap_err();
    assert_eq!(error.code(), "COMMITTED_PATH_MISMATCH");

    let custody = Harness::new("manifest-receipt");
    let claim_id = custody.claim_and_handoff("paper", &["docs/paper.md"]);
    let commit_oid = custody.commit("docs/paper.md", "paper\n");
    fs::write(custody.root().join("repos.manifest.toml"), "").unwrap();
    let before = custody.segment_len();
    let error = custody
        .store()
        .receipt(&custody.mutation(CommitReceiptInput {
            claim_id,
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
            committed_paths: vec!["docs/paper.md".to_owned()],
        }))
        .unwrap_err();
    assert_eq!(error.code(), "REPOSITORY_IDENTITY_MISMATCH");
    assert_eq!(custody.segment_len(), before);
}

#[test]
fn normalized_subject_order_replays_but_duplicate_group_refuses() {
    let harness = Harness::new("normalized-order");
    let first = harness.mutation(bullet_family::coord::ClaimInput {
        agent: "agent-a".to_owned(),
        lane: "lane-agent-a".to_owned(),
        repo: "bullet-farm".to_owned(),
        paths: vec!["src/z.rs".to_owned(), "src/a.rs".to_owned()],
        ttl_seconds: 600,
    });
    let applied = harness.store().claim(&first).unwrap();
    let reordered = bullet_family::coord::MutationEnvelope {
        request_id: first.request_id,
        expected_generation_id: first.expected_generation_id,
        command: bullet_family::coord::ClaimInput {
            paths: vec!["src/a.rs".to_owned(), "src/z.rs".to_owned()],
            ..first.command
        },
    };
    let replayed = harness.store().claim(&reordered).unwrap();
    assert!(replayed.replayed);
    assert_eq!(replayed.receipt, applied.receipt);

    let claim_id = applied.projection.claim_id;
    let manifest = harness.root().join("repos.manifest.toml");
    let admitted_manifest = fs::read(&manifest).unwrap();
    fs::write(&manifest, "[[repo]").unwrap();
    let before = harness.segment_len();
    let error = harness
        .store()
        .handoff(&harness.mutation(HandoffInput {
            claim_id: claim_id.clone(),
            agent: "agent-a".to_owned(),
            proof_command: "cargo test --locked".to_owned(),
            proof_exit_code: 0,
            changed_paths: vec!["src/a.rs".to_owned(), "src/z.rs".to_owned()],
            commit_oid: None,
        }))
        .unwrap_err();
    assert_eq!(error.code(), "REPOSITORY_IDENTITY_MISMATCH");
    assert_eq!(harness.segment_len(), before);
    fs::write(manifest, admitted_manifest).unwrap();
    harness.handoff(&claim_id, "agent-a", &["src/a.rs", "src/z.rs"]);
    let commit_oid = harness.commit_many(&[("src/a.rs", "a\n"), ("src/z.rs", "z\n")]);
    let before = harness.segment_len();
    let error = harness
        .store()
        .receipt_group(&harness.mutation(CommitReceiptGroupInput {
            claim_ids: vec![claim_id.clone(), claim_id],
            orchestrator: "orchestrator".to_owned(),
            commit_oid,
        }))
        .unwrap_err();
    assert_eq!(error.code(), "DUPLICATE_CLAIM_ID");
    assert_eq!(harness.segment_len(), before);
}
