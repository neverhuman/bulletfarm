use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use bullet_family::family_lock::{
    ExternalSubjectManifest, ExternalSubjects, JeryuSubject, PortalSubject, ProviderSubject,
    ReleaseSigningSubject, SandboxSubject, ToolchainSubject,
};

const TAG: &str = "v1.0.0-fuse.1";
const PRINCIPAL: &str = "fuse-fixture@bullet.invalid";
const REPOSITORIES: &[&str] = &[
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn local_fusion_is_exact_idempotent_and_fail_closed() {
    let fixture = Fixture::local();
    let first = fuse(&fixture.root, "local");
    assert!(first.status.success(), "{first:?}");
    let initial = fusion_snapshot(&fixture.hub());
    assert_local_manifest(&fixture.root, &initial);

    let second = fuse(&fixture.root, "local");
    assert!(second.status.success(), "{second:?}");
    assert_eq!(initial, fusion_snapshot(&fixture.hub()));
    assert_family_clean(&fixture.root);

    fs::write(fixture.root.join("bullet-kernel/UNTRACKED"), "dirty\n").unwrap();
    let dirty = fuse(&fixture.root, "local");
    assert_error(&dirty, "DIRTY_CHECKOUT");
    assert_eq!(initial, fusion_snapshot(&fixture.hub()));
    fs::remove_file(fixture.root.join("bullet-kernel/UNTRACKED")).unwrap();

    let unsupported = fuse(&fixture.root, "lock");
    assert_error(&unsupported, "UNSUPPORTED_SCHEMA");
    assert_eq!(initial, fusion_snapshot(&fixture.hub()));

    #[cfg(unix)]
    {
        let link = fixture.root.with_extension("fusion-root-link");
        std::os::unix::fs::symlink(&fixture.root, &link).unwrap();
        let escaped = fuse(&link, "local");
        assert_error(&escaped, "INVALID_CHECKOUT");
        fs::remove_file(link).unwrap();
        assert_eq!(initial, fusion_snapshot(&fixture.hub()));
    }
}

#[test]
fn local_fusion_reports_an_absent_canonical_member_before_publication() {
    let fixture = Fixture::local();
    fs::remove_dir_all(fixture.root.join("bullet-portal")).unwrap();

    let output = fuse(&fixture.root, "local");
    assert_error(&output, "FAMILY_MEMBER_MISSING");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("bullet-portal"));
    assert!(stderr.contains("bullet-family doctor --json"));
    assert!(!fixture.hub().join(".fusion").exists());
}

#[test]
fn lock_fusion_binds_verified_schema_three_sources_and_subjects() {
    let fixture = Fixture::signed();
    let output = fuse(&fixture.root, "lock");
    assert!(output.status.success(), "{output:?}");
    let snapshot = fusion_snapshot(&fixture.hub());
    let manifest: toml::Value =
        toml::from_str(std::str::from_utf8(snapshot.get("manifest.toml").unwrap()).unwrap())
            .unwrap();
    assert_eq!(manifest["schema_version"].as_str(), Some("1"));
    assert_eq!(manifest["source"].as_str(), Some("lock"));
    assert_eq!(
        manifest["bullet_wire_path"].as_str(),
        Some("../crates/bullet-wire")
    );
    let records = manifest["repository"].as_array().unwrap();
    assert_eq!(records.len(), REPOSITORIES.len());
    for (record, name) in records.iter().zip(REPOSITORIES) {
        let repo = fixture.root.join(name);
        let commit = subject(&repo, "HEAD^{commit}");
        let tree = subject(&repo, "HEAD^{tree}");
        assert_eq!(record["name"].as_str(), Some(*name));
        assert_eq!(record["commit_oid"].as_str(), Some(commit.as_str()));
        assert_eq!(record["tree_oid"].as_str(), Some(tree.as_str()));
        assert_eq!(record["tag"].as_str(), Some(TAG));
        if *name == "bullet-farm" {
            assert!(record.get("jeryu_url").is_none());
        } else {
            assert_eq!(
                record["jeryu_url"].as_str(),
                Some(format!("https://jeryu.example/git/root/{name}.git").as_str())
            );
            assert_eq!(
                record["jeryu_slug"].as_str(),
                Some(format!("root/{name}").as_str())
            );
        }
    }
    let first = snapshot;
    assert!(fuse(&fixture.root, "lock").status.success());
    assert_eq!(first, fusion_snapshot(&fixture.hub()));
    assert_family_clean(&fixture.root);
}

