use std::io::Read;

use sha2::{Digest, Sha256};

use super::*;
use crate::coord::{
    RecoveryReceiptAdoptionRequestV1,
    generation::manifest::{
        ArtifactBinding, FROZEN_LIVE_SOURCE_PATH, GenerationManifestBody, TRUSTED_PREFIX_PATH,
    },
    model::{
        GENERATION_SCHEMA_VERSION, Record, RecoveryAdoptionWatermarkV1,
        RecoveryGenerationRecordKindV1, RecoveryGenerationRecordRefV1, RecoveryProductionPlanV1,
        RecoveryProductionWatermarkV1, RecoveryProofReceiptRecordV1, RecoveryProofRequestV1,
        RecoveryReviewReceiptRecordV1, RecoveryReviewRequestV1, produced_adoption_request,
    },
    recovery_adoption_verify::{self, ForensicSources},
    state,
};

const PROOF_CHECKS: u32 = 6;
const PROOF_COMMAND: &[u8] = b"bullet-family/recovery-production/fixed-proof/v1";
const PROOF_OUTPUT_DOMAIN: &str = "bullet-family.coord.recovery-production-proof-output.v1";

impl Ledger {
    pub(in crate::coord::store) fn derive_recovery_plan(
        &self,
    ) -> Result<RecoveryProductionPlanV1, CoordError> {
        self.read_loaded(|loaded| {
            let orchestrator = recovery_manifest(loaded)?.recovery_operator.clone();
            derive_loaded_plan(
                self,
                loaded,
                watermark(&loaded.view.watermark),
                orchestrator,
            )
        })
    }

    pub(in crate::coord::store) fn record_recovery_proof<F>(
        &self,
        request: &RecoveryProofRequestV1,
        clock: F,
    ) -> Result<(RequestTransaction, String), CoordError>
    where
        F: FnOnce() -> Result<u64, CoordError>,
    {
        request.validate()?;
        let transaction = self.transact_loaded(
            &request.plan.expected_watermark.generation_id,
            request.request_id.as_str(),
            |loaded| {
                require_ledger_watermark(&request.plan.expected_watermark, &loaded.view.watermark)?;
                let derived = derive_loaded_plan(
                    self,
                    loaded,
                    request.plan.expected_watermark.clone(),
                    request.plan.recovery_orchestrator.clone(),
                )?;
                same_canonical(&derived, &request.plan, "recovery proof plan changed")?;
                let body = expected_proof(&request.plan)?;
                let at_unix_ms = clock()?;
                let record = Record::RecoveryProofReceiptV1 {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms,
                    body,
                };
                validate_prospective(loaded, &record, at_unix_ms)?;
                Ok(record)
            },
        )?;
        let proof_id = require_exact_proof_replay(request, &transaction)?;
        Ok((transaction, proof_id))
    }

    pub(in crate::coord::store) fn record_recovery_review<F>(
        &self,
        request: &RecoveryReviewRequestV1,
        clock: F,
    ) -> Result<(RequestTransaction, String), CoordError>
    where
        F: FnOnce() -> Result<u64, CoordError>,
    {
        request.validate()?;
        let transaction = self.transact_loaded(
            &request.plan.expected_watermark.generation_id,
            request.request_id.as_str(),
            |loaded| {
                let derived = derive_loaded_plan(
                    self,
                    loaded,
                    request.plan.expected_watermark.clone(),
                    request.plan.recovery_orchestrator.clone(),
                )?;
                same_canonical(&derived, &request.plan, "recovery review plan changed")?;
                let (proof, proof_receipt) = exact_proof(&loaded.view, &request.plan)?;
                let proof_watermark = receipt_watermark(&loaded.view.watermark, proof_receipt)?;
                require_ledger_watermark(&proof_watermark, &loaded.view.watermark)?;
                if request.approval.proof_receipt_ids != vec![proof.proof_receipt_id().to_owned()] {
                    return Err(invalid(
                        "review approval does not name the exact fixed proof",
                    ));
                }
                let body = expected_review(request, proof)?;
                let at_unix_ms = clock()?;
                let record = Record::RecoveryReviewReceiptV1 {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms,
                    body,
                };
                validate_prospective(loaded, &record, at_unix_ms)?;
                Ok(record)
            },
        )?;
        let review_id = require_exact_review_replay(request, &transaction)?;
        Ok((transaction, review_id))
    }

