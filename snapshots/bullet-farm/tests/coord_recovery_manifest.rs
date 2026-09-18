#![cfg(target_os = "linux")]

use std::{
    ffi::OsString,
    fs,
    os::unix::{ffi::OsStringExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use sha2::{Digest, Sha256};

#[path = "coord_recovery_manifest/parent_custody.rs"]
mod parent_custody;

fn write_private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}

fn run(root: &Path, action: &str, args: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bullet-family"));
    command.arg("--root").arg(root).arg("coord").arg(action);
    for (name, value) in args {
        command.arg(format!("--{name}")).arg(value);
    }
    command.output().unwrap()
}

fn success_output(action: &str, output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{action} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    bullet_wire::decode_unique_value(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{action} returned non-JSON stdout ({error}): stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn success(root: &Path, action: &str, args: &[(&str, &Path)]) -> serde_json::Value {
    success_output(action, run(root, action, args))
}

struct Fixture {
    root: tempfile::TempDir,
    interrupted: PathBuf,
    tainted: PathBuf,
    frozen: PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(root.path().join("repos.manifest.toml"), "version = 1\n").unwrap();
    let coord = root.path().join(".bullet-family/coord");
    fs::create_dir_all(&coord).unwrap();
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o700)).unwrap();
    let claim = serde_json::json!({
        "kind": "claim",
        "schema_version": 1,
        "at_unix_ms": 5,
        "claim_id": format!("clm_{}", "a".repeat(64)),
        "agent": "fixture-agent",
        "lane": "fixture-lane",
        "repo": "bullet-farm",
        "paths": ["src/coord"],
        "expires_unix_ms": 60005,
    });
    let mut prefix = bullet_wire::canonical_json(&claim).unwrap();
    prefix.push(b'\n');
    let mut interrupted_bytes = prefix.clone();
    interrupted_bytes.extend_from_slice(b"partial-record");
    let mut tainted_bytes = interrupted_bytes.clone();
    tainted_bytes.extend_from_slice(b"-tainted-and-committed\n");
    let mut frozen_bytes = prefix;
    frozen_bytes.extend_from_slice(b"different-frozen-record-one\n");
    frozen_bytes.extend_from_slice(b"different-frozen-record-two-with-padding\n");
    assert!(interrupted_bytes.len() < tainted_bytes.len());
    assert!(tainted_bytes.len() < frozen_bytes.len());

    let interrupted = root.path().join("interrupted.partial");
    let tainted = root.path().join("tainted.jsonl");
    let frozen = coord.join("events.jsonl");
    write_private(&interrupted, &interrupted_bytes);
    write_private(&tainted, &tainted_bytes);
    write_private(&frozen, &frozen_bytes);
    Fixture {
        root,
        interrupted,
        tainted,
        frozen,
    }
}

struct AuthorityPaths {
    authorization: PathBuf,
    signature: PathBuf,
    provenance: PathBuf,
}

fn write_authority(inspection: &Path, parent: &Path, add_unknown: bool) -> AuthorityPaths {
    let inspection_bytes = fs::read(inspection).unwrap();
    let inspection_value = bullet_wire::decode_unique_value(&inspection_bytes).unwrap();
    let provenance = parent.join("bootstrap-provenance.json");
    let provenance_value = serde_json::json!({
        "kind": "bullet.coord.recovery-bootstrap-provenance.v1",
        "schema_version": 1,
        "bootstrap_commit_oid": "1".repeat(40),
        "bootstrap_tree_oid": "2".repeat(40),
        "archive_sha256": format!("sha256:{}", "3".repeat(64)),
        "cargo_lock_sha256": format!("sha256:{}", "4".repeat(64)),
        "source_files": [{
            "path": "Cargo.lock",
            "byte_length": 1,
            "sha256": format!("sha256:{}", "4".repeat(64)),
        }],
        "rustc_version": "rustc process-test",
        "cargo_version": "cargo process-test",
        "executable_byte_length": 1,
        "executable_sha256": format!("sha256:{}", "5".repeat(64)),
    });
    let mut provenance_bytes = bullet_wire::canonical_json(&provenance_value).unwrap();
    provenance_bytes.push(b'\n');
    write_private(&provenance, &provenance_bytes);
    let authorization = parent.join("authorization.json");
    let mut authorization_value = serde_json::json!({
        "kind": "bullet.coord.recovery-authorization.v1",
        "schema_version": 1,
        "decision": "APPROVE",
        "inspection_id": inspection_value["inspection_id"],
        "inspection_sha256": format!("sha256:{:x}", Sha256::digest(&inspection_bytes)),
        "recovery_operator": "bullet-recovery-operator",
        "recovery_operator_uid": rustix::process::geteuid().as_raw(),
        "reviewer_principal": "bullet-recovery-reviewer",
        "reviewer_fingerprint": format!("sha256:{}", "6".repeat(64)),
        "policy_namespace": "bullet-family-coordinator-recovery-v1",
        "bootstrap_provenance_sha256": format!("sha256:{:x}", Sha256::digest(&provenance_bytes)),
        "decision_at_unix_ms": 10,
        "authorized_at_unix_ms": 10,
        "expires_at_unix_ms": 100,
        "authority_boot_id": "00000000-0000-4000-8000-000000000001",
        "authority_time_namespace_device": 1,
        "authority_time_namespace_inode": 1,
        "authorized_at_boottime_ms": 10,
        "expires_at_boottime_ms": 100,
    });
    if add_unknown {
        authorization_value["caller_forensic_hash"] =
            serde_json::json!(format!("sha256:{}", "f".repeat(64)));
    }
    let mut authorization_bytes = bullet_wire::canonical_json(&authorization_value).unwrap();
    authorization_bytes.push(b'\n');
    write_private(&authorization, &authorization_bytes);
    let signature = parent.join("authorization-signature.json");
    let signature_value = serde_json::json!({
        "kind": "bullet.coord.recovery-authorization-signature.v1",
        "schema_version": 1,
        "namespace": "bullet-family-coordinator-recovery-v1",
        "reviewer_principal": "bullet-recovery-reviewer",
        "reviewer_fingerprint": format!("sha256:{}", "6".repeat(64)),
        "authorization_sha256": format!("sha256:{:x}", Sha256::digest(&authorization_bytes)),
        "signature_ed25519": format!("ed25519:{}", "0".repeat(128)),
    });
    let mut signature_bytes = bullet_wire::canonical_json(&signature_value).unwrap();
    signature_bytes.push(b'\n');
    write_private(&signature, &signature_bytes);
    AuthorityPaths {
        authorization,
        signature,
        provenance,
    }
}

#[test]
fn production_binary_inspects_but_policy_disables_manifest_and_rollover() {
    let fixture = fixture();
    let inspection = fixture.root.path().join("inspection.json");
    let manifest = fixture.root.path().join("manifest.json");
    let inspect = success(
        fixture.root.path(),
        "recovery-inspect",
        &[
            ("interrupted-capture", &fixture.interrupted),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", &inspection),
        ],
    );
    assert_eq!(inspect["kind"], "bullet.coord.recovery-inspection.v1");
    assert_eq!(
        fs::metadata(&inspection).unwrap().permissions().mode() & 0o7777,
        0o400
    );
    let authority = write_authority(&inspection, fixture.root.path(), false);

    let produced = run(
        fixture.root.path(),
        "recovery-manifest",
        &[
            ("inspection", &inspection),
            ("authorization", &authority.authorization),
            ("authorization-signature", &authority.signature),
            ("bootstrap-provenance", &authority.provenance),
            ("interrupted-capture", &fixture.interrupted),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", &manifest),
        ],
    );
    assert!(!produced.status.success());
    let stderr = String::from_utf8(produced.stderr).unwrap();
    assert!(stderr.contains("RECOVERY_POLICY_DISABLED"), "{stderr}");
    assert!(!manifest.exists());
    assert!(
        !fixture
            .root
            .path()
            .join(".bullet-family/coord/CURRENT")
            .exists()
    );
}

#[test]
fn inspect_refuses_unknown_input_without_output_or_coord_mutation() {
    let fixture = fixture();
    let output = fixture.root.path().join("inspection.json");
    let current = fixture.root.path().join(".bullet-family/coord/CURRENT");
    let before = fs::read_dir(fixture.root.path().join(".bullet-family/coord"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    let relative = Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .arg("--root")
        .arg(fixture.root.path())
        .args([
            "coord",
            "recovery-inspect",
            "--interrupted-capture",
            "interrupted.jsonl",
            "--tainted-generation",
        ])
        .arg(&fixture.tainted)
        .arg("--frozen-live-source")
        .arg(&fixture.frozen)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!relative.status.success());
    assert!(!output.exists());
    let result = Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .arg("--root")
        .arg(fixture.root.path())
        .args(["coord", "recovery-inspect", "--output"])
        .arg(&output)
        .args(["--caller-hash", "sha256:deadbeef"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!output.exists());
    let non_utf8_path = PathBuf::from(OsString::from_vec(vec![b'/', 0xff]));
    let non_utf8 = run(
        fixture.root.path(),
        "recovery-inspect",
        &[
            ("interrupted-capture", &non_utf8_path),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", &output),
        ],
    );
    assert!(!non_utf8.status.success());
    let non_utf8_stderr = String::from_utf8(non_utf8.stderr).unwrap();
    assert!(
        non_utf8_stderr.contains("INVALID_ARGUMENT")
            && non_utf8_stderr.contains("arguments must be valid UTF-8"),
        "unexpected non-UTF-8 refusal: {non_utf8_stderr}"
    );
    assert!(!output.exists());
    assert!(!current.exists());
    let after = fs::read_dir(fixture.root.path().join(".bullet-family/coord"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    every_recovery_document_role_refuses_lexical_aliases_before_mutation();
}

#[test]
fn source_mode_and_symlink_refuse_without_output_or_coord_mutation() {
    for symlinked in [false, true] {
        let fixture = fixture();
        let output = fixture.root.path().join("inspection.json");
        if symlinked {
            fs::remove_file(&fixture.interrupted).unwrap();
            std::os::unix::fs::symlink(&fixture.tainted, &fixture.interrupted).unwrap();
        } else {
            fs::set_permissions(&fixture.interrupted, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let before = fs::read_dir(fixture.root.path().join(".bullet-family/coord"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        let result = run(
            fixture.root.path(),
            "recovery-inspect",
            &[
                ("interrupted-capture", &fixture.interrupted),
                ("tainted-generation", &fixture.tainted),
                ("frozen-live-source", &fixture.frozen),
                ("output", &output),
            ],
        );
        assert!(!result.status.success());
        assert!(!output.exists());
        let after = fs::read_dir(fixture.root.path().join(".bullet-family/coord"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(before, after);
    }
}

#[test]
fn changed_source_and_unknown_authorization_refuse_manifest_creation() {
    for unknown_authorization in [false, true] {
        let fixture = fixture();
        let inspection = fixture.root.path().join("inspection.json");
        let manifest = fixture.root.path().join("manifest.json");
        success(
            fixture.root.path(),
            "recovery-inspect",
            &[
                ("interrupted-capture", &fixture.interrupted),
                ("tainted-generation", &fixture.tainted),
                ("frozen-live-source", &fixture.frozen),
                ("output", &inspection),
            ],
        );
        let authority = write_authority(&inspection, fixture.root.path(), unknown_authorization);
        if !unknown_authorization {
            let mut changed = fs::read(&fixture.interrupted).unwrap();
            *changed.last_mut().unwrap() ^= 1;
            fs::set_permissions(&fixture.interrupted, fs::Permissions::from_mode(0o600)).unwrap();
            fs::write(&fixture.interrupted, changed).unwrap();
            fs::set_permissions(&fixture.interrupted, fs::Permissions::from_mode(0o400)).unwrap();
        }
        let result = run(
            fixture.root.path(),
            "recovery-manifest",
            &[
                ("inspection", &inspection),
                ("authorization", &authority.authorization),
                ("authorization-signature", &authority.signature),
                ("bootstrap-provenance", &authority.provenance),
                ("interrupted-capture", &fixture.interrupted),
                ("tainted-generation", &fixture.tainted),
                ("frozen-live-source", &fixture.frozen),
                ("output", &manifest),
            ],
        );
        assert!(!result.status.success());
        assert!(!manifest.exists());
        assert!(
            !fixture
                .root
                .path()
                .join(".bullet-family/coord/CURRENT")
                .exists()
        );
    }
}

fn lexical_aliases(path: &Path) -> [PathBuf; 7] {
    let raw = path.to_str().unwrap();
    let parent = path.parent().unwrap();
    let parent_name = parent.file_name().unwrap();
    let file_name = path.file_name().unwrap();
    [
        PathBuf::from(format!("/{raw}")),
        PathBuf::from(format!("/./{}", raw.strip_prefix('/').unwrap())),
        PathBuf::from(format!("{raw}/")),
        PathBuf::from(format!("{raw}/.")),
        PathBuf::from(format!("{raw}/..")),
        parent.join("..").join(parent_name).join(file_name),
        PathBuf::from("/"),
    ]
}

fn assert_lexical_refusal(result: Output, output: &Path, current: &Path) {
    assert!(!result.status.success());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("INVALID_RECOVERY_PRODUCTION")
            && stderr.contains("normalized absolute lexical bytes"),
        "unexpected lexical-refusal error: {stderr}"
    );
    assert!(!output.exists());
    assert!(!current.exists());
}

fn every_recovery_document_role_refuses_lexical_aliases_before_mutation() {
    let fixture = fixture();
    let inspection = fixture.root.path().join("inspection-λ.json");
    let current = fixture.root.path().join(".bullet-family/coord/CURRENT");
    let inspect_values = [
        fixture.interrupted.clone(),
        fixture.tainted.clone(),
        fixture.frozen.clone(),
        inspection.clone(),
    ];
    for role in 0..inspect_values.len() {
        for alias in lexical_aliases(&inspect_values[role]) {
            let mut values = inspect_values.clone();
            values[role] = alias;
            let result = run(
                fixture.root.path(),
                "recovery-inspect",
                &[
                    ("interrupted-capture", &values[0]),
                    ("tainted-generation", &values[1]),
                    ("frozen-live-source", &values[2]),
                    ("output", &values[3]),
                ],
            );
            assert_lexical_refusal(result, &inspection, &current);
        }
    }

    success(
        fixture.root.path(),
        "recovery-inspect",
        &[
            ("interrupted-capture", &fixture.interrupted),
            ("tainted-generation", &fixture.tainted),
            ("frozen-live-source", &fixture.frozen),
            ("output", &inspection),
        ],
    );
    let manifest = fixture.root.path().join("manifest-λ.json");
    let authority = write_authority(&inspection, fixture.root.path(), false);
    let manifest_values = [
        inspection.clone(),
        authority.authorization,
        authority.signature,
        authority.provenance,
        fixture.interrupted.clone(),
        fixture.tainted.clone(),
        fixture.frozen.clone(),
        manifest.clone(),
    ];
    for role in 0..manifest_values.len() {
        for alias in lexical_aliases(&manifest_values[role]) {
            let mut values = manifest_values.clone();
            values[role] = alias;
            let result = run(
                fixture.root.path(),
                "recovery-manifest",
                &[
                    ("inspection", &values[0]),
                    ("authorization", &values[1]),
                    ("authorization-signature", &values[2]),
                    ("bootstrap-provenance", &values[3]),
                    ("interrupted-capture", &values[4]),
                    ("tainted-generation", &values[5]),
                    ("frozen-live-source", &values[6]),
                    ("output", &values[7]),
                ],
            );
            assert_lexical_refusal(result, &manifest, &current);
        }
    }
}
