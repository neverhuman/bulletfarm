use std::fs::File;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::*;

pub(super) const CREATION_PLAN: &str = "genesis-fence-create-plan.json";
pub(super) const CREATION_OBSERVATION: &str = "genesis-fence-create-observation.json";
pub(super) const FENCE_INTENT: &str = "genesis-fence-publish-intent.json";
pub(super) const SEAL_OBSERVATION: &str = "genesis-fence-seal-observation.json";
pub(super) const PUBLICATION_PLAN: &str = "genesis-fence-publication-plan.json";
pub(super) const PUBLICATION_OBSERVATION: &str = "genesis-fence-observation.json";

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FenceCreationPlan {
    pub(super) kind: String,
    pub(super) schema_version: u32,
    pub(super) generation_id: String,
    pub(super) initialization_intent_blake3: String,
    pub(super) sibling_name: String,
    pub(super) parent_device: u64,
    pub(super) parent_inode: u64,
    pub(super) parent_owner: u32,
    pub(super) intended_mode: u32,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FenceCreationObservation {
    pub(super) kind: String,
    pub(super) schema_version: u32,
    pub(super) generation_id: String,
    pub(super) creation_plan_blake3: String,
    pub(super) empty_inventory_blake3: String,
    pub(super) observed_name: String,
    pub(super) device: u64,
    pub(super) inode: u64,
    pub(super) owner: u32,
    pub(super) file_type: String,
    pub(super) mode: u32,
    pub(super) links: u64,
    pub(super) size: u64,
    pub(super) ctime_seconds: i64,
    pub(super) ctime_nanoseconds: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FencePublishIntent {
    pub(super) kind: String,
    pub(super) schema_version: u32,
    pub(super) generation_id: String,
    pub(super) initialization_intent_blake3: String,
    pub(super) creation_observation_blake3: String,
    pub(super) empty_inventory_blake3: String,
    pub(super) sibling_name: String,
    pub(super) device: u64,
    pub(super) inode: u64,
    pub(super) owner: u32,
    pub(super) file_type: String,
    pub(super) mode: u32,
    pub(super) links: u64,
    pub(super) size: u64,
    pub(super) ctime_seconds: i64,
    pub(super) ctime_nanoseconds: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FencePublicationPlan {
    pub(super) kind: String,
    pub(super) schema_version: u32,
    pub(super) generation_id: String,
    pub(super) fence_intent_blake3: String,
    pub(super) seal_observation_blake3: String,
    pub(super) empty_inventory_blake3: String,
    pub(super) sibling_name: String,
    pub(super) authority_name: String,
    pub(super) device: u64,
    pub(super) inode: u64,
    pub(super) owner: u32,
    pub(super) file_type: String,
    pub(super) mode: u32,
    pub(super) links: u64,
    pub(super) size: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FenceObservation {
    pub(super) kind: String,
    pub(super) schema_version: u32,
    pub(super) generation_id: String,
    pub(super) fence_intent_blake3: String,
    pub(super) empty_inventory_blake3: String,
    pub(super) phase: String,
    pub(super) observed_name: String,
    pub(super) device: u64,
    pub(super) inode: u64,
    pub(super) owner: u32,
    pub(super) file_type: String,
    pub(super) mode: u32,
    pub(super) links: u64,
    pub(super) size: u64,
    pub(super) ctime_seconds: i64,
    pub(super) ctime_nanoseconds: u64,
}

pub(super) fn validate_preseal(
    intent: &FencePublishIntent,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
    creation_observation: &[u8],
) -> Result<(), CoordError> {
    if intent.kind != "coord_genesis_fence_publish_intent_v2"
        || intent.schema_version != 2
        || intent.generation_id != generation_id
        || intent.initialization_intent_blake3
            != digest(
                "bullet.coord.genesis-initialization-intent.v2",
                initialization_intent,
            )?
        || intent.creation_observation_blake3
            != digest(
                "bullet.coord.genesis-fence-create-observation.v2",
                creation_observation,
            )?
        || intent.empty_inventory_blake3 != empty_inventory_digest()?
        || intent.sibling_name != sibling_name
        || intent.device == 0
        || intent.inode == 0
        || intent.owner != owner()
        || intent.file_type != "directory"
        || intent.mode != 0o700
        || intent.links != 2
    {
        return Err(unknown("Genesis fence publish intent is invalid"));
    }
    Ok(())
}

pub(super) fn validate_seal_history(
    parent: &File,
    generation_id: &str,
    intent_bytes: &[u8],
    intent: &FencePublishIntent,
) -> Result<(), CoordError> {
    let bytes = read_named(parent, SEAL_OBSERVATION)?;
    let observation: FenceObservation = decode(&bytes)?;
    if observation.kind != "coord_genesis_fence_observation_v2"
        || observation.schema_version != 2
        || observation.generation_id != generation_id
        || observation.fence_intent_blake3
            != digest("bullet.coord.genesis-fence-publish-intent.v2", intent_bytes)?
        || observation.empty_inventory_blake3 != intent.empty_inventory_blake3
        || observation.phase != "SEALED_SIBLING"
        || observation.observed_name != intent.sibling_name
        || observation.device != intent.device
        || observation.inode != intent.inode
        || observation.owner != intent.owner
        || observation.file_type != "directory"
        || observation.mode != 0
        || observation.links != intent.links
        || observation.size != intent.size
    {
        return Err(unknown("Genesis fence seal observation is invalid"));
    }
    Ok(())
}

pub(super) fn empty_inventory_digest() -> Result<String, CoordError> {
    digest("bullet.coord.genesis-fence-empty-inventory.v2", b"[]")
}

pub(super) fn read_named(parent: &File, name: &str) -> Result<Vec<u8>, CoordError> {
    let mut file = open_file_at(parent, name, false, 0o400, None)
        .map_err(|_| unknown("Genesis fence evidence is missing"))?;
    read_canonical(&mut file).map_err(|_| unknown("Genesis fence evidence cannot be read"))
}

pub(super) fn exact_named(parent: &File, name: &str, bytes: &[u8]) -> Result<(), CoordError> {
    let mut file = open_file_at(parent, name, false, 0o400, None)?;
    exact(&mut file, bytes)
}

pub(super) fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, CoordError> {
    bullet_wire::canonical_json(value)
        .map_err(|error| unknown(format!("cannot encode Genesis fence evidence: {error}")))
}

pub(super) fn decode<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T, CoordError> {
    let value = bullet_wire::decode_unique_value(bytes)
        .map_err(|error| unknown(format!("cannot decode Genesis fence evidence: {error}")))?;
    let value: T = serde_json::from_value(value)
        .map_err(|error| unknown(format!("cannot decode Genesis fence evidence: {error}")))?;
    if canonical(&value)? != bytes {
        return Err(unknown(
            "Genesis fence evidence is not exact canonical JSON",
        ));
    }
    Ok(value)
}

pub(super) fn digest(domain: &str, bytes: &[u8]) -> Result<String, CoordError> {
    bullet_wire::hash_framed_bytes(domain, bytes)
        .map(|value| format!("blake3:{}", value.to_hex()))
        .map_err(|error| unknown(format!("cannot hash Genesis fence evidence: {error}")))
}
