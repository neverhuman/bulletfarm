//! Localhost-first forge verbs. Live install/supervise is not implemented.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::coord::CoordError;

/// Process-facing forge result. Kept separate so this module does not
/// depend on [`crate::cli`] while that file is claimed elsewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForgeOutcome {
    output: String,
    exit_code: u8,
}

impl ForgeOutcome {
    /// Operator-visible text.
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Process exit code.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }
}

const LOCAL_BANNER: &str = "\
forge: local (Jeryu at http://127.0.0.1:8787)

  What this unlocks that a hosted forge cannot:
    exact expected-old-OID compare-and-swap on every candidate push
    protected-ref rules we define, including required proof roots
    no rate limits, no third-party credential, no network egress
    offline determinism: a benchmark corpus replays identically forever
    sub-millisecond read-back

  What it does NOT give you, honestly:
    no merge-queue / merge-group composition   (Jeryu has none today)
    no signed release artifacts to pin against (Jeryu tags are unsigned)
    the forge runs on your machine, so it is not an independent authority

  V1 GA requires BOTH a Jeryu effect receipt AND a GitHub App effect receipt.
  Choosing local does not remove the GitHub requirement.
";

const GITHUB_BANNER: &str = "\
forge: github  [profile github-adapter-v1 — NOT self-hosted-v1]

  Selecting github alone gives up, permanently:
    exact-OID CAS      -> approximated by client-side --force-with-lease
    proof-root binding -> a check name and free text; GitHub cannot enforce it
    offline replay     -> unavailable
    zero credentials   -> a GitHub App installation token must exist

  You gain: real merge-queue composition, signed artifacts, third-party sovereignty.
  This profile cannot inherit self-hosted-v1.
";

const GITLAB_BANNER: &str = "\
forge: gitlab
UNSUPPORTED_BY_ADAPTER: gitlab-adapter-v1 is not implemented
";

const PROBE_JSON: &str = r#"{
  "local": {
    "exact_oid_cas": "supported",
    "protected_refs": "supported",
    "check_runs": "supported_with_limitations",
    "merge_group": "unsupported",
    "exact_oid_readback": "supported",
    "third_party_credential": "unsupported",
    "live": "unprobed"
  },
  "github": {
    "exact_oid_cas": "supported_with_limitations",
    "merge_group": "supported_with_limitations",
    "live": "unprobed"
  },
  "gitlab": {
    "all": "unsupported"
  }
}"#;

/// Whether argv should be handled here instead of [`crate::cli`].
#[must_use]
pub fn should_intercept(args: &[OsString]) -> bool {
    command_name(args).as_deref() == Some("forge")
}

/// Banner for `setup --forge <profile>`. `None` if setup has no `--forge`.
#[must_use]
pub fn setup_forge_banner(args: &[OsString]) -> Option<String> {
    if command_name(args).as_deref() != Some("setup") {
        return None;
    }
    let profile = named_value(args, "--forge")?;
    Some(banner(&profile))
}

/// `setup --forge` with no other setup mutation flags.
#[must_use]
pub fn setup_forge_only(args: &[OsString]) -> bool {
    if command_name(args).as_deref() != Some("setup") || named_value(args, "--forge").is_none() {
        return false;
    }
    !args.iter().any(|arg| {
        matches!(
            arg.to_str(),
            Some("--source" | "--cargo-bin" | "--node-bin" | "--npm-cli" | "--offline")
        )
    })
}

/// Run `forge probe|pin|status`.
///
/// # Errors
///
/// Unknown verb, unsigned tag, or GitLab.
pub fn execute(
    args: Vec<OsString>,
    _current_dir: Result<PathBuf, std::io::Error>,
) -> Result<ForgeOutcome, CoordError> {
    let mut rest = command_args(args)?;
    if rest.first().is_some_and(|arg| arg == "forge") {
        rest.remove(0);
    }
    let verb = rest.first().map(String::as_str).unwrap_or("status");
    match verb {
        "probe" => {
            if named_value_from_strings(&rest, "--url").is_some() {
                return Err(CoordError::new(
                    "CAPABILITY_UNPROBED",
                    "live forge probe is not admitted; unsigned tags and no HTTP client",
                ));
            }
            Ok(outcome(PROBE_JSON, 0))
        }
        "pin" => Err(CoordError::new(
            "UNSIGNED_FORGE_TAG",
            "Jeryu tags are not annotated signed tags; pin refuses until they are",
        )),
        "status" => Ok(outcome(
            "pinned: none\nunprobed: local live, github live\nunsupported: gitlab, jeryu merge-group\n",
            0,
        )),
        other => Err(CoordError::new(
            "INVALID_ARGUMENT",
            format!("unknown forge verb {other}; expected probe|pin|status"),
        )),
    }
}

fn banner(profile: &str) -> String {
    match profile {
        "local" | "localhost" | "jeryu" => LOCAL_BANNER.to_string(),
        "github" => GITHUB_BANNER.to_string(),
        "gitlab" => GITLAB_BANNER.to_string(),
        other => format!("UNSUPPORTED_BY_ADAPTER: unknown forge profile {other}\n"),
    }
}

fn outcome(output: &str, exit_code: u8) -> ForgeOutcome {
    ForgeOutcome {
        output: output.to_string(),
        exit_code,
    }
}

fn command_name(args: &[OsString]) -> Option<String> {
    let mut i = 1;
    while i < args.len() {
        match args[i].to_str() {
            Some("--root") => i += 2,
            Some(arg) if arg.starts_with('-') => i += 1,
            Some(arg) => return Some(arg.to_string()),
            None => i += 1,
        }
    }
    None
}

fn named_value(args: &[OsString], name: &str) -> Option<String> {
    let mut i = 0;
    while i + 1 < args.len() {
        if args[i].to_str() == Some(name) {
            return args[i + 1].to_str().map(ToString::to_string);
        }
        i += 1;
    }
    None
}

fn named_value_from_strings(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| pair[1].clone()))
}

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
