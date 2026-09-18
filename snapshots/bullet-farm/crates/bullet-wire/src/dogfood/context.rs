//! Immutable repository disclosure context and separate post-run observation.
//! These records describe bytes; they do not prove filesystem or Git custody.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    AttemptId, Blake3Digest, CheckpointId, DogfoodRunId, GitOid, MAX_CANONICAL_DOCUMENT_BYTES,
    PrincipalId, RepoPath, RepositoryContextSnapshotId, RepositoryId, SourceDescriptorId,
    WireError, canonical_json, decode_canonical, hash_canonical, ids::require_exact_wire,
};

use super::{DOGFOOD_SCHEMA_VERSION, DogfoodReadOnlyIntentV1};

pub const REPOSITORY_CONTEXT_SNAPSHOT_DIGEST_DOMAIN: &str =
    "dogfood.repository-context-snapshot.v1alpha1";
pub const REPOSITORY_CONTEXT_VISIBLE_MANIFEST_DIGEST_DOMAIN: &str =
    "dogfood.repository-context-visible-manifest.v1alpha1";
pub const REPOSITORY_CONTEXT_POST_OBSERVATION_DIGEST_DOMAIN: &str =
    "dogfood.repository-context-post-observation.v1alpha1";
pub const MAX_REPOSITORY_CONTEXT_SCOPES: usize = 128;
pub const MAX_REPOSITORY_CONTEXT_FILES: usize = 4_096;
pub const MAX_REPOSITORY_CONTEXT_TOTAL_BYTES: u64 = 32 * 1024 * 1024;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryVisibleFileV1 {
    pub path: RepoPath,
    pub preimage_digest: Blake3Digest,
    pub size_bytes: u64,
    pub executable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryContextSnapshotV1 {
    pub schema_version: String,
    pub run_id: DogfoodRunId,
    pub repository_id: RepositoryId,
    pub source_descriptor_id: SourceDescriptorId,
    pub attempt_id: AttemptId,
    pub attempt_fence: u64,
    pub owner_principal_id: PrincipalId,
    pub head_oid: GitOid,
    pub tree_oid: GitOid,
    pub checkpoint_id: CheckpointId,
    pub checkpoint_digest: Blake3Digest,
    pub scope_grant_digest: Blake3Digest,
    pub visible_scopes: Vec<RepoPath>,
    pub files: Vec<RepositoryVisibleFileV1>,
    pub aggregate_file_count: u64,
    pub aggregate_size_bytes: u64,
    pub visible_manifest_digest: Blake3Digest,
    pub prepared_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryContextPostObservationV1 {
    pub schema_version: String,
    pub context_snapshot_id: RepositoryContextSnapshotId,
    pub run_id: DogfoodRunId,
    pub observer_principal_id: PrincipalId,
    pub observed_at_unix_ms: u64,
    pub observed_owner_principal_id: PrincipalId,
    pub observed_head_oid: GitOid,
    pub observed_tree_oid: GitOid,
    pub observed_checkpoint_id: CheckpointId,
    pub observed_checkpoint_digest: Blake3Digest,
    pub observed_visible_manifest_digest: Blake3Digest,
}

impl RepositoryContextSnapshotV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        if self.attempt_fence == 0 || self.attempt_fence > MAX_SAFE_INTEGER {
            return Err(invalid("attempt_fence must be a positive safe integer"));
        }
        if self.prepared_at_unix_ms > MAX_SAFE_INTEGER
            || self.aggregate_file_count > MAX_SAFE_INTEGER
            || self.aggregate_size_bytes > MAX_SAFE_INTEGER
        {
            return Err(invalid("snapshot integer exceeds the safe integer range"));
        }
        if self.head_oid.algorithm() != self.tree_oid.algorithm() {
            return Err(invalid(
                "head_oid and tree_oid use different Git algorithms",
            ));
        }
        validate_scopes(&self.visible_scopes)?;
        self.validate_files()?;
        if self.computed_visible_manifest_digest()? != self.visible_manifest_digest {
            return Err(WireError::new(
                "REPOSITORY_CONTEXT_MANIFEST_MISMATCH",
                "visible manifest digest does not bind scopes, files, count, and size",
            ));
        }
        let bytes = canonical_json(self).map_err(|error| invalid(error.to_string()))?;
        if bytes.len() > MAX_CANONICAL_DOCUMENT_BYTES {
            return Err(invalid("repository context exceeds the 1 MiB wire ceiling"));
        }
        Ok(())
    }

    pub fn computed_visible_manifest_digest(&self) -> Result<Blake3Digest, WireError> {
        hash_canonical(
            REPOSITORY_CONTEXT_VISIBLE_MANIFEST_DIGEST_DOMAIN,
            &VisibleManifestV1 {
                visible_scopes: &self.visible_scopes,
                files: &self.files,
                aggregate_file_count: self.aggregate_file_count,
                aggregate_size_bytes: self.aggregate_size_bytes,
            },
        )
    }

    pub fn context_snapshot_id(&self) -> Result<RepositoryContextSnapshotId, WireError> {
        self.validate()?;
        hash_canonical(REPOSITORY_CONTEXT_SNAPSHOT_DIGEST_DOMAIN, self)
            .map(RepositoryContextSnapshotId::from_digest)
    }

    fn validate_files(&self) -> Result<(), WireError> {
        if self.files.is_empty() || self.files.len() > MAX_REPOSITORY_CONTEXT_FILES {
            return Err(invalid("files must contain 1..=4096 visible entries"));
        }
        let mut total = 0_u64;
        let mut previous: Option<&str> = None;
        let mut collision_keys = BTreeSet::new();
        for file in &self.files {
            if previous.is_some_and(|path| path.as_bytes() >= file.path.as_str().as_bytes()) {
                return Err(invalid("files must be raw-UTF-8-byte-sorted and unique"));
            }
            previous = Some(file.path.as_str());
            if !collision_keys.insert(collision_key(&file.path)?) {
                return Err(invalid("files contain a portable case collision"));
            }
            if !self
                .visible_scopes
                .iter()
                .any(|scope| scope.contains(&file.path))
            {
                return Err(invalid("visible file is outside every disclosed scope"));
            }
            if file.size_bytes > MAX_SAFE_INTEGER {
                return Err(invalid("file size exceeds the safe integer range"));
            }
            total = total
                .checked_add(file.size_bytes)
                .ok_or_else(|| invalid("aggregate visible size overflowed"))?;
        }
        if self.aggregate_file_count != self.files.len() as u64
            || self.aggregate_size_bytes != total
            || total > MAX_REPOSITORY_CONTEXT_TOTAL_BYTES
        {
            return Err(invalid(
                "aggregate file count or size is inconsistent or exceeds 32 MiB",
            ));
        }
        Ok(())
    }
}

