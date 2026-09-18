//! Purpose-separated PASETO v4.public signing for read-only dogfood launch grants.
//!
//! This module authenticates immutable wire subjects. Policy selection, trusted-time
//! custody, nonce replay state, and durable admission remain Kernel responsibilities.

use std::convert::TryFrom;

use pasetors::{
    Public,
    keys::{AsymmetricPublicKey, AsymmetricSecretKey},
    token::UntrustedToken,
    version4::{PublicToken, V4},
};
use serde::{Deserialize, Serialize};

use super::{
    DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE, DOGFOOD_SCHEMA_VERSION, DogfoodLaunchGrantClaimsV1,
    DogfoodReadOnlyIntentV1, ProviderEnrollmentClaimsV2, decode_dogfood_launch_grant_claims,
    verify_dogfood_subjects,
};
use crate::{
    Blake3Digest, WireError, canonical_json, hash_framed_bytes, ids::is_bounded_wire_label,
};

pub const DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION: &[u8] =
    b"bullet-farm.dogfood-launch-grant.v1alpha1";
pub const DOGFOOD_LAUNCH_GRANT_ENVELOPE_DOMAIN: &str =
    "authority.dogfood-launch-grant-envelope.v1alpha1";
pub const MAX_DOGFOOD_LAUNCH_GRANT_TOKEN_BYTES: usize = 32_768;

const MAX_KEY_IDENTITY_BYTES: usize = 128;

/// Crate-internal PASETO carrier shared by fixed-purpose W8/W9/W10 wrappers.
/// It deliberately exposes neither raw material nor a public generic signer.
pub(crate) struct PurposeSeparatedPasetoSigningKey {
    issuer: String,
    key_id: String,
    secret: AsymmetricSecretKey<V4>,
}

impl PurposeSeparatedPasetoSigningKey {
    pub(crate) fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Option<Self> {
        if !valid_key_identity(issuer, key_id)
            || bytes.len() != 64
            || bytes.iter().all(|byte| *byte == 0)
        {
            return None;
        }
        Some(Self {
            issuer: issuer.to_owned(),
            key_id: key_id.to_owned(),
            secret: AsymmetricSecretKey::<V4>::from(bytes).ok()?,
        })
    }

    pub(crate) fn issuer(&self) -> &str {
        &self.issuer
    }

    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    pub(crate) fn sign(
        &self,
        payload: &[u8],
        footer: &[u8],
        implicit_assertion: &[u8],
    ) -> Option<String> {
        PublicToken::sign(
            &self.secret,
            payload,
            Some(footer),
            Some(implicit_assertion),
        )
        .ok()
    }
}

/// Crate-internal verifier paired with [`PurposeSeparatedPasetoSigningKey`].
pub(crate) struct PurposeSeparatedPasetoVerificationKey {
    issuer: String,
    key_id: String,
    public: AsymmetricPublicKey<V4>,
}

impl PurposeSeparatedPasetoVerificationKey {
    pub(crate) fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Option<Self> {
        if !valid_key_identity(issuer, key_id)
            || bytes.len() != 32
            || bytes.iter().all(|byte| *byte == 0)
        {
            return None;
        }
        Some(Self {
            issuer: issuer.to_owned(),
            key_id: key_id.to_owned(),
            public: AsymmetricPublicKey::<V4>::from(bytes).ok()?,
        })
    }

    // W9/W10 consume policy text through this strict parser; W8 only supplies the carrier.
    #[allow(dead_code)]
    pub(crate) fn from_lower_hex(issuer: &str, key_id: &str, raw: &str) -> Option<Self> {
        if raw.len() != 64 {
            return None;
        }
        let nibble = |byte| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        let mut bytes = [0_u8; 32];
        for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
            bytes[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
        }
        Self::from_bytes(issuer, key_id, &bytes)
    }

    pub(crate) fn issuer(&self) -> &str {
        &self.issuer
    }

    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    pub(crate) fn authenticate(
        &self,
        token: &str,
        footer: &[u8],
        implicit_assertion: &[u8],
    ) -> Option<Vec<u8>> {
        let untrusted = UntrustedToken::<Public, V4>::try_from(token).ok()?;
        let trusted = PublicToken::verify(
            &self.public,
            &untrusted,
            Some(footer),
            Some(implicit_assertion),
        )
        .ok()?;
        Some(trusted.payload().as_bytes().to_vec())
    }
}

/// Dedicated signing wrapper; it cannot sign live grants, enrollment, or run receipts.
pub struct DogfoodLaunchSigningKey(PurposeSeparatedPasetoSigningKey);

