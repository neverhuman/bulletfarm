use std::path::PathBuf;

use crate::coord::{
    ClaimInput, ClaimState, CommitReceiptGroupInput, CommitReceiptInput, CoordError, CoordStore,
    DEFAULT_TTL_SECONDS, GenerationId, GenesisInput, GroupReceiptCorrectionInput, HandoffInput,
    HeartbeatInput, MutationEnvelope, ReceiptCorrectionInput, RequestId,
};

pub(super) mod options;
pub(super) mod recovery;
pub(crate) use options::{Options, parse_ascii_u64};

pub(super) fn run(root: PathBuf, args: &[String], usage: &str) -> Result<String, CoordError> {
    let Some(action) = args.first() else {
        return Err(CoordError::new("USAGE", usage));
    };
    let options = Options::parse(&args[1..])?;
    if action == "recovery-build-observe" {
        return recovery::build_observe(&options);
    }
    let store = CoordStore::new(root);
    match action.as_str() {
        "init" => initialize(&store, &options),
        "claim" => claim(&store, &options),
        "heartbeat" => heartbeat(&store, &options),
        "handoff" => handoff(&store, &options),
        "receipt" => receipt(&store, &options),
        "receipt-group" => receipt_group(&store, &options),
        "correct-receipt" => correct_receipt(&store, &options),
        "correct-receipt-group" => correct_receipt_group(&store, &options),
        "recovery-inspect" => recovery::inspect(&store, &options),
        "recovery-provenance" => recovery::provenance(&store, &options),
        "recovery-authorization-draft" => recovery::authorization_draft(&options),
        "recovery-authorization-message" => recovery::authorization_message(&options),
        "recovery-authorization-signature-import" => {
            recovery::authorization_signature_import(&options)
        }
        "recovery-manifest" => recovery::manifest(&store, &options),
        "recover-rollover" => recovery::recover_rollover(&store, &options),
        "recovery-plan" => recovery::plan(&store, &options),
        "recovery-proof" => recovery::proof(&store, &options),
        "recovery-review" => recovery::review(&store, &options),
        "recovery-request" => recovery::request(&store, &options),
        "adopt" => recovery::adopt(&store, &options),
        "wave0-observe" => wave0_observe(&store, &options),
        "wave0-review" => wave0_review(&options),
        "incident-observe" => incident_observe(&options),
        "incident-verify" => incident_verify(&options),
        "status" => status(&store, &options),
        _ => Err(CoordError::new("USAGE", usage)),
    }
}

#[cfg(test)]
pub(crate) fn test_recovery_action(action: &str, args: &[String]) -> Result<String, CoordError> {
    let command = std::iter::once(action.to_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    run(PathBuf::from("/test-only-family"), &command, "test usage")
}

fn initialize(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "operator",
        "policy-sha256",
        "replay-contract-version",
        "replay-contract-sha256",
        "bootstrap-commit",
        "bootstrap-path",
        "wave0-subject",
        "incident-inventory",
    ])?;
    let wave0 = options.optional_one("wave0-subject")?;
    let inventory = options.optional_one("incident-inventory")?;
    match (wave0, inventory) {
        (None, None) => {}
        (Some(wave0), Some(inventory)) => {
            let (validated_inventory, validated_wave0) = crate::coord::consume_wave0_and_inventory(
                PathBuf::from(inventory).as_path(),
                PathBuf::from(wave0).as_path(),
                store.family_root(),
            )?;
            // Persist what was just validated: ADR 0015 requires the exact
            // W0 subject and incident inventory as durable, create-once
            // authority records adjacent to Genesis -- validating and then
            // discarding them left Genesis unable to prove what it consumed.
            let sealed_dir = store.family_root().join(".bullet-family/fresh-genesis");
            {
                use std::os::unix::fs::DirBuilderExt;
                let mut builder = std::fs::DirBuilder::new();
                builder.recursive(true).mode(0o700);
                builder
                    .create(&sealed_dir)
                    .or_else(|error| {
                        if error.kind() == std::io::ErrorKind::AlreadyExists {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })
                    .map_err(CoordError::io)?;
            }
            let publication = crate::coord::fresh_genesis_publish(
                store.family_root().join("bullet-farm").as_path(),
                sealed_dir.join("incident-inventory.v1.json").as_path(),
                sealed_dir.join("wave0-subject.v1.json").as_path(),
                &validated_inventory,
                &validated_wave0,
            )?;
            let _ = publication;
        }
        _ => {
            return Err(CoordError::new(
                "INVALID_FRESH_GENESIS_PRODUCTION",
                "wave0-subject and incident-inventory must be supplied together",
            ));
        }
    }
    let status = store.initialize(&GenesisInput {
        operator: options.one("operator")?,
        policy_sha256: options.one("policy-sha256")?,
        replay_contract_version: options.u32_or("replay-contract-version", 1)?,
        replay_contract_sha256: options.one("replay-contract-sha256")?,
        bootstrap_commit_oid: options.one("bootstrap-commit")?,
        bootstrap_paths: options.many("bootstrap-path")?,
    })?;
    serde_json::to_string_pretty(&status).map_err(CoordError::json)
}

