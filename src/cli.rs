use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use crate::coord::{
    ClaimInput, ClaimState, CommitReceiptGroupInput, CommitReceiptInput, CoordError, CoordStore,
    DEFAULT_TTL_SECONDS, GroupReceiptCorrectionInput, HandoffInput, HeartbeatInput,
    ReceiptCorrectionInput, discover_family_root, unix_millis,
};

const USAGE: &str = "usage: bullet-family [--root PATH] <doctor --json|setup --root PATH --source jeryu --cargo-bin ABSOLUTE_PATH --node-bin ABSOLUTE_PATH --npm-cli ABSOLUTE_PATH [--offline]|release <build|verify|extract|receipt-verify> [options]|checkout verify|hub check|deps check|lock <generate --tag VERSION --subjects ABSOLUTE_PATH|verify --tag VERSION>|fuse --source <local|lock>|check <fast|required|release> [options]|coord <claim|heartbeat|handoff|receipt|receipt-group|correct-receipt|correct-receipt-group|status> [options]>";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutcome {
    output: String,
    exit_code: u8,
}

impl CliOutcome {
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }

    pub fn output(&self) -> &str {
        &self.output
    }
}

/// Process-facing command execution. Existing [`run`] callers retain their exact behavior.
pub fn execute(
    args: impl IntoIterator<Item = OsString>,
    current_dir: Result<PathBuf, std::io::Error>,
) -> Result<CliOutcome, CoordError> {
    let args = args.into_iter().collect::<Vec<_>>();
    if is_command(&args, "check") {
        return execute_check(args, current_dir);
    }
    // `doctor` reports its own verdict in its exit status, so a stranger
    // scripting on exit code can never read a BLOCKED hub as success.
    if is_command(&args, "doctor") {
        return execute_doctor(args, current_dir);
    }
    run(args, current_dir).map(|output| CliOutcome {
        output,
        exit_code: 0,
    })
}

fn is_command(args: &[OsString], name: &str) -> bool {
    args.get(1).is_some_and(|arg| arg == name)
        || (args.get(1).is_some_and(|arg| arg == "--root")
            && args.get(3).is_some_and(|arg| arg == name))
}

/// Argv without the program name, as UTF-8.
fn command_args(args: Vec<OsString>) -> Result<Vec<String>, CoordError> {
    let mut args = args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| CoordError::new("INVALID_ARGUMENT", "arguments must be valid UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !args.is_empty() {
        args.remove(0);
    }
    Ok(args)
}

fn execute_check(
    args: Vec<OsString>,
    current_dir: Result<PathBuf, std::io::Error>,
) -> Result<CliOutcome, CoordError> {
    let mut args = command_args(args)?;
    let explicit_root = remove_root(&mut args)?;
    let current_dir = current_dir.map_err(CoordError::io)?;
    let hub = crate::doctor::discover_hub(&current_dir, explicit_root.as_deref())?;
    if args.get(1).map(String::as_str) == Some("scorecard") {
        let report = crate::scorecard::evaluate(&hub)?;
        let output = if args.iter().any(|arg| arg == "--json") {
            serde_json::to_string_pretty(&report).map_err(CoordError::json)?
        } else {
            crate::scorecard::render_markdown(&report)
        };
        return Ok(CliOutcome {
            output,
            exit_code: 0,
        });
    }
    let execution = crate::check::run(&hub, &args[1..])?;
    Ok(CliOutcome {
        output: execution.output()?,
        exit_code: execution.exit_code(),
    })
}

fn execute_doctor(
    args: Vec<OsString>,
    current_dir: Result<PathBuf, std::io::Error>,
) -> Result<CliOutcome, CoordError> {
    let mut args = command_args(args)?;
    let explicit_root = remove_root(&mut args)?;
    let current_dir = current_dir.map_err(CoordError::io)?;
    let execution = crate::doctor::execute(&current_dir, explicit_root.as_deref(), &args[1..])?;
    Ok(CliOutcome {
        output: execution.output().to_string(),
        exit_code: execution.exit_code(),
    })
}

