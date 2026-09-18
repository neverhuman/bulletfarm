use std::{fmt::Write as _, os::unix::ffi::OsStrExt, path::Path};

use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use super::{
    CoordError,
    model::{
        FreshGenesisAdmissionReferencesV1, FreshGenesisRecordKindV1, FreshGenesisSealedRecordRefV1,
        FreshPreservationSubjectV1, IncidentInventoryV1, Wave0SubjectV1,
    },
    recovery_manifest::require_normalized_absolute,
    sealed,
};

#[cfg(target_os = "linux")]
pub(in crate::coord) mod incident;

const MAX_RECORD_BYTES: u64 = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FreshGenesisPublicationOutcome {
    Created,
    AdoptedExactExisting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FreshGenesisPublication {
    pub(crate) references: FreshGenesisAdmissionReferencesV1,
    pub(crate) references_subject_blake3: String,
    pub(crate) inventory_outcome: FreshGenesisPublicationOutcome,
    pub(crate) wave0_outcome: FreshGenesisPublicationOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FreshPreservationPublication {
    pub(crate) subject: FreshPreservationSubjectV1,
    pub(crate) sealed_sha256: String,
    pub(crate) byte_length: u64,
    pub(crate) outcome: FreshGenesisPublicationOutcome,
}

/// Bind supplied inventory observations without observing or moving either incident.
pub(crate) fn publish_preservation_record(
    family_root: &Path,
    output: &Path,
    outer_inventory: &IncidentInventoryV1,
    hub_inventory: &IncidentInventoryV1,
) -> Result<FreshPreservationPublication, CoordError> {
    require_normalized_absolute(family_root, "preservation family root")?;
    require_normalized_absolute(output, "preservation output")?;
    if output.starts_with(family_root) {
        return Err(invalid(
            "preservation output must remain outside the family root",
        ));
    }
    let subject = FreshPreservationSubjectV1::from_inventories(
        path_hex(family_root),
        outer_inventory.clone(),
        hub_inventory.clone(),
    )?;
    let bytes = canonical_lf(&subject)?;
    let outcome = publish_one(
        output,
        &subject,
        &bytes,
        FreshPreservationSubjectV1::validate,
        "two-location preservation subject",
    )?;
    Ok(FreshPreservationPublication {
        subject,
        sealed_sha256: sha256(&bytes),
        byte_length: bytes.len() as u64,
        outcome,
    })
}

/// Refuse legacy single-location inputs before reading or publishing records.
///
/// V1 cannot bind both incident locations, reviewed dispositions, and the
/// operator checkpoint. Its observation/publication components remain available
/// for engineering, but they cannot authorize sidecar or Genesis creation.
pub(crate) fn consume_wave0_and_inventory(
    inventory_path: &Path,
    wave0_path: &Path,
    _family_root: &Path,
) -> Result<(IncidentInventoryV1, Wave0SubjectV1), CoordError> {
    if !inventory_path.is_file() || !wave0_path.is_file() {
        return Err(invalid(
            "incident inventory or W0 subject is missing or not a regular file",
        ));
    }
    Err(CoordError::new(
        "FRESH_GENESIS_ADMISSION_UNAVAILABLE",
        "single-location V1 records cannot authorize incident-derived Genesis; complete two-location admission, durable references, independent review, and operator checkpoint are required",
    ))
}

pub(crate) fn publish_records(
    checkout_root: &Path,
    inventory_output: &Path,
    wave0_output: &Path,
    inventory: &IncidentInventoryV1,
    wave0: &Wave0SubjectV1,
) -> Result<FreshGenesisPublication, CoordError> {
    inventory.validate()?;
    wave0.validate()?;
    validate_paths(checkout_root, inventory_output, wave0_output)?;

    let inventory_bytes = canonical_lf(inventory)?;
    let wave0_bytes = canonical_lf(wave0)?;
    let references = references(
        inventory_output,
        wave0_output,
        inventory,
        wave0,
        &inventory_bytes,
        &wave0_bytes,
    )?;
    let references_subject_blake3 = references.subject_blake3()?;

    let inventory_outcome = publish_one(
        inventory_output,
        inventory,
        &inventory_bytes,
        IncidentInventoryV1::validate,
        "incident inventory",
    )?;
    inventory.validate()?;
    let wave0_outcome = publish_one(
        wave0_output,
        wave0,
        &wave0_bytes,
        Wave0SubjectV1::validate,
        "W0 subject",
    )?;
    wave0.validate()?;

    verify_exact(
        inventory_output,
        inventory,
        &inventory_bytes,
        IncidentInventoryV1::validate,
        "incident inventory",
    )?;
    verify_exact(
        wave0_output,
        wave0,
        &wave0_bytes,
        Wave0SubjectV1::validate,
        "W0 subject",
    )?;
    references.validate()?;
    if references.subject_blake3()? != references_subject_blake3 {
        return Err(changed(
            "fresh-Genesis sealed references changed during publication",
        ));
    }

    Ok(FreshGenesisPublication {
        references,
        references_subject_blake3,
        inventory_outcome,
        wave0_outcome,
    })
}

fn validate_paths(
    checkout_root: &Path,
    inventory_output: &Path,
    wave0_output: &Path,
) -> Result<(), CoordError> {
    require_normalized_absolute(checkout_root, "fresh-Genesis checkout root")?;
    require_normalized_absolute(inventory_output, "incident inventory output")?;
    require_normalized_absolute(wave0_output, "W0 output")?;
    if inventory_output == wave0_output {
        return Err(invalid(
            "incident inventory and W0 outputs must be distinct absolute paths",
        ));
    }
    if inventory_output.starts_with(checkout_root) || wave0_output.starts_with(checkout_root) {
        return Err(invalid(
            "fresh-Genesis sealed outputs must remain outside the supplied checkout root",
        ));
    }
    Ok(())
}

fn references(
    inventory_output: &Path,
    wave0_output: &Path,
    inventory: &IncidentInventoryV1,
    wave0: &Wave0SubjectV1,
    inventory_bytes: &[u8],
    wave0_bytes: &[u8],
) -> Result<FreshGenesisAdmissionReferencesV1, CoordError> {
    let value = FreshGenesisAdmissionReferencesV1 {
        incident_inventory: FreshGenesisSealedRecordRefV1 {
            record_kind: FreshGenesisRecordKindV1::IncidentInventoryV1,
            absolute_path_hex: path_hex(inventory_output),
            record_id: inventory.inventory_id.clone(),
            sealed_sha256: sha256(inventory_bytes),
            byte_length: inventory_bytes.len() as u64,
        },
        wave0_subject: FreshGenesisSealedRecordRefV1 {
            record_kind: FreshGenesisRecordKindV1::Wave0SubjectV1,
            absolute_path_hex: path_hex(wave0_output),
            record_id: wave0.subject_id.clone(),
            sealed_sha256: sha256(wave0_bytes),
            byte_length: wave0_bytes.len() as u64,
        },
    };
    value.validate()?;
    Ok(value)
}

fn publish_one<T>(
    path: &Path,
    expected: &T,
    expected_bytes: &[u8],
    validate: fn(&T) -> Result<(), CoordError>,
    label: &str,
) -> Result<FreshGenesisPublicationOutcome, CoordError>
where
    T: DeserializeOwned + Eq + Serialize,
{
    let outcome = match sealed::write(path, expected) {
        Ok(()) => FreshGenesisPublicationOutcome::Created,
        Err(write_error) => {
            if verify_exact(path, expected, expected_bytes, validate, label).is_err() {
                return Err(changed(format!(
                    "{label} was not published or adopted as exact existing bytes: {write_error}"
                )));
            }
            FreshGenesisPublicationOutcome::AdoptedExactExisting
        }
    };
    verify_exact(path, expected, expected_bytes, validate, label)?;
    Ok(outcome)
}

fn verify_exact<T>(
    path: &Path,
    expected: &T,
    expected_bytes: &[u8],
    validate: fn(&T) -> Result<(), CoordError>,
    label: &str,
) -> Result<(), CoordError>
where
    T: DeserializeOwned + Eq + Serialize,
{
    let observed = sealed::read_synced_raw(path, MAX_RECORD_BYTES)
        .map_err(|error| changed(format!("cannot read back sealed {label}: {error}")))?;
    if observed != expected_bytes {
        return Err(changed(format!(
            "sealed {label} differs from the exact canonical publication bytes"
        )));
    }
    let body = observed
        .strip_suffix(b"\n")
        .ok_or_else(|| changed(format!("sealed {label} lacks exact LF framing")))?;
    let decoded = bullet_wire::decode_canonical::<T>(body)
        .map_err(|error| changed(format!("sealed {label} is not canonical: {error}")))?;
    validate(&decoded)
        .map_err(|error| changed(format!("sealed {label} identity is invalid: {error}")))?;
    if &decoded != expected || canonical_lf(&decoded)? != observed {
        return Err(changed(format!(
            "sealed {label} differs after identity revalidation"
        )));
    }
    Ok(())
}

fn canonical_lf(value: &impl Serialize) -> Result<Vec<u8>, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value)
        .map_err(|error| invalid(format!("cannot canonicalize fresh-Genesis record: {error}")))?;
    if bytes.is_empty() || bytes.len() as u64 >= MAX_RECORD_BYTES {
        return Err(invalid(
            "fresh-Genesis record exceeds its closed canonical byte bound",
        ));
    }
    bytes.push(b'\n');
    Ok(bytes)
}

fn path_hex(path: &Path) -> String {
    let raw = path.as_os_str().as_bytes();
    let mut output = String::with_capacity(raw.len() * 2);
    for byte in raw {
        write!(&mut output, "{byte:02x}").expect("formatting bytes into String cannot fail");
    }
    output
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FRESH_GENESIS_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("FRESH_GENESIS_SUBJECT_CHANGED", reason)
}

#[cfg(test)]
mod tests;
