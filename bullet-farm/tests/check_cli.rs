use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "check_cli/semantic_registry.rs"]
mod semantic_registry;

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run bullet-family")
}

struct FamilyFixture {
    root: PathBuf,
}

impl FamilyFixture {
    fn new(kernel_fast: &str) -> Self {
        let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "bullet-check-cli-{}-{sequence}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove stale exact fixture");
        }
        fs::create_dir_all(&root).expect("fixture root");
        write(
            &root.join("repos.manifest.toml"),
            "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
        );
        for name in [
            "bullet-farm",
            "bullet-kernel",
            "bullet-git",
            "bullet-portal",
        ] {
            let repo = root.join(name);
            fs::create_dir_all(&repo).expect("repository fixture");
            if name == "bullet-farm" {
                write(
                    &repo.join("Cargo.toml"),
                    "[package]\nname='fixture-hub'\nversion='0.0.0'\n",
                );
                write(&repo.join("family.lock"), "schema_version = \"2\"\n");
                write(&repo.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n");
                write(&repo.join("scripts/ci-local.sh"), "#!/bin/sh\nexit 0\n");
                write(
                    &repo.join("scripts/sync-family-contracts.sh"),
                    "#!/bin/sh\nexit 0\n",
                );
                write(&repo.join("scripts/demo.sh"), "#!/bin/sh\nexit 0\n");
                write(
                    &repo.join("ops/ci/family-contract.sh"),
                    "#!/bin/sh\nexit 0\n",
                );
            } else {
                let script = if name == "bullet-kernel" {
                    kernel_fast
                } else {
                    "#!/bin/sh\nexit 0\n"
                };
                write(&repo.join("scripts/ci-local.sh"), script);
            }
            git(&repo, &["init", "-q"]);
            git(&repo, &["config", "user.name", "Check Fixture"]);
            git(&repo, &["config", "user.email", "check@example.invalid"]);
            git(&repo, &["add", "."]);
            git(&repo, &["commit", "-q", "-m", "fixture"]);
        }
        Self { root }
    }

    fn args<'a>(&'a self, tail: &'a [&'a str]) -> Vec<&'a str> {
        let mut args = vec!["--root", self.root.to_str().expect("UTF-8 fixture")];
        args.extend_from_slice(tail);
        args
    }
}

impl Drop for FamilyFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove exact fixture");
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
        .output()
        .expect("fixture Git");
    assert!(output.status.success(), "fixture Git failed: {output:?}");
}

fn unsupported_lock_fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-check-cli-unsupported-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    let hub = root.join("bullet-farm");
    fs::create_dir_all(hub.join("scripts")).expect("fixture directories");
    fs::write(
        hub.join("Cargo.toml"),
        "[package]\nname='bullet-family'\nversion='0.0.0'\n",
    )
    .expect("Cargo fixture");
    fs::write(hub.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").expect("setup fixture");
    fs::write(
        hub.join("repos.manifest.toml"),
        concat!(
            "schema_version = \"1.2.0\"\n",
            "family = \"bullet-farm\"\n",
            "umbrella_repo = \"bullet-farm\"\n",
            "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
        ),
    )
    .expect("manifest fixture");
    fs::write(
        hub.join("family.lock"),
        "schema_version = \"unsupported\"\n",
    )
    .expect("lock fixture");
    root
}

#[test]
fn fast_catalog_passes_only_on_clean_unchanged_exact_subjects() {
    let fixture = FamilyFixture::new("#!/bin/sh\nexit 0\n");
    let first = command(&fixture.args(&["check", "fast", "--json"]));
    let second = command(&fixture.args(&["check", "fast", "--json"]));
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["schema_version"], 2);
    assert_eq!(report["tier"], "FAST");
    assert_eq!(report["status"], "PASS");
    let gates = report["gates"].as_array().unwrap();
    assert_eq!(gates.len(), 7);
    assert!(gates.iter().all(|gate| gate["status"] == "PASS"));
    assert!(gates.iter().all(|gate| {
        gate["subjects"].as_array().is_some_and(|subjects| {
            !subjects.is_empty()
                && subjects.iter().all(|subject| {
                    subject["commit_oid"]
                        .as_str()
                        .is_some_and(|oid| oid.starts_with("sha1:"))
                        && subject["tree_oid"]
                            .as_str()
                            .is_some_and(|oid| oid.starts_with("sha1:"))
                })
        })
    }));
}

