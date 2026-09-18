use super::*;
use std::collections::BTreeSet;

fn object(kind: &str, size: u64) -> Object {
    Object {
        oid: "a".repeat(40),
        kind: kind.into(),
        size,
    }
}

fn repository() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    git::bytes(temp.path(), &["init", "--bare", "objects.git"]).unwrap();
    let repo = temp.path().join("objects.git");
    (temp, repo)
}

fn commit(repo: &Path, tree: &str, parent: Option<&str>, message: &[u8]) -> String {
    let mut command = git::command(repo);
    command
        .env("GIT_AUTHOR_NAME", "scan fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@bullet.invalid")
        .env("GIT_COMMITTER_NAME", "scan fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@bullet.invalid")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .args(["-c", "commit.gpgSign=false", "commit-tree", tree]);
    if let Some(parent) = parent {
        command.args(["-p", parent]);
    }
    String::from_utf8(git::execute(&mut command, Some(message)).unwrap())
        .unwrap()
        .trim()
        .into()
}

#[test]
fn cat_file_frames_preserve_nul_newline_and_binary_bytes() {
    let selected = [object("blob", 5)];
    let mut raw = format!("{} blob 5\n", selected[0].oid).into_bytes();
    raw.extend_from_slice(&[0, 255, b'\n', b'x', 0, b'\n']);
    assert_eq!(
        batch(&raw, &selected).unwrap()[0],
        &[0, 255, b'\n', b'x', 0]
    );
    let mut trailing = raw.clone();
    trailing.push(b'x');
    assert!(batch(&trailing, &selected).is_err());
    raw.pop();
    assert!(batch(&raw, &selected).is_err());
    assert!(batch(b"missing\n", &selected).is_err());
}

#[test]
fn object_and_total_limits_refuse_without_truncation() {
    assert!(validate_limits(&[]).is_err());
    assert!(validate_limits(&[object("blob", MAX_OBJECT_BYTES + 1)]).is_err());
    assert!(validate_limits(&vec![object("blob", MAX_OBJECT_BYTES); 65]).is_err());
    assert_eq!(
        validate_limits(&[object("blob", MAX_OBJECT_BYTES)]).unwrap(),
        MAX_OBJECT_BYTES
    );
}

#[test]
fn scanner_config_requires_pinned_defaults_and_refuses_external_extends() {
    assert!(validate_config(b"[extend]\nuseDefault = true\n").is_ok());
    for bad in [
        "",
        "[extend]\nuseDefault = false\n",
        "[extend]\nuseDefault = true\npath = '/tmp/unbound.toml'\n",
        "[extend]\nuseDefault = true\ndisabledRules = ['github-pat']\n",
    ] {
        assert!(validate_config(bad.as_bytes()).is_err());
    }
}

#[test]
fn scanner_report_requires_empty_structural_array() {
    assert!(empty_report(b"[]\n").is_ok());
    for bad in [b"{}".as_slice(), b"null", b"[{}]", b"", b"[] []"] {
        assert!(empty_report(bad).is_err());
    }
}

#[test]
fn synthetic_root_retains_deleted_history_and_commit_metadata() {
    let (_temp, repo) = repository();
    let old = hash_blob(&repo, b"removed historical content\0").unwrap();
    let tree = git::execute(
        git::command(&repo).arg("mktree"),
        Some(format!("100644 blob {old}\told\n").as_bytes()),
    )
    .unwrap();
    let tree = String::from_utf8(tree).unwrap();
    let first = commit(&repo, tree.trim(), None, b"historical commit metadata\n");
    let empty =
        String::from_utf8(git::execute(git::command(&repo).arg("mktree"), Some(b"")).unwrap())
            .unwrap();
    let latest = commit(&repo, empty.trim(), Some(&first), b"deletion\n");
    let objects = inventory(&repo, &[latest]).unwrap();
    assert!(objects.iter().any(|o| o.oid == old));
    assert!(objects.iter().any(|o| o.oid == first));
    let deadline = Instant::now() + Duration::from_secs(30);
    let (scan_commit, raw_digest) = synthetic(&repo, &objects, deadline).unwrap();
    assert_eq!(
        synthetic(&repo, &objects, deadline).unwrap(),
        (scan_commit.clone(), raw_digest)
    );
    for object in objects {
        let original = git::bytes(&repo, &["cat-file", &object.kind, &object.oid]).unwrap();
        let preserved =
            git::bytes(&repo, &["show", &format!("{scan_commit}:{}", object.oid)]).unwrap();
        assert_eq!(original, preserved);
    }
    assert!(
        !git::text(&repo, &["cat-file", "commit", &scan_commit])
            .unwrap()
            .lines()
            .any(|line| line.starts_with("parent "))
    );
}

