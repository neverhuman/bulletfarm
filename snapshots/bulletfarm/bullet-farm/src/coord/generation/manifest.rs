use std::path::{Component, Path};

#[cfg(test)]
use std::path::PathBuf;

use serde::{Serialize, de::DeserializeOwned};

use crate::coord::CoordError;

mod inventory;
#[cfg(test)]
mod io;
mod types;

pub(crate) use inventory::{
    TrustedClaimOutcomeCounts, TrustedProjectionInventory, TrustedRecordKindCounts,
};
pub(crate) use types::RelativeArtifactPath;
pub(crate) use types::{
    ArtifactBinding, ArtifactLineage, ByteRange, CreateBodyInput, CreateGenesisBodyInput,
    CurrentPointer, GenerationId, GenerationManifest, GenerationManifestBody, GenesisManifestBody,
    PostPrefixDisposition, RecoveryArtifacts, RecoveryAuthority, RecoveryManifestBody,
    RecoveryReason, Sha256Digest,
};

pub(crate) const STORAGE_EPOCH: u32 = 2;
pub(crate) const MANIFEST_SCHEMA_VERSION: u32 = 2;
pub(crate) const CURRENT_POINTER_SCHEMA_VERSION: u32 = 2;
#[cfg(test)]
pub(crate) const MANIFEST_FILE: &str = "manifest.json";
#[cfg(test)]
pub(crate) const CURRENT_FILE: &str = "CURRENT";
pub(crate) const TRUSTED_PREFIX_PATH: &str = "archive/trusted-prefix.jsonl";
pub(crate) const INTERRUPTED_CAPTURE_PATH: &str = "archive/interrupted-observation.jsonl.partial";
pub(crate) const TAINTED_GENERATION_PATH: &str = "archive/tainted-generation.jsonl";
pub(crate) const FROZEN_LIVE_SOURCE_PATH: &str = "archive/frozen-live-source.jsonl";

const GENERATION_ID_DOMAIN: &[u8] = b"bullet.coord.generation.v2\0";
const MANIFEST_DIGEST_DOMAIN: &[u8] = b"bullet.coord.generation-manifest.v2\0";
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES;

pub(crate) fn create_body(
    mut input: CreateBodyInput,
) -> Result<GenerationManifestBody, CoordError> {
    input.bootstrap_paths.sort();
    input
        .frozen_claims
        .sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    if input
        .frozen_claims
        .windows(2)
        .any(|pair| pair[0].claim_id == pair[1].claim_id)
    {
        return Err(invalid("frozen claim IDs must be unique"));
    }
    let body = RecoveryManifestBody {
        schema_version: MANIFEST_SCHEMA_VERSION,
        storage_epoch: STORAGE_EPOCH,
        reason: RecoveryReason::AmbiguousPartialWrite,
        recovery_authority: RecoveryAuthority::LocalOsAuthority,
        recovery_operator: input.recovery_operator,
        recovery_policy_sha256: input.recovery_policy_sha256,
        operator_decision_sha256: input.operator_decision_sha256,
        replay_contract_version: input.replay_contract_version,
        replay_contract_sha256: input.replay_contract_sha256,
        bootstrap_commit_oid: input.bootstrap_commit_oid,
        bootstrap_paths: input.bootstrap_paths,
        legacy_source_device: input.legacy_source_device,
        legacy_source_inode: input.legacy_source_inode,
        parent_generation: input.parent_generation,
        incident_at_unix_ms: input.incident_at_unix_ms,
        recovered_at_unix_ms: input.recovered_at_unix_ms,
        trusted_record_count: input.trusted_record_count,
        trusted_projection_inventory: input.trusted_projection_inventory,
        discarded_range: input.discarded_range,
        ambiguous_tail_range: input.ambiguous_tail_range,
        ambiguous_tail_sha256: input.ambiguous_tail_sha256,
        lineage: ArtifactLineage::TrustedPrefixThenAmbiguousThenQuarantined,
        artifacts: input.artifacts,
        trusted_state_blake3: input.trusted_state_blake3,
        frozen_claims: input.frozen_claims,
        post_prefix_inventory_blake3: input.post_prefix_inventory_blake3,
        post_prefix_default: PostPrefixDisposition::Quarantined,
        implicit_adoptions: 0,
    };
    let body = GenerationManifestBody::RecoveryBaseline(body);
    body.validate()?;
    Ok(body)
}