    pub(in crate::coord::store) fn build_recovery_adoption_request(
        &self,
        request: &RecoveryReviewRequestV1,
    ) -> Result<RecoveryReceiptAdoptionRequestV1, CoordError> {
        request.validate()?;
        self.read_loaded(|loaded| {
            let derived = derive_loaded_plan(
                self,
                loaded,
                request.plan.expected_watermark.clone(),
                request.plan.recovery_orchestrator.clone(),
            )?;
            same_canonical(&derived, &request.plan, "adoption request plan changed")?;
            let (proof, proof_receipt) = exact_proof(&loaded.view, &request.plan)?;
            let (review, review_receipt) = exact_review(&loaded.view, request, proof)?;
            let review_watermark = receipt_watermark(&loaded.view.watermark, review_receipt)?;
            require_ledger_watermark(&review_watermark, &loaded.view.watermark)?;
            produced_adoption_request(
                &request.plan,
                adoption_watermark(&loaded.view.watermark)?,
                evidence_ref(proof_receipt, RecoveryGenerationRecordKindV1::ProofReceipt)?,
                evidence_ref(
                    review_receipt,
                    RecoveryGenerationRecordKindV1::ReviewReceipt,
                )?,
            )
            .and_then(|produced| {
                if review.review_receipt_id().is_empty() {
                    Err(invalid("stored review receipt identity is empty"))
                } else {
                    Ok(produced)
                }
            })
        })
    }

    fn read_loaded<T>(
        &self,
        read: impl FnOnce(&mut Loaded) -> Result<T, CoordError>,
    ) -> Result<T, CoordError> {
        let probe = fs::probe(&self.coord_dir)?;
        let lock = match probe.presence() {
            fs::Presence::Absent => return Err(uninitialized()),
            fs::Presence::Legacy => return Err(recovery_required()),
            fs::Presence::Retired => return Err(recovery_in_progress()),
            fs::Presence::Current => probe.into_lock(&self.coord_dir, false)?,
        };
        let mut loaded = self.load_locked(&lock, None, false)?;
        read(&mut loaded)
    }
}

fn derive_loaded_plan(
    ledger: &Ledger,
    loaded: &mut Loaded,
    expected_watermark: RecoveryProductionWatermarkV1,
    recovery_orchestrator: String,
) -> Result<RecoveryProductionPlanV1, CoordError> {
    let manifest = recovery_manifest(loaded)?;
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
    recovery_adoption_verify::derive_plan(
        &ledger.family_root,
        manifest,
        &loaded.view.records,
        ForensicSources {
            trusted_prefix: &trusted,
            frozen_live_source: &frozen,
        },
        expected_watermark,
        recovery_orchestrator,
    )
}

fn recovery_manifest(
    loaded: &Loaded,
) -> Result<&crate::coord::generation::manifest::RecoveryManifestBody, CoordError> {
    match &loaded.manifest.body {
        GenerationManifestBody::RecoveryBaseline(manifest) => Ok(manifest),
        GenerationManifestBody::Genesis(_) => Err(CoordError::new(
            "RECOVERY_AUTHORITY_INSUFFICIENT",
            "recovery production requires a published recovery generation",
        )),
    }
}

fn expected_proof(
    plan: &RecoveryProductionPlanV1,
) -> Result<RecoveryProofReceiptRecordV1, CoordError> {
    let command = format!("sha256:{:x}", Sha256::digest(PROOF_COMMAND));
    let output = format!(
        "sha256:{:x}",
        Sha256::digest(
            bullet_wire::canonical_json(&(PROOF_OUTPUT_DOMAIN, plan))
                .map_err(|error| invalid(error.to_string()))?
        )
    );
    RecoveryProofReceiptRecordV1::verified_pass(
        plan.evidence_subject_blake3.clone(),
        plan.recovery_orchestrator.clone(),
        command,
        output,
        PROOF_CHECKS,
    )
}

fn expected_review(
    request: &RecoveryReviewRequestV1,
    proof: &RecoveryProofReceiptRecordV1,
) -> Result<RecoveryReviewReceiptRecordV1, CoordError> {
    RecoveryReviewReceiptRecordV1::verified_approval(
        request.plan.evidence_subject_blake3.clone(),
        vec![proof.proof_receipt_id().to_owned()],
        request.plan.recovery_orchestrator.clone(),
        request.approval.reviewer.clone(),
        request.approval_sha256.clone(),
    )
}

