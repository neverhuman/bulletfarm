use serde::{Deserialize, Serialize};

use crate::coord::{
    CoordError,
    generation::manifest::{
        self, CreateGenesisBodyInput, CurrentPointer, GenerationManifest, GenerationManifestBody,
        Sha256Digest,
    },
};

const INTENT_KIND: &str = "coord_genesis_initialization_intent_v2";
const INTENT_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord::store) struct GenesisProvenance {
    pub(in crate::coord::store) operator: String,
    pub(in crate::coord::store) policy_sha256: Sha256Digest,
    pub(in crate::coord::store) replay_contract_version: u32,
    pub(in crate::coord::store) replay_contract_sha256: Sha256Digest,
    pub(in crate::coord::store) bootstrap_commit_oid: String,
    pub(in crate::coord::store) bootstrap_paths: Vec<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct GenesisInitializationIntent {
    kind: String,
    schema_version: u32,
    provenance: GenesisProvenance,
    created_at_unix_ms: u64,
    manifest: GenerationManifest,
    current: CurrentPointer,
}

pub(super) struct PreparedGenesis {
    pub(super) manifest: GenerationManifest,
    pub(super) current: CurrentPointer,
    pub(super) intent_bytes: Vec<u8>,
}

pub(super) fn prepare(
    provenance: &GenesisProvenance,
    created_at_unix_ms: u64,
) -> Result<PreparedGenesis, CoordError> {
    let provenance = normalized(provenance)?;
    let manifest = manifest(&provenance, created_at_unix_ms)?;
    let current = CurrentPointer::for_manifest(&manifest)?;
    let intent = GenesisInitializationIntent {
        kind: INTENT_KIND.to_owned(),
        schema_version: INTENT_SCHEMA_VERSION,
        provenance,
        created_at_unix_ms,
        manifest: manifest.clone(),
        current: current.clone(),
    };
    let intent_bytes = bullet_wire::canonical_json(&intent).map_err(wire)?;
    Ok(PreparedGenesis {
        manifest,
        current,
        intent_bytes,
    })
}

pub(super) fn decode(
    bytes: &[u8],
    requested: &GenesisProvenance,
) -> Result<PreparedGenesis, CoordError> {
    let prepared = decode_authority(bytes)?;
    let intent = decode_intent(bytes)?;
    ensure_same_provenance(requested, &intent.provenance)?;
    Ok(prepared)
}

pub(super) fn decode_for_presence(
    bytes: &[u8],
    requested: &GenesisProvenance,
    fence_is_sealed: bool,
) -> Result<PreparedGenesis, CoordError> {
    decode(bytes, requested).map_err(|error| {
        if fence_is_sealed && error.code() != "COORD_GENESIS_CONFLICT" {
            CoordError::new(
                "COORD_FENCE_UNKNOWN",
                "sealed Genesis fence has an invalid initialization intent",
            )
        } else {
            error
        }
    })
}

pub(super) fn decode_authority(bytes: &[u8]) -> Result<PreparedGenesis, CoordError> {
    let intent = decode_intent(bytes)?;
    if bullet_wire::canonical_json(&intent).map_err(wire)? != bytes {
        return Err(invalid(
            "Genesis initialization intent is not exact canonical JSON",
        ));
    }
    if intent.kind != INTENT_KIND || intent.schema_version != INTENT_SCHEMA_VERSION {
        return Err(invalid(
            "Genesis initialization intent kind or schema is unsupported",
        ));
    }
    let expected = prepare(&intent.provenance, intent.created_at_unix_ms)?;
    if expected.manifest != intent.manifest || expected.current != intent.current {
        return Err(invalid(
            "Genesis initialization intent does not bind its exact manifest and CURRENT",
        ));
    }
    Ok(PreparedGenesis {
        manifest: intent.manifest,
        current: intent.current,
        intent_bytes: bytes.to_vec(),
    })
}

fn decode_intent(bytes: &[u8]) -> Result<GenesisInitializationIntent, CoordError> {
    let value = bullet_wire::decode_unique_value(bytes).map_err(|error| {
        invalid(format!(
            "cannot decode Genesis initialization intent: {error}"
        ))
    })?;
    serde_json::from_value(value).map_err(|error| {
        invalid(format!(
            "cannot decode Genesis initialization intent: {error}"
        ))
    })
}

pub(super) fn ensure_manifest_matches(
    requested: &GenesisProvenance,
    actual: &GenerationManifest,
) -> Result<(), CoordError> {
    let GenerationManifestBody::Genesis(body) = &actual.body else {
        return Err(conflict(
            "CURRENT is a recovery generation, not the requested Genesis subject",
        ));
    };
    let expected = manifest(&normalized(requested)?, body.created_at_unix_ms)?;
    if &expected == actual {
        Ok(())
    } else {
        Err(conflict(
            "Genesis provenance differs from the durable initialized subject",
        ))
    }
}

fn ensure_same_provenance(
    requested: &GenesisProvenance,
    actual: &GenesisProvenance,
) -> Result<(), CoordError> {
    if normalized(requested)? == normalized(actual)? {
        Ok(())
    } else {
        Err(conflict(
            "Genesis provenance differs from the durable initialization intent",
        ))
    }
}

fn normalized(value: &GenesisProvenance) -> Result<GenesisProvenance, CoordError> {
    let mut normalized = value.clone();
    normalized.bootstrap_paths.sort();
    manifest(&normalized, 1)?;
    Ok(normalized)
}

fn manifest(
    provenance: &GenesisProvenance,
    created_at_unix_ms: u64,
) -> Result<GenerationManifest, CoordError> {
    GenerationManifest::from_body(manifest::create_genesis_body(CreateGenesisBodyInput {
        created_at_unix_ms,
        operator: provenance.operator.clone(),
        policy_sha256: provenance.policy_sha256.clone(),
        replay_contract_version: provenance.replay_contract_version,
        replay_contract_sha256: provenance.replay_contract_sha256.clone(),
        bootstrap_commit_oid: provenance.bootstrap_commit_oid.clone(),
        bootstrap_paths: provenance.bootstrap_paths.clone(),
    })?)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_GENESIS_INTENT", reason)
}

fn conflict(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_GENESIS_CONFLICT", reason)
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!(
        "cannot encode Genesis initialization intent: {error}"
    ))
}
