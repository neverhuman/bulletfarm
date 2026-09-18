use serde::Serialize;

use crate::coord::{
    ClaimInput, CommitReceiptGroupInput, CommitReceiptInput, CoordError,
    GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput, ReceiptCorrectionInput,
    model::{GENERATION_SCHEMA_VERSION, Record},
    state::{normalized_paths, validate_claim_id},
    validate_commit_oid, validate_field, validate_repo_name, validate_ttl,
};

const COMMAND_DOMAIN: &str = "bullet-family.coord.command.v2";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum CommandSubjectV2 {
    Claim {
        agent: String,
        lane: String,
        repo: String,
        paths: Vec<String>,
        ttl_seconds: u64,
    },
    Heartbeat {
        claim_id: String,
        agent: String,
        ttl_seconds: u64,
        note: Option<String>,
    },
    Handoff {
        claim_id: String,
        agent: String,
        proof_command: String,
        proof_exit_code: i32,
        changed_paths: Vec<String>,
        commit_oid: Option<String>,
    },
    Receipt {
        claim_id: String,
        orchestrator: String,
        commit_oid: String,
        committed_paths: Vec<String>,
    },
    CorrectReceipt {
        claim_id: String,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        committed_paths: Vec<String>,
        reason: String,
    },
    ReceiptGroup {
        claim_ids: Vec<String>,
        orchestrator: String,
        commit_oid: String,
    },
    CorrectReceiptGroup {
        claim_ids: Vec<String>,
        orchestrator: String,
        previous_commit_oid: String,
        commit_oid: String,
        reason: String,
    },
}

impl CommandSubjectV2 {
    pub(super) fn claim(input: &ClaimInput) -> Result<Self, CoordError> {
        validate_field("agent", &input.agent)?;
        validate_field("lane", &input.lane)?;
        validate_repo_name(&input.repo)?;
        validate_ttl(input.ttl_seconds)?;
        Ok(Self::Claim {
            agent: input.agent.clone(),
            lane: input.lane.clone(),
            repo: input.repo.clone(),
            paths: normalized_paths(&input.paths)?,
            ttl_seconds: input.ttl_seconds,
        })
    }

    pub(super) fn heartbeat(input: &HeartbeatInput) -> Result<Self, CoordError> {
        validate_claim_id(&input.claim_id)?;
        validate_field("agent", &input.agent)?;
        validate_ttl(input.ttl_seconds)?;
        if let Some(note) = &input.note {
            validate_field("note", note)?;
        }
        Ok(Self::Heartbeat {
            claim_id: input.claim_id.clone(),
            agent: input.agent.clone(),
            ttl_seconds: input.ttl_seconds,
            note: input.note.clone(),
        })
    }

    pub(super) fn handoff(input: &HandoffInput) -> Result<Self, CoordError> {
        validate_claim_id(&input.claim_id)?;
        validate_field("agent", &input.agent)?;
        validate_field("proof_command", &input.proof_command)?;
        if input.proof_exit_code != 0 {
            return Err(CoordError::new(
                "PROOF_FAILED",
                "handoff requires a proof command with exit code 0",
            ));
        }
        if input.commit_oid.is_some() {
            return Err(CoordError::new(
                "COMMIT_REQUIRES_RECEIPT",
                "only an orchestrator commit receipt may attach a commit OID",
            ));
        }
        Ok(Self::Handoff {
            claim_id: input.claim_id.clone(),
            agent: input.agent.clone(),
            proof_command: input.proof_command.clone(),
            proof_exit_code: input.proof_exit_code,
            changed_paths: normalized_paths(&input.changed_paths)?,
            commit_oid: None,
        })
    }

    pub(super) fn receipt(input: &CommitReceiptInput) -> Result<Self, CoordError> {
        validate_claim_id(&input.claim_id)?;
        validate_field("orchestrator", &input.orchestrator)?;
        validate_commit_oid(&input.commit_oid)?;
        Ok(Self::Receipt {
            claim_id: input.claim_id.clone(),
            orchestrator: input.orchestrator.clone(),
            commit_oid: input.commit_oid.clone(),
            committed_paths: normalized_paths(&input.committed_paths)?,
        })
    }