#[test]
fn launcher_is_only_a_strict_rust_forwarder() {
    let launcher = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/fuse.sh"))
        .expect("launcher");
    assert!(
        launcher.contains("exec cargo run --quiet --locked --bin bullet-family -- fuse \"$@\"")
    );
    for forbidden in ["rm -rf", "mkdir", "cat >", "git "] {
        assert!(
            !launcher.contains(forbidden),
            "launcher contains {forbidden:?}"
        );
    }
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn local() -> Self {
        let root = fixture_root();
        write_manifests(&root);
        for name in REPOSITORIES {
            init_repo(&root, name, None, None);
        }
        fs::write(
            root.join("bullet-farm/family.lock"),
            "schema_version = \"2\"\n",
        )
        .unwrap();
        git(&root.join("bullet-farm"), &["add", "family.lock"]);
        git(
            &root.join("bullet-farm"),
            &["commit", "-q", "-m", "Legacy lock fixture"],
        );
        Self { root }
    }

    fn signed() -> Self {
        let root = fixture_root();
        let key = root.join("release-key");
        command(
            &root,
            "/usr/bin/ssh-keygen",
            &["-q", "-t", "ed25519", "-N", "", "-f", text(&key)],
        );
        let public_key = fs::read_to_string(key.with_extension("pub")).unwrap();
        let allowed = format!("{PRINCIPAL} namespaces=\"git\" {}\n", public_key.trim());
        write_manifests(&root);
        for name in REPOSITORIES {
            let origin = (*name != "bullet-farm")
                .then(|| format!("https://jeryu.example/git/root/{name}.git"));
            init_repo(&root, name, Some(&allowed), origin.as_deref());
            if *name != "bullet-farm" {
                sign_tag(&root.join(name), &key);
            }
        }
        let subjects = root.join("external-subjects.toml");
        fs::write(
            &subjects,
            ExternalSubjectManifest::new(external_subjects(&root, &public_key, &allowed))
                .encode()
                .unwrap(),
        )
        .unwrap();
        bullet_family::family_lock::run(
            &root,
            &[
                "generate".into(),
                "--tag".into(),
                TAG.into(),
                "--subjects".into(),
                subjects.display().to_string(),
            ],
        )
        .expect("generate schema-3 lock from signed member subjects");
        let hub = root.join("bullet-farm");
        git(&hub, &["add", "family.lock"]);
        git(&hub, &["commit", "-q", "-m", "Bind exact family"]);
        sign_tag(&hub, &key);
        Self { root }
    }

    fn hub(&self) -> PathBuf {
        self.root.join("bullet-farm")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove exact fixture");
    }
}

fn init_repo(root: &Path, name: &str, allowed: Option<&str>, origin: Option<&str>) {
    let repo = root.join(name);
    fs::create_dir_all(repo.join("agent")).unwrap();
    fs::create_dir_all(repo.join("scripts")).unwrap();
    fs::write(repo.join("agent/generated-zones.toml"), "zone = []\n").unwrap();
    fs::write(repo.join("scripts/ci-local.sh"), "#!/bin/sh\nexit 0\n").unwrap();
    if name == "bullet-portal" {
        fs::write(repo.join("package-lock.json"), "{}\n").unwrap();
    } else {
        fs::write(repo.join("Cargo.lock"), "# exact fixture lock\n").unwrap();
    }
    if name == "bullet-farm" {
        fs::create_dir_all(repo.join("crates/bullet-wire")).unwrap();
        fs::create_dir_all(repo.join("release")).unwrap();
        fs::write(
            repo.join("crates/bullet-wire/schema.rs"),
            "pub struct Fixture;\n",
        )
        .unwrap();
        fs::write(
            repo.join("release/allowed_signers"),
            allowed.unwrap_or("fixture\n"),
        )
        .unwrap();
        fs::write(repo.join("repos.manifest.toml"), hub_manifest()).unwrap();
        fs::write(repo.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").unwrap();
        fs::write(repo.join(".gitignore"), "/.fusion/\n/.fusion.stage.*\n").unwrap();
    }
    fs::write(
        repo.join("Cargo.toml"),
        format!("[package]\nname = \"{name}-fixture\"\nversion = \"0.0.0\"\n"),
    )
    .unwrap();
    command(&repo, "/usr/bin/git", &["init", "--quiet"]);
    git(&repo, &["config", "user.name", "Fuse Fixture"]);
    git(&repo, &["config", "user.email", PRINCIPAL]);
    if let Some(url) = origin {
        git(&repo, &["remote", "add", "origin", url]);
    }
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "Exact fixture subject"]);
}

