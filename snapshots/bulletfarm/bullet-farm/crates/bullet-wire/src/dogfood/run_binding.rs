use crate::{
    PatchMutation, PatchProposal, ProviderRuntimePassportV1, RuntimePassportError, WireError,
    canonical_json, hash_framed_bytes,
};

use super::{
    DogfoodBudgetReservationV1, DogfoodCleanupObservationV1, DogfoodLaunchGrantClaimsV1,
    DogfoodProposalObservationV1, DogfoodReadOnlyIntentV1, DogfoodRunV1, DogfoodTerminalStateV1,
    MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES, ProviderCredentialProjectionV1,
    ProviderEndpointObservationV1, ProviderEnrollmentClaimsV2, ProviderProbeObservationV1,
    ProviderProfileObservationV1, ProviderVersionObservationV1, RepositoryContextPostObservationV1,
    RepositoryContextSnapshotV1, verify_dogfood_budget_binding, verify_dogfood_runtime_binding,
    verify_dogfood_subjects, verify_provider_observations, verify_repository_context_binding,
    verify_repository_context_post_observation,
};

pub const DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN: &str = "dogfood.patch-proposal.v1alpha1";
pub const DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN: &str =
    "dogfood.patch-proposal-artifact.v1alpha1";
pub const MAX_DOGFOOD_PATCH_OPERATIONS: usize = 128;
pub const MAX_DOGFOOD_PATCH_CONTENT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy)]
pub struct DogfoodRunBindingSubjects<'a> {
    pub grant: &'a DogfoodLaunchGrantClaimsV1,
    pub intent: &'a DogfoodReadOnlyIntentV1,
    pub enrollment: &'a ProviderEnrollmentClaimsV2,
    pub passport: &'a ProviderRuntimePassportV1,
    pub projection: &'a ProviderCredentialProjectionV1,
    pub reservation: &'a DogfoodBudgetReservationV1,
    pub context_snapshot: &'a RepositoryContextSnapshotV1,
    pub post_context: &'a RepositoryContextPostObservationV1,
    pub probe: &'a ProviderProbeObservationV1,
    pub endpoint: &'a ProviderEndpointObservationV1,
    pub version: &'a ProviderVersionObservationV1,
    pub profile: &'a ProviderProfileObservationV1,
    pub proposal: Option<&'a PatchProposal>,
}

pub fn verify_dogfood_run_binding(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    prevalidate(run, subjects)?;
    verify_dogfood_subjects(subjects.grant, subjects.intent, subjects.enrollment)?;
    verify_dogfood_runtime_binding(
        subjects.grant,
        subjects.intent,
        subjects.enrollment,
        subjects.passport,
    )?;
    verify_dogfood_budget_binding(subjects.reservation, subjects.intent, subjects.enrollment)?;
    verify_repository_context_binding(subjects.intent, subjects.context_snapshot)?;
    verify_repository_context_post_observation(subjects.context_snapshot, subjects.post_context)?;
    verify_provider_observations(
        subjects.enrollment,
        subjects.passport,
        subjects.probe,
        subjects.endpoint,
        subjects.version,
        subjects.profile,
    )?;
    verify_run_subject(run, subjects)?;
    verify_projection(run, subjects)?;
    run.budget_settlement
        .validate_against(subjects.reservation)?;
    verify_observation_digests(run, subjects)?;
    verify_times(run, subjects)?;
    verify_proposal(run, subjects)
}

fn prevalidate(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    run.validate()?;
    subjects.intent.validate()?;
    subjects.grant.validate()?;
    subjects.enrollment.validate()?;
    subjects.passport.validate().map_err(runtime_error)?;
    subjects.projection.validate()?;
    subjects.reservation.validate()?;
    subjects.context_snapshot.validate()?;
    subjects.post_context.validate()?;
    subjects.probe.validate()?;
    subjects.endpoint.validate()?;
    subjects.version.validate()?;
    subjects.profile.validate()?;
    if let Some(proposal) = subjects.proposal {
        proposal.validate()?;
    }
    Ok(())
}

