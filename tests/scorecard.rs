//! The scorecard instrument awards typed deltas only after semantic admission.

use std::path::PathBuf;

use bullet_family::scorecard::{evaluate, render_markdown};
use serde_json::Value;

fn hub() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn add_unknown_field(value: &mut Value) {
    value["unexpected"] = Value::Bool(true);
}

fn duplicate_dimension(value: &mut Value) {
    value["dimensions"][1]["id"] = Value::from(1);
}

fn set_bad_weight(value: &mut Value) {
    value["dimensions"][0]["weight"] = Value::from(13);
}

fn redistribute_weights(value: &mut Value) {
    value["dimensions"][0]["weight"] = Value::from(13);
    value["dimensions"][1]["weight"] = Value::from(12);
}

fn change_blend(value: &mut Value) {
    value["blend"]["architecture"] = Value::from(0.3);
    value["blend"]["implemented"] = Value::from(0.5);
}

fn change_baseline(value: &mut Value) {
    value["architecture_implemented_floor"] = Value::from(100.0);
}

fn change_dimension_floor(value: &mut Value) {
    value["dimensions"][0]["implemented_floor"] = Value::from(65);
}

fn change_dimension_name(value: &mut Value) {
    value["dimensions"][0]["name"] = Value::from("Authority-ish");
}

fn change_rubric(value: &mut Value) {
    value["rubric"] = Value::from("d2-v2");
}

fn duplicate_row(value: &mut Value) {
    value["rows"][1]["id"] = value["rows"][0]["id"].clone();
}

fn set_unknown_kind(value: &mut Value) {
    value["rows"][0]["kind"] = Value::from("file");
}

fn change_row_claim(value: &mut Value) {
    value["rows"][0]["claim"] = Value::from("Everything is done");
}

fn change_row_mapping(value: &mut Value) {
    value["rows"][0]["dimension"] = Value::from(2);
}

fn set_path_shaped_evidence(value: &mut Value) {
    value["rows"][0]["evidence"] = serde_json::json!({
        "source": "ci-observation",
        "subject_id": "/etc/passwd"
    });
}

fn freeze_status(value: &mut Value) {
    value["status"] = Value::from("frozen-baseline");
}

fn incomplete_inventory(value: &mut Value) {
    value["criterion_inventory_complete"] = Value::Bool(false);
}

fn inflate_delta(value: &mut Value) {
    value["rows"][0]["implemented_delta"] = Value::from(32);
}

#[test]
fn rubric_admits_only_rederived_product_rows() {
    let report = evaluate(&hub()).expect("scorecard");
    assert_eq!(report.rubric, "d2-v1");
    assert_eq!(report.dimensions.len(), 12);
    assert!(
        (report.implemented - 40.0).abs() < 0.15,
        "implemented={}",
        report.implemented
    );
    assert!(
        (report.blended - 43.5).abs() < 0.15,
        "blended={}",
        report.blended
    );
    assert!(report.blended > 43.3);
    assert!(!report.authoritative);
    let admitted: Vec<&str> = report
        .rows
        .iter()
        .filter(|row| row.admitted)
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(admitted, ["d7.evolution-off"]);
    let nonce = report
        .rows
        .iter()
        .find(|row| row.id == "d1.nonce-ledger")
        .expect("nonce row");
    assert!(!nonce.admitted);
    assert_eq!(nonce.refusal_reason, "PINNED_FAMILY_SUBJECT_UNAVAILABLE");
    let txn = report
        .rows
        .iter()
        .find(|row| row.id == "g2.transaction-proof")
        .expect("g2");
    assert!(!txn.admitted);
    assert_eq!(txn.refusal_reason, "NO_EVIDENCE_REFERENCE");
    let evolution = report
        .rows
        .iter()
        .find(|row| row.id == "d7.evolution-off")
        .expect("evolution row");
    assert!(evolution.admitted);
    assert_eq!(evolution.refusal_reason, "-");

    let original: Value = bullet_wire::decode_unique_value(
        &std::fs::read(hub().join("policy/scorecard-v1.json")).unwrap(),
    )
    .unwrap();
    for (name, mutate) in [
        ("unknown field", add_unknown_field as fn(&mut Value)),
        ("duplicate dimension", duplicate_dimension),
        ("bad weight", set_bad_weight),
        ("redistributed weights", redistribute_weights),
        ("changed blend", change_blend),
        ("changed baseline", change_baseline),
        ("changed dimension floor", change_dimension_floor),
        ("changed dimension name", change_dimension_name),
        ("changed rubric", change_rubric),
        ("duplicate row", duplicate_row),
        ("unknown kind", set_unknown_kind),
        ("changed row claim", change_row_claim),
        ("changed row mapping", change_row_mapping),
        ("path-shaped evidence", set_path_shaped_evidence),
        ("frozen status", freeze_status),
        ("incomplete inventory", incomplete_inventory),
        ("inflated delta", inflate_delta),
    ] {
        let mut hostile = original.clone();
        mutate(&mut hostile);
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir(directory.path().join("policy")).unwrap();
        std::fs::write(
            directory.path().join("policy/scorecard-v1.json"),
            serde_json::to_vec(&hostile).unwrap(),
        )
        .unwrap();
        assert!(evaluate(directory.path()).is_err(), "{name} was accepted");
    }
}