#[test]
fn required_executes_components_but_retains_real_blockers() {
    let fixture = FamilyFixture::new("#!/bin/sh\nexit 0\n");
    let output = command(&fixture.args(&["check", "required", "--json"]));
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "BLOCKED");
    let gates = report["gates"].as_array().unwrap();
    assert_eq!(gates.len(), 8);
    for id in ["required.demo-component", "required.family-contract"] {
        let gate = gates.iter().find(|gate| gate["id"] == id).unwrap();
        assert_eq!(gate["status"], "PASS");
        assert_eq!(gate["subjects"].as_array().unwrap().len(), 4);
    }
    assert_eq!(
        gates
            .iter()
            .filter(|gate| gate["status"] == "BLOCKED")
            .count(),
        6
    );
}

#[test]
fn dirty_or_mutated_subjects_never_pass() {
    let dirty = FamilyFixture::new("#!/bin/sh\nexit 0\n");
    write(&dirty.root.join("bullet-kernel/UNTRACKED"), "dirty\n");
    let output = command(&dirty.args(&["check", "fast", "--json"]));
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["gates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|gate| { gate["id"] == "catalog.exact-subjects" && gate["status"] == "BLOCKED" })
    );

    let mutated = FamilyFixture::new("#!/bin/sh\ntouch CHECK_MUTATION\nexit 0\n");
    let output = command(&mutated.args(&["check", "fast", "--json"]));
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["gates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|gate| { gate["id"] == "fast.kernel" && gate["status"] == "UNKNOWN" })
    );
}

#[test]
fn nonzero_command_is_fail_and_invalid_manifest_is_blocked() {
    let failed = FamilyFixture::new("#!/bin/sh\nexit 7\n");
    let output = command(&failed.args(&["check", "fast", "--json"]));
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let gate = report["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|gate| gate["id"] == "fast.kernel")
        .unwrap();
    assert_eq!(gate["status"], "FAIL");
    assert_eq!(gate["subjects"].as_array().unwrap().len(), 1);

    let invalid = FamilyFixture::new("#!/bin/sh\nexit 0\n");
    write(
        &invalid.root.join("repos.manifest.toml"),
        "required_repos = [\"bullet-farm\", \"bullet-farm\"]\n",
    );
    let output = command(&invalid.args(&["check", "fast", "--json"]));
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["gates"][0]["id"], "catalog.family-layout");
    assert_eq!(report["gates"][0]["status"], "BLOCKED");
}

#[cfg(unix)]
#[test]
fn linked_git_metadata_is_blocked_before_execution() {
    use std::os::unix::fs::symlink;

    let fixture = FamilyFixture::new("#!/bin/sh\nexit 0\n");
    let repo = fixture.root.join("bullet-kernel");
    fs::rename(repo.join(".git"), repo.join(".git-real")).unwrap();
    symlink(".git-real", repo.join(".git")).unwrap();
    let output = command(&fixture.args(&["check", "fast", "--json"]));
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["gates"][0]["id"], "catalog.family-layout");
    assert_eq!(report["gates"][0]["status"], "BLOCKED");
}

