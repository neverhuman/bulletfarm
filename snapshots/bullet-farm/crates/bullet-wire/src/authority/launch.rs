//! Signed launch grant: the Kernel-issued PASETO v4.public token a provider
//! runner must present before any provider process is spawned.
//!
//! The wire shape mirrors the ADR 0005 authority contract exactly:
//!
//! - payload: RFC 8785 canonical JSON of [`LaunchGrantClaims`]
//!   (`LaunchGrantClaimsV1` in the catalog);
//! - footer: canonical JSON of schema, issuer, key ID, and the fixed
//!   [`LAUNCH_GRANT_SIGNING_PURPOSE`];
//! - implicit assertion: [`LAUNCH_GRANT_IMPLICIT_ASSERTION`];
//! - envelope digest: [`LAUNCH_GRANT_ENVELOPE_DOMAIN`] over the exact compact
//!   token bytes, the same framed BLAKE3 family as `authority.envelope.v1alpha1`.
//!
//! Audience `provider-runner` is the third [`AuthorityAudience`] variant so that
//! `IssuerKeyV1.audiences` and `PolicySnapshotV1::authority_key_at` admit grant
//! keys through the existing `authority-signing` key purpose. No new key purpose
//! exists: `launch-grant-signing` is only the PASETO footer purpose that domain
//! separates grants from authority envelopes and mutation permits.
//! `AuthorityClaims::validate` still refuses the new audience because no mutation
//! operation requires it.
//!
//! Replay: `grant_nonce` is single use. This module proves signature, key
//! identity, audience, exact subject binding, and the time window. Consumed
//! nonces are Kernel state; the Kernel persists them with their expiry and
//! refuses `LAUNCH_GRANT_REPLAYED` before any spawn.

use std::{collections::BTreeMap, convert::TryFrom};

use pasetors::{
    Public,
    token::UntrustedToken,
    version4::{PublicToken, V4},
};
use serde::{Deserialize, Serialize};

use super::{
    AUTHORITY_SCHEMA_VERSION, AuthorityAudience, AuthoritySigningKey, AuthorityVerificationKey,
    authority_error, validate_key_identity,
};
use crate::{
    AttemptId, Blake3Digest, GateId, GraphRevisionId, MissionId, ProviderProfileId, RepositoryId,
    RunnerId, VariantId, WireError, WorkPackageId, WorkspaceId, canonical_json, decode_canonical,
    hash_canonical, hash_framed_bytes,
};

mod subject;
mod validate;
pub use subject::{LaunchLeaseSubject, LaunchProviderSubject};

pub const LAUNCH_GRANT_IMPLICIT_ASSERTION: &[u8] = b"bullet-farm.launch-grant.v1alpha1";
pub const LAUNCH_GRANT_SIGNING_PURPOSE: &str = "launch-grant-signing";
pub const LAUNCH_GRANT_CLAIMS_DOMAIN: &str = "authority.launch-grant-claims.v1alpha1";
pub const LAUNCH_GRANT_ENVELOPE_DOMAIN: &str = "authority.launch-grant-envelope.v1alpha1";
pub const LAUNCH_GRANT_WORKSPACE_NONCE_DOMAIN: &str = "launch-grant.workspace-nonce.v1alpha1";
pub const LAUNCH_GRANT_ENVIRONMENT_DOMAIN: &str = "launch-grant.environment.v1alpha1";
pub const LAUNCH_GRANT_POLICY_DOMAIN: &str = "policy.snapshot";
pub const MAX_LAUNCH_GRANT_TTL_MS: u64 = 15_000;
pub const MAX_LAUNCH_GRANT_GATE_IDS: usize = 16;

/// The only operation a launch grant can authorize.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchOperation {
    LaunchProvider,
}

/// Closed set of provider executables a grant may bind.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchProvider {
    Claude,
    Codex,
    Cursor,
    Agy,
}

impl LaunchProvider {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Agy => "agy",
        }
    }
}

