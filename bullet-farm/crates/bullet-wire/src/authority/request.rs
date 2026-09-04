use std::collections::BTreeSet;

use serde::Serialize;

use super::{AUTHORITY_SCHEMA_VERSION, AuthorityClaims, MutationOperation, authority_error};
use crate::{
    AttemptId, Blake3Digest, CandidateId, CandidateProofRoot, ChangeId, CheckpointId, ContentId,
    EffectIntentId, GateId, GitOid, MutationId, RepoPath, RepositoryId, ScopeGrantId,
    SourceDescriptorId, WireError, WorkspaceId, hash_canonical, v1alpha1,
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

mod private {
    pub trait Sealed {}
}

mod validation;
use validation::{validate_patch_proposal, validate_paths, validate_scope_grant};

pub trait AuthorityRequest: private::Sealed + Serialize + Sized {
    const OPERATION: MutationOperation;

    fn validate(&self) -> Result<(), WireError>;

    #[doc(hidden)]
    fn binding(&self) -> Result<AuthorityRequestBinding, WireError>;

    #[doc(hidden)]
    fn validate_claim_binding(&self, _claims: &AuthorityClaims) -> Result<(), WireError> {
        Ok(())
    }

    fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(Self::OPERATION.request_domain(), self)
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityRequestBinding {
    pub(super) mutation_id: MutationId,
    pub(super) repository_id: RepositoryId,
    pub(super) workspace_id: WorkspaceId,
    pub(super) workspace_generation: u64,
}

pub fn authority_request_digest<R: AuthorityRequest>(
    request: &R,
) -> Result<Blake3Digest, WireError> {
    request.digest()
}

macro_rules! seal {
    ($type:ty) => {
        impl private::Sealed for $type {}
    };
}

macro_rules! request_binding {
    () => {
        fn binding(&self) -> Result<AuthorityRequestBinding, WireError> {
            parse_workspace_subject(
                &self.schema_version,
                &self.mutation_id,
                &self.repository_id,
                &self.workspace_id,
                self.workspace_generation,
            )
        }
    };
}

seal!(v1alpha1::CloneWorkspaceRequestV1);
seal!(v1alpha1::ReadWorkspaceRequestV1);
seal!(v1alpha1::ApplyPatchRequestV1);
seal!(v1alpha1::CheckpointRequestV1);
seal!(v1alpha1::PrepareCandidateRequestV1);
seal!(v1alpha1::PreserveWorkspaceRequestV1);
seal!(v1alpha1::CleanupWorkspaceRequestV1);
seal!(v1alpha1::DispatchEffectRequestV1);
seal!(v1alpha1::ReconcileEffectRequestV1);

impl AuthorityRequest for v1alpha1::CloneWorkspaceRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::CloneWorkspace;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        parse::<GitOid>("base_oid", &self.base_oid)?;
        parse::<SourceDescriptorId>("source_descriptor_id", &self.source_descriptor_id)?;
        require_time(
            "trusted_commit_time_unix_ms",
            self.trusted_commit_time_unix_ms,
        )?;
        validate_scope_grant(&self.scope_grant)?;
        let supplied = parse::<Blake3Digest>("scope_grant_digest", &self.scope_grant_digest)?;
        let actual = hash_canonical("authority.scope-grant.v1alpha1", &self.scope_grant)?;
        if supplied != actual {
            return Err(authority_error(
                "SCOPE_GRANT_DIGEST_MISMATCH",
                "clone request does not bind its exact scope grant",
            ));
        }
        Ok(())
    }

    request_binding!();

    fn validate_claim_binding(&self, claims: &AuthorityClaims) -> Result<(), WireError> {
        let scope_grant_digest =
            parse::<Blake3Digest>("scope_grant_digest", &self.scope_grant_digest)?;
        if claims.scope_grant_digest != scope_grant_digest
            || claims.scope_revision != self.scope_grant.scope_revision
        {
            return Err(claim_binding_mismatch(
                "clone request scope does not match signed authority claims",
            ));
        }
        Ok(())
    }
}

impl AuthorityRequest for v1alpha1::ReadWorkspaceRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::ReadWorkspace;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        parse::<CheckpointId>("checkpoint_id", &self.checkpoint_id)?;
        parse::<Blake3Digest>("checkpoint_digest", &self.checkpoint_digest)?;
        validate_paths("paths", &self.paths, true)
    }

    request_binding!();
}

impl AuthorityRequest for v1alpha1::ApplyPatchRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::ApplyPatch;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        validate_patch_proposal(&self.proposal)
    }

    request_binding!();

    fn validate_claim_binding(&self, claims: &AuthorityClaims) -> Result<(), WireError> {
        let producing_attempt_id =
            parse::<AttemptId>("producing_attempt_id", &self.proposal.producing_attempt_id)?;
        if claims.attempt_id != producing_attempt_id {
            return Err(claim_binding_mismatch(
                "patch proposal attempt does not match signed authority claims",
            ));
        }
        Ok(())
    }
}