pub fn run(
    args: impl IntoIterator<Item = OsString>,
    current_dir: Result<PathBuf, std::io::Error>,
) -> Result<String, CoordError> {
    let mut args = command_args(args.into_iter().collect())?;
    let explicit_root = remove_root(&mut args)?;
    let current_dir = current_dir.map_err(CoordError::io)?;
    if args.first().is_some_and(|arg| arg == "setup") {
        return crate::setup::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "release") {
        if explicit_root.is_some() {
            return Err(CoordError::new(
                "USAGE",
                "release build, verification, and extraction take explicit absolute paths, never --root",
            ));
        }
        return crate::release::run(&args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "checkout") {
        return crate::checkout::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "doctor") {
        return crate::doctor::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "hub") {
        return crate::hub_check::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "deps") {
        return crate::deps_check::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    if args.first().is_some_and(|arg| arg == "fuse") {
        return crate::fuse::run(&current_dir, explicit_root.as_deref(), &args[1..]);
    }
    let root = discover_family_root(&current_dir, explicit_root.map(OsString::from))?;
    if args.first().is_some_and(|arg| arg == "lock") {
        return crate::family_lock::run(&root, &args[1..]);
    }
    if args.len() < 2 || args[0] != "coord" {
        return Err(CoordError::new("USAGE", USAGE));
    }
    let action = args[1].clone();
    let options = Options::parse(&args[2..])?;
    let store = CoordStore::new(root);
    let now = unix_millis()?;

    match action.as_str() {
        "claim" => claim(&store, &options, now),
        "heartbeat" => heartbeat(&store, &options, now),
        "handoff" => handoff(&store, &options, now),
        "receipt" => receipt(&store, &options, now),
        "receipt-group" => receipt_group(&store, &options, now),
        "correct-receipt" => correct_receipt(&store, &options, now),
        "correct-receipt-group" => correct_receipt_group(&store, &options, now),
        "status" => status(&store, &options, now),
        _ => Err(CoordError::new("USAGE", USAGE)),
    }
}

fn claim(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["agent", "lane", "repo", "path", "ttl-seconds"])?;
    let claim = store.claim(
        &ClaimInput {
            agent: options.one("agent")?,
            lane: options.one("lane")?,
            repo: options.one("repo")?,
            paths: options.many("path")?,
            ttl_seconds: options.u64_or("ttl-seconds", DEFAULT_TTL_SECONDS)?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claim).map_err(CoordError::json)
}

fn heartbeat(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["claim", "agent", "ttl-seconds", "note"])?;
    let claim = store.heartbeat(
        &HeartbeatInput {
            claim_id: options.one("claim")?,
            agent: options.one("agent")?,
            ttl_seconds: options.u64_or("ttl-seconds", DEFAULT_TTL_SECONDS)?,
            note: options.optional_one("note")?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claim).map_err(CoordError::json)
}

fn handoff(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["claim", "agent", "proof", "exit-code", "changed-path"])?;
    let claim = store.handoff(
        &HandoffInput {
            claim_id: options.one("claim")?,
            agent: options.one("agent")?,
            proof_command: options.one("proof")?,
            proof_exit_code: options.i32_or("exit-code", 0)?,
            changed_paths: options.many("changed-path")?,
            commit_oid: None,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claim).map_err(CoordError::json)
}

fn receipt(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["claim", "orchestrator", "commit", "committed-path"])?;
    let claim = store.receipt(
        &CommitReceiptInput {
            claim_id: options.one("claim")?,
            orchestrator: options.one("orchestrator")?,
            commit_oid: options.one("commit")?,
            committed_paths: options.many("committed-path")?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claim).map_err(CoordError::json)
}

fn receipt_group(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&["claim", "orchestrator", "commit"])?;
    let claims = store.receipt_group(
        &CommitReceiptGroupInput {
            claim_ids: options.many("claim")?,
            orchestrator: options.one("orchestrator")?,
            commit_oid: options.one("commit")?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claims).map_err(CoordError::json)
}

fn correct_receipt(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "claim",
        "orchestrator",
        "previous-commit",
        "commit",
        "committed-path",
        "reason",
    ])?;
    let claim = store.correct_receipt(
        &ReceiptCorrectionInput {
            claim_id: options.one("claim")?,
            orchestrator: options.one("orchestrator")?,
            previous_commit_oid: options.one("previous-commit")?,
            commit_oid: options.one("commit")?,
            committed_paths: options.many("committed-path")?,
            reason: options.one("reason")?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claim).map_err(CoordError::json)
}

fn correct_receipt_group(
    store: &CoordStore,
    options: &Options,
    now: u64,
) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "claim",
        "orchestrator",
        "previous-commit",
        "commit",
        "reason",
    ])?;
    let claims = store.correct_receipt_group(
        &GroupReceiptCorrectionInput {
            claim_ids: options.many("claim")?,
            orchestrator: options.one("orchestrator")?,
            previous_commit_oid: options.one("previous-commit")?,
            commit_oid: options.one("commit")?,
            reason: options.one("reason")?,
        },
        now,
    )?;
    serde_json::to_string_pretty(&claims).map_err(CoordError::json)
}

fn status(store: &CoordStore, options: &Options, now: u64) -> Result<String, CoordError> {
    options.reject_values()?;
    options.reject_unknown_flags(&["json", "all"])?;
    let include_all = options.flag("all");
    let mut status = store.status(now)?;
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

fn remove_root(args: &mut Vec<String>) -> Result<Option<String>, CoordError> {
    if args.first().is_none_or(|value| value != "--root") {
        return Ok(None);
    }
    if args.len() < 2 {
        return Err(CoordError::new("MISSING_VALUE", "--root needs a path"));
    }
    let value = args.remove(1);
    args.remove(0);
    Ok(Some(value))
}

#[derive(Default)]
struct Options {
    values: BTreeMap<String, Vec<String>>,
    flags: Vec<String>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, CoordError> {
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

    fn one(&self, name: &str) -> Result<String, CoordError> {
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

    fn optional_one(&self, name: &str) -> Result<Option<String>, CoordError> {
        match self.values.get(name) {
            None => Ok(None),
            Some(values) if values.len() == 1 => Ok(Some(values[0].clone())),
            Some(_) => Err(CoordError::new(
                "DUPLICATE_OPTION",
                format!("--{name} must appear at most once"),
            )),
        }
    }

    fn many(&self, name: &str) -> Result<Vec<String>, CoordError> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| CoordError::new("MISSING_OPTION", format!("--{name} is required")))
    }

    fn u64_or(&self, name: &str, default: u64) -> Result<u64, CoordError> {
        let Some(value) = self.optional_one(name)? else {
            return Ok(default);
        };
        parse_ascii_u64(&value).ok_or_else(|| {
            CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value"))
        })
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

    fn reject_flags(&self) -> Result<(), CoordError> {
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

    fn reject_unknown_values(&self, allowed: &[&str]) -> Result<(), CoordError> {
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