/// Strict `LaunchGrantClaimsV1`. Every field is identity-bound by the signature.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchGrantClaims {
    pub schema_version: String,
    pub grant_id: Blake3Digest,
    pub audience: AuthorityAudience,
    pub operation: LaunchOperation,
    pub issuer: String,
    pub key_id: String,
    pub issued_at_unix_ms: u64,
    pub not_before_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub grant_nonce: Blake3Digest,
    pub mission_id: MissionId,
    pub repository_id: RepositoryId,
    pub graph_revision_id: GraphRevisionId,
    pub work_package_id: WorkPackageId,
    pub variant_id: VariantId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub runner_id: RunnerId,
    pub runner_epoch: u64,
    pub workspace_id: WorkspaceId,
    pub workspace_nonce_digest: Blake3Digest,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
    pub provider: LaunchProvider,
    pub adapter: String,
    pub provider_profile_id: ProviderProfileId,
    pub model: String,
    pub credential_generation: u64,
    pub protocol: String,
    pub executable_path: String,
    pub executable_digest: Blake3Digest,
    pub descriptor_digest: Blake3Digest,
    pub capability_digest: Blake3Digest,
    pub policy_snapshot_digest: Blake3Digest,
    pub policy_generation: u64,
    pub sandbox_manifest_digest: Blake3Digest,
    pub environment_digest: Blake3Digest,
    pub gate_ids: Vec<GateId>,
    pub budget_reservation_id: Blake3Digest,
    pub max_invocations: u64,
    pub max_wall_clock_ms: u64,
    pub max_cost_micro_usd: u64,
}

impl LaunchGrantClaims {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate_shape()?;
        hash_canonical(LAUNCH_GRANT_CLAIMS_DOMAIN, self)
    }
}

/// `SignedLaunchGrantV1`: the compact token plus the key identity that signed it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedLaunchGrant {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

/// Everything a verifier must supply from durable state before a spawn:
/// the gateway audience, the active lease row, the evaluated admission, and
/// the loaded policy digest. `now` is passed separately at verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchGrantExpectation {
    pub audience: AuthorityAudience,
    pub lease: LaunchLeaseSubject,
    pub provider: LaunchProviderSubject,
    pub policy_snapshot_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LaunchFooter {
    schema_version: String,
    issuer: String,
    key_id: String,
    purpose: String,
}

impl AuthoritySigningKey {
    pub fn sign_launch_grant(
        &self,
        claims: &LaunchGrantClaims,
    ) -> Result<SignedLaunchGrant, WireError> {
        claims.validate_shape()?;
        if claims.issuer != self.issuer || claims.key_id != self.key_id {
            return Err(launch_error(
                "LAUNCH_GRANT_KEY_UNKNOWN",
                "launch grant claims do not name the signing key",
            ));
        }
        let payload = canonical_json(claims)?;
        let footer = canonical_json(&launch_footer(&self.issuer, &self.key_id))?;
        let paseto = PublicToken::sign(
            &self.secret,
            &payload,
            Some(&footer),
            Some(LAUNCH_GRANT_IMPLICIT_ASSERTION),
        )
        .map_err(|_| launch_error("LAUNCH_GRANT_SIGNING_FAILED", "PASETO signing failed"))?;
        Ok(SignedLaunchGrant {
            schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            paseto,
        })
    }
}

impl SignedLaunchGrant {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_launch_envelope(self)?;
        hash_framed_bytes(LAUNCH_GRANT_ENVELOPE_DOMAIN, self.paseto.as_bytes())
    }

    /// Authenticate the token with `key`, then require the exact expected subject
    /// and an open time window at `now_unix_ms` (`not_before` inclusive,
    /// `expires_at` exclusive). Every failure is a typed `LAUNCH_GRANT_*` refusal;
    /// nonce replay is not checked here.
    pub fn verify(
        &self,
        key: &AuthorityVerificationKey,
        expected: &LaunchGrantExpectation,
        now_unix_ms: u64,
    ) -> Result<LaunchGrantClaims, WireError> {
        validate_launch_envelope(self)?;
        if self.issuer != key.issuer || self.key_id != key.key_id {
            return Err(launch_error(
                "LAUNCH_GRANT_KEY_UNKNOWN",
                "grant issuer or key ID does not match the selected verification key",
            ));
        }
        let footer = canonical_json(&launch_footer(&key.issuer, &key.key_id))?;
        let untrusted = UntrustedToken::<Public, V4>::try_from(self.paseto.as_str())
            .map_err(|_| launch_error("LAUNCH_GRANT_INVALID", "invalid PASETO framing"))?;
        let trusted = PublicToken::verify(
            &key.public,
            &untrusted,
            Some(&footer),
            Some(LAUNCH_GRANT_IMPLICIT_ASSERTION),
        )
        .map_err(|_| {
            launch_error(
                "LAUNCH_GRANT_INVALID",
                "PASETO signature, footer, or implicit assertion is invalid",
            )
        })?;
        let claims = decode_canonical::<LaunchGrantClaims>(trusted.payload().as_bytes()).map_err(
            |error| {
                launch_error(
                    "LAUNCH_GRANT_INVALID",
                    format!("launch grant payload is not strict canonical claims: {error}"),
                )
            },
        )?;
        claims.validate_shape()?;
        if claims.issuer != key.issuer || claims.key_id != key.key_id {
            return Err(launch_error(
                "LAUNCH_GRANT_KEY_UNKNOWN",
                "signed claims do not name the selected verification key",
            ));
        }
        if claims.audience != expected.audience {
            return Err(launch_error(
                "LAUNCH_GRANT_AUDIENCE_MISMATCH",
                "launch grant audience does not match this gateway",
            ));
        }
        claims.verify_subject(expected)?;
        if now_unix_ms < claims.not_before_unix_ms {
            return Err(launch_error(
                "LAUNCH_GRANT_NOT_YET_VALID",
                "launch grant is not valid yet",
            ));
        }
        if now_unix_ms >= claims.expires_at_unix_ms {
            return Err(launch_error(
                "LAUNCH_GRANT_EXPIRED",
                "launch grant has expired",
            ));
        }
        Ok(claims)
    }
}

