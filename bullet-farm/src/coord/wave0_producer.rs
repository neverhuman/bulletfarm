//! Producers for the fresh-Genesis ceremony inputs (plan G1.1).
//!
//! `coord init --wave0-subject --incident-inventory` consumes a sealed
//! `Wave0SubjectV1` and `IncidentInventoryV1`; until now nothing in
//! production could construct either. These producers wire the tested
//! observers into operator verbs, emitting the same canonical encoding the
//! consumer round-trips, create-once at 0600.
//!
//! Nothing here is authority: the facts are mechanical observations of the
//! four members and the frozen claim ledger; the review binding requires a
//! second principal by construction (no key, by the type's own design); the
//! inventory describes a directory the operator has not yet moved.

use super::CoordError;
use super::fresh_genesis::incident::{
    observe_incident_inventory, verify_retired_incident_inventory,
};
use super::git::wave0::observe_wave0_mechanical;
use super::model::{IncidentInventoryV1, Wave0ClaimHighWaterV1, Wave0FactsV1, Wave0SubjectV1};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Domain prefix for the claim-projection digest: BLAKE3 over the RFC 8785
/// encoding of the sorted active-claim-id list, domain-separated so the
/// digest can never be confused with a file digest.
const CLAIM_PROJECTION_DOMAIN: &[u8] = b"bullet-family.coord.wave0-claim-projection.v1\0";

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("WAVE0_PRODUCER_INVALID", reason)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Write canonical bytes create-once at 0600 and require byte read-back.
fn write_canonical_once(path: &Path, bytes: &[u8]) -> Result<(), CoordError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| invalid(format!("cannot create {}: {error}", path.display())))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| invalid(format!("cannot write {}: {error}", path.display())))?;
    let reread = fs::read(path).map_err(CoordError::io)?;
    if reread != bytes {
        return Err(invalid("written bytes differ on read-back"));
    }
    Ok(())
}

/// One legacy claim-ledger record, as far as the high-water projection needs
/// it. Unknown fields are deliberately tolerated: the projection consumes the
/// frozen generation's schema without claiming to validate it.
#[derive(serde::Deserialize)]
struct LegacyRecord {
    kind: String,
    #[serde(default)]
    claim_id: Option<String>,
    #[serde(default)]
    expires_unix_ms: Option<u64>,
}

/// Replay the frozen claim ledger into its high-water mark. Refuses when any
/// claim is still active at `now`: the W0 subject's own validator requires
/// `active_claim_count == 0`, and observing a live claim means the ceremony
/// is premature rather than the count being negotiable.
fn claim_high_water(ledger: &Path, now_unix_ms: u64) -> Result<Wave0ClaimHighWaterV1, CoordError> {
    let bytes = fs::read(ledger)
        .map_err(|error| invalid(format!("cannot read {}: {error}", ledger.display())))?;
    let mut entry_count: u64 = 0;
    let mut active: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for line in bytes.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        entry_count += 1;
        // decode_unique_value refuses duplicate keys; the legacy records are
        // then mapped through the tolerant LegacyRecord view.
        let value = bullet_wire::decode_unique_value(line)
            .map_err(|error| invalid(format!("frozen ledger line is not JSON: {error}")))?;
        let record: LegacyRecord = serde_json::from_value(value)
            .map_err(|error| invalid(format!("frozen ledger record is malformed: {error}")))?;
        match record.kind.as_str() {
            "claim" => {
                let id = record
                    .claim_id
                    .ok_or_else(|| invalid("claim record without claim_id"))?;
                let expires = record
                    .expires_unix_ms
                    .ok_or_else(|| invalid("claim record without expires_unix_ms"))?;
                active.insert(id, expires);
            }
            "handoff" => {
                if let Some(id) = record.claim_id {
                    active.remove(&id);
                }
            }
            _ => {}
        }
    }
    active.retain(|_, expires| *expires > now_unix_ms);
    if !active.is_empty() {
        return Err(invalid(format!(
            "{} claims are still active; the W0 subject requires zero",
            active.len()
        )));
    }
    let projection_ids: Vec<&String> = active.keys().collect();
    let projection_bytes = serde_json::to_vec(&projection_ids).map_err(CoordError::json)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(CLAIM_PROJECTION_DOMAIN);
    hasher.update(&projection_bytes);
    let path_hex = hex_lower(ledger.as_os_str().as_bytes());
    let sha256 = {
        use sha2::{Digest, Sha256};
        let mut sha = Sha256::new();
        sha.update(&bytes);
        hex_lower(&sha.finalize())
    };
    Ok(Wave0ClaimHighWaterV1 {
        claim_ledger_path_hex: path_hex,
        claim_ledger_sha256: format!("sha256:{sha256}"),
        claim_projection_blake3: format!("blake3:{}", hasher.finalize().to_hex()),
        byte_length: bytes.len() as u64,
        entry_count,
        active_claim_count: 0,
    })
}

