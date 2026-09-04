use std::collections::BTreeMap;

use super::validate_receipt_coverage;
use crate::coord::{
    CoordError,
    model::{
        ClaimState, ClaimSummary, RecoveryAdoptionAuthorityClassV1, RecoveryAdoptionSummaryV1,
        RecoveryBaselineBody, RecoveryReceiptAdoptionRecordV1,
    },
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RecoveryAdoptionAuthority {
    generation_id: String,
    manifest_blake3: String,
    recovered_at_unix_ms: u64,
    frozen_claims: BTreeMap<String, String>,
}

impl RecoveryAdoptionAuthority {
    pub(super) fn from_baseline(generation_id: &str, body: &RecoveryBaselineBody) -> Self {
        Self {
            generation_id: generation_id.to_owned(),
            manifest_blake3: body.manifest_blake3.clone(),
            recovered_at_unix_ms: body.recovered_at_unix_ms,
            frozen_claims: body
                .frozen_claims
                .iter()
                .map(|claim| (claim.claim_id.clone(), claim.claim_blake3.clone()))
                .collect(),
        }
    }

    pub(super) fn validate_time(&self, at_unix_ms: u64) -> Result<(), CoordError> {
        if at_unix_ms == 0
            || at_unix_ms > MAX_SAFE_INTEGER
            || at_unix_ms < self.recovered_at_unix_ms
        {
            return Err(error(
                "RECOVERY_AUTHORITY_INSUFFICIENT",
                "recovery evidence time is outside the admitted recovery generation",
            ));
        }
        Ok(())
    }
}

pub(super) fn apply(
    at_unix_ms: u64,
    body: &RecoveryReceiptAdoptionRecordV1,
    authority: Option<&RecoveryAdoptionAuthority>,
    claims: &mut BTreeMap<String, ClaimSummary>,
) -> Result<(), CoordError> {
    body.validate()?;
    let authority = authority.ok_or_else(|| {
        error(
            "RECOVERY_AUTHORITY_INSUFFICIENT",
            "receipt adoption requires an admitted recovery baseline",
        )
    })?;
    validate_authority(at_unix_ms, body, authority)?;
    reject_existing_adoption(body, claims)?;

    let mut staged = claims.clone();
    let request = body.request();
    for requested in &request.subject.claims {
        let claim = staged.get_mut(&requested.claim_id).ok_or_else(|| {
            error(
                "RECOVERY_CLAIM_NOT_FROZEN",
                format!(
                    "recovery adoption references missing claim {}",
                    requested.claim_id
                ),
            )
        })?;
        if claim.recovery_adoption.is_some() || claim.state == ClaimState::RecoveredReceipted {
            return Err(error(
                "RECOVERY_ADOPTION_CONFLICT",
                format!(
                    "claim {} already has recovery adoption provenance",
                    requested.claim_id
                ),
            ));
        }
        if claim.state != ClaimState::FrozenRecovery {
            return Err(error(
                "RECOVERY_CLAIM_NOT_FROZEN",
                format!("claim {} is not frozen for recovery", requested.claim_id),
            ));
        }
        if claim.repo != request.subject.repo {
            return Err(evidence(format!(
                "claim {} belongs to repository {}",
                requested.claim_id, claim.repo
            )));
        }
        let expected_digest = authority
            .frozen_claims
            .get(&requested.claim_id)
            .ok_or_else(|| evidence("claim is absent from the recovery freeze authority"))?;
        if expected_digest != &requested.frozen_claim_blake3 {
            return Err(evidence(format!(
                "claim {} frozen digest differs from recovery authority",
                requested.claim_id
            )));
        }
        if claim.proof_command.is_some()
            || !claim.changed_paths.is_empty()
            || claim.commit_oid.is_some()
            || claim.commit_orchestrator.is_some()
            || claim.commit_recorded_at_unix_ms.is_some()
        {
            return Err(evidence(format!(
                "claim {} has ordinary receipt provenance",
                requested.claim_id
            )));
        }
        validate_receipt_coverage(&claim.paths, &requested.committed_paths)
            .map_err(|error| evidence(error.to_string()))?;
        if at_unix_ms < claim.last_event_unix_ms {
            return Err(error(
                "RECOVERY_AUTHORITY_INSUFFICIENT",
                format!(
                    "claim {} adoption time precedes trusted state",
                    requested.claim_id
                ),
            ));
        }

        claim.last_event_unix_ms = at_unix_ms;
        claim.state = ClaimState::RecoveredReceipted;
        claim.recovery_adoption = Some(summary(at_unix_ms, body));
    }
    *claims = staged;
    Ok(())
}

fn reject_existing_adoption(
    body: &RecoveryReceiptAdoptionRecordV1,
    claims: &BTreeMap<String, ClaimSummary>,
) -> Result<(), CoordError> {
    let subject = &body.request().subject;
    for claim in claims.values() {
        let Some(existing) = &claim.recovery_adoption else {
            continue;
        };
        if existing.adoption_id == body.adoption_id()
            || (claim.repo == subject.repo
                && existing.commit_oid == subject.git_expectation.commit_oid)
        {
            return Err(error(
                "RECOVERY_ADOPTION_CONFLICT",
                "repository commit already has recovery adoption provenance",
            ));
        }
    }
    Ok(())
}

fn validate_authority(
    at_unix_ms: u64,
    body: &RecoveryReceiptAdoptionRecordV1,
    authority: &RecoveryAdoptionAuthority,
) -> Result<(), CoordError> {
    let watermark = &body.request().expected_watermark;
    if watermark.generation_id != authority.generation_id
        || watermark.manifest_blake3 != authority.manifest_blake3
    {
        return Err(error(
            "STALE_COORD_GENERATION",
            "recovery adoption generation or manifest is no longer current",
        ));
    }
    authority.validate_time(at_unix_ms)
}

fn summary(at_unix_ms: u64, body: &RecoveryReceiptAdoptionRecordV1) -> RecoveryAdoptionSummaryV1 {
    RecoveryAdoptionSummaryV1 {
        adoption_id: body.adoption_id().to_owned(),
        generation_id: body.request().expected_watermark.generation_id.clone(),
        request_id: body.request().request_id.as_str().to_owned(),
        request_subject_blake3: body.request_subject_blake3().to_owned(),
        commit_oid: body.request().subject.git_expectation.commit_oid.clone(),
        tree_oid: body
            .request()
            .subject
            .git_expectation
            .result_tree_oid
            .clone(),
        adopted_at_unix_ms: at_unix_ms,
        proof_subject_blake3: body.proof_subject_blake3().to_owned(),
        review_subject_blake3: body.review_subject_blake3().to_owned(),
        authority_class: RecoveryAdoptionAuthorityClassV1::LocalOsAuthority,
    }
}

fn evidence(reason: impl Into<String>) -> CoordError {
    error("RECOVERY_EVIDENCE_MISMATCH", reason)
}

fn error(code: &'static str, reason: impl Into<String>) -> CoordError {
    CoordError::new(code, reason)
}

#[cfg(test)]
#[path = "recovery_adoption/tests.rs"]
mod tests;
