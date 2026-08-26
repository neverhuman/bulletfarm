use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bullet-doctor-{name}-{}", std::process::id()));
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
            "required_repos = [\"bullet-farm\", \"bullet-kernel\"]\n",
        ),
    )
    .expect("hub manifest fixture");
    fs::write(hub.join("family.lock"), legacy_lock("")).expect("lock fixture");
    root
}

fn legacy_lock(extra: &str) -> String {
    format!(
        concat!(
            "schema_version = \"2\"\n",
            "family = \"bullet-farm\"\n",
            "tag = \"v0.1.0-alpha.4\"\n",
            "schema_bundle_hash = \"blake3:{digest}\"\n",
            "{extra}",
            "[[member]]\n",
            "name = \"bullet-farm\"\n",
            "tag = \"v0.1.0-alpha.4\"\n",
            "commit_oid = \"{commit}\"\n",
            "schema_bundle_hash = \"blake3:{digest}\"\n",
            "release_signing_identity = \"bot@invalid.example|ed25519|SHA256:fixture\"\n",
            "generated_client_hash = \"blake3:{digest}\"\n",
            "[[member]]\n",
            "name = \"bullet-kernel\"\n",
            "tag = \"v0.1.0-alpha.4\"\n",
            "commit_oid = \"{commit}\"\n",
            "schema_bundle_hash = \"blake3:{digest}\"\n",
            "release_signing_identity = \"bot@invalid.example|ed25519|SHA256:fixture\"\n",
            "generated_client_hash = \"blake3:{digest}\"\n",
        ),
        digest = "a".repeat(64),
        commit = "b".repeat(40),
        extra = extra,
    )
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).expect("read fixture") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).expect("relative").to_path_buf(),
                    fs::read(path).expect("file bytes"),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn hub_only_doctor_is_json_read_only_and_honest() {
    let root = fixture_root("hub-only");
    let hub = root.join("bullet-farm");
    let before = snapshot(&root);
    let output = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        Ok(hub),
    )
    .expect("doctor report");
    let report: serde_json::Value = serde_json::from_str(&output).expect("JSON report");
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["status"], "BLOCKED");
    let checks = report["checks"].as_array().expect("checks");
    for expected in ["hub_checkout", "source_metadata", "family_layout"] {
        let check = checks
            .iter()
            .find(|check| check["id"] == expected)
            .unwrap_or_else(|| panic!("missing {expected}"));
        assert_eq!(check["status"], "BLOCKED");
        assert!(
            check["repair"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
    }
    assert_eq!(
        snapshot(&root),
        before,
        "doctor modified the hub-only clone"
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn doctor_rejects_non_json_and_invalid_roots() {
    let root = fixture_root("arguments");
    let hub = root.join("bullet-farm");
    let usage = bullet_family::cli::run(
        [OsString::from("bullet-family"), OsString::from("doctor")],
        Ok(hub.clone()),
    )
    .expect_err("--json is required");
    assert_eq!(usage.code(), "USAGE");
    let invalid = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("--root"),
            root.join("missing").into_os_string(),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        Ok(hub),
    )
    .expect_err("invalid explicit root");
    assert_eq!(invalid.code(), "COORD_IO_FAILED");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn doctor_rejects_unknown_legacy_lock_fields() {
    let root = fixture_root("unknown-lock-field");
    let hub = root.join("bullet-farm");
    fs::write(hub.join("family.lock"), legacy_lock("unexpected = true\n"))
        .expect("hostile lock fixture");
    let error = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        Ok(hub),
    )
    .expect_err("unknown lock field must fail closed");
    assert_eq!(error.code(), "INVALID_FAMILY_LOCK");
    fs::remove_dir_all(root).expect("remove fixture");
}

/// The first command a newcomer runs must not signal success while it reports
/// blockers. `doctor` exits 3 — the family's "diagnosed, not usable" code,
/// shared with `check`'s blocked gates and the coordinator's claim refusals —
/// and the `--json` body stays machine-parsable on that path. The READY half of
/// the mapping (exit 0) is proved in `src/doctor/model.rs`; a READY fixture
/// would need a signed schema-3 lock and real clones at exact OIDs, which is
/// operator input this repository does not have.
#[test]
fn doctor_exit_status_agrees_with_its_reported_status() {
    let root = fixture_root("exit-status");
    let hub = root.join("bullet-farm");
    for argv in [
        vec![
            OsString::from("bullet-family"),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        vec![
            OsString::from("bullet-family"),
            OsString::from("--root"),
            hub.clone().into_os_string(),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
    ] {
        let outcome = bullet_family::cli::execute(argv, Ok(hub.clone())).expect("doctor report");
        let report: serde_json::Value =
            serde_json::from_str(outcome.output()).expect("JSON on the blocked path");
        assert_eq!(report["status"], "BLOCKED");
        assert_eq!(report["schema_version"], 1);
        assert_eq!(
            outcome.exit_code(),
            3,
            "a hub reporting BLOCKED must never exit 0"
        );
    }

    // A typed refusal keeps its own exit code; 3 means "diagnosed, not usable",
    // never "the arguments were wrong".
    let usage = bullet_family::cli::execute(
        [OsString::from("bullet-family"), OsString::from("doctor")],
        Ok(hub.clone()),
    )
    .expect_err("--json is required");
    assert_eq!(usage.code(), "USAGE");
    assert_eq!(usage.exit_code(), 2);

    let missing = bullet_family::cli::execute(
        [
            OsString::from("bullet-family"),
            OsString::from("--root"),
            root.join("missing").into_os_string(),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        Ok(hub),
    )
    .expect_err("invalid explicit root");
    assert_eq!(missing.code(), "COORD_IO_FAILED");
    assert_eq!(missing.exit_code(), 2);

    fs::remove_dir_all(root).expect("remove fixture");
}
