use std::io::Read;

use super::*;
use crate::coord::{
    RecoveryReceiptAdoptionRequestV1,
    generation::manifest::{
        ArtifactBinding, FROZEN_LIVE_SOURCE_PATH, GenerationManifestBody, TRUSTED_PREFIX_PATH,
    },
    model::{GENERATION_SCHEMA_VERSION, Record},
    recovery_adoption_verify::{self, ForensicSources},
    state,
};

impl Ledger {
    pub(in crate::coord::store) fn adopt_recovery_receipts<F>(
        &self,
        request: &RecoveryReceiptAdoptionRequestV1,
        clock: F,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce() -> Result<u64, CoordError>,
    {
        let transaction = self.transact_loaded(
            &request.expected_watermark.generation_id,
            request.request_id.as_str(),
            |loaded| {
                request.validate()?;
                require_watermark(&request.expected_watermark, &loaded.view.watermark)?;
                reject_existing_subject(request, &loaded.view.records)?;
                let GenerationManifestBody::RecoveryBaseline(manifest) = &loaded.manifest.body
                else {
                    return Err(insufficient(
                        "receipt adoption requires a recovery generation",
                    ));
                };
                let trusted = read_artifact(
                    &loaded.files,
                    TRUSTED_PREFIX_PATH,
                    &manifest.artifacts.trusted_prefix,
                )?;
                let frozen = read_artifact(
                    &loaded.files,
                    FROZEN_LIVE_SOURCE_PATH,
                    &manifest.artifacts.frozen_live_source,
                )?;
                let verified = recovery_adoption_verify::verify(
                    &self.family_root,
                    request,
                    manifest,
                    &loaded.view.records,
                    &loaded.inspection.entries,
                    ForensicSources {
                        trusted_prefix: &trusted,
                        frozen_live_source: &frozen,
                    },
                )?;
                let at_unix_ms = clock()?;
                let body = crate::coord::model::RecoveryReceiptAdoptionRecordV1::verified(
                    request.clone(),
                    manifest.recovery_operator.clone(),
                    manifest.recovery_policy_sha256.as_str().to_owned(),
                    manifest.operator_decision_sha256.as_str().to_owned(),
                    manifest.replay_contract_version,
                    manifest.replay_contract_sha256.as_str().to_owned(),
                    verified.recovery_orchestrator,
                    verified.reviewer,
                )?;
                let record = Record::RecoveryReceiptAdoptionV1 {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms,
                    body,
                };
                let mut prospective = loaded.view.records.clone();
                prospective.push(record.clone());
                state::summaries(&prospective, at_unix_ms)?;
                Ok(record)
            },
        )?;
        require_exact_replay(request, &transaction)?;
        Ok(transaction)
    }
}

fn read_artifact(
    files: &fs::GenerationFiles,
    path: &str,
    binding: &ArtifactBinding,
) -> Result<Vec<u8>, CoordError> {
    let mut file = files.artifact(path, binding.byte_length)?;
    let capacity = usize::try_from(binding.byte_length)
        .map_err(|_| mismatch("recovery artifact length does not fit this host"))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes).map_err(CoordError::io)?;
    if bytes.len() != capacity {
        return Err(mismatch("recovery artifact changed while being read"));
    }
    files.revalidate_artifact(path, &file, binding.byte_length)?;
    Ok(bytes)
}

fn require_watermark(
    expected: &crate::coord::RecoveryAdoptionWatermarkV1,
    actual: &LedgerWatermark,
) -> Result<(), CoordError> {
    if expected.generation_id != actual.generation_id
        || expected.manifest_blake3 != actual.manifest_blake3
        || expected.last_sequence != actual.last_sequence
        || expected.next_sequence != actual.next_sequence
        || expected.head_envelope_blake3 != actual.head_envelope_digest
        || expected.last_record_blake3 != actual.last_record_digest
        || expected.last_request_id.as_str() != actual.last_request_id
        || expected.last_request_blake3 != actual.last_request_digest
        || expected.byte_length != actual.byte_length
    {
        return Err(CoordError::new(
            "STALE_COORD_WATERMARK",
            "recovery adoption expected another complete ledger watermark",
        ));
    }
    Ok(())
}

fn reject_existing_subject(
    request: &RecoveryReceiptAdoptionRequestV1,
    records: &[Record],
) -> Result<(), CoordError> {
    let adoption_id = request.adoption_id()?;
    for record in records {
        let Record::RecoveryReceiptAdoptionV1 { body, .. } = record else {
            continue;
        };
        if body.adoption_id() == adoption_id
            || (body.request().subject.repo == request.subject.repo
                && body.request().subject.git_expectation.commit_oid
                    == request.subject.git_expectation.commit_oid)
        {
            return Err(CoordError::new(
                "RECOVERY_ADOPTION_CONFLICT",
                "recovery claim group or repository commit was already adopted",
            ));
        }
    }
    Ok(())
}

fn require_exact_replay(
    request: &RecoveryReceiptAdoptionRequestV1,
    transaction: &RequestTransaction,
) -> Result<(), CoordError> {
    let Record::RecoveryReceiptAdoptionV1 { body, .. } = &transaction.record else {
        return Err(CoordError::new(
            "COORD_REQUEST_CONFLICT",
            "request ID already binds another coordination record kind",
        ));
    };
    if bullet_wire::canonical_json(body.request()).map_err(wire)?
        != bullet_wire::canonical_json(request).map_err(wire)?
    {
        return Err(CoordError::new(
            "COORD_REQUEST_CONFLICT",
            "request ID already binds another recovery adoption subject",
        ));
    }
    Ok(())
}

fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_EVIDENCE_MISMATCH", reason)
}

fn insufficient(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_AUTHORITY_INSUFFICIENT", reason)
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    CoordError::new(
        "INVALID_RECOVERY_ADOPTION",
        format!("cannot canonicalize recovery adoption request: {error}"),
    )
}

#[cfg(all(test, target_os = "linux"))]
#[path = "adoption/tests.rs"]
mod tests;
