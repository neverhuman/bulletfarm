use std::{fs, path::PathBuf, process::Command};

use bullet_family::family_lock::{
    ExternalSubjectManifest, ExternalSubjects, FamilyLock, JeryuSubject, LOCK_SCHEMA_VERSION,
    LockedFile, LockedHub, LockedMember, PortalSubject, ProviderSubject, ReleaseSigningSubject,
    SandboxSubject, ToolchainSubject, load, parse,
};

const IDENTITY: &str = "release@bullet.farm|ed25519|SHA256:abc+123=";

fn digest(byte: char) -> String {
    format!("blake3:{}", byte.to_string().repeat(64))
}

fn oid(byte: char) -> String {
    format!("sha1:{}", byte.to_string().repeat(40))
}

fn file(path: &str, byte: char) -> LockedFile {
    LockedFile {
        path: path.to_owned(),
        digest: digest(byte),
    }
}

fn valid_lock() -> FamilyLock {
    let member = vec![
        member("bullet-git", 'b', 'c'),
        member("bullet-kernel", 'd', 'e'),
        member("bullet-portal", 'f', '1'),
    ];
    FamilyLock {
        schema_version: LOCK_SCHEMA_VERSION.to_owned(),
        family: "bullet-farm".to_owned(),
        tag: "v1.0.0".to_owned(),
        schema_bundle_hash: digest('a'),
        hub: LockedHub {
            name: "bullet-farm".to_owned(),
            tag: "v1.0.0".to_owned(),
            release_signing_identity: IDENTITY.to_owned(),
        },
        member,
        external: valid_external(),
    }
}

fn member(name: &str, commit: char, tree: char) -> LockedMember {
    let lockfile = if name == "bullet-portal" {
        "package-lock.json"
    } else {
        "Cargo.lock"
    };
    LockedMember {
        name: name.to_owned(),
        jeryu_url: Some(format!("https://jeryu.example/git/root/{name}.git")),
        jeryu_slug: Some(format!("root/{name}")),
        tag: "v1.0.0".to_owned(),
        commit_oid: oid(commit),
        tree_oid: oid(tree),
        release_signing_identity: IDENTITY.to_owned(),
        lockfile: vec![file(lockfile, '2')],
        artifact: vec![
            file("contracts/generated/a.txt", '3'),
            file("contracts/generated/b.txt", '4'),
        ],
    }
}

fn valid_external() -> ExternalSubjects {
    ExternalSubjects {
        toolchain: vec![
            ToolchainSubject {
                id: "cargo".to_owned(),
                version: "1.95.0".to_owned(),
                install_path: "/usr/lib/bullet/toolchains/rust/1.95.0/cargo".to_owned(),
                binary_digest: digest('4'),
                manifest_path: "/usr/lib/bullet/toolchains/rust/1.95.0/manifest.toml".to_owned(),
                manifest_digest: digest('5'),
                size_bytes: 1,
            },
            ToolchainSubject {
                id: "node".to_owned(),
                version: "22.23.2".to_owned(),
                install_path: "/usr/lib/bullet/toolchains/node/22.23.2/node".to_owned(),
                binary_digest: digest('6'),
                manifest_path: "/usr/lib/bullet/toolchains/node/22.23.2/manifest.toml".to_owned(),
                manifest_digest: digest('7'),
                size_bytes: 2,
            },
            ToolchainSubject {
                id: "npm-cli".to_owned(),
                version: "10.9.8".to_owned(),
                install_path: "/usr/lib/bullet/toolchains/npm/10.9.8/npm-cli.js".to_owned(),
                binary_digest: digest('8'),
                manifest_path: "/usr/lib/bullet/toolchains/npm/10.9.8/manifest.toml".to_owned(),
                manifest_digest: digest('9'),
                size_bytes: 3,
            },
        ],
        provider: vec![ProviderSubject {
            id: "claude".to_owned(),
            version: "1.0.0".to_owned(),
            profile: "service".to_owned(),
            install_path: "/usr/lib/bullet/providers/claude/1.0.0/claude".to_owned(),
            binary_digest: digest('6'),
            protocol_digest: digest('7'),
            size_bytes: 2,
        }],
        portal: PortalSubject {
            id: "portal".to_owned(),
            version: "1.0.0".to_owned(),
            source_commit_oid: oid('f'),
            source_tree_oid: oid('1'),
            install_path: "/usr/lib/bullet/portal/1.0.0/index.html".to_owned(),
            bundle_digest: digest('8'),
            manifest_digest: digest('9'),
            size_bytes: 3,
        },
        sandbox: vec![SandboxSubject {
            id: "sandbox-s1".to_owned(),
            class: "s1".to_owned(),
            platform: "ubuntu-24-04-x86-64".to_owned(),
            install_path: "/usr/lib/bullet/sandboxes/s1/rootfs.img".to_owned(),
            image_digest: digest('a'),
            policy_digest: digest('b'),
            size_bytes: 4,
        }],
        jeryu: JeryuSubject {
            id: "jeryu".to_owned(),
            version: "1.0.0".to_owned(),
            tag: "v1.0.0".to_owned(),
            install_path: "/usr/lib/bullet/jeryu/1.0.0/jeryu".to_owned(),
            manifest_digest: digest('c'),
            binary_digest: digest('d'),
            api_schema_digest: digest('e'),
            capability_digest: digest('f'),
            sbom_digest: digest('1'),
            provenance_digest: digest('2'),
            signature_digest: digest('3'),
            size_bytes: 5,
        },
        release_signing: ReleaseSigningSubject {
            id: "release-signing".to_owned(),
            identity: IDENTITY.to_owned(),
            policy_digest: digest('4'),
            allowed_signers_digest: digest('5'),
            key_digest: digest('6'),
            policy_path: "/etc/bullet/release/allowed_signers".to_owned(),
            not_before_unix_ms: 1,
            not_after_unix_ms: 2,
        },
    }
}

