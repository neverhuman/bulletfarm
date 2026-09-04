//! Explicit `universal-v1` release report: deterministic, portable, never green.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const SELECTED_GATES: usize = 43;
const UNGATED_GAPS: usize = 1;
const BLOCKING_UNGATED_GAPS: usize = 1;

const LOCK: &str = include_str!("fixtures/release-truth/family.lock");
const CHANGELOG: &str = include_str!("../CHANGELOG.md");
const RELEASE_INDEX: &str = include_str!("../docs/release.md");
const MANIFEST: &str = include_str!("../repos.manifest.toml");
const REGISTER: &str = include_str!("../docs/assurance/product-gaps.md");

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn hub_only() -> Self {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "bullet-release-truth-{}-{sequence}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale fixture");
        }
        let hub = root.join("bullet-farm");
        write(
            &hub.join("Cargo.toml"),
            "[package]\nname='fixture-hub'\nversion='0.0.0'\n",
        );
        write(&hub.join("family.lock"), LOCK);
        write(&hub.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n");
        write(&hub.join("repos.manifest.toml"), MANIFEST);
        write(&hub.join("docs/release.md"), RELEASE_INDEX);
        write(&hub.join("docs/assurance/product-gaps.md"), REGISTER);
        Self { root }
    }

    fn family() -> Self {
        let fixture = Self::hub_only();
        write(&fixture.root.join("repos.manifest.toml"), MANIFEST);
        for name in [
            "bullet-farm",
            "bullet-kernel",
            "bullet-git",
            "bullet-portal",
        ] {
            let repo = fixture.root.join(name);
            fs::create_dir_all(&repo).expect("repository fixture");
            write(&repo.join("README.md"), "fixture\n");
            git(&repo, &["init", "-q"]);
            git(&repo, &["config", "user.name", "Truth Fixture"]);
            git(&repo, &["config", "user.email", "truth@example.invalid"]);
            git(&repo, &["add", "."]);
            git(
                &repo,
                &[
                    "-c",
                    "commit.gpgsign=false",
                    "commit",
                    "-q",
                    "-m",
                    "fixture",
                    "--date",
                    "2026-01-02T03:04:05Z",
                ],
            );
        }
        fixture
    }

    fn hub(&self) -> PathBuf {
        self.root.join("bullet-farm")
    }

    fn run(&self, tail: &[&str]) -> Output {
        let mut args = vec!["--root", self.root.to_str().expect("UTF-8 fixture")];
        args.extend_from_slice(tail);
        Command::new(env!("CARGO_BIN_EXE_bullet-family"))
            .args(args)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("run bullet-family")
    }

    fn release_report(&self, portable: bool) -> Output {
        self.release_report_for("universal-v1", portable)
    }

    fn release_report_for(&self, profile: &str, portable: bool) -> Output {
        let registry = self.root.join("release-receipts");
        fs::create_dir_all(&registry).expect("receipt registry");
        self.release_report_for_registry(profile, &registry, portable)
    }

    fn release_report_for_registry(
        &self,
        profile: &str,
        registry: &Path,
        portable: bool,
    ) -> Output {
        let mut args = vec![
            "check",
            "release",
            "--profile",
            profile,
            "--receipts",
            registry.to_str().expect("UTF-8 registry"),
            "--report",
        ];
        if portable {
            args.push("--portable");
        }
        self.run(&args)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove fixture");
    }
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directories");
    fs::write(path, content).expect("fixture file");
}

fn git(repository: &Path, args: &[&str]) {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .env_clear()
        .env("HOME", "/")
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_COMMITTER_DATE", "2026-01-02T03:04:05Z")
        .output()
        .expect("fixture Git");
    assert!(output.status.success(), "fixture Git failed: {output:?}");
}