impl AuthorityRequest for v1alpha1::CheckpointRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::Checkpoint;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        parse::<GitOid>("tree_oid", &self.tree_oid)?;
        validate_journal(self.journal_start, self.journal_end, &self.journal_digest)?;
        parse::<Blake3Digest>("cas_root", &self.cas_root)?;
        Ok(())
    }

    request_binding!();
}

impl AuthorityRequest for v1alpha1::PrepareCandidateRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::PrepareCandidate;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        parse::<ChangeId>("change_id", &self.change_id)?;
        parse::<CheckpointId>("base_checkpoint_id", &self.base_checkpoint_id)?;
        parse::<Blake3Digest>("base_checkpoint_digest", &self.base_checkpoint_digest)?;
        parse::<GitOid>("tree_oid", &self.tree_oid)?;
        parse_unique::<CandidateId>("parent_candidate_ids", &self.parent_candidate_ids)?;
        require_time(
            "trusted_commit_time_unix_ms",
            self.trusted_commit_time_unix_ms,
        )
    }

    request_binding!();
}

impl AuthorityRequest for v1alpha1::PreserveWorkspaceRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::PreserveWorkspace;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        parse::<GitOid>("tree_oid", &self.tree_oid)?;
        parse::<Blake3Digest>("dirty_manifest_digest", &self.dirty_manifest_digest)?;
        parse::<Blake3Digest>("untracked_manifest_digest", &self.untracked_manifest_digest)?;
        validate_journal(self.journal_start, self.journal_end, &self.journal_digest)?;
        parse::<ContentId>("destination_id", &self.destination_id)?;
        parse::<Blake3Digest>(
            "expected_destination_digest",
            &self.expected_destination_digest,
        )?;
        Ok(())
    }

    request_binding!();
}

impl AuthorityRequest for v1alpha1::CleanupWorkspaceRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::CleanupWorkspace;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        require_schema(&self.authorization.schema_version)?;
        parse::<Blake3Digest>(
            "preservation_receipt_digest",
            &self.authorization.preservation_receipt_digest,
        )?;
        parse::<Blake3Digest>(
            "expected_destination_digest",
            &self.authorization.expected_destination_digest,
        )?;
        parse::<Blake3Digest>(
            "authority_decision_digest",
            &self.authorization.authority_decision_digest,
        )?;
        Ok(())
    }

    request_binding!();
}

impl AuthorityRequest for v1alpha1::DispatchEffectRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::DispatchEffect;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        validate_effect_subject(
            &self.effect_intent_id,
            &self.effect_intent_digest,
            &self.endpoint_identity,
            &self.logical_key,
        )?;
        parse::<Blake3Digest>("desired_state_digest", &self.desired_state_digest)?;
        parse::<Blake3Digest>("expected_state_digest", &self.expected_state_digest)?;
        parse::<CandidateId>("candidate_id", &self.candidate_id)?;
        parse::<crate::CandidateProofRoot>("candidate_proof_root", &self.candidate_proof_root)?;
        parse::<ContentId>("policy_snapshot_id", &self.policy_snapshot_id)?;
        require_positive("authority_epoch", self.authority_epoch)?;
        require_safe("freeze_generation", self.freeze_generation)?;
        validate_label("effect_kind", &self.effect_kind)
    }

    request_binding!();

    fn validate_claim_binding(&self, claims: &AuthorityClaims) -> Result<(), WireError> {
        let policy_snapshot_id =
            parse::<ContentId>("policy_snapshot_id", &self.policy_snapshot_id)?;
        if claims.policy_snapshot_id != policy_snapshot_id
            || claims.authority_epoch != self.authority_epoch
            || claims.freeze_generation != self.freeze_generation
        {
            return Err(claim_binding_mismatch(
                "effect policy, authority epoch, or freeze generation conflicts with signed claims",
            ));
        }
        Ok(())
    }
}

impl AuthorityRequest for v1alpha1::ReconcileEffectRequestV1 {
    const OPERATION: MutationOperation = MutationOperation::ReconcileEffect;

    fn validate(&self) -> Result<(), WireError> {
        validate_workspace_subject(
            &self.schema_version,
            &self.mutation_id,
            &self.repository_id,
            &self.workspace_id,
            self.workspace_generation,
        )?;
        validate_effect_subject(
            &self.effect_intent_id,
            &self.effect_intent_digest,
            &self.endpoint_identity,
            &self.logical_key,
        )?;
        for (name, value) in [
            ("desired_state_digest", &self.desired_state_digest),
            ("dispatch_receipt_digest", &self.dispatch_receipt_digest),
            ("observed_state_digest", &self.observed_state_digest),
        ] {
            parse::<Blake3Digest>(name, value)?;
        }
        Ok(())
    }

