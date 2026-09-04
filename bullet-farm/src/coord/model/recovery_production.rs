use serde::{Deserialize, Serialize};

use super::{
    ForensicRecordRefV1, RecoveryAdoptionClaimV1, RecoveryAdoptionWatermarkV1,
    RecoveryGenerationRecordRefV1, RecoveryGitExpectationV1, RecoveryProofObservationV1,
    RecoveryProofRoleV1, RecoveryReceiptAdoptionRequestV1, RecoveryReceiptAdoptionSubjectV1,
    RecoveryReviewObservationV1, RecoveryReviewRoleV1, RequestId,
    recovery_adoption::validate_base_fields,
};
use crate::coord::{CoordError, validate_field};

pub const RECOVERY_PRODUCTION_SCHEMA_VERSION: u32 = 1;
const PLAN_DOMAIN: &str = "bullet-family.coord.recovery-production-plan.v1";
const PROOF_REQUEST_DOMAIN: &str = "bullet-family.coord.recovery-proof-request.v1";
const REVIEW_REQUEST_DOMAIN: &str = "bullet-family.coord.recovery-review-request.v1";
const ADOPTION_REQUEST_DOMAIN: &str = "bullet-family.coord.recovery-produced-adoption-request.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RecoveryProductionPlanKindV1 {
    #[serde(rename = "recovery_production_plan_v1")]
    RecoveryProductionPlanV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RecoveryProofRequestKindV1 {
    #[serde(rename = "recovery_proof_request_v1")]
    RecoveryProofRequestV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RecoveryReviewApprovalKindV1 {
    #[serde(rename = "recovery_review_approval_v1")]
    RecoveryReviewApprovalV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RecoveryReviewRequestKindV1 {
    #[serde(rename = "recovery_review_request_v1")]
    RecoveryReviewRequestV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryReviewDecisionV1 {
    Approve,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProductionSubjectV1 {
    pub repo: String,
    pub git_expectation: RecoveryGitExpectationV1,
    pub claims: Vec<RecoveryAdoptionClaimV1>,
    pub group_receipt_observation: ForensicRecordRefV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProductionWatermarkV1 {
    pub generation_id: String,
    pub manifest_blake3: String,
    pub last_sequence: u64,
    pub next_sequence: u64,
    pub head_envelope_blake3: String,
    pub last_record_blake3: String,
    pub last_request_id: String,
    pub last_request_blake3: String,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProductionPlanV1 {
    pub kind: RecoveryProductionPlanKindV1,
    pub schema_version: u32,
    pub plan_id: String,
    pub evidence_subject_blake3: String,
    pub expected_watermark: RecoveryProductionWatermarkV1,
    pub recovery_orchestrator: String,
    pub subject: RecoveryProductionSubjectV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProofRequestV1 {
    pub kind: RecoveryProofRequestKindV1,
    pub schema_version: u32,
    pub request_id: RequestId,
    pub plan: RecoveryProductionPlanV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReviewApprovalV1 {
    pub kind: RecoveryReviewApprovalKindV1,
    pub schema_version: u32,
    pub plan_id: String,
    pub evidence_subject_blake3: String,
    pub proof_receipt_ids: Vec<String>,
    pub reviewer: String,
    pub decision: RecoveryReviewDecisionV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReviewRequestV1 {
    pub kind: RecoveryReviewRequestKindV1,
    pub schema_version: u32,
    pub request_id: RequestId,
    pub plan: RecoveryProductionPlanV1,
    pub approval: RecoveryReviewApprovalV1,
    pub approval_sha256: String,
}

#[derive(Serialize)]
struct PlanIdentity<'a> {
    evidence_subject_blake3: &'a str,
    expected_watermark: &'a RecoveryProductionWatermarkV1,
    recovery_orchestrator: &'a str,
    subject: &'a RecoveryProductionSubjectV1,
}

#[derive(Serialize)]
struct RequestIdentity<'a> {
    evidence_subject_blake3: &'a str,
}

#[derive(Serialize)]
struct AdoptionRequestIdentity<'a> {
    plan_id: &'a str,
    expected_watermark: &'a RecoveryAdoptionWatermarkV1,
    proof: &'a RecoveryGenerationRecordRefV1,
    review: &'a RecoveryGenerationRecordRefV1,
}

impl RecoveryProductionPlanV1 {
    pub(crate) fn derive(
        evidence_subject_blake3: String,
        expected_watermark: RecoveryProductionWatermarkV1,
        recovery_orchestrator: String,
        subject: RecoveryProductionSubjectV1,
    ) -> Result<Self, CoordError> {
        let plan_id = tagged_digest(
            "rcp_",
            PLAN_DOMAIN,
            &PlanIdentity {
                evidence_subject_blake3: &evidence_subject_blake3,
                expected_watermark: &expected_watermark,
                recovery_orchestrator: &recovery_orchestrator,
                subject: &subject,
            },
        )?;
        let value = Self {
            kind: RecoveryProductionPlanKindV1::RecoveryProductionPlanV1,
            schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
            plan_id,
            evidence_subject_blake3,
            expected_watermark,
            recovery_orchestrator,
            subject,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CoordError> {
        require_schema(self.schema_version)?;
        tagged(&self.plan_id, "rcp_")?;
        tagged(&self.evidence_subject_blake3, "blake3:")?;
        self.expected_watermark.validate()?;
        validate_field("recovery_orchestrator", &self.recovery_orchestrator).map_err(as_invalid)?;
        validate_base_fields(
            &self.subject.repo,
            &self.subject.git_expectation,
            &self.subject.claims,
            &self.subject.group_receipt_observation,
        )?;
        let expected = tagged_digest(
            "rcp_",
            PLAN_DOMAIN,
            &PlanIdentity {
                evidence_subject_blake3: &self.evidence_subject_blake3,
                expected_watermark: &self.expected_watermark,
                recovery_orchestrator: &self.recovery_orchestrator,
                subject: &self.subject,
            },
        )?;
        if self.plan_id != expected {
            return Err(invalid("recovery production plan identity differs"));
        }
        canonical_bound(self)
    }
}

impl RecoveryProductionWatermarkV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.generation_id, "gen_")?;
        tagged(&self.manifest_blake3, "blake3:")?;
        tagged(&self.head_envelope_blake3, "")?;
        tagged(&self.last_record_blake3, "")?;
        tagged(&self.last_request_blake3, "")?;
        if self.last_sequence == 0
            || self.next_sequence != self.last_sequence.saturating_add(1)
            || self.byte_length == 0
            || self.last_sequence > 9_007_199_254_740_991
            || self.next_sequence > 9_007_199_254_740_991
            || self.byte_length > 9_007_199_254_740_991
        {
            return Err(invalid(
                "recovery producer watermark is internally inconsistent",
            ));
        }
        if self.last_sequence == 1 {
            tagged(&self.last_request_id, "recovery_")?;
        } else {
            RequestId::parse(self.last_request_id.clone())?;
        }
        Ok(())
    }
}

impl RecoveryProofRequestV1 {
    pub(crate) fn for_plan(plan: RecoveryProductionPlanV1) -> Result<Self, CoordError> {
        plan.validate()?;
        let request_id = request_id(PROOF_REQUEST_DOMAIN, &plan.evidence_subject_blake3)?;
        let value = Self {
            kind: RecoveryProofRequestKindV1::RecoveryProofRequestV1,
            schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
            request_id,
            plan,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CoordError> {
        require_schema(self.schema_version)?;
        self.plan.validate()?;
        self.request_id.validate()?;
        if self.request_id != request_id(PROOF_REQUEST_DOMAIN, &self.plan.evidence_subject_blake3)?
        {
            return Err(invalid("recovery proof request identity differs"));
        }
        canonical_bound(self)
    }
}

impl RecoveryReviewApprovalV1 {
    pub fn validate(&self) -> Result<(), CoordError> {
        require_schema(self.schema_version)?;
        tagged(&self.plan_id, "rcp_")?;
        tagged(&self.evidence_subject_blake3, "blake3:")?;
        validate_field("reviewer", &self.reviewer).map_err(as_invalid)?;
        if self.proof_receipt_ids.is_empty()
            || self.proof_receipt_ids.len() > 64
            || self
                .proof_receipt_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid(
                "review approval proof receipt IDs must be bounded, sorted, and unique",
            ));
        }
        for receipt in &self.proof_receipt_ids {
            tagged(receipt, "rpf_")?;
        }
        canonical_bound(self)
    }
}

impl RecoveryReviewRequestV1 {
    pub(crate) fn from_approval(
        plan: RecoveryProductionPlanV1,
        approval: RecoveryReviewApprovalV1,
        approval_sha256: String,
    ) -> Result<Self, CoordError> {
        let request_id = request_id(REVIEW_REQUEST_DOMAIN, &plan.evidence_subject_blake3)?;
        let value = Self {
            kind: RecoveryReviewRequestKindV1::RecoveryReviewRequestV1,
            schema_version: RECOVERY_PRODUCTION_SCHEMA_VERSION,
            request_id,
            plan,
            approval,
            approval_sha256,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), CoordError> {
        require_schema(self.schema_version)?;
        self.plan.validate()?;
        self.approval.validate()?;
        self.request_id.validate()?;
        tagged(&self.approval_sha256, "sha256:")?;
        if self.approval.plan_id != self.plan.plan_id
            || self.approval.evidence_subject_blake3 != self.plan.evidence_subject_blake3
            || self.approval.reviewer == self.plan.recovery_orchestrator
            || self.request_id
                != request_id(REVIEW_REQUEST_DOMAIN, &self.plan.evidence_subject_blake3)?
        {
            return Err(invalid(
                "recovery review request does not bind one independent exact plan",
            ));
        }
        canonical_bound(self)
    }
}

pub(crate) fn produced_adoption_request(
    plan: &RecoveryProductionPlanV1,
    expected_watermark: RecoveryAdoptionWatermarkV1,
    proof: RecoveryGenerationRecordRefV1,
    review: RecoveryGenerationRecordRefV1,
) -> Result<RecoveryReceiptAdoptionRequestV1, CoordError> {
    plan.validate()?;
    let request_id = RequestId::parse(format!(
        "req_{}",
        bullet_wire::hash_canonical(
            ADOPTION_REQUEST_DOMAIN,
            &AdoptionRequestIdentity {
                plan_id: &plan.plan_id,
                expected_watermark: &expected_watermark,
                proof: &proof,
                review: &review,
            },
        )
        .map_err(wire)?
        .to_hex()
    ))?;
    let request = RecoveryReceiptAdoptionRequestV1 {
        kind: super::RecoveryAdoptionRequestKindV1::RecoveryReceiptAdoptionRequestV1,
        schema_version: super::RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION,
        request_id,
        expected_watermark,
        subject: RecoveryReceiptAdoptionSubjectV1 {
            repo: plan.subject.repo.clone(),
            git_expectation: plan.subject.git_expectation.clone(),
            claims: plan.subject.claims.clone(),
            group_receipt_observation: plan.subject.group_receipt_observation.clone(),
            proof_observations: vec![RecoveryProofObservationV1 {
                record: proof,
                expected_subject_blake3: plan.evidence_subject_blake3.clone(),
                expected_role: RecoveryProofRoleV1::RecoveryProof,
            }],
            review_observation: RecoveryReviewObservationV1 {
                record: review,
                expected_subject_blake3: plan.evidence_subject_blake3.clone(),
                expected_role: RecoveryReviewRoleV1::IndependentReview,
            },
        },
    };
    request.validate()?;
    Ok(request)
}

fn request_id(domain: &str, evidence_subject_blake3: &str) -> Result<RequestId, CoordError> {
    let digest = bullet_wire::hash_canonical(
        domain,
        &RequestIdentity {
            evidence_subject_blake3,
        },
    )
    .map_err(wire)?;
    RequestId::parse(format!("req_{}", digest.to_hex()))
}

fn tagged_digest(prefix: &str, domain: &str, value: &impl Serialize) -> Result<String, CoordError> {
    Ok(format!(
        "{prefix}{}",
        bullet_wire::hash_canonical(domain, value)
            .map_err(wire)?
            .to_hex()
    ))
}

fn require_schema(value: u32) -> Result<(), CoordError> {
    if value == RECOVERY_PRODUCTION_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(invalid("recovery production schema version is unsupported"))
    }
}

fn tagged(value: &str, prefix: &str) -> Result<(), CoordError> {
    let valid = value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    valid.then_some(()).ok_or_else(|| {
        invalid(format!(
            "identity must be {prefix} plus 64 lowercase hex digits"
        ))
    })
}

fn canonical_bound(value: &impl Serialize) -> Result<(), CoordError> {
    if bullet_wire::canonical_json(value).map_err(wire)?.len()
        > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES
    {
        Err(invalid(
            "canonical recovery production document exceeds its wire bound",
        ))
    } else {
        Ok(())
    }
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!(
        "canonical recovery production document failed: {error}"
    ))
}

fn as_invalid(error: CoordError) -> CoordError {
    invalid(error.to_string())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

#[cfg(test)]
#[path = "recovery_production/tests.rs"]
mod tests;
