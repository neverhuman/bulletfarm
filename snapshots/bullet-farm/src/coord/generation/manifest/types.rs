use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::coord::{CoordError, model::FrozenClaimSubject};

use super::{
    FROZEN_LIVE_SOURCE_PATH, INTERRUPTED_CAPTURE_PATH, MANIFEST_SCHEMA_VERSION, MAX_ARTIFACT_BYTES,
    STORAGE_EPOCH, TAINTED_GENERATION_PATH, TRUSTED_PREFIX_PATH, TrustedProjectionInventory,
    invalid, validate_prefixed_hex, validate_relative_path,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct GenerationId(pub(super) String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct Sha256Digest(pub(super) String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct RelativeArtifactPath(pub(super) String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArtifactBinding {
    pub(crate) relative_path: RelativeArtifactPath,
    pub(crate) byte_length: u64,
    pub(crate) record_count: Option<u64>,
    pub(crate) ends_with_lf: bool,
    pub(crate) sha256: Sha256Digest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryArtifacts {
    pub(crate) trusted_prefix: ArtifactBinding,
    pub(crate) interrupted_capture: ArtifactBinding,
    pub(crate) tainted_generation: ArtifactBinding,
    pub(crate) frozen_live_source: ArtifactBinding,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryReason {
    AmbiguousPartialWrite,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryAuthority {
    LocalOsAuthority,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ArtifactLineage {
    TrustedPrefixThenAmbiguousThenQuarantined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PostPrefixDisposition {
    Quarantined,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ByteRange {
    pub(crate) start_inclusive: u64,
    pub(crate) end_exclusive: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum GenerationManifestBody {
    Genesis(GenesisManifestBody),
    RecoveryBaseline(RecoveryManifestBody),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenesisManifestBody {
    pub(crate) schema_version: u32,
    pub(crate) storage_epoch: u32,
    pub(crate) created_at_unix_ms: u64,
    pub(crate) operator: String,
    pub(crate) policy_sha256: Sha256Digest,
    pub(crate) replay_contract_version: u32,
    pub(crate) replay_contract_sha256: Sha256Digest,
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) bootstrap_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryManifestBody {
    pub(crate) schema_version: u32,
    pub(crate) storage_epoch: u32,
    pub(crate) reason: RecoveryReason,
    pub(crate) recovery_authority: RecoveryAuthority,
    pub(crate) recovery_operator: String,
    pub(crate) recovery_policy_sha256: Sha256Digest,
    pub(crate) operator_decision_sha256: Sha256Digest,
    pub(crate) replay_contract_version: u32,
    pub(crate) replay_contract_sha256: Sha256Digest,
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) bootstrap_paths: Vec<String>,
    pub(crate) legacy_source_device: u64,
    pub(crate) legacy_source_inode: u64,
    pub(crate) parent_generation: String,
    pub(crate) incident_at_unix_ms: u64,
    pub(crate) recovered_at_unix_ms: u64,
    pub(crate) trusted_record_count: u64,
    pub(crate) trusted_projection_inventory: TrustedProjectionInventory,
    pub(crate) discarded_range: ByteRange,
    pub(crate) ambiguous_tail_range: ByteRange,
    pub(crate) ambiguous_tail_sha256: Sha256Digest,
    pub(crate) lineage: ArtifactLineage,
    pub(crate) artifacts: RecoveryArtifacts,
    pub(crate) trusted_state_blake3: String,
    pub(crate) frozen_claims: Vec<FrozenClaimSubject>,
    pub(crate) post_prefix_inventory_blake3: String,
    pub(crate) post_prefix_default: PostPrefixDisposition,
    pub(crate) implicit_adoptions: u32,
}

pub(crate) struct CreateGenesisBodyInput {
    pub(crate) created_at_unix_ms: u64,
    pub(crate) operator: String,
    pub(crate) policy_sha256: Sha256Digest,
    pub(crate) replay_contract_version: u32,
    pub(crate) replay_contract_sha256: Sha256Digest,
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) bootstrap_paths: Vec<String>,
}

pub(crate) struct CreateBodyInput {
    pub(crate) recovery_operator: String,
    pub(crate) recovery_policy_sha256: Sha256Digest,
    pub(crate) operator_decision_sha256: Sha256Digest,
    pub(crate) replay_contract_version: u32,
    pub(crate) replay_contract_sha256: Sha256Digest,
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) bootstrap_paths: Vec<String>,
    pub(crate) legacy_source_device: u64,
    pub(crate) legacy_source_inode: u64,
    pub(crate) parent_generation: String,
    pub(crate) incident_at_unix_ms: u64,
    pub(crate) recovered_at_unix_ms: u64,
    pub(crate) trusted_record_count: u64,
    pub(crate) trusted_projection_inventory: TrustedProjectionInventory,
    pub(crate) discarded_range: ByteRange,
    pub(crate) ambiguous_tail_range: ByteRange,
    pub(crate) ambiguous_tail_sha256: Sha256Digest,
    pub(crate) artifacts: RecoveryArtifacts,
    pub(crate) trusted_state_blake3: String,
    pub(crate) frozen_claims: Vec<FrozenClaimSubject>,
    pub(crate) post_prefix_inventory_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenerationManifest {
    pub(crate) generation_id: GenerationId,
    pub(crate) body: GenerationManifestBody,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CurrentPointer {
    pub(super) schema_version: u32,
    pub(super) storage_epoch: u32,
    pub(super) generation_id: GenerationId,
    pub(super) manifest_blake3: String,
}

impl GenerationId {
    pub(crate) fn parse(raw: impl Into<String>) -> Result<Self, CoordError> {
        let id = Self(raw.into());
        validate_prefixed_hex(id.as_str(), "gen_", "GenerationId")?;
        Ok(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Sha256Digest {
    pub(crate) fn parse(raw: impl Into<String>) -> Result<Self, CoordError> {
        let digest = Self(raw.into());
        validate_prefixed_hex(digest.as_str(), "sha256:", "SHA-256 digest")?;
        Ok(digest)
    }

    pub(crate) fn for_bytes(bytes: &[u8]) -> Self {
        Self(format!("sha256:{:x}", Sha256::digest(bytes)))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl RelativeArtifactPath {
    pub(crate) fn parse(raw: impl Into<String>) -> Result<Self, CoordError> {
        let path = Self(raw.into());
        validate_relative_path(path.as_str())?;
        Ok(path)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl ArtifactBinding {
    pub(crate) fn new(
        relative_path: RelativeArtifactPath,
        byte_length: u64,
        record_count: Option<u64>,
        ends_with_lf: bool,
        sha256: Sha256Digest,
    ) -> Result<Self, CoordError> {
        let binding = Self {
            relative_path,
            byte_length,
            record_count,
            ends_with_lf,
            sha256,
        };
        binding.validate()?;
        Ok(binding)
    }

    pub(super) fn validate(&self) -> Result<(), CoordError> {
        validate_relative_path(self.relative_path.as_str())?;
        Sha256Digest::parse(self.sha256.as_str())?;
        if self.byte_length == 0
            || self.byte_length > MAX_ARTIFACT_BYTES
            || self.record_count.is_none()
        {
            return Err(invalid(format!(
                "artifact length must be within 1..={MAX_ARTIFACT_BYTES} bytes and record count must be bound"
            )));
        }
        Ok(())
    }
}

impl GenerationManifestBody {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        match self {
            Self::Genesis(body) => validate_genesis_body(body),
            Self::RecoveryBaseline(body) => validate_recovery_body(body),
        }
    }

    pub(crate) fn recovery(&self) -> Result<&RecoveryManifestBody, CoordError> {
        match self {
            Self::RecoveryBaseline(body) => Ok(body),
            Self::Genesis(_) => Err(invalid(
                "GENESIS manifest has no recovery artifacts or legacy source",
            )),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_common(
    schema_version: u32,
    storage_epoch: u32,
    operator: &str,
    policy_sha256: &Sha256Digest,
    replay_contract_version: u32,
    replay_contract_sha256: &Sha256Digest,
    bootstrap_commit_oid: &str,
    bootstrap_paths: &[String],
) -> Result<(), CoordError> {
    if schema_version != MANIFEST_SCHEMA_VERSION || storage_epoch != STORAGE_EPOCH {
        return Err(invalid(
            "manifest schema version or storage epoch is unsupported",
        ));
    }
    crate::coord::validate_field("operator", operator)?;
    crate::coord::validate_commit_oid(bootstrap_commit_oid)?;
    if replay_contract_version != 1 {
        return Err(invalid("replay contract version must be exactly 1"));
    }
    validate_prefixed_hex(policy_sha256.as_str(), "sha256:", "policy digest")?;
    validate_prefixed_hex(
        replay_contract_sha256.as_str(),
        "sha256:",
        "replay contract digest",
    )?;
    validate_sorted_paths(bootstrap_paths)
}

fn validate_genesis_body(body: &GenesisManifestBody) -> Result<(), CoordError> {
    validate_common(
        body.schema_version,
        body.storage_epoch,
        &body.operator,
        &body.policy_sha256,
        body.replay_contract_version,
        &body.replay_contract_sha256,
        &body.bootstrap_commit_oid,
        &body.bootstrap_paths,
    )?;
    if body.created_at_unix_ms == 0 {
        return Err(invalid("GENESIS creation time must be nonzero"));
    }
    Ok(())
}

fn validate_recovery_body(body: &RecoveryManifestBody) -> Result<(), CoordError> {
    validate_common(
        body.schema_version,
        body.storage_epoch,
        &body.recovery_operator,
        &body.recovery_policy_sha256,
        body.replay_contract_version,
        &body.replay_contract_sha256,
        &body.bootstrap_commit_oid,
        &body.bootstrap_paths,
    )?;
    if body.implicit_adoptions != 0 {
        return Err(invalid("implicit adoptions must be zero"));
    }
    validate_prefixed_hex(
        body.operator_decision_sha256.as_str(),
        "sha256:",
        "operator decision digest",
    )?;
    validate_prefixed_hex(
        body.ambiguous_tail_sha256.as_str(),
        "sha256:",
        "ambiguous tail digest",
    )?;
    validate_prefixed_hex(
        &body.trusted_state_blake3,
        "blake3:",
        "trusted state digest",
    )?;
    validate_prefixed_hex(
        &body.post_prefix_inventory_blake3,
        "blake3:",
        "post-prefix inventory digest",
    )?;
    if body.legacy_source_device == 0 || body.legacy_source_inode == 0 {
        return Err(invalid("legacy source device and inode must be nonzero"));
    }
    validate_parent_generation(&body.parent_generation)?;
    if body.incident_at_unix_ms == 0
        || body.recovered_at_unix_ms <= body.incident_at_unix_ms
        || body.trusted_record_count == 0
    {
        return Err(invalid(
            "manifest time and trusted record fields are invalid",
        ));
    }
    body.trusted_projection_inventory
        .validate(body.trusted_record_count)?;
    validate_artifacts(body)?;
    validate_frozen_claims(&body.frozen_claims)
}

fn validate_artifacts(body: &RecoveryManifestBody) -> Result<(), CoordError> {
    let artifacts = [
        (&body.artifacts.trusted_prefix, TRUSTED_PREFIX_PATH),
        (
            &body.artifacts.interrupted_capture,
            INTERRUPTED_CAPTURE_PATH,
        ),
        (&body.artifacts.tainted_generation, TAINTED_GENERATION_PATH),
        (&body.artifacts.frozen_live_source, FROZEN_LIVE_SOURCE_PATH),
    ];
    for (artifact, expected) in artifacts {
        artifact.validate()?;
        if artifact.relative_path.as_str() != expected {
            return Err(invalid(format!("artifact path must be exactly {expected}")));
        }
    }
    let lengths = [
        body.artifacts.trusted_prefix.byte_length,
        body.artifacts.interrupted_capture.byte_length,
        body.artifacts.tainted_generation.byte_length,
        body.artifacts.frozen_live_source.byte_length,
    ];
    if !lengths.windows(2).all(|pair| pair[0] < pair[1])
        || body.discarded_range.start_inclusive != lengths[0]
        || body.discarded_range.end_exclusive != lengths[3]
        || body.ambiguous_tail_range.start_inclusive != lengths[0]
        || body.ambiguous_tail_range.end_exclusive != lengths[1]
    {
        return Err(invalid(
            "artifact lengths and discarded range do not form the exact recovery lineage",
        ));
    }
    if body.artifacts.trusted_prefix.record_count != Some(body.trusted_record_count)
        || !body.artifacts.trusted_prefix.ends_with_lf
        || body.artifacts.interrupted_capture.record_count != Some(body.trusted_record_count)
        || body.artifacts.interrupted_capture.ends_with_lf
        || body.artifacts.tainted_generation.record_count <= Some(body.trusted_record_count)
        || !body.artifacts.tainted_generation.ends_with_lf
        || body.artifacts.frozen_live_source.record_count
            <= body.artifacts.tainted_generation.record_count
        || !body.artifacts.frozen_live_source.ends_with_lf
    {
        return Err(invalid("artifact LF and record-count lineage is invalid"));
    }
    Ok(())
}

fn validate_frozen_claims(claims: &[FrozenClaimSubject]) -> Result<(), CoordError> {
    if claims.is_empty() {
        return Err(invalid(
            "recovery manifest must freeze at least one trusted claim",
        ));
    }
    let mut previous: Option<&str> = None;
    for claim in claims {
        validate_prefixed_hex(&claim.claim_id, "clm_", "frozen claim ID")?;
        validate_prefixed_hex(&claim.claim_blake3, "blake3:", "frozen claim digest")?;
        if previous.is_some_and(|value| value >= claim.claim_id.as_str()) {
            return Err(invalid("frozen claim IDs must be sorted and unique"));
        }
        previous = Some(&claim.claim_id);
    }
    Ok(())
}

fn validate_sorted_paths(paths: &[String]) -> Result<(), CoordError> {
    if paths.is_empty() {
        return Err(invalid("bootstrap paths must not be empty"));
    }
    let mut previous: Option<&str> = None;
    for path in paths {
        crate::coord::validate_path(path)?;
        if previous.is_some_and(|value| value >= path.as_str()) {
            return Err(invalid("bootstrap paths must be sorted and unique"));
        }
        previous = Some(path);
    }
    Ok(())
}

fn validate_parent_generation(parent: &str) -> Result<(), CoordError> {
    if parent != "legacy-v1" {
        return Err(invalid(
            "schema-1 recovery parent must be exactly legacy-v1",
        ));
    }
    Ok(())
}