    request_binding!();
}

fn validate_workspace_subject(
    schema_version: &str,
    mutation_id: &str,
    repository_id: &str,
    workspace_id: &str,
    workspace_generation: u64,
) -> Result<(), WireError> {
    parse_workspace_subject(
        schema_version,
        mutation_id,
        repository_id,
        workspace_id,
        workspace_generation,
    )?;
    Ok(())
}

fn parse_workspace_subject(
    schema_version: &str,
    mutation_id: &str,
    repository_id: &str,
    workspace_id: &str,
    workspace_generation: u64,
) -> Result<AuthorityRequestBinding, WireError> {
    require_schema(schema_version)?;
    let mutation_id = parse::<MutationId>("mutation_id", mutation_id)?;
    let repository_id = parse::<RepositoryId>("repository_id", repository_id)?;
    let workspace_id = parse::<WorkspaceId>("workspace_id", workspace_id)?;
    require_positive("workspace_generation", workspace_generation)?;
    Ok(AuthorityRequestBinding {
        mutation_id,
        repository_id,
        workspace_id,
        workspace_generation,
    })
}

fn validate_effect_subject(
    effect_intent_id: &str,
    effect_intent_digest: &str,
    endpoint_identity: &str,
    logical_key: &str,
) -> Result<(), WireError> {
    parse::<EffectIntentId>("effect_intent_id", effect_intent_id)?;
    parse::<Blake3Digest>("effect_intent_digest", effect_intent_digest)?;
    validate_label("endpoint_identity", endpoint_identity)?;
    validate_label("logical_key", logical_key)
}

trait AuthorityField: Sized {
    fn parse_authority_field(value: &str) -> Result<Self, WireError>;
}

macro_rules! authority_fields {
    ($($type:ty),+ $(,)?) => {
        $(
            impl AuthorityField for $type {
                fn parse_authority_field(value: &str) -> Result<Self, WireError> {
                    <$type>::parse_checked(value)
                }
            }
        )+
    };
}

authority_fields!(
    AttemptId,
    Blake3Digest,
    CandidateId,
    CandidateProofRoot,
    ChangeId,
    CheckpointId,
    ContentId,
    EffectIntentId,
    GateId,
    GitOid,
    MutationId,
    RepoPath,
    RepositoryId,
    ScopeGrantId,
    SourceDescriptorId,
    WorkspaceId,
);

fn parse<T: AuthorityField>(name: &str, value: &str) -> Result<T, WireError> {
    T::parse_authority_field(value).map_err(|error| {
        authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} is invalid: {error}"),
        )
    })
}

fn parse_unique<T>(name: &str, values: &[String]) -> Result<(), WireError>
where
    T: AuthorityField + ToString,
{
    let mut unique = BTreeSet::new();
    for value in values {
        let parsed = parse::<T>(name, value)?;
        if !unique.insert(parsed.to_string()) {
            return Err(authority_error(
                "DUPLICATE_AUTHORITY_ID",
                format!("{name} contains a duplicate identity"),
            ));
        }
    }
    Ok(())
}

fn validate_journal(start: u64, end: u64, digest: &str) -> Result<(), WireError> {
    require_safe("journal_start", start)?;
    require_safe("journal_end", end)?;
    if end < start {
        return Err(authority_error(
            "INVALID_JOURNAL_RANGE",
            "journal end precedes journal start",
        ));
    }
    parse::<Blake3Digest>("journal_digest", digest)?;
    Ok(())
}

fn require_schema(actual: &str) -> Result<(), WireError> {
    if actual != AUTHORITY_SCHEMA_VERSION {
        return Err(authority_error(
            "UNSUPPORTED_AUTHORITY_SCHEMA",
            "authority request requires schema v1alpha1",
        ));
    }
    Ok(())
}

fn require_positive(name: &str, value: u64) -> Result<(), WireError> {
    if value == 0 {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} must be positive"),
        ));
    }
    require_safe(name, value)
}

fn require_time(name: &str, value: u64) -> Result<(), WireError> {
    require_safe(name, value)
}

fn require_safe(name: &str, value: u64) -> Result<(), WireError> {
    if value > MAX_SAFE_INTEGER {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} exceeds the interoperable integer range"),
        ));
    }
    Ok(())
}

fn validate_label(name: &str, value: &str) -> Result<(), WireError> {
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} must be bounded non-control text"),
        ));
    }
    Ok(())
}

fn claim_binding_mismatch(reason: &'static str) -> WireError {
    authority_error("AUTHORITY_REQUEST_BINDING_MISMATCH", reason)
}

#[cfg(test)]
mod tests;
