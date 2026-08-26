use std::{collections::BTreeSet, fs, path::Path};

use toml::Value;

const NON_AUTHORITY: &str = "; unsigned component observation only; no release authority";

struct ExpectedLane {
    name: &'static str,
    command: &'static str,
    purpose: &'static str,
    requires_network: bool,
}

struct ExpectedDispatch {
    label: &'static str,
    lane: &'static str,
    script: &'static str,
    artifacts: &'static [&'static str],
}

const EXPECTED: &[ExpectedLane] = &[
    lane(
        "source-scan",
        "bash scripts/ci-local.sh source-scan",
        "current-tree source and lockfile secret admission before dependency installation",
        false,
    ),
    lane(
        "fast",
        "just fast",
        "deterministic Hub metadata, setup refusal, and exact bullet-family test partition",
        false,
    ),
    lane(
        "lint",
        "just lint",
        "Rust format/Clippy, actionlint, ShellCheck, and CI negative meta-controls",
        false,
    ),
    lane(
        "required",
        "just check",
        "five atomic lanes sequentially and exactly once",
        true,
    ),
    lane(
        "contract-drift",
        "just contract-check",
        "canonical policy, schema, generated binding, and fixture drift proof",
        false,
    ),
    lane(
        "contract",
        "just contract",
        "contract drift plus exactly two pinned bounded formal models",
        true,
    ),
    lane(
        "security",
        "just security",
        "required secret, dependency, and workflow scans",
        true,
    ),
    lane(
        "docs",
        "just docs",
        "rustdoc, relative links, release truth, and reproducible README media",
        false,
    ),
    lane(
        "readme-media",
        "just readme-check",
        "schema, claims, redaction, hashes, limits, frame identity, and deterministic double rendering",
        false,
    ),
    lane(
        "audit",
        "just audit",
        "local Jankurai full-score ratchet with JSON, Markdown, and repair artifacts",
        false,
    ),
    lane(
        "family",
        "just family",
        "clean dependency-ordered BulletGit, Kernel, Portal, and Hub component proof",
        true,
    ),
    lane(
        "family-contract",
        "just family-contract",
        "compatibility alias for the same dependency-ordered family proof without duplicate Hub contract execution",
        true,
    ),
    lane(
        "history",
        "bash scripts/ci-local.sh history",
        "scheduled full-history secret scan over a non-shallow checkout",
        false,
    ),
    lane(
        "links",
        "bash scripts/ci-local.sh links",
        "scheduled external-link reachability diagnostic with a nonempty inventory",
        true,
    ),
    lane(
        "advisory",
        "bash scripts/ci-local.sh advisory",
        "scheduled RustSec refresh, freshness admission, and advisory-only dependency scan",
        true,
    ),
    lane(
        "coverage",
        "bash scripts/ci-local.sh coverage",
        "scheduled full-workspace coverage with sanitized repository-relative Cobertura output",
        false,
    ),
    lane(
        "platform",
        "bash scripts/ci-local.sh platform",
        "scheduled macOS and Windows compile plus typed unsupported-mutation refusal",
        false,
    ),
    lane(
        "toolchain-pinned",
        "just toolchain-pinned",
        "operator-local Rust 1.97.1 build and test observation",
        false,
    ),
];