impl DogfoodLaunchSigningKey {
    pub fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Result<Self, WireError> {
        PurposeSeparatedPasetoSigningKey::from_bytes(issuer, key_id, bytes)
            .map(Self)
            .ok_or_else(|| {
                error(
                    "INVALID_DOGFOOD_GRANT_KEY",
                    "dogfood PASETO signing keys require bounded identity and 64 nonzero bytes",
                )
            })
    }

    pub fn sign(
        &self,
        claims: &DogfoodLaunchGrantClaimsV1,
    ) -> Result<SignedDogfoodLaunchGrantV1, WireError> {
        claims.validate()?;
        if claims.issuer != self.0.issuer() || claims.key_id != self.0.key_id() {
            return Err(error(
                "DOGFOOD_GRANT_KEY_UNKNOWN",
                "dogfood launch claims do not name the signing key",
            ));
        }
        let payload = canonical_json(claims)?;
        let footer = canonical_json(&footer(self.0.issuer(), self.0.key_id()))?;
        let paseto = self
            .0
            .sign(&payload, &footer, DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION)
            .ok_or_else(|| {
                error(
                    "DOGFOOD_GRANT_SIGNING_FAILED",
                    "dogfood launch PASETO signing failed",
                )
            })?;
        Ok(SignedDogfoodLaunchGrantV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            issuer: self.0.issuer().to_owned(),
            key_id: self.0.key_id().to_owned(),
            paseto,
        })
    }
}

/// Dedicated verification wrapper; callers cannot select its PASETO purpose.
pub struct DogfoodLaunchVerificationKey(PurposeSeparatedPasetoVerificationKey);

impl DogfoodLaunchVerificationKey {
    pub fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Result<Self, WireError> {
        PurposeSeparatedPasetoVerificationKey::from_bytes(issuer, key_id, bytes)
            .map(Self)
            .ok_or_else(|| {
                error(
                    "INVALID_DOGFOOD_GRANT_KEY",
                    "dogfood PASETO verification keys require bounded identity and 32 nonzero bytes",
                )
            })
    }
}

/// Strict signed-envelope carrier for one dogfood launch grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedDogfoodLaunchGrantV1 {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

impl SignedDogfoodLaunchGrantV1 {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_envelope(self)?;
        hash_framed_bytes(DOGFOOD_LAUNCH_GRANT_ENVELOPE_DOMAIN, self.paseto.as_bytes())
    }

    /// Verify fixed-purpose authentication, exact intent/enrollment subjects, and
    /// the inclusive-not-before/exclusive-expiry window at a caller-supplied trusted instant.
    pub fn verify(
        &self,
        key: &DogfoodLaunchVerificationKey,
        intent: &DogfoodReadOnlyIntentV1,
        enrollment: &ProviderEnrollmentClaimsV2,
        trusted_now_unix_ms: u64,
    ) -> Result<DogfoodLaunchGrantClaimsV1, WireError> {
        validate_envelope(self)?;
        if self.issuer != key.0.issuer() || self.key_id != key.0.key_id() {
            return Err(error(
                "DOGFOOD_GRANT_KEY_UNKNOWN",
                "dogfood grant envelope does not name the selected verification key",
            ));
        }
        let footer = canonical_json(&footer(key.0.issuer(), key.0.key_id()))?;
        let payload = key
            .0
            .authenticate(
                &self.paseto,
                &footer,
                DOGFOOD_LAUNCH_GRANT_IMPLICIT_ASSERTION,
            )
            .ok_or_else(|| {
                error(
                    "DOGFOOD_GRANT_INVALID",
                    "dogfood grant PASETO framing, signature, footer, or assertion is invalid",
                )
            })?;
        let claims = decode_dogfood_launch_grant_claims(&payload).map_err(|source| {
            error(
                "DOGFOOD_GRANT_INVALID",
                format!("dogfood grant payload is not strict canonical claims: {source}"),
            )
        })?;
        if claims.issuer != key.0.issuer()
            || claims.key_id != key.0.key_id()
            || claims.issuer != self.issuer
            || claims.key_id != self.key_id
        {
            return Err(error(
                "DOGFOOD_GRANT_KEY_UNKNOWN",
                "signed dogfood claims do not name the selected envelope key",
            ));
        }
        verify_dogfood_subjects(&claims, intent, enrollment)?;
        if trusted_now_unix_ms < claims.not_before_unix_ms {
            return Err(error(
                "DOGFOOD_GRANT_NOT_YET_VALID",
                "dogfood launch grant is not valid yet",
            ));
        }
        if trusted_now_unix_ms >= claims.expires_at_unix_ms {
            return Err(error(
                "DOGFOOD_GRANT_EXPIRED",
                "dogfood launch grant has expired",
            ));
        }
        Ok(claims)
    }
}

