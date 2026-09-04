use serde::{Deserialize, Serialize};

mod api;
pub use api::{
    Applied, COORD_SCHEMA_VERSION, CommandReceipt, GenerationId, GenesisInput, MutationEnvelope,
    RequestId, Status, StatusOrigin, Watermark,
};
mod fresh_genesis;
#[allow(
    unused_imports,
    reason = "COMPONENT_ONLY contracts await their descriptor-bound producers"
)]
pub(crate) use fresh_genesis::{
    FreshGenesisAdmissionReferencesV1, FreshGenesisRecordKindV1, FreshGenesisSealedRecordRefV1,
    IncidentDirectoryIdentityV1, IncidentInventoryNodeTypeV1, IncidentInventoryNodeV1,
    IncidentInventorySubjectV1, IncidentInventoryV1, Wave0ClaimHighWaterV1, Wave0CleanStateV1,
    Wave0FactsV1, Wave0MemberRoleV1, Wave0MemberV1, Wave0ReviewBindingV1, Wave0SubjectV1,
};
mod recovery_adoption;
#[cfg(test)]
pub(crate) use recovery_adoption::fixture_request as recovery_adoption_request_fixture;
pub use recovery_adoption::{
    ForensicRecordRefV1, RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION,
    RecoveryAdoptionAuthorityClassV1, RecoveryAdoptionClaimV1, RecoveryAdoptionRequestKindV1,
    RecoveryAdoptionSummaryV1, RecoveryAdoptionWatermarkV1, RecoveryForensicArtifactKindV1,
    RecoveryForensicRecordKindV1, RecoveryGenerationRecordKindV1, RecoveryGenerationRecordRefV1,
    RecoveryGitExpectationV1, RecoveryGitLeafStatusV1, RecoveryGitLeafTransitionV1,
    RecoveryGitObjectFormatV1, RecoveryProofObservationV1, RecoveryProofRoleV1,
    RecoveryReceiptAdoptionRecordV1, RecoveryReceiptAdoptionRequestV1,
    RecoveryReceiptAdoptionSubjectV1, RecoveryReviewObservationV1, RecoveryReviewRoleV1,
};
pub(crate) use recovery_adoption::{RecoveryProofReceiptRecordV1, RecoveryReviewReceiptRecordV1};
mod recovery_production;
pub(crate) use recovery_production::produced_adoption_request;
pub use recovery_production::{
    RECOVERY_PRODUCTION_SCHEMA_VERSION, RecoveryProductionPlanKindV1, RecoveryProductionPlanV1,
    RecoveryProductionSubjectV1, RecoveryProductionWatermarkV1, RecoveryProofRequestKindV1,
    RecoveryProofRequestV1, RecoveryReviewApprovalKindV1, RecoveryReviewApprovalV1,
    RecoveryReviewDecisionV1, RecoveryReviewRequestKindV1, RecoveryReviewRequestV1,
};
mod recovery_manifest;
#[cfg(test)]
pub(crate) use recovery_manifest::RecoveryBootstrapSourceV1;
#[allow(
    unused_imports,
    reason = "COMPONENT_ONLY build contracts await the sibling artifact verifier"
)]
pub(in crate::coord) use recovery_manifest::{
    CargoOfflineCacheManifestV1, RecoveryBootstrapBuildObservationV1,
    RecoveryBootstrapBuilderContractV1, RecoveryBootstrapCommandContractV1,
    RecoveryBootstrapToolchainContractV1, ToolchainArtifactKindV1, ToolchainMemberV1,
    ToolchainRoleV1,
};
pub(crate) use recovery_manifest::{
    RecoveryAuthorizationDecisionV1, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
    RecoveryBootstrapProvenanceV1, RecoveryFileIdentityV1, RecoveryInspectionArtifactsV1,
    RecoveryInspectionSubjectV1, RecoveryInspectionV1, RecoverySourceInspectionV1,
    validate_linux_boot_id,
};

