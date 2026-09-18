//! Strict unsigned dogfood subjects; no signing, admission, persistence, or launch.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    AttemptId, Blake3Digest, CheckpointId, CommandId, DogfoodAudienceV1,
    DogfoodBudgetReservationId, DogfoodGrantId, DogfoodIntentId, DogfoodOperationV1,
    DogfoodProviderProtocolV1, DogfoodRunId, GateId, GitOid, GraphRevisionId, LaunchProvider,
    MissionId, ProviderCredentialProjectionId, ProviderEnrollmentId, ProviderProfileId,
    RepositoryContextSnapshotId, RepositoryId, RunnerId, RuntimePassportId, VariantId, WireError,
    WorkPackageId, decode_canonical, hash_canonical,
    ids::{is_bounded_wire_label, require_exact_wire},
};

#[cfg(test)]
use crate::PrincipalId;

mod credential_projection;
mod enrollment_signing;
pub(crate) mod grant_signing;
pub(crate) mod run_signing;
mod runtime_binding;
pub use credential_projection::{
    CREDENTIAL_PROJECTION_DIGEST_DOMAIN, MAX_CREDENTIAL_PROJECTION_TTL_MS,
    ProviderCredentialProjectionV1, decode_provider_credential_projection,
};
pub use enrollment_signing::{
    PROVIDER_ENROLLMENT_CLAIMS_DOMAIN, PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN,
    PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION, PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
    ProviderEnrollmentClaimsV2, ProviderEnrollmentExpectationV2, ProviderEnrollmentSigningKey,
    SignedProviderEnrollmentV2, decode_provider_enrollment_claims,
};
pub use runtime_binding::verify_dogfood_runtime_binding;

pub const DOGFOOD_SCHEMA_VERSION: &str = "v1alpha1";
pub const DOGFOOD_INTENT_DIGEST_DOMAIN: &str = "dogfood.read-only-intent.v1alpha1";
pub const DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN: &str =
    "authority.dogfood-launch-grant-claims.v1alpha1";
pub const DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE: &str = "dogfood-launch-signing";
pub const MAX_DOGFOOD_GATE_IDS: usize = 16;
pub const MAX_DOGFOOD_GRANT_TTL_MS: u64 = 15_000;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_LABEL_BYTES: usize = 128;