#[derive(Serialize)]
struct DogfoodLaunchFooter<'a> {
    schema_version: &'static str,
    issuer: &'a str,
    key_id: &'a str,
    purpose: &'static str,
}

fn footer<'a>(issuer: &'a str, key_id: &'a str) -> DogfoodLaunchFooter<'a> {
    DogfoodLaunchFooter {
        schema_version: DOGFOOD_SCHEMA_VERSION,
        issuer,
        key_id,
        purpose: DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
    }
}

fn validate_envelope(envelope: &SignedDogfoodLaunchGrantV1) -> Result<(), WireError> {
    if envelope.schema_version != DOGFOOD_SCHEMA_VERSION {
        return Err(error(
            "DOGFOOD_GRANT_INVALID",
            "dogfood launch grant envelope requires schema v1alpha1",
        ));
    }
    if !valid_key_identity(&envelope.issuer, &envelope.key_id) {
        return Err(error(
            "DOGFOOD_GRANT_INVALID",
            "dogfood launch grant envelope requires bounded issuer and key identity",
        ));
    }
    if !envelope.paseto.starts_with("v4.public.")
        || envelope.paseto.len() > MAX_DOGFOOD_LAUNCH_GRANT_TOKEN_BYTES
    {
        return Err(error(
            "DOGFOOD_GRANT_INVALID",
            "dogfood launch grant must be a bounded compact PASETO v4.public token",
        ));
    }
    Ok(())
}

pub(crate) fn valid_key_identity(issuer: &str, key_id: &str) -> bool {
    is_bounded_wire_label(issuer, MAX_KEY_IDENTITY_BYTES)
        && is_bounded_wire_label(key_id, MAX_KEY_IDENTITY_BYTES)
}

fn error(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}

#[cfg(test)]
mod test_support {
    use super::*;
    use crate::{
        AttemptId, CheckpointId, CommandId, CredentialProjectionProfileId, DogfoodAudienceV1,
        DogfoodBudgetReservationId, DogfoodExecutionSubjectV1, DogfoodOperationV1,
        DogfoodPolicySubjectV1, DogfoodProviderProtocolV1, DogfoodProviderSubjectV1,
        DogfoodRepositorySubjectV1, DogfoodRunId, DogfoodRunSubjectV1, GateId, GitOid,
        GraphRevisionId, LaunchProvider, MissionId, PrincipalId, ProviderCredentialProjectionId,
        ProviderProfileId, RepositoryContextSnapshotId, RepositoryId, RunnerId, RuntimePassportId,
        VariantId, WorkPackageId,
    };

    pub(super) const NOT_BEFORE: u64 = 2_000;
    pub(super) const EXPIRES_AT: u64 = NOT_BEFORE + 15_000;
    pub(super) const ISSUER: &str = "kernel.example";
    pub(super) const KEY_ID: &str = "dogfood-launch-alpha";

    fn digest(seed: u8) -> Blake3Digest {
        Blake3Digest::from_bytes([seed; 32])
    }