/// Observe the four members mechanically plus the frozen claim ledger's
/// high-water mark, and emit unreviewed `Wave0FactsV1` for a second
/// principal to review. Refuses any dirty member and any active claim.
pub(crate) fn produce_wave0_facts(
    family_root: &Path,
    ledger: &Path,
    producer_principal: &str,
    now_unix_ms: u64,
    out: &Path,
) -> Result<(), CoordError> {
    if producer_principal.is_empty() {
        return Err(invalid("producer principal must not be empty"));
    }
    let observation = observe_wave0_mechanical(family_root)?;
    let facts = Wave0FactsV1 {
        producer_principal: producer_principal.to_owned(),
        claim_high_water: claim_high_water(ledger, now_unix_ms)?,
        members: observation.members.to_vec(),
    };
    let bytes = bullet_wire::canonical_json(&facts)
        .map_err(|error| invalid(format!("facts are not canonical: {error}")))?;
    write_canonical_once(out, &bytes)
}

/// Complete a reviewed `Wave0SubjectV1` from produced facts plus a second
/// principal's review record. The type itself refuses reviewer == producer
/// and any facts drift — no key, by design.
pub(crate) fn produce_wave0_subject(
    facts_path: &Path,
    reviewer_principal: &str,
    review_record: &Path,
    out: &Path,
) -> Result<(), CoordError> {
    let facts_bytes = fs::read(facts_path).map_err(CoordError::io)?;
    let facts: Wave0FactsV1 = bullet_wire::decode_canonical(&facts_bytes)
        .map_err(|error| invalid(format!("facts are not canonical: {error}")))?;
    let record_bytes = fs::read(review_record).map_err(CoordError::io)?;
    if record_bytes.is_empty() {
        return Err(invalid("review record must not be empty"));
    }
    let record_sha256 = {
        use sha2::{Digest, Sha256};
        let mut sha = Sha256::new();
        sha.update(&record_bytes);
        hex_lower(&sha.finalize())
    };
    let subject = Wave0SubjectV1::from_reviewed(
        facts,
        reviewer_principal.to_owned(),
        hex_lower(review_record.as_os_str().as_bytes()),
        format!("sha256:{record_sha256}"),
        record_bytes.len() as u64,
    )?;
    let bytes = bullet_wire::canonical_json(&subject)
        .map_err(|error| invalid(format!("subject is not canonical: {error}")))?;
    write_canonical_once(out, &bytes)
}

/// Seal the complete pre-move inventory of the frozen coordination directory.
pub(crate) fn produce_incident_inventory(
    coord_dir: &Path,
    destination_name: &OsStr,
    out: &Path,
) -> Result<(), CoordError> {
    let inventory: IncidentInventoryV1 = observe_incident_inventory(coord_dir, destination_name)?;
    inventory.validate()?;
    let bytes = bullet_wire::canonical_json(&inventory)
        .map_err(|error| invalid(format!("inventory is not canonical: {error}")))?;
    write_canonical_once(out, &bytes)
}

/// Prove the operator's `mv` relocated a byte-identical tree: load the sealed
/// inventory and run the existing post-move verifier, which requires the
/// source name to be absent and the destination to match exactly.
pub(crate) fn verify_incident_inventory(inventory_path: &Path) -> Result<(), CoordError> {
    let bytes = fs::read(inventory_path).map_err(CoordError::io)?;
    let inventory: IncidentInventoryV1 = bullet_wire::decode_canonical(&bytes)
        .map_err(|error| invalid(format!("inventory is not canonical: {error}")))?;
    verify_retired_incident_inventory(&inventory)
}
