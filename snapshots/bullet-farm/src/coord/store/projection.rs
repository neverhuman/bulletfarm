use crate::coord::{
    Applied, COORD_SCHEMA_VERSION, ClaimSummary, CommandReceipt, CoordError, Status, StatusOrigin,
    Watermark, model::Record, state::summaries,
};

use super::{
    ledger::{GenerationKind, LedgerView, LedgerWatermark, RequestTransaction},
    subject::record_time,
};

pub(super) fn status(view: LedgerView, observed_at_unix_ms: u64) -> Result<Status, CoordError> {
    let claims = summaries(&view.records, observed_at_unix_ms)?
        .into_values()
        .collect();
    let watermark = &view.watermark;
    Ok(Status {
        schema_version: COORD_SCHEMA_VERSION,
        generation_id: watermark.generation_id.clone(),
        manifest_blake3: watermark.manifest_blake3.clone(),
        origin: origin(&watermark.kind),
        as_of_sequence: watermark.last_sequence,
        next_sequence: watermark.next_sequence,
        last_request_id: watermark.last_request_id.clone(),
        last_request_blake3: watermark.last_request_digest.clone(),
        last_record_blake3: watermark.last_record_digest.clone(),
        last_envelope_blake3: watermark.head_envelope_digest.clone(),
        byte_length: watermark.byte_length,
        observed_at_unix_ms,
        source: view.source.display().to_string(),
        claims,
    })
}

pub(super) fn one(
    transaction: RequestTransaction,
    command_subject_blake3: String,
) -> Result<Applied<ClaimSummary>, CoordError> {
    let ids = record_claim_ids(&transaction.record);
    if ids.len() != 1 {
        return Err(invalid(
            "single-claim command did not bind exactly one claim",
        ));
    }
    let mut projected = projected_claims(&transaction, &ids)?;
    applied(
        transaction,
        command_subject_blake3,
        projected
            .pop()
            .ok_or_else(|| invalid("request projection omitted its claim"))?,
    )
}

pub(super) fn many(
    transaction: RequestTransaction,
    command_subject_blake3: String,
) -> Result<Applied<Vec<ClaimSummary>>, CoordError> {
    let ids = record_claim_ids(&transaction.record);
    if ids.len() < 2 {
        return Err(invalid("group command did not bind at least two claims"));
    }
    let projected = projected_claims(&transaction, &ids)?;
    applied(transaction, command_subject_blake3, projected)
}

pub(super) fn producer(
    transaction: RequestTransaction,
    command_subject_blake3: String,
    evidence_receipt_id: String,
) -> Result<Applied<String>, CoordError> {
    applied(transaction, command_subject_blake3, evidence_receipt_id)
}

fn projected_claims(
    transaction: &RequestTransaction,
    ids: &[String],
) -> Result<Vec<ClaimSummary>, CoordError> {
    let now = record_time(&transaction.record)?;
    let records = request_records(transaction)?;
    let claims = summaries(records, now)?;
    ids.iter()
        .map(|claim_id| {
            claims
                .get(claim_id)
                .cloned()
                .ok_or_else(|| invalid(format!("request projection omitted claim {claim_id}")))
        })
        .collect()
}

fn request_records(transaction: &RequestTransaction) -> Result<&[Record], CoordError> {
    let trusted = match transaction.watermark.kind {
        GenerationKind::Genesis => 0,
        GenerationKind::Recovery {
            trusted_records, ..
        } => usize::try_from(trusted_records)
            .map_err(|_| invalid("trusted record count does not fit this host"))?,
    };
    let segment = usize::try_from(transaction.receipt.sequence)
        .map_err(|_| invalid("request sequence does not fit this host"))?;
    let end = trusted
        .checked_add(segment)
        .ok_or_else(|| invalid("request projection prefix overflowed"))?;
    transaction
        .view
        .records
        .get(..end)
        .ok_or_else(|| invalid("locked replay is shorter than its request watermark"))
}

fn record_claim_ids(record: &Record) -> Vec<String> {
    match record {
        Record::Claim { claim_id, .. }
        | Record::Heartbeat { claim_id, .. }
        | Record::Handoff { claim_id, .. }
        | Record::CommitReceipt { claim_id, .. }
        | Record::CommitReceiptCorrection { claim_id, .. } => vec![claim_id.clone()],
        Record::CommitReceiptGroup { receipts, .. }
        | Record::CommitReceiptGroupCorrection { receipts, .. } => receipts
            .iter()
            .map(|receipt| receipt.claim_id.clone())
            .collect(),
        Record::RecoveryReceiptAdoptionV1 { body, .. } => body
            .request()
            .subject
            .claims
            .iter()
            .map(|claim| claim.claim_id.clone())
            .collect(),
        Record::GenesisV2 { .. }
        | Record::RecoveryBaselineV2 { .. }
        | Record::RecoveryProofReceiptV1 { .. }
        | Record::RecoveryReviewReceiptV1 { .. } => Vec::new(),
    }
}

fn applied<T>(
    transaction: RequestTransaction,
    command_subject_blake3: String,
    projection: T,
) -> Result<Applied<T>, CoordError> {
    let receipt = CommandReceipt {
        generation_id: transaction.receipt.generation_id.clone(),
        request_id: transaction.receipt.request_id.clone(),
        command_subject_blake3,
        stored_request_blake3: transaction.receipt.request_digest.clone(),
        sequence: transaction.receipt.sequence,
        record_blake3: transaction.receipt.record_digest.clone(),
        envelope_blake3: transaction.receipt.envelope_digest.clone(),
        byte_offset: transaction.receipt.byte_offset,
        frame_length: transaction.receipt.frame_length,
    };
    Ok(Applied {
        receipt,
        watermark: watermark(&transaction.watermark),
        projection,
        replayed: transaction.existing,
    })
}

fn watermark(value: &LedgerWatermark) -> Watermark {
    Watermark {
        generation_id: value.generation_id.clone(),
        manifest_blake3: value.manifest_blake3.clone(),
        last_sequence: value.last_sequence,
        next_sequence: value.next_sequence,
        head_envelope_blake3: value.head_envelope_digest.clone(),
        last_record_blake3: value.last_record_digest.clone(),
        last_request_id: value.last_request_id.clone(),
        last_request_blake3: value.last_request_digest.clone(),
        byte_length: value.byte_length,
    }
}

fn origin(value: &GenerationKind) -> StatusOrigin {
    match value {
        GenerationKind::Genesis => StatusOrigin::Genesis,
        GenerationKind::Recovery {
            incident_at_unix_ms,
            recovered_at_unix_ms,
            trusted_records,
        } => StatusOrigin::Recovered {
            incident_at_unix_ms: *incident_at_unix_ms,
            recovered_at_unix_ms: *recovered_at_unix_ms,
            trusted_records: *trusted_records,
        },
    }
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_PROJECTION", reason)
}
