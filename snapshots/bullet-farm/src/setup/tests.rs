use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::*;
use crate::checkout::verify_family;

mod lock_subjects;

use self::lock_subjects::fixture_external_subjects;

pub(super) const TAG: &str = "v1.0.0";
pub(super) const MEMBERS: [&str; 4] = [
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];

#[test]
fn setup_arguments_are_strict() {
    let good = [
        "--root".into(),
        "/tmp/family".into(),
        "--source".into(),
        "jeryu".into(),
        "--offline".into(),
    ];
    let parsed = parse_args(None, &good).unwrap();
    assert!(parsed.offline);
    for denied in [
        vec!["--root".into(), "/tmp/family".into()],
        vec![
            "--source".into(),
            "github".into(),
            "--root".into(),
            "/tmp/family".into(),
        ],
        vec![
            "--offline".into(),
            "--offline".into(),
            "--source".into(),
            "jeryu".into(),
            "--root".into(),
            "/tmp/family".into(),
        ],
    ] {
        assert!(parse_args(None, &denied).is_err());
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
#[test]
fn checkout_publication_never_replaces_a_racing_destination() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("publication-race");
    let root = admitted_root_for_test(&fixture).expect("admit fixture root");
    let staging = root
        .create_staging("checkout-test")
        .expect("private staging");
    let staged = staging.path().join("bullet-kernel");
    let target = fixture.join("bullet-kernel");
    let raced_checkout = fixture.join("raced-checkout");
    fs::create_dir(&staged).expect("staged checkout");
    fs::write(staged.join("candidate"), "new checkout\n").expect("staged sentinel");
    fs::create_dir(&raced_checkout).expect("racing checkout");
    fs::write(raced_checkout.join("sentinel"), "preserve me\n").expect("race sentinel");
    symlink(&raced_checkout, &target).expect("racing destination symlink");

    let error = publish_staged_for_test(&staging, "bullet-kernel")
        .expect_err("publication must not replace a path created after preflight");
    assert_eq!(error.code(), "CHECKOUT_CONFLICT");
    assert!(
        fs::symlink_metadata(&target)
            .expect("racing destination")
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_to_string(raced_checkout.join("sentinel")).expect("preserved race sentinel"),
        "preserve me\n"
    );
    assert!(staged.join("candidate").is_file());
    staging.finish().expect("remove private staging");
    fs::remove_dir_all(fixture).expect("remove publication fixture");
}

#[test]
fn signed_local_family_installs_twice_without_drift() {
    super::validate::assert_synchronizer_link_completeness_for_test();
    let fixture = fixture_root("signed-install");
    let sources = fixture.join("sources");
    let install_root = fixture.join("install");
    let home = fixture.join("home");
    fs::create_dir(&sources).expect("source root");
    fs::create_dir(&install_root).expect("install root");
    fs::create_dir(&home).expect("fresh HOME");
    let signing_key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &signing_key);

    let hub = install_root.join("bullet-farm");
    clone_repository(sources.join("bullet-farm").as_os_str(), &hub, true)
        .expect("clone signed hub");
    test_git(&hub, &home, &["checkout", "--detach", "--force", TAG]);
    let lock = family_lock::load(&hub.join("family.lock")).expect("strict lock");
    let transport = LocalTransport {
        sources: sources.clone(),
        clone_count: Cell::new(0),
    };

    install(&install_root, &hub, &lock, true, &transport).expect("first install");
    assert_eq!(transport.clone_count.get(), 3);
    verify_family(&install_root, &hub, &lock).expect("first exact verification");
    let first_heads = installed_heads(&install_root, &home);

    install(&install_root, &hub, &lock, true, &transport).expect("idempotent install");
    assert_eq!(
        transport.clone_count.get(),
        3,
        "second run recloned members"
    );
    verify_family(&install_root, &hub, &lock).expect("second exact verification");
    assert_eq!(installed_heads(&install_root, &home), first_heads);
    assert_eq!(
        fs::read(install_root.join("repos.manifest.toml")).expect("published manifest"),
        fs::read(hub.join("repos.manifest.toml")).expect("hub manifest")
    );
    assert_no_staging(&install_root);

    let kernel = install_root.join("bullet-kernel");
    let canary = fixture.join("fsmonitor-canary");
    test_git(
        &kernel,
        &home,
        &["update-index", "--assume-unchanged", "Cargo.lock"],
    );
    test_git(
        &kernel,
        &home,
        &[
            "config",
            "core.fsmonitor",
            &format!("touch {}", canary.display()),
        ],
    );
    fs::write(kernel.join("Cargo.lock"), "hidden tracked mutation\n")
        .expect("hidden worktree mutation");
    let error = verify_family(&install_root, &hub, &lock)
        .expect_err("hostile local config and index flags must not hide changed bytes");
    assert_eq!(error.code(), "UNSAFE_GIT_METADATA");
    assert!(
        !canary.exists(),
        "verification executed a hostile fsmonitor"
    );
    test_git(&kernel, &home, &["config", "--unset", "core.fsmonitor"]);
    let error = verify_family(&install_root, &hub, &lock)
        .expect_err("assume-unchanged must not hide changed worktree bytes");
    assert_eq!(error.code(), "DIRTY_CHECKOUT");
    assert!(
        !canary.exists(),
        "verification executed a removed fsmonitor"
    );
    fs::write(kernel.join("Cargo.lock"), "# fixture\n").expect("restore tracked fixture");
    test_git(
        &kernel,
        &home,
        &["update-index", "--no-assume-unchanged", "Cargo.lock"],
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let external = fixture.join("external-generated");
        fs::create_dir(&external).expect("external generated directory");
        fs::write(external.join("out.txt"), "bullet-kernel\n").expect("external exact bytes");
        fs::remove_dir_all(kernel.join("generated")).expect("replace tracked parent");
        symlink(&external, kernel.join("generated")).expect("hostile intermediate symlink");
        let error = verify_family(&install_root, &hub, &lock)
            .expect_err("an intermediate symlink must not satisfy exact tree bytes");
        assert_eq!(error.code(), "DIRTY_CHECKOUT");
    }
    fs::remove_dir_all(fixture).expect("remove signed fixture");
}

