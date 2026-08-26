use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    ClaimInput, CoordError,
    model::{ClaimState, ClaimSummary, Record},
    receipt_state, validate_field, validate_path, validate_repo_name,
};

static CLAIM_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) fn summaries(
    records: &[Record],
    now: u64,
) -> Result<BTreeMap<String, ClaimSummary>, CoordError> {
    let mut claims = BTreeMap::new();
    for record in records {
        match record {
            Record::Claim {
                at_unix_ms,
                claim_id,
                agent,
                lane,
                repo,
                paths,
                expires_unix_ms,
                ..
            } => {
                validate_claim_record(record)?;
                if claims.contains_key(claim_id) {
                    return Err(corrupt(format!("duplicate claim id {claim_id}")));
                }
                claims.insert(
                    claim_id.clone(),
                    ClaimSummary {
                        claim_id: claim_id.clone(),
                        agent: agent.clone(),
                        lane: lane.clone(),
                        repo: repo.clone(),
                        paths: paths.clone(),
                        claimed_at_unix_ms: *at_unix_ms,
                        last_event_unix_ms: *at_unix_ms,
                        expires_unix_ms: *expires_unix_ms,
                        state: ClaimState::Active,
                        proof_command: None,
                        changed_paths: Vec::new(),
                        commit_oid: None,
                        commit_orchestrator: None,
                        commit_recorded_at_unix_ms: None,
                    },
                );
            }
            Record::Heartbeat {
                at_unix_ms,
                claim_id,
                agent,
                expires_unix_ms,
                note,
                ..
            } => {
                validate_event_fields(claim_id, agent, *at_unix_ms)?;
                validate_window(*at_unix_ms, *expires_unix_ms)?;
                if let Some(note) = note {
                    validate_field("note", note).map_err(as_corrupt)?;
                }
                let claim = event_claim(&mut claims, claim_id, agent, *at_unix_ms)?;
                claim.last_event_unix_ms = *at_unix_ms;
                claim.expires_unix_ms = *expires_unix_ms;
            }
            Record::Handoff {
                at_unix_ms,
                claim_id,
                agent,
                proof_command,
                proof_exit_code,
                changed_paths,
                commit_oid,
                ..
            } => {
                validate_event_fields(claim_id, agent, *at_unix_ms)?;
                validate_field("proof_command", proof_command).map_err(as_corrupt)?;
                if *proof_exit_code != 0 {
                    return Err(corrupt(format!(
                        "claim {claim_id} handed off with nonzero proof"
                    )));
                }
                let paths = normalized_paths(changed_paths).map_err(as_corrupt)?;
                if &paths != changed_paths {
                    return Err(corrupt(format!(
                        "claim {claim_id} has noncanonical changed paths"
                    )));
                }
                if commit_oid.is_some() {
                    return Err(corrupt(format!(
                        "claim {claim_id} attached a commit before orchestrator receipt"
                    )));
                }
                let claim = event_claim(&mut claims, claim_id, agent, *at_unix_ms)?;
                reject_outside_claim(&claim.paths, changed_paths).map_err(as_corrupt)?;
                claim.last_event_unix_ms = *at_unix_ms;
                claim.state = ClaimState::HandedOff;
                claim.proof_command = Some(proof_command.clone());
                claim.changed_paths.clone_from(changed_paths);
                claim.commit_oid = None;
            }
            Record::CommitReceipt { .. }
            | Record::CommitReceiptCorrection { .. }
            | Record::CommitReceiptGroup { .. }
            | Record::CommitReceiptGroupCorrection { .. } => {
                receipt_state::apply(record, &mut claims)?
            }
        }
    }
    for claim in claims.values_mut() {
        claim.refresh_state(now);
    }
    Ok(claims)
}

