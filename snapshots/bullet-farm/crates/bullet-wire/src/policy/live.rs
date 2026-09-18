//! v1alpha2 live-admission rule (ADR 0012). A snapshot may set
//! `sandbox_policy.live_admission_enabled` only at an operator generation of
//! at least [`LIVE_ADMISSION_MIN_GENERATION`] and only while it registers an
//! unrevoked `authority-signing` PASETO key admitted for the `provider-runner`
//! audience. `validate` stays structural; `validate_at` applies the same
//! instant semantics as `authority_key_at`.

use super::{IssuerKeyV1, KeyAlgorithmV1, KeyPurposeV1, PolicySnapshotV1};
use crate::{AuthorityAudience, WireError};

/// First policy generation that may enable live provider admission. Generation 1
/// is the committed Gate 0 offline policy and can never admit a provider.
pub const LIVE_ADMISSION_MIN_GENERATION: u64 = 2;

impl PolicySnapshotV1 {
    /// `validate` plus the time-bound checks at `now_unix_ms`: the snapshot
    /// window must contain the instant and, when live admission is enabled, at
    /// least one qualifying provider-runner key must be active at that instant.
    pub fn validate_at(&self, now_unix_ms: u64) -> Result<(), WireError> {
        self.validate()?;
        if now_unix_ms < self.activation_at_unix_ms || now_unix_ms >= self.expires_at_unix_ms {
            return Err(WireError::new(
                "POLICY_NOT_ACTIVE",
                "policy snapshot is not active at the validation instant",
            ));
        }
        if self.sandbox_policy.live_admission_enabled
            && !self
                .issuer_keys
                .iter()
                .any(|key| qualifies_for_live_admission(key) && key_active_at(key, now_unix_ms))
        {
            return Err(WireError::new(
                "LIVE_ADMISSION_REQUIRES_RUNNER_KEY",
                "no provider-runner authority key is active at the validation instant",
            ));
        }
        Ok(())
    }
}

/// Structural rule for a v1alpha2 snapshot whose live admission is enabled.
/// The caller has already enforced the immutable conservatism set.
pub fn validate_live_admission(policy: &PolicySnapshotV1) -> Result<(), WireError> {
    if !policy.sandbox_policy.live_admission_enabled {
        return Err(WireError::new(
            "LIVE_ADMISSION_DISABLED",
            "live admission cannot be satisfied by a dogfood-only or offline policy",
        ));
    }
    if policy.policy_generation < LIVE_ADMISSION_MIN_GENERATION {
        return Err(WireError::new(
            "LIVE_ADMISSION_REQUIRES_GENERATION",
            format!(
                "live admission requires policy generation {LIVE_ADMISSION_MIN_GENERATION} or later; generation {} is refused",
                policy.policy_generation
            ),
        ));
    }
    if !policy
        .issuer_keys
        .iter()
        .any(|key| qualifies_for_live_admission(key) && overlaps_policy_window(key, policy))
    {
        return Err(WireError::new(
            "LIVE_ADMISSION_REQUIRES_RUNNER_KEY",
            "live admission requires an unrevoked authority-signing PASETO key admitted for the provider-runner audience within the policy window",
        ));
    }
    Ok(())
}