    fn enrollment(provider: LaunchProvider) -> ProviderEnrollmentClaimsV2 {
        ProviderEnrollmentClaimsV2 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            issuer: "operator.example".to_owned(),
            key_id: "provider-enrollment-alpha".to_owned(),
            signing_purpose: super::super::PROVIDER_ENROLLMENT_SIGNING_PURPOSE.to_owned(),
            claims_domain: super::super::PROVIDER_ENROLLMENT_CLAIMS_DOMAIN.to_owned(),
            provider,
            protocol: DogfoodProviderProtocolV1::required_for(provider),
            runtime_passport_id: RuntimePassportId::from_digest(digest(1)),
            provider_profile_id: ProviderProfileId::from_digest(digest(2)),
            service_identity_id: PrincipalId::from_digest(digest(3)),
            credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
            runtime_version: "v1.2.3".to_owned(),
            enrollment_generation: 2,
            activates_at_unix_ms: NOT_BEFORE,
            expires_at_unix_ms: EXPIRES_AT + 1_000,
            revoked_at_unix_ms: None,
            egress_policy_digest: digest(5),
            tool_policy_digest: digest(6),
            budget_policy_digest: digest(7),
            endpoint_observation_digest: digest(8),
            version_observation_digest: digest(9),
            profile_observation_digest: digest(10),
            policy_snapshot_digest: digest(11),
            policy_generation: 2,
        }
    }

    fn intent(enrollment: &ProviderEnrollmentClaimsV2) -> DogfoodReadOnlyIntentV1 {
        DogfoodReadOnlyIntentV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            request_digest: digest(12),
            subject: DogfoodRunSubjectV1 {
                execution: DogfoodExecutionSubjectV1 {
                    command_id: CommandId::from_digest(digest(13)),
                    run_id: DogfoodRunId::from_digest(digest(14)),
                    mission_id: MissionId::from_digest(digest(15)),
                    repository_id: RepositoryId::from_digest(digest(16)),
                    graph_revision_id: GraphRevisionId::from_digest(digest(17)),
                    work_package_id: WorkPackageId::from_digest(digest(18)),
                    variant_id: VariantId::from_digest(digest(19)),
                    attempt_id: AttemptId::from_digest(digest(20)),
                    attempt_fence: 3,
                    runner_id: RunnerId::from_digest(digest(21)),
                    runner_epoch: 4,
                    authority_epoch: 5,
                    freeze_generation: 6,
                },
                provider: DogfoodProviderSubjectV1 {
                    provider: enrollment.provider,
                    protocol: enrollment.protocol,
                    provider_profile_id: enrollment.provider_profile_id.clone(),
                    runtime_passport_id: enrollment.runtime_passport_id.clone(),
                    provider_enrollment_id: enrollment.enrollment_id().unwrap(),
                    credential_projection_id: ProviderCredentialProjectionId::from_digest(digest(
                        22,
                    )),
                },
                repository: DogfoodRepositorySubjectV1 {
                    context_snapshot_id: RepositoryContextSnapshotId::from_digest(digest(23)),
                    head_oid: GitOid::Sha256(format!("{:02x}", 24).repeat(32)),
                    tree_oid: GitOid::Sha256(format!("{:02x}", 25).repeat(32)),
                    checkpoint_id: CheckpointId::from_digest(digest(26)),
                },
                gate_ids: vec![GateId::from_digest(digest(27))],
                prompt_digest: digest(28),
                policy: DogfoodPolicySubjectV1 {
                    policy_snapshot_digest: enrollment.policy_snapshot_digest,
                    policy_generation: enrollment.policy_generation,
                    dogfood_binding_digest: digest(29),
                    tool_policy_digest: enrollment.tool_policy_digest,
                    egress_policy_digest: enrollment.egress_policy_digest,
                    containment_policy_digest: digest(30),
                },
                budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(31)),
                deadline_unix_ms: EXPIRES_AT,
                output_schema_digest: digest(32),
            },
        }
    }

    fn claims(intent: &DogfoodReadOnlyIntentV1) -> DogfoodLaunchGrantClaimsV1 {
        DogfoodLaunchGrantClaimsV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            audience: DogfoodAudienceV1::DogfoodRunner,
            operation: DogfoodOperationV1::ReadOnlyPropose,
            issuer: ISSUER.to_owned(),
            key_id: KEY_ID.to_owned(),
            signing_purpose: DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE.to_owned(),
            claims_domain: super::super::DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN.to_owned(),
            issued_at_unix_ms: NOT_BEFORE,
            not_before_unix_ms: NOT_BEFORE,
            expires_at_unix_ms: EXPIRES_AT,
            grant_nonce: digest(33),
            request_digest: intent.request_digest,
            intent_id: intent.intent_id().unwrap(),
            subject: intent.subject.clone(),
        }
    }

    pub(super) fn providers() -> [(LaunchProvider, DogfoodProviderProtocolV1); 4] {
        use DogfoodProviderProtocolV1 as P;
        use LaunchProvider as L;
        [
            (L::Claude, P::ClaudeStreamJson),
            (L::Codex, P::CodexAppServerJsonl),
            (L::Cursor, P::CursorAcp),
            (L::Agy, P::AntigravityHeadlessStructured),
        ]
    }

    pub(super) fn fixture(
        provider: LaunchProvider,
    ) -> (
        ProviderEnrollmentClaimsV2,
        DogfoodReadOnlyIntentV1,
        DogfoodLaunchGrantClaimsV1,
    ) {
        let enrollment = enrollment(provider);
        let intent = intent(&enrollment);
        let claims = claims(&intent);
        (enrollment, intent, claims)
    }
}

#[cfg(test)]
mod tests;
