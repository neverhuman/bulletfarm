use super::{AuthorityAudience, MutationOperation};

impl MutationOperation {
    #[must_use]
    pub(crate) const fn request_domain(self) -> &'static str {
        match self {
            Self::CloneWorkspace => "authority.request.clone-workspace.v1alpha1",
            Self::ReadWorkspace => "authority.request.read-workspace.v1alpha1",
            Self::ApplyPatch => "authority.request.apply-patch.v1alpha1",
            Self::Checkpoint => "authority.request.checkpoint.v1alpha1",
            Self::PrepareCandidate => "authority.request.prepare-candidate.v1alpha1",
            Self::PreserveWorkspace => "authority.request.preserve-workspace.v1alpha1",
            Self::CleanupWorkspace => "authority.request.cleanup-workspace.v1alpha1",
            Self::DispatchEffect => "authority.request.dispatch-effect.v1alpha1",
            Self::ReconcileEffect => "authority.request.reconcile-effect.v1alpha1",
        }
    }

    pub(crate) const fn required_audience(self) -> AuthorityAudience {
        match self {
            Self::DispatchEffect | Self::ReconcileEffect => AuthorityAudience::EffectBroker,
            Self::CloneWorkspace
            | Self::ReadWorkspace
            | Self::ApplyPatch
            | Self::Checkpoint
            | Self::PrepareCandidate
            | Self::PreserveWorkspace
            | Self::CleanupWorkspace => AuthorityAudience::BulletGitd,
        }
    }
}