fn encoded(lock: &FamilyLock) -> Vec<u8> {
    toml::to_string_pretty(lock)
        .expect("encode fixture")
        .into_bytes()
}

type LockMutation = Box<dyn Fn(&mut FamilyLock)>;

#[test]
fn strict_schema_accepts_complete_install_authority() {
    let lock = parse(&encoded(&valid_lock())).expect("valid schema 3 lock");
    assert_eq!(lock.member("bullet-kernel").unwrap().tree_oid, oid('e'));
    assert!(lock.member("bullet-farm").is_none());
    lock.validate_required_members(&[
        "bullet-farm".into(),
        "bullet-kernel".into(),
        "bullet-git".into(),
        "bullet-portal".into(),
    ])
    .expect("exact member set");
    assert!(
        lock.validate_required_members(&["bullet-farm".into()])
            .is_err()
    );
}

#[test]
fn strict_schema_rejects_hostile_identity_and_path_mutations() {
    let mutations: Vec<LockMutation> = vec![
        Box::new(|lock| lock.member[0].commit_oid = "a".repeat(40)),
        Box::new(|lock| lock.member[0].tree_oid = format!("sha1:{}", "A".repeat(40))),
        Box::new(|lock| lock.member[0].artifact[0].path = "../escape".to_owned()),
        Box::new(|lock| lock.member[0].artifact[0].path = ".git/config".to_owned()),
        Box::new(|lock| lock.member[0].artifact.swap(0, 1)),
        Box::new(|lock| lock.member[0].jeryu_url = None),
        Box::new(|lock| {
            lock.member[0].jeryu_url =
                Some("https://jeryu.example/git/root/different.git".to_owned());
        }),
        Box::new(|lock| lock.member[0].lockfile[0].digest = digest('A')),
        Box::new(|lock| lock.member[0].name = "bullet-farm".to_owned()),
        Box::new(|lock| {
            lock.member[1].name = lock.member[0].name.clone();
        }),
        Box::new(|lock| {
            lock.member.pop();
        }),
        Box::new(|lock| lock.external.toolchain[0].install_path = "relative/rustc".to_owned()),
        Box::new(|lock| lock.external.toolchain[0].manifest_path = "relative/manifest".to_owned()),
        Box::new(|lock| {
            lock.external.toolchain[0].manifest_path =
                lock.external.toolchain[0].install_path.clone();
        }),
        Box::new(|lock| {
            lock.external.toolchain[0].manifest_path =
                lock.external.toolchain[1].install_path.clone();
        }),
        Box::new(|lock| lock.external.toolchain[0].version = "1.95".to_owned()),
        Box::new(|lock| lock.external.toolchain[1].version = "26.1.0".to_owned()),
        Box::new(|lock| lock.external.toolchain[2].version = "11.13.0".to_owned()),
        Box::new(|lock| lock.external.provider[0].binary_digest = "a".repeat(64)),
        Box::new(|lock| lock.external.provider[0].id = "cargo".to_owned()),
        Box::new(|lock| lock.external.portal.source_commit_oid = oid('a')),
        Box::new(|lock| lock.external.portal.size_bytes = 9_007_199_254_740_992),
        Box::new(|lock| lock.external.toolchain.clear()),
        Box::new(|lock| {
            lock.external.toolchain.remove(0);
        }),
        Box::new(|lock| lock.external.provider.clear()),
        Box::new(|lock| lock.external.sandbox.clear()),
        Box::new(|lock| lock.hub.release_signing_identity = "wrong".to_owned()),
    ];
    for mutate in mutations {
        let mut lock = valid_lock();
        mutate(&mut lock);
        assert!(parse(&encoded(&lock)).is_err());
    }
}

