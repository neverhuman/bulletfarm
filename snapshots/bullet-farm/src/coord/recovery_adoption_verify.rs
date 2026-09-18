use std::{collections::BTreeMap, path::Path};

use serde::Serialize;

use self::{forensic::ForensicOutcome, generation::GenerationEvidenceOutcome};
use super::{
    CoordError,
    generation::{manifest::RecoveryManifestBody, segment::StoredEnvelope},
    git,
    model::{ClaimState, ClaimSummary, Record, RecoveryReceiptAdoptionRequestV1},
    state,
};

mod derive;
mod forensic;
mod generation;

pub(super) use derive::derive_plan;

pub(super) struct ForensicSources<'a> {
    pub(super) trusted_prefix: &'a [u8],
    pub(super) frozen_live_source: &'a [u8],
}

pub(super) struct VerifiedAdoptionEvidence {
    pub(super) recovery_orchestrator: String,
    pub(super) reviewer: String,
}

#[derive(Serialize)]
struct EvidenceSubject<'a> {
    repo: &'a str,
    git_expectation: &'a super::model::RecoveryGitExpectationV1,
    claims: &'a [super::model::RecoveryAdoptionClaimV1],
    group_receipt_observation: &'a super::model::ForensicRecordRefV1,
}

pub(super) fn verify(
    family_root: &Path,
    request: &RecoveryReceiptAdoptionRequestV1,
    manifest: &RecoveryManifestBody,
    records: &[Record],
    entries: &[StoredEnvelope],
    sources: ForensicSources<'_>,
) -> Result<VerifiedAdoptionEvidence, CoordError> {
    request.validate()?;
    let claims = current_frozen_claims(request, manifest, records)?;
    let expected_subject = evidence_subject(request)?;
    let forensic = forensic::verify(request, manifest, &claims, sources)?;
    let generation = generation::verify(request, entries, &expected_subject)?;
    verify_identity_relation(&forensic, &generation)?;
    git::verify_recovery_commit(
        family_root,
        &request.subject.repo,
        &request.subject.git_expectation,
    )?;
    Ok(VerifiedAdoptionEvidence {
        recovery_orchestrator: generation.recovery_orchestrator,
        reviewer: generation.reviewer,
    })
}

pub(super) fn evidence_subject(
    request: &RecoveryReceiptAdoptionRequestV1,
) -> Result<String, CoordError> {
    request.validate()?;
    evidence_subject_fields(
        &request.subject.repo,
        &request.subject.git_expectation,
        &request.subject.claims,
        &request.subject.group_receipt_observation,
    )
}

pub(super) fn evidence_subject_fields(
    repo: &str,
    git_expectation: &super::model::RecoveryGitExpectationV1,
    claims: &[super::model::RecoveryAdoptionClaimV1],
    group_receipt_observation: &super::model::ForensicRecordRefV1,
) -> Result<String, CoordError> {
    let subject = EvidenceSubject {
        repo,
        git_expectation,
        claims,
        group_receipt_observation,
    };
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_canonical(
            "bullet-family.coord.recovery-adoption-evidence-subject.v1",
            &subject,
        )
        .map_err(wire)?
        .to_hex()
    ))
}

pub(super) fn verify_replay_evidence(
    adoption: &super::model::RecoveryReceiptAdoptionRecordV1,
    entries: &[StoredEnvelope],
) -> Result<(), CoordError> {
    adoption.validate()?;
    let request = adoption.request();
    let expected_subject = evidence_subject(request)?;
    let verified = generation::verify(request, entries, &expected_subject)?;
    if verified.recovery_orchestrator != adoption.verified_orchestrator()
        || verified.reviewer != adoption.verified_reviewer()
    {
        return Err(mismatch(
            "stored adoption actors differ from exact proof/review evidence",
        ));
    }
    Ok(())
}

fn current_frozen_claims(
    request: &RecoveryReceiptAdoptionRequestV1,
    manifest: &RecoveryManifestBody,
    records: &[Record],
) -> Result<BTreeMap<String, ClaimSummary>, CoordError> {
    let claims = state::summaries(records, manifest.recovered_at_unix_ms)?;
    for requested in &request.subject.claims {
        let claim = claims.get(&requested.claim_id).ok_or_else(|| {
            mismatch(format!(
                "recovery request references missing claim {}",
                requested.claim_id
            ))
        })?;
        if claim.state != ClaimState::FrozenRecovery
            || claim.recovery_adoption.is_some()
            || claim.repo != request.subject.repo
        {
            return Err(mismatch(format!(
                "claim {} is not an unadopted frozen claim for repository {}",
                requested.claim_id, request.subject.repo
            )));
        }
        let frozen = manifest
            .frozen_claims
            .iter()
            .find(|frozen| frozen.claim_id == requested.claim_id)
            .ok_or_else(|| mismatch("claim is absent from recovery manifest authority"))?;
        if frozen.claim_blake3 != requested.frozen_claim_blake3 {
            return Err(mismatch(format!(
                "claim {} digest differs from recovery manifest authority",
                requested.claim_id
            )));
        }
        state::validate_receipt_coverage(&claim.paths, &requested.committed_paths)
            .map_err(|error| mismatch(error.to_string()))?;
    }
    Ok(claims)
}

fn verify_identity_relation(
    forensic: &ForensicOutcome,
    generation: &GenerationEvidenceOutcome,
) -> Result<(), CoordError> {
    if forensic.quarantined_orchestrator == generation.recovery_orchestrator {
        return Err(mismatch(
            "fresh recovery orchestrator must differ from quarantined receipt orchestrator",
        ));
    }
    Ok(())
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    mismatch(format!("cannot derive recovery evidence subject: {error}"))
}

fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_EVIDENCE_MISMATCH", reason)
}
