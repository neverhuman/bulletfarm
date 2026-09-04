//! Fixed-purpose PASETO authentication for one exact terminal dogfood run.
//!
//! Policy custody, trusted time, durable admission, and replay state remain consumer
//! responsibilities. This component authenticates immutable W5 facts only.

use serde::{Deserialize, Serialize};

use super::{
    DOGFOOD_SCHEMA_VERSION, DogfoodRunBindingSubjects, DogfoodRunV1, decode_dogfood_run,
    grant_signing::{
        PurposeSeparatedPasetoSigningKey, PurposeSeparatedPasetoVerificationKey, valid_key_identity,
    },
    verify_dogfood_run_binding,
};
use crate::{
    Blake3Digest, DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE, PolicySnapshotV1, PrincipalId,
    WireError, canonical_json, hash_framed_bytes, policy_snapshot_digest,
};

pub const DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION: &[u8] =
    b"bullet-farm.dogfood-run-attestation.v1alpha1";
pub const DOGFOOD_RUN_ATTESTATION_ENVELOPE_DOMAIN: &str =
    "dogfood.run-attestation-envelope.v1alpha1";
pub const MAX_DOGFOOD_RUN_ATTESTATION_TOKEN_BYTES: usize = 96 * 1024;
pub const MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS: u64 = 300_000;

/// Dedicated signer for a typed terminal-run attestor principal.
pub struct DogfoodRunAttestationSigningKey(PurposeSeparatedPasetoSigningKey);

impl DogfoodRunAttestationSigningKey {
    pub fn from_bytes(
        attestor_principal_id: &PrincipalId,
        key_id: &str,
        bytes: &[u8],
    ) -> Result<Self, WireError> {
        PurposeSeparatedPasetoSigningKey::from_bytes(attestor_principal_id.as_str(), key_id, bytes)
            .map(Self)
            .ok_or_else(|| {
                error(
                    "INVALID_DOGFOOD_RUN_ATTESTATION_KEY",
                    "run-attestation signing keys require bounded identity and 64 nonzero bytes",
                )
            })
    }

    pub fn sign(
        &self,
        run: &DogfoodRunV1,
        subjects: &DogfoodRunBindingSubjects<'_>,
    ) -> Result<SignedDogfoodRunV1, WireError> {
        run.digest()?;
        verify_dogfood_run_binding(run, subjects)?;
        if run.attestor_principal_id.as_str() != self.0.issuer() {
            return Err(error(
                "DOGFOOD_RUN_ATTESTOR_MISMATCH",
                "terminal run attestor does not name the signing principal",
            ));
        }
        let payload = canonical_json(run)?;
        let footer = canonical_json(&footer(self.0.issuer(), self.0.key_id()))?;
        let paseto = self
            .0
            .sign(
                &payload,
                &footer,
                DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
            )
            .ok_or_else(|| {
                error(
                    "DOGFOOD_RUN_ATTESTATION_SIGNING_FAILED",
                    "terminal run PASETO signing failed",
                )
            })?;
        Ok(SignedDogfoodRunV1 {
            schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
            issuer: self.0.issuer().to_owned(),
            key_id: self.0.key_id().to_owned(),
            paseto,
        })
    }
}

/// Strict signed envelope for the exact canonical [`DogfoodRunV1`] payload.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedDogfoodRunV1 {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub paseto: String,
}

impl SignedDogfoodRunV1 {
    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        validate_envelope(self)?;
        hash_framed_bytes(
            DOGFOOD_RUN_ATTESTATION_ENVELOPE_DOMAIN,
            self.paseto.as_bytes(),
        )
    }

    pub fn verify(
        &self,
        policy: &PolicySnapshotV1,
        subjects: &DogfoodRunBindingSubjects<'_>,
        trusted_now_unix_ms: u64,
    ) -> Result<DogfoodRunV1, WireError> {
        let principal = validate_envelope(self)?;
        let selected =
            policy.dogfood_run_attestor_key_at(&principal, &self.key_id, trusted_now_unix_ms)?;
        let key = PurposeSeparatedPasetoVerificationKey::from_lower_hex(
            &selected.issuer,
            &selected.key_id,
            &selected.public_key,
        )
        .ok_or_else(|| {
            error(
                "INVALID_DOGFOOD_RUN_ATTESTATION_KEY",
                "validated policy key could not construct a terminal-run verifier",
            )
        })?;
        let footer = canonical_json(&footer(key.issuer(), key.key_id()))?;
        let payload = key
            .authenticate(
                &self.paseto,
                &footer,
                DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
            )
            .ok_or_else(|| {
                error(
                    "DOGFOOD_RUN_ATTESTATION_INVALID",
                    "terminal run PASETO framing, signature, footer, or assertion is invalid",
                )
            })?;
        let run = decode_dogfood_run(&payload)?;
        if run.attestor_principal_id != principal {
            return Err(error(
                "DOGFOOD_RUN_ATTESTOR_MISMATCH",
                "signed terminal run does not name the selected attestor principal",
            ));
        }
        verify_dogfood_run_binding(&run, subjects)?;
        let loaded_policy_digest = policy_snapshot_digest(&canonical_json(policy)?)?;
        if run.subject.policy.policy_snapshot_digest != loaded_policy_digest
            || run.subject.policy.policy_generation != policy.policy_generation
        {
            return Err(error(
                "DOGFOOD_RUN_POLICY_MISMATCH",
                "terminal run does not bind the exact loaded policy snapshot",
            ));
        }
        if trusted_now_unix_ms < run.attested_at_unix_ms {
            return Err(error(
                "DOGFOOD_RUN_ATTESTATION_IN_FUTURE",
                "terminal run attestation is later than the supplied trusted instant",
            ));
        }
        if trusted_now_unix_ms - run.attested_at_unix_ms > MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS {
            return Err(error(
                "DOGFOOD_RUN_ATTESTATION_STALE",
                "terminal run attestation exceeds the five-minute admission age",
            ));
        }
        policy.dogfood_run_attestor_key_at(
            &run.attestor_principal_id,
            &self.key_id,
            run.attested_at_unix_ms,
        )?;
        Ok(run)
    }
}

#[derive(Serialize)]
struct DogfoodRunAttestationFooter<'a> {
    schema_version: &'static str,
    issuer: &'a str,
    key_id: &'a str,
    purpose: &'static str,
}

fn footer<'a>(issuer: &'a str, key_id: &'a str) -> DogfoodRunAttestationFooter<'a> {
    DogfoodRunAttestationFooter {
        schema_version: DOGFOOD_SCHEMA_VERSION,
        issuer,
        key_id,
        purpose: DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
    }
}

fn validate_envelope(envelope: &SignedDogfoodRunV1) -> Result<PrincipalId, WireError> {
    let principal = PrincipalId::parse_checked(&envelope.issuer).map_err(|_| {
        error(
            "DOGFOOD_RUN_ATTESTATION_INVALID",
            "terminal run envelope issuer must be a full-width principal ID",
        )
    })?;
    if envelope.schema_version != DOGFOOD_SCHEMA_VERSION
        || !valid_key_identity(principal.as_str(), &envelope.key_id)
        || !envelope.paseto.starts_with("v4.public.")
        || envelope.paseto.len() > MAX_DOGFOOD_RUN_ATTESTATION_TOKEN_BYTES
    {
        return Err(error(
            "DOGFOOD_RUN_ATTESTATION_INVALID",
            "terminal run envelope requires exact schema, identity, and bounded v4.public token",
        ));
    }
    Ok(principal)
}

fn error(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}

#[cfg(test)]
mod tests;
