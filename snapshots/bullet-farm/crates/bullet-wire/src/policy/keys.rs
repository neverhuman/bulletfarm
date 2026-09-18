use std::collections::BTreeSet;

use super::{
    DogfoodAudienceV1, DogfoodBindingV1, DogfoodOperationV1, IssuerKeyV1, KeyAlgorithmV1,
    KeyPurposeV1, POLICY_SCHEMA_VERSION_V1ALPHA2, PolicySnapshotV1, require_v1alpha1,
};
use crate::{AuthorityAudience, AuthorityVerificationKey, PrincipalId, WireError};

impl PolicySnapshotV1 {
    pub fn authority_key_at(
        &self,
        issuer: &str,
        key_id: &str,
        audience: AuthorityAudience,
        now_unix_ms: u64,
    ) -> Result<&IssuerKeyV1, WireError> {
        self.validate()?;
        if now_unix_ms < self.activation_at_unix_ms || now_unix_ms >= self.expires_at_unix_ms {
            return Err(WireError::new(
                "POLICY_NOT_ACTIVE",
                "authority verification requires an active policy snapshot",
            ));
        }
        let key = self
            .issuer_keys
            .iter()
            .find(|key| key.issuer == issuer && key.key_id == key_id)
            .ok_or_else(|| {
                WireError::new(
                    "AUTHORITY_KEY_UNKNOWN",
                    "authority issuer and key ID are not registered",
                )
            })?;
        if key.key_purpose != KeyPurposeV1::AuthoritySigning
            || key.algorithm != KeyAlgorithmV1::PasetoV4Public
        {
            return Err(WireError::new(
                "AUTHORITY_KEY_WRONG_PURPOSE",
                "selected key is not an authority-signing PASETO key",
            ));
        }
        if !key.audiences.contains(&audience) {
            return Err(WireError::new(
                "AUTHORITY_KEY_AUDIENCE_MISMATCH",
                "selected key is not admitted for this authority audience",
            ));
        }
        if now_unix_ms < key.activates_at_unix_ms
            || now_unix_ms >= key.expires_at_unix_ms
            || key
                .revoked_at_unix_ms
                .is_some_and(|revoked| now_unix_ms >= revoked)
        {
            return Err(WireError::new(
                "AUTHORITY_KEY_INACTIVE",
                "selected authority key is not active at the verification instant",
            ));
        }
        Ok(key)
    }

    /// Resolve one active key that is cryptographically limited to the
    /// dogfood read-only launch boundary.
    pub fn dogfood_signer_key_at(
        &self,
        issuer: &str,
        key_id: &str,
        binding: &DogfoodBindingV1,
        now_unix_ms: u64,
    ) -> Result<&IssuerKeyV1, WireError> {
        self.validate()?;
        validate_dogfood_scope(self, binding)?;
        if now_unix_ms < self.activation_at_unix_ms || now_unix_ms >= self.expires_at_unix_ms {
            return Err(WireError::new(
                "POLICY_NOT_ACTIVE",
                "dogfood signer resolution requires an active policy snapshot",
            ));
        }
        let key = self
            .issuer_keys
            .iter()
            .find(|key| key.issuer == issuer && key.key_id == key_id)
            .ok_or_else(|| {
                WireError::new(
                    "DOGFOOD_SIGNER_KEY_UNKNOWN",
                    "dogfood signer issuer and key ID are not registered",
                )
            })?;
        if key.key_purpose != KeyPurposeV1::DogfoodLaunchSigning
            || key.algorithm != KeyAlgorithmV1::PasetoV4Public
            || !key.audiences.is_empty()
        {
            return Err(WireError::new(
                "DOGFOOD_SIGNER_KEY_WRONG_PURPOSE",
                "selected key is not a dogfood-launch-signing PASETO key",
            ));
        }
        if !overlaps_policy_window(key, self)
            || now_unix_ms < key.activates_at_unix_ms
            || now_unix_ms >= key.expires_at_unix_ms
            || key
                .revoked_at_unix_ms
                .is_some_and(|revoked| now_unix_ms >= revoked)
        {
            return Err(WireError::new(
                "DOGFOOD_SIGNER_KEY_INACTIVE",
                "selected dogfood signer key is not active at the verification instant",
            ));
        }
        Ok(key)
    }

