use std::convert::TryFrom;

use pasetors::{
    Public,
    token::UntrustedToken,
    version4::{PublicToken, V4},
};
use serde::{Deserialize, Serialize};

use super::{
    AUTHORITY_SCHEMA_VERSION, AuthorityAudience, AuthoritySigningKey, AuthorityVerificationKey,
    MAX_SAFE_INTEGER, MutationOperation, authority_error, validate_key_identity, validate_label,
};
use crate::{
    AttemptId, Blake3Digest, MutationId, MutationReservationId, RepositoryId, WireError,
    WorkspaceId, canonical_json, decode_canonical, hash_framed_bytes,
};

mod subject;

pub const MUTATION_PERMIT_IMPLICIT_ASSERTION: &[u8] = b"bullet-farm.mutation-permit.v1alpha1";
pub const MAX_MUTATION_PERMIT_TTL_MS: u64 = 1_000;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationPermitClaims {
    pub schema_version: String,
    pub issuer: String,
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub authority_envelope_digest: Blake3Digest,
    pub authority_token_nonce: Blake3Digest,
    pub mutation_id: MutationId,
    pub reservation_id: MutationReservationId,
    pub request_digest: Blake3Digest,
    pub repository_id: RepositoryId,
    pub workspace_id: WorkspaceId,
    pub workspace_generation: u64,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
    pub issued_at_unix_ms: u64,
    pub not_before_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub permit_nonce: Blake3Digest,
}

