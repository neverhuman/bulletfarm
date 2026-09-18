use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};

use super::{CoordError, MAX_DOCUMENT_BYTES, ParentAdmission, invalid, read_bytes};

pub(crate) fn read<T>(path: &Path) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    read_canonical(path, MAX_DOCUMENT_BYTES, ParentAdmission::Sealed)
}

pub(crate) fn read_root_runtime<T>(path: &Path, maximum: u64) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    if maximum == 0 {
        return Err(invalid("root runtime document bound must be positive"));
    }
    read_canonical(path, maximum, ParentAdmission::RootRuntime)
}

fn read_canonical<T>(path: &Path, maximum: u64, admission: ParentAdmission) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    let bytes = read_bytes(path, maximum, admission)?;
    if bytes.last() != Some(&b'\n') || bytes[..bytes.len() - 1].contains(&b'\n') {
        return Err(invalid(
            "sealed recovery document must end in exactly one LF",
        ));
    }
    let body = &bytes[..bytes.len() - 1];
    let value = bullet_wire::decode_canonical::<T>(body).map_err(|error| {
        invalid(format!(
            "sealed recovery document is not canonical: {error}"
        ))
    })?;
    if bullet_wire::canonical_json(&value).map_err(|error| {
        invalid(format!(
            "cannot re-encode sealed recovery document: {error}"
        ))
    })? != body
    {
        return Err(invalid(
            "sealed recovery document changed after strict decode",
        ));
    }
    Ok(value)
}

/// Read the canonical mode-0600, unframed output of an observation producer.
/// This role never admits a sealed authorization or changes the input bytes.
pub(crate) fn read_observation<T>(path: &Path) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    let bytes = read_bytes(path, MAX_DOCUMENT_BYTES - 1, ParentAdmission::Observation)?;
    let value = bullet_wire::decode_canonical::<T>(&bytes)
        .map_err(|error| invalid(format!("observation is not canonical: {error}")))?;
    if bullet_wire::canonical_json(&value)
        .map_err(|error| invalid(format!("cannot re-encode observation: {error}")))?
        != bytes
    {
        return Err(invalid("observation changed after strict decode"));
    }
    Ok(value)
}