fn write_manifests(root: &Path) {
    let mut manifest = format!(
        "family = \"bullet-farm\"\nrequired_repos = {:?}\n",
        REPOSITORIES
    );
    for name in REPOSITORIES {
        manifest.push_str(&format!(
            "[[repo]]\nname = \"{name}\"\npath = \"{}/{name}\"\n",
            root.display()
        ));
        if *name != "bullet-farm" {
            manifest.push_str(&format!(
                "jeryu_url = \"https://jeryu.example/git/root/{name}.git\"\njeryu_slug = \"root/{name}\"\n"
            ));
        }
    }
    fs::write(root.join("repos.manifest.toml"), manifest).unwrap();
}

fn external_subjects(root: &Path, public_key: &str, allowed: &str) -> ExternalSubjects {
    let fingerprint = ssh_fingerprint(&root.join("release-key.pub"));
    ExternalSubjects {
        toolchain: fixture_toolchains(),
        provider: vec![ProviderSubject {
            id: "claude".into(),
            version: "1.0.0".into(),
            profile: "service".into(),
            install_path: "/usr/lib/bullet/providers/claude/1.0.0/claude".into(),
            binary_digest: digest(b'p'),
            protocol_digest: digest(b'r'),
            size_bytes: 1,
        }],
        portal: PortalSubject {
            id: "portal".into(),
            version: "1.0.0".into(),
            source_commit_oid: tagged_subject(&root.join("bullet-portal"), "commit"),
            source_tree_oid: tagged_subject(&root.join("bullet-portal"), "tree"),
            install_path: "/usr/lib/bullet/portal/1.0.0/index.html".into(),
            bundle_digest: digest(b'b'),
            manifest_digest: digest(b'a'),
            size_bytes: 1,
        },
        sandbox: vec![SandboxSubject {
            id: "sandbox-s1".into(),
            class: "s1".into(),
            platform: "ubuntu-24-04-x86-64".into(),
            install_path: "/usr/lib/bullet/sandboxes/s1/rootfs.img".into(),
            image_digest: digest(b'i'),
            policy_digest: digest(b's'),
            size_bytes: 1,
        }],
        jeryu: JeryuSubject {
            id: "jeryu".into(),
            version: "1.0.0".into(),
            tag: "v1.0.0".into(),
            install_path: "/usr/lib/bullet/jeryu/1.0.0/jeryu".into(),
            manifest_digest: digest(b'1'),
            binary_digest: digest(b'2'),
            api_schema_digest: digest(b'3'),
            capability_digest: digest(b'4'),
            sbom_digest: digest(b'5'),
            provenance_digest: digest(b'6'),
            signature_digest: digest(b'7'),
            size_bytes: 1,
        },
        release_signing: ReleaseSigningSubject {
            id: "release-signing".into(),
            identity: format!("{PRINCIPAL}|ed25519|{fingerprint}"),
            policy_digest: digest(b'8'),
            allowed_signers_digest: format!("blake3:{}", blake3::hash(allowed.as_bytes()).to_hex()),
            key_digest: format!("blake3:{}", blake3::hash(public_key.as_bytes()).to_hex()),
            policy_path: "/etc/bullet/release/allowed_signers".into(),
            not_before_unix_ms: 1,
            not_after_unix_ms: 2,
        },
    }
}

