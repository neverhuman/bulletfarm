use std::collections::BTreeSet;

use super::MutationOperation;

#[test]
fn every_operation_has_a_unique_private_request_domain() {
    let domains = [
        MutationOperation::CloneWorkspace,
        MutationOperation::ReadWorkspace,
        MutationOperation::ApplyPatch,
        MutationOperation::Checkpoint,
        MutationOperation::PrepareCandidate,
        MutationOperation::PreserveWorkspace,
        MutationOperation::CleanupWorkspace,
        MutationOperation::DispatchEffect,
        MutationOperation::ReconcileEffect,
    ]
    .map(MutationOperation::request_domain);
    assert_eq!(domains.into_iter().collect::<BTreeSet<_>>().len(), 9);
}