pub(super) fn require_active(
    records: &[Record],
    claim_id: &str,
    agent: &str,
    now: u64,
) -> Result<ClaimSummary, CoordError> {
    let claims = summaries(records, now)?;
    let claim = claims
        .get(claim_id)
        .ok_or_else(|| CoordError::new("CLAIM_NOT_FOUND", format!("no claim {claim_id}")))?;
    if claim.agent != agent {
        return Err(CoordError::new(
            "CLAIM_OWNER_MISMATCH",
            format!("claim {claim_id} belongs to {}", claim.agent),
        ));
    }
    if claim.state != ClaimState::Active {
        return Err(CoordError::new(
            "CLAIM_NOT_ACTIVE",
            format!("claim {claim_id} is {:?}", claim.state),
        ));
    }
    Ok(claim.clone())
}

pub(super) fn reject_overlap(
    claims: &BTreeMap<String, ClaimSummary>,
    repo: &str,
    paths: &[String],
) -> Result<(), CoordError> {
    for claim in claims
        .values()
        .filter(|claim| claim.state == ClaimState::Active && claim.repo == repo)
    {
        for requested in paths {
            if let Some(existing) = claim
                .paths
                .iter()
                .find(|path| paths_overlap(path, requested))
            {
                return Err(CoordError::new(
                    "CLAIM_OVERLAP",
                    format!(
                        "{repo}:{requested} overlaps active {} path {existing} owned by {}",
                        claim.claim_id, claim.agent
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn reject_outside_claim(
    claimed: &[String],
    changed: &[String],
) -> Result<(), CoordError> {
    for path in changed {
        if !claimed.iter().any(|root| contains_path(root, path)) {
            return Err(CoordError::new(
                "PATH_OUTSIDE_CLAIM",
                format!("changed path {path} is outside the completed claim"),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_receipt_coverage(
    handoff_scopes: &[String],
    commit_paths: &[String],
) -> Result<(), CoordError> {
    if commit_paths.is_empty() {
        return Err(CoordError::new(
            "COMMIT_PATH_MISMATCH",
            "receipted commit has no leaf paths",
        ));
    }
    for path in commit_paths {
        if !handoff_scopes
            .iter()
            .any(|scope| contains_path(scope, path))
        {
            return Err(CoordError::new(
                "COMMIT_PATH_MISMATCH",
                format!("commit leaf {path} is outside every handed-off path"),
            ));
        }
    }
    for scope in handoff_scopes {
        if !commit_paths.iter().any(|path| contains_path(scope, path)) {
            return Err(CoordError::new(
                "COMMIT_PATH_MISMATCH",
                format!("handed-off path {scope} covers no commit leaf"),
            ));
        }
    }
    Ok(())
}

pub(super) fn receipt_paths_for_scopes(
    handoff_scopes: &[String],
    commit_paths: &[String],
) -> Result<Vec<String>, CoordError> {
    let paths = commit_paths
        .iter()
        .filter(|path| {
            handoff_scopes
                .iter()
                .any(|scope| contains_path(scope, path))
        })
        .cloned()
        .collect::<Vec<_>>();
    validate_receipt_coverage(handoff_scopes, &paths)?;
    Ok(paths)
}

pub(super) fn normalized_paths(paths: &[String]) -> Result<Vec<String>, CoordError> {
    if paths.is_empty() {
        return Err(CoordError::new(
            "PATH_REQUIRED",
            "at least one --path is required",
        ));
    }
    let mut normalized = paths
        .iter()
        .map(|path| validate_path(path))
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

pub(super) fn expiry(now: u64, ttl_seconds: u64) -> Result<u64, CoordError> {
    let ttl_millis = ttl_seconds
        .checked_mul(1_000)
        .ok_or_else(|| CoordError::new("INVALID_TTL", "TTL overflows milliseconds"))?;
    now.checked_add(ttl_millis)
        .ok_or_else(|| CoordError::new("INVALID_TTL", "TTL overflows the clock"))
}

pub(super) fn claim_id(input: &ClaimInput, paths: &[String], now: u64) -> String {
    let sequence = CLAIM_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut hash = blake3::Hasher::new();
    hash.update(b"bullet-family.coord.claim.v1\0");
    for field in [
        now.to_string(),
        std::process::id().to_string(),
        sequence.to_string(),
        input.agent.clone(),
        input.lane.clone(),
        input.repo.clone(),
    ] {
        hash.update(&(field.len() as u64).to_le_bytes());
        hash.update(field.as_bytes());
    }
    for path in paths {
        hash.update(&(path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
    }
    format!("clm_{}", hash.finalize().to_hex())
}

fn validate_claim_record(record: &Record) -> Result<(), CoordError> {
    let Record::Claim {
        at_unix_ms,
        claim_id,
        agent,
        lane,
        repo,
        paths,
        expires_unix_ms,
        ..
    } = record
    else {
        return Err(corrupt("expected claim record"));
    };
    validate_event_fields(claim_id, agent, *at_unix_ms)?;
    validate_field("lane", lane).map_err(as_corrupt)?;
    validate_repo_name(repo).map_err(as_corrupt)?;
    validate_window(*at_unix_ms, *expires_unix_ms)?;
    let normalized = normalized_paths(paths).map_err(as_corrupt)?;
    if &normalized != paths {
        return Err(corrupt(format!("claim {claim_id} has noncanonical paths")));
    }
    Ok(())
}

fn validate_event_fields(claim_id: &str, agent: &str, at: u64) -> Result<(), CoordError> {
    validate_field("claim_id", claim_id).map_err(as_corrupt)?;
    validate_field("agent", agent).map_err(as_corrupt)?;
    if at == 0 {
        return Err(corrupt("event timestamp must be nonzero"));
    }
    Ok(())
}

fn validate_window(at: u64, expires: u64) -> Result<(), CoordError> {
    let ttl = expires
        .checked_sub(at)
        .ok_or_else(|| corrupt("claim expiry precedes its event"))?;
    if !(30_000..=86_400_000).contains(&ttl) {
        return Err(corrupt("claim expiry is outside the permitted TTL"));
    }
    Ok(())
}

fn event_claim<'a>(
    claims: &'a mut BTreeMap<String, ClaimSummary>,
    claim_id: &str,
    agent: &str,
    at: u64,
) -> Result<&'a mut ClaimSummary, CoordError> {
    let claim = claims
        .get_mut(claim_id)
        .ok_or_else(|| corrupt(format!("event references missing claim {claim_id}")))?;
    if claim.agent != agent {
        return Err(corrupt(format!(
            "event agent does not own claim {claim_id}"
        )));
    }
    if claim.state == ClaimState::HandedOff {
        return Err(corrupt(format!(
            "claim {claim_id} has an event after handoff"
        )));
    }
    if at < claim.last_event_unix_ms {
        return Err(corrupt(format!("claim {claim_id} time moved backwards")));
    }
    Ok(claim)
}

fn paths_overlap(left: &str, right: &str) -> bool {
    contains_path(left, right) || contains_path(right, left)
}

pub(super) fn contains_path(root: &str, path: &str) -> bool {
    root == "."
        || root == path
        || path
            .strip_prefix(root)
            .is_some_and(|tail| tail.starts_with('/'))
}

fn as_corrupt(error: CoordError) -> CoordError {
    corrupt(error.to_string())
}

fn corrupt(reason: impl Into<String>) -> CoordError {
    CoordError::new("CORRUPT_COORD_LOG", reason)
}

#[cfg(test)]
mod tests {
    use super::{contains_path, paths_overlap};

    #[test]
    fn overlap_is_segment_aware() {
        assert!(paths_overlap("src", "src/main.rs"));
        assert!(paths_overlap(".", "src/main.rs"));
        assert!(!paths_overlap("src", "src-old/main.rs"));
        assert!(contains_path("Cargo.toml", "Cargo.toml"));
    }
}
