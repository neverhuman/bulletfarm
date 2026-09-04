//! Typed Wave-0 assurance-inventory proof.
//!
//! The generator consumes machine JSON plus real BLOCKED release reports.
//! Markdown is output only and never status or release authority.

use std::{path::PathBuf, process::Command};

fn run_script(argument: &str) -> std::process::Output {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join("scripts/orphan-inventory.sh");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bullet-family"));
    assert!(script.is_file(), "expected {} to exist", script.display());
    assert!(binary.is_file(), "expected {} to exist", binary.display());
    Command::new("bash")
        .arg(&script)
        .arg(argument)
        .env("BULLET_FAMILY_BIN", &binary)
        .current_dir(&manifest_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to spawn {}: {error}", script.display()))
}

#[test]
fn typed_orphan_inventory_and_hostiles_pass() {
    let check = run_script("check");
    assert!(
        check.status.success(),
        "typed inventory check failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(
        String::from_utf8_lossy(&check.stdout).contains("typed graph and double rendering passed"),
        "typed inventory check did not report its exact boundary"
    );

    let hostile = run_script("--self-test");
    assert!(
        hostile.status.success(),
        "typed inventory self-test failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&hostile.stdout),
        String::from_utf8_lossy(&hostile.stderr)
    );
    let output = String::from_utf8_lossy(&hostile.stdout);
    for expected in [
        "missing G5",
        "one-way gap/wave edge",
        "unknown evidence label",
        "duplicate corpus unit",
        "enforced invariant without proof",
        "missing profile report",
        "zero-gate profile partition",
        "runtime gate class drift",
        "unknown receipt kind",
        "13/13 typed hostile mutations",
    ] {
        assert!(
            output.contains(expected),
            "self-test output lacks {expected:?}: {output}"
        );
    }
}
