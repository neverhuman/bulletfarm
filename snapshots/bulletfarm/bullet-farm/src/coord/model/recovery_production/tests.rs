use super::*;
use crate::coord::{model::recovery_adoption::fixture_request, recovery_adoption_verify};

fn plan() -> RecoveryProductionPlanV1 {
    let request = fixture_request();
    let evidence_subject_blake3 = recovery_adoption_verify::evidence_subject(&request).unwrap();
    let watermark = request.expected_watermark;
    RecoveryProductionPlanV1::derive(
        evidence_subject_blake3,
        RecoveryProductionWatermarkV1 {
            generation_id: watermark.generation_id,
            manifest_blake3: watermark.manifest_blake3,
            last_sequence: watermark.last_sequence,
            next_sequence: watermark.next_sequence,
            head_envelope_blake3: watermark.head_envelope_blake3,
            last_record_blake3: watermark.last_record_blake3,
            last_request_id: watermark.last_request_id.as_str().to_owned(),
            last_request_blake3: watermark.last_request_blake3,
            byte_length: watermark.byte_length,
        },
        "fresh-recovery-orchestrator".to_owned(),
        RecoveryProductionSubjectV1 {
            repo: request.subject.repo,
            git_expectation: request.subject.git_expectation,
            claims: request.subject.claims,
            group_receipt_observation: request.subject.group_receipt_observation,
        },
    )
    .unwrap()
}

fn approval(plan: &RecoveryProductionPlanV1) -> RecoveryReviewApprovalV1 {
    RecoveryReviewApprovalV1 {
        kind: RecoveryReviewApprovalKindV1::RecoveryReviewApprovalV1,
        schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
        plan_id: plan.plan_id.clone(),
        evidence_subject_blake3: plan.evidence_subject_blake3.clone(),
        proof_receipt_ids: vec![format!("rpf_{}", "a".repeat(64))],
        reviewer: "independent-reviewer".to_owned(),
        decision: RecoveryReviewDecisionV1::Approve,
    }
}

#[test]
fn plan_and_request_identities_are_deterministic_and_closed() {
    let plan = plan();
    plan.validate().unwrap();
    assert_eq!(plan, self::plan());

    let proof = RecoveryProofRequestV1::for_plan(plan.clone()).unwrap();
    assert_eq!(
        proof,
        RecoveryProofRequestV1::for_plan(plan.clone()).unwrap()
    );
    let review = RecoveryReviewRequestV1::from_approval(
        plan.clone(),
        approval(&plan),
        format!("sha256:{}", "b".repeat(64)),
    )
    .unwrap();
    review.validate().unwrap();

    let mut changed = plan;
    changed.expected_watermark.byte_length += 1;
    assert!(changed.validate().is_err());
}

#[test]
fn review_is_approve_only_independent_and_exact() {
    let plan = plan();
    let mut value = approval(&plan);
    value.validate().unwrap();
    value.reviewer = plan.recovery_orchestrator.clone();
    assert!(
        RecoveryReviewRequestV1::from_approval(
            plan.clone(),
            value,
            format!("sha256:{}", "b".repeat(64)),
        )
        .is_err()
    );

    let mut duplicate = approval(&plan);
    duplicate
        .proof_receipt_ids
        .push(duplicate.proof_receipt_ids[0].clone());
    assert!(duplicate.validate().is_err());

    let canonical =
        String::from_utf8(bullet_wire::canonical_json(&approval(&plan)).unwrap()).unwrap();
    let denied = canonical.replace("\"APPROVE\"", "\"UNKNOWN\"");
    assert!(bullet_wire::decode_canonical::<RecoveryReviewApprovalV1>(denied.as_bytes()).is_err());
    let unknown = canonical.replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"extra\":true",
        1,
    );
    assert!(bullet_wire::decode_canonical::<RecoveryReviewApprovalV1>(unknown.as_bytes()).is_err());
}
