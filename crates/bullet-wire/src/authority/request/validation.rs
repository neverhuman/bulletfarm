use std::collections::BTreeSet;

use super::{parse, require_positive, require_schema, validate_label};
use crate::{
    Blake3Digest, ContentId, GateId, PatchMutation, PatchOperation, PatchProposal, Preimage,
    RepoPath, ScopeGrantId, WireError, authority::authority_error, v1alpha1,
};

pub(super) fn validate_scope_grant(grant: &v1alpha1::ScopeGrantV1) -> Result<(), WireError> {
    require_schema(&grant.schema_version)?;
    parse::<ScopeGrantId>("scope_grant_id", &grant.scope_grant_id)?;
    require_positive("scope_revision", grant.scope_revision)?;
    validate_paths("normalized_paths", &grant.normalized_paths, true)?;
    if grant
        .protected_resources
        .iter()
        .any(|value| validate_label("protected_resource", value).is_err())
    {
        return Err(authority_error(
            "INVALID_SCOPE_GRANT",
            "protected resources must be bounded identifiers",
        ));
    }
    validate_label("envelope_class", &grant.envelope_class)
}

pub(super) fn validate_patch_proposal(
    proposal: &v1alpha1::PatchProposalV1,
) -> Result<(), WireError> {
    require_schema(&proposal.schema_version)?;
    let operations = proposal
        .operations
        .iter()
        .map(|operation| {
            require_schema(&operation.schema_version)?;
            let path = parse::<RepoPath>("path", &operation.path)?;
            let preimage = match operation.preimage_kind {
                v1alpha1::PatchPreimageKindV1::Absent if operation.preimage_digest.is_none() => {
                    Preimage::Absent
                }
                v1alpha1::PatchPreimageKindV1::Digest => Preimage::Digest {
                    digest: parse::<Blake3Digest>(
                        "preimage_digest",
                        operation.preimage_digest.as_deref().unwrap_or_default(),
                    )?,
                },
                _ => {
                    return Err(authority_error(
                        "INVALID_PATCH_PREIMAGE",
                        "patch preimage kind and digest disagree",
                    ));
                }
            };
            let mutation = match operation.mutation_kind {
                v1alpha1::PatchMutationKindV1::Write => PatchMutation::Write {
                    content_utf8: operation.content_utf8.clone().ok_or_else(|| {
                        authority_error("INVALID_PATCH_MUTATION", "write requires content_utf8")
                    })?,
                },
                v1alpha1::PatchMutationKindV1::Delete if operation.content_utf8.is_none() => {
                    PatchMutation::Delete
                }
                _ => {
                    return Err(authority_error(
                        "INVALID_PATCH_MUTATION",
                        "delete forbids content_utf8",
                    ));
                }
            };
            Ok(PatchOperation {
                path,
                preimage,
                mutation,
            })
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    let proposal = PatchProposal {
        schema_version: crate::SCHEMA_VERSION,
        proposal_id: parse::<ContentId>("proposal_id", &proposal.proposal_id)?,
        producing_attempt_id: parse("producing_attempt_id", &proposal.producing_attempt_id)?,
        base_checkpoint_id: parse("base_checkpoint_id", &proposal.base_checkpoint_id)?,
        base_checkpoint_digest: parse("base_checkpoint_digest", &proposal.base_checkpoint_digest)?,
        operations,
        gate_ids: proposal
            .gate_ids
            .iter()
            .map(|gate| parse::<GateId>("gate_id", gate))
            .collect::<Result<Vec<_>, _>>()?,
    };
    proposal.validate()
}

pub(super) fn validate_paths(
    name: &str,
    paths: &[String],
    require_nonempty: bool,
) -> Result<(), WireError> {
    if require_nonempty && paths.is_empty() {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} must not be empty"),
        ));
    }
    let mut unique = BTreeSet::new();
    for path in paths {
        let parsed = parse::<RepoPath>(name, path)?;
        if !unique.insert(parsed.to_string().to_lowercase()) {
            return Err(authority_error(
                "DUPLICATE_AUTHORITY_PATH",
                format!("{name} contains a duplicate or case-colliding path"),
            ));
        }
    }
    Ok(())
}