mod budget;
pub use budget::{
    DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN, DogfoodBudgetReservationV1,
    MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS, decode_dogfood_budget_reservation,
    verify_dogfood_budget_binding,
};
mod context;
pub use context::{
    MAX_REPOSITORY_CONTEXT_FILES, MAX_REPOSITORY_CONTEXT_SCOPES,
    MAX_REPOSITORY_CONTEXT_TOTAL_BYTES, REPOSITORY_CONTEXT_POST_OBSERVATION_DIGEST_DOMAIN,
    REPOSITORY_CONTEXT_SNAPSHOT_DIGEST_DOMAIN, REPOSITORY_CONTEXT_VISIBLE_MANIFEST_DIGEST_DOMAIN,
    RepositoryContextPostObservationV1, RepositoryContextSnapshotV1, RepositoryVisibleFileV1,
    decode_repository_context_post_observation, decode_repository_context_snapshot,
    verify_repository_context_binding, verify_repository_context_post_observation,
};
mod provider_observation;
pub use provider_observation::{
    MAX_PROVIDER_OBSERVATION_BYTES, MAX_PROVIDER_OBSERVATION_STALENESS_MS,
    PROVIDER_ENDPOINT_OBSERVATION_DIGEST_DOMAIN, PROVIDER_PROBE_OBSERVATION_DIGEST_DOMAIN,
    PROVIDER_PROFILE_OBSERVATION_DIGEST_DOMAIN, PROVIDER_VERSION_OBSERVATION_DIGEST_DOMAIN,
    ProviderEndpointObservationV1, ProviderObservationSubjectV1, ProviderProbeObservationV1,
    ProviderProfileObservationV1, ProviderVersionObservationV1,
    decode_provider_endpoint_observation, decode_provider_probe_observation,
    decode_provider_profile_observation, decode_provider_version_observation,
    verify_provider_observations,
};
mod run;
pub use run::*;
mod run_binding;
pub use run_binding::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodExecutionSubjectV1 {
    pub command_id: CommandId,
    pub run_id: DogfoodRunId,
    pub mission_id: MissionId,
    pub repository_id: RepositoryId,
    pub graph_revision_id: GraphRevisionId,
    pub work_package_id: WorkPackageId,
    pub variant_id: VariantId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub runner_id: RunnerId,
    pub runner_epoch: u64,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodProviderSubjectV1 {
    pub provider: LaunchProvider,
    pub protocol: DogfoodProviderProtocolV1,
    pub provider_profile_id: ProviderProfileId,
    pub runtime_passport_id: RuntimePassportId,
    pub provider_enrollment_id: ProviderEnrollmentId,
    pub credential_projection_id: ProviderCredentialProjectionId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodRepositorySubjectV1 {
    pub context_snapshot_id: RepositoryContextSnapshotId,
    pub head_oid: GitOid,
    pub tree_oid: GitOid,
    pub checkpoint_id: CheckpointId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodPolicySubjectV1 {
    pub policy_snapshot_digest: Blake3Digest,
    pub policy_generation: u64,
    pub dogfood_binding_digest: Blake3Digest,
    pub tool_policy_digest: Blake3Digest,
    pub egress_policy_digest: Blake3Digest,
    pub containment_policy_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodRunSubjectV1 {
    pub execution: DogfoodExecutionSubjectV1,
    pub provider: DogfoodProviderSubjectV1,
    pub repository: DogfoodRepositorySubjectV1,
    pub gate_ids: Vec<GateId>,
    pub prompt_digest: Blake3Digest,
    pub policy: DogfoodPolicySubjectV1,
    pub budget_reservation_id: DogfoodBudgetReservationId,
    pub deadline_unix_ms: u64,
    pub output_schema_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodReadOnlyIntentV1 {
    pub schema_version: String,
    pub request_digest: Blake3Digest,
    pub subject: DogfoodRunSubjectV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodLaunchGrantClaimsV1 {
    pub schema_version: String,
    pub audience: DogfoodAudienceV1,
    pub operation: DogfoodOperationV1,
    pub issuer: String,
    pub key_id: String,
    pub signing_purpose: String,
    pub claims_domain: String,
    pub issued_at_unix_ms: u64,
    pub not_before_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub grant_nonce: Blake3Digest,
    pub request_digest: Blake3Digest,
    pub intent_id: DogfoodIntentId,
    pub subject: DogfoodRunSubjectV1,
}

pub fn decode_dogfood_read_only_intent(bytes: &[u8]) -> Result<DogfoodReadOnlyIntentV1, WireError> {
    let intent: DogfoodReadOnlyIntentV1 = decode_canonical(bytes)?;
    intent.validate()?;
    Ok(intent)
}

pub fn decode_dogfood_launch_grant_claims(
    bytes: &[u8],
) -> Result<DogfoodLaunchGrantClaimsV1, WireError> {
    let grant: DogfoodLaunchGrantClaimsV1 = decode_canonical(bytes)?;
    grant.validate()?;
    Ok(grant)
}

impl DogfoodReadOnlyIntentV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        require_exact_wire(
            "schema_version",
            &self.schema_version,
            DOGFOOD_SCHEMA_VERSION,
            "DOGFOOD_INTENT_INVALID",
        )?;
        validate_run_subject(&self.subject, "DOGFOOD_INTENT_INVALID")
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(DOGFOOD_INTENT_DIGEST_DOMAIN, self)
    }

    pub fn intent_id(&self) -> Result<DogfoodIntentId, WireError> {
        self.digest().map(DogfoodIntentId::from_digest)
    }
}

impl DogfoodLaunchGrantClaimsV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "DOGFOOD_GRANT_INVALID";
        require_exact_wire(
            "schema_version",
            &self.schema_version,
            DOGFOOD_SCHEMA_VERSION,
            code,
        )?;
        if self.audience != DogfoodAudienceV1::DogfoodRunner
            || self.operation != DogfoodOperationV1::ReadOnlyPropose
        {
            return Err(invalid(
                code,
                "grant must be dogfood-runner/read-only-propose",
            ));
        }
        require_exact_wire(
            "signing_purpose",
            &self.signing_purpose,
            DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
            code,
        )?;
        require_exact_wire(
            "claims_domain",
            &self.claims_domain,
            DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN,
            code,
        )?;
        for (name, value) in [
            ("issuer", self.issuer.as_str()),
            ("key_id", self.key_id.as_str()),
        ] {
            if !is_bounded_wire_label(value, MAX_LABEL_BYTES) {
                return Err(invalid(
                    code,
                    format!("{name} must be bounded identifier text"),
                ));
            }
        }
        for (name, value) in [
            ("issued_at_unix_ms", self.issued_at_unix_ms),
            ("not_before_unix_ms", self.not_before_unix_ms),
            ("expires_at_unix_ms", self.expires_at_unix_ms),
        ] {
            if value > MAX_SAFE_INTEGER {
                return Err(invalid(
                    code,
                    format!("{name} exceeds the safe integer range"),
                ));
            }
        }
        if self.issued_at_unix_ms > self.not_before_unix_ms
            || self.not_before_unix_ms >= self.expires_at_unix_ms
        {
            return Err(invalid(
                code,
                "grant requires issued_at <= not_before < expires_at",
            ));
        }
        if self.expires_at_unix_ms - self.not_before_unix_ms > MAX_DOGFOOD_GRANT_TTL_MS {
            return Err(invalid(code, "grant window exceeds the 15s maximum"));
        }
        if self.expires_at_unix_ms > self.subject.deadline_unix_ms {
            return Err(invalid(code, "grant expiry exceeds the bound run deadline"));
        }
        validate_run_subject(&self.subject, code)
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN, self)
    }

    pub fn grant_id(&self) -> Result<DogfoodGrantId, WireError> {
        self.digest().map(DogfoodGrantId::from_digest)
    }
}

/// Require the grant's complete run subject and the enrollment claims selected by the intent.
pub fn verify_dogfood_subjects(
    grant: &DogfoodLaunchGrantClaimsV1,
    intent: &DogfoodReadOnlyIntentV1,
    enrollment: &ProviderEnrollmentClaimsV2,
) -> Result<(), WireError> {
    grant.validate()?;
    intent.validate()?;
    enrollment.validate()?;
    let intent_id = intent.intent_id()?;
    if grant.request_digest != intent.request_digest
        || grant.intent_id.as_str() != intent_id.as_str()
        || grant.subject != intent.subject
    {
        return Err(invalid(
            "DOGFOOD_GRANT_SUBJECT_MISMATCH",
            "grant does not bind the exact request, intent, and run subject",
        ));
    }
    let bound = &intent.subject.provider;
    let policy = &intent.subject.policy;
    let enrollment_id = enrollment.enrollment_id()?;
    if bound.provider_enrollment_id.as_str() != enrollment_id.as_str()
        || enrollment.activates_at_unix_ms > grant.not_before_unix_ms
        || enrollment.expires_at_unix_ms < grant.expires_at_unix_ms
        || enrollment
            .revoked_at_unix_ms
            .is_some_and(|revoked| revoked <= grant.expires_at_unix_ms)
        || bound.provider != enrollment.provider
        || bound.protocol != enrollment.protocol
        || bound.provider_profile_id.as_str() != enrollment.provider_profile_id.as_str()
        || bound.runtime_passport_id.as_str() != enrollment.runtime_passport_id.as_str()
        || policy.policy_snapshot_digest != enrollment.policy_snapshot_digest
        || policy.policy_generation != enrollment.policy_generation
        || policy.egress_policy_digest != enrollment.egress_policy_digest
        || policy.tool_policy_digest != enrollment.tool_policy_digest
    {
        return Err(invalid(
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
            "intent does not bind the exact enrollment provider and policy subject",
        ));
    }
    Ok(())
}

fn validate_run_subject(
    subject: &DogfoodRunSubjectV1,
    code: &'static str,
) -> Result<(), WireError> {
    validate_provider_pair(subject.provider.provider, subject.provider.protocol)?;
    for (name, value) in [
        ("attempt_fence", subject.execution.attempt_fence),
        ("runner_epoch", subject.execution.runner_epoch),
        ("authority_epoch", subject.execution.authority_epoch),
        ("freeze_generation", subject.execution.freeze_generation),
        ("policy_generation", subject.policy.policy_generation),
        ("deadline_unix_ms", subject.deadline_unix_ms),
    ] {
        if value == 0 || value > MAX_SAFE_INTEGER {
            return Err(invalid(
                code,
                format!("{name} must be a positive safe integer"),
            ));
        }
    }
    if subject.gate_ids.is_empty() || subject.gate_ids.len() > MAX_DOGFOOD_GATE_IDS {
        return Err(invalid(
            code,
            "gate_ids must contain between 1 and 16 entries",
        ));
    }
    let unique = subject.gate_ids.iter().collect::<BTreeSet<_>>();
    if unique.len() != subject.gate_ids.len() {
        return Err(invalid(code, "gate_ids must be unique"));
    }
    Ok(())
}

fn validate_provider_pair(
    provider: LaunchProvider,
    protocol: DogfoodProviderProtocolV1,
) -> Result<(), WireError> {
    let expected = DogfoodProviderProtocolV1::required_for(provider);
    if protocol != expected {
        return Err(invalid(
            "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH",
            format!(
                "provider {} requires {}, got {}",
                provider.as_str(),
                expected.as_str(),
                protocol.as_str()
            ),
        ));
    }
    Ok(())
}

fn invalid(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}

#[cfg(test)]
mod tests;
