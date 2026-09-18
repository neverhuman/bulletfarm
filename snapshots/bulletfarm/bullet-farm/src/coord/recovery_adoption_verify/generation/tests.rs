use super::*;
use crate::coord::{
    generation::segment::AppendReceipt,
    model::{
        GENERATION_SCHEMA_VERSION, RecoveryProofReceiptRecordV1, RecoveryReceiptAdoptionRequestV1,
        RecoveryReviewReceiptRecordV1, recovery_adoption_request_fixture,
    },
};

struct Prepared {
    request: RecoveryReceiptAdoptionRequestV1,
    subject: String,
    entries: Vec<StoredEnvelope>,
}

fn prepared() -> Prepared {
    let mut request = recovery_adoption_request_fixture();
    let subject = super::super::evidence_subject(&request).unwrap();
    for proof in &mut request.subject.proof_observations {
        proof.expected_subject_blake3 = subject.clone();
    }
    request
        .subject
        .review_observation
        .expected_subject_blake3
        .clone_from(&subject);
    request.validate().unwrap();
    let proof = RecoveryProofReceiptRecordV1::verified_pass(
        subject.clone(),
        "recovery-orchestrator".to_owned(),
        tagged("sha256:", '1'),
        tagged("sha256:", '2'),
        3,
    )
    .unwrap();
    let review = RecoveryReviewReceiptRecordV1::verified_approval(
        subject.clone(),
        vec![proof.proof_receipt_id().to_owned()],
        "recovery-orchestrator".to_owned(),
        "independent-reviewer".to_owned(),
        tagged("sha256:", '3'),
    )
    .unwrap();
    let entries = vec![
        envelope(
            &request.subject.proof_observations[0].record,
            Record::RecoveryProofReceiptV1 {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 21,
                body: proof,
            },
        ),
        envelope(
            &request.subject.review_observation.record,
            Record::RecoveryReviewReceiptV1 {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 22,
                body: review,
            },
        ),
    ];
    Prepared {
        request,
        subject,
        entries,
    }
}

fn envelope(reference: &RecoveryGenerationRecordRefV1, record: Record) -> StoredEnvelope {
    StoredEnvelope {
        generation_id: reference.generation_id.clone(),
        sequence: reference.sequence,
        previous_digest: "0".repeat(64),
        request_id: reference.request_id.as_str().to_owned(),
        request_digest: reference.request_blake3.clone(),
        record,
        receipt: AppendReceipt {
            sequence: reference.sequence,
            envelope_digest: reference.envelope_blake3.clone(),
            record_digest: reference.record_blake3.clone(),
            request_id: reference.request_id.as_str().to_owned(),
            request_digest: reference.request_blake3.clone(),
            byte_offset: reference.byte_offset,
            frame_length: reference.frame_length,
        },
    }
}

fn tagged(prefix: &str, marker: char) -> String {
    format!("{prefix}{}", marker.to_string().repeat(64))
}

fn assert_mismatch(request: &RecoveryReceiptAdoptionRequestV1, entries: &[StoredEnvelope]) {
    let Err(error) = verify(
        request,
        entries,
        &super::super::evidence_subject(request).unwrap(),
    ) else {
        panic!("changed or missing recovery evidence unexpectedly verified");
    };
    assert_eq!(error.code(), "RECOVERY_EVIDENCE_MISMATCH");
}

#[test]
fn exact_envelopes_and_bodies_pass() {
    let prepared = prepared();
    let outcome = verify(&prepared.request, &prepared.entries, &prepared.subject).unwrap();
    assert_eq!(outcome.recovery_orchestrator, "recovery-orchestrator");
    assert_eq!(outcome.reviewer, "independent-reviewer");
}

#[test]
fn missing_or_changed_public_envelope_reference_refuses() {
    let prepared = prepared();
    assert_mismatch(&prepared.request, &prepared.entries[1..]);
    for field in 0..7 {
        let mut changed = prepared.request.clone();
        let reference = &mut changed.subject.proof_observations[0].record;
        match field {
            0 => {
                reference.request_id = crate::coord::RequestId::parse(tagged("req_", '0')).unwrap()
            }
            1 => reference.request_blake3 = "0".repeat(64),
            2 => reference.record_blake3 = "0".repeat(64),
            3 => reference.envelope_blake3 = "0".repeat(64),
            4 => reference.byte_offset += 1,
            5 => reference.frame_length += 1,
            _ => reference.sequence -= 1,
        }
        changed.validate().unwrap();
        assert_mismatch(&changed, &prepared.entries);
    }
}

#[test]
fn wrong_body_subject_or_actor_refuses() {
    let prepared = prepared();
    for (subject, orchestrator) in [
        (tagged("blake3:", '0'), "recovery-orchestrator"),
        (prepared.subject.clone(), "other-orchestrator"),
    ] {
        let mut entries = prepared.entries.clone();
        entries[0].record = Record::RecoveryProofReceiptV1 {
            schema_version: GENERATION_SCHEMA_VERSION,
            at_unix_ms: 21,
            body: RecoveryProofReceiptRecordV1::verified_pass(
                subject,
                orchestrator.to_owned(),
                tagged("sha256:", '1'),
                tagged("sha256:", '2'),
                3,
            )
            .unwrap(),
        };
        assert_mismatch(&prepared.request, &entries);
    }
}
