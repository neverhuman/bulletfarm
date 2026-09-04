use std::{fs, path::Path, process::Command};

use bullet_wire::decode_unique_value;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

#[path = "assurance_controls/proof_lanes.rs"]
mod proof_lanes;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn parse_toml(path: &str) -> TomlValue {
    toml::from_str(&read(path)).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn parse_json(path: &str) -> JsonValue {
    let bytes = fs::read(root().join(path)).unwrap_or_else(|error| panic!("read {path}: {error}"));
    decode_unique_value(&bytes).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

#[test]
fn control_manifests_route_to_real_local_proof() {
    proof_lanes::assert_registry(root());
    let security = parse_toml("agent/security-policy.toml");
    let required = security["required_tools"]
        .as_array()
        .expect("required tools");
    assert_eq!(required.len(), 3);
    assert!(
        required
            .iter()
            .any(|tool| tool.as_str() == Some("gitleaks"))
    );
    assert!(
        required
            .iter()
            .any(|tool| tool.as_str() == Some("cargo-deny"))
    );
    assert!(required.iter().any(|tool| tool.as_str() == Some("zizmor")));

    let boundaries = parse_toml("agent/boundaries.toml");
    assert_eq!(
        boundaries["stack"]["id"].as_str(),
        Some("rust-ts-vite-react-sqlite")
    );
    assert_eq!(
        boundaries["rust"]["domain_paths"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(
        boundaries["db"]["root_paths"].as_array().map(Vec::len),
        Some(0)
    );

    let audit = parse_toml("agent/audit-policy.toml");
    // The Hub audit floor is an upward-only ratchet, and this assertion is the
    // control that keeps the committed policy and the documented floor in step.
    // Raise it only together with `agent/audit-policy.toml`, and only to a score
    // that `bash ops/ci/audit.sh` has produced twice in a row. 58 -> 65 on
    // 2026-08-25 (claude-audit-caps, AUDIT-CAPS-RAISE-R1).
    assert_eq!(audit["minimum_score"].as_integer(), Some(65));
    assert_eq!(
        audit["fail_on"]
            .as_array()
            .expect("audit fail_on")
            .iter()
            .filter_map(TomlValue::as_str)
            .collect::<Vec<_>>(),
        ["critical"]
    );
    assert_eq!(
        audit["scan"]["excluded_paths"].as_array().map(Vec::len),
        Some(0),
        "audit policy must not hide source paths to raise the score"
    );

    let adoption = parse_toml("agent/tool-adoption.toml");
    let tools = adoption["tools"].as_array().expect("adopted tools");
    assert_eq!(tools.len(), 1, "unproved tool adoption was declared");
    assert_eq!(tools[0]["id"].as_str(), Some("audit-ci"));
    assert_eq!(tools[0]["mode"].as_str(), Some("auto"));

    let audit_lane = read("ops/ci/audit.sh");
    assert!(audit_lane.contains("--policy \"$AUDIT_POLICY\""));
    assert!(audit_lane.contains("--fail-under \"$minimum_score\""));
    assert!(!audit_lane.contains("--fail-under 65"));
    assert!(!audit_lane.contains("--fail-on"));
}

#[test]
fn exception_repairs_and_documented_budgets_are_complete_and_policy_bound() {
    let exceptions = parse_toml("agent/exceptions.toml");
    let surface = exceptions["exception_surface"]
        .as_table()
        .expect("exception surface");
    assert_eq!(surface["owner"].as_str(), Some("ops"));
    assert_eq!(surface["docs_url"].as_str(), Some("docs/errors.md"));
    let required = surface["required_fields"]
        .as_array()
        .expect("required exception fields")
        .iter()
        .map(|field| field.as_str().expect("field name"))
        .collect::<Vec<_>>();
    assert_eq!(
        required,
        [
            "purpose",
            "reason",
            "common_fixes",
            "docs_url",
            "repair_hint"
        ]
    );

    let kinds = exceptions["error_kind"].as_array().expect("error kinds");
    assert!(kinds.len() >= 8, "repair taxonomy is too narrow");
    for kind in kinds {
        for field in &required {
            let value = &kind[*field];
            let populated = value.as_str().is_some_and(|text| !text.trim().is_empty())
                || value.as_array().is_some_and(|items| !items.is_empty());
            assert!(populated, "error kind missing {field}: {kind:?}");
        }
    }

    let errors = read("docs/errors.md");
    for heading in [
        "## Invalid input",
        "## Conflict or changed subject",
        "## Unsupported or corrupt state",
        "## Dependency unavailable",
        "## Verification failed",
        "## Outcome unknown",
        "## Receipt missing",
    ] {
        assert!(
            errors.contains(heading),
            "missing repair heading: {heading}"
        );
    }

    let policy = parse_json("policy/v1alpha1/policy.json");
    let budget = &policy["budget_policy"];
    let testing = read("docs/testing.md");
    for (field, label) in [
        ("maximum_lease_ttl_seconds", "15-second maximum lease TTL"),
        ("maximum_attempt_seconds", "1,800-second maximum Attempt"),
        ("maximum_changed_paths", "128 changed paths"),
    ] {
        assert!(budget[field].as_u64().is_some(), "missing policy {field}");
        assert!(
            testing.contains(label),
            "testing guide does not explain current policy field {field}"
        );
    }
    assert_eq!(budget["unknown_quota_is_headroom"].as_bool(), Some(false));
    assert!(testing.contains("`unknown_quota_is_headroom` is `false`"));
    assert!(read("docs/exceptions/README.md").contains("expiry date"));

    let docs_index = read("docs/README.md");
    for entrypoint in ["architecture.md", "boundaries.md", "errors.md"] {
        assert!(
            docs_index.contains(entrypoint),
            "documentation entrypoint is not indexed: {entrypoint}"
        );
    }
    assert!(read("docs/architecture.md").contains("## Current proof boundary"));
    assert!(read("docs/boundaries.md").contains("## Credential custody"));
    let gaps = read("docs/assurance/product-gaps.md");
    let registry_docs = read("docs/assurance/invariant-registry.md");
    let paper = read("docs/paper/bullet_farm_ieee.tex");
    for text in [&gaps, &registry_docs, &paper] {
        assert!(
            text.contains("declared") && text.contains("whole-product"),
            "invariant completeness must be scoped to declared registry entries"
        );
    }
    assert!(gaps.contains("best-known current gap inventory"));
    assert!(gaps.contains("may discover additional orphaned requirements"));
    assert!(!gaps.contains("There is no remaining *undocumented* V1 product gap"));
    let release = read("docs/release.md");
    assert!(release.contains("it is not any release authority"));
    assert!(release.contains("kind-specific semantic verifiers"));
    assert!(release.contains("reads neither a registry nor a fixed machine descriptor"));
    let release_build = read("docs/runbooks/release-build.md");
    assert!(release_build.contains("First-GA `self-hosted-v1` requires one signed"));
    assert!(release_build.contains("frozen universal-envelope component"));
    let roadmap = read("docs/assurance/closure-roadmap.md");
    assert!(roadmap.contains("sign the Hub tag last"));
    assert!(roadmap.contains("generation reads Hub `HEAD`"));
    assert!(roadmap.contains("pre-existing Hub tag would make the order circular"));
    assert!(roadmap.contains("unsigned, non-promotional"));
    assert!(
        roadmap.contains("admits a signed\n`BaselineReceiptV1` over that unchanged frozen subject")
    );
    let schema_removal = read("docs/runbooks/schema-removal.md");
    assert!(schema_removal.contains("OD-B is later live-forge mutation"));
    assert!(!schema_removal.contains("OD-B/OD-D"));
    let historical_scorecard = read("docs/assurance/path-to-100.md");
    assert!(historical_scorecard.contains("historical M0–M5 snapshot"));
    assert!(!historical_scorecard.contains("M0–M5, the V1 contract"));
    let operator_register = read("docs/decisions/0013-operator-decision-register.md");
    assert!(operator_register.contains("traffic <=1%"));
    assert!(operator_register.contains("OD-H → bounded canary"));
    assert!(operator_register.contains("never closes evolution alone"));
    assert!(!operator_register.contains("OD-H is terminal after release/evolution evidence"));
    assert!(
        !operator_register.contains("OD-H waits on passing self-hosted and evolution receipts")
    );
    let registry = parse_json("policy/v1alpha1/invariant-registry.json");
    for id in ["BF-CTL-0C1", "BF-G0-REGISTRY"] {
        let entry = registry["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"].as_str() == Some(id))
            .unwrap();
        assert!(
            entry["statement"]
                .as_str()
                .is_some_and(|statement| statement.contains("declared in this registry")),
            "{id} overclaims whole-product completeness"
        );
    }
}

#[test]
fn every_generated_zone_has_source_and_executable_regeneration_route() {
    let manifest = parse_toml("agent/generated-zones.toml");
    let zones = manifest["zone"].as_array().expect("zone array");
    let mut paths = Vec::new();
    for zone in zones {
        for field in ["path", "source", "command"] {
            assert!(
                zone[field]
                    .as_str()
                    .is_some_and(|value| !value.trim().is_empty()),
                "generated zone missing {field}: {zone:?}"
            );
        }
        assert_eq!(zone["read_only"].as_bool(), Some(true));
        let command = zone["command"].as_str().expect("command");
        assert!(
            command.starts_with("just ")
                || command.starts_with("bash ")
                || command.starts_with("cargo run --locked "),
            "unsupported regeneration route: {command}"
        );
        paths.push(zone["path"].as_str().expect("path"));
    }
    for expected in [
        "family.lock",
        ".fusion/",
        "contracts/v1alpha1/schema-bundle.json",
        "contracts/v1alpha1/bundle-manifest.json",
        "contracts/generated/",
        "policy/v1alpha1/policy.json",
        "fixtures/hostile/cases/",
        "fixtures/hostile/fixture-manifest.json",
        "fixtures/canonical/canonical-golden.json",
        "fixtures/canonical/authority-golden.json",
        "docs/assurance/invariant-crosswalk.generated.md",
        "docs/assurance/release-truth.generated.md",
        "formal/traces/",
    ] {
        assert!(
            paths.contains(&expected),
            "generated zone is unrouted: {expected}"
        );
    }
}

#[test]
fn ignored_and_top_level_surfaces_have_owner_and_test_routes() {
    let owners = parse_json("agent/owner-map.json");
    let tests = parse_json("agent/test-map.json");
    for path in [
        ".fusion/",
        ".gitignore",
        ".jankurai/",
        "Justfile",
        "LICENSE",
        "SPLIT.md",
        "rust-toolchain.toml",
    ] {
        assert!(
            owners["owners"].get(path).is_some(),
            "owner route missing: {path}"
        );
        assert!(
            tests["tests"].get(path).is_some(),
            "test route missing: {path}"
        );
    }
}

#[test]
fn hosted_workflow_is_pinned_and_delegates_to_local_entrypoints() {
    let workflow = read(".github/workflows/ci.yml");
    for command in [
        "run: bash scripts/ci-local.sh source-scan",
        "run: bash scripts/ci-local.sh fast",
        "run: bash scripts/ci-local.sh lint",
        "run: bash scripts/ci-local.sh contract",
        "run: bash scripts/ci-local.sh security",
        "run: bash scripts/ci-local.sh docs",
    ] {
        assert!(
            workflow.contains(command),
            "missing workflow delegation: {command}"
        );
    }
    assert!(workflow.contains("name: CI"));
    assert!(workflow.contains("\n  required:\n    name: required\n    if: ${{ always() }}"));
    let required = workflow
        .split_once("\n  required:\n")
        .expect("required job")
        .1;
    let checkout_positions: Vec<_> = required
        .match_indices("bash ops/ci/checkout-subject.sh")
        .map(|(position, _)| position)
        .collect();
    assert_eq!(checkout_positions.len(), 2, "required job must seal twice");
    let aggregate_position = required
        .find("bash ops/ci/aggregate.sh")
        .expect("required job aggregate");
    assert!(
        checkout_positions[0] < aggregate_position && aggregate_position < checkout_positions[1],
        "required job must seal before and after aggregation"
    );
    for line in workflow
        .lines()
        .filter(|line| line.trim_start().starts_with("- uses:"))
    {
        let reference = line.split_once('@').expect("action pin").1;
        let revision = reference
            .split_whitespace()
            .next()
            .expect("action revision");
        assert_eq!(revision.len(), 40, "action is not pinned: {line}");
        assert!(
            revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid action pin: {line}"
        );
    }
    assert!(read("rust-toolchain.toml").contains("channel = \"1.95.0\""));
    assert!(workflow.contains("zizmor@1.25.2"));
}

#[test]
fn doctor_and_pre_push_gate_are_executable_controls() {
    let doctor = Command::new("bash")
        .args(["scripts/ci-doctor.sh", "fast"])
        .current_dir(root())
        .env_remove("RUSTUP_TOOLCHAIN")
        .status()
        .expect("run CI doctor");
    assert!(doctor.success());
    assert_eq!(read(".node-version"), "22.23.2\n");
    assert_eq!(read(".npm-version"), "10.9.8\n");
    for consumer in [
        "scripts/ci-doctor.sh",
        "scripts/dev.sh",
        "ops/ci/family.sh",
        "ops/ci/artifact-check.sh",
    ] {
        let text = read(consumer);
        assert!(
            text.contains("toolchain-pins.sh"),
            "{consumer} bypasses the canonical Node/npm pins"
        );
        assert!(!text.contains("v22.23.2"), "{consumer} duplicates Node pin");
        assert!(!text.contains("10.9.8"), "{consumer} duplicates npm pin");
    }
    assert!(read("ops/git-hooks/pre-push").contains("ops/ci/quality-gates.sh"));
    assert!(read("ops/ci/quality-gates.sh").contains("exec bash ops/ci/fast.sh"));
    assert!(read("tools/security-lane.sh").contains("ops/ci/security.sh"));
}

#[test]
fn public_lock_recipe_uses_verify_vocabulary() {
    let justfile = read("Justfile");
    assert!(justfile.contains("lock-generate tag subjects:"));
    assert!(justfile.contains("-- lock generate --tag \"$1\" --subjects \"$2\""));
    assert!(justfile.contains("lock-verify tag:"));
    assert!(justfile.contains("-- lock verify --tag \"$1\""));
    assert!(!justfile.contains("--tag {{tag}}"));
    assert!(!justfile.contains("lock-check"));
    assert!(!justfile.contains("-- lock check"));
}

#[cfg(unix)]
#[path = "assurance_controls/recipes.rs"]
mod recipes;
