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

/// The complete locked family, in lock order.
const FAMILY: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];
const FAMILY_MANIFEST: &str = concat!(
    "schema_version = \"1.2.0\"\n",
    "family = \"bullet-farm\"\n",
    "umbrella_repo = \"bullet-farm\"\n",
    "required_repos = [\"bullet-farm\", \"bullet-git\", \"bullet-kernel\", \"bullet-portal\"]\n",
);
const PLAIN_CONFIG: &str = "[core]\n\trepositoryformatversion = 0\n\tbare = false\n";

/// A four-member family: an outer `repos.manifest.toml`, a hub, and sibling
/// member checkouts each carrying a minimally valid `.git`. Everything lives
/// under the returned tempdir; no test here reads or writes outside it.
fn family_fixture(name: &str) -> PathBuf {
    let root = fixture_root(name);
    fs::write(root.join("repos.manifest.toml"), FAMILY_MANIFEST).expect("family manifest");
    fs::write(
        root.join("bullet-farm/repos.manifest.toml"),
        FAMILY_MANIFEST,
    )
    .expect("hub manifest");
    fs::write(root.join("bullet-farm/family.lock"), family_lock()).expect("lock fixture");
    for member in FAMILY {
        plant_admin_tree(&root.join(member), PLAIN_CONFIG);
    }
    root
}

fn family_lock() -> String {
    let digest = "a".repeat(64);
    let commit = "b".repeat(40);
    let mut lock = format!(
        concat!(
            "schema_version = \"2\"\n",
            "family = \"bullet-farm\"\n",
            "tag = \"v0.1.0-alpha.4\"\n",
            "schema_bundle_hash = \"blake3:{digest}\"\n",
        ),
        digest = digest,
    );
    for member in FAMILY {
        lock.push_str(&format!(
            concat!(
                "[[member]]\n",
                "name = \"{member}\"\n",
                "tag = \"v0.1.0-alpha.4\"\n",
                "commit_oid = \"{commit}\"\n",
                "schema_bundle_hash = \"blake3:{digest}\"\n",
                "release_signing_identity = \"bot@invalid.example|ed25519|SHA256:fixture\"\n",
                "generated_client_hash = \"blake3:{digest}\"\n",
            ),
            member = member,
            commit = commit,
            digest = digest,
        ));
    }
    lock
}

fn plant_admin_tree(repo: &Path, config: &str) {
    let git_dir = repo.join(".git");
    fs::create_dir_all(git_dir.join("objects")).expect("objects directory");
    fs::create_dir_all(git_dir.join("refs")).expect("refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD fixture");
    fs::write(git_dir.join("index"), b"").expect("index fixture");
    fs::write(git_dir.join("config"), config).expect("config fixture");
}

fn doctor_report(hub: &Path) -> serde_json::Value {
    let output = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("doctor"),
            OsString::from("--json"),
        ],
        Ok(hub.to_path_buf()),
    )
    .expect("doctor report");
    serde_json::from_str(&output).expect("JSON report")
}

fn check<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|check| check["id"] == id)
        .unwrap_or_else(|| panic!("doctor dropped the {id} check"))
}

fn detail<'a>(report: &'a serde_json::Value, id: &str) -> &'a str {
    check(report, id)["detail"].as_str().expect("check detail")
}

/// Restores a mode-000 fixture directory even when the test panics, so the
/// tempdir is always removable and nothing is left unreadable on the host.
#[cfg(unix)]
struct RestoredMode<'a>(&'a Path);

#[cfg(unix)]
impl Drop for RestoredMode<'_> {
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
    }
}

/// Mode-000 CI quarantine directories inside a member's `.git` are the observed
/// real-world trigger. One of them used to abort the whole administrative walk
/// with a bare `COORD_IO_FAILED`, and the member then vanished from every later
/// per-member verdict — the family's own health command silently stopped
/// reporting one quarter of the family. The refusal must now be typed, name the
/// exact unreadable path, and keep that member fail-closed, while the other
/// three members are still walked and still reported.
#[cfg(unix)]
#[test]
fn an_unreadable_admin_node_names_that_member_and_still_reports_every_other() {
    use std::os::unix::fs::PermissionsExt;

    let root = family_fixture("unreadable-admin-node");
    let hub = root.join("bullet-farm");
    let quarantine = root
        .join("bullet-kernel/.git")
        .join("bullet-ci-target-quarantine.66310.87642858.uHJvvkWo7O");
    fs::create_dir_all(&quarantine).expect("quarantine fixture");
    fs::set_permissions(&quarantine, fs::Permissions::from_mode(0o000)).expect("mode 000");
    let restore = RestoredMode(&quarantine);

    let report = doctor_report(&hub);
    let layout = detail(&report, "family_layout");
    assert!(
        layout.contains("bullet-kernel (UNREADABLE_GIT_METADATA"),
        "the unreadable member needs a typed per-member refusal, got: {layout}"
    );
    assert!(
        layout.contains(&quarantine.display().to_string()),
        "the refusal must name the offending path, got: {layout}"
    );
    assert!(
        !layout.contains("COORD_IO_FAILED"),
        "an anonymous I/O failure is not a diagnosis, got: {layout}"
    );
    assert_eq!(
        layout.matches("UNREADABLE_GIT_METADATA").count(),
        1,
        "only the member with the unreadable node may carry that refusal"
    );
    for member in FAMILY {
        assert!(
            layout.contains(member),
            "{member} dropped out of family_layout: {layout}"
        );
    }

    // Fail-closed: a member whose administrative tree could not be read is
    // never reported at its locked OID and never reported clean.
    for id in ["member_oids", "clean_checkouts"] {
        assert_eq!(
            check(&report, id)["status"],
            "BLOCKED",
            "{id} must not pass while a member is unverifiable"
        );
        assert!(
            detail(&report, id).contains("bullet-kernel (UNREADABLE_GIT_METADATA)"),
            "{id} silently dropped the unreadable member: {}",
            detail(&report, id)
        );
    }
    assert_eq!(report["status"], "BLOCKED");

    drop(restore);
    fs::remove_dir_all(root).expect("remove fixture");
}
