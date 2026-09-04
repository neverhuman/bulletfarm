use std::{collections::BTreeMap, path::PathBuf};

use crate::coord::{
    ClaimInput, ClaimState, CommitReceiptGroupInput, CommitReceiptInput, CoordError, CoordStore,
    DEFAULT_TTL_SECONDS, GenerationId, GenesisInput, GroupReceiptCorrectionInput, HandoffInput,
    HeartbeatInput, MutationEnvelope, ReceiptCorrectionInput, RequestId,
};

pub(super) mod recovery;

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
            crate::coord::consume_wave0_and_inventory(
                PathBuf::from(inventory).as_path(),
                PathBuf::from(wave0).as_path(),
                store.family_root(),
            )?;
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

#[derive(Default)]
pub(super) struct Options {
    values: BTreeMap<String, Vec<String>>,
    flags: Vec<String>,
}

impl Options {
    pub(super) fn parse(args: &[String]) -> Result<Self, CoordError> {
        let mut options = Self::default();
        let mut index = 0;
        while index < args.len() {
            let name = args[index].strip_prefix("--").ok_or_else(|| {
                CoordError::new("INVALID_ARGUMENT", format!("unexpected {}", args[index]))
            })?;
            if matches!(name, "json" | "all") {
                if options.flags.iter().any(|flag| flag == name) {
                    return Err(CoordError::new(
                        "DUPLICATE_OPTION",
                        format!("--{name} repeated"),
                    ));
                }
                options.flags.push(name.to_owned());
                index += 1;
                continue;
            }
            let value = args.get(index + 1).ok_or_else(|| {
                CoordError::new("MISSING_VALUE", format!("--{name} needs a value"))
            })?;
            options
                .values
                .entry(name.to_owned())
                .or_default()
                .push(value.clone());
            index += 2;
        }
        Ok(options)
    }

    pub(super) fn one(&self, name: &str) -> Result<String, CoordError> {
        let values = self
            .values
            .get(name)
            .ok_or_else(|| CoordError::new("MISSING_OPTION", format!("--{name} is required")))?;
        if values.len() != 1 {
            return Err(CoordError::new(
                "DUPLICATE_OPTION",
                format!("--{name} must appear once"),
            ));
        }
        Ok(values[0].clone())
    }

    pub(super) fn optional_one(&self, name: &str) -> Result<Option<String>, CoordError> {
        match self.values.get(name) {
            None => Ok(None),
            Some(values) if values.len() == 1 => Ok(Some(values[0].clone())),
            Some(_) => Err(CoordError::new(
                "DUPLICATE_OPTION",
                format!("--{name} must appear at most once"),
            )),
        }
    }

    pub(super) fn many(&self, name: &str) -> Result<Vec<String>, CoordError> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| CoordError::new("MISSING_OPTION", format!("--{name} is required")))
    }

    pub(super) fn u64_or(&self, name: &str, default: u64) -> Result<u64, CoordError> {
        let Some(value) = self.optional_one(name)? else {
            return Ok(default);
        };
        parse_ascii_u64(&value).ok_or_else(|| {
            CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value"))
        })
    }

    fn u32_or(&self, name: &str, default: u32) -> Result<u32, CoordError> {
        let value = self.u64_or(name, u64::from(default))?;
        u32::try_from(value)
            .map_err(|_| CoordError::new("INVALID_OPTION", format!("--{name} exceeds u32")))
    }

    fn i32_or(&self, name: &str, default: i32) -> Result<i32, CoordError> {
        let Some(value) = self.optional_one(name)? else {
            return Ok(default);
        };
        parse_ascii_i32(&value).ok_or_else(|| {
            CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value"))
        })
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }

    pub(super) fn reject_flags(&self) -> Result<(), CoordError> {
        if self.flags.is_empty() {
            Ok(())
        } else {
            Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{}", self.flags[0]),
            ))
        }
    }

    fn reject_values(&self) -> Result<(), CoordError> {
        if self.values.is_empty() {
            Ok(())
        } else {
            let name = self.values.keys().next().expect("checked non-empty");
            Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{name}"),
            ))
        }
    }

    fn reject_unknown_flags(&self, allowed: &[&str]) -> Result<(), CoordError> {
        if let Some(flag) = self
            .flags
            .iter()
            .find(|flag| !allowed.contains(&flag.as_str()))
        {
            return Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{flag}"),
            ));
        }
        Ok(())
    }

    pub(super) fn reject_unknown_values(&self, allowed: &[&str]) -> Result<(), CoordError> {
        if let Some(name) = self
            .values
            .keys()
            .find(|name| !allowed.contains(&name.as_str()))
        {
            return Err(CoordError::new(
                "UNKNOWN_OPTION",
                format!("unexpected --{name}"),
            ));
        }
        Ok(())
    }
}

fn parse_ascii_u64(value: &str) -> Option<u64> {
    let digits = value.strip_prefix('+').unwrap_or(value);
    if digits.is_empty() {
        return None;
    }
    digits.bytes().try_fold(0_u64, |number, byte| {
        byte.is_ascii_digit()
            .then_some(byte - b'0')
            .and_then(|digit| number.checked_mul(10)?.checked_add(u64::from(digit)))
    })
}

fn parse_ascii_i32(value: &str) -> Option<i32> {
    let (negative, digits) = if let Some(digits) = value.strip_prefix('-') {
        (true, digits)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let magnitude = parse_ascii_u64(digits)?;
    if negative {
        (magnitude <= i32::MAX as u64 + 1).then(|| -(magnitude as i64) as i32)
    } else {
        (magnitude <= i32::MAX as u64).then_some(magnitude as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn action_allowlists_reject_unused_options() {
        let options = Options::parse(&[
            "--agent".to_owned(),
            "agent-a".to_owned(),
            "--untrusted".to_owned(),
            "value".to_owned(),
        ])
        .unwrap();
        let error = options.reject_unknown_values(&["agent"]).unwrap_err();
        assert_eq!(error.code(), "UNKNOWN_OPTION");
    }
}
