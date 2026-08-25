//! Fixture-only gitd authority. Production `Daemon::new()` still refuses clone.

use bullet_gitd::daemon::Daemon;
use serde_json::{json, Value};
#[cfg(feature = "fixture-authority")]
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "fixture-authority")]
use std::time::{SystemTime, UNIX_EPOCH};

const NONCE: [u8; 32] = [9u8; 32];
const ATTEMPT: &str = "atm_1111111111111111111111111111111111111111111111111111111111111111";
const VARIANT: &str = "var_2222222222222222222222222222222222222222222222222222222222222222";
#[cfg(feature = "fixture-authority")]
const FIXTURE_KEY: [u8; 32] = [0x5a; 32];

fn token() -> Value {
    json!({
        "organization_id": "org_fixture",
        "variant_id": VARIANT,
        "attempt_id": ATTEMPT,
        "attempt_fence": 1,
        "workspace_nonce": NONCE.to_vec(),
        "runner_epoch": 1,
    })
}

#[cfg(feature = "fixture-authority")]
fn now_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis(),
    )
    .expect("millis")
}

#[cfg(feature = "fixture-authority")]
fn fixture_git(home: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").expect("PATH"))
        .env("HOME", home)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_DATE", "2026-08-20T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-08-20T00:00:00+00:00")
        .args(args)
        .output()
        .expect("spawn fixture git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[cfg(feature = "fixture-authority")]
fn init_source(root: &Path) -> (PathBuf, String) {
    let home = root.join("fixture-home");
    std::fs::create_dir_all(&home).expect("home");
    let src = root.join("source");
    std::fs::create_dir_all(&src).expect("source");
    let src_str = src.to_string_lossy().into_owned();
    fixture_git(&home, &["init", "-q", "-b", "main", &src_str]);
    std::fs::create_dir_all(src.join("src")).expect("src");
    std::fs::write(src.join("src").join("lib.rs"), "pub fn seed() {}\n").expect("lib");
    fixture_git(&home, &["-C", &src_str, "add", "-A"]);
    fixture_git(
        &home,
        &[
            "-C",
            &src_str,
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@test.local",
            "commit",
            "-q",
            "-m",
            "init",
        ],
    );
    let hex = fixture_git(&home, &["-C", &src_str, "rev-parse", "HEAD"]);
    (src, format!("sha1:{hex}"))
}

fn handle(daemon: &mut Daemon, request: Value) -> Value {
    serde_json::from_str(&daemon.handle_line(&request.to_string())).expect("response json")
}

#[cfg(feature = "fixture-authority")]
fn private_dir(path: &Path) {
    std::fs::create_dir_all(path).expect("create");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).expect("0700");
    }
}

#[cfg(feature = "fixture-authority")]
fn private_tempdir() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private fixture root");
    }
    root
}

#[test]
fn production_constructor_still_refuses_clone() {
    let mut daemon = Daemon::new();
    let response = handle(
        &mut daemon,
        json!({
            "id": 1,
            "method": "clone",
            "token": token(),
            "params": {
                "source_repo": "/does/not/matter",
                "base_sha": format!("sha1:{}", "a".repeat(40)),
                "root": "/tmp/unused-gitd-production",
                "created_at": "2026-08-24T00:00:00Z",
                "allowed_prefixes": ["src"],
                "commit_date": "2026-08-24T00:00:00+00:00"
            }
        }),
    );
    assert_eq!(response["err"]["code"], "AUTHORITY_CONTRACT_UNAVAILABLE");
}

#[test]
fn production_binary_without_flag_still_refuses_clone() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut child = Command::new(env!("CARGO_BIN_EXE_bullet-gitd"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .env("HOME", temp.path())
        .spawn()
        .expect("spawn");
    let mut stdin = child.stdin.take().expect("stdin");
    let request = json!({
        "id": 1,
        "method": "clone",
        "token": token(),
        "params": {
            "source_repo": "/does/not/matter",
            "base_sha": format!("sha1:{}", "a".repeat(40)),
            "root": temp.path().join("farm").display().to_string(),
            "created_at": "2026-08-24T00:00:00Z",
            "allowed_prefixes": ["src"],
            "commit_date": "2026-08-24T00:00:00+00:00"
        }
    });
    use std::io::{BufRead, BufReader, Write};
    writeln!(stdin, "{request}").expect("write");
    stdin.flush().expect("flush");
    drop(stdin);
    let mut line = String::new();
    BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut line)
        .expect("read");
    let response: Value = serde_json::from_str(&line).expect("json");
    assert_eq!(response["err"]["code"], "AUTHORITY_CONTRACT_UNAVAILABLE");
    let status = child.wait().expect("exit");
    assert!(status.success(), "{status:?}");
}

#[test]
fn production_binary_rejects_fixture_flag() {
    let out = Command::new(env!("CARGO_BIN_EXE_bullet-gitd"))
        .arg("--fixture-authority")
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown argument"));
}