fn head(repository: &Path) -> String {
    let output = Command::new("/usr/bin/git")
        .args(["-C", repository.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .expect("fixture Git");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn claim_lines<'a>(page: &'a str, start: &str, end: &str) -> Vec<&'a str> {
    let (_, section) = page.split_once(start).expect("section start");
    let (section, _) = section.split_once(end).expect("section end");
    section
        .lines()
        .filter(|line| {
            line.contains("**")
                || line.contains("- Why it matters:")
                || line.contains("- Acceptance:")
        })
        .collect()
}

fn gate_lines(page: &str) -> Vec<&str> {
    let mut lines = claim_lines(page, "## Gates", "## Product-gap crosswalk");
    lines.extend(claim_lines(
        page,
        "## Blocking but not a release gate",
        "## Excluded",
    ));
    lines
}

fn selected_gate_gaps(page: &str) -> BTreeSet<String> {
    let gates = page
        .split_once("## Gates")
        .expect("gates section")
        .1
        .split_once("## Product-gap crosswalk")
        .expect("crosswalk section")
        .0;
    gates
        .lines()
        .filter_map(|line| line.strip_prefix("   - Product gap: "))
        .flat_map(|gaps| gaps.split(", "))
        .map(str::to_owned)
        .collect()
}

fn next_command_lines(page: &str) -> Vec<&str> {
    page.lines()
        .filter(|line| line.starts_with("   - Next command: "))
        .collect()
}

fn assert_release_commands_target_profile(page: &str, expected_profile: &str) {
    for line in next_command_lines(page)
        .into_iter()
        .filter(|line| line.contains("check release --profile"))
    {
        let target = line
            .split_once("--profile ")
            .expect("profile argument")
            .1
            .split_ascii_whitespace()
            .next()
            .expect("profile value");
        assert_eq!(
            target, expected_profile,
            "repair command widened scope: {line}"
        );
    }
}

const OWNER_LABELS: [&str; 3] = ["LOCAL (", "LOCAL-then-EXTERNAL (", "EXTERNAL ("];

fn assert_fields_are_closed_vocabulary(page: &str) {
    assert_eq!(page.matches("   - Product gap: G").count(), SELECTED_GATES);
    assert_eq!(
        page.matches("   - Release-blocking: yes").count(),
        SELECTED_GATES + BLOCKING_UNGATED_GAPS
    );
    assert_eq!(page.matches("   - Release-blocking: no for V1").count(), 0);
    let mut owners = 0;
    for line in page.lines().filter(|line| line.starts_with("   - Owner: ")) {
        let owner = line.trim_start_matches("   - Owner: ");
        assert!(
            OWNER_LABELS.iter().any(|label| owner.starts_with(label)),
            "owner outside the closed vocabulary: {line}"
        );
        owners += 1;
    }
    assert_eq!(owners, SELECTED_GATES + UNGATED_GAPS);
    let mut nexts = 0;
    for line in page
        .lines()
        .filter(|line| line.starts_with("   - Next command: "))
    {
        let next = line.trim_start_matches("   - Next command: ");
        assert!(
            next.starts_with('`') || next.starts_with("NONE — no typed command exists yet"),
            "next command is neither typed nor honestly absent: {line}"
        );
        nexts += 1;
    }
    assert_eq!(nexts, SELECTED_GATES + UNGATED_GAPS);
}

fn assert_never_closed(page: &str) {
    let lines = gate_lines(page);
    assert_eq!(lines.len(), (SELECTED_GATES + UNGATED_GAPS) * 3);
    for line in lines {
        for token in line.split(|byte: char| !byte.is_ascii_alphanumeric()) {
            assert!(
                !matches!(
                    token.to_ascii_lowercase().as_str(),
                    "verified" | "proven" | "done" | "complete"
                ),
                "unreceipted claim reads as closed: {line}"
            );
        }
    }
}

#[test]
fn portable_report_matches_the_golden_page_from_a_hub_only_checkout() {
    let fixture = Fixture::hub_only();
    let first = fixture.release_report(true);
    let second = fixture.release_report(true);
    assert_eq!(first.status.code(), Some(3), "{first:?}");
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let page = String::from_utf8(first.stdout).unwrap();
    let golden = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/assurance/release-truth.generated.md"),
    )
    .expect("generated release-truth page");
    assert_eq!(page, golden, "portable page drifted from golden");
    assert!(page.contains(
        "bullet-family lock generate --tag <prospective-version> --subjects <absolute-path>"
    ));
    assert!(!page.contains("<signed-hub-tag>"));
    for gate in [
        "release.forge.github-app",
        "release.forge.jeryu",
        "release.installer-twice",
        "release.manifest-non-circular",
        "release.platform-containment",
        "release.provenance",
        "release.provider.antigravity",
        "release.provider.claude",
        "release.provider.codex",
        "release.provider.cursor",
        "release.signatures",
    ] {
        let heading = format!("— `{gate}` BLOCKED\n");
        let start = page
            .find(&heading)
            .unwrap_or_else(|| panic!("{gate} row missing"));
        let row = &page[start..];
        let end = row
            .find("   - Release-blocking: yes\n")
            .unwrap_or_else(|| panic!("{gate} row has no release-blocking terminator"));
        assert!(
            row[..end].contains("   - Owner: LOCAL-then-EXTERNAL "),
            "{gate} hides unfinished local engineering behind an EXTERNAL owner"
        );
    }
    assert!(!page.contains(fixture.root.to_str().unwrap()));
    assert_never_closed(&page);
    assert_fields_are_closed_vocabulary(&page);
    assert!(page.contains("`release.backup-restore` BLOCKED\n   - Product gap: G3, G9\n"));
    assert!(page.contains("Agreement with `docs/assurance/product-gaps.md`: YES — all 46 crosswalk rows and the G-id list agree"));
    assert!(
        page.contains("| G12 | Family `check release` | this `universal-v1` inventory — 43 selected gates, 0 receipted")
    );
    assert!(page.contains("| G4 | "), "G4 missing from crosswalk");
    assert!(page.contains("** — G4 "), "G4 missing from ungated section");
    assert!(page.contains("| G11 | Evolutionary runtime | NOT SELECTED BY `universal-v1` — independently owned by `release.profile.evolution-v1` |"));
    assert!(!page.contains("UNANSWERED"));
    for provider in ["claude", "codex", "cursor", "agy"] {
        assert!(page.contains(&format!(
            "BULLET_LIVE_PROVIDERS={provider} bash ops/ci/nightly.sh"
        )));
    }
    assert!(page.contains("docs/runbooks/live-conformance.md"));
    assert!(page.contains(
        "the quarantined internal Linux component writes and re-reads unsigned in-toto provenance bound to its one-target component subject"
    ));
    assert!(!page.contains("no provenance producer"));
    assert!(page.contains("| product-gap register | `docs/assurance/product-gaps.md` | blake3:"));
}

