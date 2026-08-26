use std::collections::BTreeSet;

use super::{IssuerKeyV1, KeyAlgorithmV1, KeyPurposeV1, PolicySnapshotV1, require_v1alpha1};
use crate::{AuthorityAudience, AuthorityVerificationKey, WireError};

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
}

pub(super) fn validate_issuer_keys(keys: &[IssuerKeyV1]) -> Result<(), WireError> {
    let mut identities = BTreeSet::new();
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
    }
    Ok(())
}

fn decode_authority_key(raw: &str) -> Result<[u8; 32], WireError> {
    let mut bytes = [0_u8; 32];
    for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).map_err(|_| {
            WireError::new(
                "INVALID_AUTHORITY_PUBLIC_KEY",
                "authority public key is not lowercase hexadecimal",
            )
        })?;
        bytes[index] = u8::from_str_radix(text, 16).map_err(|_| {
            WireError::new(
                "INVALID_AUTHORITY_PUBLIC_KEY",
                "authority public key is not lowercase hexadecimal",
            )
        })?;
    }
    Ok(bytes)
}
