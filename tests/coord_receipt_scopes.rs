use std::{collections::BTreeMap, fs, path::Path, path::PathBuf, process::Command, thread};

use bullet_family::coord::{
    ClaimInput, CommitReceiptGroupInput, CommitReceiptInput, CoordStore, HandoffInput,
};

fn test_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-family-receipt-{name}-{}-{:?}",
        std::process::id(),
        thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    let repo = root.join("bullet-farm");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["config", "user.name", "Coord Test"]);
    git(&repo, &["config", "user.email", "coord@example.invalid"]);
    root
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

fn commit_many(root: &Path, files: &[(&str, &str)]) -> String {
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

fn handoff(store: &CoordStore, agent: &str, paths: &[&str], now: u64) -> String {
    let claim = store
        .claim(
            &ClaimInput {
                agent: agent.to_owned(),
                lane: format!("lane-{agent}"),
                repo: "bullet-farm".to_owned(),
                paths: paths.iter().map(|path| (*path).to_owned()).collect(),
                ttl_seconds: 600,
            },
            now,
        )
        .unwrap();
    store
        .handoff(
            &HandoffInput {
                claim_id: claim.claim_id.clone(),
                agent: agent.to_owned(),
                proof_command: "cargo test --locked".to_owned(),
                proof_exit_code: 0,
                changed_paths: paths.iter().map(|path| (*path).to_owned()).collect(),
                commit_oid: None,
            },
            now + 1,
        )
        .unwrap();
    claim.claim_id
}

fn last_record(root: &Path) -> serde_json::Value {
    let log = fs::read_to_string(root.join(".bullet-family/coord/events.jsonl")).unwrap();
    serde_json::from_str(log.lines().last().unwrap()).unwrap()
}

#[test]
fn directory_handoff_records_only_exact_commit_leaves() {
    let root = test_root("directory");
    let store = CoordStore::new(root.clone());
    let claim_id = handoff(&store, "paper", &["docs/paper"], 1_000);
    let leaves = vec!["docs/paper/README.md", "docs/paper/paper.tex"];
    let commit_oid = commit_many(&root, &[(leaves[0], "readme\n"), (leaves[1], "paper\n")]);
    store
        .receipt(
            &CommitReceiptInput {
                claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid,
                committed_paths: leaves.iter().map(|path| (*path).to_owned()).collect(),
            },
            2_000,
        )
        .unwrap();
    assert_eq!(
        last_record(&root)["committed_paths"],
        serde_json::json!(leaves)
    );
    store.status(3_000).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn grouped_directory_handoffs_duplicate_shared_leaves_deterministically() {
    let root = test_root("paper-brand");
    let store = CoordStore::new(root.clone());
    let paper = handoff(
        &store,
        "paper",
        &["docs/README.md", "docs/paper", "docs/spec/paper.md"],
        1_000,
    );
    let competitor = handoff(
        &store,
        "competitor",
        &["docs/assurance/competitor-snapshot.md"],
        2_000,
    );
    let brand = handoff(&store, "brand", &["docs/README.md", "docs/brand"], 3_000);
    let commit_oid = commit_many(
        &root,
        &[
            ("docs/README.md", "index\n"),
            ("docs/paper/README.md", "paper\n"),
            ("docs/paper/paper.tex", "tex\n"),
            ("docs/spec/paper.md", "pointer\n"),
            ("docs/assurance/competitor-snapshot.md", "snapshot\n"),
            ("docs/brand/mascots/README.md", "brand\n"),
        ],
    );
    store
        .receipt_group(
            &CommitReceiptGroupInput {
                claim_ids: vec![paper.clone(), competitor.clone(), brand.clone()],
                orchestrator: "orchestrator".to_owned(),
                commit_oid,
            },
            4_000,
        )
        .unwrap();

    let receipts = last_record(&root)["receipts"]
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
    store.status(5_000).unwrap();

    let log_path = root.join(".bullet-family/coord/events.jsonl");
    let text = fs::read_to_string(&log_path).unwrap();
    let mut lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut record: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
    let brand_receipt = record["receipts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|receipt| receipt["claim_id"] == brand)
        .unwrap();
    brand_receipt["committed_paths"] = serde_json::json!(["docs/brand/mascots/README.md"]);
    *lines.last_mut().unwrap() = serde_json::to_string(&record).unwrap();
    fs::write(&log_path, format!("{}\n", lines.join("\n"))).unwrap();
    let error = store.status(6_000).unwrap_err();
    assert_eq!(error.code(), "CORRUPT_COORD_LOG");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn group_rejects_uncovered_near_prefix_and_empty_commit() {
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
        let root = test_root(name);
        let store = CoordStore::new(root.clone());
        let paper = handoff(&store, "paper", &["docs/paper"], 1_000);
        let readme = handoff(&store, "readme", &["README.md"], 2_000);
        let oid = commit_many(&root, &files);
        let error = store
            .receipt_group(
                &CommitReceiptGroupInput {
                    claim_ids: vec![paper, readme],
                    orchestrator: "orchestrator".to_owned(),
                    commit_oid: oid,
                },
                3_000,
            )
            .unwrap_err();
        assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");
        assert_ne!(last_record(&root)["kind"], "commit_receipt_group");
        fs::remove_dir_all(root).unwrap();
    }

    let root = test_root("empty");
    let store = CoordStore::new(root.clone());
    let first = handoff(&store, "first", &["docs/paper"], 1_000);
    let second = handoff(&store, "second", &["README.md"], 2_000);
    let repo = root.join("bullet-farm");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "-m", "empty"],
    );
    let oid = git(&repo, &["rev-parse", "HEAD"]);
    let error = store
        .receipt_group(
            &CommitReceiptGroupInput {
                claim_ids: vec![first, second],
                orchestrator: "orchestrator".to_owned(),
                commit_oid: oid,
            },
            3_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMIT_PATH_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn receipt_rejects_a_handoff_scope_covering_no_leaf() {
    let root = test_root("unused-scope");
    let store = CoordStore::new(root.clone());
    let claim_id = handoff(
        &store,
        "paper",
        &["docs/paper", "docs/spec/paper.md"],
        1_000,
    );
    let oid = commit_many(&root, &[("docs/paper/paper.tex", "paper\n")]);
    let error = store
        .receipt(
            &CommitReceiptInput {
                claim_id,
                orchestrator: "orchestrator".to_owned(),
                commit_oid: oid,
                committed_paths: vec!["docs/paper/paper.tex".to_owned()],
            },
            2_000,
        )
        .unwrap_err();
    assert_eq!(error.code(), "COMMITTED_PATH_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}
