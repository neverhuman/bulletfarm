use crate::coord::{
    Applied, ClaimInput, ClaimSummary, CommitReceiptGroupInput, CommitReceiptInput, CoordError,
    GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput, MutationEnvelope,
    ReceiptCorrectionInput,
    git::{RepositoryGuard, commit_paths, verify_repository},
    model::{ClaimState, GENERATION_SCHEMA_VERSION, GroupReceipt, Record},
    state::{
        claim_id, expiry, receipt_paths_for_scopes, reject_outside_claim, reject_overlap,
        require_active, summaries, validate_receipt_coverage,
    },
};

use super::{
    CoordStore,
    ledger::{Ledger, LedgerView, RequestTransaction},
    projection,
    subject::CommandSubjectV2,
};

struct GuardedRecord {
    record: Record,
    repository: RepositoryGuard,
}

fn guarded(record: Record, repository: RepositoryGuard) -> GuardedRecord {
    GuardedRecord { record, repository }
}

impl CoordStore {
    pub fn claim(
        &self,
        envelope: &MutationEnvelope<ClaimInput>,
    ) -> Result<Applied<ClaimSummary>, CoordError> {
        let subject = CommandSubjectV2::claim(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::Claim {
                agent,
                lane,
                repo,
                paths,
                ttl_seconds,
            } = &subject
            else {
                return Err(invalid("claim normalized to another command"));
            };
            let repository = verify_repository(&self.root, repo)?;
            let claims = summaries(&view.records, now)?;
            reject_overlap(&claims, repo, paths)?;
            Ok(guarded(
                Record::Claim {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    claim_id: claim_id(
                        envelope.expected_generation_id.as_str(),
                        envelope.request_id.as_str(),
                        agent,
                        lane,
                        repo,
                        paths,
                        *ttl_seconds,
                    )?,
                    agent: agent.clone(),
                    lane: lane.clone(),
                    repo: repo.clone(),
                    paths: paths.clone(),
                    expires_unix_ms: expiry(now, *ttl_seconds)?,
                },
                repository,
            ))
        })?;
        projection::one(transaction, subject.digest()?)
    }

    pub fn heartbeat(
        &self,
        envelope: &MutationEnvelope<HeartbeatInput>,
    ) -> Result<Applied<ClaimSummary>, CoordError> {
        let subject = CommandSubjectV2::heartbeat(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::Heartbeat {
                claim_id,
                agent,
                ttl_seconds,
                note,
            } = &subject
            else {
                return Err(invalid("heartbeat normalized to another command"));
            };
            let claim = require_active(&view.records, claim_id, agent, now)?;
            let repository = verify_repository(&self.root, &claim.repo)?;
            Ok(guarded(
                Record::Heartbeat {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    claim_id: claim_id.clone(),
                    agent: agent.clone(),
                    expires_unix_ms: expiry(now, *ttl_seconds)?,
                    note: note.clone(),
                },
                repository,
            ))
        })?;
        projection::one(transaction, subject.digest()?)
    }

    pub fn handoff(
        &self,
        envelope: &MutationEnvelope<HandoffInput>,
    ) -> Result<Applied<ClaimSummary>, CoordError> {
        let subject = CommandSubjectV2::handoff(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::Handoff {
                claim_id,
                agent,
                proof_command,
                proof_exit_code,
                changed_paths,
                commit_oid,
            } = &subject
            else {
                return Err(invalid("handoff normalized to another command"));
            };
            let claim = require_active(&view.records, claim_id, agent, now)?;
            let repository = verify_repository(&self.root, &claim.repo)?;
            reject_outside_claim(&claim.paths, changed_paths)?;
            Ok(guarded(
                Record::Handoff {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    claim_id: claim_id.clone(),
                    agent: agent.clone(),
                    proof_command: proof_command.clone(),
                    proof_exit_code: *proof_exit_code,
                    changed_paths: changed_paths.clone(),
                    commit_oid: commit_oid.clone(),
                },
                repository,
            ))
        })?;
        projection::one(transaction, subject.digest()?)
    }

    pub fn receipt(
        &self,
        envelope: &MutationEnvelope<CommitReceiptInput>,
    ) -> Result<Applied<ClaimSummary>, CoordError> {
        let subject = CommandSubjectV2::receipt(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::Receipt {
                claim_id,
                orchestrator,
                commit_oid,
                committed_paths,
            } = &subject
            else {
                return Err(invalid("receipt normalized to another command"));
            };
            let claims = summaries(&view.records, now)?;
            let claim = claims.get(claim_id).ok_or_else(|| not_found(claim_id))?;
            if claim.state != ClaimState::HandedOff || claim.commit_oid.is_some() {
                return Err(CoordError::new(
                    "CLAIM_NOT_RECEIPTABLE",
                    "receipt requires a handed-off claim without an existing commit",
                ));
            }
            validate_input_coverage(&claim.changed_paths, committed_paths)?;
            let actual = commit_paths(&self.root, &claim.repo, commit_oid)?;
            require_exact_paths(commit_oid, &actual.paths, committed_paths)?;
            Ok(guarded(
                Record::CommitReceipt {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    claim_id: claim_id.clone(),
                    orchestrator: orchestrator.clone(),
                    commit_oid: commit_oid.clone(),
                    committed_paths: committed_paths.clone(),
                },
                actual.repository,
            ))
        })?;
        projection::one(transaction, subject.digest()?)
    }

    pub fn correct_receipt(
        &self,
        envelope: &MutationEnvelope<ReceiptCorrectionInput>,
    ) -> Result<Applied<ClaimSummary>, CoordError> {
        let subject = CommandSubjectV2::correct_receipt(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::CorrectReceipt {
                claim_id,
                orchestrator,
                previous_commit_oid,
                commit_oid,
                committed_paths,
                reason,
            } = &subject
            else {
                return Err(invalid("receipt correction normalized to another command"));
            };
            let claims = summaries(&view.records, now)?;
            let claim = claims.get(claim_id).ok_or_else(|| not_found(claim_id))?;
            if claim.commit_oid.as_deref() != Some(previous_commit_oid.as_str()) {
                return Err(CoordError::new(
                    "RECEIPT_CORRECTION_MISMATCH",
                    "correction must bind the currently recorded commit OID",
                ));
            }
            validate_input_coverage(&claim.changed_paths, committed_paths)?;
            let actual = commit_paths(&self.root, &claim.repo, commit_oid)?;
            require_exact_paths(commit_oid, &actual.paths, committed_paths)?;
            Ok(guarded(
                Record::CommitReceiptCorrection {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    claim_id: claim_id.clone(),
                    orchestrator: orchestrator.clone(),
                    previous_commit_oid: previous_commit_oid.clone(),
                    commit_oid: commit_oid.clone(),
                    committed_paths: committed_paths.clone(),
                    reason: reason.clone(),
                },
                actual.repository,
            ))
        })?;
        projection::one(transaction, subject.digest()?)
    }

    pub fn receipt_group(
        &self,
        envelope: &MutationEnvelope<CommitReceiptGroupInput>,
    ) -> Result<Applied<Vec<ClaimSummary>>, CoordError> {
        let subject = CommandSubjectV2::receipt_group(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::ReceiptGroup {
                claim_ids,
                orchestrator,
                commit_oid,
            } = &subject
            else {
                return Err(invalid("group receipt normalized to another command"));
            };
            let (receipts, repository) =
                self.group_receipts(view, claim_ids, commit_oid, None, now)?;
            Ok(guarded(
                Record::CommitReceiptGroup {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    orchestrator: orchestrator.clone(),
                    commit_oid: commit_oid.clone(),
                    receipts,
                },
                repository,
            ))
        })?;
        projection::many(transaction, subject.digest()?)
    }

    pub fn correct_receipt_group(
        &self,
        envelope: &MutationEnvelope<GroupReceiptCorrectionInput>,
    ) -> Result<Applied<Vec<ClaimSummary>>, CoordError> {
        let subject = CommandSubjectV2::correct_receipt_group(&envelope.command)?;
        let transaction = self.transact(envelope, &subject, |view, now| {
            let CommandSubjectV2::CorrectReceiptGroup {
                claim_ids,
                orchestrator,
                previous_commit_oid,
                commit_oid,
                reason,
            } = &subject
            else {
                return Err(invalid("group correction normalized to another command"));
            };
            let (receipts, repository) =
                self.group_receipts(view, claim_ids, commit_oid, Some(previous_commit_oid), now)?;
            Ok(guarded(
                Record::CommitReceiptGroupCorrection {
                    schema_version: GENERATION_SCHEMA_VERSION,
                    at_unix_ms: now,
                    orchestrator: orchestrator.clone(),
                    previous_commit_oid: previous_commit_oid.clone(),
                    commit_oid: commit_oid.clone(),
                    receipts,
                    reason: reason.clone(),
                },
                repository,
            ))
        })?;
        projection::many(transaction, subject.digest()?)
    }

    fn transact<T, F>(
        &self,
        envelope: &MutationEnvelope<T>,
        subject: &CommandSubjectV2,
        make_record: F,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce(&LedgerView, u64) -> Result<GuardedRecord, CoordError>,
    {
        envelope.request_id.validate()?;
        envelope.expected_generation_id.validate()?;
        let requested_digest = subject.digest()?;
        let transaction = Ledger::new(&self.root).transact_guarded(
            envelope.expected_generation_id.as_str(),
            envelope.request_id.as_str(),
            |view| {
                let guarded = make_record(view, self.now()?)?;
                Ok((guarded.record, guarded.repository))
            },
            |repository| repository.revalidate(&self.root),
        )?;
        let stored_digest = CommandSubjectV2::from_record(&transaction.record)?.digest()?;
        if stored_digest != requested_digest {
            return Err(CoordError::new(
                "COORD_REQUEST_CONFLICT",
                "request ID already binds another normalized command subject",
            ));
        }
        Ok(transaction)
    }

    fn group_receipts(
        &self,
        view: &LedgerView,
        claim_ids: &[String],
        commit_oid: &str,
        previous_commit_oid: Option<&String>,
        now: u64,
    ) -> Result<(Vec<GroupReceipt>, RepositoryGuard), CoordError> {
        let claims = summaries(&view.records, now)?;
        let mut selected = Vec::with_capacity(claim_ids.len());
        let mut repo = None;
        let mut handoff_scopes = Vec::new();
        for claim_id in claim_ids {
            let claim = claims.get(claim_id).ok_or_else(|| not_found(claim_id))?;
            if let Some(previous) = previous_commit_oid {
                if claim.commit_oid.as_deref() != Some(previous.as_str()) {
                    return Err(CoordError::new(
                        "RECEIPT_CORRECTION_MISMATCH",
                        format!("claim {claim_id} is not bound to the previous commit"),
                    ));
                }
            } else if claim.state != ClaimState::HandedOff || claim.commit_oid.is_some() {
                return Err(CoordError::new(
                    "CLAIM_NOT_RECEIPTABLE",
                    format!("claim {claim_id} is not an unreceipted handoff"),
                ));
            }
            if repo.as_ref().is_some_and(|value| value != &claim.repo) {
                return Err(CoordError::new(
                    "RECEIPT_REPO_MISMATCH",
                    "all grouped claims must belong to one repository",
                ));
            }
            repo = Some(claim.repo.clone());
            handoff_scopes.extend(claim.changed_paths.iter().cloned());
            selected.push(claim);
        }
        handoff_scopes.sort();
        handoff_scopes.dedup();
        let repo = repo
            .ok_or_else(|| CoordError::new("RECEIPT_GROUP_REQUIRED", "group has no repository"))?;
        let actual = commit_paths(&self.root, &repo, commit_oid)?;
        validate_receipt_coverage(&handoff_scopes, &actual.paths)?;
        let receipts = selected
            .iter()
            .map(|claim| {
                Ok(GroupReceipt {
                    claim_id: claim.claim_id.clone(),
                    committed_paths: receipt_paths_for_scopes(&claim.changed_paths, &actual.paths)?,
                })
            })
            .collect::<Result<Vec<_>, CoordError>>()?;
        Ok((receipts, actual.repository))
    }
}

fn validate_input_coverage(scopes: &[String], committed: &[String]) -> Result<(), CoordError> {
    validate_receipt_coverage(scopes, committed).map_err(|error| {
        CoordError::new(
            "COMMITTED_PATH_MISMATCH",
            format!("receipt leaf paths do not match the handoff: {error}"),
        )
    })
}

fn require_exact_paths(
    commit_oid: &str,
    actual: &[String],
    expected: &[String],
) -> Result<(), CoordError> {
    if actual == expected {
        Ok(())
    } else {
        Err(CoordError::new(
            "COMMIT_PATH_MISMATCH",
            format!(
                "commit {commit_oid} leaf paths {actual:?} differ from receipted leaf paths {expected:?}"
            ),
        ))
    }
}

fn not_found(claim_id: &str) -> CoordError {
    CoordError::new("CLAIM_NOT_FOUND", format!("no claim {claim_id}"))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_COMMAND", reason)
}