#[test]
fn every_product_profile_report_is_renderable_and_has_no_orphan_gap() {
    let fixture = Fixture::hub_only();
    for profile in [
        "self-hosted-v1",
        "evolution-v1",
        "provider-claude",
        "provider-codex",
        "provider-cursor",
        "provider-antigravity",
        "jeryu-forge-v1",
        "github-adapter-v1",
        "gitlab-adapter-v1",
        "gitlab-self-managed-v1",
        "platform-linux-x86_64",
        "platform-linux-aarch64",
        "platform-macos-x86_64",
        "platform-macos-aarch64",
        "platform-windows-x86_64",
        "universal-v1",
        "team-v1",
        "saga-v1",
        "legacy-v1-26",
        "linux-preview",
    ] {
        let output = fixture.release_report_for(profile, true);
        assert_eq!(output.status.code(), Some(3), "{profile}: {output:?}");
        assert!(output.stderr.is_empty(), "{profile}: {output:?}");
        let page = String::from_utf8(output.stdout).unwrap();
        assert!(page.contains(&format!("this `{profile}` inventory")));
        assert!(page.contains(&format!("| profile | `{profile}` |")));
        assert_eq!(
            page.matches("check report schema 3").count(),
            2,
            "{profile} must render the profiled schema consistently in the source and freshness rows"
        );
        assert!(page.contains("| required closure | "));
        if profile == "legacy-v1-26" {
            assert!(page.contains("| receipt admission | diagnostic-only: `--receipts` is syntactically required but its contents are deliberately ignored; no registry or legacy fixed `/etc` descriptor is read |"));
            assert!(page.contains("static diagnostic inventory; the syntactically required `--receipts` argument is ignored and cannot change any gate"));
            assert!(
                !next_command_lines(&page)
                    .join("\n")
                    .contains("--receipts /absolute/admitted-registry")
            );
        } else {
            assert!(page.contains("| receipt admission | the selected semantic registry was evaluated before rendering, but this portable projection excludes its path and identity and is not evidence; the legacy fixed `/etc/bullet-farm/release-msrv-1-95-admission.toml` descriptor is deliberately not read |"));
            assert_release_commands_target_profile(&page, profile);
        }
        if profile == "universal-v1" {
            assert!(page.contains("<!-- Command: just release-truth -->"));
        } else {
            assert!(page.contains(&format!(
                "<!-- Command: bullet-family check release --profile {profile} --receipts ABSOLUTE_REGISTRY --report --portable -->"
            )));
            assert!(!page.contains("<!-- Command: just release-truth -->"));
        }
        assert!(!page.contains("UNANSWERED"), "{profile} has an orphan gap");
        assert!(page.contains("Agreement with `docs/assurance/product-gaps.md`: YES"));
    }
}

