use std::fs;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::claim_high_water;

fn claim(id: &str) -> Value {
    json!({
        "kind": "claim", "schema_version": 1, "at_unix_ms": 1_000,
        "claim_id": id, "agent": "worker", "lane": "docs", "repo": "bullet-farm",
        "paths": ["README.md"], "expires_unix_ms": 31_000
    })
}

fn heartbeat(id: &str) -> Value {
    json!({
        "kind": "heartbeat", "schema_version": 1, "at_unix_ms": 20_000,
        "claim_id": id, "agent": "worker", "expires_unix_ms": 60_000, "note": null
    })
}

fn handoff(id: &str) -> Value {
    json!({
        "kind": "handoff", "schema_version": 1, "at_unix_ms": 21_000,
        "claim_id": id, "agent": "worker", "proof_command": "just docs",
        "proof_exit_code": 0, "changed_paths": ["README.md"], "commit_oid": null
    })
}

fn receipt(id: &str) -> Value {
    json!({
        "kind": "commit_receipt", "schema_version": 1, "at_unix_ms": 22_000,
        "claim_id": id, "orchestrator": "integration-owner",
        "commit_oid": "a".repeat(40), "committed_paths": ["README.md"]
    })
}

fn framed(records: &[Value]) -> Vec<u8> {
    records
        .iter()
        .flat_map(|record| {
            let mut bytes = serde_json::to_vec(record).unwrap();
            bytes.push(b'\n');
            bytes
        })
        .collect()
}

fn observe(
    bytes: &[u8],
    now: u64,
) -> Result<super::Wave0ClaimHighWaterV1, crate::coord::CoordError> {
    let root = tempfile::tempdir().unwrap();
    let ledger = root.path().join("events.jsonl");
    fs::write(&ledger, bytes).unwrap();
    let result = claim_high_water(&ledger, now);
    assert_eq!(
        fs::read(&ledger).unwrap(),
        bytes,
        "observation mutated its source"
    );
    result
}

#[test]
fn strict_framing_and_schema_refuse_before_high_water() {
    let valid = framed(&[claim("claim-a")]);
    let mut crlf = valid[..valid.len() - 1].to_vec();
    crlf.extend_from_slice(b"\r\n");
    let mut unknown_field = claim("claim-a");
    unknown_field["ignored_before"] = json!(true);
    for bytes in [
        valid[..valid.len() - 1].to_vec(),
        crlf,
        b"\n".to_vec(),
        b"{\"kind\":\"unknown\"}\n".to_vec(),
        b"{\"kind\":\"claim\",\"kind\":\"handoff\"}\n".to_vec(),
        framed(&[unknown_field]),
    ] {
        assert_eq!(
            observe(&bytes, 40_000).unwrap_err().code(),
            "CORRUPT_COORD_LOG"
        );
    }
    let mut future_schema = claim("claim-a");
    future_schema["schema_version"] = json!(2);
    assert_eq!(
        observe(&framed(&[future_schema]), 40_000)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_SCHEMA"
    );
}

#[test]
fn heartbeat_extension_remains_active_after_original_expiry() {
    let bytes = framed(&[claim("claim-a"), heartbeat("claim-a")]);
    let error = observe(&bytes, 40_000).unwrap_err();
    assert_eq!(error.code(), "WAVE0_PRODUCER_INVALID");
    assert!(error.to_string().contains("claim-a remains Active"));
}

#[test]
fn expired_and_handed_off_work_require_recorded_dispositions() {
    for (records, state) in [
        (vec![claim("claim-a")], "Expired"),
        (vec![claim("claim-a"), handoff("claim-a")], "HandedOff"),
    ] {
        let error = observe(&framed(&records), 40_000).unwrap_err();
        assert_eq!(error.code(), "WAVE0_PRODUCER_INVALID");
        assert!(
            error
                .to_string()
                .contains(&format!("claim-a remains {state}"))
        );
    }
}

#[test]
fn semantic_replay_refuses_duplicate_claim_wrong_owner_and_unbound_receipts() {
    let mut wrong_owner = heartbeat("claim-a");
    wrong_owner["agent"] = json!("other-worker");
    let mut wrong_scope = receipt("claim-a");
    wrong_scope["committed_paths"] = json!(["Cargo.toml"]);
    for records in [
        vec![claim("claim-a"), claim("claim-a")],
        vec![claim("claim-a"), wrong_owner],
        vec![claim("claim-a"), receipt("claim-a")],
        vec![
            claim("claim-a"),
            handoff("claim-a"),
            receipt("missing-claim"),
        ],
        vec![claim("claim-a"), handoff("claim-a"), wrong_scope],
    ] {
        assert_eq!(
            observe(&framed(&records), 40_000).unwrap_err().code(),
            "CORRUPT_COORD_LOG"
        );
    }
}

#[test]
fn exact_receipt_disposition_binds_source_bytes_and_retries_identically() {
    let bytes = framed(&[
        claim("claim-a"),
        heartbeat("claim-a"),
        handoff("claim-a"),
        receipt("claim-a"),
    ]);
    let root = tempfile::tempdir().unwrap();
    let ledger = root.path().join("events.jsonl");
    fs::write(&ledger, &bytes).unwrap();
    let first = claim_high_water(&ledger, 40_000).unwrap();
    let retried = claim_high_water(&ledger, 80_000).unwrap();
    assert_eq!(first, retried);
    assert_eq!(first.entry_count, 4);
    assert_eq!(first.byte_length, bytes.len() as u64);
    assert_eq!(first.active_claim_count, 0);
    assert_eq!(
        first.claim_ledger_sha256,
        format!("sha256:{:x}", Sha256::digest(&bytes))
    );
    assert_eq!(fs::read(&ledger).unwrap(), bytes);

    let mut changed_receipt = receipt("claim-a");
    changed_receipt["commit_oid"] = json!("b".repeat(40));
    let changed = observe(
        &framed(&[
            claim("claim-a"),
            heartbeat("claim-a"),
            handoff("claim-a"),
            changed_receipt,
        ]),
        40_000,
    )
    .unwrap();
    assert_ne!(first.claim_ledger_sha256, changed.claim_ledger_sha256);
}

#[test]
fn grouped_receipts_resolve_each_exact_claim_and_missing_member_refuses() {
    let group = json!({
        "kind": "commit_receipt_group", "schema_version": 1, "at_unix_ms": 22_000,
        "orchestrator": "integration-owner", "commit_oid": "a".repeat(40),
        "receipts": [
            {"claim_id": "claim-a", "committed_paths": ["README.md"]},
            {"claim_id": "claim-b", "committed_paths": ["README.md"]}
        ]
    });
    let records = vec![
        claim("claim-a"),
        claim("claim-b"),
        handoff("claim-a"),
        handoff("claim-b"),
        group,
    ];
    assert_eq!(observe(&framed(&records), 40_000).unwrap().entry_count, 5);
    let mut missing_member = records;
    missing_member[4]["receipts"][1]["claim_id"] = json!("claim-c");
    assert_eq!(
        observe(&framed(&missing_member), 40_000)
            .unwrap_err()
            .code(),
        "CORRUPT_COORD_LOG"
    );
}

#[test]
fn empty_legacy_ledger_has_zero_high_water() {
    let observation = observe(b"", 40_000).unwrap();
    assert_eq!(observation.entry_count, 0);
    assert_eq!(observation.byte_length, 0);
    assert_eq!(observation.active_claim_count, 0);
}
