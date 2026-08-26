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
pub(super) fn validate_live_admission(policy: &PolicySnapshotV1) -> Result<(), WireError> {
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

fn qualifies_for_live_admission(key: &IssuerKeyV1) -> bool {
    key.key_purpose == KeyPurposeV1::AuthoritySigning
        && key.algorithm == KeyAlgorithmV1::PasetoV4Public
        && key.audiences.contains(&AuthorityAudience::ProviderRunner)
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