/// `workspace_nonce_digest`: framed BLAKE3 of the raw 32-byte workspace nonce.
pub fn workspace_nonce_digest(nonce: &[u8; 32]) -> Result<Blake3Digest, WireError> {
    if nonce.iter().all(|byte| *byte == 0) {
        return Err(launch_error(
            "LAUNCH_GRANT_INVALID",
            "workspace nonce must not be all zero",
        ));
    }
    hash_framed_bytes(LAUNCH_GRANT_WORKSPACE_NONCE_DOMAIN, nonce)
}

/// `environment_digest`: framed BLAKE3 over the canonical JSON of the sorted,
/// allow-listed child environment.
pub fn environment_digest(
    environment: &BTreeMap<String, String>,
) -> Result<Blake3Digest, WireError> {
    for (name, value) in environment {
        if name.is_empty()
            || name.len() > 256
            || value.len() > 32_768
            || name.bytes().any(|byte| byte == b'=' || byte == 0)
            || value.bytes().any(|byte| byte == 0)
        {
            return Err(launch_error(
                "LAUNCH_GRANT_INVALID",
                "child environment names and values must be bounded and free of '=' and NUL",
            ));
        }
    }
    hash_canonical(LAUNCH_GRANT_ENVIRONMENT_DOMAIN, environment)
}

/// `policy_snapshot_digest`: the pinned `policy.snapshot` identity of the exact
/// canonical `policy.json` bytes the verifier loaded.
pub fn policy_snapshot_digest(canonical_policy: &[u8]) -> Result<Blake3Digest, WireError> {
    if canonical_policy.is_empty() {
        return Err(launch_error(
            "LAUNCH_GRANT_INVALID",
            "policy snapshot bytes are empty",
        ));
    }
    hash_framed_bytes(LAUNCH_GRANT_POLICY_DOMAIN, canonical_policy)
}

fn launch_footer(issuer: &str, key_id: &str) -> LaunchFooter {
    LaunchFooter {
        schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
        issuer: issuer.to_owned(),
        key_id: key_id.to_owned(),
        purpose: LAUNCH_GRANT_SIGNING_PURPOSE.to_owned(),
    }
}

fn validate_launch_envelope(grant: &SignedLaunchGrant) -> Result<(), WireError> {
    if grant.schema_version != AUTHORITY_SCHEMA_VERSION {
        return Err(launch_error(
            "LAUNCH_GRANT_INVALID",
            "launch grant envelope requires schema v1alpha1",
        ));
    }
    validate_key_identity(&grant.issuer, &grant.key_id)
        .map_err(|error| launch_error("LAUNCH_GRANT_INVALID", error.reason().to_owned()))?;
    if !grant.paseto.starts_with("v4.public.") || grant.paseto.len() > 32_768 {
        return Err(launch_error(
            "LAUNCH_GRANT_INVALID",
            "launch grant must be a bounded compact PASETO v4.public token",
        ));
    }
    Ok(())
}

pub(super) fn launch_error(code: &'static str, message: impl Into<String>) -> WireError {
    authority_error(code, message)
}