fn exact_proof<'a>(
    view: &'a LedgerView,
    plan: &RecoveryProductionPlanV1,
) -> Result<(&'a RecoveryProofReceiptRecordV1, &'a RequestReceipt), CoordError> {
    let request = RecoveryProofRequestV1::for_plan(plan.clone())?;
    let receipt = view
        .request(request.request_id.as_str())
        .ok_or_else(|| invalid("fixed recovery proof request is absent"))?;
    let record = record_at(view, receipt.sequence)?;
    let Record::RecoveryProofReceiptV1 { body, .. } = record else {
        return Err(conflict("fixed proof request binds another record kind"));
    };
    if body != &expected_proof(plan)? {
        return Err(conflict("stored fixed proof differs from the exact plan"));
    }
    let pre = pre_watermark(view, receipt)?;
    require_watermark(&plan.expected_watermark, &pre)?;
    Ok((body, receipt))
}

fn exact_review<'a>(
    view: &'a LedgerView,
    request: &RecoveryReviewRequestV1,
    proof: &RecoveryProofReceiptRecordV1,
) -> Result<(&'a RecoveryReviewReceiptRecordV1, &'a RequestReceipt), CoordError> {
    let receipt = view
        .request(request.request_id.as_str())
        .ok_or_else(|| invalid("independent recovery review request is absent"))?;
    let record = record_at(view, receipt.sequence)?;
    let Record::RecoveryReviewReceiptV1 { body, .. } = record else {
        return Err(conflict("review request binds another record kind"));
    };
    if body != &expected_review(request, proof)? {
        return Err(conflict("stored review differs from the exact approval"));
    }
    Ok((body, receipt))
}

fn require_exact_proof_replay(
    request: &RecoveryProofRequestV1,
    transaction: &RequestTransaction,
) -> Result<String, CoordError> {
    require_transaction_prefix(transaction)?;
    require_ledger_watermark(
        &request.plan.expected_watermark,
        &transaction.pre_request_watermark()?,
    )?;
    let Record::RecoveryProofReceiptV1 { body, .. } = &transaction.record else {
        return Err(conflict("proof request binds another record kind"));
    };
    if body != &expected_proof(&request.plan)? {
        return Err(conflict(
            "proof request binds another canonical producer subject",
        ));
    }
    Ok(body.proof_receipt_id().to_owned())
}

fn require_exact_review_replay(
    request: &RecoveryReviewRequestV1,
    transaction: &RequestTransaction,
) -> Result<String, CoordError> {
    require_transaction_prefix(transaction)?;
    let (proof, proof_receipt) = exact_proof(&transaction.view, &request.plan)?;
    let proof_watermark = receipt_watermark(&transaction.view.watermark, proof_receipt)?;
    require_ledger_watermark(&proof_watermark, &transaction.pre_request_watermark()?)?;
    let Record::RecoveryReviewReceiptV1 { body, .. } = &transaction.record else {
        return Err(conflict("review request binds another record kind"));
    };
    if body != &expected_review(request, proof)? {
        return Err(conflict(
            "review request binds another canonical producer subject",
        ));
    }
    Ok(body.review_receipt_id().to_owned())
}

fn require_transaction_prefix(transaction: &RequestTransaction) -> Result<(), CoordError> {
    let last = transaction
        .request_records()?
        .last()
        .ok_or_else(|| invalid("request projection prefix is empty"))?;
    same_canonical(
        last,
        &transaction.record,
        "request projection prefix differs from its exact record",
    )
}

fn validate_prospective(
    loaded: &Loaded,
    record: &Record,
    at_unix_ms: u64,
) -> Result<(), CoordError> {
    let mut records = loaded.view.records.clone();
    records.push(record.clone());
    state::summaries(&records, at_unix_ms)?;
    Ok(())
}

fn record_at(view: &LedgerView, sequence: u64) -> Result<&Record, CoordError> {
    let trusted = match view.watermark.kind {
        GenerationKind::Genesis => 0,
        GenerationKind::Recovery {
            trusted_records, ..
        } => usize::try_from(trusted_records)
            .map_err(|_| invalid("trusted record count does not fit this host"))?,
    };
    let index = trusted
        .checked_add(
            usize::try_from(sequence).map_err(|_| invalid("sequence does not fit this host"))?,
        )
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| invalid("record projection index overflowed"))?;
    view.records
        .get(index)
        .ok_or_else(|| invalid("request record is absent from the locked projection"))
}