#[cfg(feature = "fixture-authority")]
fn signed_token(fixture_root: &Path) -> Value {
    use bullet_gitd::fixture_permit::{mint_fixture_permit, FixturePermitClaims};
    let claims = FixturePermitClaims {
        schema_version: "v1".into(),
        attempt_id: ATTEMPT.into(),
        attempt_fence: 1,
        workspace_nonce: hex::encode(NONCE),
        workspace_generation: 1,
        fixture_root: fixture_root.display().to_string(),
        expires_at_unix_ms: now_ms().saturating_add(5_000),
    };
    let mut token = token();
    token["fixture_permit"] =
        serde_json::to_value(mint_fixture_permit(&FIXTURE_KEY, claims)).expect("permit json");
    token
}

#[cfg(feature = "fixture-authority")]
fn clone_params(source: &Path, base: &str, root: &Path) -> Value {
    json!({
        "source_repo": source.display().to_string(),
        "base_sha": base,
        "root": root.display().to_string(),
        "created_at": "2026-08-24T00:00:00Z",
        "allowed_prefixes": ["src"],
        "commit_date": "2026-08-24T00:00:00+00:00"
    })
}

#[cfg(feature = "fixture-authority")]
#[test]
fn unsigned_legacy_token_is_refused() {
    let temp = private_tempdir();
    let root = temp.path().join("farm");
    private_dir(&root);
    let ledger = temp.path().join("mutation-ledger");
    let mut daemon = Daemon::fixture(&ledger, &root, FIXTURE_KEY).expect("fixture daemon");
    let (source, base) = init_source(temp.path());
    let response = handle(
        &mut daemon,
        json!({
            "id": 1,
            "method": "clone",
            "token": token(),
            "params": clone_params(&source, &base, &root)
        }),
    );
    assert_eq!(response["err"]["code"], "AUTHORITY_REFUSED", "{response}");
}

#[cfg(feature = "fixture-authority")]
#[test]
fn arbitrary_destination_is_refused() {
    let temp = private_tempdir();
    let root = temp.path().join("farm");
    private_dir(&root);
    let other = temp.path().join("other");
    private_dir(&other);
    let ledger = temp.path().join("mutation-ledger");
    let mut daemon = Daemon::fixture(&ledger, &root, FIXTURE_KEY).expect("fixture daemon");
    let (source, base) = init_source(temp.path());
    let response = handle(
        &mut daemon,
        json!({
            "id": 1,
            "method": "clone",
            "token": signed_token(&root),
            "params": clone_params(&source, &base, &other)
        }),
    );
    assert_eq!(
        response["err"]["code"], "FIXTURE_DESTINATION_REFUSED",
        "{response}"
    );
}

#[cfg(feature = "fixture-authority")]
#[test]
fn missing_preopened_root_is_refused() {
    let temp = private_tempdir();
    let missing = temp.path().join("missing");
    let ledger = temp.path().join("mutation-ledger");
    let error = match Daemon::fixture(&ledger, &missing, FIXTURE_KEY) {
        Ok(_) => panic!("missing root was accepted"),
        Err(error) => error,
    };
    assert!(error.contains("already exist"), "{error}");
}

#[cfg(feature = "fixture-authority")]
#[test]
fn fixture_daemon_clones_applies_and_checkpoints() {
    let temp = private_tempdir();
    let (source, base) = init_source(temp.path());
    let ledger = temp.path().join("mutation-ledger");
    let root = temp.path().join("farm");
    private_dir(&root);
    let mut daemon = Daemon::fixture(&ledger, &root, FIXTURE_KEY).expect("fixture daemon");
    let cloned = handle(
        &mut daemon,
        json!({
            "id": 1,
            "method": "clone",
            "token": signed_token(&root),
            "params": clone_params(&source, &base, &root)
        }),
    );
    assert!(cloned.get("ok").is_some(), "{cloned}");
    let applied = handle(
        &mut daemon,
        json!({
            "id": 2,
            "method": "apply_change",
            "token": signed_token(&root),
            "params": {
                "patches": [{
                    "path": "src/lib.rs",
                    "contents_hex": hex::encode("pub fn demo() {}\n")
                }]
            }
        }),
    );
    assert_eq!(applied["ok"]["applied"], 1, "{applied}");
    let checkpoint = handle(
        &mut daemon,
        json!({
            "id": 3,
            "method": "checkpoint",
            "token": signed_token(&root),
            "params": {}
        }),
    );
    assert!(checkpoint["ok"]["id"].is_string(), "{checkpoint}");
}

#[cfg(feature = "fixture-authority")]
#[test]
fn second_generation_on_the_same_root_is_refused() {
    let temp = private_tempdir();
    let (source, base) = init_source(temp.path());
    let ledger = temp.path().join("mutation-ledger");
    let root = temp.path().join("farm");
    private_dir(&root);
    let mut first = Daemon::fixture(&ledger, &root, FIXTURE_KEY).expect("first");
    let cloned = handle(
        &mut first,
        json!({
            "id": 1,
            "method": "clone",
            "token": signed_token(&root),
            "params": clone_params(&source, &base, &root)
        }),
    );
    assert!(cloned.get("ok").is_some(), "{cloned}");
    drop(first);
    let mut second = Daemon::fixture(&ledger, &root, FIXTURE_KEY).expect("second");
    let refused = handle(
        &mut second,
        json!({
            "id": 1,
            "method": "clone",
            "token": signed_token(&root),
            "params": clone_params(&source, &base, &root)
        }),
    );
    assert_eq!(
        refused["err"]["code"], "FIXTURE_GENERATION_CONSUMED",
        "{refused}"
    );
}
