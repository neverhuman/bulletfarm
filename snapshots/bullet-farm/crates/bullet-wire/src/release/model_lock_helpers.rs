use serde_json::Value;

use super::{
    FORMAL_MODEL_IDS, LOCK_SCHEMA_VERSION, MAX_SAFE_INTEGER, MODEL_LOCK_MALFORMED,
    MODEL_LOCK_MISSING, MODEL_SET_INCOMPLETE, ModelLockV1,
};
use crate::{WireError, decode_unique_value};

pub(super) fn model_set(locks: &[ModelLockV1]) -> Result<(), WireError> {
    let ids = locks
        .iter()
        .map(|lock| lock.model_id.as_str())
        .collect::<Vec<_>>();
    if let Some(unknown) = ids.iter().find(|id| !FORMAL_MODEL_IDS.contains(id)) {
        return Err(refuse(
            MODEL_SET_INCOMPLETE,
            format!("{unknown} is not one of the pinned formal models"),
        ));
    }
    if let Some(missing) = FORMAL_MODEL_IDS.iter().find(|id| !ids.contains(id)) {
        return Err(refuse(
            MODEL_LOCK_MISSING,
            format!("lock for pinned model {missing} is missing"),
        ));
    }
    if ids.len() != FORMAL_MODEL_IDS.len() {
        return Err(refuse(
            MODEL_SET_INCOMPLETE,
            "lock set repeats a pinned model",
        ));
    }
    if ids != FORMAL_MODEL_IDS {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            "locks must be byte-sorted by model id",
        ));
    }
    Ok(())
}

pub(super) fn strict_document<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, WireError> {
    let value = decode_unique_value(bytes)
        .map_err(|error| refuse(MODEL_LOCK_MALFORMED, format!("{label}: {error}")))?;
    serde_json::from_value(value).map_err(|error| {
        refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} does not match its exact shape: {error}"),
        )
    })
}

pub(super) fn trace_model(bytes: &[u8]) -> Result<String, WireError> {
    let value = decode_unique_value(bytes)
        .map_err(|error| refuse(MODEL_LOCK_MALFORMED, format!("formal trace: {error}")))?;
    let malformed = || refuse(MODEL_LOCK_MALFORMED, "formal trace is not a strict v1alpha1 fixture");
    let Value::Object(members) = &value else {
        return Err(malformed());
    };
    let keys = members.keys().map(String::as_str).collect::<Vec<_>>();
    if keys != ["model", "schema_version", "steps"]
        || members.get("schema_version").and_then(Value::as_str) != Some(LOCK_SCHEMA_VERSION)
    {
        return Err(malformed());
    }
    let steps = members.get("steps").and_then(Value::as_array).ok_or_else(malformed)?;
    if steps.is_empty()
        || !steps.iter().all(|step| {
            step.get("action").is_some_and(Value::is_string)
                && step.get("expected").is_some_and(Value::is_string)
        })
    {
        return Err(malformed());
    }
    let model = members.get("model").and_then(Value::as_str).ok_or_else(malformed)?;
    model_id(model)?;
    Ok(model.to_owned())
}

pub(super) fn model_id(value: &str) -> Result<(), WireError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 64
        || !bytes[0].is_ascii_alphabetic()
        || !bytes.iter().all(u8::is_ascii_alphanumeric)
    {
        return Err(refuse(MODEL_LOCK_MALFORMED, "model id is malformed"));
    }
    Ok(())
}

pub(super) fn positive(value: u64, label: &str) -> Result<(), WireError> {
    if value == 0 || value > MAX_SAFE_INTEGER {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} is zero or outside the exact integer range"),
        ));
    }
    Ok(())
}

pub(super) fn tagged(value: &str, tag: &str, label: &str) -> Result<(), WireError> {
    let hex = value
        .strip_prefix(tag)
        .ok_or_else(|| refuse(MODEL_LOCK_MALFORMED, format!("{label} digest lacks its {tag} tag")))?;
    lower_hex(hex, 64, label)
}

pub(super) fn lower_hex(value: &str, length: usize, label: &str) -> Result<(), WireError> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(refuse(
            MODEL_LOCK_MALFORMED,
            format!("{label} digest is not {length} lowercase hexadecimal characters"),
        ));
    }
    Ok(())
}

pub(super) fn refuse(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}