    pub(super) fn correct_receipt(input: &ReceiptCorrectionInput) -> Result<Self, CoordError> {
        validate_claim_id(&input.claim_id)?;
        validate_field("orchestrator", &input.orchestrator)?;
        validate_field("reason", &input.reason)?;
        validate_commit_oid(&input.previous_commit_oid)?;
        validate_commit_oid(&input.commit_oid)?;
        Ok(Self::CorrectReceipt {
            claim_id: input.claim_id.clone(),
            orchestrator: input.orchestrator.clone(),
            previous_commit_oid: input.previous_commit_oid.clone(),
            commit_oid: input.commit_oid.clone(),
            committed_paths: normalized_paths(&input.committed_paths)?,
            reason: input.reason.clone(),
        })
    }

    pub(super) fn receipt_group(input: &CommitReceiptGroupInput) -> Result<Self, CoordError> {
        validate_field("orchestrator", &input.orchestrator)?;
        validate_commit_oid(&input.commit_oid)?;
        Ok(Self::ReceiptGroup {
            claim_ids: normalized_group_claim_ids(&input.claim_ids)?,
            orchestrator: input.orchestrator.clone(),
            commit_oid: input.commit_oid.clone(),
        })
    }

    pub(super) fn correct_receipt_group(
        input: &GroupReceiptCorrectionInput,
    ) -> Result<Self, CoordError> {
        validate_field("orchestrator", &input.orchestrator)?;
        validate_field("reason", &input.reason)?;
        validate_commit_oid(&input.previous_commit_oid)?;
        validate_commit_oid(&input.commit_oid)?;
        Ok(Self::CorrectReceiptGroup {
            claim_ids: normalized_group_claim_ids(&input.claim_ids)?,
            orchestrator: input.orchestrator.clone(),
            previous_commit_oid: input.previous_commit_oid.clone(),
            commit_oid: input.commit_oid.clone(),
            reason: input.reason.clone(),
        })
    }

    pub(super) fn from_record(record: &Record) -> Result<Self, CoordError> {
        if record.schema_version() != GENERATION_SCHEMA_VERSION {
            return Err(invalid("stored request is not a schema-2 command record"));
        }
        match record {
            Record::Claim {
                at_unix_ms,
                agent,
                lane,
                repo,
                paths,
                expires_unix_ms,
                ..
            } => Self::claim(&ClaimInput {
                agent: agent.clone(),
                lane: lane.clone(),
                repo: repo.clone(),
                paths: paths.clone(),
                ttl_seconds: ttl_seconds(*at_unix_ms, *expires_unix_ms)?,
            }),
            Record::Heartbeat {
                at_unix_ms,
                claim_id,
                agent,
                expires_unix_ms,
                note,
                ..
            } => Self::heartbeat(&HeartbeatInput {
                claim_id: claim_id.clone(),
                agent: agent.clone(),
                ttl_seconds: ttl_seconds(*at_unix_ms, *expires_unix_ms)?,
                note: note.clone(),
            }),
            Record::Handoff {
                claim_id,
                agent,
                proof_command,
                proof_exit_code,
                changed_paths,
                commit_oid,
                ..
            } => Self::handoff(&HandoffInput {
                claim_id: claim_id.clone(),
                agent: agent.clone(),
                proof_command: proof_command.clone(),
                proof_exit_code: *proof_exit_code,
                changed_paths: changed_paths.clone(),
                commit_oid: commit_oid.clone(),
            }),
            Record::CommitReceipt {
                claim_id,
                orchestrator,
                commit_oid,
                committed_paths,
                ..
            } => Self::receipt(&CommitReceiptInput {
                claim_id: claim_id.clone(),
                orchestrator: orchestrator.clone(),
                commit_oid: commit_oid.clone(),
                committed_paths: committed_paths.clone(),
            }),
            Record::CommitReceiptCorrection {
                claim_id,
                orchestrator,
                previous_commit_oid,
                commit_oid,
                committed_paths,
                reason,
                ..
            } => Self::correct_receipt(&ReceiptCorrectionInput {
                claim_id: claim_id.clone(),
                orchestrator: orchestrator.clone(),
                previous_commit_oid: previous_commit_oid.clone(),
                commit_oid: commit_oid.clone(),
                committed_paths: committed_paths.clone(),
                reason: reason.clone(),
            }),
            Record::CommitReceiptGroup {
                orchestrator,
                commit_oid,
                receipts,
                ..
            } => Self::receipt_group(&CommitReceiptGroupInput {
                claim_ids: receipts
                    .iter()
                    .map(|receipt| receipt.claim_id.clone())
                    .collect(),
                orchestrator: orchestrator.clone(),
                commit_oid: commit_oid.clone(),
            }),
            Record::CommitReceiptGroupCorrection {
                orchestrator,
                previous_commit_oid,
                commit_oid,
                receipts,
                reason,
                ..
            } => Self::correct_receipt_group(&GroupReceiptCorrectionInput {
                claim_ids: receipts
                    .iter()
                    .map(|receipt| receipt.claim_id.clone())
                    .collect(),
                orchestrator: orchestrator.clone(),
                previous_commit_oid: previous_commit_oid.clone(),
                commit_oid: commit_oid.clone(),
                reason: reason.clone(),
            }),
            Record::GenesisV2 { .. }
            | Record::RecoveryBaselineV2 { .. }
            | Record::RecoveryReceiptAdoptionV1 { .. }
            | Record::RecoveryProofReceiptV1 { .. }
            | Record::RecoveryReviewReceiptV1 { .. } => Err(invalid(
                "generation or recovery record cannot be an ordinary command",
            )),
        }
    }

