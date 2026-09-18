use std::path::Path;

use crate::coord::{
    CoordError,
    generation::{
        manifest::{
            CurrentPointer, FROZEN_LIVE_SOURCE_PATH, GenerationManifest, GenerationManifestBody,
            INTERRUPTED_CAPTURE_PATH, TAINTED_GENERATION_PATH, TRUSTED_PREFIX_PATH,
        },
        recovery,
        segment::{self, AppendReceipt, AppendRequest, SegmentInspection},
    },
    model::{GENERATION_SCHEMA_VERSION, Record},
    state::summaries,
};

use super::{
    GenerationKind, LedgerView, LedgerWatermark, RequestReceipt, RequestTransaction,
    fs::GenerationFiles, invalid,
};

impl RequestTransaction {
    pub(super) fn request_records(&self) -> Result<&[Record], CoordError> {
        let trusted = match self.watermark.kind {
            GenerationKind::Genesis => 0,
            GenerationKind::Recovery {
                trusted_records, ..
            } => usize::try_from(trusted_records)
                .map_err(|_| invalid("trusted record prefix does not fit this host"))?,
        };
        let segment = usize::try_from(self.receipt.sequence)
            .map_err(|_| invalid("request sequence does not fit this host"))?;
        let end = trusted
            .checked_add(segment)
            .ok_or_else(|| invalid("request projection prefix overflowed"))?;
        self.view
            .records
            .get(..end)
            .ok_or_else(|| invalid("locked replay is shorter than the request projection prefix"))
    }

    pub(super) fn pre_request_watermark(&self) -> Result<LedgerWatermark, CoordError> {
        let previous_sequence = self
            .receipt
            .sequence
            .checked_sub(1)
            .ok_or_else(|| invalid("request has no preceding ledger sequence"))?;
        let previous = self
            .view
            .requests
            .values()
            .find(|receipt| receipt.sequence == previous_sequence)
            .ok_or_else(|| invalid("request has no retained preceding receipt"))?;
        request_watermark(&self.view.watermark, previous)
    }
}

pub(super) fn generation(
    files: &GenerationFiles,
    pointer: &CurrentPointer,
    manifest: &GenerationManifest,
    inspection: &SegmentInspection,
) -> Result<Vec<Record>, CoordError> {
    let first = inspection
        .entries
        .first()
        .ok_or_else(|| invalid("generation segment has no sequence-1 transition"))?;
    match &manifest.body {
        GenerationManifestBody::Genesis(body) => {
            let expected = Record::GenesisV2 {
                schema_version: GENERATION_SCHEMA_VERSION,
                generation_id: manifest.generation_id().as_str().to_owned(),
                manifest_blake3: pointer.manifest_blake3().to_owned(),
                created_at_unix_ms: body.created_at_unix_ms,
            };
            same_record(&first.record, &expected)?;
            Ok(inspection
                .entries
                .iter()
                .map(|entry| entry.record.clone())
                .collect())
        }
        GenerationManifestBody::RecoveryBaseline(body) => {
            let mut trusted = files.artifact(
                TRUSTED_PREFIX_PATH,
                body.artifacts.trusted_prefix.byte_length,
            )?;
            let mut interrupted = files.artifact(
                INTERRUPTED_CAPTURE_PATH,
                body.artifacts.interrupted_capture.byte_length,
            )?;
            let mut tainted = files.artifact(
                TAINTED_GENERATION_PATH,
                body.artifacts.tainted_generation.byte_length,
            )?;
            let mut frozen = files.artifact(
                FROZEN_LIVE_SOURCE_PATH,
                body.artifacts.frozen_live_source.byte_length,
            )?;
            let prefix = recovery::verify_retained_artifacts(
                &mut trusted,
                &mut interrupted,
                &mut tainted,
                &mut frozen,
                manifest,
            )?;
            for (path, length, file) in [
                (
                    TRUSTED_PREFIX_PATH,
                    body.artifacts.trusted_prefix.byte_length,
                    &trusted,
                ),
                (
                    INTERRUPTED_CAPTURE_PATH,
                    body.artifacts.interrupted_capture.byte_length,
                    &interrupted,
                ),
                (
                    TAINTED_GENERATION_PATH,
                    body.artifacts.tainted_generation.byte_length,
                    &tainted,
                ),
                (
                    FROZEN_LIVE_SOURCE_PATH,
                    body.artifacts.frozen_live_source.byte_length,
                    &frozen,
                ),
            ] {
                files.revalidate_artifact(path, file, length)?;
            }
            recovery_transition(pointer, manifest, inspection, body, prefix)
        }
    }
}

fn recovery_transition(
    pointer: &CurrentPointer,
    manifest: &GenerationManifest,
    inspection: &SegmentInspection,
    body: &crate::coord::generation::manifest::RecoveryManifestBody,
    mut records: Vec<Record>,
) -> Result<Vec<Record>, CoordError> {
    let expected = recovery::baseline_identity(manifest)?;
    let first = &inspection.entries[0];
    same_record(&first.record, &expected.record)?;
    if expected.genesis_digest != chain_genesis(pointer)?
        || first.request_id != expected.request_id
        || first.request_digest != expected.request_digest
    {
        return Err(invalid(
            "recovery baseline request differs from manifest authority",
        ));
    }
    for (index, entry) in inspection.entries.iter().enumerate() {
        if let Record::RecoveryReceiptAdoptionV1 { body, .. } = &entry.record {
            crate::coord::recovery_adoption_verify::verify_replay_evidence(
                body,
                &inspection.entries[..index],
            )?;
        }
    }
    records.extend(inspection.entries.iter().map(|entry| entry.record.clone()));
    summaries(&records, body.recovered_at_unix_ms)?;
    Ok(records)
}