#[test]
fn legacy_release_inventory_is_stable_sorted_and_blocked() {
    let registry =
        std::env::temp_dir().join(format!("bullet-legacy-registry-{}", std::process::id()));
    if registry.exists() {
        fs::remove_dir_all(&registry).unwrap();
    }
    fs::create_dir(&registry).unwrap();
    let registry = registry.to_str().unwrap();
    let args = [
        "check",
        "release",
        "--profile",
        "legacy-v1-26",
        "--receipts",
        registry,
        "--json",
    ];
    let first = command(&args);
    let second = command(&args);
    assert_eq!(first.status.code(), Some(3));
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["schema_version"], 3);
    assert_eq!(report["command"], "check");
    assert_eq!(report["tier"], "RELEASE");
    assert_eq!(report["profile"], "legacy-v1-26");
    assert_eq!(report["status"], "BLOCKED");
    let gates = report["gates"].as_array().unwrap();
    assert_eq!(gates.len(), 26);
    assert!(gates.iter().all(|gate| {
        gate["status"] == "BLOCKED"
            && gate["repair"]
                .as_str()
                .is_some_and(|repair| !repair.is_empty())
    }));
    assert!(
        gates
            .windows(2)
            .all(|pair| pair[0]["id"].as_str() < pair[1]["id"].as_str())
    );
    for required_v1_gate in [
        "release.provider.claude",
        "release.provider.codex",
        "release.provider.cursor",
        "release.provider.antigravity",
        "release.forge.jeryu",
        "release.forge.github-app",
        "release.package-matrix",
    ] {
        assert!(
            gates.iter().any(|gate| gate["id"] == required_v1_gate),
            "canonical V1 gate missing: {required_v1_gate}"
        );
    }
    let msrv = gates
        .iter()
        .find(|gate| gate["id"] == "release.rust-msrv-1-95")
        .unwrap();
    assert_eq!(msrv["status"], "BLOCKED");
    assert!(msrv["detail"].as_str().unwrap().contains("receipt"));

    let ignored_environment = Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .args(args)
        .env(
            "BULLET_RELEASE_EVIDENCE_ADMISSION",
            "/tmp/self-selected-policy",
        )
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert_eq!(ignored_environment.status.code(), Some(3));
    assert_eq!(ignored_environment.stdout, first.stdout);
    fs::remove_dir_all(registry).unwrap();
}

#[test]
fn profiled_release_rejects_ambiguous_or_relative_registry_arguments() {
    for args in [
        vec![
            "check",
            "release",
            "--profile",
            "linux-preview",
            "--receipts",
            "relative",
        ],
        vec![
            "check",
            "release",
            "--profile",
            "unknown-profile",
            "--receipts",
            "/tmp/receipts",
        ],
        vec![
            "check",
            "release",
            "--profile",
            "linux-preview",
            "--profile",
            "provider-codex",
            "--receipts",
            "/tmp/receipts",
        ],
    ] {
        let output = command(&args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn arguments_are_strict() {
    for args in [
        vec!["check"],
        vec!["check", "other"],
        vec!["check", "fast", "--yaml"],
        vec!["check", "fast", "--json", "--json"],
        vec!["check", "--json", "fast"],
    ] {
        let output = command(&args);
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stdout.is_empty(), "args={args:?}");
        assert!(String::from_utf8(output.stderr).unwrap().contains("USAGE"));
    }
}

#[test]
fn legacy_success_and_corrupt_schema_exit_codes_remain_available() {
    let success = command(&["hub", "check"]);
    assert_eq!(success.status.code(), Some(0));
    assert_eq!(success.stdout, b"hub-check: ok\n");

    let fixture = unsupported_lock_fixture();
    let unsupported = command(&[
        "--root",
        fixture.to_str().expect("UTF-8 fixture"),
        "checkout",
        "verify",
    ]);
    assert_eq!(unsupported.status.code(), Some(4));
    assert!(unsupported.stdout.is_empty());
    assert!(
        String::from_utf8(unsupported.stderr)
            .unwrap()
            .contains("UNSUPPORTED_SCHEMA")
    );
    fs::remove_dir_all(fixture).expect("remove fixture");
}
