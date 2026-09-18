use super::{LaunchGrantClaims, LaunchGrantExpectation, LaunchProvider, launch_error};
use crate::{
    AttemptId, Blake3Digest, GraphRevisionId, MissionId, ProviderProfileId, RepositoryId, RunnerId,
    VariantId, WireError, WorkPackageId, WorkspaceId,
};

/// The durable active lease row a grant must bind exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchLeaseSubject {
    pub mission_id: MissionId,
    pub repository_id: RepositoryId,
    pub graph_revision_id: GraphRevisionId,
    pub work_package_id: WorkPackageId,
    pub variant_id: VariantId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub runner_id: RunnerId,
    pub runner_epoch: u64,
    pub workspace_id: WorkspaceId,
    pub workspace_nonce_digest: Blake3Digest,
    pub authority_epoch: u64,
    pub freeze_generation: u64,
}

/// The evaluated provider admission a grant must bind exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchProviderSubject {
    pub provider: LaunchProvider,
    pub adapter: String,
    pub provider_profile_id: ProviderProfileId,
    pub model: String,
    pub credential_generation: u64,
    pub protocol: String,
    pub executable_path: String,
    pub executable_digest: Blake3Digest,
    pub descriptor_digest: Blake3Digest,
    pub capability_digest: Blake3Digest,
}

impl LaunchGrantClaims {
    #[must_use]
    pub fn lease_subject(&self) -> LaunchLeaseSubject {
        LaunchLeaseSubject {
            mission_id: self.mission_id.clone(),
            repository_id: self.repository_id.clone(),
            graph_revision_id: self.graph_revision_id.clone(),
            work_package_id: self.work_package_id.clone(),
            variant_id: self.variant_id.clone(),
            attempt_id: self.attempt_id.clone(),
            attempt_fence: self.attempt_fence,
            runner_id: self.runner_id.clone(),
            runner_epoch: self.runner_epoch,
            workspace_id: self.workspace_id.clone(),
            workspace_nonce_digest: self.workspace_nonce_digest,
            authority_epoch: self.authority_epoch,
            freeze_generation: self.freeze_generation,
        }
    }

    #[must_use]
    pub fn provider_subject(&self) -> LaunchProviderSubject {
        LaunchProviderSubject {
            provider: self.provider,
            adapter: self.adapter.clone(),
            provider_profile_id: self.provider_profile_id.clone(),
            model: self.model.clone(),
            credential_generation: self.credential_generation,
            protocol: self.protocol.clone(),
            executable_path: self.executable_path.clone(),
            executable_digest: self.executable_digest,
            descriptor_digest: self.descriptor_digest,
            capability_digest: self.capability_digest,
        }
    }

    pub(super) fn verify_subject(
        &self,
        expected: &LaunchGrantExpectation,
    ) -> Result<(), WireError> {
        if self.lease_subject() != expected.lease {
            return Err(launch_error(
                "LAUNCH_GRANT_SUBJECT_MISMATCH",
                "launch grant does not bind the durable active lease exactly",
            ));
        }
        if self.provider_subject() != expected.provider {
            return Err(launch_error(
                "LAUNCH_GRANT_SUBJECT_MISMATCH",
                "launch grant does not bind the evaluated provider admission exactly",
            ));
        }
        if self.policy_snapshot_digest != expected.policy_snapshot_digest {
            return Err(launch_error(
                "LAUNCH_GRANT_SUBJECT_MISMATCH",
                "launch grant does not bind the loaded policy snapshot",
            ));
        }
        Ok(())
    }
}
