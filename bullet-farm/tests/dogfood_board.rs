//! Diagnostic dogfood board: the Python entry point transparently delegates to Rust.

use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn hub() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn decode_board(stdout: &[u8]) -> Value {
    bullet_wire::decode_unique_value(stdout).expect("unique board json")
}

fn assert_diagnostic_board(stdout: &[u8]) {
    let text = std::str::from_utf8(stdout).expect("utf-8 board");
    assert!(
        !text.contains("LIVE_PROOF"),
        "board must never mention LIVE_PROOF"
    );
    let board = decode_board(stdout);
    assert_eq!(board["authoritative"], false);
    assert_eq!(board["kind"], "DIAGNOSTIC");
    assert_eq!(board["schema_version"], 1);
    assert_eq!(board["scorecard"]["authoritative"], false);
    assert!(board["scorecard"]["blended"].is_number());
    assert_eq!(board["release"]["profile"], "self-hosted-v1");
    assert_eq!(board["release"]["status"], "BLOCKED");
    assert!(board["leftover_allowlist"].is_array());
    assert!(board["next_free_lanes"].is_array());
    assert_eq!(board["loop_operable"], false);
    assert!(board["loop_blockers"].is_array());
    let blockers = board["loop_blockers"]
        .as_array()
        .expect("loop blockers must be an array");
    assert!(
        blockers.iter().any(|value| value == "COORD_UNAVAILABLE"
            || value == "WAVE0_DIRTY_SUBJECTS"
            || value == "DOGFOOD_POLICY_MISSING"),
        "broken loop must name a typed blocker: {blockers:?}"
    );

    let available = board["coord"]["available"]
        .as_bool()
        .expect("coordinator availability must be typed");
    assert!(board["coord"]["active"].is_u64());
    assert!(board["coord"]["handed_off_uncommitted"].is_u64());
    assert!(board["coord"]["paths"].is_array());
    if available {
        assert!(
            board["coord"].get("error").is_none(),
            "available coordinator must not carry an error"
        );
    } else {
        assert_eq!(board["coord"]["active"], 0);
        assert_eq!(board["coord"]["handed_off_uncommitted"], 0);
        assert_eq!(board["coord"]["paths"].as_array().map(Vec::len), Some(0));
        assert_eq!(board["next_free_lanes"].as_array().map(Vec::len), Some(0));
        let error = board["coord"]["error"]
            .as_str()
            .expect("unavailable coordinator must carry a typed code");
        assert!(!error.is_empty());
        assert!(
            error
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_'),
            "coordinator error must be a stable typed code: {error:?}"
        );
    }
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    Command::new("/bin/sh")
        .args(["-c", "kill -0 \"$1\" 2>/dev/null", "sh", &pid.to_string()])
        .status()
        .expect("inspect launcher process")
        .success()
}