fn claim(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "request-id",
        "expected-generation",
        "agent",
        "lane",
        "repo",
        "path",
        "ttl-seconds",
    ])?;
    let applied = store.claim(&envelope(
        options,
        ClaimInput {
            agent: options.one("agent")?,
            lane: options.one("lane")?,
            repo: options.one("repo")?,
            paths: options.many("path")?,
            ttl_seconds: options.u64_or("ttl-seconds", DEFAULT_TTL_SECONDS)?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn heartbeat(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "request-id",
        "expected-generation",
        "claim",
        "agent",
        "ttl-seconds",
        "note",
    ])?;
    let applied = store.heartbeat(&envelope(
        options,
        HeartbeatInput {
            claim_id: options.one("claim")?,
            agent: options.one("agent")?,
            ttl_seconds: options.u64_or("ttl-seconds", DEFAULT_TTL_SECONDS)?,
            note: options.optional_one("note")?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn handoff(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "request-id",
        "expected-generation",
        "claim",
        "agent",
        "proof",
        "exit-code",
        "changed-path",
    ])?;
    let applied = store.handoff(&envelope(
        options,
        HandoffInput {
            claim_id: options.one("claim")?,
            agent: options.one("agent")?,
            proof_command: options.one("proof")?,
            proof_exit_code: options.i32_or("exit-code", 0)?,
            changed_paths: options.many("changed-path")?,
            commit_oid: None,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn receipt(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "request-id",
        "expected-generation",
        "claim",
        "orchestrator",
        "commit",
        "committed-path",
    ])?;
    let applied = store.receipt(&envelope(
        options,
        CommitReceiptInput {
            claim_id: options.one("claim")?,
            orchestrator: options.one("orchestrator")?,
            commit_oid: options.one("commit")?,
            committed_paths: options.many("committed-path")?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn receipt_group(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "request-id",
        "expected-generation",
        "claim",
        "orchestrator",
        "commit",
    ])?;
    let applied = store.receipt_group(&envelope(
        options,
        CommitReceiptGroupInput {
            claim_ids: options.many("claim")?,
            orchestrator: options.one("orchestrator")?,
            commit_oid: options.one("commit")?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn correct_receipt(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "claim",
        "orchestrator",
        "previous-commit",
        "commit",
        "committed-path",
        "reason",
        "request-id",
        "expected-generation",
    ])?;
    let applied = store.correct_receipt(&envelope(
        options,
        ReceiptCorrectionInput {
            claim_id: options.one("claim")?,
            orchestrator: options.one("orchestrator")?,
            previous_commit_oid: options.one("previous-commit")?,
            commit_oid: options.one("commit")?,
            committed_paths: options.many("committed-path")?,
            reason: options.one("reason")?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn correct_receipt_group(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "claim",
        "orchestrator",
        "previous-commit",
        "commit",
        "reason",
        "request-id",
        "expected-generation",
    ])?;
    let applied = store.correct_receipt_group(&envelope(
        options,
        GroupReceiptCorrectionInput {
            claim_ids: options.many("claim")?,
            orchestrator: options.one("orchestrator")?,
            previous_commit_oid: options.one("previous-commit")?,
            commit_oid: options.one("commit")?,
            reason: options.one("reason")?,
        },
    )?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn status(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_values()?;
    options.reject_unknown_flags(&["json", "all"])?;
    let include_all = options.flag("all");
    let mut status = store.status()?;
    if !include_all {
        status
            .claims
            .retain(|claim| claim.state == ClaimState::Active);
    }
    if options.flag("json") {
        return serde_json::to_string_pretty(&status).map_err(CoordError::json);
    }
    let mut output = format!("coord source: {}\n", status.source);
    if status.claims.is_empty() {
        output.push_str("no active claims");
    } else {
        for claim in status.claims {
            output.push_str(&format!(
                "{} {:?} {} {}:{} [{}]\n",
                claim.claim_id,
                claim.state,
                claim.agent,
                claim.repo,
                claim.paths.join(","),
                claim.lane
            ));
        }
        output.pop();
    }
    Ok(output)
}

fn envelope<T>(options: &Options, command: T) -> Result<MutationEnvelope<T>, CoordError> {
    Ok(MutationEnvelope {
        request_id: RequestId::parse(options.one("request-id")?)?,
        expected_generation_id: GenerationId::parse(options.one("expected-generation")?)?,
        command,
    })
}

/// Observe the four members plus the frozen claim ledger into unreviewed
/// `Wave0FactsV1` (plan G1.1). Refuses dirty members and active claims.
fn wave0_observe(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["producer", "ledger", "out"])?;
    let producer = options.one("producer")?;
    let ledger = PathBuf::from(options.one("ledger")?);
    let out = PathBuf::from(options.one("out")?);
    let now = u64::try_from(chrono_now_ms())
        .map_err(|_| CoordError::new("WAVE0_PRODUCER_INVALID", "clock before epoch"))?;
    crate::coord::wave0_producer::produce_wave0_facts(store.root(), &ledger, &producer, now, &out)?;
    Ok(format!("wave0 facts sealed at {}", out.display()))
}

/// Complete a reviewed `Wave0SubjectV1` from produced facts plus a second
/// principal's review record. Reviewer == producer is refused by the type.
fn wave0_review(options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["facts", "reviewer", "record", "out"])?;
    let facts = PathBuf::from(options.one("facts")?);
    let reviewer = options.one("reviewer")?;
    let record = PathBuf::from(options.one("record")?);
    let out = PathBuf::from(options.one("out")?);
    crate::coord::wave0_producer::produce_wave0_subject(&facts, &reviewer, &record, &out)?;
    Ok(format!("wave0 subject sealed at {}", out.display()))
}

/// Seal the complete pre-move inventory of the frozen coordination directory.
fn incident_observe(options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["coord-dir", "destination-name", "out"])?;
    let coord_dir = PathBuf::from(options.one("coord-dir")?);
    let destination = options.one("destination-name")?;
    let out = PathBuf::from(options.one("out")?);
    crate::coord::wave0_producer::produce_incident_inventory(
        &coord_dir,
        std::ffi::OsStr::new(&destination),
        &out,
    )?;
    Ok(format!("incident inventory sealed at {}", out.display()))
}

/// Prove the operator's relocation moved a byte-identical tree, using the
/// sealed inventory: the source name must be absent and the destination
/// exact. Run between the mv and `coord init`.
fn incident_verify(options: &Options) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["inventory"])?;
    let inventory = PathBuf::from(options.one("inventory")?);
    crate::coord::wave0_producer::verify_incident_inventory(&inventory)?;
    Ok("retired incident inventory verified".to_owned())
}

fn chrono_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
