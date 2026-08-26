use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;

use super::{
    ClaimInput, CommitReceiptGroupInput, CommitReceiptInput, CoordError,
    GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput, ReceiptCorrectionInput,
    git::commit_paths,
    model::{ClaimState, ClaimSummary, GroupReceipt, Record, SCHEMA_VERSION, Status},
    state::{
        claim_id, expiry, normalized_paths, receipt_paths_for_scopes, reject_outside_claim,
        reject_overlap, require_active, summaries, validate_receipt_coverage,
    },
    validate_commit_oid, validate_field, validate_repo_name, validate_ttl,
};

pub struct CoordStore {
    root: PathBuf,
    log_path: PathBuf,
}

const MAX_COORD_RECORD_BYTES: usize = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES;
const MAX_COORD_LOG_BYTES: u64 = 64 * 1024 * 1024;

impl CoordStore {
    pub fn new(root: PathBuf) -> Self {
        let log_path = root.join(".bullet-family/coord/events.jsonl");
        Self { root, log_path }
    }

    pub fn claim(&self, input: &ClaimInput, now: u64) -> Result<ClaimSummary, CoordError> {
        validate_field("agent", &input.agent)?;
        validate_field("lane", &input.lane)?;
        validate_repo_name(&input.repo)?;
        validate_ttl(input.ttl_seconds)?;
        let paths = normalized_paths(&input.paths)?;
        let expires = expiry(now, input.ttl_seconds)?;
        self.mutate(|records| {
            let claims = summaries(records, now)?;
            reject_overlap(&claims, &input.repo, &paths)?;
            let claim_id = claim_id(input, &paths, now);
            let record = Record::Claim {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                claim_id: claim_id.clone(),
                agent: input.agent.clone(),
                lane: input.lane.clone(),
                repo: input.repo.clone(),
                paths: paths.clone(),
                expires_unix_ms: expires,
            };
            let summary = ClaimSummary {
                claim_id,
                agent: input.agent.clone(),
                lane: input.lane.clone(),
                repo: input.repo.clone(),
                paths,
                claimed_at_unix_ms: now,
                last_event_unix_ms: now,
                expires_unix_ms: expires,
                state: ClaimState::Active,
                proof_command: None,
                changed_paths: Vec::new(),
                commit_oid: None,
                commit_orchestrator: None,
                commit_recorded_at_unix_ms: None,
            };
            Ok((record, summary))
        })
    }

    pub fn heartbeat(&self, input: &HeartbeatInput, now: u64) -> Result<ClaimSummary, CoordError> {
        validate_field("agent", &input.agent)?;
        validate_field("claim_id", &input.claim_id)?;
        validate_ttl(input.ttl_seconds)?;
        if let Some(note) = &input.note {
            validate_field("note", note)?;
        }
        let expires = expiry(now, input.ttl_seconds)?;
        self.mutate(|records| {
            let mut claim = require_active(records, &input.claim_id, &input.agent, now)?;
            claim.expires_unix_ms = expires;
            claim.last_event_unix_ms = now;
            let record = Record::Heartbeat {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                claim_id: input.claim_id.clone(),
                agent: input.agent.clone(),
                expires_unix_ms: expires,
                note: input.note.clone(),
            };
            Ok((record, claim))
        })
    }

    pub fn handoff(&self, input: &HandoffInput, now: u64) -> Result<ClaimSummary, CoordError> {
        validate_field("agent", &input.agent)?;
        validate_field("claim_id", &input.claim_id)?;
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
        let changed_paths = normalized_paths(&input.changed_paths)?;
        self.mutate(|records| {
            let mut claim = require_active(records, &input.claim_id, &input.agent, now)?;
            reject_outside_claim(&claim.paths, &changed_paths)?;
            claim.last_event_unix_ms = now;
            claim.state = ClaimState::HandedOff;
            claim.proof_command = Some(input.proof_command.clone());
            claim.changed_paths.clone_from(&changed_paths);
            claim.commit_oid = None;
            let record = Record::Handoff {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                claim_id: input.claim_id.clone(),
                agent: input.agent.clone(),
                proof_command: input.proof_command.clone(),
                proof_exit_code: input.proof_exit_code,
                changed_paths,
                commit_oid: input.commit_oid.clone(),
            };
            Ok((record, claim))
        })
    }

