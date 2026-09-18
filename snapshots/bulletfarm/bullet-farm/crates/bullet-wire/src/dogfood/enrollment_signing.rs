//! Purpose-separated PASETO v4.public signing for provider enrollments.
//!
//! This component authenticates an enrollment against one current policy snapshot
//! and caller-supplied time. Policy custody, trusted time, high-water state, and
//! durable admission remain consumer responsibilities.

use serde::{Deserialize, Serialize};

use super::{
    DOGFOOD_SCHEMA_VERSION,
    grant_signing::{
        PurposeSeparatedPasetoSigningKey, PurposeSeparatedPasetoVerificationKey, valid_key_identity,
    },
};
use crate::{
    Blake3Digest, CredentialProjectionProfileId, DogfoodProviderProtocolV1, LaunchProvider,
    PolicySnapshotV1, PrincipalId, ProviderEnrollmentId, ProviderProfileId, RuntimePassportId,
    WireError, canonical_json, decode_canonical, hash_canonical, hash_framed_bytes,
    ids::{is_bounded_wire_label, require_exact_wire},
    policy_snapshot_digest,
};

pub const PROVIDER_ENROLLMENT_CLAIMS_DOMAIN: &str = "provider.enrollment-claims.v2";
pub const PROVIDER_ENROLLMENT_SIGNING_PURPOSE: &str = "provider-enrollment-signing";
pub const PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION: &[u8] = b"bullet-farm.provider-enrollment.v2";
pub const PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN: &str = "authority.provider-enrollment-envelope.v2";

const MAX_PROVIDER_ENROLLMENT_TOKEN_BYTES: usize = 32_768;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_LABEL_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderEnrollmentClaimsV2 {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub signing_purpose: String,
    pub claims_domain: String,
    pub provider: LaunchProvider,
    pub protocol: DogfoodProviderProtocolV1,
    pub runtime_passport_id: RuntimePassportId,
    pub provider_profile_id: ProviderProfileId,
    pub service_identity_id: PrincipalId,
    pub credential_projection_profile_id: CredentialProjectionProfileId,
    pub runtime_version: String,
    pub enrollment_generation: u64,
    pub activates_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub revoked_at_unix_ms: Option<u64>,
    pub egress_policy_digest: Blake3Digest,
    pub tool_policy_digest: Blake3Digest,
    pub budget_policy_digest: Blake3Digest,
    pub endpoint_observation_digest: Blake3Digest,
    pub version_observation_digest: Blake3Digest,
    pub profile_observation_digest: Blake3Digest,
    pub policy_snapshot_digest: Blake3Digest,
    pub policy_generation: u64,
}

pub fn decode_provider_enrollment_claims(
    bytes: &[u8],
) -> Result<ProviderEnrollmentClaimsV2, WireError> {
    let enrollment: ProviderEnrollmentClaimsV2 = decode_canonical(bytes)?;
    enrollment.validate()?;
    Ok(enrollment)
}

impl ProviderEnrollmentClaimsV2 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "PROVIDER_ENROLLMENT_INVALID";
        require_exact_wire(
            "schema_version",
            &self.schema_version,
            DOGFOOD_SCHEMA_VERSION,
            code,
        )?;
        require_exact_wire(
            "signing_purpose",
            &self.signing_purpose,
            PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
            code,
        )?;
        require_exact_wire(
            "claims_domain",
            &self.claims_domain,
            PROVIDER_ENROLLMENT_CLAIMS_DOMAIN,
            code,
        )?;
        for (name, value) in [
            ("issuer", self.issuer.as_str()),
            ("key_id", self.key_id.as_str()),
            ("runtime_version", self.runtime_version.as_str()),
        ] {
            if !is_bounded_wire_label(value, MAX_LABEL_BYTES) {
                return Err(error(
                    code,
                    format!("{name} must be bounded identifier text"),
                ));
            }
        }
        validate_provider_pair(self.provider, self.protocol)?;
        for (name, value) in [
            ("enrollment_generation", self.enrollment_generation),
            ("policy_generation", self.policy_generation),
        ] {
            if value == 0 || value > MAX_SAFE_INTEGER {
                return Err(error(
                    code,
                    format!("{name} must be a positive safe integer"),
                ));
            }
        }
        for (name, value) in [
            ("activates_at_unix_ms", self.activates_at_unix_ms),
            ("expires_at_unix_ms", self.expires_at_unix_ms),
        ] {
            if value > MAX_SAFE_INTEGER {
                return Err(error(
                    code,
                    format!("{name} exceeds the safe integer range"),
                ));
            }
        }
        if self.activates_at_unix_ms >= self.expires_at_unix_ms {
            return Err(error(code, "enrollment activation must precede expiry"));
        }
        if let Some(revoked) = self.revoked_at_unix_ms {
            if revoked > MAX_SAFE_INTEGER {
                return Err(error(
                    code,
                    "revoked_at_unix_ms exceeds the safe integer range",
                ));
            }
            if revoked < self.activates_at_unix_ms || revoked > self.expires_at_unix_ms {
                return Err(error(
                    code,
                    "enrollment revocation is outside its validity window",
                ));
            }
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(PROVIDER_ENROLLMENT_CLAIMS_DOMAIN, self)
    }

    pub fn enrollment_id(&self) -> Result<ProviderEnrollmentId, WireError> {
        self.digest().map(ProviderEnrollmentId::from_digest)
    }
}

