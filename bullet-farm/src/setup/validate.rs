//! Fallible dependency and generated-contract checks over an unpublished candidate family.

use std::{fs, path::Path};

use super::{
    command::{SetupEnvironment, Toolchain},
    transaction::{CandidateFamily, CandidateValidator},
};
use crate::coord::CoordError;

const MAX_CONTRACT_BYTES: u64 = 16 * 1024 * 1024;
const CONTRACT_LINKS: &[(&str, &str, &str, &str)] = &[
    (
        "bullet-farm",
        "contracts/generated/rust/schema_bundle.rs",
        "bullet-kernel",
        "crates/domain/src/schema_bundle.rs",
    ),
    (
        "bullet-farm",
        "policy/v1alpha1/policy.json",
        "bullet-kernel",
        "crates/application/tests/fixtures/policy-v1alpha1.json",
    ),
    (
        "bullet-farm",
        "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json",
        "bullet-kernel",
        "crates/application/tests/fixtures/policy-v1alpha2-live-enabled.json",
    ),
    (
        "bullet-farm",
        "contracts/generated/rust/schema_bundle.rs",
        "bullet-git",
        "contracts/generated/rust/schema_bundle.rs",
    ),
    (
        "bullet-farm",
        "contracts/generated/typescript/schemaBundle.ts",
        "bullet-portal",
        "src/generated/schemaBundle.ts",
    ),
    (
        "bullet-kernel",
        "contracts/generated/api.ts",
        "bullet-portal",
        "src/generated/api.ts",
    ),
    (
        "bullet-farm",
        "formal/traces/effect-check-ambiguity.json",
        "bullet-kernel",
        "crates/adapters/tests/fixtures/formal/effect-check-ambiguity.json",
    ),
    (
        "bullet-farm",
        "formal/traces/effect-third-party.json",
        "bullet-kernel",
        "crates/adapters/tests/fixtures/formal/effect-third-party.json",
    ),
    (
        "bullet-farm",
        "formal/traces/lease-fence-reclaim.json",
        "bullet-kernel",
        "crates/adapters/tests/fixtures/formal/lease-fence-reclaim.json",
    ),
];

pub(super) struct SetupValidator<'a> {
    offline: bool,
    toolchain: &'a Toolchain,
    environment: &'a SetupEnvironment,
}

impl<'a> SetupValidator<'a> {
    pub(super) const fn new(
        offline: bool,
        toolchain: &'a Toolchain,
        environment: &'a SetupEnvironment,
    ) -> Self {
        Self {
            offline,
            toolchain,
            environment,
        }
    }
}

impl CandidateValidator for SetupValidator<'_> {
    fn validate(&self, candidate: &CandidateFamily<'_>) -> Result<(), CoordError> {
        for name in ["bullet-farm", "bullet-kernel", "bullet-git"] {
            let mut args = vec!["fetch", "--locked"];
            if self.offline {
                args.push("--offline");
            }
            self.toolchain
                .run_cargo(candidate.path(name)?, &args, self.environment)?;
        }
        let mut npm_args = vec!["ci", "--ignore-scripts", "--no-audit", "--no-fund"];
        if self.offline {
            npm_args.push("--offline");
        }
        self.toolchain.run_npm(
            candidate.path("bullet-portal")?,
            &npm_args,
            self.environment,
        )?;
        self.toolchain.run_cargo(
            candidate.path("bullet-farm")?,
            &[
                "run",
                "--locked",
                "--quiet",
                "-p",
                "bullet-wire",
                "--bin",
                "bullet-contract",
                "--",
                "check",
                "--root",
                ".",
            ],
            self.environment,
        )?;
        self.toolchain.run_cargo(
            candidate.path("bullet-kernel")?,
            &[
                "run",
                "--locked",
                "--quiet",
                "-p",
                "bullet",
                "--",
                "contracts",
                "check",
            ],
            self.environment,
        )?;
        self.toolchain.run_bash(
            candidate.path("bullet-farm")?,
            &["-n", "scripts/sync-family-contracts.sh"],
            self.environment,
        )?;
        validate_contract_links(candidate)
    }
}