const DISPATCH: &[ExpectedDispatch] = &[
    dispatch("source-scan", "source-scan", "ops/ci/source-scan.sh", &[]),
    dispatch(
        "fast",
        "fast",
        "ops/ci/fast.sh",
        &[".ci-artifacts/junit/fast.xml"],
    ),
    dispatch("lint", "lint", "ops/ci/lint.sh", &[]),
    dispatch(
        "contract",
        "contract",
        "ops/ci/contract.sh",
        &[
            ".ci-artifacts/junit/contract.xml",
            ".ci-artifacts/formal/contract.json",
            ".ci-artifacts/formal/contract.log",
            ".ci-artifacts/contracts/bundle-manifest.json",
        ],
    ),
    dispatch("security", "security", "ops/ci/security.sh", &[]),
    dispatch("docs", "docs", "ops/ci/docs.sh", &[]),
    dispatch(
        "required",
        "required",
        "ops/ci/required.sh",
        &[
            ".ci-artifacts/junit/fast.xml",
            ".ci-artifacts/junit/contract.xml",
            ".ci-artifacts/formal/contract.json",
            ".ci-artifacts/formal/contract.log",
            ".ci-artifacts/contracts/bundle-manifest.json",
        ],
    ),
    dispatch(
        "family",
        "family",
        "ops/ci/family.sh",
        &[".ci-artifacts/family/subjects.json"],
    ),
    dispatch(
        "family-contract",
        "family-contract",
        "ops/ci/family-contract.sh",
        &[".ci-artifacts/family/subjects.json"],
    ),
    dispatch("history", "history", "ops/ci/history.sh", &[]),
    dispatch("links", "links", "ops/ci/external-links.sh", &[]),
    dispatch("advisory", "advisory", "ops/ci/advisory.sh", &[]),
    dispatch(
        "coverage",
        "coverage",
        "ops/ci/coverage.sh",
        &[".ci-artifacts/coverage/cobertura.xml"],
    ),
    dispatch("platform", "platform", "ops/ci/platform-refusal.sh", &[]),
    dispatch("audit", "audit", "ops/ci/audit.sh", &[]),
    dispatch(
        "toolchain-pinned",
        "toolchain-pinned",
        "ops/ci/toolchain-pinned.sh",
        &[],
    ),
    dispatch(
        "all",
        "required",
        "ops/ci/required.sh",
        &[
            ".ci-artifacts/junit/fast.xml",
            ".ci-artifacts/junit/contract.xml",
            ".ci-artifacts/formal/contract.json",
            ".ci-artifacts/formal/contract.log",
            ".ci-artifacts/contracts/bundle-manifest.json",
        ],
    ),
];

const fn lane(
    name: &'static str,
    command: &'static str,
    purpose: &'static str,
    requires_network: bool,
) -> ExpectedLane {
    ExpectedLane {
        name,
        command,
        purpose,
        requires_network,
    }
}

const fn dispatch(
    label: &'static str,
    lane: &'static str,
    script: &'static str,
    artifacts: &'static [&'static str],
) -> ExpectedDispatch {
    ExpectedDispatch {
        label,
        lane,
        script,
        artifacts,
    }
}

pub(super) fn assert_registry(root: &Path) {
    let registry = read(root, "agent/proof-lanes.toml");
    let dispatcher = read(root, "scripts/ci-local.sh");
    let justfile = read(root, "Justfile");
    validate(root, &registry, &dispatcher, &justfile).expect("strict proof-lane registry");

    let duplicate = registry.replacen("name = \"source-scan\"", "name = \"fast\"", 1);
    assert!(validate(root, &duplicate, &dispatcher, &justfile).is_err());
    let unknown = registry.replacen(
        "name = \"source-scan\"",
        "name = \"source-scan\"\nrelease_authority = true",
        1,
    );
    assert!(validate(root, &unknown, &dispatcher, &justfile).is_err());
    let command_shadow = registry.replacen(
        "command = \"bash scripts/ci-local.sh source-scan\"",
        "command = \"true\"",
        1,
    );
    assert!(validate(root, &command_shadow, &dispatcher, &justfile).is_err());
    let authority_suffix = registry.replacen(NON_AUTHORITY, "; release authority", 1);
    assert!(validate(root, &authority_suffix, &dispatcher, &justfile).is_err());
    let network_lie = registry.replacen(
        "name = \"contract\"\ncommand = \"just contract\"\npurpose = \"contract drift plus exactly two pinned bounded formal models; unsigned component observation only; no release authority\"\nrequires_network = true",
        "name = \"contract\"\ncommand = \"just contract\"\npurpose = \"contract drift plus exactly two pinned bounded formal models; unsigned component observation only; no release authority\"\nrequires_network = false",
        1,
    );
    assert!(validate(root, &network_lie, &dispatcher, &justfile).is_err());
    let hidden_alias = dispatcher.replacen("  all) run_observed", "  gates|all) run_observed", 1);
    assert!(validate(root, &registry, &hidden_alias, &justfile).is_err());
    let missing_alias = dispatcher.replacen("  all) run_observed", "  required) run_observed", 1);
    assert!(validate(root, &registry, &missing_alias, &justfile).is_err());
    let action_swap = dispatcher.replacen(
        "  all) run_observed required ops/ci/required.sh \\",
        "  all) run_observed audit ops/ci/audit.sh ;;",
        1,
    );
    assert!(validate(root, &registry, &action_swap, &justfile).is_err());
    let missing_target =
        dispatcher.replacen("ops/ci/source-scan.sh", "ops/ci/missing-source-scan.sh", 1);
    assert!(validate(root, &registry, &missing_target, &justfile).is_err());
    let recipe_swap = justfile.replacen(
        "fast:\n    bash scripts/ci-local.sh fast",
        "fast:\n    bash scripts/ci-local.sh security",
        1,
    );
    assert!(validate(root, &registry, &dispatcher, &recipe_swap).is_err());
}

