use super::{MAX_SAFE_INTEGER, authority_error};
use crate::WireError;

pub(super) fn require_positive(name: &str, value: u64) -> Result<(), WireError> {
    if value == 0 {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} must be positive"),
        ));
    }
    require_safe(name, value)
}

pub(super) fn require_time(name: &str, value: u64) -> Result<(), WireError> {
    require_safe(name, value)
}

pub(super) fn require_safe(name: &str, value: u64) -> Result<(), WireError> {
    if value > MAX_SAFE_INTEGER {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} exceeds the interoperable integer range"),
        ));
    }
    Ok(())
}

pub(super) fn validate_label(name: &str, value: &str) -> Result<(), WireError> {
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(authority_error(
            "INVALID_AUTHORITY_REQUEST",
            format!("{name} must be bounded non-control text"),
        ));
    }
    Ok(())
}

pub(super) fn claim_binding_mismatch(reason: &'static str) -> WireError {
    authority_error("AUTHORITY_REQUEST_BINDING_MISMATCH", reason)
}