/// Exact durable subjects the enrollment consumer must already have selected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderEnrollmentExpectationV2 {
    pub provider_enrollment_id: ProviderEnrollmentId,
    pub enrollment_generation: u64,
    pub policy_snapshot_digest: Blake3Digest,
    pub policy_generation: u64,
}

/// Dedicated signing wrapper; it cannot sign launch grants or terminal runs.
pub struct ProviderEnrollmentSigningKey(PurposeSeparatedPasetoSigningKey);

impl ProviderEnrollmentSigningKey {
    pub fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Result<Self, WireError> {
        PurposeSeparatedPasetoSigningKey::from_bytes(issuer, key_id, bytes)
            .map(Self)
            .ok_or_else(|| {
                error(
                    "INVALID_PROVIDER_ENROLLMENT_KEY",
                    "provider enrollment signing keys require bounded identity and 64 nonzero bytes",
                )
            })
    }

    pub fn sign(
        &self,
        claims: &ProviderEnrollmentClaimsV2,
    ) -> Result<SignedProviderEnrollmentV2, WireError> {
        claims.validate()?;
        if claims.issuer != self.0.issuer() || claims.key_id != self.0.key_id() {
            return Err(error(
                "PROVIDER_ENROLLMENT_KEY_UNKNOWN",
                "provider enrollment claims do not name the signing key",
            ));
        }
        let payload = canonical_json(claims)?;
        let footer = canonical_json(&footer(self.0.issuer(), self.0.key_id()))?;
        let paseto = self
            .0
            .sign(&payload, &footer, PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION)
            .ok_or_else(|| {
                error(
                    "PROVIDER_ENROLLMENT_SIGNING_FAILED",
                    "provider enrollment PASETO signing failed",
                )
            })?;
        Ok(SignedProviderEnrollmentV2 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            issuer: self.0.issuer().to_owned(),
            key_id: self.0.key_id().to_owned(),
            paseto,
        })
    }
}

/// Strict signed-envelope carrier. Verification always selects its key through policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedProviderEnrollmentV2 {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

