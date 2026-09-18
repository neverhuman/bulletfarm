use std::collections::BTreeSet;

use super::mismatch;
use crate::coord::{
    CoordError,
    generation::segment::StoredEnvelope,
    model::{
        Record, RecoveryGenerationRecordKindV1, RecoveryGenerationRecordRefV1,
        RecoveryReceiptAdoptionRequestV1,
    },
};

pub(super) struct GenerationEvidenceOutcome {
    pub(super) recovery_orchestrator: String,
    pub(super) reviewer: String,
}

pub(super) fn verify(
    request: &RecoveryReceiptAdoptionRequestV1,
    entries: &[StoredEnvelope],
    expected_subject: &str,
) -> Result<GenerationEvidenceOutcome, CoordError> {
    let mut orchestrator = None;
    let mut proof_ids = Vec::new();
    let mut proof_sequences = BTreeSet::new();
    for observation in &request.subject.proof_observations {
        if observation.expected_subject_blake3 != expected_subject {
            return Err(mismatch(
                "proof receipt does not bind the canonical recovery evidence subject",
            ));
        }
        let entry = exact_entry(entries, &observation.record)?;
        let Record::RecoveryProofReceiptV1 { body, .. } = &entry.record else {
            return Err(mismatch("proof reference resolves to another record kind"));
        };
        body.validate()?;
        if body.subject_blake3() != expected_subject {
            return Err(mismatch("proof receipt body names another subject"));
        }
        if orchestrator
            .as_ref()
            .is_some_and(|value| value != body.recovery_orchestrator())
        {
            return Err(mismatch(
                "proof receipts do not share one recovery orchestrator",
            ));
        }
        orchestrator = Some(body.recovery_orchestrator().to_owned());
        proof_ids.push(body.proof_receipt_id().to_owned());
        proof_sequences.insert(entry.sequence);
    }
    proof_ids.sort();
    let review_observation = &request.subject.review_observation;
    if review_observation.expected_subject_blake3 != expected_subject {
        return Err(mismatch(
            "review receipt does not bind the canonical recovery evidence subject",
        ));
    }
    let review_entry = exact_entry(entries, &review_observation.record)?;
    if proof_sequences
        .last()
        .is_some_and(|sequence| *sequence >= review_entry.sequence)
    {
        return Err(mismatch(
            "independent review receipt does not follow every proof receipt",
        ));
    }
    let Record::RecoveryReviewReceiptV1 { body, .. } = &review_entry.record else {
        return Err(mismatch("review reference resolves to another record kind"));
    };
    body.validate()?;
    let orchestrator = orchestrator.ok_or_else(|| mismatch("proof receipts are absent"))?;
    if body.subject_blake3() != expected_subject
        || body.proof_receipt_ids() != proof_ids
        || body.recovery_orchestrator() != orchestrator
        || body.reviewer() == orchestrator
    {
        return Err(mismatch(
            "independent review does not approve the exact proof receipt set",
        ));
    }
    Ok(GenerationEvidenceOutcome {
        recovery_orchestrator: orchestrator,
        reviewer: body.reviewer().to_owned(),
    })
}

fn exact_entry<'a>(
    entries: &'a [StoredEnvelope],
    reference: &RecoveryGenerationRecordRefV1,
) -> Result<&'a StoredEnvelope, CoordError> {
    let entry = entries
        .iter()
        .find(|entry| entry.sequence == reference.sequence)
        .ok_or_else(|| mismatch("recovery evidence sequence is absent"))?;
    let kind = match entry.record {
        Record::RecoveryProofReceiptV1 { .. } => RecoveryGenerationRecordKindV1::ProofReceipt,
        Record::RecoveryReviewReceiptV1 { .. } => RecoveryGenerationRecordKindV1::ReviewReceipt,
        _ => {
            return Err(mismatch(
                "recovery evidence sequence has another record kind",
            ));
        }
    };
    if entry.generation_id != reference.generation_id
        || entry.request_id != reference.request_id.as_str()
        || entry.request_digest != reference.request_blake3
        || entry.receipt.request_id != reference.request_id.as_str()
        || entry.receipt.sequence != reference.sequence
        || entry.receipt.request_digest != reference.request_blake3
        || entry.receipt.record_digest != reference.record_blake3
        || entry.receipt.envelope_digest != reference.envelope_blake3
        || entry.receipt.byte_offset != reference.byte_offset
        || entry.receipt.frame_length != reference.frame_length
        || kind != reference.expected_record_kind
    {
        return Err(mismatch(
            "recovery evidence receipt reference differs from the stored envelope",
        ));
    }
    Ok(entry)
}

#[cfg(test)]
#[path = "generation/tests.rs"]
mod tests;