fn pre_watermark(
    view: &LedgerView,
    receipt: &RequestReceipt,
) -> Result<RecoveryProductionWatermarkV1, CoordError> {
    let previous = receipt
        .sequence
        .checked_sub(1)
        .and_then(|sequence| {
            view.requests
                .values()
                .find(|candidate| candidate.sequence == sequence)
        })
        .ok_or_else(|| invalid("request has no retained preceding receipt"))?;
    receipt_watermark(&view.watermark, previous)
}

fn receipt_watermark(
    current: &LedgerWatermark,
    receipt: &RequestReceipt,
) -> Result<RecoveryProductionWatermarkV1, CoordError> {
    Ok(watermark(&verify::request_watermark(current, receipt)?))
}

fn watermark(value: &LedgerWatermark) -> RecoveryProductionWatermarkV1 {
    RecoveryProductionWatermarkV1 {
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

fn require_ledger_watermark(
    expected: &RecoveryProductionWatermarkV1,
    actual: &LedgerWatermark,
) -> Result<(), CoordError> {
    require_watermark(expected, &watermark(actual))
}

fn require_watermark(
    expected: &RecoveryProductionWatermarkV1,
    actual: &RecoveryProductionWatermarkV1,
) -> Result<(), CoordError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CoordError::new(
            "STALE_COORD_WATERMARK",
            "recovery producer expected another complete ledger watermark",
        ))
    }
}

fn adoption_watermark(value: &LedgerWatermark) -> Result<RecoveryAdoptionWatermarkV1, CoordError> {
    Ok(RecoveryAdoptionWatermarkV1 {
        generation_id: value.generation_id.clone(),
        manifest_blake3: value.manifest_blake3.clone(),
        last_sequence: value.last_sequence,
        next_sequence: value.next_sequence,
        head_envelope_blake3: value.head_envelope_digest.clone(),
        last_record_blake3: value.last_record_digest.clone(),
        last_request_id: crate::coord::RequestId::parse(value.last_request_id.clone())?,
        last_request_blake3: value.last_request_digest.clone(),
        byte_length: value.byte_length,
    })
}

fn evidence_ref(
    receipt: &RequestReceipt,
    expected_record_kind: RecoveryGenerationRecordKindV1,
) -> Result<RecoveryGenerationRecordRefV1, CoordError> {
    Ok(RecoveryGenerationRecordRefV1 {
        generation_id: receipt.generation_id.clone(),
        sequence: receipt.sequence,
        request_id: crate::coord::RequestId::parse(receipt.request_id.clone())?,
        request_blake3: receipt.request_digest.clone(),
        record_blake3: receipt.record_digest.clone(),
        envelope_blake3: receipt.envelope_digest.clone(),
        byte_offset: receipt.byte_offset,
        frame_length: receipt.frame_length,
        expected_record_kind,
    })
}

fn read_artifact(
    files: &fs::GenerationFiles,
    path: &str,
    binding: &ArtifactBinding,
) -> Result<Vec<u8>, CoordError> {
    let mut file = files.artifact(path, binding.byte_length)?;
    let capacity = usize::try_from(binding.byte_length)
        .map_err(|_| invalid("recovery artifact length does not fit this host"))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes).map_err(CoordError::io)?;
    files.revalidate_artifact(path, &file, binding.byte_length)?;
    if bytes.len() != capacity {
        return Err(invalid("recovery artifact changed while being read"));
    }
    Ok(bytes)
}

fn same_canonical(
    left: &impl serde::Serialize,
    right: &impl serde::Serialize,
    reason: &str,
) -> Result<(), CoordError> {
    if bullet_wire::canonical_json(left).map_err(|error| invalid(error.to_string()))?
        == bullet_wire::canonical_json(right).map_err(|error| invalid(error.to_string()))?
    {
        Ok(())
    } else {
        Err(CoordError::new("RECOVERY_EVIDENCE_MISMATCH", reason))
    }
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

fn conflict(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_REQUEST_CONFLICT", reason)
}

#[cfg(all(test, target_os = "linux"))]
#[path = "recovery_production/tests.rs"]
mod tests;
