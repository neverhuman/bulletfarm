use crate::coord::{CoordError, generation::manifest::CurrentPointer};

pub(super) fn chain_genesis(pointer: &CurrentPointer) -> Result<String, CoordError> {
    pointer
        .manifest_blake3()
        .strip_prefix("blake3:")
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid("CURRENT manifest digest is not tagged BLAKE3"))
}

pub(super) fn genesis_request_id(pointer: &CurrentPointer) -> Result<String, CoordError> {
    let digest = bullet_wire::hash_framed_bytes(
        "bullet.coord.genesis-request-id.v2",
        pointer.manifest_blake3().as_bytes(),
    )
    .map_err(|error| invalid(format!("cannot derive GENESIS request ID: {error}")))?;
    Ok(format!("req_genesis_{}", digest.to_hex()))
}

pub(super) fn uninitialized() -> CoordError {
    CoordError::new(
        "COORD_NOT_INITIALIZED",
        "coordination generation has not been initialized",
    )
}

pub(super) fn recovery_required() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_REQUIRED",
        "legacy events.jsonl exists without CURRENT; explicit recovery is required",
    )
}

pub(super) fn recovery_in_progress() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_IN_PROGRESS",
        "legacy source is retired but CURRENT is not yet durably published",
    )
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_LEDGER", reason)
}

pub(super) fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}

pub(super) fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("canonical coordination record failed: {error}"))
}

pub(super) fn fence_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_FENCE_UNKNOWN", reason)
}

pub(super) fn validate_request_id(value: &str) -> Result<(), CoordError> {
    if value.len() == 68
        && value.starts_with("req_")
        && value[4..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_COORD_REQUEST_ID",
            "request ID must be req_ plus 64 lowercase hexadecimal digits",
        ))
    }
}
