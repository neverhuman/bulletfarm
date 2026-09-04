//! Phase R engineering-half proofs: DF-R4 inventory is covered by
//! `canonical_hostile`; these tests cover DF-R5 native-evidence honesty and
//! DF-R7 rehearsal/compare fail-closed behavior.

use std::{fs, path::PathBuf, process::Command};

fn run_script(script: &str, args: &[&str]) -> std::process::Output {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join(script);
    assert!(path.is_file(), "expected {}", path.display());
    Command::new("bash")
        .arg(&path)
        .args(args)
        .current_dir(&manifest_dir)
        .output()
        .unwrap_or_else(|error| panic!("failed to spawn {}: {error}", path.display()))
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn r5_linux_refuses_native_macos_windows_evidence() {
    let output = run_script("scripts/platform-native-evidence.sh", &["--self-test"]);
    assert!(
        !output.status.success(),
        "Linux must not admit native macos/windows evidence: {}",
        stdout(&output)
    );
    let err = stderr(&output);
    assert!(
        err.contains("NATIVE_PLATFORM_EVIDENCE_UNAVAILABLE"),
        "missing typed refusal: {err}"
    );
}

#[test]
fn r7a_rehearsal_self_test_stays_component_and_unsigned() {
    let output = run_script("scripts/recovery-rehearsal.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "rehearsal self-test failed: stdout={}\nstderr={}",
        stdout(&output),
        stderr(&output)
    );
    let text = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        text.contains("live incident remains unauthorized"),
        "rehearsal must keep the live incident unauthorized: {text}"
    );
}

#[test]
fn wave1_schema3_lock_generate_stays_refused_without_od_d_e() {
    let output = run_script("scripts/od-lock-generate-refuse.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "od-lock self-test failed: {}",
        stderr(&output)
    );
}

#[test]
fn first_ga_and_later_profiles_remain_blocked() {
    let output = run_script("scripts/check-release-still-blocked.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "release-blocked self-test failed: {}",
        stderr(&output)
    );
}

#[test]
fn wave0_self_test_sees_four_checkouts_without_claiming_clean_heads() {
    let output = run_script("scripts/wave0-family-observation.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "wave0 self-test failed: {}",
        stderr(&output)
    );
}

#[test]
fn dog0_self_test_keeps_frozen_ledger_unauthorized() {
    let family = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let coord = family.join(".bullet-family/coord");
    let events = coord.join("events.jsonl");
    let before = fs::read(&events).unwrap();
    assert!(!coord.join("CURRENT").exists());

    let output = run_script("scripts/dogfood-coord-loop.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "dog0 self-test failed: {}",
        stderr(&output)
    );
    let out = stdout(&output);
    assert!(
        out.contains(concat!(
            "COORD_RECOVERY_REQUIRED and COORD_RECOVERY_IN_PROGRESS ",
            "map to COORD_FROZEN; no coord command executed"
        )),
        "missing exact recovery refusal mapping: {out}"
    );
    assert!(
        !out.contains("COORD_STATUS_FAILED"),
        "generic refusal leaked: {out}"
    );
    assert_eq!(fs::read(events).unwrap(), before);
    assert!(!coord.join("CURRENT").exists());
}

#[test]
fn r7b_compare_refuses_live_execution_and_missing_approval() {
    let output = run_script("scripts/recovery-incident-compare.sh", &["--self-test"]);
    assert!(
        output.status.success(),
        "incident-compare self-test failed: stdout={}\nstderr={}",
        stdout(&output),
        stderr(&output)
    );
    let text = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        text.contains("execution remains forbidden"),
        "DF-R7b must remain fail-closed: {text}"
    );
}
