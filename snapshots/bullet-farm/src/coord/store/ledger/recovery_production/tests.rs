use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};

use crate::coord::{
    ClaimState, CoordStore,
    generation::recovery::{self, adoption_fixture},
    model::{
        RECOVERY_PRODUCTION_SCHEMA_VERSION, RecoveryReviewApprovalKindV1, RecoveryReviewApprovalV1,
        RecoveryReviewDecisionV1, RecoveryReviewRequestV1,
    },
};

use super::super::git_fixture_support::{clone_repo, git_fixture};

#[test]
fn producer_chain_is_exact_replayable_and_adoptable_after_restart() {
    let (source, git_expectation) = git_fixture();
    let parent = git_expectation.parent_oid.strip_prefix("sha1:").unwrap();
    let commit = git_expectation.commit_oid.strip_prefix("sha1:").unwrap();
    let fixture = adoption_fixture::fixture(parent, commit);
    clone_repo(source.path(), &fixture.family.path().join("bullet-kernel"));
    recovery::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();

    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let store = CoordStore::with_clock(fixture.family.path().to_owned(), move || {
        observed.fetch_add(1, Ordering::SeqCst);
        Ok(30 + observed.load(Ordering::SeqCst) as u64)
    });
    let plan = store.derive_recovery_plan().unwrap();
    assert_eq!(
        plan.subject.git_expectation.commit_oid,
        format!("sha1:{commit}")
    );
    let proof_request =
        crate::coord::model::RecoveryProofRequestV1::for_plan(plan.clone()).unwrap();
    let proof = store.record_recovery_proof(&proof_request).unwrap();
    assert!(!proof.replayed);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let approval = RecoveryReviewApprovalV1 {
        kind: RecoveryReviewApprovalKindV1::RecoveryReviewApprovalV1,
        schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
        plan_id: plan.plan_id.clone(),
        evidence_subject_blake3: plan.evidence_subject_blake3.clone(),
        proof_receipt_ids: vec![proof.projection.clone()],
        reviewer: "independent-recovery-reviewer".to_owned(),
        decision: RecoveryReviewDecisionV1::Approve,
    };
    let approval_bytes = bullet_wire::canonical_json(&approval).unwrap();
    let approval_sha = format!("sha256:{:x}", Sha256::digest(&approval_bytes));
    let review_request =
        RecoveryReviewRequestV1::from_approval(plan, approval, approval_sha).unwrap();
    let review = store.record_recovery_review(&review_request).unwrap();
    assert!(!review.replayed);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let request = store
        .build_recovery_adoption_request(&review_request)
        .unwrap();
    request.validate().unwrap();
    let applied = store.adopt_recovery_receipts(&request).unwrap();
    assert!(
        applied
            .projection
            .iter()
            .all(|claim| claim.state == ClaimState::RecoveredReceipted)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    let restarted = CoordStore::with_clock(fixture.family.path().to_owned(), || Ok(100));
    let status = restarted.status().unwrap();
    assert!(
        status
            .claims
            .iter()
            .filter(|claim| fixture.claim_ids.contains(&claim.claim_id))
            .all(|claim| claim.state == ClaimState::RecoveredReceipted)
    );

    let segment = fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(fixture.manifest.generation_id().as_str())
        .join("events.jsonl");
    let length = fs::metadata(&segment).unwrap().len();
    fs::rename(
        fixture.family.path().join("bullet-kernel/.git"),
        fixture.family.path().join("bullet-kernel/.git-unavailable"),
    )
    .unwrap();
    let replay = CoordStore::with_clock(fixture.family.path().to_owned(), || {
        panic!("proof replay invoked the clock")
    })
    .record_recovery_proof(&proof_request)
    .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.receipt, proof.receipt);
    assert_eq!(fs::metadata(segment).unwrap().len(), length);
}

#[test]
fn review_conflict_is_zero_append_and_zero_clock() {
    let (source, git_expectation) = git_fixture();
    let parent = git_expectation.parent_oid.strip_prefix("sha1:").unwrap();
    let commit = git_expectation.commit_oid.strip_prefix("sha1:").unwrap();
    let fixture = adoption_fixture::fixture(parent, commit);
    clone_repo(source.path(), &fixture.family.path().join("bullet-kernel"));
    recovery::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    let store = CoordStore::with_clock(fixture.family.path().to_owned(), || Ok(30));
    let plan = store.derive_recovery_plan().unwrap();
    let proof_request =
        crate::coord::model::RecoveryProofRequestV1::for_plan(plan.clone()).unwrap();
    let proof = store.record_recovery_proof(&proof_request).unwrap();
    let approval = |reviewer: &str| RecoveryReviewApprovalV1 {
        kind: RecoveryReviewApprovalKindV1::RecoveryReviewApprovalV1,
        schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
        plan_id: plan.plan_id.clone(),
        evidence_subject_blake3: plan.evidence_subject_blake3.clone(),
        proof_receipt_ids: vec![proof.projection.clone()],
        reviewer: reviewer.to_owned(),
        decision: RecoveryReviewDecisionV1::Approve,
    };
    let first_approval = approval("reviewer-one");
    let first = RecoveryReviewRequestV1::from_approval(
        plan.clone(),
        first_approval.clone(),
        format!(
            "sha256:{:x}",
            Sha256::digest(bullet_wire::canonical_json(&first_approval).unwrap())
        ),
    )
    .unwrap();
    store.record_recovery_review(&first).unwrap();
    let segment = fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(fixture.manifest.generation_id().as_str())
        .join("events.jsonl");
    let length = fs::metadata(&segment).unwrap().len();
    let changed_approval = approval("reviewer-two");
    let changed = RecoveryReviewRequestV1::from_approval(
        plan,
        changed_approval.clone(),
        format!(
            "sha256:{:x}",
            Sha256::digest(bullet_wire::canonical_json(&changed_approval).unwrap())
        ),
    )
    .unwrap();
    let replay = CoordStore::with_clock(fixture.family.path().to_owned(), || {
        panic!("conflict invoked clock")
    });
    assert_eq!(
        replay.record_recovery_review(&changed).unwrap_err().code(),
        "COORD_REQUEST_CONFLICT"
    );
    assert_eq!(fs::metadata(segment).unwrap().len(), length);
}