impl RepositoryContextPostObservationV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        require_schema(&self.schema_version)?;
        if self.observed_at_unix_ms > MAX_SAFE_INTEGER {
            return Err(invalid(
                "post-observation time exceeds the safe integer range",
            ));
        }
        if self.observed_head_oid.algorithm() != self.observed_tree_oid.algorithm() {
            return Err(invalid(
                "observed head and tree use different Git algorithms",
            ));
        }
        Ok(())
    }

    pub fn observation_digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(REPOSITORY_CONTEXT_POST_OBSERVATION_DIGEST_DOMAIN, self)
    }
}

pub fn decode_repository_context_snapshot(
    bytes: &[u8],
) -> Result<RepositoryContextSnapshotV1, WireError> {
    if bytes.len() > MAX_CANONICAL_DOCUMENT_BYTES {
        return Err(invalid("repository context exceeds the 1 MiB wire ceiling"));
    }
    let snapshot: RepositoryContextSnapshotV1 = decode_canonical(bytes)?;
    snapshot.validate()?;
    Ok(snapshot)
}

pub fn decode_repository_context_post_observation(
    bytes: &[u8],
) -> Result<RepositoryContextPostObservationV1, WireError> {
    let observation: RepositoryContextPostObservationV1 = decode_canonical(bytes)?;
    observation.validate()?;
    Ok(observation)
}