    pub fn status(&self, now: u64) -> Result<Status, CoordError> {
        let file = self.open_log()?;
        FileExt::lock_shared(&file).map_err(CoordError::io)?;
        let records = read_records(&file)?;
        let claims = summaries(&records, now)?.into_values().collect();
        Ok(Status {
            schema_version: SCHEMA_VERSION,
            source: self.log_path.display().to_string(),
            as_of_unix_ms: now,
            claims,
        })
    }

    pub fn receipt(
        &self,
        input: &CommitReceiptInput,
        now: u64,
    ) -> Result<ClaimSummary, CoordError> {
        validate_field("claim_id", &input.claim_id)?;
        validate_field("orchestrator", &input.orchestrator)?;
        validate_commit_oid(&input.commit_oid)?;
        let committed_paths = normalized_paths(&input.committed_paths)?;
        self.mutate(|records| {
            let mut claim = summaries(records, now)?
                .remove(&input.claim_id)
                .ok_or_else(|| {
                    CoordError::new("CLAIM_NOT_FOUND", format!("no claim {}", input.claim_id))
                })?;
            if claim.state != ClaimState::HandedOff || claim.commit_oid.is_some() {
                return Err(CoordError::new(
                    "CLAIM_NOT_RECEIPTABLE",
                    "receipt requires a handed-off claim without an existing commit",
                ));
            }
            validate_input_coverage(&claim.changed_paths, &committed_paths)?;
            let actual = commit_paths(&self.root, &claim.repo, &input.commit_oid)?;
            require_exact_paths(&input.commit_oid, &actual, &committed_paths)?;
            claim.last_event_unix_ms = now;
            claim.commit_oid = Some(input.commit_oid.clone());
            claim.commit_orchestrator = Some(input.orchestrator.clone());
            claim.commit_recorded_at_unix_ms = Some(now);
            let record = Record::CommitReceipt {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                claim_id: input.claim_id.clone(),
                orchestrator: input.orchestrator.clone(),
                commit_oid: input.commit_oid.clone(),
                committed_paths,
            };
            Ok((record, claim))
        })
    }

    pub fn correct_receipt(
        &self,
        input: &ReceiptCorrectionInput,
        now: u64,
    ) -> Result<ClaimSummary, CoordError> {
        validate_field("claim_id", &input.claim_id)?;
        validate_field("orchestrator", &input.orchestrator)?;
        validate_field("reason", &input.reason)?;
        validate_commit_oid(&input.previous_commit_oid)?;
        validate_commit_oid(&input.commit_oid)?;
        let committed_paths = normalized_paths(&input.committed_paths)?;
        self.mutate(|records| {
            let mut claim = summaries(records, now)?
                .remove(&input.claim_id)
                .ok_or_else(|| {
                    CoordError::new("CLAIM_NOT_FOUND", format!("no claim {}", input.claim_id))
                })?;
            if claim.commit_oid.as_deref() != Some(input.previous_commit_oid.as_str()) {
                return Err(CoordError::new(
                    "RECEIPT_CORRECTION_MISMATCH",
                    "correction must bind the currently recorded commit OID",
                ));
            }
            validate_input_coverage(&claim.changed_paths, &committed_paths)?;
            let actual = commit_paths(&self.root, &claim.repo, &input.commit_oid)?;
            require_exact_paths(&input.commit_oid, &actual, &committed_paths)?;
            claim.last_event_unix_ms = now;
            claim.commit_oid = Some(input.commit_oid.clone());
            claim.commit_orchestrator = Some(input.orchestrator.clone());
            claim.commit_recorded_at_unix_ms = Some(now);
            let record = Record::CommitReceiptCorrection {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                claim_id: input.claim_id.clone(),
                orchestrator: input.orchestrator.clone(),
                previous_commit_oid: input.previous_commit_oid.clone(),
                commit_oid: input.commit_oid.clone(),
                committed_paths,
                reason: input.reason.clone(),
            };
            Ok((record, claim))
        })
    }