#[test]
fn narrow_profiles_select_only_their_exact_gap_sets_and_scoped_commands() {
    let fixture = Fixture::hub_only();
    for (profile, expected) in [
        ("provider-claude", &["G5", "G12"][..]),
        ("provider-codex", &["G5", "G12"][..]),
        ("provider-cursor", &["G5", "G12"][..]),
        ("provider-antigravity", &["G5", "G12"][..]),
        ("jeryu-forge-v1", &["G6", "G12"][..]),
        ("github-adapter-v1", &["G7", "G12"][..]),
        ("gitlab-adapter-v1", &["G12", "G16"][..]),
        ("gitlab-self-managed-v1", &["G12", "G16"][..]),
        ("platform-linux-x86_64", &["G1", "G9", "G10", "G12"][..]),
        ("platform-linux-aarch64", &["G9", "G10", "G12"][..]),
        ("platform-macos-x86_64", &["G9", "G10", "G12"][..]),
        ("platform-macos-aarch64", &["G9", "G10", "G12"][..]),
        ("platform-windows-x86_64", &["G9", "G10", "G12"][..]),
    ] {
        let output = fixture.release_report_for(profile, true);
        assert_eq!(output.status.code(), Some(3), "{profile}: {output:?}");
        let page = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            selected_gate_gaps(&page),
            expected.iter().map(|gap| (*gap).to_owned()).collect(),
            "{profile} selected an unrelated product gap"
        );
        assert!(page.contains(
            "   - Release-blocking: no for this profile — `release.transaction-demo` is not selected"
        ));
        assert_release_commands_target_profile(&page, profile);
        if profile != "platform-linux-x86_64" {
            let next = next_command_lines(&page).join("\n");
            assert!(
                !next.contains("x86_64-unknown-linux-gnu"),
                "{profile}: {next}"
            );
            assert!(!next.contains("ops/ci/egress.sh"), "{profile}: {next}");
            assert!(
                !next.contains("release verify --bundle"),
                "{profile}: {next}"
            );
        }
    }
}

#[test]
fn self_hosted_and_evolution_conditions_own_product_surface_truth() {
    let fixture = Fixture::hub_only();
    let self_hosted = fixture.release_report_for("self-hosted-v1", true);
    assert_eq!(self_hosted.status.code(), Some(3));
    let self_hosted = String::from_utf8(self_hosted.stdout).unwrap();
    assert!(self_hosted.contains("every selected product surface durable or typed OUT_OF_PROFILE"));
    assert!(
        self_hosted
            .contains("   - Release-blocking: yes — through G2 (`release.transaction-demo`)")
    );
    assert!(
        self_hosted
            .contains("--profile self-hosted-v1 --receipts /absolute/admitted-registry --json")
    );
    assert!(
        !next_command_lines(&self_hosted)
            .join("\n")
            .contains("--profile universal-v1")
    );
    assert!(self_hosted.contains("reads the selected semantic registry and deliberately does not read the legacy fixed `/etc/bullet-farm/release-msrv-1-95-admission.toml` descriptor"));
    assert!(self_hosted.contains("self-hosted-v1 requires its one Ubuntu x86_64 entry"));
    assert!(self_hosted.contains("universal-envelope component and cannot admit self-hosted-v1"));
    assert!(self_hosted.contains(
        "`release.profile.self-hosted-v1` BLOCKED\n   - Product gap: G1, G2, G3, G5, G6, G8, G9, G10, G13, G14"
    ));
    assert!(self_hosted.contains("| G15 | Cognitive persistence | NOT SELECTED BY `self-hosted-v1` — independently owned by `release.profile.evolution-v1` |"));
    for scope_widening in [
        "remaining target builders and five-target checksum aggregation",
        "five archives on macOS/Windows build platforms",
        "native macOS/Windows containment backends or their fail-closed refusal receipts on packaged bytes",
        "archive digests from the package matrix",
        "five archives of the package matrix",
        "it is not a five-target checksum set",
        "no signed five-target provenance producer or admission",
        "no five-target signed SBOM set",
    ] {
        assert!(
            !self_hosted.contains(scope_widening),
            "self-hosted report widened its selected target set: {scope_widening}"
        );
    }

    let evolution = fixture.release_report_for("evolution-v1", true);
    assert_eq!(evolution.status.code(), Some(3));
    let evolution = String::from_utf8(evolution.stdout).unwrap();
    assert!(evolution.contains("all fifteen product surfaces durable"));
    assert!(
        evolution.contains(
            "`release.profile.evolution-v1` BLOCKED\n   - Product gap: G11, G13, G14, G15"
        )
    );

    for profile in ["self-hosted-v1", "evolution-v1", "team-v1", "saga-v1"] {
        let output = fixture.release_report_for(profile, true);
        assert_eq!(output.status.code(), Some(3), "{profile}: {output:?}");
        let page = String::from_utf8(output.stdout).unwrap();
        assert_release_commands_target_profile(&page, profile);
    }
}