impl MutationPermitClaims {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.schema_version != AUTHORITY_SCHEMA_VERSION {
            return Err(authority_error(
                "UNSUPPORTED_MUTATION_PERMIT_SCHEMA",
                "mutation permit claims require schema v1alpha1",
            ));
        }
        if self.audience != self.operation.required_audience() {
            return Err(authority_error(
                "INVALID_MUTATION_PERMIT_AUDIENCE",
                "mutation permit operation is not valid for the selected gateway audience",
            ));
        }
        validate_label("issuer", &self.issuer)?;
        for (name, value) in [
            ("workspace_generation", self.workspace_generation),
            ("attempt_fence", self.attempt_fence),
            ("authority_epoch", self.authority_epoch),
        ] {
            if value == 0 || value > MAX_SAFE_INTEGER {
                return Err(authority_error(
                    "INVALID_MUTATION_PERMIT_GENERATION",
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
                "INVALID_MUTATION_PERMIT_TIME",
                "permit time or freeze generation exceeds the interoperable integer range",
            ));
        }
        if self.issued_at_unix_ms > self.not_before_unix_ms
            || self.not_before_unix_ms >= self.expires_at_unix_ms
            || self.expires_at_unix_ms - self.issued_at_unix_ms > MAX_MUTATION_PERMIT_TTL_MS
        {
            return Err(authority_error(
                "INVALID_MUTATION_PERMIT_WINDOW",
                "permit requires issued_at <= not_before < expires_at and a TTL at most 1s",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedMutationPermit {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

impl SignedMutationPermit {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_permit_envelope(self)?;
        hash_framed_bytes("authority.permit-envelope.v1alpha1", self.paseto.as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationPermitExpectation {
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub authority_envelope_digest: Blake3Digest,
    pub authority_token_nonce: Blake3Digest,
    pub mutation_id: MutationId,
    pub reservation_id: MutationReservationId,
    pub request_digest: Blake3Digest,
    pub repository_id: RepositoryId,
    pub workspace_id: WorkspaceId,
    pub workspace_generation: u64,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationPermitSubject {
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub authority_envelope_digest: Blake3Digest,
    pub authority_token_nonce: Blake3Digest,
    pub mutation_id: MutationId,
    pub reservation_id: MutationReservationId,
    pub request_digest: Blake3Digest,
    pub repository_id: RepositoryId,
    pub workspace_id: WorkspaceId,
    pub workspace_generation: u64,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PermitFooter {
    schema_version: String,
    issuer: String,
    key_id: String,
    purpose: String,
}

impl AuthoritySigningKey {
    pub fn sign_mutation_permit(
        &self,
        claims: &MutationPermitClaims,
    ) -> Result<SignedMutationPermit, WireError> {
        claims.validate()?;
        if claims.issuer != self.issuer {
            return Err(authority_error(
                "MUTATION_PERMIT_ISSUER_MISMATCH",
                "permit issuer does not match signing key issuer",
            ));
        }
        let payload = canonical_json(claims)?;
        let footer = canonical_json(&permit_footer(&self.issuer, &self.key_id))?;
        let paseto = PublicToken::sign(
            &self.secret,
            &payload,
            Some(&footer),
            Some(MUTATION_PERMIT_IMPLICIT_ASSERTION),
        )
        .map_err(|_| authority_error("MUTATION_PERMIT_SIGNING_FAILED", "PASETO signing failed"))?;
        Ok(SignedMutationPermit {
            schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            paseto,
        })
    }
}

impl AuthorityVerificationKey {
    pub fn verify_mutation_permit(
        &self,
        permit: &SignedMutationPermit,
        expected: &MutationPermitExpectation,
    ) -> Result<MutationPermitClaims, WireError> {
        let claims = self.authenticate_mutation_permit(permit)?;
        verify_expected(&claims, expected)?;
        Ok(claims)
    }

    pub(super) fn verify_mutation_permit_subject(
        &self,
        permit: &SignedMutationPermit,
        expected: &MutationPermitSubject,
    ) -> Result<MutationPermitClaims, WireError> {
        let claims = self.authenticate_mutation_permit(permit)?;
        verify_subject(&claims, expected)?;
        Ok(claims)
    }

    fn authenticate_mutation_permit(
        &self,
        permit: &SignedMutationPermit,
    ) -> Result<MutationPermitClaims, WireError> {
        validate_permit_envelope(permit)?;
        if permit.issuer != self.issuer || permit.key_id != self.key_id {
            return Err(authority_error(
                "MUTATION_PERMIT_KEY_MISMATCH",
                "permit issuer or key does not match the selected verification key",
            ));
        }
        let footer = canonical_json(&permit_footer(&self.issuer, &self.key_id))?;
        let untrusted = UntrustedToken::<Public, V4>::try_from(permit.paseto.as_str())
            .map_err(|_| authority_error("INVALID_MUTATION_PERMIT", "invalid PASETO framing"))?;
        let trusted = PublicToken::verify(
            &self.public,
            &untrusted,
            Some(&footer),
            Some(MUTATION_PERMIT_IMPLICIT_ASSERTION),
        )
        .map_err(|_| {
            authority_error(
                "INVALID_MUTATION_PERMIT_SIGNATURE",
                "permit signature, footer, or implicit assertion is invalid",
            )
        })?;
        let claims = decode_canonical::<MutationPermitClaims>(trusted.payload().as_bytes())?;
        claims.validate()?;
        if claims.issuer != self.issuer {
            return Err(authority_error(
                "MUTATION_PERMIT_ISSUER_MISMATCH",
                "signed permit issuer does not match the selected key",
            ));
        }
        Ok(claims)
    }
}

fn verify_expected(
    claims: &MutationPermitClaims,
    expected: &MutationPermitExpectation,
) -> Result<(), WireError> {
    verify_subject(claims, &expected.subject())?;
    if expected.now_unix_ms < claims.not_before_unix_ms {
        return Err(authority_error(
            "MUTATION_PERMIT_NOT_YET_VALID",
            "mutation permit is not valid yet",
        ));
    }
    if expected.now_unix_ms >= claims.expires_at_unix_ms {
        return Err(authority_error(
            "MUTATION_PERMIT_EXPIRED",
            "mutation permit has expired",
        ));
    }
    Ok(())
}

fn verify_subject(
    claims: &MutationPermitClaims,
    expected: &MutationPermitSubject,
) -> Result<(), WireError> {
    if claims.audience != expected.audience
        || claims.operation != expected.operation
        || claims.authority_envelope_digest != expected.authority_envelope_digest
        || claims.authority_token_nonce != expected.authority_token_nonce
        || claims.mutation_id != expected.mutation_id
        || claims.reservation_id != expected.reservation_id
        || claims.request_digest != expected.request_digest
        || claims.repository_id != expected.repository_id
        || claims.workspace_id != expected.workspace_id
        || claims.workspace_generation != expected.workspace_generation
        || claims.attempt_id != expected.attempt_id
        || claims.attempt_fence != expected.attempt_fence
        || claims.authority_epoch != expected.authority_epoch
        || claims.freeze_generation != expected.freeze_generation
    {
        return Err(authority_error(
            "MUTATION_PERMIT_SUBJECT_MISMATCH",
            "permit does not bind the exact reserved mutation subject",
        ));
    }
    Ok(())
}

fn permit_footer(issuer: &str, key_id: &str) -> PermitFooter {
    PermitFooter {
        schema_version: AUTHORITY_SCHEMA_VERSION.to_owned(),
        issuer: issuer.to_owned(),
        key_id: key_id.to_owned(),
        purpose: "mutation-permit-signing".to_owned(),
    }
}

fn validate_permit_envelope(permit: &SignedMutationPermit) -> Result<(), WireError> {
    if permit.schema_version != AUTHORITY_SCHEMA_VERSION {
        return Err(authority_error(
            "UNSUPPORTED_MUTATION_PERMIT_SCHEMA",
            "mutation permit envelope requires schema v1alpha1",
        ));
    }
    validate_key_identity(&permit.issuer, &permit.key_id)?;
    if !permit.paseto.starts_with("v4.public.") || permit.paseto.len() > 32_768 {
        return Err(authority_error(
            "INVALID_MUTATION_PERMIT",
            "mutation permit must be a bounded compact PASETO v4.public token",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorityDecisionKind {
    Authorized,
    Settled,
    Refused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayDisposition {
    Fresh,
    ExactReplay,
    Conflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationResultState {
    InFlight,
    Committed,
    Aborted,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationReplayResult {
    pub schema_version: String,
    pub reservation_id: MutationReservationId,
    pub mutation_id: MutationId,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
    pub state: MutationResultState,
    pub result_digest: Option<Blake3Digest>,
    pub completed_at_unix_ms: Option<u64>,
}

impl MutationReplayResult {
    pub fn validate(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        let complete = matches!(
            self.state,
            MutationResultState::Committed
                | MutationResultState::Aborted
                | MutationResultState::Unknown
        );
        if complete != self.result_digest.is_some()
            || complete != self.completed_at_unix_ms.is_some()
            || self
                .completed_at_unix_ms
                .is_some_and(|value| value > MAX_SAFE_INTEGER)
        {
            return Err(authority_error(
                "INVALID_MUTATION_REPLAY_RESULT",
                "completed replay results require both result digest and completion time",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalAuthorityDecision {
    pub schema_version: String,
    pub decision: AuthorityDecisionKind,
    pub replay: ReplayDisposition,
    pub mutation_id: MutationId,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
    pub reservation_id: Option<MutationReservationId>,
    pub permit: Option<SignedMutationPermit>,
    pub replay_result: Option<MutationReplayResult>,
    pub reason_code: Option<String>,
}

impl FinalAuthorityDecision {
    pub fn validate_shape(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        if self
            .reason_code
            .as_ref()
            .is_some_and(|value| validate_label("reason_code", value).is_err())
        {
            return Err(authority_error(
                "INVALID_AUTHORITY_DECISION",
                "authority refusal reason is invalid",
            ));
        }
        let shape_is_valid = match self.decision {
            AuthorityDecisionKind::Authorized => {
                matches!(
                    self.replay,
                    ReplayDisposition::Fresh | ReplayDisposition::ExactReplay
                ) && self.reservation_id.is_some()
                    && self.permit.is_some()
                    && self.replay_result.is_none()
                    && self.reason_code.is_none()
            }
            AuthorityDecisionKind::Settled => {
                self.replay == ReplayDisposition::ExactReplay
                    && self.reservation_id.is_some()
                    && self.permit.is_none()
                    && self.replay_result.is_some()
                    && self.reason_code.is_none()
            }
            AuthorityDecisionKind::Refused => {
                self.reservation_id.is_none()
                    && self.permit.is_none()
                    && self.replay_result.is_none()
                    && self.reason_code.is_some()
            }
        };
        if !shape_is_valid {
            return Err(authority_error(
                "INVALID_AUTHORITY_DECISION",
                "authority decision fields do not match its decision and replay state",
            ));
        }
        if let Some(permit) = &self.permit {
            permit.digest()?;
        }
        if let Some(result) = &self.replay_result {
            result.validate()?;
            if result.mutation_id != self.mutation_id
                || result.operation != self.operation
                || result.request_digest != self.request_digest
                || Some(result.reservation_id.clone()) != self.reservation_id
            {
                return Err(authority_error(
                    "AUTHORITY_REPLAY_CONFLICT",
                    "replay result does not match the authority decision subject",
                ));
            }
        }
        Ok(())
    }

    pub fn verify_authorized_permit(
        &self,
        key: &AuthorityVerificationKey,
        expected: &MutationPermitExpectation,
    ) -> Result<MutationPermitClaims, WireError> {
        self.validate_shape()?;
        if self.decision != AuthorityDecisionKind::Authorized
            || self.mutation_id != expected.mutation_id
            || self.operation != expected.operation
            || self.request_digest != expected.request_digest
            || self.reservation_id.as_ref() != Some(&expected.reservation_id)
        {
            return Err(authority_error(
                "AUTHORITY_DECISION_SUBJECT_MISMATCH",
                "authorized decision does not bind the expected mutation permit subject",
            ));
        }
        key.verify_mutation_permit(
            self.permit.as_ref().ok_or_else(|| {
                authority_error(
                    "INVALID_AUTHORITY_DECISION",
                    "authorized decision is missing its mutation permit",
                )
            })?,
            expected,
        )
    }
}

fn require_schema(actual: &str) -> Result<(), WireError> {
    if actual != AUTHORITY_SCHEMA_VERSION {
        return Err(authority_error(
            "UNSUPPORTED_AUTHORITY_SCHEMA",
            "authority protocol requires schema v1alpha1",
        ));
    }
    Ok(())
}