pub fn verify_repository_context_binding(
    intent: &DogfoodReadOnlyIntentV1,
    snapshot: &RepositoryContextSnapshotV1,
) -> Result<(), WireError> {
    intent.validate()?;
    snapshot.validate()?;
    if snapshot.context_snapshot_id()? != intent.subject.repository.context_snapshot_id {
        return Err(WireError::new(
            "REPOSITORY_CONTEXT_ID_MISMATCH",
            "snapshot content address does not match the intent context id",
        ));
    }
    let execution = &intent.subject.execution;
    let repository = &intent.subject.repository;
    if snapshot.run_id != execution.run_id
        || snapshot.repository_id != execution.repository_id
        || snapshot.attempt_id != execution.attempt_id
        || snapshot.attempt_fence != execution.attempt_fence
        || snapshot.head_oid != repository.head_oid
        || snapshot.tree_oid != repository.tree_oid
        || snapshot.checkpoint_id != repository.checkpoint_id
    {
        return Err(WireError::new(
            "REPOSITORY_CONTEXT_SUBJECT_MISMATCH",
            "snapshot does not bind the exact intent repository and Attempt subject",
        ));
    }
    Ok(())
}

pub fn verify_repository_context_post_observation(
    snapshot: &RepositoryContextSnapshotV1,
    observation: &RepositoryContextPostObservationV1,
) -> Result<(), WireError> {
    snapshot.validate()?;
    observation.validate()?;
    if observation.context_snapshot_id != snapshot.context_snapshot_id()? {
        return Err(WireError::new(
            "REPOSITORY_CONTEXT_POST_MISMATCH",
            "post observation names a different context snapshot",
        ));
    }
    if observation.observed_at_unix_ms < snapshot.prepared_at_unix_ms
        || observation.run_id != snapshot.run_id
        || observation.observed_owner_principal_id != snapshot.owner_principal_id
        || observation.observed_head_oid != snapshot.head_oid
        || observation.observed_tree_oid != snapshot.tree_oid
        || observation.observed_checkpoint_id != snapshot.checkpoint_id
        || observation.observed_checkpoint_digest != snapshot.checkpoint_digest
        || observation.observed_visible_manifest_digest != snapshot.visible_manifest_digest
    {
        return Err(WireError::new(
            "REPOSITORY_CONTEXT_POST_MISMATCH",
            "post observation does not exactly read back the prepared context",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct VisibleManifestV1<'a> {
    visible_scopes: &'a [RepoPath],
    files: &'a [RepositoryVisibleFileV1],
    aggregate_file_count: u64,
    aggregate_size_bytes: u64,
}

fn validate_scopes(scopes: &[RepoPath]) -> Result<(), WireError> {
    if scopes.is_empty() || scopes.len() > MAX_REPOSITORY_CONTEXT_SCOPES {
        return Err(invalid("visible_scopes must contain 1..=128 entries"));
    }
    let mut previous: Option<&str> = None;
    let mut collision_keys = BTreeSet::new();
    for scope in scopes {
        if previous.is_some_and(|path| path.as_bytes() >= scope.as_str().as_bytes()) {
            return Err(invalid(
                "visible_scopes must be raw-UTF-8-byte-sorted and unique",
            ));
        }
        previous = Some(scope.as_str());
        if !collision_keys.insert(collision_key(scope)?) {
            return Err(invalid("visible_scopes contain a portable case collision"));
        }
    }
    for (index, left) in scopes.iter().enumerate() {
        if scopes[index + 1..]
            .iter()
            .any(|right| left.contains(right) || right.contains(left))
        {
            return Err(invalid("visible_scopes must not overlap"));
        }
    }
    Ok(())
}

fn collision_key(path: &RepoPath) -> Result<String, WireError> {
    if !path.as_str().is_ascii() {
        return Err(invalid(
            "repository context paths require ASCII until canonical Unicode casefold is published",
        ));
    }
    Ok(path.as_str().to_ascii_lowercase())
}

fn require_schema(actual: &str) -> Result<(), WireError> {
    require_exact_wire(
        "schema_version",
        actual,
        DOGFOOD_SCHEMA_VERSION,
        "REPOSITORY_CONTEXT_INVALID",
    )
}

fn invalid(reason: impl Into<String>) -> WireError {
    WireError::new("REPOSITORY_CONTEXT_INVALID", reason)
}