impl SignedProviderEnrollmentV2 {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_envelope(self)?;
        hash_framed_bytes(PROVIDER_ENROLLMENT_ENVELOPE_DOMAIN, self.paseto.as_bytes())
    }

    pub fn verify(
        &self,
        policy: &PolicySnapshotV1,
        expected: &ProviderEnrollmentExpectationV2,
        trusted_now_unix_ms: u64,
    ) -> Result<ProviderEnrollmentClaimsV2, WireError> {
        validate_envelope(self)?;
        let selected = policy.provider_enrollment_signer_key_at(
            &self.issuer,
            &self.key_id,
            trusted_now_unix_ms,
        )?;
        let key = PurposeSeparatedPasetoVerificationKey::from_lower_hex(
            &selected.issuer,
            &selected.key_id,
            &selected.public_key,
        )
        .ok_or_else(|| {
            error(
                "INVALID_PROVIDER_ENROLLMENT_KEY",
                "validated policy key could not construct a provider enrollment verifier",
            )
        })?;
        let footer = canonical_json(&footer(key.issuer(), key.key_id()))?;
        let payload = key
            .authenticate(
                &self.paseto,
                &footer,
                PROVIDER_ENROLLMENT_IMPLICIT_ASSERTION,
            )
            .ok_or_else(|| {
                error(
                    "PROVIDER_ENROLLMENT_INVALID",
                    "provider enrollment PASETO framing, signature, footer, or assertion is invalid",
                )
            })?;
        let claims: ProviderEnrollmentClaimsV2 = decode_canonical(&payload).map_err(|source| {
            error(
                "PROVIDER_ENROLLMENT_INVALID",
                format!("provider enrollment payload is not strict canonical claims: {source}"),
            )
        })?;
        claims.validate()?;
        if claims.issuer != key.issuer()
            || claims.key_id != key.key_id()
            || claims.issuer != self.issuer
            || claims.key_id != self.key_id
        {
            return Err(error(
                "PROVIDER_ENROLLMENT_KEY_UNKNOWN",
                "signed enrollment claims do not name the selected envelope key",
            ));
        }
        if claims.enrollment_generation != expected.enrollment_generation {
            return Err(error(
                "PROVIDER_ENROLLMENT_GENERATION_MISMATCH",
                "signed enrollment generation does not match the expected generation",
            ));
        }
        let enrollment_id = claims.enrollment_id()?;
        let loaded_policy_digest = policy_snapshot_digest(&canonical_json(policy)?)?;
        if policy.policy_generation != expected.policy_generation
            || claims.policy_generation != expected.policy_generation
            || claims.policy_generation != policy.policy_generation
            || loaded_policy_digest != expected.policy_snapshot_digest
            || claims.policy_snapshot_digest != expected.policy_snapshot_digest
            || enrollment_id.as_str() != expected.provider_enrollment_id.as_str()
        {
            return Err(error(
                "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
                "signed enrollment does not bind the expected policy and enrollment subject",
            ));
        }
        if trusted_now_unix_ms < claims.activates_at_unix_ms {
            return Err(error(
                "PROVIDER_ENROLLMENT_NOT_YET_VALID",
                "provider enrollment is not active yet",
            ));
        }
        if claims
            .revoked_at_unix_ms
            .is_some_and(|revoked| trusted_now_unix_ms >= revoked)
        {
            return Err(error(
                "PROVIDER_ENROLLMENT_REVOKED",
                "provider enrollment has been revoked",
            ));
        }
        if trusted_now_unix_ms >= claims.expires_at_unix_ms {
            return Err(error(
                "PROVIDER_ENROLLMENT_EXPIRED",
                "provider enrollment has expired",
            ));
        }
        Ok(claims)
    }
}

#[derive(Serialize)]
struct ProviderEnrollmentFooter<'a> {
    schema_version: &'static str,
    issuer: &'a str,
    key_id: &'a str,
    purpose: &'static str,
}

fn footer<'a>(issuer: &'a str, key_id: &'a str) -> ProviderEnrollmentFooter<'a> {
    ProviderEnrollmentFooter {
        schema_version: DOGFOOD_SCHEMA_VERSION,
        issuer,
        key_id,
        purpose: PROVIDER_ENROLLMENT_SIGNING_PURPOSE,
    }
}

fn validate_envelope(envelope: &SignedProviderEnrollmentV2) -> Result<(), WireError> {
    if envelope.schema_version != DOGFOOD_SCHEMA_VERSION
        || !valid_key_identity(&envelope.issuer, &envelope.key_id)
        || !envelope.paseto.starts_with("v4.public.")
        || envelope.paseto.len() > MAX_PROVIDER_ENROLLMENT_TOKEN_BYTES
    {
        return Err(error(
            "PROVIDER_ENROLLMENT_INVALID",
            "provider enrollment requires schema v1alpha1, bounded identity, and bounded v4.public PASETO",
        ));
    }
    Ok(())
}