fn validate(root: &Path, registry: &str, dispatcher: &str, justfile: &str) -> Result<(), String> {
    let document: Value = toml::from_str(registry).map_err(|error| error.to_string())?;
    let records = document
        .get("lane")
        .and_then(Value::as_array)
        .ok_or_else(|| "lane array is absent".to_owned())?;
    if records.len() != EXPECTED.len() {
        return Err("lane record count differs".to_owned());
    }
    let mut names = BTreeSet::new();
    let fields = BTreeSet::from(["command", "name", "purpose", "requires_network"]);
    for (record, expected) in records.iter().zip(EXPECTED) {
        let table = record
            .as_table()
            .ok_or_else(|| "lane record is not a table".to_owned())?;
        if table.keys().map(String::as_str).collect::<BTreeSet<_>>() != fields {
            return Err(format!("lane {} fields differ", expected.name));
        }
        let name = string_field(table, "name")?;
        if !names.insert(name) || name != expected.name {
            return Err(format!("lane {} name/order differs", expected.name));
        }
        if string_field(table, "command")? != expected.command {
            return Err(format!("lane {} command differs", expected.name));
        }
        let exact_purpose = format!("{}{}", expected.purpose, NON_AUTHORITY);
        if string_field(table, "purpose")? != exact_purpose {
            return Err(format!("lane {} purpose differs", expected.name));
        }
        if table.get("requires_network").and_then(Value::as_bool) != Some(expected.requires_network)
        {
            return Err(format!(
                "lane {} network classification differs",
                expected.name
            ));
        }
        validate_entrypoint(root, expected.command, justfile)?;
    }

    let actual_dispatcher = dispatcher_rows(dispatcher)?;
    if actual_dispatcher.len() != DISPATCH.len() {
        return Err("dispatcher row count differs".to_owned());
    }
    for (actual, expected) in actual_dispatcher.iter().zip(DISPATCH) {
        if actual.label != expected.label
            || actual.lane != expected.lane
            || actual.script != expected.script
            || actual
                .artifacts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != expected.artifacts
        {
            return Err(format!("dispatcher row {} differs", expected.label));
        }
        regular_non_symlink(root, expected.script)?;
    }
    Ok(())
}

fn string_field<'a>(table: &'a toml::Table, field: &str) -> Result<&'a str, String> {
    table
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("lane {field} is not a string"))
}