/// Admit one dogfood-scoped binding against a snapshot that stays offline.
///
/// # Errors
///
/// `DOGFOOD_REFUSES_LIVE_ADMISSION` when general live admission is enabled;
/// `UNSUPPORTED_POLICY_SCHEMA` / `DOGFOOD_ADMISSION_REQUIRES_GENERATION` for a
/// Gate 0 snapshot; `INVALID_DOGFOOD_BINDING` for a wrong audience, operation,
/// or schema; `DOGFOOD_ADMISSION_REQUIRES_SIGNER_KEY` when no overlapping
/// unrevoked dogfood-launch-signing PASETO key exists.
pub fn validate_dogfood_admission(
    policy: &PolicySnapshotV1,
    binding: &super::DogfoodBindingV1,
) -> Result<(), WireError> {
    if policy.sandbox_policy.live_admission_enabled {
        return Err(WireError::new(
            "DOGFOOD_REFUSES_LIVE_ADMISSION",
            "dogfood admission refuses a general live binding",
        ));
    }
    policy.validate()?;
    let schema = policy.schema()?;
    if schema != super::PolicySchemaVersion::V1Alpha2 {
        return Err(WireError::new(
            "UNSUPPORTED_POLICY_SCHEMA",
            "dogfood admission requires policy schema v1alpha2",
        ));
    }
    if policy.policy_generation < LIVE_ADMISSION_MIN_GENERATION {
        return Err(WireError::new(
            "DOGFOOD_ADMISSION_REQUIRES_GENERATION",
            format!(
                "dogfood admission requires policy generation {LIVE_ADMISSION_MIN_GENERATION} or later; generation {} is refused",
                policy.policy_generation
            ),
        ));
    }
    validate_binding(binding)?;
    if !policy
        .issuer_keys
        .iter()
        .any(|key| qualifies_for_dogfood_admission(key) && overlaps_policy_window(key, policy))
    {
        return Err(WireError::new(
            "DOGFOOD_ADMISSION_REQUIRES_SIGNER_KEY",
            "dogfood admission requires an unrevoked dogfood-launch-signing PASETO key within the policy window",
        ));
    }
    Ok(())
}

/// A dogfood binding can never satisfy the general live path.
///
/// # Errors
///
/// Always `LIVE_ADMISSION_REFUSES_DOGFOOD_BINDING`.
pub fn refuse_dogfood_binding_as_live(binding: &super::DogfoodBindingV1) -> Result<(), WireError> {
    validate_binding(binding)?;
    Err(WireError::new(
        "LIVE_ADMISSION_REFUSES_DOGFOOD_BINDING",
        "a dogfood audience/operation binding cannot satisfy live admission",
    ))
}

fn validate_binding(binding: &super::DogfoodBindingV1) -> Result<(), WireError> {
    if binding.schema_version != super::DogfoodBindingV1::SCHEMA_VERSION {
        return Err(WireError::new(
            "INVALID_DOGFOOD_BINDING",
            "dogfood binding requires schema v1alpha1",
        ));
    }
    if binding.audience != super::DogfoodAudienceV1::DogfoodRunner
        || binding.operation != super::DogfoodOperationV1::ReadOnlyPropose
    {
        return Err(WireError::new(
            "INVALID_DOGFOOD_BINDING",
            "dogfood binding must be dogfood-runner / read-only-propose",
        ));
    }
    Ok(())
}

fn qualifies_for_live_admission(key: &IssuerKeyV1) -> bool {
    key.key_purpose == KeyPurposeV1::AuthoritySigning
        && key.algorithm == KeyAlgorithmV1::PasetoV4Public
        && key.audiences.contains(&AuthorityAudience::ProviderRunner)
        && key.revoked_at_unix_ms.is_none()
}

fn qualifies_for_dogfood_admission(key: &IssuerKeyV1) -> bool {
    key.key_purpose == KeyPurposeV1::DogfoodLaunchSigning
        && key.algorithm == KeyAlgorithmV1::PasetoV4Public
        && key.audiences.is_empty()
        && key.revoked_at_unix_ms.is_none()
}

fn overlaps_policy_window(key: &IssuerKeyV1, policy: &PolicySnapshotV1) -> bool {
    key.activates_at_unix_ms < policy.expires_at_unix_ms
        && key.expires_at_unix_ms > policy.activation_at_unix_ms
}

/// Same instant semantics as `authority_key_at`: activation inclusive, expiry
/// and revocation exclusive.
fn key_active_at(key: &IssuerKeyV1, now_unix_ms: u64) -> bool {
    now_unix_ms >= key.activates_at_unix_ms
        && now_unix_ms < key.expires_at_unix_ms
        && key
            .revoked_at_unix_ms
            .is_none_or(|revoked| now_unix_ms < revoked)
}
