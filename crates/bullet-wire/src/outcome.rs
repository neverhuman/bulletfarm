use serde::{Deserialize, Serialize};

use crate::{
    AttemptId, Blake3Digest, CandidateId, EffectIntentId, EffectReceiptId, EvidenceId, GateId,
    GitOid, WireError, hash_canonical,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceOutcome {
    Pass,
    Fail,
    NotRun,
    TimedOut,
    Flaky,
    Unsupported,
    InfraError,
    Invalidated,
    Unknown,
}

impl EvidenceOutcome {
    pub const fn satisfies_requirement(self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema_version: u32,
    pub evidence_id: EvidenceId,
    pub candidate_id: CandidateId,
    pub exact_head: GitOid,
    pub exact_tree: GitOid,
    pub gate_id: GateId,
    pub outcome: EvidenceOutcome,
    pub verifier_identity: String,
    pub writer_attempt_id: AttemptId,
    pub verifier_is_independent: bool,
    pub environment_digest: Blake3Digest,
    pub toolchain_digest: Blake3Digest,
    pub proof_bundle_digest: Blake3Digest,
}

impl Evidence {
    pub const fn satisfies_requirement(&self) -> bool {
        self.verifier_is_independent && self.outcome.satisfies_requirement()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectOutcome {
    Proposed,
    Authorized,
    Dispatched,
    Verified,
    Failed,
    Unknown,
    OrphanedRemote,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectIntentManifest {
    pub schema_version: u32,
    pub candidate_id: CandidateId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub provider: String,
    pub logical_effect_key: String,
    pub desired_oid: GitOid,
    pub request_digest: Blake3Digest,
}

impl EffectIntentManifest {
    pub fn intent_id(&self) -> Result<EffectIntentId, WireError> {
        if self.schema_version != crate::SCHEMA_VERSION || self.attempt_fence == 0 {
            return Err(WireError::new(
                "INVALID_EFFECT_INTENT",
                "effect intent requires the current schema and a nonzero fence",
            ));
        }
        hash_canonical("effect.intent", self).map(EffectIntentId::from_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectReceipt {
    pub schema_version: u32,
    pub receipt_id: EffectReceiptId,
    pub intent_id: EffectIntentId,
    pub candidate_id: CandidateId,
    pub desired_oid: GitOid,
    pub provider: String,
    pub logical_effect_key: String,
    pub outcome: EffectOutcome,
    pub remote_receipt_digest: Option<Blake3Digest>,
    pub observed_oid: Option<GitOid>,
    pub adopted_after_unknown: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Observation<T> {
    Value(T),
    Empty,
    Unknown { reason: String },
    Contradictory { values: Vec<T>, reason: String },
}