fn verify_run_subject(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    if run.subject != subjects.intent.subject
        || run.intent_id != subjects.intent.intent_id()?
        || run.launch_grant_id != subjects.grant.grant_id()?
    {
        return Err(mismatch(
            "DOGFOOD_RUN_SUBJECT_MISMATCH",
            "run does not bind the exact intent, grant, and complete subject",
        ));
    }
    Ok(())
}

fn verify_projection(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    let projection = subjects.projection;
    let provider = &run.subject.provider;
    if projection.projection_instance_id != provider.credential_projection_id
        || projection.credential_projection_profile_id
            != subjects.enrollment.credential_projection_profile_id
        || projection.run_id != run.subject.execution.run_id
        || projection.provider != provider.provider
        || projection.provider != subjects.enrollment.provider
        || projection.service_identity_id != subjects.enrollment.service_identity_id
        || projection.projection_digest()? != run.credential_projection_digest
    {
        return Err(mismatch(
            "DOGFOOD_RUN_RESOURCE_MISMATCH",
            "credential projection does not bind the exact run, provider, identity, and body",
        ));
    }
    if projection.activates_at_unix_ms > subjects.grant.not_before_unix_ms
        || projection.expires_at_unix_ms < subjects.grant.expires_at_unix_ms
    {
        return Err(time_mismatch(
            "credential projection does not cover the launch-grant window",
        ));
    }
    Ok(())
}

fn verify_observation_digests(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    if run.repository_context_post_observation_digest
        != subjects.post_context.observation_digest()?
        || run.provider_probe_observation_digest != subjects.probe.digest()?
    {
        return Err(mismatch(
            "DOGFOOD_RUN_RESOURCE_MISMATCH",
            "run does not bind the exact repository-post and provider-probe observations",
        ));
    }
    Ok(())
}

fn verify_times(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    let grant = subjects.grant;
    let reservation = subjects.reservation;
    let projection = subjects.projection;
    if let Some(start) = run.process.started_at_unix_ms
        && (start < grant.not_before_unix_ms
            || start >= grant.expires_at_unix_ms
            || start < reservation.reserved_at_unix_ms
            || start >= reservation.consume_before_unix_ms
            || start < projection.activates_at_unix_ms
            || start >= projection.expires_at_unix_ms
            || start < subjects.context_snapshot.prepared_at_unix_ms)
    {
        return Err(time_mismatch(
            "process start is outside an exact launch or resource window",
        ));
    }
    if let Some(end) = run.process.ended_at_unix_ms {
        let end_precedes_known_lower_bound = end < subjects.context_snapshot.prepared_at_unix_ms
            || (run.process.started_at_unix_ms.is_none()
                && (end < grant.not_before_unix_ms
                    || end < reservation.reserved_at_unix_ms
                    || end < projection.activates_at_unix_ms));
        if end_precedes_known_lower_bound {
            return Err(time_mismatch(
                "process end precedes an exact launch or resource lower bound",
            ));
        }
        if end > run.subject.deadline_unix_ms {
            return Err(time_mismatch("process end exceeds the run deadline"));
        }
    }
    let process_time = run
        .process
        .ended_at_unix_ms
        .or(run.process.started_at_unix_ms)
        .unwrap_or(subjects.context_snapshot.prepared_at_unix_ms);
    let cleanup_time = cleanup_time(&run.cleanup);
    if subjects.post_context.observed_at_unix_ms < process_time
        || run.budget_settlement.settled_at_unix_ms < subjects.post_context.observed_at_unix_ms
        || cleanup_time < run.budget_settlement.settled_at_unix_ms
        || run.attested_at_unix_ms < cleanup_time
    {
        return Err(time_mismatch(
            "post-context, settlement, cleanup, and attestation are not causal",
        ));
    }
    if run.terminal_state(subjects.reservation)? == DogfoodTerminalStateV1::ProposalReady
        && run.attested_at_unix_ms > run.subject.deadline_unix_ms
    {
        return Err(time_mismatch(
            "proposal-ready attestation exceeds the run deadline",
        ));
    }
    Ok(())
}

