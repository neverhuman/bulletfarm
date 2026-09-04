use std::{fmt::Write as _, os::unix::ffi::OsStrExt, path::Path, process::Command};

use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use super::{
    CoordError,
    model::{
        FreshGenesisAdmissionReferencesV1, FreshGenesisRecordKindV1, FreshGenesisSealedRecordRefV1,
        IncidentInventoryV1, Wave0SubjectV1,
    },
    recovery_manifest::require_normalized_absolute,
    sealed,
};

#[cfg(target_os = "linux")]
pub(in crate::coord) mod incident;

const MAX_RECORD_BYTES: u64 = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Load sealed W0 + incident inventory and refuse dirty porcelain or HEAD drift.
///
/// # Errors
///
/// Missing files, invalid records, dirty trees, or commit drift.
pub(crate) fn consume_wave0_and_inventory(
    inventory_path: &Path,
    wave0_path: &Path,
    family_root: &Path,
) -> Result<(IncidentInventoryV1, Wave0SubjectV1), CoordError> {
    if !inventory_path.is_file() {
        return Err(invalid("incident inventory is missing"));
    }
    if !wave0_path.is_file() {
        return Err(invalid("W0 subject is missing"));
    }
    let inventory: IncidentInventoryV1 = load_canonical(inventory_path, "incident inventory")?;
    let wave0: Wave0SubjectV1 = load_canonical(wave0_path, "W0 subject")?;
    inventory.validate()?;
    wave0.validate()?;
    observe_clean_members(family_root, &wave0)?;
    Ok((inventory, wave0))
}

fn load_canonical<T: DeserializeOwned + Serialize>(
    path: &Path,
    label: &str,
) -> Result<T, CoordError> {
    let bytes =
        std::fs::read(path).map_err(|error| invalid(format!("cannot read {label}: {error}")))?;
    let body = bytes.strip_suffix(b"\n").unwrap_or(bytes.as_slice());
    bullet_wire::decode_canonical(body)
        .map_err(|error| invalid(format!("{label} is not canonical: {error}")))
}

fn observe_clean_members(family_root: &Path, wave0: &Wave0SubjectV1) -> Result<(), CoordError> {
    for member in &wave0.facts.members {
        let dir = match member.repository_identity.as_str() {
            "root/bullet-farm" => family_root.join("bullet-farm"),
            "root/bullet-kernel" => family_root.join("bullet-kernel"),
            "root/bullet-git" => family_root.join("bullet-git"),
            "root/bullet-portal" => family_root.join("bullet-portal"),
            other => {
                return Err(invalid(format!(
                    "W0 member identity {other} is not a family repository"
                )));
            }
        };
        if porcelain_dirty(&dir) {
            return Err(invalid(format!(
                "W0 refuses dirty porcelain in {}",
                dir.display()
            )));
        }
        let head = git_stdout(&dir, &["rev-parse", "HEAD"])?;
        let expected = member
            .commit_oid
            .strip_prefix("sha1:")
            .ok_or_else(|| invalid("W0 commit OID must be sha1-tagged"))?;
        if head != expected {
            return Err(invalid(format!(
                "W0 commit drift in {}: observed {head} expected {expected}",
                dir.display()
            )));
        }
    }
    Ok(())
}

fn porcelain_dirty(path: &Path) -> bool {
    Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
        .output()
        .map(|output| !output.status.success() || !output.stdout.is_empty())
        .unwrap_or(true)
}

fn git_stdout(path: &Path, args: &[&str]) -> Result<String, CoordError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|error| invalid(format!("git failed: {error}")))?;
    if !output.status.success() {
        return Err(invalid("git observation failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
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
    let observed = sealed::read_raw(path, MAX_RECORD_BYTES)
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
