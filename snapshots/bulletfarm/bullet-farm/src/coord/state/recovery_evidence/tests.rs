use super::*;
use crate::coord::model::{
    FrozenClaimSubject, GENERATION_SCHEMA_VERSION, RecoveryBaselineBody,
    RecoveryProofReceiptRecordV1, RecoveryReceiptAdoptionRecordV1,
    RecoveryReceiptAdoptionRequestV1, RecoveryReviewReceiptRecordV1,
    recovery_adoption_request_fixture,
};

struct Prepared {
    authority: RecoveryAdoptionAuthority,
    proof: Record,
    review: Record,
    adoption: RecoveryReceiptAdoptionRecordV1,
}

fn prepared() -> Prepared {
    let mut request = recovery_adoption_request_fixture();
    let subject = recovery_adoption_verify::evidence_subject(&request).unwrap();
    for proof in &mut request.subject.proof_observations {
        proof.expected_subject_blake3 = subject.clone();
    }
    request
        .subject
        .review_observation
        .expected_subject_blake3
        .clone_from(&subject);
    request.validate().unwrap();
    let proof_body = proof(&subject, "recovery-orchestrator");
    let proof_id = proof_body.proof_receipt_id().to_owned();
    let review_body = review(
        &subject,
        vec![proof_id],
        "recovery-orchestrator",
        "independent-reviewer",
    );
    let authority = authority(&request);
    let adoption = adoption(request, "recovery-orchestrator", "independent-reviewer");
    Prepared {
        authority,
        proof: Record::RecoveryProofReceiptV1 {
            schema_version: GENERATION_SCHEMA_VERSION,
            at_unix_ms: 21,
            body: proof_body,
        },
        review: Record::RecoveryReviewReceiptV1 {
            schema_version: GENERATION_SCHEMA_VERSION,
            at_unix_ms: 22,
            body: review_body,
        },
        adoption,
    }
}

fn proof(subject: &str, orchestrator: &str) -> RecoveryProofReceiptRecordV1 {
    RecoveryProofReceiptRecordV1::verified_pass(
        subject.to_owned(),
        orchestrator.to_owned(),
        tagged("sha256:", '1'),
        tagged("sha256:", '2'),
        3,
    )
    .unwrap()
}

fn review(
    subject: &str,
    proof_ids: Vec<String>,
    orchestrator: &str,
    reviewer: &str,
) -> RecoveryReviewReceiptRecordV1 {
    RecoveryReviewReceiptRecordV1::verified_approval(
        subject.to_owned(),
        proof_ids,
        orchestrator.to_owned(),
        reviewer.to_owned(),
        tagged("sha256:", '3'),
    )
    .unwrap()
}

fn adoption(
    request: RecoveryReceiptAdoptionRequestV1,
    orchestrator: &str,
    reviewer: &str,
) -> RecoveryReceiptAdoptionRecordV1 {
    RecoveryReceiptAdoptionRecordV1::verified(
        request,
        "recovery-operator".to_owned(),
        tagged("sha256:", '4'),
        tagged("sha256:", '5'),
        1,
        tagged("sha256:", '6'),
        orchestrator.to_owned(),
        reviewer.to_owned(),
    )
    .unwrap()
}

fn authority(request: &RecoveryReceiptAdoptionRequestV1) -> RecoveryAdoptionAuthority {
    let body = RecoveryBaselineBody {
        manifest_blake3: request.expected_watermark.manifest_blake3.clone(),
        incident_at_unix_ms: 10,
        recovered_at_unix_ms: 20,
        trusted_state_blake3: tagged("blake3:", '7'),
        frozen_claims: request
            .subject
            .claims
            .iter()
            .map(|claim| FrozenClaimSubject {
                claim_id: claim.claim_id.clone(),
                claim_blake3: claim.frozen_claim_blake3.clone(),
            })
            .collect(),
    };
    RecoveryAdoptionAuthority::from_baseline(&request.expected_watermark.generation_id, &body)
}

fn tagged(prefix: &str, marker: char) -> String {
    format!("{prefix}{}", marker.to_string().repeat(64))
}

#[test]
fn exact_earlier_proof_and_review_admit_adoption() {
    let prepared = prepared();
    let mut state = RecoveryEvidenceState::default();
    state
        .apply(&prepared.proof, Some(&prepared.authority))
        .unwrap();
    state
        .apply(&prepared.review, Some(&prepared.authority))
        .unwrap();
    state.verify_adoption(&prepared.adoption).unwrap();
}

#[test]
fn missing_or_reordered_evidence_refuses() {
    let prepared = prepared();
    assert_eq!(
        RecoveryEvidenceState::default()
            .verify_adoption(&prepared.adoption)
            .unwrap_err()
            .code(),
        "CORRUPT_COORD_LOG"
    );
    let mut state = RecoveryEvidenceState::default();
    assert_eq!(
        state
            .apply(&prepared.review, Some(&prepared.authority))
            .unwrap_err()
            .code(),
        "CORRUPT_COORD_LOG"
    );
}

#[test]
fn wrong_subject_or_actor_refuses() {
    let prepared = prepared();
    let request = prepared.adoption.request();
    let expected = recovery_adoption_verify::evidence_subject(request).unwrap();
    for (proof_subject, proof_orchestrator, review_orchestrator) in [
        (
            tagged("blake3:", '0'),
            "recovery-orchestrator",
            "recovery-orchestrator",
        ),
        (
            expected.clone(),
            "other-orchestrator",
            "recovery-orchestrator",
        ),
    ] {
        let proof = proof(&proof_subject, proof_orchestrator);
        let review = review(
            &expected,
            vec![proof.proof_receipt_id().to_owned()],
            review_orchestrator,
            "independent-reviewer",
        );
        let mut state = RecoveryEvidenceState::default();
        state
            .apply(
                &Record::RecoveryProofReceiptV1 {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: 21,
                    body: proof,
                },
                Some(&prepared.authority),
            )
            .unwrap();
        assert_eq!(
            state
                .apply(
                    &Record::RecoveryReviewReceiptV1 {
                        schema_version: GENERATION_SCHEMA_VERSION,
                        at_unix_ms: 22,
                        body: review,
                    },
                    Some(&prepared.authority),
                )
                .unwrap_err()
                .code(),
            "CORRUPT_COORD_LOG"
        );
    }
}
