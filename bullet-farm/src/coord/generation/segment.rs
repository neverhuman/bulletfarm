use std::{
    collections::BTreeMap,
    fs::File,
    io::{Seek, SeekFrom, Write},
};

#[cfg(test)]
use std::path::Path;

use self::io::{
    exact_readback, inspect_bytes, pending_names, publish_intent_at, read_bounded, read_intent_at,
    remove_intent_at, validate_pending_descriptor, validate_segment_descriptor,
};
#[cfg(test)]
use self::io::{open_pending, open_segment};
use self::validate::*;
use super::super::{CoordError, model::Record};

mod io;
mod validate;

pub(super) const MAX_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_FRAME_BYTES: usize = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES + 1;
const MAX_INTENT_BYTES: usize = 4 * MAX_FRAME_BYTES;
const INTENT_NAME: &str = "append.intent.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::coord) struct SegmentPosition {
    pub next_sequence: u64,
    pub previous_digest: String,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::coord) struct AppendReceipt {
    pub sequence: u64,
    pub envelope_digest: String,
    pub record_digest: String,
    pub request_id: String,
    pub request_digest: String,
    pub byte_offset: u64,
    pub frame_length: u64,
}

#[derive(Clone, Debug)]
pub(in crate::coord) struct StoredEnvelope {
    pub generation_id: String,
    pub sequence: u64,
    pub previous_digest: String,
    pub request_id: String,
    pub request_digest: String,
    pub record: Record,
    pub receipt: AppendReceipt,
}

#[derive(Clone, Debug)]
pub(in crate::coord) struct SegmentInspection {
    pub position: SegmentPosition,
    pub entries: Vec<StoredEnvelope>,
    requests: BTreeMap<String, AppendReceipt>,
}

impl SegmentInspection {
    pub(in crate::coord) fn receipt_for_request(&self, request_id: &str) -> Option<&AppendReceipt> {
        self.requests.get(request_id)
    }
}

pub(in crate::coord) struct AppendRequest<'a> {
    pub generation_id: &'a str,
    pub sequence: u64,
    pub previous_digest: &'a str,
    pub request_id: &'a str,
    pub record: &'a Record,
}