fn fixture_toolchains() -> Vec<ToolchainSubject> {
    [
        ("cargo", "1.95.0", "rust/1.95.0/cargo", b't', b'm'),
        ("node", "22.23.2", "node/22.23.2/node", b'n', b'o'),
        ("npm-cli", "10.9.8", "npm/10.9.8/npm-cli.js", b'q', b's'),
    ]
    .into_iter()
    .map(
        |(id, version, relative, binary, manifest)| ToolchainSubject {
            id: id.into(),
            version: version.into(),
            install_path: format!("/usr/lib/bullet/toolchains/{relative}"),
            binary_digest: digest(binary),
            manifest_path: format!("/usr/lib/bullet/toolchains/{relative}.manifest"),
            manifest_digest: digest(manifest),
            size_bytes: 1,
        },
    )
    .collect()
}

fn digest(seed: u8) -> String {
    format!("blake3:{}", blake3::hash(&[seed]).to_hex())
}

fn tagged_subject(repo: &Path, class: &str) -> String {
    let suffix = if class == "commit" {
        "^{commit}"
    } else {
        "^{tree}"
    };
    subject(repo, &format!("{TAG}{suffix}"))
}

fn ssh_fingerprint(public_key: &Path) -> String {
    let output = Command::new("/usr/bin/ssh-keygen")
        .args(["-lf", text(public_key), "-E", "sha256"])
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_owned()
}

fn hub_manifest() -> &'static str {
    concat!(
        "schema_version = \"1.2.0\"\n",
        "family = \"bullet-farm\"\n",
        "umbrella_repo = \"bullet-farm\"\n",
        "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
    )
}

fn fuse(root: &Path, source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .args(["--root", text(root), "fuse", "--source", source])
        .current_dir(root)
        .output()
        .expect("run fuse")
}

fn fusion_snapshot(hub: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    [
        ".bullet-family-fusion-v1",
        "dev.sh",
        "manifest.toml",
        "source",
    ]
    .into_iter()
    .map(|name| {
        (
            name.to_owned(),
            fs::read(hub.join(".fusion").join(name)).unwrap(),
        )
    })
    .collect()
}

fn assert_local_manifest(root: &Path, snapshot: &std::collections::BTreeMap<String, Vec<u8>>) {
    let manifest: toml::Value = toml::from_slice(snapshot.get("manifest.toml").unwrap()).unwrap();
    assert_eq!(manifest["source"].as_str(), Some("local"));
    for (record, name) in manifest["repository"]
        .as_array()
        .unwrap()
        .iter()
        .zip(REPOSITORIES)
    {
        let repo = root.join(name);
        let commit = subject(&repo, "HEAD^{commit}");
        let tree = subject(&repo, "HEAD^{tree}");
        assert_eq!(record["commit_oid"].as_str(), Some(commit.as_str()));
        assert_eq!(record["tree_oid"].as_str(), Some(tree.as_str()));
        assert!(record.get("jeryu_url").is_none());
    }
}

fn assert_family_clean(root: &Path) {
    for name in REPOSITORIES {
        assert_eq!(git_output(&root.join(name), &["status", "--porcelain"]), "");
    }
}

fn assert_error(output: &Output, code: &str) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(code),
        "{output:?}"
    );
}

fn subject(repo: &Path, revision: &str) -> String {
    format!(
        "sha1:{}",
        git_output(repo, &["rev-parse", "--verify", revision]).trim()
    )
}

fn sign_tag(repo: &Path, key: &Path) {
    git(
        repo,
        &[
            "-c",
            "gpg.format=ssh",
            "-c",
            &format!("user.signingkey={}", key.display()),
            "tag",
            "-s",
            "-m",
            "Exact fuse fixture release",
            TAG,
        ],
    );
}

fn git(repo: &Path, args: &[&str]) {
    let output = git_command(repo, args);
    assert!(output.status.success(), "fixture Git failed: {output:?}");
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = git_command(repo, args);
    assert!(output.status.success(), "fixture Git failed: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn git_command(repo: &Path, args: &[&str]) -> Output {
    Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap()
}

fn command(cwd: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {output:?}"
    );
}

fn fixture_root() -> PathBuf {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("bullet-fuse-cli-{}-{sequence}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir(&root).unwrap();
    root
}

fn text(path: &Path) -> &str {
    path.to_str().expect("fixture path is UTF-8")
}