#[cfg(unix)]
fn wait_for_processes_to_exit(pids: &[u32]) -> bool {
    for _ in 0..200 {
        if pids.iter().all(|pid| !process_is_alive(*pid)) {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[cfg(unix)]
fn kill_process(pid: u32) {
    let _ = Command::new("/bin/sh")
        .args([
            "-c",
            "kill -KILL \"$1\" 2>/dev/null || true",
            "sh",
            &pid.to_string(),
        ])
        .status();
}

#[test]
fn board_tracks_fail_for_their_own_reasons_only() {
    // The M0.2 split: the coord track never mentions the dogfood binding, the
    // dogfood track never mentions the coordinator, and `all` is their union.
    let bin = env!("CARGO_BIN_EXE_bullet-family");
    let run = |track: &str| {
        let output = Command::new(bin)
            .args(["check", "dogfood", "--json", "--track", track])
            .current_dir(hub())
            .env_remove("BULLET_DOGFOOD_BINDING")
            .env_remove("BULLET_DOGFOOD_POLICY")
            .output()
            .expect("run board");
        let board = decode_board(&output.stdout);
        let blockers: Vec<String> = board["loop_blockers"]
            .as_array()
            .expect("blockers array")
            .iter()
            .map(|value| value.as_str().unwrap_or_default().to_owned())
            .collect();
        (
            board["track"].as_str().unwrap_or_default().to_owned(),
            blockers,
        )
    };

    let (track, coord_blockers) = run("coord");
    assert_eq!(track, "coord");
    assert!(
        !coord_blockers
            .iter()
            .any(|blocker| blocker.starts_with("DOGFOOD_")),
        "coord track must not report dogfood blockers: {coord_blockers:?}"
    );

    let (track, dogfood_blockers) = run("dogfood");
    assert_eq!(track, "dogfood");
    assert!(
        dogfood_blockers
            .iter()
            .all(|blocker| blocker.starts_with("DOGFOOD_")),
        "dogfood track must report only dogfood blockers: {dogfood_blockers:?}"
    );
    assert!(
        dogfood_blockers
            .iter()
            .any(|code| code == "DOGFOOD_POLICY_MISSING")
    );
    assert!(
        dogfood_blockers
            .iter()
            .any(|code| code == "DOGFOOD_BINDING_MISSING")
    );
    assert!(
        dogfood_blockers
            .iter()
            .any(|code| code == "DOGFOOD_RUNTIME_CHECK_UNAVAILABLE")
    );

    let (track, all_blockers) = run("all");
    assert_eq!(track, "all");
    for blocker in coord_blockers.iter().chain(dogfood_blockers.iter()) {
        assert!(
            all_blockers.contains(blocker),
            "all-track must be the union; missing {blocker}"
        );
    }

    // An unknown track is refused, never defaulted.
    let output = Command::new(bin)
        .args(["check", "dogfood", "--track", "release"])
        .current_dir(hub())
        .output()
        .expect("run board");
    assert!(!output.status.success(), "unknown track must refuse");
}

#[cfg(unix)]
#[test]
fn compatibility_launcher_forwards_exact_rust_result() {
    let temp = tempfile::tempdir().expect("launcher relay fixture");
    let relay = temp.path().join("bullet-family-relay");
    let rust_stdout = temp.path().join("rust.stdout");
    let rust_stderr = temp.path().join("rust.stderr");
    let rust_status = temp.path().join("rust.status");
    fs::write(
        &relay,
        r#"#!/bin/sh
if [ "$#" -ne 3 ] || [ "$1" != check ] || [ "$2" != dogfood ] || [ "$3" != --json ]; then
    printf '%s\n' DOGFOOD_TEST_ARGV_MISMATCH >&2
    exit 97
fi
"$BULLET_TEST_REAL_BIN" "$@" >"$BULLET_TEST_RUST_STDOUT" 2>"$BULLET_TEST_RUST_STDERR"
status=$?
printf '%s\n' "$status" >"$BULLET_TEST_RUST_STATUS"
cat -- "$BULLET_TEST_RUST_STDOUT"
cat -- "$BULLET_TEST_RUST_STDERR" >&2
exit "$status"
"#,
    )
    .expect("write launcher relay");
    fs::set_permissions(&relay, fs::Permissions::from_mode(0o755))
        .expect("make launcher relay executable");

    let output = Command::new("python3")
        .arg(hub().join("tests/dogfood-board.py"))
        .arg("--json")
        .env("BULLET_FAMILY_BIN", &relay)
        .env("BULLET_TEST_REAL_BIN", env!("CARGO_BIN_EXE_bullet-family"))
        .env("BULLET_TEST_RUST_STDOUT", &rust_stdout)
        .env("BULLET_TEST_RUST_STDERR", &rust_stderr)
        .env("BULLET_TEST_RUST_STATUS", &rust_status)
        .env_remove("HOME")
        .current_dir(hub())
        .output()
        .expect("run compatibility launcher");

    assert_eq!(fs::read(&rust_status).expect("read Rust status"), b"1\n");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        fs::read(&rust_stdout).expect("read exact Rust stdout")
    );
    assert_eq!(
        output.stderr,
        fs::read(&rust_stderr).expect("read exact Rust stderr")
    );
    assert_diagnostic_board(&output.stdout);
}

#[cfg(unix)]
#[test]
fn compatibility_launcher_timeout_reaps_its_term_resistant_process_group() {
    let temp = tempfile::tempdir().expect("launcher timeout fixture");
    let relay = temp.path().join("bullet-family-relay");
    let leader_pid = temp.path().join("leader.pid");
    let descendant_pid = temp.path().join("descendant.pid");
    fs::write(
        &relay,
        r#"#!/bin/sh
if [ "$#" -ne 3 ] || [ "$1" != check ] || [ "$2" != dogfood ] || [ "$3" != --json ]; then
    exit 97
fi
trap '' TERM
printf '%s\n' "$$" >"$BULLET_TEST_LEADER_PID"
/bin/sh -c '
    trap "" TERM
    printf "%s\n" "$$" >"$BULLET_TEST_DESCENDANT_PID"
    while :; do /bin/sleep 60; done
' </dev/null >/dev/null 2>&1 &
while [ ! -s "$BULLET_TEST_DESCENDANT_PID" ]; do :; done
wait
"#,
    )
    .expect("write timeout relay");
    fs::set_permissions(&relay, fs::Permissions::from_mode(0o755))
        .expect("make timeout relay executable");

    let driver = r#"
import importlib.util
import sys

path = sys.argv[1]
spec = importlib.util.spec_from_file_location("bullet_dogfood_board_test", path)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)
module.TIMEOUT_SECONDS = 0.25
sys.argv = [path, "--json"]
raise SystemExit(module.main())
"#;
    let started = Instant::now();
    let output = Command::new("python3")
        .args(["-c", driver])
        .arg(hub().join("tests/dogfood-board.py"))
        .env("BULLET_FAMILY_BIN", &relay)
        .env("BULLET_TEST_LEADER_PID", &leader_pid)
        .env("BULLET_TEST_DESCENDANT_PID", &descendant_pid)
        .env_remove("HOME")
        .current_dir(hub())
        .output()
        .expect("run timeout-bounded compatibility launcher");
    let elapsed = started.elapsed();

    let read_pid = |path: &std::path::Path| {
        let text = fs::read_to_string(path).expect("read launcher pid");
        let digits = text.trim().as_bytes();
        assert!(
            !digits.is_empty() && digits.len() <= 10 && digits.iter().all(u8::is_ascii_digit),
            "launcher pid is not bounded ASCII decimal"
        );
        digits
            .iter()
            .copied()
            .try_fold(0_u32, |value, digit| {
                value.checked_mul(10)?.checked_add(u32::from(digit - b'0'))
            })
            .expect("launcher pid overflow")
    };
    let pids = [read_pid(&leader_pid), read_pid(&descendant_pid)];
    let processes_gone = wait_for_processes_to_exit(&pids);
    if !processes_gone {
        for pid in pids {
            kill_process(pid);
        }
    }

    assert_eq!(output.status.code(), Some(124));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"DOGFOOD_BOARD_TIMEOUT\n");
    assert!(elapsed < Duration::from_secs(3), "timeout took {elapsed:?}");
    assert!(processes_gone, "launcher left a process alive: {pids:?}");
}