#[test]
fn mutable_sibling_bytes_cannot_buy_points() {
    let directory = tempfile::tempdir().unwrap();
    let isolated = directory.path().join("bullet-farm");
    std::fs::create_dir_all(isolated.join("policy/v1alpha1")).unwrap();
    std::fs::copy(
        hub().join("policy/scorecard-v1.json"),
        isolated.join("policy/scorecard-v1.json"),
    )
    .unwrap();
    std::fs::copy(
        hub().join("policy/v1alpha1/policy.json"),
        isolated.join("policy/v1alpha1/policy.json"),
    )
    .unwrap();

    let kernel = directory.path().join("bullet-kernel");
    std::fs::create_dir_all(kernel.join("crates/application/src")).unwrap();
    std::fs::create_dir_all(kernel.join("crates/application/tests")).unwrap();
    std::fs::create_dir_all(kernel.join("crates/budgets/src")).unwrap();
    std::fs::write(
        kernel.join("crates/application/src/nonce_ledger.rs"),
        "fn issue() {} fn consume() {} enum State { Issued, Consumed }",
    )
    .unwrap();
    std::fs::write(
        kernel.join("crates/application/tests/nonce_ledger.rs"),
        "fn issue_does_not_consume() { ledger.issue(); ledger.consume(); } fn consume_replay_is_refused() {}",
    )
    .unwrap();
    std::fs::write(
        kernel.join("crates/budgets/src/lib.rs"),
        "fn reserve() {} fn settle() {} fn conserved() {} remaining: u64, reserved: u64, settled: u64, unknown_liability: u64,",
    )
    .unwrap();

    let git = directory.path().join("bullet-git");
    std::fs::create_dir_all(git.join("crates/bullet-git-types/src")).unwrap();
    std::fs::create_dir_all(git.join("crates/bullet-git-types/tests")).unwrap();
    std::fs::write(
        git.join("crates/bullet-git-types/src/change.rs"),
        "pub const fn named_leaves() -> [(&'static str, &[u8]); 8] { todo!() }",
    )
    .unwrap();
    std::fs::write(
        git.join("crates/bullet-git-types/tests/proof_root.rs"),
        "fn proof_root_each_of_the_eight_leaves_is_tamper_evident() {}",
    )
    .unwrap();

    let report = evaluate(&isolated).expect("portable scorecard");
    let admitted = report
        .rows
        .iter()
        .filter(|row| row.admitted)
        .map(|row| row.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(admitted, ["d7.evolution-off"]);
    assert!((report.implemented - 40.0).abs() < 0.15);
    assert!((report.blended - 43.5).abs() < 0.15);
}

#[test]
fn isolated_hub_and_self_authored_envelopes_cannot_buy_points() {
    let original = std::fs::read(hub().join("policy/scorecard-v1.json")).unwrap();
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("policy")).unwrap();
    std::fs::write(directory.path().join("policy/scorecard-v1.json"), original).unwrap();
    let subjects = directory.path().join("policy/scorecard/subjects");
    std::fs::create_dir_all(&subjects).unwrap();
    let excerpt = "fn issue(\nfn consume(\nfn state(\nNONCE_CONSUMED\n";
    let envelope = serde_json::json!({
        "schema_version": "scorecard-subject-v1",
        "subject_id": "scorecard.d1.nonce-ledger",
        "excerpt_blake3": blake3::hash(excerpt.as_bytes()).to_hex().to_string(),
        "required_tokens": ["fn issue(", "fn consume(", "fn state(", "NONCE_CONSUMED"],
        "excerpt": excerpt,
    });
    std::fs::write(
        subjects.join("scorecard.d1.nonce-ledger.json"),
        serde_json::to_vec(&envelope).unwrap(),
    )
    .unwrap();
    let report = evaluate(directory.path()).expect("isolated scorecard");
    assert!(
        (report.implemented - 39.7).abs() < 0.15,
        "implemented={}",
        report.implemented
    );
    assert!(
        (report.blended - 43.3).abs() < 0.15,
        "blended={}",
        report.blended
    );
    assert!(report.rows.iter().all(|row| !row.admitted));
}

#[test]
fn markdown_names_the_instrument() {
    let report = evaluate(&hub()).expect("scorecard");
    let page = render_markdown(&report);
    assert!(page.contains("not release authority"));
    assert!(page.contains("instrumented estimate"));
    assert!(page.contains("Frozen baseline was **43.3**"));
    assert!(page.contains("g2.transaction-proof"));
    assert!(page.contains("| yes |"));
    assert_eq!(
        page,
        include_str!("../docs/assurance/scorecard.generated.md"),
        "tracked generated scorecard must equal the renderer byte-for-byte"
    );
}