    /// Resolve one active key used only for signed provider enrollments.
    pub fn provider_enrollment_signer_key_at(
        &self,
        issuer: &str,
        key_id: &str,
        now_unix_ms: u64,
    ) -> Result<&IssuerKeyV1, WireError> {
        self.validate()?;
        if now_unix_ms < self.activation_at_unix_ms || now_unix_ms >= self.expires_at_unix_ms {
            return Err(WireError::new(
                "POLICY_NOT_ACTIVE",
                "provider enrollment signer resolution requires an active policy snapshot",
            ));
        }
        let key = self
            .issuer_keys
            .iter()
            .find(|key| key.issuer == issuer && key.key_id == key_id)
            .ok_or_else(|| {
                WireError::new(
                    "PROVIDER_ENROLLMENT_SIGNER_KEY_UNKNOWN",
                    "provider enrollment signer issuer and key ID are not registered",
                )
            })?;
        if key.key_purpose != KeyPurposeV1::ProviderEnrollmentSigning
            || key.algorithm != KeyAlgorithmV1::PasetoV4Public
            || !key.audiences.is_empty()
        {
            return Err(WireError::new(
                "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE",
                "selected key is not a provider-enrollment-signing PASETO key",
            ));
        }
        if !overlaps_policy_window(key, self)
            || now_unix_ms < key.activates_at_unix_ms
            || now_unix_ms >= key.expires_at_unix_ms
            || key
                .revoked_at_unix_ms
                .is_some_and(|revoked| now_unix_ms >= revoked)
        {
            return Err(WireError::new(
                "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE",
                "selected provider enrollment signer key is not active",
            ));
        }
        Ok(key)
    }

    /// Resolve one active key owned by the terminal run's typed attestor.
    pub fn dogfood_run_attestor_key_at(
        &self,
        attestor_principal_id: &PrincipalId,
        key_id: &str,
        now_unix_ms: u64,
    ) -> Result<&IssuerKeyV1, WireError> {
        self.validate()?;
        if now_unix_ms < self.activation_at_unix_ms || now_unix_ms >= self.expires_at_unix_ms {
            return Err(WireError::new(
                "POLICY_NOT_ACTIVE",
                "dogfood run attestor resolution requires an active policy snapshot",
            ));
        }
        let key = self
            .issuer_keys
            .iter()
            .find(|key| key.issuer == attestor_principal_id.as_str() && key.key_id == key_id)
            .ok_or_else(|| {
                WireError::new(
                    "DOGFOOD_RUN_ATTESTOR_KEY_UNKNOWN",
                    "dogfood run attestor principal and key ID are not registered",
                )
            })?;
        if key.key_purpose != KeyPurposeV1::DogfoodRunAttestationSigning
            || key.algorithm != KeyAlgorithmV1::PasetoV4Public
            || !key.audiences.is_empty()
        {
            return Err(WireError::new(
                "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE",
                "selected key is not a dogfood-run-attestation-signing PASETO key",
            ));
        }
        if !overlaps_policy_window(key, self)
            || now_unix_ms < key.activates_at_unix_ms
            || now_unix_ms >= key.expires_at_unix_ms
            || key
                .revoked_at_unix_ms
                .is_some_and(|revoked| now_unix_ms >= revoked)
        {
            return Err(WireError::new(
                "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE",
                "selected dogfood run attestor key is not active",
            ));
        }
        Ok(key)
    }
}

