//! Fail-closed acceptance for the quarantined public release builder.
//!
//! Internal archive-building components remain available for isolated tests,
//! but this public surface cannot safely run them until exact source and tool
//! reconstruction executes under a different identity.

#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use bullet_family::release;

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn build(args: &[&str]) -> bullet_family::coord::CoordError {
    let args = std::iter::once("build")
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    release::run(&args).expect_err("release build refuses this input")
}

fn git(repository: &Path, args: &[&str]) {
    let status = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repository)
        .args([
            "-c",
            "user.name=Release Build Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

fn git_stdout(repository: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {args:?}");
    output.stdout
}

/// A single-member family whose only repository is a real ordinary checkout.
fn family(root: &Path) {
    fs::write(
        root.join("repos.manifest.toml"),
        "schema_version = \"1.2.0\"\nfamily = \"bullet-farm\"\nrequired_repos = [\"bullet-farm\"]\n",
    )
    .expect("manifest");
    let member = root.join("bullet-farm");
    fs::create_dir(&member).expect("member");
    fs::write(member.join("family.lock"), FIXTURE_LOCK).expect("lock");
    git(&member, &["init", "--quiet"]);
    git(&member, &["add", "--all"]);
    git(&member, &["commit", "--quiet", "--message", "fixture"]);
}

const FIXTURE_LOCK: &str = concat!(
    "schema_version = \"2\"\nfamily = \"bullet-farm\"\ntag = \"v0.1.0-alpha.4\"\n",
    "schema_bundle_hash = \"blake3:00\"\n\n[[member]]\nname = \"bullet-farm\"\n",
    "tag = \"v0.1.0-alpha.4\"\ncommit_oid = \"0000000000000000000000000000000000000000\"\n",
    "schema_bundle_hash = \"blake3:00\"\n",
    "release_signing_identity = \"bot@jekko.ai|ed25519|SHA256:+FbqtZF+hPgrjJRh5Oq5gNUKUmtCykur2ZEUKAFRv+Y\"\n",
    "generated_client_hash = \"blake3:00\"\n",
);

#[test]
fn every_public_release_target_is_refused_before_target_admission() {
    let out = tempfile::tempdir().expect("out");
    for target in [
        TARGET,
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-musl",
        "",
    ] {
        let error = build(&[
            "--target",
            target,
            "--out",
            out.path().join("bundle").to_str().expect("path"),
        ]);
        assert_eq!(
            error.code(),
            "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE",
            "{target}"
        );
        assert!(
            error
                .to_string()
                .contains("private exact-OID reconstruction")
                && error.to_string().contains("sealed toolchain")
                && error.to_string().contains("different-identity"),
            "{target}: {error}"
        );
    }
}

#[test]
fn containment_refusal_precedes_output_path_admission() {
    let out = tempfile::tempdir().expect("out");
    let existing = out.path().join("already-here");
    fs::create_dir(&existing).expect("existing");
    assert_eq!(
        build(&["--target", TARGET, "--out", "relative/bundle",]).code(),
        "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE"
    );
    assert_eq!(
        build(&[
            "--target",
            TARGET,
            "--out",
            existing.to_str().expect("path"),
        ])
        .code(),
        "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE"
    );
}

#[test]
fn public_build_refuses_without_output_or_source_mutation() {
    let root = tempfile::tempdir().expect("family");
    let root = root.path().canonicalize().expect("canonical family");
    family(&root);
    let member = root.join("bullet-farm");
    let before_head = git_stdout(&member, &["rev-parse", "HEAD"]);
    let before_status = git_stdout(
        &member,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    );
    assert!(before_status.is_empty(), "fixture begins clean");
    let out = tempfile::tempdir().expect("out");
    let bundle = out.path().join("bundle");
    let error = build(&[
        "--target",
        TARGET,
        "--out",
        bundle.to_str().expect("path"),
        "--family-root",
        root.to_str().expect("path"),
    ]);
    assert_eq!(error.code(), "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE");
    assert!(
        !bundle.exists(),
        "a refused build must not create its output directory"
    );
    assert_eq!(git_stdout(&member, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(
        git_stdout(
            &member,
            &["status", "--porcelain=v1", "--untracked-files=all"],
        ),
        before_status,
        "a refused build must not mutate tracked or untracked source state"
    );
}

#[test]
fn containment_refusal_precedes_argument_validation() {
    for args in [
        vec![],
        vec!["--target", TARGET],
        vec!["--out", "/absolute/bundle"],
        vec!["--target", TARGET, "--out"],
    ] {
        assert_eq!(
            build(&args).code(),
            "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE",
            "{args:?}"
        );
    }
    assert_eq!(
        build(&[
            "--target",
            TARGET,
            "--out",
            "/absolute/bundle",
            "--offline",
            "--offline",
        ])
        .code(),
        "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE"
    );
    assert_eq!(
        build(&[
            "--target",
            TARGET,
            "--out",
            "/absolute/bundle",
            "--unknown",
            "x",
        ])
        .code(),
        "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE"
    );
}
