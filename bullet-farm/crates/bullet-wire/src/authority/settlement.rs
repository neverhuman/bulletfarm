use serde::{Deserialize, Serialize};

use super::{
    AUTHORITY_SCHEMA_VERSION, AuthorityVerificationKey, MutationOperation, MutationPermitClaims,
    MutationPermitSubject, ReplayDisposition, SignedMutationPermit, authority_error,
    validate_label,
};
use crate::{Blake3Digest, MutationId, MutationReservationId, WireError};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationOutcome {
    Committed,
    Aborted,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationSettlementRequest {
    pub schema_version: String,
    pub reservation_id: MutationReservationId,
    pub mutation_id: MutationId,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
    pub permit: SignedMutationPermit,
    pub permit_digest: Blake3Digest,
    pub outcome: MutationOutcome,
    pub result_digest: Blake3Digest,
    pub completed_at_unix_ms: u64,
}

impl MutationSettlementRequest {
    pub fn validate_shape(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        if self.completed_at_unix_ms > MAX_SAFE_INTEGER
            || self.permit.digest()? != self.permit_digest
        {
            return Err(authority_error(
                "INVALID_MUTATION_SETTLEMENT",
                "settlement time or permit digest is invalid",
            ));
        }
        Ok(())
    }

    pub fn verify_permit(
        &self,
        key: &AuthorityVerificationKey,
        expected: &MutationPermitSubject,
    ) -> Result<MutationPermitClaims, WireError> {
        self.validate_shape()?;
        if self.reservation_id != expected.reservation_id
            || self.mutation_id != expected.mutation_id
            || self.operation != expected.operation
            || self.request_digest != expected.request_digest
        {
            return Err(authority_error(
                "MUTATION_SETTLEMENT_SUBJECT_MISMATCH",
                "settlement does not bind the expected durable permit subject",
            ));
        }
        key.verify_mutation_permit_subject(&self.permit, expected)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SettlementStatus {
    Accepted,
    ExactReplay,
    Conflict,
    Refused,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationSettlementResult {
    pub schema_version: String,
    pub status: SettlementStatus,
    pub replay: ReplayDisposition,
    pub mutation_id: MutationId,
    pub reservation_id: MutationReservationId,
    pub result_digest: Option<Blake3Digest>,
    pub reason_code: Option<String>,
}

impl MutationSettlementResult {
    pub fn validate(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        if self
            .reason_code
            .as_ref()
            .is_some_and(|value| validate_label("reason_code", value).is_err())
        {
            return Err(authority_error(
                "INVALID_MUTATION_SETTLEMENT_RESULT",
                "settlement reason code is invalid",
            ));
        }
        let shape_is_valid = match self.status {
            SettlementStatus::Accepted => {
                self.replay == ReplayDisposition::Fresh
                    && self.result_digest.is_some()
                    && self.reason_code.is_none()
            }
            SettlementStatus::ExactReplay => {
                self.replay == ReplayDisposition::ExactReplay
                    && self.result_digest.is_some()
                    && self.reason_code.is_none()
            }
            SettlementStatus::Conflict => {
                self.replay == ReplayDisposition::Conflict
                    && self.result_digest.is_none()
                    && self.reason_code.is_some()
            }
            SettlementStatus::Refused => self.result_digest.is_none() && self.reason_code.is_some(),
        };
        if !shape_is_valid {
            return Err(authority_error(
                "INVALID_MUTATION_SETTLEMENT_RESULT",
                "settlement result fields do not match its status",
            ));
        }
        Ok(())
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