#[test]
fn legacy_report_is_static_and_ignores_the_required_registry_argument() {
    let fixture = Fixture::hub_only();
    let directory = fixture.root.join("legacy-empty-registry");
    fs::create_dir_all(&directory).unwrap();
    let sentinel = fixture.root.join("legacy-sentinel-registry");
    write(&sentinel, "not a registry\n");

    let empty = fixture.release_report_for_registry("legacy-v1-26", &directory, true);
    let hostile = fixture.release_report_for_registry("legacy-v1-26", &sentinel, true);
    assert_eq!(empty.status.code(), Some(3), "{empty:?}");
    assert_eq!(hostile.status.code(), Some(3), "{hostile:?}");
    assert_eq!(empty.stdout, hostile.stdout);
    assert!(empty.stderr.is_empty());
    assert!(hostile.stderr.is_empty());
    let page = String::from_utf8(empty.stdout).unwrap();
    assert!(page.contains("G12 is inventory-only"));
    assert!(page.contains("this diagnostic does not evaluate or admit registry contents"));
    assert!(page.contains("this diagnostic cannot close or waive G12"));
    assert!(page.contains("this diagnostic cannot admit an MSRV receipt"));
    assert!(page.contains("LOCAL (diagnostic maintenance only)"));
    assert!(page.contains("diagnostic `release.receipt-contracts` row selected statically; `--receipts` contents ignored"));
    for false_use in [
        "G12 combines this exact profile inventory with its selected semantic-admission machinery",
        "G12 deliberately shows both the inventory projection and its semantic-admission gate",
        "the selected semantic registry is read for this exact profile",
        "evaluates generic signer, trusted-time, replay, exact-family, and semantic admission",
        "reads the selected semantic registry and deliberately",
    ] {
        assert!(
            !page.contains(false_use),
            "legacy report claims registry use: {false_use}"
        );
    }

    let semantic = fixture.release_report_for_registry("provider-codex", &sentinel, true);
    assert_eq!(semantic.status.code(), Some(1), "{semantic:?}");
    assert!(semantic.stderr.is_empty());
    let semantic = String::from_utf8(semantic.stdout).unwrap();
    assert!(semantic.contains("`release.receipt-contracts` FAIL"));
    assert!(semantic.contains("the requested profile-condition receipt must also pass, and neither half can substitute for the other"));
    assert!(semantic.contains("kind-specific semantic validators and typed admission for the exact requested profile closure are Hub engineering"));

    assert!(RELEASE_INDEX.contains("The public `legacy-v1-26` diagnostic is static"));
    assert!(RELEASE_INDEX.contains("older unprofiled one-gate MSRV evaluator"));
    assert!(RELEASE_INDEX.contains("No public command or current profile\ninvokes that path"));
    assert!(CHANGELOG.contains("command or current profile invokes it"));
    for stale in [
        "Only the historical `legacy-v1-26` evaluator has a semantic receipt-admission path",
        "its absence keeps the gate `BLOCKED`",
    ] {
        assert!(
            !RELEASE_INDEX.contains(stale),
            "stale release docs: {stale}"
        );
        assert!(!CHANGELOG.contains(stale), "stale changelog: {stale}");
    }
}

#[test]
fn register_crosswalk_drift_is_visible_and_absent_register_is_unknown() {
    let fixture = Fixture::hub_only();
    write(
        &fixture.hub().join("docs/assurance/product-gaps.md"),
        &REGISTER.replace(
            "| `release.fault-suite` | G2, G3 |",
            "| `release.fault-suite` | G3 |",
        ),
    );
    let output = fixture.release_report(true);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains(
        "Agreement with `docs/assurance/product-gaps.md`: NO — `release.fault-suite`: page G2, G3 vs register G3"
    ));
    assert!(page.contains("   - Product gap: G2, G3\n"));
    fs::remove_file(fixture.hub().join("docs/assurance/product-gaps.md")).unwrap();
    let output = fixture.release_report(true);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains(
        "Agreement with `docs/assurance/product-gaps.md`: UNKNOWN — the register is absent"
    ));
    assert!(page.contains("| product-gap register | `docs/assurance/product-gaps.md` | absent |"));
    assert_never_closed(&page);
}