#[test]
fn process_facing_cli_executes_the_complete_sealed_sequence() {
    let (source, git_expectation) = git_fixture();
    let parent = git_expectation.parent_oid.strip_prefix("sha1:").unwrap();
    let commit = git_expectation.commit_oid.strip_prefix("sha1:").unwrap();
    let fixture = adoption_fixture::fixture(parent, commit);
    fs::set_permissions(fixture.family.path(), fs::Permissions::from_mode(0o700)).unwrap();
    clone_repo(source.path(), &fixture.family.path().join("bullet-kernel"));
    recovery::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();

    let plan_path = fixture.family.path().join("recovery-plan.json");
    cli(
        fixture.family.path(),
        "recovery-plan",
        &["--output", plan_path.to_str().unwrap()],
    );
    assert_eq!(
        fs::metadata(&plan_path).unwrap().permissions().mode() & 0o777,
        0o400
    );
    let plan_bytes = fs::read(&plan_path).unwrap();
    let plan = bullet_wire::decode_canonical::<crate::coord::RecoveryProductionPlanV1>(
        &plan_bytes[..plan_bytes.len() - 1],
    )
    .unwrap();

    let proof_output = cli(
        fixture.family.path(),
        "recovery-proof",
        &["--plan", plan_path.to_str().unwrap()],
    );
    let proof: serde_json::Value = serde_json::from_str(&proof_output).unwrap();
    let proof_id = proof["projection"].as_str().unwrap().to_owned();
    let approval = RecoveryReviewApprovalV1 {
        kind: RecoveryReviewApprovalKindV1::RecoveryReviewApprovalV1,
        schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
        plan_id: plan.plan_id.clone(),
        evidence_subject_blake3: plan.evidence_subject_blake3.clone(),
        proof_receipt_ids: vec![proof_id],
        reviewer: "independent-cli-reviewer".to_owned(),
        decision: RecoveryReviewDecisionV1::Approve,
    };
    let approval_path = fixture.family.path().join("recovery-approval.json");
    let mut approval_bytes = bullet_wire::canonical_json(&approval).unwrap();
    approval_bytes.push(b'\n');
    fs::write(&approval_path, approval_bytes).unwrap();
    fs::set_permissions(&approval_path, fs::Permissions::from_mode(0o400)).unwrap();

    cli(
        fixture.family.path(),
        "recovery-review",
        &[
            "--plan",
            plan_path.to_str().unwrap(),
            "--approval",
            approval_path.to_str().unwrap(),
        ],
    );
    let request_path = fixture.family.path().join("recovery-request.json");
    cli(
        fixture.family.path(),
        "recovery-request",
        &[
            "--plan",
            plan_path.to_str().unwrap(),
            "--approval",
            approval_path.to_str().unwrap(),
            "--output",
            request_path.to_str().unwrap(),
        ],
    );
    assert_eq!(
        fs::metadata(&request_path).unwrap().permissions().mode() & 0o777,
        0o400
    );
    let adoption_output = cli(
        fixture.family.path(),
        "adopt",
        &["--request", request_path.to_str().unwrap()],
    );
    let adoption: serde_json::Value = serde_json::from_str(&adoption_output).unwrap();
    assert_eq!(adoption["replayed"], false);
    assert_eq!(adoption["projection"].as_array().unwrap().len(), 2);
    let restarted = CoordStore::with_clock(fixture.family.path().to_owned(), || Ok(100));
    assert!(restarted.status().unwrap().claims.iter().all(|claim| {
        !fixture.claim_ids.contains(&claim.claim_id)
            || claim.state == ClaimState::RecoveredReceipted
    }));
}

fn cli(root: &std::path::Path, action: &str, options: &[&str]) -> String {
    let mut args = vec![
        OsString::from("bullet-family"),
        OsString::from("--root"),
        root.as_os_str().to_owned(),
        OsString::from("coord"),
        OsString::from(action),
    ];
    args.extend(options.iter().map(OsString::from));
    crate::cli::run(args, Ok(root.to_owned())).unwrap()
}
