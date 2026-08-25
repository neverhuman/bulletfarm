//! `bullet-family check release --report`: deterministic, portable, never green.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

const LOCK: &str = include_str!("fixtures/release-truth/family.lock");
const RELEASE_INDEX: &str = "# Fixture release contract\n\nStatus: **BLOCKED — fixture**  \nOwner: fixture\nLast reviewed: 2026-01-01\n";
const MANIFEST: &str = "schema_version = \"1.2.0\"\nfamily = \"bullet-farm\"\numbrella_repo = \"bullet-farm\"\nrequired_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n";
/// The register's two bound tables, byte-stable for the golden page; prose is omitted on purpose.
const REGISTER: &str = "# Fixture product gap register\n\n| ID | Gap |\n| --- | --- |\n| G1 | Hub-only signed install |\n| G2 | Connected five-plane transaction |\n| G3 | Production Kernel write path |\n| G4 | Production BulletGit write path |\n| G5 | Live provider conformance |\n| G6 | Jeryu live effect |\n| G7 | GitHub live effect |\n| G8 | Security release floor |\n| G9 | Signed five-target release |\n| G10 | Non-Linux containment |\n| G11 | Evolutionary runtime |\n| G12 | Family `check release` |\n| G13 | Portal product surfaces |\n| G14 | farmd production API |\n| G15 | Cognitive persistence |\n\n| Gate ID | Product gap | Class |\n| --- | --- | --- |\n| `release.installable-lock` | G1 | Release |\n| `release.installer-twice` | G1 | Release |\n| `release.transaction-demo` | G2 | Transaction |\n| `release.fault-suite` | G2, G3 | Release |\n| `release.backup-restore` | G3, G9 | Release |\n| `release.provider.claude` | G5 | Live |\n| `release.provider.codex` | G5 | Live |\n| `release.provider.cursor` | G5 | Live |\n| `release.provider.antigravity` | G5 | Live |\n| `release.forge.jeryu` | G6 | Live |\n| `release.forge.github-app` | G7 | Live |\n| `release.jankurai-90` | G8 | Release |\n| `release.scan.dependency` | G8, G9 | Release |\n| `release.scan.license` | G8, G9 | Release |\n| `release.scan.secret` | G8, G9 | Release |\n| `release.scan.workflow` | G8, G9 | Release |\n| `release.checksums` | G9 | Release |\n| `release.manifest-non-circular` | G9 | Release |\n| `release.package-matrix` | G9 | Release |\n| `release.provenance` | G9 | Release |\n| `release.receipt-contracts` | G9 | Release |\n| `release.rust-msrv-1-95` | G9 | Release |\n| `release.rust-pinned-1-97-1` | G9 | Release |\n| `release.sbom` | G9 | Release |\n| `release.signatures` | G9 | Release |\n| `release.platform-containment` | G10 | Release |\n";

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

const OWNER_LABELS: [&str; 3] = ["LOCAL (", "LOCAL-then-EXTERNAL (", "EXTERNAL ("];

fn assert_fields_are_closed_vocabulary(page: &str) {
    assert_eq!(page.matches("   - Product gap: G").count(), 26);
    assert_eq!(page.matches("   - Release-blocking: yes").count(), 26 + 4);
    assert_eq!(page.matches("   - Release-blocking: no for V1").count(), 1);
    let mut owners = 0;
    for line in page.lines().filter(|line| line.starts_with("   - Owner: ")) {
        let owner = line.trim_start_matches("   - Owner: ");
        assert!(
            OWNER_LABELS.iter().any(|label| owner.starts_with(label)),
            "owner outside the closed vocabulary: {line}"
        );
        owners += 1;
    }
    assert_eq!(owners, 26 + 5);
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
    assert_eq!(nexts, 26 + 5);
}

fn assert_never_closed(page: &str) {
    let lines = gate_lines(page);
    assert_eq!(lines.len(), (26 + 5) * 3);
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
    let first = fixture.run(&["check", "release", "--report", "--portable"]);
    let second = fixture.run(&["check", "release", "--report", "--portable"]);
    assert_eq!(first.status.code(), Some(3), "{first:?}");
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let page = String::from_utf8(first.stdout).unwrap();
    let golden = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/release-truth/golden.md"),
    )
    .expect("golden page");
    assert_eq!(page, golden, "portable page drifted from golden");
    assert!(!page.contains(fixture.root.to_str().unwrap()));
    assert_never_closed(&page);
    assert_fields_are_closed_vocabulary(&page);
    assert!(page.contains("`release.backup-restore` BLOCKED\n   - Product gap: G3, G9\n"));
    assert!(page.contains("Agreement with `docs/assurance/product-gaps.md`: YES — all 26 crosswalk rows and the G-id list agree"));
    assert!(
        page.contains("| G12 | Family `check release` | this inventory — 26 gates, 0 receipted")
    );
    for gap in ["G4", "G11", "G13", "G14", "G15"] {
        assert!(
            page.contains(&format!("| {gap} | ")),
            "{gap} missing from crosswalk"
        );
        assert!(
            page.contains(&format!("** — {gap} ")),
            "{gap} missing from ungated section"
        );
    }
    assert!(page.contains(
        "Release-blocking: no for V1 — self-tuning optimization is post-V1; `linux-preview` surfaces an extra evolution diagnostic that cannot alter the canonical 26-gate GA contract"
    ));
    for provider in ["claude", "codex", "cursor", "agy"] {
        assert!(page.contains(&format!(
            "BULLET_LIVE_PROVIDERS={provider} bash ops/ci/nightly.sh"
        )));
    }
    assert!(page.contains("docs/runbooks/live-conformance.md"));
    assert!(page.contains("| product-gap register | `docs/assurance/product-gaps.md` | blake3:"));
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
    let output = fixture.run(&["check", "release", "--report", "--portable"]);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains(
        "Agreement with `docs/assurance/product-gaps.md`: NO — `release.fault-suite`: page G2, G3 vs register G3"
    ));
    assert!(page.contains("   - Product gap: G2, G3\n"));
    fs::remove_file(fixture.hub().join("docs/assurance/product-gaps.md")).unwrap();
    let output = fixture.run(&["check", "release", "--report", "--portable"]);
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
    let first = fixture.run(&["check", "release", "--report"]);
    let second = fixture.run(&["check", "release", "--report"]);
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
    assert!(page.contains("| Evidence completeness | 0 of 26 receipted |"));
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
    let output = fixture.run(&["check", "release", "--report"]);
    assert_eq!(output.status.code(), Some(3));
    let page = String::from_utf8(output.stdout).unwrap();
    assert!(page.contains("| Mechanical gates (fast) | STALE — PASS over 1 gates recorded against other subjects (bullet-farm recorded sha1:0000"));
    assert!(page.contains("| Mechanical gates (required) | PASS — 1 gates on current HEADs |"));
    assert!(page.contains("| bullet-farm | `sha1:"));
    assert!(page.contains("| dirty (3 entries) |"));
    assert!(page.contains("| fast check report | `.bullet-family/check-fast.json` | blake3:"));
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