pub(crate) fn create_genesis_body(
    mut input: CreateGenesisBodyInput,
) -> Result<GenerationManifestBody, CoordError> {
    input.bootstrap_paths.sort();
    let body = GenerationManifestBody::Genesis(GenesisManifestBody {
        schema_version: MANIFEST_SCHEMA_VERSION,
        storage_epoch: STORAGE_EPOCH,
        created_at_unix_ms: input.created_at_unix_ms,
        operator: input.operator,
        policy_sha256: input.policy_sha256,
        replay_contract_version: input.replay_contract_version,
        replay_contract_sha256: input.replay_contract_sha256,
        bootstrap_commit_oid: input.bootstrap_commit_oid,
        bootstrap_paths: input.bootstrap_paths,
    });
    body.validate()?;
    Ok(body)
}

impl GenerationManifest {
    pub(crate) fn from_body(body: GenerationManifestBody) -> Result<Self, CoordError> {
        body.validate()?;
        Ok(Self {
            generation_id: generation_id(&body)?,
            body,
        })
    }

    pub(crate) fn generation_id(&self) -> &GenerationId {
        &self.generation_id
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        self.body.validate()?;
        if self.generation_id != generation_id(&self.body)? {
            return Err(invalid(
                "GenerationId does not bind the canonical manifest body",
            ));
        }
        Ok(())
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        canonical_line(self)
    }

    pub(crate) fn decode_canonical(bytes: &[u8]) -> Result<Self, CoordError> {
        let manifest: Self = decode_canonical_line(bytes)?;
        manifest.validate()?;
        Ok(manifest)
    }
}

pub(crate) fn generation_id(body: &GenerationManifestBody) -> Result<GenerationId, CoordError> {
    body.validate()?;
    let canonical = bullet_wire::canonical_json(body).map_err(wire)?;
    GenerationId::parse(format!(
        "gen_{}",
        blake3_hex(GENERATION_ID_DOMAIN, &canonical)
    ))
}

impl CurrentPointer {
    pub(crate) fn for_manifest(manifest: &GenerationManifest) -> Result<Self, CoordError> {
        manifest.validate()?;
        let bytes = manifest.canonical_bytes()?;
        Ok(Self {
            schema_version: CURRENT_POINTER_SCHEMA_VERSION,
            storage_epoch: STORAGE_EPOCH,
            generation_id: manifest.generation_id.clone(),
            manifest_blake3: format!("blake3:{}", blake3_hex(MANIFEST_DIGEST_DOMAIN, &bytes)),
        })
    }

    pub(crate) fn generation_id(&self) -> &GenerationId {
        &self.generation_id
    }

    pub(crate) fn manifest_blake3(&self) -> &str {
        &self.manifest_blake3
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.schema_version != CURRENT_POINTER_SCHEMA_VERSION
            || self.storage_epoch != STORAGE_EPOCH
        {
            return Err(invalid(
                "CURRENT schema version or storage epoch is unsupported",
            ));
        }
        GenerationId::parse(self.generation_id.as_str())?;
        validate_prefixed_hex(&self.manifest_blake3, "blake3:", "manifest BLAKE3 digest")
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        canonical_line(self)
    }

    pub(crate) fn decode_canonical(bytes: &[u8]) -> Result<Self, CoordError> {
        let pointer: Self = decode_canonical_line(bytes)?;
        pointer.validate()?;
        Ok(pointer)
    }

