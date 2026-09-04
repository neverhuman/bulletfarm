use serde::{Deserialize, Serialize};

use crate::{
    AttemptId, Blake3Digest, CandidateId, CandidateProofRoot, ChangeId, CheckpointId, ContentId,
    EffectReceiptId, GateId, GitOid, GraphRevisionId, IntegrationProofRoot, PlanRevisionId,
    RepoPath, RepositoryId, VariantId, WireError, WorkPackageId, hash_canonical,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateManifest {
    pub schema_version: u32,
    pub repository_id: RepositoryId,
    pub change_id: ChangeId,
    pub producing_attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub work_package_id: WorkPackageId,
    pub variant_id: VariantId,
    pub plan_revision_id: PlanRevisionId,
    pub graph_revision_id: GraphRevisionId,
    pub base_checkpoint_id: CheckpointId,
    pub base_commit: GitOid,
    pub head_commit: GitOid,
    pub tree_oid: GitOid,
    pub patch_digest: Blake3Digest,
    pub parent_candidate_ids: Vec<CandidateId>,
    pub granted_scope: Vec<RepoPath>,
    pub actual_scope: Vec<RepoPath>,
    pub context_capsule_id: ContentId,
    pub configuration_snapshot_id: ContentId,
    pub policy_snapshot_id: ContentId,
    pub routing_snapshot_id: ContentId,
    pub environment_digest: Blake3Digest,
    pub toolchain_digest: Blake3Digest,
}

impl CandidateManifest {
    pub fn content_id(&self) -> Result<ContentId, WireError> {
        let content = CandidateContentManifest {
            repository_id: &self.repository_id,
            base_commit: &self.base_commit,
            head_commit: &self.head_commit,
            tree_oid: &self.tree_oid,
            patch_digest: self.patch_digest,
        };
        hash_canonical("candidate.content", &content).map(ContentId::from_digest)
    }

    pub fn candidate_id(&self) -> Result<CandidateId, WireError> {
        self.validate()?;
        hash_canonical("candidate.provenance", self).map(CandidateId::from_digest)
    }

    pub fn validate(&self) -> Result<(), WireError> {
        require_schema(self.schema_version, "CandidateManifest")?;
        if self.attempt_fence == 0 {
            return Err(WireError::new(
                "INVALID_FENCE",
                "Candidate fence must be nonzero",
            ));
        }
        for path in &self.actual_scope {
            if !self.granted_scope.iter().any(|grant| grant.contains(path)) {
                return Err(WireError::new(
                    "ACTUAL_SCOPE_EXCEEDS_GRANT",
                    format!("actual path {path} is outside the granted scope"),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct CandidateContentManifest<'a> {
    repository_id: &'a RepositoryId,
    base_commit: &'a GitOid,
    head_commit: &'a GitOid,
    tree_oid: &'a GitOid,
    patch_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointManifest {
    pub schema_version: u32,
    pub repository_id: RepositoryId,
    pub attempt_id: AttemptId,
    pub fence: u64,
    pub workspace_generation: u64,
    pub base_commit: GitOid,
    pub head_commit: GitOid,
    pub tree_oid: GitOid,
    pub journal_start: u64,
    pub journal_end: u64,
    pub journal_digest: Blake3Digest,
    pub cas_root: Blake3Digest,
}

impl CheckpointManifest {
    pub fn checkpoint_id(&self) -> Result<CheckpointId, WireError> {
        require_schema(self.schema_version, "CheckpointManifest")?;
        if self.fence == 0 || self.journal_end < self.journal_start {
            return Err(WireError::new(
                "INVALID_CHECKPOINT",
                "checkpoint requires a nonzero fence and ordered journal range",
            ));
        }
        hash_canonical("checkpoint.manifest", self).map(CheckpointId::from_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationReceiptManifest {
    pub schema_version: u32,
    pub attempt_id: AttemptId,
    pub fence: u64,
    pub workspace_nonce: Blake3Digest,
    pub tree_oid: GitOid,
    pub dirty_manifest_digest: Blake3Digest,
    pub untracked_manifest_digest: Blake3Digest,
    pub journal_start: u64,
    pub journal_end: u64,
    pub bundle_or_cas_digest: Blake3Digest,
    pub external_destination: String,
    pub external_destination_digest: Blake3Digest,
    pub daemon_identity: String,
    pub signature_digest: Blake3Digest,
}

impl PreservationReceiptManifest {
    pub fn receipt_digest(&self) -> Result<Blake3Digest, WireError> {
        require_schema(self.schema_version, "PreservationReceiptManifest")?;
        if self.fence == 0 || self.journal_end < self.journal_start {
            return Err(WireError::new(
                "INVALID_PRESERVATION_RECEIPT",
                "receipt requires a nonzero fence and ordered journal range",
            ));
        }
        hash_canonical("preservation.receipt", self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateProofManifest {
    pub schema_version: u32,
    pub candidate_id: CandidateId,
    pub checkpoint_id: CheckpointId,
    pub journal_digest: Blake3Digest,
    pub cas_root: Blake3Digest,
    pub preservation_receipt_digest: Blake3Digest,
    pub gate_ids: Vec<GateId>,
}

impl CandidateProofManifest {
    pub fn proof_root(&self) -> Result<CandidateProofRoot, WireError> {
        require_schema(self.schema_version, "CandidateProofManifest")?;
        hash_canonical("candidate.proof-root", self).map(CandidateProofRoot::from_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrationProofManifest {
    pub schema_version: u32,
    pub candidate_proof_root: CandidateProofRoot,
    pub selection_digest: Blake3Digest,
    pub approval_digests: Vec<Blake3Digest>,
    pub effect_receipt_ids: Vec<EffectReceiptId>,
    pub integrated_commit: GitOid,
    pub observed_ref: String,
    pub observation_digest: Blake3Digest,
}

impl IntegrationProofManifest {
    pub fn proof_root(&self) -> Result<IntegrationProofRoot, WireError> {
        require_schema(self.schema_version, "IntegrationProofManifest")?;
        hash_canonical("integration.proof-root", self).map(IntegrationProofRoot::from_digest)
    }
}

fn require_schema(schema_version: u32, kind: &str) -> Result<(), WireError> {
    if schema_version != crate::SCHEMA_VERSION {
        return Err(WireError::new(
            "UNSUPPORTED_SCHEMA",
            format!("{kind} schema {schema_version} is unsupported"),
        ));
    }
    Ok(())
}