#[cfg(test)]
pub(in crate::coord) fn inspect(
    segment_path: &Path,
    pending_dir: &Path,
    expected_generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentInspection, CoordError> {
    inspect_files(
        &mut open_segment(segment_path, false)?,
        &open_pending(pending_dir)?,
        expected_generation_id,
        genesis_digest,
    )
}

pub(in crate::coord) fn inspect_files(
    segment: &mut File,
    pending_dir: &File,
    expected_generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentInspection, CoordError> {
    validate_generation_id(expected_generation_id)?;
    validate_digest("genesis_digest", genesis_digest)?;
    validate_segment_descriptor(segment)?;
    validate_pending_descriptor(pending_dir)?;
    if !pending_names(pending_dir)?.is_empty() {
        return Err(CoordError::new(
            "PENDING_COORD_APPEND",
            "status refuses while a segment append intent requires reconciliation",
        ));
    }
    require_initialized(inspect_segment_file(
        segment,
        expected_generation_id,
        genesis_digest,
    )?)
}

pub(in crate::coord) fn validate_append_request(
    request: &AppendRequest<'_>,
    genesis_digest: &str,
) -> Result<String, CoordError> {
    validate_request(request)?;
    validate_digest("genesis_digest", genesis_digest)?;
    if request.sequence == 1 && request.previous_digest != genesis_digest {
        return Err(CoordError::new(
            "COORD_SEGMENT_POSITION_MISMATCH",
            "sequence-1 append must name the exact generation genesis digest",
        ));
    }
    let request_digest = append_request_digest(request)?;
    encode_frame(request)?;
    Ok(request_digest)
}

#[cfg(test)]
pub(in crate::coord) fn append(
    segment_path: &Path,
    pending_dir: &Path,
    request: &AppendRequest<'_>,
    genesis_digest: &str,
) -> Result<AppendReceipt, CoordError> {
    append_files(
        &mut open_segment(segment_path, true)?,
        &open_pending(pending_dir)?,
        request,
        genesis_digest,
    )
}

#[cfg(test)]
pub(in crate::coord) fn test_crash_after_intent_link() {
    io::test_crash_after_link();
}

pub(in crate::coord) fn append_files(
    segment: &mut File,
    pending_dir: &File,
    request: &AppendRequest<'_>,
    genesis_digest: &str,
) -> Result<AppendReceipt, CoordError> {
    validate_append_request(request, genesis_digest)?;
    validate_segment_descriptor(segment)?;
    validate_pending_descriptor(pending_dir)?;
    if !pending_names(pending_dir)?.is_empty() {
        return Err(CoordError::new(
            "PENDING_COORD_APPEND",
            "append refuses until the existing intent is explicitly reconciled",
        ));
    }
    let inspected = inspect_segment_file(segment, request.generation_id, genesis_digest)?;
    let record_digest = digest_record(request.record)?;
    let request_digest = append_request_digest(request)?;
    if let Some(existing) = inspected.receipt_for_request(request.request_id) {
        if existing.request_digest != request_digest
            || existing.record_digest != record_digest
            || existing.sequence != request.sequence
            || inspected
                .entries
                .iter()
                .find(|entry| entry.request_id == request.request_id)
                .is_none_or(|entry| entry.previous_digest != request.previous_digest)
        {
            return Err(CoordError::new(
                "COORD_REQUEST_CONFLICT",
                "request ID already exists with a different digest, record, sequence, or predecessor",
            ));
        }
        return Ok(existing.clone());
    }
    if request.sequence != inspected.position.next_sequence
        || request.previous_digest != inspected.position.previous_digest
    {
        return Err(CoordError::new(
            "COORD_SEGMENT_POSITION_MISMATCH",
            "append sequence or previous digest differs from the inspected segment position",
        ));
    }

    let frame = encode_frame(request)?;
    let frame_length = u64::try_from(frame.len()).map_err(|_| capacity_error())?;
    let next_length = inspected
        .position
        .byte_length
        .checked_add(frame_length)
        .ok_or_else(capacity_error)?;
    if next_length > MAX_SEGMENT_BYTES {
        return Err(capacity_error());
    }
    let intent = PendingIntent {
        kind: "coord_segment_append_intent_v2".to_owned(),
        schema_version: 2,
        generation_id: request.generation_id.to_owned(),
        sequence: request.sequence,
        previous_digest: request.previous_digest.to_owned(),
        request_id: request.request_id.to_owned(),
        request_digest: request_digest.clone(),
        segment_offset: inspected.position.byte_length,
        frame_length,
        frame_digest: frame_digest(&frame)?,
        frame_utf8: String::from_utf8(frame).map_err(|_| {
            CoordError::new(
                "INVALID_COORD_RECORD",
                "canonical segment frame is not UTF-8",
            )
        })?,
    };
    publish_intent_at(pending_dir, &intent)?;
    let position =
        reconcile_pending_files(segment, pending_dir, request.generation_id, genesis_digest)?;
    let envelope_digest =
        envelope_digest(&intent.frame_utf8.as_bytes()[..frame_length as usize - 1])?;
    if position.next_sequence != request.sequence + 1 || position.previous_digest != envelope_digest
    {
        return Err(corrupt(
            "reconciled append position differs from the intended envelope",
        ));
    }
    Ok(AppendReceipt {
        sequence: request.sequence,
        envelope_digest,
        record_digest,
        request_id: request.request_id.to_owned(),
        request_digest,
        byte_offset: inspected.position.byte_length,
        frame_length,
    })
}

#[cfg(test)]
pub(in crate::coord) fn reconcile_pending(
    segment_path: &Path,
    pending_dir: &Path,
    expected_generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentPosition, CoordError> {
    reconcile_pending_files(
        &mut open_segment(segment_path, true)?,
        &open_pending(pending_dir)?,
        expected_generation_id,
        genesis_digest,
    )
}

pub(in crate::coord) fn reconcile_pending_files(
    segment: &mut File,
    pending_dir: &File,
    expected_generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentPosition, CoordError> {
    validate_generation_id(expected_generation_id)?;
    validate_digest("genesis_digest", genesis_digest)?;
    validate_segment_descriptor(segment)?;
    validate_pending_descriptor(pending_dir)?;
    let entries = pending_names(pending_dir)?;
    if entries.is_empty() {
        return Ok(require_initialized(inspect_segment_file(
            segment,
            expected_generation_id,
            genesis_digest,
        )?)?
        .position);
    }
    if entries.len() != 1 || entries[0] != INTENT_NAME {
        return Err(CoordError::new(
            "CORRUPT_COORD_PENDING",
            "pending directory must contain exactly the one admitted append intent",
        ));
    }
    let intent = read_intent_at(pending_dir)?;
    validate_intent(&intent, expected_generation_id)?;
    let frame = intent.frame_utf8.as_bytes();
    let bytes = read_bounded(segment, MAX_SEGMENT_BYTES)?;
    let offset = usize::try_from(intent.segment_offset).map_err(|_| capacity_error())?;
    if offset > bytes.len() {
        return Err(CoordError::new(
            "CORRUPT_COORD_PENDING",
            "pending append offset is beyond the current segment",
        ));
    }
    let prefix = &bytes[..offset];
    let before = inspect_bytes(prefix, expected_generation_id, genesis_digest)?;
    if before.position.next_sequence != intent.sequence
        || before.position.previous_digest != intent.previous_digest
        || before.receipt_for_request(&intent.request_id).is_some()
    {
        return Err(CoordError::new(
            "CORRUPT_COORD_PENDING",
            "pending append does not follow the exact committed segment position",
        ));
    }
    let tail = &bytes[offset..];
    if tail.len() > frame.len() || tail != &frame[..tail.len()] {
        return Err(CoordError::new(
            "PARTIAL_COORD_WRITE",
            "segment tail is not an absent, exact-prefix, or exact-complete pending frame",
        ));
    }
    if bytes.len().saturating_add(frame.len() - tail.len()) > MAX_SEGMENT_BYTES as usize {
        return Err(capacity_error());
    }
    segment.seek(SeekFrom::End(0)).map_err(CoordError::io)?;
    segment
        .write_all(&frame[tail.len()..])
        .map_err(CoordError::io)?;
    exact_readback(segment, intent.segment_offset, frame)?;
    segment.sync_data().map_err(CoordError::io)?;
    exact_readback(segment, intent.segment_offset, frame)?;
    let committed = require_initialized(inspect_segment_file(
        segment,
        expected_generation_id,
        genesis_digest,
    )?)?;
    remove_intent_at(pending_dir)?;
    validate_segment_descriptor(segment)?;
    validate_pending_descriptor(pending_dir)?;
    let final_position = require_initialized(inspect_segment_file(
        segment,
        expected_generation_id,
        genesis_digest,
    )?)?
    .position;
    if final_position != committed.position {
        return Err(corrupt("segment changed after pending intent retirement"));
    }
    Ok(final_position)
}

fn inspect_segment_file(
    file: &mut File,
    expected_generation_id: &str,
    genesis_digest: &str,
) -> Result<SegmentInspection, CoordError> {
    let bytes = read_bounded(file, MAX_SEGMENT_BYTES)?;
    inspect_bytes(&bytes, expected_generation_id, genesis_digest)
}

fn encode_frame(request: &AppendRequest<'_>) -> Result<Vec<u8>, CoordError> {
    let envelope = SegmentEnvelope {
        kind: "coord_segment_record_v2".to_owned(),
        schema_version: 2,
        generation_id: request.generation_id.to_owned(),
        sequence: request.sequence,
        previous_digest: request.previous_digest.to_owned(),
        request_id: request.request_id.to_owned(),
        request_digest: append_request_digest(request)?,
        record: request.record.clone(),
    };
    let mut bytes = canonical(&envelope, "segment envelope")?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(CoordError::new(
            "COORD_SEGMENT_CAPACITY_EXCEEDED",
            "canonical segment frame exceeds the one-record bound",
        ));
    }
    Ok(bytes)
}

fn validate_intent(intent: &PendingIntent, generation_id: &str) -> Result<(), CoordError> {
    if intent.kind != "coord_segment_append_intent_v2" || intent.schema_version != 2 {
        return Err(corrupt_pending(
            "pending intent kind or schema is unsupported",
        ));
    }
    validate_generation_id(&intent.generation_id)?;
    validate_digest("previous_digest", &intent.previous_digest)?;
    validate_digest("request_digest", &intent.request_digest)?;
    validate_token("request_id", &intent.request_id)?;
    if intent.generation_id != generation_id {
        return Err(CoordError::new(
            "STALE_COORD_GENERATION",
            "pending intent belongs to another generation",
        ));
    }
    let frame = intent.frame_utf8.as_bytes();
    if frame.len() > MAX_FRAME_BYTES
        || intent.frame_length != frame.len() as u64
        || intent.frame_digest != frame_digest(frame)?
        || frame.last() != Some(&b'\n')
    {
        return Err(corrupt_pending(
            "pending frame length, digest, or LF binding is invalid",
        ));
    }
    let envelope: SegmentEnvelope = bullet_wire::decode_canonical(&frame[..frame.len() - 1])
        .map_err(|error| corrupt_pending(format!("pending frame is not canonical: {error}")))?;
    validate_envelope(&envelope, generation_id)?;
    if envelope.sequence != intent.sequence
        || envelope.previous_digest != intent.previous_digest
        || envelope.request_id != intent.request_id
        || envelope.request_digest != intent.request_digest
    {
        return Err(corrupt_pending(
            "pending metadata does not match its exact frame",
        ));
    }
    Ok(())
}

fn validate_request(request: &AppendRequest<'_>) -> Result<(), CoordError> {
    validate_generation_id(request.generation_id)?;
    validate_digest("previous_digest", request.previous_digest)?;
    validate_token("request_id", request.request_id)?;
    if request.sequence == 0 || request.sequence > 9_007_199_254_740_991 {
        return Err(CoordError::new(
            "INVALID_COORD_SEQUENCE",
            "sequence is outside safe JSON integers",
        ));
    }
    validate_record_position(request)
}

fn validate_envelope(envelope: &SegmentEnvelope, generation_id: &str) -> Result<(), CoordError> {
    if envelope.kind != "coord_segment_record_v2" || envelope.schema_version != 2 {
        return Err(corrupt("segment envelope kind or schema is unsupported"));
    }
    validate_request(&AppendRequest {
        generation_id: &envelope.generation_id,
        sequence: envelope.sequence,
        previous_digest: &envelope.previous_digest,
        request_id: &envelope.request_id,
        record: &envelope.record,
    })?;
    if envelope.request_digest
        != append_request_digest(&AppendRequest {
            generation_id: &envelope.generation_id,
            sequence: envelope.sequence,
            previous_digest: &envelope.previous_digest,
            request_id: &envelope.request_id,
            record: &envelope.record,
        })?
    {
        return Err(corrupt(
            "segment request digest does not bind its canonical append subject",
        ));
    }
    validate_envelope_identity(envelope, generation_id)
}

fn require_initialized(inspected: SegmentInspection) -> Result<SegmentInspection, CoordError> {
    if inspected.entries.is_empty() {
        Err(CoordError::new(
            "EMPTY_COORD_SEGMENT",
            "published coordination status requires a sequence-1 generation transition",
        ))
    } else {
        Ok(inspected)
    }
}

#[cfg(test)]
mod tests;