    pub(super) fn digest(&self) -> Result<String, CoordError> {
        let bytes = bullet_wire::canonical_json(self).map_err(wire)?;
        let digest = bullet_wire::hash_framed_bytes(COMMAND_DOMAIN, &bytes).map_err(wire)?;
        Ok(format!("blake3:{}", digest.to_hex()))
    }
}

pub(in crate::coord) fn record_time(record: &Record) -> Result<u64, CoordError> {
    match record {
        Record::Claim { at_unix_ms, .. }
        | Record::Heartbeat { at_unix_ms, .. }
        | Record::Handoff { at_unix_ms, .. }
        | Record::CommitReceipt { at_unix_ms, .. }
        | Record::CommitReceiptCorrection { at_unix_ms, .. }
        | Record::CommitReceiptGroup { at_unix_ms, .. }
        | Record::CommitReceiptGroupCorrection { at_unix_ms, .. }
        | Record::RecoveryReceiptAdoptionV1 { at_unix_ms, .. }
        | Record::RecoveryProofReceiptV1 { at_unix_ms, .. }
        | Record::RecoveryReviewReceiptV1 { at_unix_ms, .. } => Ok(*at_unix_ms),
        Record::GenesisV2 {
            created_at_unix_ms, ..
        } => Ok(*created_at_unix_ms),
        Record::RecoveryBaselineV2 { body, .. } => Ok(body.recovered_at_unix_ms),
    }
}

fn ttl_seconds(at: u64, expires: u64) -> Result<u64, CoordError> {
    let millis = expires
        .checked_sub(at)
        .ok_or_else(|| invalid("stored command expiry precedes its timestamp"))?;
    if millis % 1_000 != 0 {
        return Err(invalid("stored command TTL is not whole seconds"));
    }
    let ttl = millis / 1_000;
    validate_ttl(ttl).map_err(|error| invalid(error.to_string()))?;
    Ok(ttl)
}

fn normalized_group_claim_ids(claim_ids: &[String]) -> Result<Vec<String>, CoordError> {
    for claim_id in claim_ids {
        validate_claim_id(claim_id)?;
    }
    let mut normalized = claim_ids.to_vec();
    normalized.sort();
    if normalized.len() < 2 {
        return Err(CoordError::new(
            "RECEIPT_GROUP_REQUIRED",
            "a grouped receipt requires at least two claims",
        ));
    }
    if normalized.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CoordError::new(
            "DUPLICATE_CLAIM_ID",
            "grouped receipt claim IDs must be unique",
        ));
    }
    Ok(normalized)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_COMMAND", reason)
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("cannot canonicalize command subject: {error}"))
}
