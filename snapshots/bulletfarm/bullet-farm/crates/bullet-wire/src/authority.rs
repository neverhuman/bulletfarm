use std::convert::TryFrom;

mod check;
mod launch;
mod operation;
mod permit;
mod request;
mod settlement;
pub use check::*;
pub use launch::*;
pub use permit::*;
pub use request::*;
pub use settlement::*;

use pasetors::{
    Public,
    keys::{AsymmetricPublicKey, AsymmetricSecretKey},
    token::UntrustedToken,
    version4::{PublicToken, V4},
};
use serde::{Deserialize, Serialize};

use crate::{
    AcceptanceContractId, AttemptId, Blake3Digest, ContentId, GraphRevisionId, MissionId,
    MutationId, OrganizationId, PlanRevisionId, PrincipalId, ProviderProfileId, RepositoryId,
    RunnerId, SelectionGroupId, VariantId, WireError, WorkPackageId, WorkspaceId, canonical_json,
    decode_canonical, hash_canonical, hash_framed_bytes,
};

pub const AUTHORITY_SCHEMA_VERSION: &str = "v1alpha1";
pub const AUTHORITY_IMPLICIT_ASSERTION: &[u8] = b"bullet-farm.authority.v1alpha1";
pub const MAX_AUTHORITY_TTL_MS: u64 = 15_000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorityAudience {
    BulletGitd,
    EffectBroker,
    ProviderRunner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationOperation {
    CloneWorkspace,
    ReadWorkspace,
    ApplyPatch,
    Checkpoint,
    PrepareCandidate,
    PreserveWorkspace,
    CleanupWorkspace,
    DispatchEffect,
    ReconcileEffect,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityClaims {
    pub schema_version: String,
    pub issuer: String,
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
    pub mutation_id: MutationId,
    pub subject_principal: PrincipalId,
    pub organization_id: OrganizationId,
    pub repository_id: RepositoryId,
    pub mission_id: MissionId,
    pub acceptance_contract_id: AcceptanceContractId,
    pub plan_revision_id: PlanRevisionId,
    pub graph_revision_id: GraphRevisionId,
    pub graph_sequence: u64,
    pub work_package_id: WorkPackageId,
    pub selection_group_id: SelectionGroupId,
    pub variant_id: VariantId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub runner_id: RunnerId,
    pub runner_epoch: u64,
    pub workspace_id: WorkspaceId,
    pub workspace_generation: u64,
    pub workspace_nonce: Blake3Digest,
    pub scope_grant_digest: Blake3Digest,
    pub scope_revision: u64,
    pub context_revision: u64,
    pub configuration_snapshot_id: ContentId,
    pub configuration_generation: u64,
    pub policy_snapshot_id: ContentId,
    pub policy_generation: u64,
    pub routing_snapshot_id: ContentId,
    pub routing_generation: u64,
    pub provider: String,
    pub model: String,
    pub adapter: String,
    pub provider_profile_id: ProviderProfileId,
    pub credential_generation: u64,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
    pub issued_at_unix_ms: u64,
    pub not_before_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub token_nonce: Blake3Digest,
}

impl AuthorityClaims {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.schema_version != AUTHORITY_SCHEMA_VERSION {
            return Err(authority_error(
                "UNSUPPORTED_AUTHORITY_SCHEMA",
                "authority claims require schema v1alpha1",
            ));
        }
        if self.audience != self.operation.required_audience() {
            return Err(authority_error(
                "INVALID_AUTHORITY_AUDIENCE",
                "authority operation is not valid for the selected gateway audience",
            ));
        }
        for (name, value) in [
            ("issuer", self.issuer.as_str()),
            ("provider", self.provider.as_str()),
            ("model", self.model.as_str()),
            ("adapter", self.adapter.as_str()),
        ] {
            validate_label(name, value)?;
        }
        for (name, value) in [
            ("graph_sequence", self.graph_sequence),
            ("attempt_fence", self.attempt_fence),
            ("runner_epoch", self.runner_epoch),
            ("workspace_generation", self.workspace_generation),
            ("scope_revision", self.scope_revision),
            ("context_revision", self.context_revision),
            ("configuration_generation", self.configuration_generation),
            ("policy_generation", self.policy_generation),
            ("routing_generation", self.routing_generation),
            ("credential_generation", self.credential_generation),
            ("authority_epoch", self.authority_epoch),
        ] {
            if value == 0 || value > MAX_SAFE_INTEGER {
                return Err(authority_error(
                    "INVALID_AUTHORITY_GENERATION",
                    format!("{name} must be a positive interoperable integer"),
                ));
            }
        }
        if self.freeze_generation > MAX_SAFE_INTEGER
            || self.issued_at_unix_ms > MAX_SAFE_INTEGER
            || self.not_before_unix_ms > MAX_SAFE_INTEGER
            || self.expires_at_unix_ms > MAX_SAFE_INTEGER
        {
            return Err(authority_error(
                "INVALID_AUTHORITY_TIME",
                "authority time or freeze generation exceeds the interoperable integer range",
            ));
        }
        if self.issued_at_unix_ms > self.not_before_unix_ms
            || self.not_before_unix_ms >= self.expires_at_unix_ms
            || self.expires_at_unix_ms - self.issued_at_unix_ms > MAX_AUTHORITY_TTL_MS
        {
            return Err(authority_error(
                "INVALID_AUTHORITY_WINDOW",
                "authority requires issued_at <= not_before < expires_at and a TTL at most 15s",
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical("authority.claims.v1alpha1", self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedAuthorityEnvelope {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

impl SignedAuthorityEnvelope {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_envelope(self)?;
        hash_framed_bytes("authority.envelope.v1alpha1", self.paseto.as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityExpectation {
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityFooter {
    schema_version: String,
    issuer: String,
    key_id: String,
    purpose: String,
}

pub struct AuthoritySigningKey {
    issuer: String,
    key_id: String,
    secret: AsymmetricSecretKey<V4>,
}

impl AuthoritySigningKey {
    pub fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Result<Self, WireError> {
        validate_key_identity(issuer, key_id)?;
        if bytes.len() != 64 || bytes.iter().all(|byte| *byte == 0) {
            return Err(authority_error(
                "INVALID_AUTHORITY_KEY",
                "PASETO v4.public secret keys are 64 nonzero bytes",
            ));
        }
        let secret = AsymmetricSecretKey::<V4>::from(bytes)
            .map_err(|_| authority_error("INVALID_AUTHORITY_KEY", "invalid signing key"))?;
        Ok(Self {
            issuer: issuer.to_owned(),
            key_id: key_id.to_owned(),
            secret,
        })
    }

    pub fn sign_for_request<R: AuthorityRequest>(
        &self,
        claims: &AuthorityClaims,
        request: &R,
    ) -> Result<SignedAuthorityEnvelope, WireError> {
        claims.validate()?;
        let binding = request.binding()?;
        let request_digest = request.digest()?;
        if claims.operation != R::OPERATION
            || claims.mutation_id != binding.mutation_id
            || claims.repository_id != binding.repository_id
            || claims.workspace_id != binding.workspace_id
            || claims.workspace_generation != binding.workspace_generation
            || claims.request_digest != request_digest
        {
            return Err(authority_error(
                "AUTHORITY_REQUEST_BINDING_MISMATCH",
                "authority claims do not match the exact validated request subject",
            ));
        }
        request.validate_claim_binding(claims)?;
        if claims.issuer != self.issuer {
            return Err(authority_error(
                "AUTHORITY_ISSUER_MISMATCH",
                "claims issuer does not match signing key issuer",
            ));
        }
        let payload = canonical_json(claims)?;
        let footer = canonical_json(&footer(&self.issuer, &self.key_id))?;
        let paseto = PublicToken::sign(
            &self.secret,
            &payload,
            Some(&footer),
            Some(AUTHORITY_IMPLICIT_ASSERTION),
        )
        .map_err(|_| authority_error("AUTHORITY_SIGNING_FAILED", "PASETO signing failed"))?;
        Ok(SignedAuthorityEnvelope {
            schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            paseto,
        })
    }
}

#[derive(Clone, Debug)]
pub struct AuthorityVerificationKey {
    issuer: String,
    key_id: String,
    public: AsymmetricPublicKey<V4>,
}

impl AuthorityVerificationKey {
    pub fn from_bytes(issuer: &str, key_id: &str, bytes: &[u8]) -> Result<Self, WireError> {
        validate_key_identity(issuer, key_id)?;
        if bytes.len() != 32 || bytes.iter().all(|byte| *byte == 0) {
            return Err(authority_error(
                "INVALID_AUTHORITY_KEY",
                "PASETO v4.public public keys are 32 nonzero bytes",
            ));
        }
        let public = AsymmetricPublicKey::<V4>::from(bytes)
            .map_err(|_| authority_error("INVALID_AUTHORITY_KEY", "invalid verification key"))?;
        Ok(Self {
            issuer: issuer.to_owned(),
            key_id: key_id.to_owned(),
            public,
        })
    }

    pub fn verify(
        &self,
        envelope: &SignedAuthorityEnvelope,
        expected: &AuthorityExpectation,
    ) -> Result<AuthorityClaims, WireError> {
        validate_envelope(envelope)?;
        if envelope.issuer != self.issuer || envelope.key_id != self.key_id {
            return Err(authority_error(
                "AUTHORITY_KEY_MISMATCH",
                "envelope issuer or key does not match the selected verification key",
            ));
        }
        let footer = canonical_json(&footer(&self.issuer, &self.key_id))?;
        let untrusted = UntrustedToken::<Public, V4>::try_from(envelope.paseto.as_str())
            .map_err(|_| authority_error("INVALID_AUTHORITY_TOKEN", "invalid PASETO framing"))?;
        let trusted = PublicToken::verify(
            &self.public,
            &untrusted,
            Some(&footer),
            Some(AUTHORITY_IMPLICIT_ASSERTION),
        )
        .map_err(|_| {
            authority_error(
                "INVALID_AUTHORITY_SIGNATURE",
                "PASETO signature, footer, or implicit assertion is invalid",
            )
        })?;
        let claims = decode_canonical::<AuthorityClaims>(trusted.payload().as_bytes())?;
        claims.validate()?;
        if claims.issuer != self.issuer {
            return Err(authority_error(
                "AUTHORITY_ISSUER_MISMATCH",
                "signed claims issuer does not match the selected key",
            ));
        }
        if claims.audience != expected.audience {
            return Err(authority_error(
                "AUTHORITY_AUDIENCE_MISMATCH",
                "authority audience does not match this gateway",
            ));
        }
        if claims.operation != expected.operation {
            return Err(authority_error(
                "AUTHORITY_OPERATION_MISMATCH",
                "authority operation does not match this request",
            ));
        }
        if claims.request_digest != expected.request_digest {
            return Err(authority_error(
                "AUTHORITY_REQUEST_MISMATCH",
                "authority request digest does not match the exact request",
            ));
        }
        if expected.now_unix_ms < claims.not_before_unix_ms {
            return Err(authority_error(
                "AUTHORITY_NOT_YET_VALID",
                "authority is not valid yet",
            ));
        }
        if expected.now_unix_ms >= claims.expires_at_unix_ms {
            return Err(authority_error(
                "AUTHORITY_EXPIRED",
                "authority has expired",
            ));
        }
        Ok(claims)
    }
}

fn footer(issuer: &str, key_id: &str) -> AuthorityFooter {
    AuthorityFooter {
        schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
        issuer: issuer.to_owned(),
        key_id: key_id.to_owned(),
        purpose: "authority-signing".to_owned(),
    }
}

fn validate_envelope(envelope: &SignedAuthorityEnvelope) -> Result<(), WireError> {
    if envelope.schema_version != AUTHORITY_SCHEMA_VERSION {
        return Err(authority_error(
            "UNSUPPORTED_AUTHORITY_SCHEMA",
            "authority envelope requires schema v1alpha1",
        ));
    }
    validate_key_identity(&envelope.issuer, &envelope.key_id)?;
    if !envelope.paseto.starts_with("v4.public.") || envelope.paseto.len() > 32_768 {
        return Err(authority_error(
            "INVALID_AUTHORITY_TOKEN",
            "authority token must be a bounded compact PASETO v4.public token",
        ));
    }
    Ok(())
}

fn validate_key_identity(issuer: &str, key_id: &str) -> Result<(), WireError> {
    validate_label("issuer", issuer)?;
    validate_label("key_id", key_id)
}

fn validate_label(name: &str, value: &str) -> Result<(), WireError> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')
        })
    {
        return Err(authority_error(
            "INVALID_AUTHORITY_LABEL",
            format!("{name} must be bounded printable identifier text"),
        ));
    }
    Ok(())
}

fn authority_error(code: &'static str, message: impl Into<String>) -> WireError {
    WireError::new(code, message)
}
