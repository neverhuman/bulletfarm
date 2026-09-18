use std::{fs, path::Path, process::Command};

use bullet_family::family_lock::{
    ExternalSubjectManifest, ExternalSubjects, JeryuSubject, PortalSubject, ProviderSubject,
    ReleaseSigningSubject, SandboxSubject, ToolchainSubject, load, verify_hub_checkout,
    verify_locked_checkout,
};

const TAG: &str = "v1.0.0-fixture.1";
const PRINCIPAL: &str = "fixture@bullet.invalid";

#[test]
fn signed_generation_and_exact_checkout_verification_are_non_circular() {
    let root = fixture_root();
    let key = root.join("release-key");
    command(
        &root,
        "/usr/bin/ssh-keygen",
        &["-q", "-t", "ed25519", "-N", "", "-f", text(&key)],
    );
    let public_key = fs::read_to_string(key.with_extension("pub")).unwrap();
    let allowed = format!("{PRINCIPAL} namespaces=\"git\" {}\n", public_key.trim());

    let hub = root.join("bullet-farm");
    init_repo(&hub, "bullet-farm", &key, Some(&allowed));
    for name in ["bullet-git", "bullet-kernel", "bullet-portal"] {
        init_repo(&root.join(name), name, &key, None);
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
    .expect("generation does not require a pre-existing hub tag");
    assert!(hub.join("family.lock").is_file());
    assert!(git_ok(&hub, &["rev-parse", &format!("refs/tags/{TAG}")]).is_none());

    git(&hub, &["add", "family.lock"]);
    git(&hub, &["commit", "-m", "Bind fixture family"]);
    sign_tag(&hub, &key);

    let lock = load(&hub.join("family.lock")).expect("strict generated lock");
    assert_eq!(
        lock.member.len(),
        3,
        "hub must not be serialized as a member"
    );
    assert!(lock.member("bullet-farm").is_none());
    let allowed_signers = hub.join("release/allowed_signers");
    let signer = verify_hub_checkout(&lock, &hub, &allowed_signers).expect("signed hub subject");
    assert!(signer.starts_with(&format!("{PRINCIPAL}|ed25519|SHA256:")));
    verify_locked_checkout(
        lock.member("bullet-kernel").unwrap(),
        &root.join("bullet-kernel"),
        &allowed_signers,
    )
    .expect("exact signed non-hub subject");
    bullet_family::family_lock::run(&root, &["verify".into(), "--tag".into(), TAG.into()])
        .expect("complete lock verification");

    fs::write(
        root.join("repos.manifest.toml"),
        concat!(
            "family = \"bullet-farm\"\n",
            "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
        ),
    )
    .unwrap();
    bullet_family::family_lock::run(&root, &["verify".into(), "--tag".into(), TAG.into()])
        .expect("verification needs no mutable source table");

    let kernel = root.join("bullet-kernel");
    fs::write(kernel.join("changed-after-tag.txt"), "new head\n").unwrap();
    git(&kernel, &["add", "changed-after-tag.txt"]);
    git(&kernel, &["commit", "-m", "Move past locked subject"]);
    let error = verify_locked_checkout(
        lock.member("bullet-kernel").unwrap(),
        &kernel,
        &allowed_signers,
    )
    .expect_err("changed HEAD must not satisfy the exact lock");
    assert_eq!(error.code(), "LOCKED_COMMIT_MISMATCH");
    fs::remove_dir_all(root).unwrap();
}

fn init_repo(repo: &Path, name: &str, key: &Path, allowed_signers: Option<&str>) {
    fs::create_dir_all(repo.join("agent")).unwrap();
    let lockfile = if name == "bullet-portal" {
        "package-lock.json"
    } else {
        "Cargo.lock"
    };
    fs::write(repo.join(lockfile), "# fixture dependency lock\n").unwrap();
    fs::write(repo.join("generated.txt"), "generated fixture\n").unwrap();
    fs::write(
        repo.join("agent/generated-zones.toml"),
        concat!(
            "[[zone]]\n",
            "path = \"generated.txt\"\n",
            "source = \"fixture generator\"\n",
            "owner = \"contracts\"\n",
        ),
    )
    .unwrap();
    if let Some(allowed) = allowed_signers {
        fs::create_dir_all(repo.join("release")).unwrap();
        fs::create_dir_all(repo.join("crates/bullet-wire")).unwrap();
        fs::write(repo.join("release/allowed_signers"), allowed).unwrap();
        fs::write(
            repo.join("crates/bullet-wire/schema.rs"),
            "pub struct Fixture;\n",
        )
        .unwrap();
    }
    command(repo, "/usr/bin/git", &["init", "--quiet"]);
    git(repo, &["config", "user.name", "Fixture Release"]);
    git(repo, &["config", "user.email", PRINCIPAL]);
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "Fixture source"]);
    if allowed_signers.is_none() {
        sign_tag(repo, key);
    }
}

fn external_subjects(root: &Path, public_key: &str, allowed: &str) -> ExternalSubjects {
    let portal = root.join("bullet-portal");
    let fingerprint = ssh_fingerprint(&root.join("release-key.pub"));
    let identity = format!("{PRINCIPAL}|ed25519|{fingerprint}");
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
            source_commit_oid: tagged_subject(&portal, "commit"),
            source_tree_oid: tagged_subject(&portal, "tree"),
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
            identity,
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
    format!(
        "sha1:{}",
        git_ok(repo, &["rev-parse", &format!("{TAG}{suffix}")])
            .unwrap()
            .trim()
    )
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
            "Fixture release",
            TAG,
        ],
    );
}

fn git(repo: &Path, args: &[&str]) {
    command(repo, "/usr/bin/git", args);
}

fn git_ok(repo: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).unwrap())
}

fn command(cwd: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fixture_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("bullet-family-lock-git-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir(&root).unwrap();
    root
}

fn text(path: &Path) -> &str {
    path.to_str().expect("fixture path is UTF-8")
}
