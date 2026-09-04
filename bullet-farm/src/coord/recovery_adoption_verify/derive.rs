use std::{collections::BTreeMap, path::Path};

use super::{ForensicSources, evidence_subject_fields, forensic, git, mismatch};
use crate::coord::{
    CoordError,
    generation::manifest::RecoveryManifestBody,
    model::{
        ClaimSummary, Record, RecoveryProductionPlanV1, RecoveryProductionSubjectV1,
        RecoveryProductionWatermarkV1,
    },
    state,
};

pub(in crate::coord) fn derive_plan(
    family_root: &Path,
    manifest: &RecoveryManifestBody,
    records: &[Record],
    sources: ForensicSources<'_>,
    expected_watermark: RecoveryProductionWatermarkV1,
    recovery_orchestrator: String,
) -> Result<RecoveryProductionPlanV1, CoordError> {
    let claims = state::summaries(records, manifest.recovered_at_unix_ms)?;
    derive_plan_with_claims(
        family_root,
        manifest,
        &claims,
        sources,
        expected_watermark,
        recovery_orchestrator,
    )
}

fn derive_plan_with_claims(
    family_root: &Path,
    manifest: &RecoveryManifestBody,
    claims: &BTreeMap<String, ClaimSummary>,
    sources: ForensicSources<'_>,
    expected_watermark: RecoveryProductionWatermarkV1,
    recovery_orchestrator: String,
) -> Result<RecoveryProductionPlanV1, CoordError> {
    crate::coord::validate_field("recovery_orchestrator", &recovery_orchestrator)?;
    let candidate = forensic::derive_next(manifest, claims, sources)?;
    if recovery_orchestrator == candidate.quarantined_orchestrator {
        return Err(mismatch(
            "fresh recovery orchestrator equals the quarantined receipt orchestrator",
        ));
    }
    let git_expectation = git::derive_recovery_commit(
        family_root,
        &candidate.repo,
        &candidate.commit_oid,
        &candidate.parent_receipts,
    )?;
    let subject = RecoveryProductionSubjectV1 {
        repo: candidate.repo,
        git_expectation,
        claims: candidate.claims,
        group_receipt_observation: candidate.group_receipt_observation,
    };
    let evidence_subject_blake3 = evidence_subject_fields(
        &subject.repo,
        &subject.git_expectation,
        &subject.claims,
        &subject.group_receipt_observation,
    )?;
    RecoveryProductionPlanV1::derive(
        evidence_subject_blake3,
        expected_watermark,
        recovery_orchestrator,
        subject,
    )
    .map_err(|error| mismatch(error.to_string()))
}