#[test]
fn offline_jeryu_cache_miss_preserves_hub_only_root() {
    let fixture = fixture_root("offline-jeryu-cache-miss");
    let sources = fixture.join("sources");
    let install_root = fixture.join("install");
    let home = fixture.join("home");
    fs::create_dir(&sources).expect("source root");
    fs::create_dir(&install_root).expect("install root");
    fs::create_dir(&home).expect("fresh HOME");
    let signing_key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &signing_key);

    let hub = install_root.join("bullet-farm");
    clone_repository(sources.join("bullet-farm").as_os_str(), &hub, true)
        .expect("clone signed hub");
    test_git(&hub, &home, &["checkout", "--detach", "--force", TAG]);
    let lock = family_lock::load(&hub.join("family.lock")).expect("strict lock");
    let hub_head = test_git_output(&hub, &home, &["rev-parse", "HEAD"]);

    let error = install(&install_root, &hub, &lock, true, &JeryuTransport)
        .expect_err("offline setup cannot fetch a missing Jeryu member");
    assert_eq!(error.code(), "OFFLINE_SOURCE_UNAVAILABLE");
    assert_eq!(
        test_git_output(&hub, &home, &["rev-parse", "HEAD"]),
        hub_head
    );
    assert_eq!(
        test_git_output(&hub, &home, &["status", "--porcelain=v2"]),
        ""
    );
    for member in ["bullet-kernel", "bullet-git", "bullet-portal"] {
        assert!(!install_root.join(member).exists(), "published {member}");
    }
    assert!(!install_root.join("repos.manifest.toml").exists());
    assert_no_staging(&install_root);
    assert_eq!(
        fs::read_dir(&install_root).expect("install root").count(),
        1,
        "offline refusal left state beside the signed hub"
    );

    fs::remove_dir_all(fixture).expect("remove offline fixture");
}