#[test]
fn live_report_binds_subjects_and_check_report_freshness() {
    let fixture = Fixture::family();
    let first = fixture.release_report(false);
    let second = fixture.release_report(false);
    assert_eq!(first.status.code(), Some(3), "{first:?}");
    assert_eq!(first.stdout, second.stdout);
    let page = String::from_utf8(first.stdout).unwrap();
    assert!(page.contains("RELEASE DECISION: BLOCKED"));
    assert!(page.contains("hub HEAD committed 2026-01-02T03:04:05+00:00"));
    assert!(page.contains(&format!("| hub | `{}` |", fixture.hub().display())));
    let hub_head = head(&fixture.hub());
    assert!(page.contains(&format!("| bullet-farm | `sha1:{hub_head}` | `sha1:")));
    assert!(page.contains("| clean |"));
    assert!(page.contains("| binds current HEADs | NO — bullet-farm locked `4d7f2173"));
    assert!(page.contains("| Mechanical gates (fast) | NOT RUN (no generated check report) |"));
    assert!(page.contains("| Mechanical gates (required) | NOT RUN (no generated check report) |"));
    assert!(page.contains("| Evidence completeness | 0 of 43 receipted |"));
    assert!(page.contains("| Release review | HOLD"));
    assert!(page.contains("| Deployment match | N/A"));
    assert!(page.contains("| Post-deploy survival | NOT ESTABLISHED |"));
    assert_never_closed(&page);
    assert_fields_are_closed_vocabulary(&page);
    assert!(page.contains("| product-gap register | `docs/assurance/product-gaps.md` | blake3:"));

    let stale = format!(
        "{{\"schema_version\":2,\"command\":\"check\",\"tier\":\"FAST\",\"status\":\"PASS\",\"gates\":[{{\"id\":\"fast.hub\",\"status\":\"PASS\",\"class\":\"COMPONENT\",\"detail\":\"d\",\"repair\":null,\"subjects\":[{{\"repository\":\"bullet-farm\",\"commit_oid\":\"sha1:{}\",\"tree_oid\":\"sha1:{}\"}}]}}]}}",
        "0".repeat(40),
        "1".repeat(40)
    );
    write(
        &fixture.hub().join(".bullet-family/check-fast.json"),
        &stale,
    );
    let fresh = stale.replace(&"0".repeat(40), &hub_head);
    write(
        &fixture.hub().join(".bullet-family/check-required.json"),
        &fresh.replace("FAST", "REQUIRED"),
    );
    write(&fixture.hub().join("UNTRACKED"), "dirty\n");
    let output = fixture.release_report(false);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains("| Mechanical gates (fast) | STALE — PASS over 1 gates recorded against other subjects (bullet-farm recorded sha1:0000"));
    assert!(page.contains("| Mechanical gates (required) | PASS — 1 gates on current HEADs |"));
    assert!(page.contains("| bullet-farm | `sha1:"));
    assert!(page.contains("| dirty (3 entries) |"));
    assert!(page.contains("| fast check report | `.bullet-family/check-fast.json` | blake3:"));

    let duplicate_member = "credential_ghp_1234567890abcdef";
    let duplicate = fresh.replacen(
        "\"status\":\"PASS\"",
        &format!(
            "\"{duplicate_member}\":\"first\",\"{duplicate_member}\":\"second\",\"status\":\"PASS\""
        ),
        1,
    );
    write(
        &fixture.hub().join(".bullet-family/check-required.json"),
        &duplicate.replace("FAST", "REQUIRED"),
    );
    let output = fixture.release_report(false);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains("Mechanical gates (required) | NOT RUN (unreadable:"));
    assert!(page.contains("DUPLICATE_JSON_KEY"));
    assert!(!page.contains(duplicate_member));
}

#[test]
fn report_mode_is_release_only_and_strict() {
    let fixture = Fixture::hub_only();
    for args in [
        vec!["check", "fast", "--report"],
        vec!["check", "required", "--report", "--portable"],
        vec!["check", "release", "--portable"],
        vec!["check", "release", "--report", "--json"],
    ] {
        let output = fixture.run(&args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stdout.is_empty(), "args={args:?}");
        assert!(String::from_utf8(output.stderr).unwrap().contains("USAGE"));
    }
}