pub const LEGACY_SCHEMA_VERSION: u32 = 1;
pub const GENERATION_SCHEMA_VERSION: u32 = 2;
pub const DEFAULT_TTL_SECONDS: u64 = 600;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenClaimSubject {
    pub claim_id: String,
    pub claim_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryBaselineBody {
    pub manifest_blake3: String,
    pub incident_at_unix_ms: u64,
    pub recovered_at_unix_ms: u64,
    pub trusted_state_blake3: String,
    pub frozen_claims: Vec<FrozenClaimSubject>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupReceipt {
    pub claim_id: String,
    pub committed_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
pub enum Record {
    GenesisV2 {
        schema_version: u32,
        generation_id: String,
        manifest_blake3: String,
        created_at_unix_ms: u64,
    },
    RecoveryBaselineV2 {
        schema_version: u32,
        generation_id: String,
        body: RecoveryBaselineBody,
    },
    Claim {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        lane: String,
        repo: String,
        paths: Vec<String>,
        expires_unix_ms: u64,
    },
    Heartbeat {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        expires_unix_ms: u64,
        note: Option<String>,
    },
    Handoff {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        agent: String,
        proof_command: String,
        proof_exit_code: i32,
        changed_paths: Vec<String>,
        commit_oid: Option<String>,
    },
    CommitReceipt {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        orchestrator: String,
        commit_oid: String,
        committed_paths: Vec<String>,
    },
    CommitReceiptCorrection {
        schema_version: u32,
        at_unix_ms: u64,
        claim_id: String,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        committed_paths: Vec<String>,
        reason: String,
    },
    CommitReceiptGroup {
        schema_version: u32,
        at_unix_ms: u64,
        orchestrator: String,
        commit_oid: String,
        receipts: Vec<GroupReceipt>,
    },
    CommitReceiptGroupCorrection {
        schema_version: u32,
        at_unix_ms: u64,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        receipts: Vec<GroupReceipt>,
        reason: String,
    },
    RecoveryReceiptAdoptionV1 {
        schema_version: u32,
        at_unix_ms: u64,
        body: RecoveryReceiptAdoptionRecordV1,
    },
    RecoveryProofReceiptV1 {
        schema_version: u32,
        at_unix_ms: u64,
        body: RecoveryProofReceiptRecordV1,
    },
    RecoveryReviewReceiptV1 {
        schema_version: u32,
        at_unix_ms: u64,
        body: RecoveryReviewReceiptRecordV1,
    },
}

impl Record {
    pub const fn schema_version(&self) -> u32 {
        match self {
            Self::GenesisV2 { schema_version, .. }
            | Self::RecoveryBaselineV2 { schema_version, .. }
            | Self::Claim { schema_version, .. }
            | Self::Heartbeat { schema_version, .. }
            | Self::Handoff { schema_version, .. }
            | Self::CommitReceipt { schema_version, .. }
            | Self::CommitReceiptCorrection { schema_version, .. }
            | Self::CommitReceiptGroup { schema_version, .. }
            | Self::CommitReceiptGroupCorrection { schema_version, .. }
            | Self::RecoveryReceiptAdoptionV1 { schema_version, .. }
            | Self::RecoveryProofReceiptV1 { schema_version, .. }
            | Self::RecoveryReviewReceiptV1 { schema_version, .. } => *schema_version,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimSummary {
    pub claim_id: String,
    pub agent: String,
    pub lane: String,
    pub repo: String,
    pub paths: Vec<String>,
    pub claimed_at_unix_ms: u64,
    pub last_event_unix_ms: u64,
    pub expires_unix_ms: u64,
    pub state: ClaimState,
    pub proof_command: Option<String>,
    pub changed_paths: Vec<String>,
    pub commit_oid: Option<String>,
    pub commit_orchestrator: Option<String>,
    pub commit_recorded_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_adoption: Option<RecoveryAdoptionSummaryV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimState {
    Active,
    Expired,
    HandedOff,
    FrozenRecovery,
    RecoveredReceipted,
}

impl ClaimSummary {
    pub fn refresh_state(&mut self, now_unix_ms: u64) {
        if matches!(self.state, ClaimState::Active | ClaimState::Expired) {
            self.state = if self.expires_unix_ms > now_unix_ms {
                ClaimState::Active
            } else {
                ClaimState::Expired
            };
        }
    }
}