#[test]
fn missing_tool_authority_blocks_signed_setup_before_install_mutation() {
    let fixture = fixture_root("missing-tool-authority");
    let sources = fixture.join("sources");
    let install_root = fixture.join("install");
    let home = fixture.join("home");
    fs::create_dir(&sources).expect("source root");
    fs::create_dir(&install_root).expect("install root");
    fs::create_dir(&home).expect("fresh HOME");
    let signing_key = create_signing_key(&fixture);
    create_source_family(&sources, &home, &signing_key);

    let hub = install_root.join("bullet-farm");
    clone_repository(sources.join("bullet-farm").as_os_str(), &hub, true)
        .expect("clone signed hub");
    test_git(&hub, &home, &["checkout", "--detach", "--force", TAG]);

    let error = run(
        &hub,
        None,
        &[
            "--root".into(),
            install_root.to_string_lossy().into_owned(),
            "--source".into(),
            "jeryu".into(),
        ],
    )
    .expect_err("missing explicit tool authority must block setup");
    assert_eq!(error.code(), "SETUP_TOOL_MISSING");
    for member in ["bullet-kernel", "bullet-git", "bullet-portal"] {
        assert!(!install_root.join(member).exists(), "created {member}");
    }
    assert!(!install_root.join("repos.manifest.toml").exists());
    assert_no_staging(&install_root);

    let marker = fixture.join("tampered-tool-executed");
    let fake_tool = fixture.join("tampered-tool");
    fs::write(
        &fake_tool,
        format!("#!/bin/sh\nprintf executed > '{}'\n", marker.display()),
    )
    .expect("tampered tool fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fake_tool, fs::Permissions::from_mode(0o700))
            .expect("tampered tool mode");
    }
    let mut tampered = family_lock::load(&hub.join("family.lock")).expect("signed fixture lock");
    tampered.external.toolchain[0].size_bytes += 1;
    fs::write(
        hub.join("family.lock"),
        family_lock::encode(&tampered).expect("encode tampered lock"),
    )
    .expect("publish tampered lock bytes");
    let error = run(
        &hub,
        None,
        &[
            "--root".into(),
            install_root.to_string_lossy().into_owned(),
            "--source".into(),
            "jeryu".into(),
            "--cargo-bin".into(),
            fake_tool.display().to_string(),
            "--node-bin".into(),
            fake_tool.display().to_string(),
            "--npm-cli".into(),
            fake_tool.display().to_string(),
        ],
    )
    .expect_err("tampered lock must fail before candidate tool execution");
    assert_eq!(error.code(), "HUB_LOCK_MISMATCH");
    assert!(!marker.exists(), "untrusted tool candidate executed");
    assert!(!install_root.join("repos.manifest.toml").exists());
    assert_no_staging(&install_root);
    fs::remove_dir_all(fixture).expect("remove missing tool fixture");
}

pub(super) struct LocalTransport {
    pub(super) sources: PathBuf,
    pub(super) clone_count: Cell<usize>,
}

impl CloneTransport for LocalTransport {
    fn clone_member(
        &self,
        member: &LockedMember,
        destination: &Path,
        _offline: bool,
    ) -> Result<(), CoordError> {
        self.clone_count.set(self.clone_count.get() + 1);
        clone_repository(
            self.sources.join(&member.name).as_os_str(),
            destination,
            true,
        )?;
        let origin = member
            .jeryu_url
            .as_deref()
            .expect("validated fixture Jeryu URL");
        run_git(
            Some(destination),
            &[
                OsStr::new("remote"),
                OsStr::new("set-url"),
                OsStr::new("origin"),
                OsStr::new(origin),
            ],
        )
    }
}

pub(super) fn create_source_family(root: &Path, home: &Path, signing_key: &Path) {
    for member in MEMBERS {
        let repo = root.join(member);
        fs::create_dir(&repo).expect("member directory");
        test_git(&repo, home, &["init", "--initial-branch=main"]);
        test_git(&repo, home, &["config", "user.name", "Bullet Fixture"]);
        test_git(
            &repo,
            home,
            &["config", "user.email", "release@bullet.farm"],
        );
        test_git(&repo, home, &["config", "gpg.format", "ssh"]);
        test_git(
            &repo,
            home,
            &[
                "config",
                "user.signingkey",
                signing_key.to_str().expect("UTF-8 key path"),
            ],
        );
        write_member_files(&repo, member);
        test_git(&repo, home, &["add", "--all"]);
        test_git(&repo, home, &["commit", "-m", "fixture source"]);
        if member != "bullet-farm" {
            test_git(&repo, home, &["tag", "-s", TAG, "-m", "signed fixture"]);
        }
    }
    write_rich_manifest(root);
    let subjects = root.join("external-subjects.toml");
    fs::write(
        &subjects,
        family_lock::ExternalSubjectManifest::new(fixture_external_subjects(root, home))
            .encode()
            .expect("encode external subjects"),
    )
    .expect("write external subjects");
    family_lock::run(
        root,
        &[
            "generate".into(),
            "--tag".into(),
            TAG.into(),
            "--subjects".into(),
            subjects.display().to_string(),
        ],
    )
    .expect("generate non-circular lock");
    let hub = root.join("bullet-farm");
    test_git(&hub, home, &["add", "family.lock"]);
    test_git(&hub, home, &["commit", "-m", "bind family lock"]);
    test_git(&hub, home, &["tag", "-s", TAG, "-m", "signed hub"]);
    family_lock::run(root, &["verify".into(), "--tag".into(), TAG.into()])
        .expect("verify signed family");
}

