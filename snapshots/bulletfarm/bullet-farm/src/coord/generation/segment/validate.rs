use serde::{Deserialize, Serialize};

use super::AppendRequest;
use crate::coord::{
    CoordError,
    model::{GENERATION_SCHEMA_VERSION, Record},
};

const ENVELOPE_DOMAIN: &str = "bullet.coord.segment-envelope.v2";
const RECORD_DOMAIN: &str = "bullet.coord.segment-record.v2";
const FRAME_DOMAIN: &str = "bullet.coord.segment-frame.v2";
const REQUEST_DOMAIN: &str = "bullet.coord.segment-append-request.v2";

#[derive(Serialize)]
struct AppendSubject<'a> {
    kind: &'static str,
    schema_version: u32,
    generation_id: &'a str,
    sequence: u64,
    previous_digest: &'a str,
    request_id: &'a str,
    record_digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SegmentEnvelope {
    pub kind: String,
    pub schema_version: u32,
    pub generation_id: String,
    pub sequence: u64,
    pub previous_digest: String,
    pub request_id: String,
    pub request_digest: String,
    pub record: Record,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PendingIntent {
    pub kind: String,
    pub schema_version: u32,
    pub generation_id: String,
    pub sequence: u64,
    pub previous_digest: String,
    pub request_id: String,
    pub request_digest: String,
    pub segment_offset: u64,
    pub frame_length: u64,
    pub frame_digest: String,
    pub frame_utf8: String,
}

pub(super) fn canonical<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, CoordError> {
    bullet_wire::canonical_json(value).map_err(|error| {
        CoordError::new(
            "INVALID_COORD_RECORD",
            format!("cannot encode canonical {label}: {error}"),
        )
    })
}

pub(super) fn digest_record(record: &Record) -> Result<String, CoordError> {
    digest(
        RECORD_DOMAIN,
        &canonical(record, "typed coordination record")?,
    )
}

pub(super) fn append_request_digest(request: &AppendRequest<'_>) -> Result<String, CoordError> {
    let subject = AppendSubject {
        kind: "coord_segment_append_request_v2",
        schema_version: 2,
        generation_id: request.generation_id,
        sequence: request.sequence,
        previous_digest: request.previous_digest,
        request_id: request.request_id,
        record_digest: digest_record(request.record)?,
    };
    digest(
        REQUEST_DOMAIN,
        &canonical(&subject, "append request subject")?,
    )
}

pub(super) fn validate_record_position(request: &AppendRequest<'_>) -> Result<(), CoordError> {
    match (request.sequence, request.record) {
        (
            1,
            Record::GenesisV2 {
                schema_version,
                generation_id,
                ..
            }
            | Record::RecoveryBaselineV2 {
                schema_version,
                generation_id,
                ..
            },
        ) if *schema_version == GENERATION_SCHEMA_VERSION
            && generation_id == request.generation_id =>
        {
            Ok(())
        }
        (1, _) => Err(CoordError::new(
            "INVALID_COORD_BASELINE",
            "sequence 1 must be a matching schema-2 GenesisV2 or RecoveryBaselineV2",
        )),
        (_, Record::GenesisV2 { .. } | Record::RecoveryBaselineV2 { .. }) => Err(CoordError::new(
            "INVALID_COORD_BASELINE",
            "generation transition records are admitted only at sequence 1",
        )),
        (_, record) if record.schema_version() == GENERATION_SCHEMA_VERSION => Ok(()),
        _ => Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            "post-baseline coordination records must use schema version 2",
        )),
    }
}

pub(super) fn digest(domain: &str, bytes: &[u8]) -> Result<String, CoordError> {
    bullet_wire::hash_framed_bytes(domain, bytes)
        .map(|value| value.to_hex())
        .map_err(|error| {
            CoordError::new(
                "INVALID_COORD_RECORD",
                format!("cannot digest bytes: {error}"),
            )
        })
}

pub(super) fn validate_digest(label: &str, value: &str) -> Result<(), CoordError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_COORD_DIGEST",
            format!("{label} must be 64 lowercase hex bytes"),
        ))
    }
}

pub(super) fn validate_generation_id(value: &str) -> Result<(), CoordError> {
    value
        .strip_prefix("gen_")
        .ok_or_else(|| {
            CoordError::new(
                "INVALID_COORD_GENERATION",
                "generation ID must use the gen_<64lower> form",
            )
        })
        .and_then(|digest| validate_digest("generation_id", digest))
}

pub(super) fn validate_token(label: &str, value: &str) -> Result<(), CoordError> {
    if (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_COORD_REQUEST",
            format!("{label} is not a bounded ASCII token"),
        ))
    }
}

pub(super) fn validate_envelope_identity(
    envelope: &SegmentEnvelope,
    generation_id: &str,
) -> Result<(), CoordError> {
    if envelope.generation_id != generation_id {
        return Err(CoordError::new(
            "STALE_COORD_GENERATION",
            "segment frame belongs to another generation",
        ));
    }
    Ok(())
}

pub(super) fn capacity_error() -> CoordError {
    CoordError::new(
        "COORD_SEGMENT_CAPACITY_EXCEEDED",
        "coordination segment exceeds its 64 MiB bound",
    )
}

pub(super) fn corrupt(reason: impl Into<String>) -> CoordError {
    CoordError::new("CORRUPT_COORD_SEGMENT", reason)
}

pub(super) fn corrupt_pending(reason: impl Into<String>) -> CoordError {
    CoordError::new("CORRUPT_COORD_PENDING", reason)
}

pub(super) fn envelope_digest(canonical: &[u8]) -> Result<String, CoordError> {
    digest(ENVELOPE_DOMAIN, canonical)
}

pub(super) fn frame_digest(frame: &[u8]) -> Result<String, CoordError> {
    digest(FRAME_DOMAIN, frame)
}