fn validate_provider_pair(
    provider: LaunchProvider,
    protocol: DogfoodProviderProtocolV1,
) -> Result<(), WireError> {
    let expected = DogfoodProviderProtocolV1::required_for(provider);
    if protocol != expected {
        return Err(error(
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

fn error(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}

#[cfg(test)]
pub(super) mod test_support {
    use super::*;
    use crate::*;

    pub(super) const DOGFOOD_SECRET_HEX: &str = "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    pub(super) const DOGFOOD_PUBLIC_HEX: &str =
        "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    fn digest(seed: u8) -> Blake3Digest {
        Blake3Digest::from_bytes([seed; 32])
    }

    pub(super) fn dogfood_launch_fixture(
        enrollment: &ProviderEnrollmentClaimsV2,
    ) -> (DogfoodReadOnlyIntentV1, DogfoodLaunchGrantClaimsV1) {
        let subject = DogfoodRunSubjectV1 {
            execution: DogfoodExecutionSubjectV1 {
                command_id: CommandId::from_digest(digest(40)),
                run_id: DogfoodRunId::from_digest(digest(41)),
                mission_id: MissionId::from_digest(digest(42)),
                repository_id: RepositoryId::from_digest(digest(43)),
                graph_revision_id: GraphRevisionId::from_digest(digest(44)),
                work_package_id: WorkPackageId::from_digest(digest(45)),
                variant_id: VariantId::from_digest(digest(46)),
                attempt_id: AttemptId::from_digest(digest(47)),
                attempt_fence: 3,
                runner_id: RunnerId::from_digest(digest(48)),
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
                credential_projection_id: ProviderCredentialProjectionId::from_digest(digest(49)),
            },
            repository: DogfoodRepositorySubjectV1 {
                context_snapshot_id: RepositoryContextSnapshotId::from_digest(digest(50)),
                head_oid: GitOid::Sha256(format!("{:02x}", 51).repeat(32)),
                tree_oid: GitOid::Sha256(format!("{:02x}", 52).repeat(32)),
                checkpoint_id: CheckpointId::from_digest(digest(53)),
            },
            gate_ids: vec![GateId::from_digest(digest(54))],
            prompt_digest: digest(55),
            policy: DogfoodPolicySubjectV1 {
                policy_snapshot_digest: enrollment.policy_snapshot_digest,
                policy_generation: enrollment.policy_generation,
                dogfood_binding_digest: digest(56),
                tool_policy_digest: enrollment.tool_policy_digest,
                egress_policy_digest: enrollment.egress_policy_digest,
                containment_policy_digest: digest(57),
            },
            budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(58)),
            deadline_unix_ms: enrollment.activates_at_unix_ms + 2_000,
            output_schema_digest: digest(59),
        };
        let intent = DogfoodReadOnlyIntentV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            request_digest: digest(60),
            subject,
        };
        let claims = DogfoodLaunchGrantClaimsV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            audience: DogfoodAudienceV1::DogfoodRunner,
            operation: DogfoodOperationV1::ReadOnlyPropose,
            issuer: "dogfood-operator".to_owned(),
            key_id: "dogfood-launch-1".to_owned(),
            signing_purpose: DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE.to_owned(),
            claims_domain: DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN.to_owned(),
            issued_at_unix_ms: enrollment.activates_at_unix_ms,
            not_before_unix_ms: enrollment.activates_at_unix_ms,
            expires_at_unix_ms: enrollment.activates_at_unix_ms + 1_000,
            grant_nonce: digest(61),
            request_digest: intent.request_digest,
            intent_id: intent.intent_id().unwrap(),
            subject: intent.subject.clone(),
        };
        (intent, claims)
    }

    pub(super) fn signed_dogfood_launch(
        enrollment: &ProviderEnrollmentClaimsV2,
    ) -> (
        SignedDogfoodLaunchGrantV1,
        DogfoodReadOnlyIntentV1,
        DogfoodLaunchGrantClaimsV1,
    ) {
        let (intent, claims) = dogfood_launch_fixture(enrollment);
        let secret = hex::decode(DOGFOOD_SECRET_HEX).unwrap();
        let public = hex::decode(DOGFOOD_PUBLIC_HEX).unwrap();
        let signer =
            DogfoodLaunchSigningKey::from_bytes("dogfood-operator", "dogfood-launch-1", &secret)
                .unwrap();
        let verifier = DogfoodLaunchVerificationKey::from_bytes(
            "dogfood-operator",
            "dogfood-launch-1",
            &public,
        )
        .unwrap();
        let signed = signer.sign(&claims).unwrap();
        signed
            .verify(&verifier, &intent, enrollment, claims.not_before_unix_ms)
            .unwrap();
        (signed, intent, claims)
    }
}

#[cfg(test)]
mod tests;
