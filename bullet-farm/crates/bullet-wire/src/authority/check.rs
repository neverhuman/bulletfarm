use serde::{Deserialize, Serialize};

use super::{
    AUTHORITY_SCHEMA_VERSION, AuthorityAudience, AuthorityClaims, AuthorityExpectation,
    AuthorityVerificationKey, MutationOperation, SignedAuthorityEnvelope, authority_error,
};
use crate::{Blake3Digest, MutationId, WireError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalAuthorityCheckRequest {
    pub schema_version: String,
    pub envelope: SignedAuthorityEnvelope,
    pub envelope_digest: Blake3Digest,
    pub mutation_id: MutationId,
    pub audience: AuthorityAudience,
    pub operation: MutationOperation,
    pub request_digest: Blake3Digest,
}

impl FinalAuthorityCheckRequest {
    pub fn verify(
        &self,
        key: &AuthorityVerificationKey,
        now_unix_ms: u64,
    ) -> Result<AuthorityClaims, WireError> {
        if self.schema_version != AUTHORITY_SCHEMA_VERSION {
            return Err(authority_error(
                "UNSUPPORTED_AUTHORITY_SCHEMA",
                "final authority check requires schema v1alpha1",
            ));
        }
        if self.envelope.digest()? != self.envelope_digest {
            return Err(authority_error(
                "AUTHORITY_ENVELOPE_DIGEST_MISMATCH",
                "final check does not bind the exact authority envelope",
            ));
        }
        let claims = key.verify(
            &self.envelope,
            &AuthorityExpectation {
                audience: self.audience,
                operation: self.operation,
                request_digest: self.request_digest,
                now_unix_ms,
            },
        )?;
        if claims.mutation_id != self.mutation_id {
            return Err(authority_error(
                "AUTHORITY_MUTATION_MISMATCH",
                "final check Mutation ID does not match signed claims",
            ));
        }
        Ok(claims)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
pub enum PreservationDecision {
    PreserveRequired {
        reason: String,
    },
    CleanupAuthorized {
        preservation_receipt_digest: Blake3Digest,
        expected_destination_digest: Blake3Digest,
    },
    CleanupDenied {
        reason: String,
    },
}