fn validate_contract_links(candidate: &CandidateFamily<'_>) -> Result<(), CoordError> {
    for &(source_member, source, destination_member, destination) in CONTRACT_LINKS {
        let expected = read_bounded(&candidate.path(source_member)?.join(source))?;
        let actual = read_bounded(&candidate.path(destination_member)?.join(destination))?;
        if actual != expected {
            return Err(CoordError::new(
                "GENERATED_CONTRACT_DRIFT",
                format!("{destination_member}/{destination} differs from {source_member}/{source}"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn assert_synchronizer_link_completeness_for_test() {
    let policy_links = [
        (
            "bullet-farm",
            "policy/v1alpha1/policy.json",
            "bullet-kernel",
            "crates/application/tests/fixtures/policy-v1alpha1.json",
        ),
        (
            "bullet-farm",
            "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json",
            "bullet-kernel",
            "crates/application/tests/fixtures/policy-v1alpha2-live-enabled.json",
        ),
    ];
    assert_eq!(
        CONTRACT_LINKS.len(),
        9,
        "unexpected synchronizer link count"
    );
    for link in policy_links {
        assert!(
            CONTRACT_LINKS.contains(&link),
            "setup validation omitted synchronized policy link {link:?}"
        );
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err(CoordError::new(
            "INVALID_GENERATED_CONTRACT",
            format!(
                "{} is not an admitted regular contract file",
                path.display()
            ),
        ));
    }
    fs::read(path).map_err(CoordError::io)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, path::Path};

    use super::CONTRACT_LINKS;

    type Link = (String, String, String, String);

    const SYNC_SCRIPT: &str = "scripts/sync-family-contracts.sh";

    /// The validator and the committed synchronizer must name exactly the same
    /// (source, destination) pairs; the script is read as text, never executed.
    #[test]
    fn contract_links_equal_synchronizer_script_destinations() {
        let script = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(SYNC_SCRIPT))
            .expect("committed synchronizer script");
        let written: BTreeSet<Link> = parse_sync_links(&script).into_iter().collect();
        let validated: BTreeSet<Link> = CONTRACT_LINKS
            .iter()
            .map(
                |&(source_member, source, destination_member, destination)| {
                    (
                        source_member.to_owned(),
                        source.to_owned(),
                        destination_member.to_owned(),
                        destination.to_owned(),
                    )
                },
            )
            .collect();
        for link in &validated {
            assert!(
                written.contains(link),
                "setup validates {link:?}, which {SYNC_SCRIPT} never writes"
            );
        }
        for link in &written {
            assert!(
                validated.contains(link),
                "{SYNC_SCRIPT} writes {link:?}, which setup never validates"
            );
        }
        assert_eq!(
            validated.len(),
            CONTRACT_LINKS.len(),
            "duplicate contract link"
        );
    }

    /// Textual reading of the synchronizer: joins backslash continuations, expands
    /// `$HUB`/`$FAMILY` roots and one single-variable `for … in …; do` loop, and
    /// refuses any construct it does not understand rather than guessing.
    fn parse_sync_links(script: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let mut active_loop: Option<(String, Vec<String>)> = None;
        for logical in script.replace("\\\n", " ").lines() {
            let line = logical.trim();
            if let Some(header) = line.strip_prefix("for ") {
                assert!(active_loop.is_none(), "nested loop is not admitted: {line}");
                let (variable, values) = header
                    .split_once(" in ")
                    .and_then(|(variable, rest)| Some((variable, rest.strip_suffix("; do")?)))
                    .unwrap_or_else(|| panic!("unrecognized loop header: {line}"));
                active_loop = Some((
                    variable.trim().to_owned(),
                    values.split_whitespace().map(str::to_owned).collect(),
                ));
            } else if line == "done" {
                assert!(active_loop.take().is_some(), "`done` outside a loop");
            } else if let Some(arguments) = line.strip_prefix("sync_file ") {
                let arguments: Vec<&str> = arguments.split_whitespace().map(unquote).collect();
                assert_eq!(
                    arguments.len(),
                    2,
                    "sync_file takes source and destination: {line}"
                );
                let (source, destination) = (arguments[0], arguments[1]);
                match &active_loop {
                    None => links.push(link(source, destination)),
                    Some((variable, values)) => {
                        let pattern = format!("${{{variable}}}");
                        for value in values {
                            links.push(link(
                                &source.replace(&pattern, value),
                                &destination.replace(&pattern, value),
                            ));
                        }
                    }
                }
            }
        }
        assert!(active_loop.is_none(), "unterminated loop");
        links
    }

    fn unquote(token: &str) -> &str {
        token
            .strip_prefix('"')
            .and_then(|inner| inner.strip_suffix('"'))
            .unwrap_or_else(|| panic!("sync_file arguments must be double-quoted: {token}"))
    }

    fn link(source: &str, destination: &str) -> Link {
        let (source_member, source) = split_member(source);
        let (destination_member, destination) = split_member(destination);
        (source_member, source, destination_member, destination)
    }

    fn split_member(path: &str) -> (String, String) {
        let (member, relative) = if let Some(relative) = path.strip_prefix("$HUB/") {
            ("bullet-farm", relative)
        } else if let Some(rest) = path.strip_prefix("$FAMILY/") {
            rest.split_once('/')
                .unwrap_or_else(|| panic!("family path names no member: {path}"))
        } else {
            panic!("sync_file path is not rooted at $HUB or $FAMILY: {path}");
        };
        assert!(
            !relative.contains('$') && !relative.is_empty(),
            "unexpanded or empty path: {path}"
        );
        (member.to_owned(), relative.to_owned())
    }
}