#[cfg(target_os = "linux")]
#[test]
fn actual_pinned_scanner_detects_binary_and_chunk_boundary_canaries() {
    let (_temp, repo) = repository();
    let scanner_path = std::env::var_os("BULLET_PUBLICATION_GITLEAKS")
        .expect("admit the pinned scanner for this conformance test");
    let scanner = pin_scanner(Path::new(&scanner_path)).unwrap();
    let mut token = b"ghp_".to_vec();
    token.extend((0..36).map(|i| b"abcdef0123456789"[i % 16]));
    let mut objects = Vec::new();
    for offset in [0, 4090, 9998, 10002, 20000] {
        let mut content = vec![b'x'; offset];
        content.push(0);
        content.push(255);
        content.push(b'\n');
        content.extend_from_slice(&token);
        content.extend_from_slice(b" gitleaks:allow");
        let oid = hash_blob(&repo, &content).unwrap();
        objects.push(Object {
            oid,
            kind: "blob".into(),
            size: content.len() as u64,
        });
    }
    let empty =
        String::from_utf8(git::execute(git::command(&repo).arg("mktree"), Some(b"")).unwrap())
            .unwrap();
    let metadata = commit(&repo, empty.trim(), None, &token);
    let metadata_size = git::text(&repo, &["cat-file", "-s", &metadata])
        .unwrap()
        .parse::<u64>()
        .unwrap();
    objects.push(Object {
        oid: metadata,
        kind: "commit".into(),
        size: metadata_size,
    });
    objects.sort_by(|a, b| a.oid.cmp(&b.oid));
    let (subject, _) =
        synthetic(&repo, &objects, Instant::now() + Duration::from_secs(30)).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let config = temp.path().join("gitleaks.toml");
    fs::write(&config, include_bytes!("../../publication/gitleaks.toml")).unwrap();
    let report = temp.path().join("report.json");
    let mut command = scanner_command(&scanner, &repo, &subject, &config, &report, temp.path());
    assert!(
        command
            .get_envs()
            .any(|(key, value)| key == "GITLEAKS_CONFIG" && value.is_none())
    );
    let output = process::run_bounded(
        &mut command,
        "pinned scanner conformance",
        process::Limits {
            timeout: Duration::from_secs(30),
            stdout_bytes: 1024 * 1024,
            stderr_bytes: 1024 * 1024,
        },
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let findings: serde_json::Value =
        serde_json::from_slice(&bounded_file(&report, 1024 * 1024).unwrap()).unwrap();
    let files = findings
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["RuleID"] == "github-pat")
        .map(|f| f["File"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        files.len(),
        objects.len(),
        "every binary/root-addition canary must be detected"
    );
    for (name, content, expected_status) in [
        (
            "public-storage-label",
            b"const CSRF_STORAGE_KEY = \"bullet-farm.csrf.v1\";\n".to_vec(),
            0,
        ),
        (
            "changed-storage-value",
            format!(
                "const CSRF_STORAGE_KEY = \"{}\";\n",
                digest(b"generated noncredential scanner canary")
            )
            .into_bytes(),
            1,
        ),
    ] {
        let oid = hash_blob(&repo, &content).unwrap();
        let objects = [Object {
            oid,
            kind: "blob".into(),
            size: content.len() as u64,
        }];
        let (subject, _) =
            synthetic(&repo, &objects, Instant::now() + Duration::from_secs(30)).unwrap();
        let report = temp.path().join(format!("{name}.json"));
        let mut command = scanner_command(&scanner, &repo, &subject, &config, &report, temp.path());
        let output = process::run_bounded(
            &mut command,
            "pinned policy scanner conformance",
            process::Limits {
                timeout: Duration::from_secs(30),
                stdout_bytes: 1024 * 1024,
                stderr_bytes: 1024 * 1024,
            },
        )
        .unwrap();
        assert_eq!(output.status.code(), Some(expected_status), "{name}");
        let report = bounded_file(&report, 1024 * 1024).unwrap();
        if expected_status == 0 {
            empty_report(&report).unwrap();
        } else {
            let findings: serde_json::Value = serde_json::from_slice(&report).unwrap();
            let findings = findings.as_array().unwrap();
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0]["RuleID"], "generic-api-key");
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn scanner_pin_refuses_substituted_executable() {
    let temp = tempfile::tempdir().unwrap();
    let fake = temp.path().join("gitleaks");
    fs::write(&fake, b"untrusted substitute").unwrap();
    assert!(pin_scanner(&fake).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn actual_scan_receipt_survives_restart_and_rejects_report_tampering() {
    let fixture = super::super::tests::Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    scan(&store, &request, &prepared).unwrap();
    let id = request.request_id.clone();
    let store_root = store.root.clone();
    let receipt_bytes = fs::read(store.path(&id, "scan")).unwrap();
    let inventory_bytes = fs::read(store.path(&id, "scan-inventory")).unwrap();
    let receipt: Receipt = decode(&receipt_bytes).unwrap();
    assert_eq!(receipt.subject.aggregate_commit, prepared.aggregate_commit);
    assert_eq!(receipt.subject.inventory_sha256, digest(&inventory_bytes));
    assert!(receipt.subject.object_count > 0);
    let attempts = || {
        fs::read_dir(&store_root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .filter(|name| name.to_string_lossy().starts_with("scan-attempt-"))
            .count()
    };
    assert_eq!(attempts(), 1);
    drop(store);
    let store = Store::open(&store_root).unwrap();
    let (request, prepared) = store.load(&id).unwrap();
    scan(&store, &request, &prepared).unwrap();
    assert_eq!(fs::read(store.path(&id, "scan")).unwrap(), receipt_bytes);
    assert_eq!(
        fs::read(store.path(&id, "scan-inventory")).unwrap(),
        inventory_bytes
    );
    assert_eq!(attempts(), 1);
    let report_path = store_root.join(receipt.attempt).join("report.json");
    let report_bytes = fs::read(&report_path).unwrap();
    fs::write(&report_path, b"[]\n ").unwrap();
    let error = scan(&store, &request, &prepared).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("PUBLICATION_SCAN_ARTIFACT_DRIFT")
    );
    assert_eq!(fs::read(store.path(&id, "scan")).unwrap(), receipt_bytes);
    assert_eq!(attempts(), 1);
    fs::write(&report_path, report_bytes).unwrap();
    fs::write(store.path(&id, "scan-inventory"), b"[]\n").unwrap();
    let error = scan(&store, &request, &prepared).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("PUBLICATION_SCAN_ARTIFACT_DRIFT")
    );
    assert_eq!(fs::read(store.path(&id, "scan")).unwrap(), receipt_bytes);
    assert_eq!(attempts(), 1);
}
