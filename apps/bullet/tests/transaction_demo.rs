//! V1-S4 negatives and the Kernel consumer that production gitd still
//! refuses clone. The happy-path saga is `just demo`.

use bullet_domain::{GateOutcome, REASON_ZERO_TESTS};
use bullet_harness_core::transaction_proof::{
    verify_transaction_proof, TransactionProofSigningKey, TransactionProofSubject,
    TRANSACTION_PROOF_CLASS, TRANSACTION_PROOF_SCHEMA_VERSION,
};
use bullet_runner_core::gitd_binary;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn subject() -> TransactionProofSubject {
    TransactionProofSubject {
        schema_version: TRANSACTION_PROOF_SCHEMA_VERSION.into(),
        evidence_class: TRANSACTION_PROOF_CLASS.into(),
        fence_first: 1,
        fence_second: 2,
        attempt_first: "atm_1".into(),
        attempt_second: "atm_2".into(),
        candidate_id: "can_1".into(),
        verifier_outcome: "FAIL".into(),
        writer_proof_refused: true,
        effect_unknown: "OUTCOME_UNKNOWN".into(),
        effect_settled: "ORPHANED_REMOTE".into(),
        stale_refused: true,
        gitd_fixture: true,
        command_id: "cmd_1".into(),
        command_phase: "pending".into(),
    }
}

#[test]
fn signed_transaction_proof_roundtrip() {
    let key = TransactionProofSigningKey::generate("kernel-demo", "txn-proof-1").expect("key");
    let proof = key.sign(&subject()).expect("sign");
    verify_transaction_proof(&proof).expect("verify");
}

#[test]
fn painted_success_and_stale_pass_cannot_be_signed() {
    let key = TransactionProofSigningKey::generate("kernel-demo", "txn-proof-1").expect("key");
    let mut painted = subject();
    painted.command_phase = "verified".into();
    assert!(key.sign(&painted).is_err());
    let mut stale = subject();
    stale.stale_refused = false;
    assert!(key.sign(&stale).is_err());
}

#[test]
fn zero_tests_never_satisfy_a_blocking_gate() {
    assert!(!GateOutcome::NotRun.satisfies_requirement());
    assert_eq!(REASON_ZERO_TESTS, "ZERO_TESTS");
}

#[test]
fn production_gitd_constructor_child_still_refuses_clone() {
    let binary = gitd_binary();
    if !binary.is_file() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    let mut child = Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("HOME", temp.path())
        .spawn()
        .expect("spawn production gitd");
    let mut stdin = child.stdin.take().expect("stdin");
    let request = json!({
        "id": 1,
        "method": "clone",
        "token": {
            "organization_id": "org_x",
            "variant_id": format!("var_{}", "2".repeat(64)),
            "attempt_id": format!("atm_{}", "1".repeat(64)),
            "attempt_fence": 1,
            "workspace_nonce": vec![9u8; 32],
        },
        "params": {
            "source_repo": "/does/not/matter",
            "base_sha": format!("sha1:{}", "a".repeat(40)),
            "root": temp.path().join("farm").display().to_string(),
            "created_at": "2026-08-24T00:00:00Z",
            "allowed_prefixes": ["src"],
            "commit_date": "2026-08-24T00:00:00+00:00"
        }
    });
    writeln!(stdin, "{request}").expect("write");
    stdin.flush().expect("flush");
    drop(stdin);
    let mut line = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut line)
        .expect("read");
    let response: Value = serde_json::from_str(&line).expect("json");
    assert_eq!(response["err"]["code"], "AUTHORITY_CONTRACT_UNAVAILABLE");
    let _ = child.kill();
}
