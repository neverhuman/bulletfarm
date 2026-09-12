//! Both actual CLI aliases resolve explicit demo/init state without cwd defaults.
#![cfg(target_os = "linux")]

use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const ALIASES: [&str; 2] = [
    env!("CARGO_BIN_EXE_bullet"),
    env!("CARGO_BIN_EXE_bulletfarm"),
];

fn invoke(binary: &str, cwd: &Path, args: &[&str], environment: &[(&str, &Path)]) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "umask 002; exec \"$@\"", "demo-default-test"])
        .arg(binary)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir(cwd);
    for (name, value) in environment {
        command.env(name, value);
    }
    command.output().expect("execute actual CLI alias")
}

fn private_root() -> TempDir {
    let root = TempDir::new().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn private_home(root: &Path) -> std::path::PathBuf {
    let home = root.join("home");
    fs::create_dir(&home).unwrap();
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    home
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn private_ledger(directory: &Path) {
    assert_eq!(
        fs::metadata(directory).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert!(directory.join("ledger.sqlite").is_file());
}

fn component_demo(output: &Output, directory: &Path) {
    success(output);
    private_ledger(directory);
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("COMPONENT_ONLY: transaction_gate_eligible=false"));
    let receipt: Value =
        serde_json::from_slice(&fs::read(directory.join("receipts.json")).unwrap()).unwrap();
    assert_eq!(receipt["candidate_head"], "NOT_PRODUCED");
    assert_eq!(receipt["evidence_result"], "NOT_RUN");
    assert_eq!(receipt["effect_outcome"], "NOT_DISPATCHED");
    assert_eq!(receipt["effect_unknown_outcome"], "NOT_DISPATCHED");
    assert_eq!(receipt["stale_refused"], true);
    assert_eq!(receipt["materialize_idempotent"], true);
}

fn demo_default(binary: &str, use_xdg: bool) {
    let root = private_root();
    let home = private_home(root.path());
    let xdg = root.path().join("state");
    let mut environment = vec![("HOME", home.as_path())];
    let expected = if use_xdg {
        environment.push(("XDG_STATE_HOME", xdg.as_path()));
        xdg.join("bullet/ledger")
    } else {
        home.join(".local/state/bullet/ledger")
    };
    let output = invoke(binary, root.path(), &["demo"], &environment);
    component_demo(&output, &expected);
    assert!(!root.path().join("target").exists());
    if use_xdg {
        assert!(!home.join(".local").exists());
    }
}

#[test]
fn bullet_demo_uses_private_home_state() {
    demo_default(ALIASES[0], false);
}

#[test]
fn bullet_demo_prefers_private_xdg_state() {
    demo_default(ALIASES[0], true);
}

#[test]
fn bulletfarm_demo_uses_private_home_state() {
    demo_default(ALIASES[1], false);
}

#[test]
fn bulletfarm_demo_prefers_private_xdg_state() {
    demo_default(ALIASES[1], true);
}

#[test]
fn both_aliases_farm_init_uses_private_default_without_demo_receipt() {
    for binary in ALIASES {
        let root = private_root();
        let home = private_home(root.path());
        let output = invoke(binary, root.path(), &["farm", "init"], &[("HOME", &home)]);
        success(&output);
        let directory = home.join(".local/state/bullet/ledger");
        private_ledger(&directory);
        assert!(!directory.join("receipts.json").exists());
        assert!(!root.path().join("target").exists());
    }
}

fn demo_explicit(binary: &str, relative: bool) {
    let root = private_root();
    let home = private_home(root.path());
    let xdg = root.path().join("state");
    let expected = root.path().join("override");
    // Existing loose directories are made private before ledger creation.
    fs::create_dir(&expected).unwrap();
    fs::set_permissions(&expected, fs::Permissions::from_mode(0o775)).unwrap();
    let configured = if relative {
        Path::new("override")
    } else {
        expected.as_path()
    };
    let output = invoke(
        binary,
        root.path(),
        &["demo"],
        &[
            ("HOME", &home),
            ("XDG_STATE_HOME", &xdg),
            ("BULLET_DATA_DIR", configured),
        ],
    );
    component_demo(&output, &expected);
    assert!(!xdg.exists());
    assert!(!home.join(".local").exists());
    if relative {
        let root = private_root();
        let output = invoke(
            binary,
            root.path(),
            &["demo"],
            &[("BULLET_DATA_DIR", Path::new("./override"))],
        );
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("SQLite path contains an empty or non-normal component"));
        assert!(!root.path().join("override/ledger.sqlite").exists());
    }
}

#[test]
fn bullet_demo_preserves_absolute_override_precedence() {
    demo_explicit(ALIASES[0], false);
}

#[test]
fn bullet_demo_preserves_relative_override_admission() {
    demo_explicit(ALIASES[0], true);
}

#[test]
fn bulletfarm_demo_preserves_absolute_override_precedence() {
    demo_explicit(ALIASES[1], false);
}

#[test]
fn bulletfarm_demo_preserves_relative_override_admission() {
    demo_explicit(ALIASES[1], true);
}

#[test]
fn both_aliases_refuse_missing_empty_or_invalid_defaults_before_writes() {
    for binary in ALIASES {
        for args in [
            &["demo"][..],
            &["farm", "init"][..],
            &["demo-synthetic"][..],
        ] {
            let cases: Vec<(Vec<(&str, &Path)>, &str)> = vec![
                (vec![], "BULLET_DATA_DIR_DEFAULT_UNAVAILABLE"),
                (
                    vec![("BULLET_DATA_DIR", Path::new(""))],
                    "BULLET_DATA_DIR_INVALID",
                ),
                (
                    vec![("HOME", Path::new(""))],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
                (
                    vec![("HOME", Path::new("relative"))],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
                (
                    vec![("HOME", Path::new("/tmp/../tmp"))],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
                (
                    vec![("HOME", Path::new("/tmp/./state"))],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
                (
                    vec![
                        ("HOME", Path::new("/tmp")),
                        ("XDG_STATE_HOME", Path::new("")),
                    ],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
                (
                    vec![
                        ("HOME", Path::new("/tmp")),
                        ("XDG_STATE_HOME", Path::new("relative")),
                    ],
                    "BULLET_DATA_DIR_DEFAULT_INVALID",
                ),
            ];
            for (environment, reason) in cases {
                let root = private_root();
                let environment: Vec<_> = environment
                    .into_iter()
                    .map(|(name, value)| {
                        (
                            name,
                            if name == "HOME" && value == Path::new("/tmp") {
                                root.path()
                            } else {
                                value
                            },
                        )
                    })
                    .collect();
                let output = invoke(binary, root.path(), args, &environment);
                assert!(!output.status.success());
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains(reason),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(output.stdout.is_empty());
                assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
            }
        }
    }
}