pub(super) fn single_genesis(
    inspection: &SegmentInspection,
    expected: &Record,
    request_id: &str,
) -> Result<(), CoordError> {
    if inspection.entries.len() != 1 || inspection.position.next_sequence != 2 {
        return Err(invalid(
            "GENESIS segment is not the unique sequence-1 subject",
        ));
    }
    let entry = &inspection.entries[0];
    same_record(&entry.record, expected)?;
    let digest = segment::validate_append_request(
        &AppendRequest {
            generation_id: &entry.generation_id,
            sequence: 1,
            previous_digest: &entry.previous_digest,
            request_id,
            record: expected,
        },
        &entry.previous_digest,
    )?;
    if entry.sequence != 1 || entry.request_id != request_id || entry.request_digest != digest {
        return Err(invalid("GENESIS request identity is not canonical"));
    }
    Ok(())
}

pub(super) fn view(
    generation_dir: &Path,
    pointer: &CurrentPointer,
    manifest: &GenerationManifest,
    inspection: &SegmentInspection,
    records: Vec<Record>,
) -> Result<LedgerView, CoordError> {
    let last_sequence = inspection
        .position
        .next_sequence
        .checked_sub(1)
        .ok_or_else(|| invalid("initialized generation has no sequence watermark"))?;
    let last = inspection
        .entries
        .last()
        .ok_or_else(|| invalid("initialized segment has no last record"))?;
    Ok(LedgerView {
        records,
        watermark: LedgerWatermark {
            generation_id: pointer.generation_id().as_str().to_owned(),
            manifest_blake3: pointer.manifest_blake3().to_owned(),
            kind: match &manifest.body {
                GenerationManifestBody::Genesis(_) => GenerationKind::Genesis,
                GenerationManifestBody::RecoveryBaseline(body) => GenerationKind::Recovery {
                    incident_at_unix_ms: body.incident_at_unix_ms,
                    recovered_at_unix_ms: body.recovered_at_unix_ms,
                    trusted_records: body.trusted_record_count,
                },
            },
            last_sequence,
            next_sequence: inspection.position.next_sequence,
            head_envelope_digest: inspection.position.previous_digest.clone(),
            last_record_digest: last.receipt.record_digest.clone(),
            last_request_id: last.request_id.clone(),
            last_request_digest: last.request_digest.clone(),
            byte_length: inspection.position.byte_length,
        },
        source: generation_dir.join("events.jsonl"),
        requests: inspection
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.request_id.clone(),
                    receipt(pointer.generation_id().as_str(), &entry.receipt),
                )
            })
            .collect(),
    })
}

pub(super) fn receipt(generation_id: &str, value: &AppendReceipt) -> RequestReceipt {
    RequestReceipt {
        generation_id: generation_id.to_owned(),
        request_id: value.request_id.clone(),
        sequence: value.sequence,
        request_digest: value.request_digest.clone(),
        record_digest: value.record_digest.clone(),
        envelope_digest: value.envelope_digest.clone(),
        byte_offset: value.byte_offset,
        frame_length: value.frame_length,
    }
}

pub(super) fn request_watermark(
    current: &LedgerWatermark,
    receipt: &RequestReceipt,
) -> Result<LedgerWatermark, CoordError> {
    let next_sequence = receipt
        .sequence
        .checked_add(1)
        .ok_or_else(|| invalid("request sequence overflowed"))?;
    let byte_length = receipt
        .byte_offset
        .checked_add(receipt.frame_length)
        .ok_or_else(|| invalid("request byte watermark overflowed"))?;
    Ok(LedgerWatermark {
        generation_id: current.generation_id.clone(),
        manifest_blake3: current.manifest_blake3.clone(),
        kind: current.kind.clone(),
        last_sequence: receipt.sequence,
        next_sequence,
        head_envelope_digest: receipt.envelope_digest.clone(),
        last_record_digest: receipt.record_digest.clone(),
        last_request_id: receipt.request_id.clone(),
        last_request_digest: receipt.request_digest.clone(),
        byte_length,
    })
}

fn chain_genesis(pointer: &CurrentPointer) -> Result<String, CoordError> {
    pointer
        .manifest_blake3()
        .strip_prefix("blake3:")
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid("CURRENT manifest digest is not tagged BLAKE3"))
}

fn same_record(actual: &Record, expected: &Record) -> Result<(), CoordError> {
    if bullet_wire::canonical_json(actual).map_err(wire)?
        == bullet_wire::canonical_json(expected).map_err(wire)?
    {
        Ok(())
    } else {
        Err(invalid("sequence-1 record differs from manifest authority"))
    }
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("canonical coordination record failed: {error}"))
}