    pub(crate) fn verify_manifest(&self, manifest: &GenerationManifest) -> Result<(), CoordError> {
        self.validate()?;
        if self.generation_id != manifest.generation_id
            || self.manifest_blake3
                != format!(
                    "blake3:{}",
                    blake3_hex(MANIFEST_DIGEST_DOMAIN, &manifest.canonical_bytes()?)
                )
        {
            return Err(invalid(
                "CURRENT does not bind the exact generation manifest",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn write_generation_manifest(
    temp_dir: &Path,
    manifest: &GenerationManifest,
) -> Result<PathBuf, CoordError> {
    let bytes = manifest.canonical_bytes()?;
    io::write_immutable_relative(temp_dir, MANIFEST_FILE, &bytes)?;
    let loaded = load_and_verify(temp_dir, manifest.generation_id())?;
    if loaded != *manifest {
        return Err(invalid("generation manifest read-back changed"));
    }
    Ok(temp_dir.join(MANIFEST_FILE))
}

#[cfg(test)]
pub(crate) fn load_and_verify(
    generation_dir: &Path,
    expected_id: &GenerationId,
) -> Result<GenerationManifest, CoordError> {
    let bytes = io::read_relative(
        generation_dir,
        MANIFEST_FILE,
        MAX_MANIFEST_BYTES as u64,
        0o400,
    )?;
    let manifest = GenerationManifest::decode_canonical(&bytes)?;
    if manifest.generation_id() != expected_id {
        return Err(invalid(
            "manifest GenerationId differs from the expected generation",
        ));
    }
    Ok(manifest)
}

#[cfg(test)]
pub(crate) fn load_current(coord_dir: &Path) -> Result<Option<CurrentPointer>, CoordError> {
    io::read_optional_relative(coord_dir, CURRENT_FILE, MAX_MANIFEST_BYTES as u64, 0o400)?
        .map(|bytes| CurrentPointer::decode_canonical(&bytes))
        .transpose()
}

#[cfg(test)]
pub(crate) fn verify_artifact(
    generation_dir: &Path,
    binding: &ArtifactBinding,
    expected_path: &RelativeArtifactPath,
) -> Result<(), CoordError> {
    binding.validate()?;
    if &binding.relative_path != expected_path {
        return Err(invalid(
            "artifact is bound to the wrong fixed relative path",
        ));
    }
    let bytes = io::read_relative(
        generation_dir,
        binding.relative_path.as_str(),
        binding.byte_length,
        0o400,
    )?;
    if bytes.len() as u64 != binding.byte_length
        || Sha256Digest::for_bytes(&bytes) != binding.sha256
    {
        return Err(invalid(
            "artifact length or SHA-256 digest differs from its binding",
        ));
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), CoordError> {
    let subject = Path::new(path);
    if path.is_empty()
        || path.len() > 240
        || path.contains('\\')
        || path.bytes().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'/' | b'.' | b'-' | b'_'))
        })
        || subject.is_absolute()
        || subject
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(
            "artifact path must be normalized lowercase relative ASCII",
        ));
    }
    Ok(())
}

fn validate_prefixed_hex(raw: &str, prefix: &str, label: &str) -> Result<(), CoordError> {
    let hex = raw
        .strip_prefix(prefix)
        .filter(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| invalid(format!("{label} must be {prefix}<64 lowercase hex>")))?;
    debug_assert_eq!(hex.len(), 64);
    Ok(())
}

fn canonical_line<T: Serialize>(value: &T) -> Result<Vec<u8>, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value).map_err(wire)?;
    if bytes.len() >= MAX_MANIFEST_BYTES {
        return Err(invalid(
            "canonical coordination metadata exceeds its byte bound",
        ));
    }
    bytes.push(b'\n');
    Ok(bytes)
}

fn decode_canonical_line<T>(bytes: &[u8]) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    if bytes.len() > MAX_MANIFEST_BYTES || bytes.last() != Some(&b'\n') {
        return Err(invalid(
            "coordination metadata must be bounded and end in one LF",
        ));
    }
    let body = &bytes[..bytes.len() - 1];
    let value = bullet_wire::decode_unique_value_bounded(body, MAX_MANIFEST_BYTES)
        .map_err(|error| invalid(format!("coordination metadata is not strict JSON: {error}")))?;
    let decoded: T = serde_json::from_value(value).map_err(|error| {
        invalid(format!(
            "coordination metadata does not match its schema: {error}"
        ))
    })?;
    if bullet_wire::canonical_json(&decoded).map_err(wire)? != body {
        return Err(invalid(
            "coordination metadata is not canonical RFC-8785 JSON",
        ));
    }
    Ok(decoded)
}

fn blake3_hex(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("RFC-8785 encoding failed: {error}"))
}

fn invalid(detail: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_GENERATION", detail)
}

#[cfg(test)]
#[path = "manifest/tests.rs"]
mod tests;