    pub fn receipt_group(
        &self,
        input: &CommitReceiptGroupInput,
        now: u64,
    ) -> Result<Vec<ClaimSummary>, CoordError> {
        validate_field("orchestrator", &input.orchestrator)?;
        validate_commit_oid(&input.commit_oid)?;
        let claim_ids = normalized_group_claim_ids(&input.claim_ids)?;
        self.mutate(|records| {
            let claims = summaries(records, now)?;
            let mut selected = Vec::with_capacity(claim_ids.len());
            let mut repo = None;
            let mut handoff_scopes = Vec::new();
            for claim_id in &claim_ids {
                let claim = claims.get(claim_id).ok_or_else(|| {
                    CoordError::new("CLAIM_NOT_FOUND", format!("no claim {claim_id}"))
                })?;
                if claim.state != ClaimState::HandedOff || claim.commit_oid.is_some() {
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
                selected.push(claim.clone());
            }
            handoff_scopes.sort();
            handoff_scopes.dedup();
            let repo = repo.ok_or_else(|| {
                CoordError::new("RECEIPT_GROUP_REQUIRED", "group has no repository")
            })?;
            let actual = commit_paths(&self.root, &repo, &input.commit_oid)?;
            validate_receipt_coverage(&handoff_scopes, &actual)?;
            let receipts = selected
                .iter()
                .map(|claim| {
                    Ok(GroupReceipt {
                        claim_id: claim.claim_id.clone(),
                        committed_paths: receipt_paths_for_scopes(&claim.changed_paths, &actual)?,
                    })
                })
                .collect::<Result<Vec<_>, CoordError>>()?;
            for claim in &mut selected {
                claim.last_event_unix_ms = now;
                claim.commit_oid = Some(input.commit_oid.clone());
                claim.commit_orchestrator = Some(input.orchestrator.clone());
                claim.commit_recorded_at_unix_ms = Some(now);
            }
            let record = Record::CommitReceiptGroup {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                orchestrator: input.orchestrator.clone(),
                commit_oid: input.commit_oid.clone(),
                receipts,
            };
            Ok((record, selected))
        })
    }

    pub fn correct_receipt_group(
        &self,
        input: &GroupReceiptCorrectionInput,
        now: u64,
    ) -> Result<Vec<ClaimSummary>, CoordError> {
        validate_field("orchestrator", &input.orchestrator)?;
        validate_field("reason", &input.reason)?;
        validate_commit_oid(&input.previous_commit_oid)?;
        validate_commit_oid(&input.commit_oid)?;
        let claim_ids = normalized_group_claim_ids(&input.claim_ids)?;
        self.mutate(|records| {
            let claims = summaries(records, now)?;
            let mut selected = Vec::with_capacity(claim_ids.len());
            let mut repo = None;
            let mut handoff_scopes = Vec::new();
            for claim_id in &claim_ids {
                let claim = claims.get(claim_id).ok_or_else(|| {
                    CoordError::new("CLAIM_NOT_FOUND", format!("no claim {claim_id}"))
                })?;
                if claim.commit_oid.as_deref() != Some(input.previous_commit_oid.as_str()) {
                    return Err(CoordError::new(
                        "RECEIPT_CORRECTION_MISMATCH",
                        format!("claim {claim_id} is not currently bound to --previous-commit"),
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
                selected.push(claim.clone());
            }
            handoff_scopes.sort();
            handoff_scopes.dedup();
            let repo = repo.ok_or_else(|| {
                CoordError::new("RECEIPT_GROUP_REQUIRED", "group has no repository")
            })?;
            let actual = commit_paths(&self.root, &repo, &input.commit_oid)?;
            validate_receipt_coverage(&handoff_scopes, &actual)?;
            let receipts = selected
                .iter()
                .map(|claim| {
                    Ok(GroupReceipt {
                        claim_id: claim.claim_id.clone(),
                        committed_paths: receipt_paths_for_scopes(&claim.changed_paths, &actual)?,
                    })
                })
                .collect::<Result<Vec<_>, CoordError>>()?;
            for claim in &mut selected {
                claim.last_event_unix_ms = now;
                claim.commit_oid = Some(input.commit_oid.clone());
                claim.commit_orchestrator = Some(input.orchestrator.clone());
                claim.commit_recorded_at_unix_ms = Some(now);
            }
            let record = Record::CommitReceiptGroupCorrection {
                schema_version: SCHEMA_VERSION,
                at_unix_ms: now,
                orchestrator: input.orchestrator.clone(),
                previous_commit_oid: input.previous_commit_oid.clone(),
                commit_oid: input.commit_oid.clone(),
                receipts,
                reason: input.reason.clone(),
            };
            Ok((record, selected))
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn mutate<T>(
        &self,
        decide: impl FnOnce(&[Record]) -> Result<(Record, T), CoordError>,
    ) -> Result<T, CoordError> {
        let mut file = self.open_log()?;
        FileExt::lock_exclusive(&file).map_err(CoordError::io)?;
        let records = read_records(&file)?;
        let (record, output) = decide(&records)?;
        append_record(&mut file, &record)?;
        Ok(output)
    }

    fn open_log(&self) -> Result<File, CoordError> {
        let parent = self
            .log_path
            .parent()
            .ok_or_else(|| CoordError::new("INVALID_ROOT", "coord log has no parent"))?;
        fs::create_dir_all(parent).map_err(CoordError::io)?;
        let mut options = OpenOptions::new();
        options.create(true).read(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&self.log_path).map_err(CoordError::io)
    }
}

fn normalized_group_claim_ids(claim_ids: &[String]) -> Result<Vec<String>, CoordError> {
    for claim_id in claim_ids {
        validate_field("claim_id", claim_id)?;
    }
    let mut normalized = claim_ids.to_vec();
    normalized.sort();
    normalized.dedup();
    if normalized.len() < 2 {
        return Err(CoordError::new(
            "RECEIPT_GROUP_REQUIRED",
            "a grouped receipt requires at least two distinct claims",
        ));
    }
    Ok(normalized)
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

fn read_records(file: &File) -> Result<Vec<Record>, CoordError> {
    let mut reader = file.try_clone().map_err(CoordError::io)?;
    reader.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    read_records_with_limits(
        &mut BufReader::new(reader),
        MAX_COORD_RECORD_BYTES,
        MAX_COORD_LOG_BYTES,
    )
}

fn read_records_with_limits(
    reader: &mut BufReader<File>,
    max_record_bytes: usize,
    max_total_bytes: u64,
) -> Result<Vec<Record>, CoordError> {
    let mut records = Vec::new();
    let mut line = Vec::new();
    let mut total = 0_u64;
    loop {
        line.clear();
        let remaining = max_total_bytes.saturating_sub(total);
        let read_limit = (max_record_bytes as u64 + 2).min(remaining + 1);
        let read = reader
            .by_ref()
            .take(read_limit)
            .read_until(b'\n', &mut line)
            .map_err(CoordError::io)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > max_total_bytes {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("coordination log exceeds {max_total_bytes} bytes"),
            ));
        }

        let index = records.len() + 1;
        if line.last() != Some(&b'\n') {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} has no final LF commit marker"),
            ));
        }
        line.pop();
        if line.last() == Some(&b'\r') {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} uses CRLF instead of its exact LF commit marker"),
            ));
        }
        if line.len() > max_record_bytes {
            return Err(CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} exceeds {max_record_bytes} bytes"),
            ));
        }
        let value = bullet_wire::decode_unique_value(&line).map_err(|error| {
            CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} is invalid strict JSON: {error}"),
            )
        })?;
        let record: Record = serde_json::from_value(value).map_err(|error| {
            CoordError::new(
                "CORRUPT_COORD_LOG",
                format!("line {index} does not match the coordination record schema: {error}"),
            )
        })?;
        if record.schema_version() != SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_SCHEMA",
                format!("line {index} uses an unsupported schema"),
            ));
        }
        records.push(record);
    }
    Ok(records)
}

