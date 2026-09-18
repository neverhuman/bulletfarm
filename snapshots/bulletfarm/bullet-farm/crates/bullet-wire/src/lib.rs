mod authority;
mod candidate_preparation;
mod canonical;
mod catalog;
mod contract_bindings;
mod contract_tool;
mod digest;
mod dogfood;
mod error;
mod event;
mod forge_profile;
mod ids;
mod ipc;
mod manifest;
mod outcome;
mod policy;
mod proposal;
mod release;
mod runtime_passport;

pub use authority::{
    AUTHORITY_IMPLICIT_ASSERTION, AUTHORITY_SCHEMA_VERSION, AuthorityAudience, AuthorityClaims,
    AuthorityDecisionKind, AuthorityExpectation, AuthorityRequest, AuthorityRequestBinding,
    AuthoritySigningKey, AuthorityVerificationKey, FinalAuthorityCheckRequest,
    FinalAuthorityDecision, LAUNCH_GRANT_CLAIMS_DOMAIN, LAUNCH_GRANT_ENVELOPE_DOMAIN,
    LAUNCH_GRANT_ENVIRONMENT_DOMAIN, LAUNCH_GRANT_IMPLICIT_ASSERTION, LAUNCH_GRANT_POLICY_DOMAIN,
    LAUNCH_GRANT_SIGNING_PURPOSE, LAUNCH_GRANT_WORKSPACE_NONCE_DOMAIN, LaunchGrantClaims,
    LaunchGrantExpectation, LaunchLeaseSubject, LaunchOperation, LaunchProvider,
    LaunchProviderSubject, MAX_AUTHORITY_TTL_MS, MAX_LAUNCH_GRANT_GATE_IDS,
    MAX_LAUNCH_GRANT_TTL_MS, MAX_MUTATION_PERMIT_TTL_MS, MUTATION_PERMIT_IMPLICIT_ASSERTION,
    MutationOperation, MutationOutcome, MutationPermitClaims, MutationPermitExpectation,
    MutationPermitSubject, MutationReplayResult, MutationResultState, MutationSettlementRequest,
    MutationSettlementResult, PreservationDecision, ReplayDisposition, SettlementStatus,
    SignedAuthorityEnvelope, SignedLaunchGrant, SignedMutationPermit, authority_request_digest,
    environment_digest, policy_snapshot_digest, workspace_nonce_digest,
};
pub use candidate_preparation::{
    CANDIDATE_PREPARATION_CLAIMS_DOMAIN, CANDIDATE_PREPARATION_DIGEST_DOMAIN,
    CANDIDATE_PREPARATION_ENVELOPE_DOMAIN, CANDIDATE_PREPARATION_SIGNING_PURPOSE,
    EXECUTION_ENVELOPE_CLAIMS_DOMAIN, EXECUTION_ENVELOPE_DIGEST_DOMAIN,
    EXECUTION_ENVELOPE_SIGNING_PURPOSE, EXECUTION_TOOLCHAIN_DIGEST_DOMAIN, MAX_SAFE_INTEGER,
    candidate_preparation_digest, decode_candidate_preparation_grant, decode_execution_envelope,
    decode_signed_candidate_preparation_grant, execution_envelope_digest,
    execution_toolchain_digest, validate_candidate_preparation_binding,
    validate_candidate_preparation_grant, validate_execution_envelope,
};
pub use canonical::{
    MAX_CANONICAL_DOCUMENT_BYTES, MAX_UNIQUE_DOCUMENT_BYTES, decode_canonical,
    decode_canonical_value, decode_unique_value, decode_unique_value_bounded,
};
pub use catalog::*;
pub use contract_tool::{ContractMode, execute as execute_contract_tool};
pub use digest::{Blake3Digest, canonical_json, hash_canonical, hash_framed_bytes};
pub use dogfood::grant_signing::{
    DOGFOOD_LAUNCH_GRANT_ENVELOPE_DOMAIN, DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
    DogfoodLaunchSigningKey, DogfoodLaunchVerificationKey, MAX_DOGFOOD_LAUNCH_GRANT_TOKEN_BYTES,
    SignedDogfoodLaunchGrantV1,
};
pub use dogfood::run_signing::{
    DOGFOOD_RUN_ATTESTATION_ENVELOPE_DOMAIN, DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
    DogfoodRunAttestationSigningKey, MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS,
    MAX_DOGFOOD_RUN_ATTESTATION_TOKEN_BYTES, SignedDogfoodRunV1,
};
pub use dogfood::{
    CREDENTIAL_PROJECTION_DIGEST_DOMAIN, DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN,
    DOGFOOD_BUDGET_SETTLEMENT_DIGEST_DOMAIN, DOGFOOD_INTENT_DIGEST_DOMAIN,
    DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN, DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
    DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN, DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN,
    DOGFOOD_RUN_DIGEST_DOMAIN, DOGFOOD_SCHEMA_VERSION, DogfoodArtifactRefV1,
    DogfoodBudgetReservationV1, DogfoodBudgetSettlementV1, DogfoodCleanupObservationV1,
    DogfoodExecutionSubjectV1, DogfoodLaunchGrantClaimsV1, DogfoodPolicySubjectV1,
    DogfoodProcessObservationV1, DogfoodProcessStateV1, DogfoodProposalObservationV1,
    DogfoodProviderSubjectV1, DogfoodReadOnlyIntentV1, DogfoodRepositorySubjectV1,
    DogfoodRunArtifactsV1, DogfoodRunBindingSubjects, DogfoodRunSubjectV1, DogfoodRunV1,
    DogfoodTerminalStateV1, DogfoodUsageSettlementV1, MAX_CREDENTIAL_PROJECTION_TTL_MS,
    MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS, MAX_DOGFOOD_CAPTURE_BYTES, MAX_DOGFOOD_GATE_IDS,
    MAX_DOGFOOD_GRANT_TTL_MS, MAX_DOGFOOD_PATCH_CONTENT_BYTES, MAX_DOGFOOD_PATCH_OPERATIONS,
    MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES, MAX_DOGFOOD_RETAINED_ARTIFACTS,
    MAX_DOGFOOD_RETAINED_BYTES, MAX_DOGFOOD_RUN_BYTES, MAX_PROVIDER_OBSERVATION_BYTES,
    MAX_PROVIDER_OBSERVATION_STALENESS_MS, MAX_REPOSITORY_CONTEXT_FILES,
    MAX_REPOSITORY_CONTEXT_SCOPES, MAX_REPOSITORY_CONTEXT_TOTAL_BYTES,
    PROVIDER_ENDPOINT_OBSERVATION_DIGEST_DOMAIN, PROVIDER_ENROLLMENT_CLAIMS_DOMAIN,
    PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN, PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
    PROVIDER_ENROLLMENT_SIGNING_PURPOSE, PROVIDER_PROBE_OBSERVATION_DIGEST_DOMAIN,
    PROVIDER_PROFILE_OBSERVATION_DIGEST_DOMAIN, PROVIDER_VERSION_OBSERVATION_DIGEST_DOMAIN,
    ProviderCredentialProjectionV1, ProviderEndpointObservationV1, ProviderEnrollmentClaimsV2,
    ProviderEnrollmentExpectationV2, ProviderEnrollmentSigningKey, ProviderObservationSubjectV1,
    ProviderProbeObservationV1, ProviderProfileObservationV1, ProviderVersionObservationV1,
    REPOSITORY_CONTEXT_POST_OBSERVATION_DIGEST_DOMAIN, REPOSITORY_CONTEXT_SNAPSHOT_DIGEST_DOMAIN,
    REPOSITORY_CONTEXT_VISIBLE_MANIFEST_DIGEST_DOMAIN, RepositoryContextPostObservationV1,
    RepositoryContextSnapshotV1, RepositoryVisibleFileV1, SignedProviderEnrollmentV2,
    decode_dogfood_budget_reservation, decode_dogfood_launch_grant_claims,
    decode_dogfood_read_only_intent, decode_dogfood_run, decode_provider_credential_projection,
    decode_provider_endpoint_observation, decode_provider_enrollment_claims,
    decode_provider_probe_observation, decode_provider_profile_observation,
    decode_provider_version_observation, decode_repository_context_post_observation,
    decode_repository_context_snapshot, verify_dogfood_budget_binding, verify_dogfood_run_binding,
    verify_dogfood_runtime_binding, verify_dogfood_subjects, verify_provider_observations,
    verify_repository_context_binding, verify_repository_context_post_observation,
};
pub use error::WireError;
pub use event::{CommandEnvelope, CommandState, EventEnvelope, Snapshot};
pub use forge_profile::{
    FORGE_PROFILE_SCHEMA_VERSION, ForgeCapability, ForgeKind, ForgeProfileId, ForgeProfileRegistry,
    IntegrationSubjectBinding, MAX_FORGE_BASE_URL_BYTES, MAX_REPLICATION_REFS,
    PRIMARY_FORGE_PROFILE_DIGEST_DOMAIN, PrimaryForgeProfileV1, REPLICATION_INTENT_DIGEST_DOMAIN,
    ReplicationIntentKind, ReplicationIntentV1, decode_primary_forge_profile,
    decode_replication_intent,
};
pub use ids::*;
pub use ipc::*;
pub use manifest::{
    CandidateManifest as ComponentCandidateManifest,
    CandidateProofManifest as ComponentCandidateProofManifest,
    CheckpointManifest as ComponentCheckpointManifest,
    IntegrationProofManifest as ComponentIntegrationProofManifest,
    PreservationReceiptManifest as ComponentPreservationReceiptManifest,
};
pub use outcome::*;
pub use policy::*;
pub use proposal::*;
pub use release::{
    RELEASE_BUNDLE_MANIFEST_V2_DIGEST_DOMAIN, RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX,
    RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, RELEASE_GATE_RECEIPT_SIGNATURE_DOMAIN,
    RELEASE_GATE_SPEC_DIGEST_DOMAIN, RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN,
    RELEASE_REGISTRY_MANIFEST_DIGEST_DOMAIN, RELEASE_REGISTRY_MANIFEST_SIGNATURE_DOMAIN,
    RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN, RELEASE_SIGNER_POLICY_DIGEST_DOMAIN,
    RELEASE_SIGNER_POLICY_SIGNATURE_DOMAIN, RELEASE_SOURCE_SUBJECT_DIGEST_DOMAIN,
    RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, RELEASE_TRUSTED_TIME_SIGNATURE_DOMAIN,
    RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, ReleaseWireRecord, decode_release_record,
    release_bundle_manifest_v2_digest, validate_release_bindings,
    validate_release_bundle_manifest_v2_binding,
};
pub use runtime_passport::{
    MAX_RUNTIME_FILE_BYTES, MAX_RUNTIME_FILES, MAX_RUNTIME_PASSPORT_BYTES,
    MAX_RUNTIME_RELATIVE_PATH_BYTES, MAX_RUNTIME_TOTAL_BYTES, MAX_RUNTIME_VERSION_BYTES,
    ProviderRuntimePassportV1, RUNTIME_DEPLOYMENT_PREFIX, RUNTIME_PASSPORT_DOMAIN,
    RUNTIME_PASSPORT_ID_PREFIX, RUNTIME_PASSPORT_SCHEMA_VERSION, RuntimeExecutionV1,
    RuntimeFileRoleV1, RuntimeFileV1, RuntimeLoaderV1, RuntimePassportError,
    decode_expected_runtime_passport,
};

/// Normative generated wire records. Security-sensitive consumers must decode this namespace.
pub mod v1alpha1 {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/generated/rust/schema_bundle.rs"
    ));
}

pub const SCHEMA_VERSION: u32 = 1;
