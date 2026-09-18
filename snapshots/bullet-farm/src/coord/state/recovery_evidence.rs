use std::collections::BTreeMap;

use super::{as_corrupt, corrupt, recovery_adoption::RecoveryAdoptionAuthority};
use crate::coord::{
    CoordError,
    model::{
        Record, RecoveryProofReceiptRecordV1, RecoveryReceiptAdoptionRecordV1,
        RecoveryReviewReceiptRecordV1,
    },
    recovery_adoption_verify,
};

#[derive(Default)]
pub(super) struct RecoveryEvidenceState {
    proof_receipts: BTreeMap<String, RecoveryProofReceiptRecordV1>,
    review_receipts: BTreeMap<String, RecoveryReviewReceiptRecordV1>,
}

impl RecoveryEvidenceState {
    pub(super) fn apply(
        &mut self,
        record: &Record,
        authority: Option<&RecoveryAdoptionAuthority>,
    ) -> Result<(), CoordError> {
        let authority = authority.ok_or_else(|| {
            corrupt("recovery evidence receipt has no recovery baseline authority")
        })?;
        match record {
            Record::RecoveryProofReceiptV1 {
                schema_version,
                at_unix_ms,
                body,
            } => {
                validate_header(*schema_version, *at_unix_ms, authority)?;
                body.validate().map_err(as_corrupt)?;
                if self
                    .proof_receipts
                    .insert(body.proof_receipt_id().to_owned(), body.clone())
                    .is_some()
                {
                    return Err(corrupt("duplicate recovery proof receipt ID"));
                }
            }
            Record::RecoveryReviewReceiptV1 {
                schema_version,
                at_unix_ms,
                body,
            } => {
                validate_header(*schema_version, *at_unix_ms, authority)?;
                body.validate().map_err(as_corrupt)?;
                for proof_id in body.proof_receipt_ids() {
                    let proof = self.proof_receipts.get(proof_id).ok_or_else(|| {
                        corrupt("recovery review precedes or omits a named proof receipt")
                    })?;
                    if proof.subject_blake3() != body.subject_blake3()
                        || proof.recovery_orchestrator() != body.recovery_orchestrator()
                    {
                        return Err(corrupt(
                            "recovery review names a proof for another subject or orchestrator",
                        ));
                    }
                }
                if self
                    .review_receipts
                    .insert(body.review_receipt_id().to_owned(), body.clone())
                    .is_some()
                {
                    return Err(corrupt("duplicate recovery review receipt ID"));
                }
            }
            _ => {
                return Err(corrupt(
                    "recovery evidence replay received another record kind",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn verify_adoption(
        &self,
        adoption: &RecoveryReceiptAdoptionRecordV1,
    ) -> Result<(), CoordError> {
        adoption.validate().map_err(as_corrupt)?;
        let request = adoption.request();
        let subject = recovery_adoption_verify::evidence_subject(request).map_err(as_corrupt)?;
        if request
            .subject
            .proof_observations
            .iter()
            .any(|proof| proof.expected_subject_blake3 != subject)
            || request.subject.review_observation.expected_subject_blake3 != subject
        {
            return Err(corrupt(
                "recovery adoption observations name another evidence subject",
            ));
        }
        let reviews = self
            .review_receipts
            .values()
            .filter(|review| {
                review.subject_blake3() == subject
                    && review.recovery_orchestrator() == adoption.verified_orchestrator()
                    && review.reviewer() == adoption.verified_reviewer()
                    && review.proof_receipt_ids().len() == request.subject.proof_observations.len()
            })
            .collect::<Vec<_>>();
        let [review] = reviews.as_slice() else {
            return Err(corrupt(
                "recovery adoption lacks one exact earlier independent review",
            ));
        };
        for proof_id in review.proof_receipt_ids() {
            let proof = self
                .proof_receipts
                .get(proof_id)
                .ok_or_else(|| corrupt("recovery adoption review names a missing proof"))?;
            if proof.subject_blake3() != subject
                || proof.recovery_orchestrator() != adoption.verified_orchestrator()
            {
                return Err(corrupt(
                    "recovery adoption proof subject or orchestrator differs",
                ));
            }
        }
        Ok(())
    }
}

fn validate_header(
    schema_version: u32,
    at_unix_ms: u64,
    authority: &RecoveryAdoptionAuthority,
) -> Result<(), CoordError> {
    if schema_version != super::super::model::GENERATION_SCHEMA_VERSION {
        return Err(corrupt(
            "recovery evidence receipt uses an unsupported schema",
        ));
    }
    authority.validate_time(at_unix_ms).map_err(as_corrupt)
}

#[cfg(test)]
#[path = "recovery_evidence/tests.rs"]
mod tests;