fn append_record(file: &mut File, record: &Record) -> Result<(), CoordError> {
    append_record_with_limit(file, record, MAX_COORD_LOG_BYTES)
}

fn append_record_with_limit(
    file: &mut File,
    record: &Record,
    max_total_bytes: u64,
) -> Result<(), CoordError> {
    let mut encoded = serde_json::to_vec(record).map_err(CoordError::json)?;
    bullet_wire::decode_unique_value(&encoded).map_err(|error| {
        CoordError::new(
            "INVALID_COORD_RECORD",
            format!("record cannot enter the strict coordination log: {error}"),
        )
    })?;
    encoded.push(b'\n');
    let existing = file.metadata().map_err(CoordError::io)?.len();
    let next = existing
        .checked_add(encoded.len() as u64)
        .ok_or_else(|| CoordError::new("COORD_LOG_CAPACITY_EXCEEDED", "log size overflowed"))?;
    if next > max_total_bytes {
        return Err(CoordError::new(
            "COORD_LOG_CAPACITY_EXCEEDED",
            format!(
                "appending {} bytes to the {existing}-byte coordination log exceeds its {max_total_bytes}-byte bound",
                encoded.len()
            ),
        ));
    }
    let written = file.write(&encoded).map_err(CoordError::io)?;
    if written != encoded.len() {
        return Err(CoordError::new(
            "PARTIAL_COORD_WRITE",
            format!("wrote {written} of {} bytes", encoded.len()),
        ));
    }
    file.sync_data().map_err(CoordError::io)
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Write};

    use super::{CoordStore, append_record_with_limit, read_records_with_limits};
    use crate::coord::model::Record;

    fn claim(at: &str, expires: &str) -> String {
        format!(
            r#"{{"kind":"claim","schema_version":1,"at_unix_ms":{at},"claim_id":"clm_test","agent":"test-agent","lane":"test-lane","repo":"bullet-farm","paths":["src"],"expires_unix_ms":{expires}}}"#
        )
    }

    fn ledger(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("temporary ledger");
        file.write_all(bytes).expect("write ledger");
        file.flush().expect("flush ledger");
        file
    }

    #[test]
    fn coordination_ledger_refuses_unsafe_numbers_and_bounded_input() {
        let root = tempfile::tempdir().expect("temporary family root");
        let path = root.path().join(".bullet-family/coord/events.jsonl");
        std::fs::create_dir_all(path.parent().expect("coord parent")).expect("coord directory");
        std::fs::write(
            &path,
            format!("{}\n", claim("9007199254740992", "9007199254770992")),
        )
        .expect("unsafe ledger");
        let error = CoordStore::new(root.path().to_path_buf())
            .status(1)
            .expect_err("an unsafe coordination integer must fail closed");
        assert_eq!(error.code(), "CORRUPT_COORD_LOG");
        assert!(error.to_string().contains("UNSAFE_JSON_INTEGER"));

        let valid = claim("1000", "31000");
        for (bytes, reason) in [
            (valid.as_bytes().to_vec(), "no final LF commit marker"),
            (format!("{valid}\r\n").into_bytes(), "uses CRLF"),
        ] {
            std::fs::write(&path, bytes).expect("hostile commit marker");
            let error = CoordStore::new(root.path().to_path_buf())
                .status(1)
                .expect_err("a non-LF coordination commit marker must fail closed");
            assert!(error.to_string().contains(reason), "{error}");
        }

        let per_line = ledger(format!("{valid}\n").as_bytes());
        let error = read_records_with_limits(
            &mut BufReader::new(per_line.reopen().expect("reopen ledger")),
            valid.len() - 1,
            4096,
        )
        .expect_err("an oversized line must fail closed");
        assert!(error.to_string().contains("line 1 exceeds"));

        let two_records = format!("{valid}\n{valid}\n");
        let over_total = ledger(two_records.as_bytes());
        let error = read_records_with_limits(
            &mut BufReader::new(over_total.reopen().expect("reopen ledger")),
            valid.len(),
            two_records.len() as u64 - 1,
        )
        .expect_err("an oversized ledger must fail closed");
        assert!(error.to_string().contains("coordination log exceeds"));

        let value = bullet_wire::decode_unique_value(valid.as_bytes()).expect("strict record");
        let record: Record = serde_json::from_value(value).expect("typed record");
        let encoded_len = serde_json::to_vec(&record).expect("encoded record").len() as u64 + 1;
        let mut append_boundary = ledger(format!("{valid}\n").as_bytes());
        let exact_limit = append_boundary
            .as_file()
            .metadata()
            .expect("ledger metadata")
            .len()
            + encoded_len;
        append_record_with_limit(append_boundary.as_file_mut(), &record, exact_limit)
            .expect("an append ending exactly at the bound is admitted");
        assert_eq!(
            append_boundary
                .as_file()
                .metadata()
                .expect("ledger metadata")
                .len(),
            exact_limit
        );
        let error = append_record_with_limit(append_boundary.as_file_mut(), &record, exact_limit)
            .expect_err("an append crossing the bound must fail before writing");
        assert_eq!(error.code(), "COORD_LOG_CAPACITY_EXCEEDED");
        assert_eq!(
            append_boundary
                .as_file()
                .metadata()
                .expect("ledger metadata")
                .len(),
            exact_limit,
            "a refused append must leave the exact-bound ledger unchanged"
        );
    }
}