#[test]
fn subject_manifest_parser_rejects_hostile_nested_inputs() {
    let manifest = ExternalSubjectManifest::new(valid_external());
    let valid = String::from_utf8(manifest.encode().unwrap()).unwrap();
    assert!(valid.contains("schema_version = \"2\""));
    ExternalSubjectManifest::parse(valid.as_bytes()).expect("strict subject manifest");

    let unknown = valid.replacen(
        "profile = \"service\"",
        "profile = \"service\"\ncaller_selected = true",
        1,
    );
    assert!(ExternalSubjectManifest::parse(unknown.as_bytes()).is_err());

    for mutate in [
        |external: &mut ExternalSubjects| external.provider[0].id = "cargo".to_owned(),
        |external: &mut ExternalSubjects| external.toolchain[0].install_path = "bin/rustc".into(),
        |external: &mut ExternalSubjects| {
            external.release_signing.not_after_unix_ms = 9_007_199_254_740_992;
        },
        |external: &mut ExternalSubjects| external.provider.clear(),
    ] {
        let mut hostile = ExternalSubjectManifest::new(valid_external());
        mutate(&mut hostile.external);
        let bytes = toml::to_string_pretty(&hostile).unwrap();
        assert!(ExternalSubjectManifest::parse(bytes.as_bytes()).is_err());
    }

    let mut missing: toml::Value = toml::from_str(&valid).unwrap();
    missing
        .get_mut("external")
        .and_then(toml::Value::as_table_mut)
        .unwrap()
        .remove("portal");
    assert!(ExternalSubjectManifest::parse(toml::to_string(&missing).unwrap().as_bytes()).is_err());
}

#[test]
fn strict_schema_rejects_unknown_duplicate_legacy_and_oversized_documents() {
    let text = String::from_utf8(encoded(&valid_lock())).unwrap();
    assert!(parse(format!("{text}\nunexpected = true\n").as_bytes()).is_err());
    let nested_unknown = text.replacen("id = \"portal\"", "id = \"portal\"\nunexpected = true", 1);
    assert!(parse(nested_unknown.as_bytes()).is_err());
    assert!(
        parse(
            text.replacen(
                "schema_version",
                "schema_version = \"3\"\nschema_version",
                1
            )
            .as_bytes()
        )
        .is_err()
    );
    assert_eq!(
        parse(b"schema_version = \"2\"\n").unwrap_err().code(),
        "UNSUPPORTED_SCHEMA"
    );
    assert!(parse(&vec![b'a'; 1024 * 1024 + 1]).is_err());
}