pub(super) fn validate_issuer_keys(keys: &[IssuerKeyV1]) -> Result<(), WireError> {
    let mut identities = BTreeSet::new();
    let mut paseto_material = BTreeSet::new();
    for key in keys {
        let audience_count = key.audiences.iter().copied().collect::<BTreeSet<_>>().len();
        require_v1alpha1(&key.schema_version, "IssuerKeyV1")?;
        if key.issuer.is_empty()
            || key.key_id.is_empty()
            || !identities.insert((key.issuer.as_str(), key.key_id.as_str()))
            || key.activates_at_unix_ms >= key.expires_at_unix_ms
            || key.retain_until_unix_ms < key.expires_at_unix_ms.saturating_add(15_000)
            || key.revoked_at_unix_ms.is_some_and(|revoked| {
                revoked < key.activates_at_unix_ms || revoked > key.retain_until_unix_ms
            })
            || audience_count != key.audiences.len()
        {
            return Err(WireError::new(
                "INVALID_ISSUER_KEY_LIFECYCLE",
                "issuer keys require unique identities and ordered activation, expiry, revocation, and retention windows",
            ));
        }
        match (&key.key_purpose, &key.algorithm) {
            (KeyPurposeV1::AuthoritySigning, KeyAlgorithmV1::PasetoV4Public) => {
                if key.audiences.is_empty()
                    || key.public_key.len() != 64
                    || !key
                        .public_key
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(WireError::new(
                        "INVALID_AUTHORITY_PUBLIC_KEY",
                        "authority keys require audiences and a raw 32-byte lowercase-hex public key",
                    ));
                }
                let bytes = decode_authority_key(&key.public_key)?;
                AuthorityVerificationKey::from_bytes(&key.issuer, &key.key_id, &bytes)?;
            }
            (KeyPurposeV1::DogfoodLaunchSigning, algorithm) => {
                if *algorithm != KeyAlgorithmV1::PasetoV4Public
                    || !key.audiences.is_empty()
                    || !is_lower_hex_64(&key.public_key)
                {
                    return Err(WireError::new(
                        "INVALID_DOGFOOD_PUBLIC_KEY",
                        "dogfood launch keys require PASETO, no authority audiences, and a raw 32-byte lowercase-hex public key",
                    ));
                }
                let bytes = decode_key(&key.public_key, "INVALID_DOGFOOD_PUBLIC_KEY")?;
                AuthorityVerificationKey::from_bytes(&key.issuer, &key.key_id, &bytes).map_err(
                    |_| {
                        WireError::new(
                            "INVALID_DOGFOOD_PUBLIC_KEY",
                            "dogfood launch verification key is invalid",
                        )
                    },
                )?;
            }
            (KeyPurposeV1::ProviderEnrollmentSigning, algorithm) => {
                if *algorithm != KeyAlgorithmV1::PasetoV4Public
                    || !key.audiences.is_empty()
                    || !is_lower_hex_64(&key.public_key)
                {
                    return Err(WireError::new(
                        "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY",
                        "provider enrollment keys require PASETO, no authority audiences, and a raw 32-byte lowercase-hex public key",
                    ));
                }
                let bytes = decode_key(&key.public_key, "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY")?;
                AuthorityVerificationKey::from_bytes(&key.issuer, &key.key_id, &bytes).map_err(
                    |_| {
                        WireError::new(
                            "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY",
                            "provider enrollment verification key is invalid",
                        )
                    },
                )?;
            }
            (KeyPurposeV1::DogfoodRunAttestationSigning, algorithm) => {
                if *algorithm != KeyAlgorithmV1::PasetoV4Public
                    || !key.audiences.is_empty()
                    || !is_lower_hex_64(&key.public_key)
                {
                    return Err(WireError::new(
                        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY",
                        "dogfood run attestor keys require PASETO, no authority audiences, and a raw 32-byte lowercase-hex public key",
                    ));
                }
                let bytes = decode_key(&key.public_key, "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY")?;
                AuthorityVerificationKey::from_bytes(&key.issuer, &key.key_id, &bytes).map_err(
                    |_| {
                        WireError::new(
                            "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY",
                            "dogfood run attestor verification key is invalid",
                        )
                    },
                )?;
            }
            (KeyPurposeV1::ReleaseSigning, KeyAlgorithmV1::SshEd25519) => {
                if !key.audiences.is_empty() || !key.public_key.starts_with("ssh-ed25519 ") {
                    return Err(WireError::new(
                        "INVALID_RELEASE_PUBLIC_KEY",
                        "release keys require SSH Ed25519 form and no runtime audience",
                    ));
                }
            }
            _ => {
                return Err(WireError::new(
                    "INVALID_KEY_USE",
                    "key purpose and algorithm are incompatible",
                ));
            }
        }
        if key.algorithm == KeyAlgorithmV1::PasetoV4Public
            && !paseto_material.insert(key.public_key.as_str())
        {
            return Err(WireError::new(
                "SIGNER_KEY_MATERIAL_REUSED",
                "PASETO public key material must have exactly one catalog identity and purpose",
            ));
        }
    }
    Ok(())
}

fn decode_authority_key(raw: &str) -> Result<[u8; 32], WireError> {
    decode_key(raw, "INVALID_AUTHORITY_PUBLIC_KEY")
}

fn decode_key(raw: &str, code: &'static str) -> Result<[u8; 32], WireError> {
    let mut bytes = [0_u8; 32];
    for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair)
            .map_err(|_| WireError::new(code, "PASETO public key is not lowercase hexadecimal"))?;
        bytes[index] = u8::from_str_radix(text, 16)
            .map_err(|_| WireError::new(code, "PASETO public key is not lowercase hexadecimal"))?;
    }
    Ok(bytes)
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_dogfood_scope(
    policy: &PolicySnapshotV1,
    binding: &DogfoodBindingV1,
) -> Result<(), WireError> {
    if policy.sandbox_policy.live_admission_enabled {
        return Err(WireError::new(
            "DOGFOOD_REFUSES_LIVE_ADMISSION",
            "dogfood signer resolution refuses a general live binding",
        ));
    }
    if policy.schema_version != POLICY_SCHEMA_VERSION_V1ALPHA2 {
        return Err(WireError::new(
            "UNSUPPORTED_POLICY_SCHEMA",
            "dogfood signer resolution requires policy schema v1alpha2",
        ));
    }
    if policy.policy_generation < super::live::LIVE_ADMISSION_MIN_GENERATION {
        return Err(WireError::new(
            "LIVE_ADMISSION_REQUIRES_GENERATION",
            "dogfood signer resolution requires policy generation 2 or later",
        ));
    }
    if binding.schema_version != DogfoodBindingV1::SCHEMA_VERSION
        || binding.audience != DogfoodAudienceV1::DogfoodRunner
        || binding.operation != DogfoodOperationV1::ReadOnlyPropose
    {
        return Err(WireError::new(
            "INVALID_DOGFOOD_BINDING",
            "dogfood binding must be dogfood-runner / read-only-propose",
        ));
    }
    Ok(())
}

fn overlaps_policy_window(key: &IssuerKeyV1, policy: &PolicySnapshotV1) -> bool {
    let effective_expiry = key
        .revoked_at_unix_ms
        .unwrap_or(key.expires_at_unix_ms)
        .min(key.expires_at_unix_ms);
    key.activates_at_unix_ms < policy.expires_at_unix_ms
        && effective_expiry > policy.activation_at_unix_ms
}
