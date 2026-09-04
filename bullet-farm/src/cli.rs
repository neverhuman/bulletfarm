mod coord;

#[cfg(test)]
pub(crate) use coord::test_recovery_action;

use std::{ffi::OsString, path::PathBuf};

use crate::coord::{CoordError, discover_family_root};

const USAGE: &str = "usage: bullet-family [--root PATH] <doctor --json|setup --root PATH --source jeryu --cargo-bin ABSOLUTE_PATH --node-bin ABSOLUTE_PATH --npm-cli ABSOLUTE_PATH [--offline]|release <build|verify|extract|receipt-verify> [options]|checkout verify|hub check|deps check|lock <generate --tag VERSION --subjects ABSOLUTE_PATH|verify --tag VERSION>|fuse --source <local|lock>|check <fast|required|release|scorecard|dogfood> [options]|coord <init|claim|heartbeat|handoff|receipt|receipt-group|correct-receipt|correct-receipt-group|recovery-inspect|recovery-provenance|recovery-build-observe|recovery-authorization-draft|recovery-authorization-message|recovery-authorization-signature-import|recovery-manifest|recover-rollover|recovery-plan|recovery-proof|recovery-review|recovery-request|adopt|status> [options]>";

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
        // A non-authoritative diagnostic is Neutral in the shared exit
        // vocabulary (check::model): it must not exit 0 as though its numbers
        // were authority. This is the same fail-closed rule ADR 0015 applied
        // to `check dogfood`.
        return Ok(CliOutcome {
            output,
            exit_code: u8::from(!report.authoritative),
        });
    }
    if args.get(1).map(String::as_str) == Some("dogfood") {
        let rest = &args[2..];
        if !rest.is_empty() && rest != ["--json"] {
            return Err(CoordError::new("USAGE", USAGE));
        }
        let (mut output, exit_code) = crate::check::dogfood_board(&hub)?;
        if !output.ends_with('\n') {
            output.push('\n');
        }
        return Ok(CliOutcome { output, exit_code });
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
    if args.first().is_some_and(|arg| arg == "coord")
        && args
            .get(1)
            .is_some_and(|arg| arg == "recovery-build-observe")
    {
        if explicit_root.is_some() {
            return Err(CoordError::new(
                "UNKNOWN_OPTION",
                "recovery-build-observe accepts only its ten absolute artifact paths",
            ));
        }
        let options = coord::Options::parse(&args[2..])?;
        return coord::recovery::build_observe(&options);
    }
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
    if args.first().is_some_and(|arg| arg == "coord") {
        return coord::run(root, &args[1..], USAGE);
    }
    Err(CoordError::new("USAGE", USAGE))
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

#[cfg(test)]
mod tests {
    use super::run;
    use std::{ffi::OsString, io, path::PathBuf};

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn recovery_build_observe_bypasses_cwd_and_refuses_global_root() {
        let no_cwd = Err(io::Error::new(io::ErrorKind::NotFound, "test cwd absent"));
        let missing = run(
            args(&["bullet-family", "coord", "recovery-build-observe"]),
            no_cwd,
        )
        .unwrap_err();
        assert_eq!(missing.code(), "MISSING_OPTION");

        let rooted = run(
            args(&[
                "bullet-family",
                "--root",
                "/unused",
                "coord",
                "recovery-build-observe",
            ]),
            Ok(PathBuf::from("/also-unused")),
        )
        .unwrap_err();
        assert_eq!(rooted.code(), "UNKNOWN_OPTION");
    }
}