#[test]
fn unsupported_lock_guidance_preserves_the_diagnostic_file() {
    let root = fixture_root("unsupported-guidance");
    let path = root.join("family.lock");
    let bytes = b"schema_version = \"2\"\nfamily = \"bullet-farm\"\n";
    fs::write(&path, bytes).unwrap();
    let error = load(&path).expect_err("schema 2 must remain unsupported");
    assert_eq!(error.code(), "UNSUPPORTED_SCHEMA");
    assert!(error.to_string().contains("retain it for diagnosis"));
    assert!(error.to_string().contains("replace it atomically"));
    assert!(!error.to_string().contains("remove it"));
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn lock_loader_rejects_symlinks() {
    use std::os::unix::fs::symlink;

    let root = fixture_root("symlink");
    let target = root.join("target.lock");
    let link = root.join("family.lock");
    fs::write(&target, encoded(&valid_lock())).unwrap();
    symlink(&target, &link).unwrap();
    assert!(load(&link).is_err());

    let subject_target = root.join("subjects.toml");
    let subject_link = root.join("subjects-link.toml");
    fs::write(
        &subject_target,
        ExternalSubjectManifest::new(valid_external())
            .encode()
            .unwrap(),
    )
    .unwrap();
    symlink(&subject_target, &subject_link).unwrap();
    assert!(ExternalSubjectManifest::load(&subject_link).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generation_fails_before_write_without_authenticated_sources() {
    let root = fixture_root("missing-source");
    for member in [
        "bullet-farm",
        "bullet-kernel",
        "bullet-git",
        "bullet-portal",
    ] {
        fs::create_dir(root.join(member)).unwrap();
    }
    let subjects = root.join("subjects.toml");
    fs::write(
        &subjects,
        ExternalSubjectManifest::new(valid_external())
            .encode()
            .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("repos.manifest.toml"),
        format!(
            concat!(
                "family = \"bullet-farm\"\n",
                "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
                "[[repo]]\nname = \"bullet-farm\"\npath = \"{root}/bullet-farm\"\n",
                "[[repo]]\nname = \"bullet-kernel\"\npath = \"{root}/bullet-kernel\"\n",
                "jeryu_slug = \"root/bullet-kernel\"\n",
                "[[repo]]\nname = \"bullet-git\"\npath = \"{root}/bullet-git\"\n",
                "jeryu_url = \"https://jeryu.example/git/root/bullet-git.git\"\n",
                "jeryu_slug = \"root/bullet-git\"\n",
                "[[repo]]\nname = \"bullet-portal\"\npath = \"{root}/bullet-portal\"\n",
                "jeryu_url = \"https://jeryu.example/git/root/bullet-portal.git\"\n",
                "jeryu_slug = \"root/bullet-portal\"\n",
            ),
            root = root.display()
        ),
    )
    .unwrap();
    let error = bullet_family::family_lock::run(
        &root,
        &[
            "generate".into(),
            "--tag".into(),
            "v1.0.0".into(),
            "--subjects".into(),
            subjects.display().to_string(),
        ],
    )
    .expect_err("missing URL must block before Git/tag access");
    assert_eq!(error.code(), "SOURCE_METADATA_UNAVAILABLE");
    assert!(!root.join("bullet-farm/family.lock").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generation_rejects_a_relative_subject_manifest_before_writing() {
    let root = fixture_root("relative-subjects");
    let hub = root.join("bullet-farm");
    fs::create_dir(&hub).unwrap();
    fs::write(
        root.join("repos.manifest.toml"),
        "family = \"bullet-farm\"\nrequired_repos = []\n",
    )
    .unwrap();
    let error = bullet_family::family_lock::run(
        &root,
        &[
            "generate".into(),
            "--tag".into(),
            "v1.0.0".into(),
            "--subjects".into(),
            "subjects.toml".into(),
        ],
    )
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_LOCK_SUBJECTS");
    assert!(!hub.join("family.lock").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unsigned_tag_is_refused() {
    let root = fixture_root("unsigned-tag");
    let allowed = b"fixture@bullet.invalid namespaces=\"git\" ssh-ed25519 AAAA\n";
    let mut external = valid_external();
    external.release_signing.allowed_signers_digest = format!(
        "blake3:{}",
        "d76b04d5cb728135bb77b766e2ec46c5c8c9d8da737cc87c5ca40b8ff1847554"
    );

    for name in [
        "bullet-farm",
        "bullet-kernel",
        "bullet-git",
        "bullet-portal",
    ] {
        let repo = root.join(name);
        fs::create_dir_all(&repo).unwrap();
        if name == "bullet-farm" {
            fs::create_dir_all(repo.join("release")).unwrap();
            fs::create_dir_all(repo.join("crates/bullet-wire")).unwrap();
            fs::write(repo.join("release/allowed_signers"), allowed).unwrap();
            fs::write(
                repo.join("crates/bullet-wire/lib.rs"),
                "pub fn fixture() {}\n",
            )
            .unwrap();
        }
        init_git_with_lightweight_tag(&repo);
    }

    fs::write(
        root.join("repos.manifest.toml"),
        format!(
            concat!(
                "family = \"bullet-farm\"\n",
                "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
                "[[repo]]\nname = \"bullet-farm\"\npath = \"{root}/bullet-farm\"\n",
                "[[repo]]\nname = \"bullet-kernel\"\npath = \"{root}/bullet-kernel\"\n",
                "jeryu_url = \"https://jeryu.example/git/root/bullet-kernel.git\"\n",
                "jeryu_slug = \"root/bullet-kernel\"\n",
                "[[repo]]\nname = \"bullet-git\"\npath = \"{root}/bullet-git\"\n",
                "jeryu_url = \"https://jeryu.example/git/root/bullet-git.git\"\n",
                "jeryu_slug = \"root/bullet-git\"\n",
                "[[repo]]\nname = \"bullet-portal\"\npath = \"{root}/bullet-portal\"\n",
                "jeryu_url = \"https://jeryu.example/git/root/bullet-portal.git\"\n",
                "jeryu_slug = \"root/bullet-portal\"\n",
            ),
            root = root.display()
        ),
    )
    .unwrap();
    let subjects = root.join("subjects.toml");
    fs::write(
        &subjects,
        ExternalSubjectManifest::new(external).encode().unwrap(),
    )
    .unwrap();

    let error = bullet_family::family_lock::run(
        &root,
        &[
            "generate".into(),
            "--tag".into(),
            "v1.0.0".into(),
            "--subjects".into(),
            subjects.display().to_string(),
        ],
    )
    .expect_err("lightweight tags cannot mint a lock");
    assert_eq!(error.code(), "UNSIGNED_OR_LIGHTWEIGHT_TAG");
    assert!(!root.join("bullet-farm/family.lock").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn html_content_type_is_never_a_capability() {
    let mut lock = valid_lock();
    lock.external.jeryu.capability_digest = "text/html".to_owned();
    let error = bullet_family::family_lock::encode(&lock)
        .expect_err("HTML MIME type is not a capability digest");
    assert_eq!(error.code(), "INVALID_FAMILY_LOCK");
    assert!(error.to_string().contains("capability_digest"));
    assert!(error.to_string().contains("BLAKE3"));

    let hostile = String::from_utf8(encoded(&valid_lock()))
        .unwrap()
        .replace(&valid_lock().external.jeryu.capability_digest, "text/html");
    let parsed = parse(hostile.as_bytes()).expect_err("parse refuses HTML capability");
    assert_eq!(parsed.code(), "INVALID_FAMILY_LOCK");

    let probe = bullet_family::forge::execute(
        vec!["bullet-family".into(), "forge".into(), "probe".into()],
        Ok(std::env::temp_dir()),
    )
    .expect("diagnostic probe");
    assert!(
        !probe.output().to_ascii_lowercase().contains("text/html"),
        "probe must never treat an HTML SPA body as a capability"
    );
}

#[test]
fn public_cli_admits_only_generate_and_verify() {
    let root = fixture_root("cli-vocabulary");
    let hub = root.join("bullet-farm");
    fs::create_dir(&hub).unwrap();
    fs::write(
        root.join("repos.manifest.toml"),
        "family = \"bullet-farm\"\nrequired_repos = [\"bullet-farm\"]\n",
    )
    .unwrap();
    let lock = hub.join("family.lock");
    fs::write(&lock, "schema_version = \"2\"\n").unwrap();

    let usage = lock_cli(&root, &[]);
    assert_eq!(usage.status.code(), Some(2));
    let usage_error = String::from_utf8(usage.stderr).unwrap();
    assert!(
        usage_error.contains(
            "lock <generate --tag VERSION --subjects ABSOLUTE_PATH|verify --tag VERSION>"
        )
    );

    let legacy = lock_cli(&root, &["lock", "check", "--tag", "v1.0.0"]);
    assert_eq!(legacy.status.code(), Some(2));
    let legacy_error = String::from_utf8(legacy.stderr).unwrap();
    assert!(legacy_error.contains("USAGE"));
    assert!(legacy_error.contains("lock generate --tag <version> --subjects <absolute-path>"));
    assert!(legacy_error.contains("lock verify --tag <version>"));
    assert!(!legacy_error.contains("generate|check"));

    let verify = lock_cli(&root, &["lock", "verify", "--tag", "v1.0.0"]);
    assert_eq!(verify.status.code(), Some(4));
    assert!(
        String::from_utf8(verify.stderr)
            .unwrap()
            .contains("UNSUPPORTED_SCHEMA")
    );
    assert_eq!(
        fs::read_to_string(&lock).unwrap(),
        "schema_version = \"2\"\n"
    );
    fs::remove_dir_all(root).unwrap();
}

fn lock_cli(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("run bullet-family lock command")
}

fn init_git_with_lightweight_tag(repo: &std::path::Path) {
    let git = |args: &[&str]| {
        let output = Command::new("/usr/bin/git")
            .current_dir(repo)
            .args(args)
            .env_clear()
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "--quiet"]);
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@bullet.invalid"]);
    fs::write(repo.join("README"), "unsigned fixture\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "Fixture"]);
    git(&["tag", "v1.0.0"]);
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-family-lock-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