fn verify_proposal(
    run: &DogfoodRunV1,
    subjects: &DogfoodRunBindingSubjects<'_>,
) -> Result<(), WireError> {
    let (observed_id, observed_digest, observed_artifact, proposal) =
        match (&run.proposal, subjects.proposal) {
            (
                DogfoodProposalObservationV1::Validated {
                    proposal_id,
                    proposal_digest,
                    artifact,
                },
                Some(proposal),
            ) => (proposal_id, proposal_digest, artifact, proposal),
            (
                DogfoodProposalObservationV1::Absent
                | DogfoodProposalObservationV1::Rejected { .. },
                None,
            ) => return Ok(()),
            _ => {
                return Err(proposal_mismatch(
                    "proposal body and observation kind disagree",
                ));
            }
        };
    if proposal.operations.len() > MAX_DOGFOOD_PATCH_OPERATIONS {
        return Err(proposal_mismatch(
            "proposal exceeds the 128-operation dogfood limit",
        ));
    }
    let written_bytes = proposal
        .operations
        .iter()
        .try_fold(0_usize, |total, operation| {
            let size = match &operation.mutation {
                PatchMutation::Write { content_utf8 } => content_utf8.len(),
                PatchMutation::Delete => 0,
            };
            total
                .checked_add(size)
                .ok_or_else(|| proposal_mismatch("proposal aggregate write content overflowed"))
        })?;
    if written_bytes > MAX_DOGFOOD_PATCH_CONTENT_BYTES {
        return Err(proposal_mismatch(
            "proposal exceeds the 32-MiB dogfood content limit",
        ));
    }
    let bytes = canonical_json(proposal)?;
    if bytes.len() > MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES as usize {
        return Err(proposal_mismatch(
            "canonical proposal exceeds its artifact ceiling",
        ));
    }
    let body_digest = hash_framed_bytes(DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN, &bytes)?;
    let artifact_digest = hash_framed_bytes(DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN, &bytes)?;
    if proposal.proposal_id != *observed_id
        || proposal.producing_attempt_id != run.subject.execution.attempt_id
        || proposal.base_checkpoint_id != run.subject.repository.checkpoint_id
        || proposal.base_checkpoint_digest != subjects.context_snapshot.checkpoint_digest
        || proposal.gate_ids != run.subject.gate_ids
        || body_digest != *observed_digest
        || artifact_digest != observed_artifact.digest
        || bytes.len() as u64 != observed_artifact.size_bytes
    {
        return Err(proposal_mismatch(
            "proposal observation does not bind the exact typed body and canonical artifact",
        ));
    }
    Ok(())
}

fn cleanup_time(cleanup: &DogfoodCleanupObservationV1) -> u64 {
    match cleanup {
        DogfoodCleanupObservationV1::ProvedEmpty {
            observed_at_unix_ms,
            ..
        }
        | DogfoodCleanupObservationV1::Quarantined {
            observed_at_unix_ms,
            ..
        }
        | DogfoodCleanupObservationV1::Unknown {
            observed_at_unix_ms,
            ..
        } => *observed_at_unix_ms,
    }
}

fn runtime_error(error: RuntimePassportError) -> WireError {
    WireError::new(error.reason_code(), error.to_string())
}

fn mismatch(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}

fn time_mismatch(reason: impl Into<String>) -> WireError {
    mismatch("DOGFOOD_RUN_TIME_MISMATCH", reason)
}

fn proposal_mismatch(reason: impl Into<String>) -> WireError {
    mismatch("DOGFOOD_RUN_PROPOSAL_MISMATCH", reason)
}
