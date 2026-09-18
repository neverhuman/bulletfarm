//! Replay supplied sealed copies. This grants no review, retirement or Genesis authority.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    CoordError,
    fresh_genesis::FreshGenesisPublicationOutcome,
    generation::{manifest::TrustedProjectionInventory, recovery::projection},
    model::{
        ClaimState, ClaimSummary, FreshPreservationSubjectV1, IncidentInventoryNodeTypeV1,
        IncidentInventoryV1,
    },
    recovery_manifest::require_normalized_absolute,
    sealed, state,
    store::{legacy, subject::record_time},
};

const MAX_BYTES: u64 = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64;
const MAX_DISPOSITIONS: usize = 2_048;
const CLAIM_DOMAIN: &str = "bullet-family.coord.fresh-replay-claim.v1";
const REPLAY_DOMAIN: &str = "bullet-family.coord.fresh-replay-subject.v1";

mod readback;
pub(crate) use readback::verify;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Location {
    Outer,
    Hub,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RequestKind {
    FreshReplayRequestV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PreservationReference {
    path: PathBuf,
    preservation_id: String,
    sealed_sha256: String,
    byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
enum Disposition {
    RecordedReceipt { commit_oid: String },
    RetainForRecovery,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClaimDisposition {
    location: Location,
    claim_id: String,
    claim_subject_id: String,
    disposition: Disposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct FreshReplayRequestV1 {
    kind: RequestKind,
    schema_version: u32,
    preservation: PreservationReference,
    outer_ledger_copy: PathBuf,
    hub_ledger_copy: PathBuf,
    dispositions: Vec<ClaimDisposition>,
}

#[derive(Serialize)]
struct LocationReplay {
    inventory_id: String,
    ledger_sha256: String,
    byte_length: u64,
    record_count: u64,
    as_of_event_unix_ms: u64,
    projection: TrustedProjectionInventory,
    claims: BTreeMap<String, ClaimSummary>,
}

#[derive(Serialize)]
struct ReplayFacts {
    kind: &'static str,
    schema_version: u32,
    purpose: &'static str,
    request: FreshReplayRequestV1,
    outer: LocationReplay,
    hub: LocationReplay,
}

#[derive(Serialize)]
struct FreshReplaySubjectV1 {
    replay_id: String,
    facts: ReplayFacts,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplayPublication {
    replay_id: String,
    sealed_sha256: String,
    byte_length: u64,
    outcome: FreshGenesisPublicationOutcome,
}

pub(crate) fn publish(
    root: &Path,
    request_path: &Path,
    output: &Path,
) -> Result<ReplayPublication, CoordError> {
    require_normalized_absolute(root, "replay family root")?;
    outside(root, request_path)?;
    outside(root, output)?;
    let request: FreshReplayRequestV1 = sealed::read(request_path)?;
    let request_bytes = canonical_lf(&request)?;
    let reconstructed = reconstruct(root, request, &[request_path, output])?;
    let mut inputs = reconstructed.inputs().to_vec();
    inputs.push((request_path, request_bytes.as_slice()));
    revalidate(&inputs)?;
    let outcome = match sealed::write(output, &reconstructed.subject) {
        Ok(()) => FreshGenesisPublicationOutcome::Created,
        Err(_) => FreshGenesisPublicationOutcome::AdoptedExactExisting,
    };
    if sealed::read_synced_raw(output, MAX_BYTES + 1)? != reconstructed.bytes {
        return Err(changed(
            "replay output differs from the exact intended subject",
        ));
    }
    revalidate(&inputs)?;
    Ok(ReplayPublication {
        replay_id: reconstructed.subject.replay_id.clone(),
        sealed_sha256: sha256(&reconstructed.bytes),
        byte_length: reconstructed.bytes.len() as u64,
        outcome,
    })
}

struct Reconstruction {
    subject: FreshReplaySubjectV1,
    bytes: Vec<u8>,
    preservation_bytes: Vec<u8>,
    outer_bytes: Vec<u8>,
    hub_bytes: Vec<u8>,
}

impl Reconstruction {
    fn inputs(&self) -> [(&Path, &[u8]); 3] {
        let request = &self.subject.facts.request;
        [
            (&request.preservation.path, &self.preservation_bytes),
            (&request.outer_ledger_copy, &self.outer_bytes),
            (&request.hub_ledger_copy, &self.hub_bytes),
        ]
    }
}

fn reconstruct(
    root: &Path,
    request: FreshReplayRequestV1,
    record_paths: &[&Path],
) -> Result<Reconstruction, CoordError> {
    require_normalized_absolute(root, "replay family root")?;
    if request.schema_version != 1 || request.dispositions.len() > MAX_DISPOSITIONS {
        return Err(invalid(
            "replay request schema or disposition bound is unsupported",
        ));
    }
    let mut paths = record_paths.to_vec();
    paths.extend([
        request.preservation.path.as_path(),
        request.outer_ledger_copy.as_path(),
        request.hub_ledger_copy.as_path(),
    ]);
    for (index, path) in paths.iter().enumerate() {
        outside(root, path)?;
        if paths[index + 1..].contains(path) {
            return Err(invalid("replay input and output paths must be distinct"));
        }
    }
    let preservation: FreshPreservationSubjectV1 = sealed::read(&request.preservation.path)?;
    preservation.validate()?;
    let preserved_bytes = canonical_lf(&preservation)?;
    if preservation.family_root_path_hex != hex(root.as_os_str().as_encoded_bytes())
        || preservation.preservation_id != request.preservation.preservation_id
        || sha256(&preserved_bytes) != request.preservation.sealed_sha256
        || preserved_bytes.len() as u64 != request.preservation.byte_length
    {
        return Err(invalid(
            "sealed preservation reference differs from its exact family subject",
        ));
    }
    let outer_bytes = ledger_copy(&request.outer_ledger_copy, &preservation.outer_inventory)?;
    let hub_bytes = ledger_copy(&request.hub_ledger_copy, &preservation.hub_inventory)?;
    let outer = replay(&outer_bytes, &preservation.outer_inventory.inventory_id)?;
    let hub = replay(&hub_bytes, &preservation.hub_inventory.inventory_id)?;
    require_dispositions(&request.dispositions, &outer, &hub)?;
    let facts = ReplayFacts {
        kind: "FRESH_REPLAY_SUBJECT_V1",
        schema_version: 1,
        purpose: "REPLAY_FACTS_ONLY",
        request,
        outer,
        hub,
    };
    let replay_id = subject_id("fgr_", REPLAY_DOMAIN, &facts)?;
    let subject = FreshReplaySubjectV1 { replay_id, facts };
    let bytes = canonical_lf(&subject)?;
    Ok(Reconstruction {
        subject,
        bytes,
        preservation_bytes: preserved_bytes,
        outer_bytes,
        hub_bytes,
    })
}

fn outside(root: &Path, path: &Path) -> Result<(), CoordError> {
    require_normalized_absolute(path, "sealed replay record")?;
    if path.starts_with(root) {
        return Err(invalid(
            "replay records must remain outside the family root",
        ));
    }
    Ok(())
}

fn ledger_copy(path: &Path, inventory: &IncidentInventoryV1) -> Result<Vec<u8>, CoordError> {
    let node = inventory
        .subject
        .nodes
        .iter()
        .find(|node| node.relative_path_hex == hex(b"events.jsonl"))
        .ok_or_else(|| invalid("preservation inventory omits its complete events.jsonl source"))?;
    if node.node_type != IncidentInventoryNodeTypeV1::RegularFile
        || node.byte_length == 0
        || node.byte_length > MAX_BYTES
    {
        return Err(invalid(
            "replay requires one complete bounded regular legacy ledger",
        ));
    }
    let bytes = sealed::read_raw(path, MAX_BYTES)?;
    if bytes.len() as u64 != node.byte_length
        || node.content_sha256.as_deref() != Some(sha256(&bytes).as_str())
    {
        return Err(invalid(
            "ledger copy differs from its exact inventory role, digest or length",
        ));
    }
    Ok(bytes)
}

fn replay(bytes: &[u8], inventory_id: &str) -> Result<LocationReplay, CoordError> {
    let records = legacy::read_record_bytes(bytes)?;
    let as_of = records
        .iter()
        .map(record_time)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .ok_or_else(|| invalid("replay ledger has no records"))?;
    let claims = state::summaries(&records, as_of)?;
    let projection = projection::inventory(&records, &claims)?;
    Ok(LocationReplay {
        inventory_id: inventory_id.into(),
        ledger_sha256: sha256(bytes),
        byte_length: bytes.len() as u64,
        record_count: records.len() as u64,
        as_of_event_unix_ms: as_of,
        projection,
        claims,
    })
}

fn require_dispositions(
    rows: &[ClaimDisposition],
    outer: &LocationReplay,
    hub: &LocationReplay,
) -> Result<(), CoordError> {
    if rows.len() != outer.claims.len() + hub.claims.len() {
        return Err(invalid(
            "every replayed claim requires exactly one explicit disposition",
        ));
    }
    let expected = outer
        .claims
        .values()
        .map(|claim| (Location::Outer, claim))
        .chain(hub.claims.values().map(|claim| (Location::Hub, claim)));
    for (row, (location, claim)) in rows.iter().zip(expected) {
        if row.location != location
            || row.claim_id != claim.claim_id
            || row.claim_subject_id != subject_id("frc_", CLAIM_DOMAIN, claim)?
        {
            return Err(invalid(
                "dispositions omit, reorder or change an exact replayed claim",
            ));
        }
        let valid = match &row.disposition {
            Disposition::RecordedReceipt { commit_oid } => {
                claim.state == ClaimState::HandedOff
                    && claim.commit_oid.as_ref() == Some(commit_oid)
                    && claim.commit_orchestrator.is_some()
                    && claim.commit_recorded_at_unix_ms.is_some()
            }
            Disposition::RetainForRecovery => {
                claim.commit_oid.is_none()
                    && claim.commit_orchestrator.is_none()
                    && claim.commit_recorded_at_unix_ms.is_none()
            }
        };
        if !valid {
            return Err(invalid(
                "disposition contradicts the complete reducer receipt state",
            ));
        }
    }
    Ok(())
}

fn revalidate(inputs: &[(&Path, &[u8])]) -> Result<(), CoordError> {
    for (path, bytes) in inputs {
        if sealed::read_raw(path, MAX_BYTES + 1)? != *bytes {
            return Err(changed("sealed replay input changed during read-back"));
        }
    }
    Ok(())
}

fn canonical_lf(value: &impl Serialize) -> Result<Vec<u8>, CoordError> {
    let mut bytes =
        bullet_wire::canonical_json(value).map_err(|error| invalid(error.to_string()))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("replay document exceeds the canonical byte bound"));
    }
    bytes.push(b'\n');
    Ok(bytes)
}

fn subject_id(prefix: &str, domain: &str, value: &impl Serialize) -> Result<String, CoordError> {
    let digest =
        bullet_wire::hash_canonical(domain, value).map_err(|error| invalid(error.to_string()))?;
    Ok(format!("{prefix}{}", digest.to_hex()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FRESH_GENESIS_SUBJECT", reason)
}
fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("FRESH_GENESIS_SUBJECT_CHANGED", reason)
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