fn validate_entrypoint(root: &Path, command: &str, justfile: &str) -> Result<(), String> {
    if let Some(lane) = command.strip_prefix("bash scripts/ci-local.sh ") {
        if lane.is_empty() || lane.contains(char::is_whitespace) {
            return Err("dispatcher command has an invalid lane".to_owned());
        }
        return regular_non_symlink(root, "scripts/ci-local.sh");
    }
    let recipe = command
        .strip_prefix("just ")
        .ok_or_else(|| "unsupported proof command".to_owned())?;
    let expected =
        just_route(recipe).ok_or_else(|| format!("Just recipe {recipe} is not admitted"))?;
    validate_just_recipe(justfile, recipe, expected)?;
    regular_non_symlink(root, "Justfile")?;
    if let Some(script) = expected.strip_prefix("bash ") {
        let script = script
            .split_whitespace()
            .next()
            .ok_or_else(|| format!("Just recipe {recipe} has no script"))?;
        regular_non_symlink(root, script)?;
    }
    Ok(())
}

fn just_route(recipe: &str) -> Option<&'static str> {
    match recipe {
        "fast" => Some("bash scripts/ci-local.sh fast"),
        "lint" => Some("bash scripts/ci-local.sh lint"),
        "check" => Some("bash scripts/ci-local.sh required"),
        "contract-check" => Some(
            "cargo run --locked --quiet -p bullet-wire --bin bullet-contract -- check --root .",
        ),
        "contract" => Some("bash scripts/ci-local.sh contract"),
        "security" => Some("bash scripts/ci-local.sh security"),
        "docs" => Some("bash scripts/ci-local.sh docs"),
        "readme-check" => Some("bash scripts/readme-check.sh"),
        "audit" => Some("bash scripts/ci-local.sh audit"),
        "family" => Some("bash scripts/ci-local.sh family"),
        "family-contract" => Some("bash scripts/ci-local.sh family-contract"),
        "toolchain-pinned" => Some("bash scripts/ci-local.sh toolchain-pinned"),
        _ => None,
    }
}

fn validate_just_recipe(justfile: &str, recipe: &str, expected: &str) -> Result<(), String> {
    let header = format!("{recipe}:");
    let lines = justfile.lines().collect::<Vec<_>>();
    let positions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == header).then_some(index))
        .collect::<Vec<_>>();
    if positions.len() != 1 {
        return Err(format!("Just recipe {recipe} count differs"));
    }
    let position = positions[0];
    if lines.get(position + 1).copied() != Some(&format!("    {expected}")) {
        return Err(format!("Just recipe {recipe} body differs"));
    }
    if lines
        .get(position + 2)
        .is_some_and(|line| line.starts_with(char::is_whitespace))
    {
        return Err(format!("Just recipe {recipe} has an extra body line"));
    }
    Ok(())
}

fn regular_non_symlink(root: &Path, relative: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root.join(relative)).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!("{relative} is not a regular non-symlink file"));
    }
    Ok(())
}

struct DispatchRow {
    label: String,
    lane: String,
    script: String,
    artifacts: Vec<String>,
}

fn dispatcher_rows(source: &str) -> Result<Vec<DispatchRow>, String> {
    let body = source
        .split_once("case \"$lane\" in")
        .and_then(|(_, tail)| tail.split_once("\n  *)"))
        .map(|(body, _)| body)
        .ok_or_else(|| "dispatcher case is malformed".to_owned())?;
    let logical = body.replace("\\\n", " ");
    let mut rows = Vec::new();
    for line in logical
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Some((labels, action)) = line.split_once(") ") else {
            return Err("dispatcher row is malformed".to_owned());
        };
        if labels.contains('|')
            || labels.is_empty()
            || !labels
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        {
            return Err("dispatcher contains an invalid or hidden alias".to_owned());
        }
        let action = action
            .strip_suffix(";;")
            .ok_or_else(|| format!("dispatcher row {labels} has no terminator"))?
            .trim();
        let fields = action.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 3 || fields[0] != "run_observed" {
            return Err(format!("dispatcher row {labels} action differs"));
        }
        rows.push(DispatchRow {
            label: labels.to_owned(),
            lane: fields[1].to_owned(),
            script: fields[2].to_owned(),
            artifacts: fields[3..]
                .iter()
                .map(|field| (*field).to_owned())
                .collect(),
        });
    }
    Ok(rows)
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}