fn write_member_files(repo: &Path, member: &str) {
    fs::create_dir_all(repo.join("agent")).expect("agent directory");
    fs::create_dir_all(repo.join("generated")).expect("generated directory");
    fs::write(
        repo.join("agent/generated-zones.toml"),
        "[[zone]]\npath = \"generated/out.txt\"\nsource = \"fixture\"\nowner = \"fixture\"\n",
    )
    .expect("generated zones");
    fs::write(repo.join("generated/out.txt"), format!("{member}\n")).expect("artifact");
    fs::write(repo.join(".gitignore"), "target/\nnode_modules/\n").expect("ignored tool outputs");
    if member == "bullet-portal" {
        fs::write(repo.join("package-lock.json"), "{}\n").expect("npm lock");
    } else {
        fs::write(repo.join("Cargo.lock"), "# fixture\n").expect("Cargo lock");
    }
    if member == "bullet-farm" {
        fs::create_dir_all(repo.join("scripts")).expect("hub scripts directory");
        fs::write(
            repo.join("Cargo.toml"),
            "[package]\nname = \"fixture-hub\"\nversion = \"0.0.0\"\n",
        )
        .expect("hub Cargo manifest");
        fs::write(repo.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").expect("hub setup script");
        fs::create_dir_all(repo.join("crates/bullet-wire")).expect("wire schema directory");
        fs::create_dir_all(repo.join("release")).expect("release directory");
        fs::write(repo.join("crates/bullet-wire/schema.txt"), "wire-v1\n").expect("wire schema");
        fs::write(
            repo.join("repos.manifest.toml"),
            concat!(
                "schema_version = \"1.2.0\"\n",
                "family = \"bullet-farm\"\n",
                "umbrella_repo = \"bullet-farm\"\n",
                "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
            ),
        )
        .expect("hub manifest");
        let public_key =
            fs::read_to_string(repo.parent().unwrap().parent().unwrap().join("signing.pub"))
                .expect("public key");
        let mut fields = public_key.split_whitespace();
        let algorithm = fields.next().expect("key algorithm");
        let key = fields.next().expect("public key body");
        fs::write(
            repo.join("release/allowed_signers"),
            format!("release@bullet.farm {algorithm} {key}\n"),
        )
        .expect("allowed signers");
    }
}

fn write_rich_manifest(root: &Path) {
    let mut text = String::from(
        "family = \"bullet-farm\"\nrequired_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
    );
    for member in MEMBERS {
        text.push_str(&format!(
            "[[repo]]\nname = \"{member}\"\npath = \"{}/{member}\"\n",
            root.display()
        ));
        if member != "bullet-farm" {
            text.push_str(&format!(
                "jeryu_url = \"http://127.0.0.1:8787/git/root/{member}.git\"\njeryu_slug = \"root/{member}\"\n"
            ));
        }
    }
    fs::write(root.join("repos.manifest.toml"), text).expect("rich manifest");
}

pub(super) fn create_signing_key(root: &Path) -> PathBuf {
    let key = root.join("signing");
    let status = Command::new("/usr/bin/ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key)
        .status()
        .expect("run ssh-keygen");
    assert!(status.success(), "ssh-keygen failed");
    key
}

pub(super) fn installed_heads(root: &Path, home: &Path) -> Vec<String> {
    MEMBERS
        .iter()
        .map(|member| {
            let repo = root.join(member);
            assert!(repo.join(".git").is_dir(), "{member} is not ordinary");
            assert_eq!(
                test_git_output(&repo, home, &["status", "--porcelain=v2"]),
                ""
            );
            test_git_output(&repo, home, &["rev-parse", "HEAD"])
        })
        .collect()
}

pub(super) fn assert_no_staging(root: &Path) {
    let leftovers = fs::read_dir(root)
        .expect("family root")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(STAGING_PREFIX))
        .collect::<Vec<_>>();
    assert!(leftovers.is_empty(), "staging leftovers: {leftovers:?}");
}

pub(super) fn test_git(repo: &Path, home: &Path, args: &[&str]) {
    let output = git_command(repo, home, args).output().expect("run Git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn test_git_output(repo: &Path, home: &Path, args: &[&str]) -> String {
    let output = git_command(repo, home, args).output().expect("run Git");
    assert!(output.status.success(), "git {args:?} failed");
    String::from_utf8(output.stdout)
        .expect("UTF-8 Git output")
        .trim()
        .to_owned()
}

fn git_command(repo: &Path, home: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(GIT_BIN);
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    command
}

pub(super) fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bullet-setup-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    fs::create_dir(&root).expect("fixture root");
    root
}
