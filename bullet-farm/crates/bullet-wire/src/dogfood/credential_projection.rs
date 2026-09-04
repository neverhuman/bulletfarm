//! Secret-free identity for one short-lived provider credential projection.
//! The record describes commitments only and cannot project or authorize a secret.

use serde::{Deserialize, Serialize};

use crate::{
    Blake3Digest, CredentialProjectionProfileId, DogfoodRunId, LaunchProvider, PrincipalId,
    ProviderCredentialProjectionId, WireError, decode_canonical, hash_canonical,
    ids::require_exact_wire,
};

use super::DOGFOOD_SCHEMA_VERSION;

pub const CREDENTIAL_PROJECTION_DIGEST_DOMAIN: &str = "provider.credential-projection.v1";
pub const MAX_CREDENTIAL_PROJECTION_TTL_MS: u64 = 15 * 60 * 1_000;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// One Kernel-selected projection instance. It contains no credential bytes,
/// credential path, environment name, or caller-selected target allowlist.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCredentialProjectionV1 {
    pub schema_version: String,
    pub projection_instance_id: ProviderCredentialProjectionId,
    pub credential_projection_profile_id: CredentialProjectionProfileId,
    pub run_id: DogfoodRunId,
    pub provider: LaunchProvider,
    pub service_identity_id: PrincipalId,
    pub activates_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub target_policy_digest: Blake3Digest,
    pub secret_commitment_digest: Blake3Digest,
}

impl ProviderCredentialProjectionV1 {
    /// Validate the strict secret-free component body.
    pub fn validate(&self) -> Result<(), WireError> {
        require_exact_wire(
            "schema_version",
            &self.schema_version,
            DOGFOOD_SCHEMA_VERSION,
            "CREDENTIAL_PROJECTION_INVALID",
        )?;
        for (name, value) in [
            ("activates_at_unix_ms", self.activates_at_unix_ms),
            ("expires_at_unix_ms", self.expires_at_unix_ms),
        ] {
            if value > MAX_SAFE_INTEGER {
                return Err(invalid(format!("{name} exceeds the safe integer range")));
            }
        }
        if self.activates_at_unix_ms >= self.expires_at_unix_ms {
            return Err(invalid("projection activation must precede expiry"));
        }
        if self.expires_at_unix_ms - self.activates_at_unix_ms > MAX_CREDENTIAL_PROJECTION_TTL_MS {
            return Err(invalid("projection window exceeds the 15 minute maximum"));
        }
        Ok(())
    }

    /// Domain-separated digest of every projection identity and commitment.
    pub fn projection_digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(CREDENTIAL_PROJECTION_DIGEST_DOMAIN, self)
    }
}

pub fn decode_provider_credential_projection(
    bytes: &[u8],
) -> Result<ProviderCredentialProjectionV1, WireError> {
    let projection: ProviderCredentialProjectionV1 = decode_canonical(bytes)?;
    projection.validate()?;
    Ok(projection)
}

fn invalid(reason: impl Into<String>) -> WireError {
    WireError::new("CREDENTIAL_PROJECTION_INVALID", reason)
}
