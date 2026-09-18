use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, RequestId};

pub const RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION: u32 = 1;
const MAX_RECOVERY_PROOF_RECEIPTS: usize = 64;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RecoveryAdoptionRequestKindV1 {
    #[serde(rename = "recovery_receipt_adoption_request_v1")]
    RecoveryReceiptAdoptionRequestV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReceiptAdoptionRequestV1 {
    pub kind: RecoveryAdoptionRequestKindV1,
    pub schema_version: u32,
    pub request_id: RequestId,
    pub expected_watermark: RecoveryAdoptionWatermarkV1,
    pub subject: RecoveryReceiptAdoptionSubjectV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryAdoptionWatermarkV1 {
    pub generation_id: String,
    pub manifest_blake3: String,
    pub last_sequence: u64,
    pub next_sequence: u64,
    pub head_envelope_blake3: String,
    pub last_record_blake3: String,
    pub last_request_id: RequestId,
    pub last_request_blake3: String,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryForensicArtifactKindV1 {
    TrustedPrefix,
    FrozenLiveSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryForensicRecordKindV1 {
    Claim,
    Handoff,
    CommitReceipt,
    CommitReceiptGroup,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForensicRecordRefV1 {
    pub artifact_kind: RecoveryForensicArtifactKindV1,
    pub artifact_sha256: String,
    pub record_index: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub record_sha256: String,
    pub expected_record_kind: RecoveryForensicRecordKindV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryGenerationRecordKindV1 {
    ProofReceipt,
    ReviewReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryGenerationRecordRefV1 {
    pub generation_id: String,
    pub sequence: u64,
    pub request_id: RequestId,
    pub request_blake3: String,
    pub record_blake3: String,
    pub envelope_blake3: String,
    pub byte_offset: u64,
    pub frame_length: u64,
    pub expected_record_kind: RecoveryGenerationRecordKindV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryProofRoleV1 {
    RecoveryProof,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryReviewRoleV1 {
    IndependentReview,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryProofObservationV1 {
    pub record: RecoveryGenerationRecordRefV1,
    pub expected_subject_blake3: String,
    pub expected_role: RecoveryProofRoleV1,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReviewObservationV1 {
    pub record: RecoveryGenerationRecordRefV1,
    pub expected_subject_blake3: String,
    pub expected_role: RecoveryReviewRoleV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryAdoptionClaimV1 {
    pub claim_id: String,
    pub frozen_claim_blake3: String,
    pub trusted_claim_record: ForensicRecordRefV1,
    pub committed_paths: Vec<String>,
    pub handoff_observation: ForensicRecordRefV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RecoveryGitObjectFormatV1 {
    Sha1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RecoveryGitLeafStatusV1 {
    Modified,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryGitLeafTransitionV1 {
    pub status: RecoveryGitLeafStatusV1,
    pub path: String,
    pub old_mode: String,
    pub new_mode: String,
    pub old_blob_oid: String,
    pub new_blob_oid: String,
    pub old_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
    pub old_sha256: String,
    pub new_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryGitExpectationV1 {
    pub object_format: RecoveryGitObjectFormatV1,
    pub commit_oid: String,
    pub raw_commit_bytes: Vec<u8>,
    pub raw_commit_sha256: String,
    pub parent_oid: String,
    pub parent_tree_oid: String,
    pub parent_receipt_observation: ForensicRecordRefV1,
    pub result_tree_oid: String,
    pub raw_tree_sha256: String,
    pub leaf_transitions: Vec<RecoveryGitLeafTransitionV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReceiptAdoptionSubjectV1 {
    pub repo: String,
    pub git_expectation: RecoveryGitExpectationV1,
    pub claims: Vec<RecoveryAdoptionClaimV1>,
    pub group_receipt_observation: ForensicRecordRefV1,
    pub proof_observations: Vec<RecoveryProofObservationV1>,
    pub review_observation: RecoveryReviewObservationV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryAdoptionAuthorityClassV1 {
    LocalOsAuthority,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryReceiptAdoptionRecordV1 {
    adoption_id: String,
    request_subject_blake3: String,
    request: RecoveryReceiptAdoptionRequestV1,
    recovery_operator: String,
    recovery_policy_sha256: String,
    operator_decision_sha256: String,
    replay_contract_version: u32,
    replay_contract_sha256: String,
    authority_class: RecoveryAdoptionAuthorityClassV1,
    verified_orchestrator: String,
    verified_reviewer: String,
    proof_subject_blake3: String,
    review_subject_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryAdoptionSummaryV1 {
    pub adoption_id: String,
    pub generation_id: String,
    pub request_id: String,
    pub request_subject_blake3: String,
    pub commit_oid: String,
    pub tree_oid: String,
    pub adopted_at_unix_ms: u64,
    pub proof_subject_blake3: String,
    pub review_subject_blake3: String,
    pub authority_class: RecoveryAdoptionAuthorityClassV1,
}

#[path = "recovery_adoption/evidence.rs"]
mod evidence;
pub(crate) use evidence::{RecoveryProofReceiptRecordV1, RecoveryReviewReceiptRecordV1};

#[path = "recovery_adoption/validate.rs"]
mod validate;
pub(super) use validate::validate_base_fields;

#[cfg(test)]
#[path = "recovery_adoption/tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) use tests::fixture_request;