#[test]
fn compatibility_launcher_rejects_unbounded_arguments_before_launch() {
    let output = Command::new("python3")
        .arg(hub().join("tests/dogfood-board.py"))
        .args(["--json", "--json"])
        .env("BULLET_FAMILY_BIN", "/definitely/not/a/bullet-family")
        .current_dir(hub())
        .output()
        .expect("run compatibility launcher argument refusal");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("utf-8 usage refusal"),
        "usage: dogfood-board.py [--json] [--self-check]\n\
         compatibility launcher for: bullet-family check dogfood --json\n"
    );
}

fn configured_board(binding: &std::path::Path, policy: Option<&std::path::Path>) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bullet-family"));
    command
        .args(["check", "dogfood", "--json", "--track", "dogfood"])
        .current_dir(hub())
        .env("BULLET_DOGFOOD_BINDING", binding)
        .env_remove("BULLET_DOGFOOD_POLICY");
    if let Some(policy) = policy {
        command.env("BULLET_DOGFOOD_POLICY", policy);
    }
    let output = command.output().expect("run configured board");
    assert_eq!(
        output.status.code(),
        Some(1),
        "configuration alone must never pass"
    );
    decode_board(&output.stdout)
}

#[test]
fn binding_alone_cannot_claim_a_usable_three_provider_loop() {
    let temp = tempfile::tempdir().unwrap();
    let binding = temp.path().join("binding.json");
    fs::write(&binding, br#"{"audience":"dogfood-runner","operation":"read-only-propose","schema_version":"v1alpha1"}"#).unwrap();
    let board = configured_board(&binding, None);
    assert_eq!(board["configuration"]["binding"], "VALIDATED");
    assert_eq!(board["configuration"]["policy"], "MISSING");
    assert_eq!(board["configuration"]["ready"], false);
    assert_eq!(board["operation"]["status"], "UNVERIFIED");
    assert_eq!(board["release"]["status"], "BLOCKED");
    let providers = board["configuration"]["providers"].as_array().unwrap();
    assert_eq!(
        providers
            .iter()
            .map(|p| p["provider"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["codex", "claude", "cursor"]
    );
    for provider in providers {
        assert_eq!(provider["required"], true);
        for check in ["enrollment", "runtime", "conformance"] {
            assert_eq!(provider[check], "UNVERIFIED");
        }
    }
    assert!(
        board["loop_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "DOGFOOD_POLICY_MISSING")
    );
}

#[test]
fn offline_policy_validation_does_not_substitute_for_runtime_readiness() {
    use bullet_wire::{KeyPurposeV1, PolicySnapshotV1};
    let temp = tempfile::tempdir().unwrap();
    let binding = temp.path().join("binding.json");
    fs::write(&binding, br#"{"audience":"dogfood-runner","operation":"read-only-propose","schema_version":"v1alpha1"}"#).unwrap();
    let mut policy: PolicySnapshotV1 = bullet_wire::decode_canonical(
        &fs::read(
            hub().join("crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json"),
        )
        .unwrap(),
    )
    .unwrap();
    policy.sandbox_policy.live_admission_enabled = false;
    let mut key = policy
        .issuer_keys
        .iter()
        .find(|k| k.key_purpose == KeyPurposeV1::AuthoritySigning)
        .unwrap()
        .clone();
    key.issuer = "fixture-dogfood".into();
    key.key_id = "fixture-launch".into();
    key.key_purpose = KeyPurposeV1::DogfoodLaunchSigning;
    key.public_key = "11".repeat(32);
    key.audiences.clear();
    policy.issuer_keys.push(key);
    let path = temp.path().join("policy.json");
    fs::write(&path, bullet_wire::canonical_json(&policy).unwrap()).unwrap();
    let board = configured_board(&binding, Some(&path));
    assert_eq!(board["configuration"]["policy"], "VALIDATED");
    assert_eq!(board["configuration"]["ready"], false);
    for blocker in [
        "DOGFOOD_ENROLLMENT_CHECK_UNAVAILABLE",
        "DOGFOOD_RUNTIME_CHECK_UNAVAILABLE",
        "DOGFOOD_CONFORMANCE_CHECK_UNAVAILABLE",
        "DOGFOOD_CONTAINMENT_CHECK_UNAVAILABLE",
        "DOGFOOD_DURABLE_LAUNCH_CHECK_UNAVAILABLE",
        "DOGFOOD_OPERATION_CHECK_UNAVAILABLE",
    ] {
        assert!(
            board["loop_blockers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|b| b == blocker)
        );
    }
    policy.sandbox_policy.live_admission_enabled = true;
    fs::write(&path, bullet_wire::canonical_json(&policy).unwrap()).unwrap();
    assert_eq!(
        configured_board(&binding, Some(&path))["configuration"]["policy"],
        "INVALID"
    );
    fs::write(&path, b"{}").unwrap();
    assert_eq!(
        configured_board(&binding, Some(&path))["configuration"]["policy"],
        "INVALID"
    );
    fs::remove_file(&path).unwrap();
    assert_eq!(
        configured_board(&binding, Some(&path))["configuration"]["policy"],
        "MISSING"
    );
}

#[test]
fn hostile_binding_inputs_are_invalid_and_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("binding.json");
    for bytes in [b"{}".to_vec(), br#"{"audience":"dogfood-runner","audience":"dogfood-runner","operation":"read-only-propose","schema_version":"v1alpha1"}"#.to_vec(), vec![b' '; bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES + 1]] {
        fs::write(&path, bytes).unwrap();
        assert_eq!(configured_board(&path, None)["configuration"]["binding"], "INVALID");
    }
    assert_eq!(
        configured_board(temp.path(), None)["configuration"]["binding"],
        "INVALID"
    );
    #[cfg(unix)]
    {
        let link = temp.path().join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert_eq!(
            configured_board(&link, None)["configuration"]["binding"],
            "INVALID"
        );
        let fifo = temp.path().join("fifo");
        nix::unistd::mkfifo(&fifo, nix::sys::stat::Mode::S_IRUSR).unwrap();
        assert_eq!(
            configured_board(&fifo, None)["configuration"]["binding"],
            "INVALID"
        );
    }
}
