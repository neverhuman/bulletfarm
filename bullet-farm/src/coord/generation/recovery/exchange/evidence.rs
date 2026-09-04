use std::{fs::File, os::unix::fs::MetadataExt};

use serde::{Deserialize, Serialize};

use super::super::authority::{BaselineSubject, metadata::write_or_verify_sealed};
use crate::coord::{
    CoordError,
    generation::manifest::{CurrentPointer, GenerationManifest, Sha256Digest},
};

const PREPARED: &str = "prepared-tombstone-seal-observation.json";
const RETIREMENT: &str = "retirement-completion-observation.json";
const EMPTY_DOMAIN: &str = "bullet.coord.empty-directory-inventory.v2";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectoryFacts {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    link_count: u64,
    byte_size: u64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceFacts {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    link_count: u64,
    byte_length: u64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PreparedSealObservationV2 {
    schema_version: u32,
    kind: String,
    generation_id: String,
    manifest_blake3: String,
    intent_sha256: String,
    request_digest: String,
    sibling_name: String,
    sibling_position: String,
    empty_inventory_blake3: String,
    tombstone: DirectoryFacts,
}

#[derive(Serialize)]
struct RetirementCompletionObservationV2<'a> {
    schema_version: u32,
    kind: &'static str,
    generation_id: &'a str,
    manifest_blake3: &'a str,
    intent_sha256: &'a str,
    prepared_observation_sha256: &'a str,
    tombstone_observation_sha256: &'a str,
    request_digest: &'a str,
    top_path: &'static str,
    sibling_name: &'a str,
    sibling_absent: bool,
    retired_path: String,
    tombstone: DirectoryFacts,
    retired_source: SourceFacts,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepared(
    recovery_dir: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    sibling_name: &str,
    tombstone: &File,
    owner: u32,
    allow_create: bool,
) -> Result<String, CoordError> {
    let pointer = CurrentPointer::for_manifest(manifest)?;
    let facts = directory_facts(tombstone, owner)?;
    let expected = PreparedSealObservationV2 {
        schema_version: 2,
        kind: "coord_prepared_tombstone_seal_observation_v2".to_owned(),
        generation_id: manifest.generation_id().as_str().to_owned(),
        manifest_blake3: pointer.manifest_blake3().to_owned(),
        intent_sha256: intent_sha256.to_owned(),
        request_digest: baseline.request_digest.clone(),
        sibling_name: sibling_name.to_owned(),
        sibling_position: "coord-root-sibling".to_owned(),
        empty_inventory_blake3: empty_inventory_digest()?,
        tombstone: facts,
    };
    let bytes = if allow_create {
        let bytes = canonical_line(&expected)?;
        write_or_verify_sealed(
            recovery_dir,
            PREPARED,
            &bytes,
            true,
            "TOMBSTONE_SEAL_OUTCOME_UNKNOWN",
        )?;
        bytes
    } else {
        let bytes = super::super::authority::metadata::read_sealed(
            recovery_dir,
            PREPARED,
            "TOMBSTONE_SEAL_OUTCOME_UNKNOWN",
        )?
        .ok_or_else(|| unknown("prepared tombstone observation is missing"))?;
        let value = bullet_wire::decode_unique_value(&bytes)
            .map_err(|error| unknown(format!("cannot decode prepared observation: {error}")))?;
        let observed: PreparedSealObservationV2 = serde_json::from_value(value)
            .map_err(|error| unknown(format!("cannot decode prepared observation: {error}")))?;
        if canonical_line(&observed)? != bytes
            || observed.schema_version != expected.schema_version
            || observed.kind != expected.kind
            || observed.generation_id != expected.generation_id
            || observed.manifest_blake3 != expected.manifest_blake3
            || observed.intent_sha256 != expected.intent_sha256
            || observed.request_digest != expected.request_digest
            || observed.sibling_name != expected.sibling_name
            || observed.sibling_position != expected.sibling_position
            || observed.empty_inventory_blake3 != expected.empty_inventory_blake3
            || !same_stable_directory(observed.tombstone, facts)
        {
            return Err(unknown("prepared tombstone observation differs"));
        }
        bytes
    };
    Ok(Sha256Digest::for_bytes(&bytes).as_str().to_owned())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn retirement(
    recovery_dir: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    prepared_observation_sha256: &str,
    tombstone_observation_sha256: &str,
    sibling_name: &str,
    tombstone: &File,
    retired_source: &File,
    owner: u32,
    allow_create: bool,
) -> Result<(), CoordError> {
    let pointer = CurrentPointer::for_manifest(manifest)?;
    let bytes = canonical_line(&RetirementCompletionObservationV2 {
        schema_version: 2,
        kind: "coord_retirement_completion_observation_v2",
        generation_id: manifest.generation_id().as_str(),
        manifest_blake3: pointer.manifest_blake3(),
        intent_sha256,
        prepared_observation_sha256,
        tombstone_observation_sha256,
        request_digest: &baseline.request_digest,
        top_path: "events.jsonl",
        sibling_name,
        sibling_absent: true,
        retired_path: format!(
            "recovery/{}/retired-v1.non-authoritative",
            manifest.generation_id().as_str()
        ),
        tombstone: directory_facts(tombstone, owner)?,
        retired_source: source_facts(retired_source, owner)?,
    })?;
    write_or_verify_sealed(
        recovery_dir,
        RETIREMENT,
        &bytes,
        allow_create,
        "COORD_RETIREMENT_OUTCOME_UNKNOWN",
    )
}

fn same_stable_directory(left: DirectoryFacts, right: DirectoryFacts) -> bool {
    left.device == right.device
        && left.inode == right.inode
        && left.owner == right.owner
        && left.mode == right.mode
        && left.link_count == right.link_count
        && left.byte_size == right.byte_size
}

fn directory_facts(file: &File, owner: u32) -> Result<DirectoryFacts, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir()
        || metadata.uid() != owner
        || metadata.mode() & 0o7777 != 0o400
        || metadata.nlink() != 2
    {
        return Err(unknown("sealed tombstone metadata is not exact"));
    }
    super::super::platform_fs::require_empty_descriptor(file)?;
    Ok(DirectoryFacts {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner: metadata.uid(),
        mode: metadata.mode() & 0o7777,
        link_count: metadata.nlink(),
        byte_size: metadata.len(),
        ctime_seconds: metadata.ctime(),
        ctime_nanoseconds: metadata.ctime_nsec(),
    })
}

fn source_facts(file: &File, owner: u32) -> Result<SourceFacts, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.mode() & 0o7777 != 0o400
        || metadata.nlink() != 1
    {
        return Err(CoordError::new(
            "COORD_RETIREMENT_OUTCOME_UNKNOWN",
            "retired legacy source metadata is not exact",
        ));
    }
    Ok(SourceFacts {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner: metadata.uid(),
        mode: metadata.mode() & 0o7777,
        link_count: metadata.nlink(),
        byte_length: metadata.len(),
        ctime_seconds: metadata.ctime(),
        ctime_nanoseconds: metadata.ctime_nsec(),
    })
}

fn empty_inventory_digest() -> Result<String, CoordError> {
    let bytes = bullet_wire::canonical_json(&serde_json::json!({ "children": [] }))
        .map_err(|error| unknown(format!("cannot encode empty inventory: {error}")))?;
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_framed_bytes(EMPTY_DOMAIN, &bytes)
            .map_err(|error| unknown(format!("cannot hash empty inventory: {error}")))?
            .to_hex()
    ))
}

fn canonical_line(value: &impl Serialize) -> Result<Vec<u8>, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value)
        .map_err(|error| unknown(format!("cannot encode exchange evidence: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("TOMBSTONE_SEAL_OUTCOME_UNKNOWN", reason)
}
