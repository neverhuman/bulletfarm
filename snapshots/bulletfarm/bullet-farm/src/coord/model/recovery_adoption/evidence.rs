use serde::{Deserialize, Serialize};

use super::{
    MAX_RECOVERY_PROOF_RECEIPTS, RecoveryAdoptionAuthorityClassV1, RecoveryAdoptionWatermarkV1,
    RecoveryGenerationRecordKindV1, RecoveryGenerationRecordRefV1, RecoveryProofObservationV1,
    RecoveryReviewObservationV1,
    validate::{bare_blake3, invalid, tagged},
};
use crate::coord::{CoordError, validate_field};

const PROOF_ID_DOMAIN: &str = "bullet-family.coord.recovery-proof-receipt.v1";
const REVIEW_ID_DOMAIN: &str = "bullet-family.coord.recovery-review-receipt.v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

impl RecoveryGenerationRecordRefV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.generation_id, "gen_")?;
        safe(self.sequence, "recovery evidence sequence")?;
        self.request_id.validate()?;
        bare_blake3(&self.request_blake3)?;
        bare_blake3(&self.record_blake3)?;
        bare_blake3(&self.envelope_blake3)?;
        safe(self.byte_offset, "recovery evidence byte offset")?;
        safe(self.frame_length, "recovery evidence frame length")?;
        if self.sequence < 2 || self.byte_offset == 0 || self.frame_length == 0 {
            return Err(invalid(
                "recovery evidence reference cannot identify the generation baseline",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_against(
        &self,
        watermark: &RecoveryAdoptionWatermarkV1,
    ) -> Result<(), CoordError> {
        self.validate()?;
        let end = self
            .byte_offset
            .checked_add(self.frame_length)
            .ok_or_else(|| invalid("recovery evidence frame range overflowed"))?;
        if self.generation_id != watermark.generation_id
            || self.sequence > watermark.last_sequence
            || end > watermark.byte_length
        {
            return Err(invalid(
                "recovery evidence reference is outside the expected watermark",
            ));
        }
        Ok(())
    }
}

impl RecoveryProofObservationV1 {
    pub(super) fn validate(&self) -> Result<(), CoordError> {
        self.record.validate()?;
        tagged(&self.expected_subject_blake3, "blake3:")?;
        if self.record.expected_record_kind != RecoveryGenerationRecordKindV1::ProofReceipt {
            return Err(invalid("proof observation has the wrong generation kind"));
        }
        Ok(())
    }
}

impl RecoveryReviewObservationV1 {
    pub(super) fn validate(&self) -> Result<(), CoordError> {
        self.record.validate()?;
        tagged(&self.expected_subject_blake3, "blake3:")?;
        if self.record.expected_record_kind != RecoveryGenerationRecordKindV1::ReviewReceipt {
            return Err(invalid("review observation has the wrong generation kind"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryProofDispositionV1 {
    Pass,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryReviewDecisionV1 {
    Approve,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryProofReceiptRecordV1 {
    proof_receipt_id: String,
    subject_blake3: String,
    recovery_orchestrator: String,
    proof_command_sha256: String,
    proof_output_sha256: String,
    passed_checks: u32,
    failed_checks: u32,
    skipped_checks: u32,
    unknown_checks: u32,
    disposition: RecoveryProofDispositionV1,
    authority_class: RecoveryAdoptionAuthorityClassV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryReviewReceiptRecordV1 {
    review_receipt_id: String,
    subject_blake3: String,
    proof_receipt_ids: Vec<String>,
    recovery_orchestrator: String,
    reviewer: String,
    review_evidence_sha256: String,
    decision: RecoveryReviewDecisionV1,
    authority_class: RecoveryAdoptionAuthorityClassV1,
}

#[derive(Serialize)]
struct ProofIdentity<'a> {
    subject_blake3: &'a str,
    recovery_orchestrator: &'a str,
    proof_command_sha256: &'a str,
    proof_output_sha256: &'a str,
    passed_checks: u32,
}

#[derive(Serialize)]
struct ReviewIdentity<'a> {
    subject_blake3: &'a str,
    proof_receipt_ids: &'a [String],
    recovery_orchestrator: &'a str,
    reviewer: &'a str,
    review_evidence_sha256: &'a str,
}

impl RecoveryProofReceiptRecordV1 {
    pub(in crate::coord) fn verified_pass(
        subject_blake3: String,
        recovery_orchestrator: String,
        proof_command_sha256: String,
        proof_output_sha256: String,
        passed_checks: u32,
    ) -> Result<Self, CoordError> {
        let identity = ProofIdentity {
            subject_blake3: &subject_blake3,
            recovery_orchestrator: &recovery_orchestrator,
            proof_command_sha256: &proof_command_sha256,
            proof_output_sha256: &proof_output_sha256,
            passed_checks,
        };
        let proof_receipt_id = id(PROOF_ID_DOMAIN, "rpf_", &identity)?;
        let value = Self {
            proof_receipt_id,
            subject_blake3,
            recovery_orchestrator,
            proof_command_sha256,
            proof_output_sha256,
            passed_checks,
            failed_checks: 0,
            skipped_checks: 0,
            unknown_checks: 0,
            disposition: RecoveryProofDispositionV1::Pass,
            authority_class: RecoveryAdoptionAuthorityClassV1::LocalOsAuthority,
        };
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn proof_receipt_id(&self) -> &str {
        &self.proof_receipt_id
    }

    pub(in crate::coord) fn subject_blake3(&self) -> &str {
        &self.subject_blake3
    }

    pub(in crate::coord) fn recovery_orchestrator(&self) -> &str {
        &self.recovery_orchestrator
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.proof_receipt_id, "rpf_")?;
        tagged(&self.subject_blake3, "blake3:")?;
        tagged(&self.proof_command_sha256, "sha256:")?;
        tagged(&self.proof_output_sha256, "sha256:")?;
        validate_field("recovery_orchestrator", &self.recovery_orchestrator)
            .map_err(|error| invalid(error.to_string()))?;
        let expected = id(
            PROOF_ID_DOMAIN,
            "rpf_",
            &ProofIdentity {
                subject_blake3: &self.subject_blake3,
                recovery_orchestrator: &self.recovery_orchestrator,
                proof_command_sha256: &self.proof_command_sha256,
                proof_output_sha256: &self.proof_output_sha256,
                passed_checks: self.passed_checks,
            },
        )?;
        if self.passed_checks == 0
            || self.failed_checks != 0
            || self.skipped_checks != 0
            || self.unknown_checks != 0
            || self.proof_receipt_id != expected
        {
            return Err(invalid("recovery proof receipt is not an exact PASS"));
        }
        Ok(())
    }
}

impl RecoveryReviewReceiptRecordV1 {
    pub(in crate::coord) fn verified_approval(
        subject_blake3: String,
        proof_receipt_ids: Vec<String>,
        recovery_orchestrator: String,
        reviewer: String,
        review_evidence_sha256: String,
    ) -> Result<Self, CoordError> {
        let identity = ReviewIdentity {
            subject_blake3: &subject_blake3,
            proof_receipt_ids: &proof_receipt_ids,
            recovery_orchestrator: &recovery_orchestrator,
            reviewer: &reviewer,
            review_evidence_sha256: &review_evidence_sha256,
        };
        let review_receipt_id = id(REVIEW_ID_DOMAIN, "rrv_", &identity)?;
        let value = Self {
            review_receipt_id,
            subject_blake3,
            proof_receipt_ids,
            recovery_orchestrator,
            reviewer,
            review_evidence_sha256,
            decision: RecoveryReviewDecisionV1::Approve,
            authority_class: RecoveryAdoptionAuthorityClassV1::LocalOsAuthority,
        };
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn review_receipt_id(&self) -> &str {
        &self.review_receipt_id
    }

    pub(in crate::coord) fn subject_blake3(&self) -> &str {
        &self.subject_blake3
    }

    pub(in crate::coord) fn proof_receipt_ids(&self) -> &[String] {
        &self.proof_receipt_ids
    }

    pub(in crate::coord) fn recovery_orchestrator(&self) -> &str {
        &self.recovery_orchestrator
    }

    pub(in crate::coord) fn reviewer(&self) -> &str {
        &self.reviewer
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.review_receipt_id, "rrv_")?;
        tagged(&self.subject_blake3, "blake3:")?;
        tagged(&self.review_evidence_sha256, "sha256:")?;
        validate_field("recovery_orchestrator", &self.recovery_orchestrator)
            .map_err(|error| invalid(error.to_string()))?;
        validate_field("reviewer", &self.reviewer).map_err(|error| invalid(error.to_string()))?;
        if self.proof_receipt_ids.is_empty()
            || self.proof_receipt_ids.len() > MAX_RECOVERY_PROOF_RECEIPTS
            || self
                .proof_receipt_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid(
                "review proof receipt IDs must be bounded and sorted",
            ));
        }
        for receipt_id in &self.proof_receipt_ids {
            tagged(receipt_id, "rpf_")?;
        }
        let expected = id(
            REVIEW_ID_DOMAIN,
            "rrv_",
            &ReviewIdentity {
                subject_blake3: &self.subject_blake3,
                proof_receipt_ids: &self.proof_receipt_ids,
                recovery_orchestrator: &self.recovery_orchestrator,
                reviewer: &self.reviewer,
                review_evidence_sha256: &self.review_evidence_sha256,
            },
        )?;
        if self.recovery_orchestrator == self.reviewer || self.review_receipt_id != expected {
            return Err(invalid("recovery review is not independent and exact"));
        }
        Ok(())
    }
}

fn id(domain: &str, prefix: &str, value: &impl Serialize) -> Result<String, CoordError> {
    Ok(format!(
        "{prefix}{}",
        bullet_wire::hash_canonical(domain, value)
            .map_err(|error| invalid(error.to_string()))?
            .to_hex()
    ))
}

fn safe(value: u64, label: &str) -> Result<(), CoordError> {
    if value > MAX_SAFE_INTEGER {
        Err(invalid(format!("{label} is not a JSON-safe integer")))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "evidence/tests.rs"]
mod tests;
