use std::{
    ffi::OsString,
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
};

use bullet_family::coord::CoordError;
use sha2::{Digest, Sha256};

fn run(root: &Path, action: &str, args: &[String]) -> Result<String, CoordError> {
    let mut argv = vec![
        OsString::from("bullet-family"),
        OsString::from("--root"),
        root.as_os_str().to_os_string(),
        OsString::from("coord"),
        OsString::from(action),
    ];
    argv.extend(args.iter().map(OsString::from));
    bullet_family::cli::run(argv, Ok(root.to_path_buf()))
}

fn family() -> tempfile::TempDir {
    let family = tempfile::tempdir().unwrap();
    fs::write(family.path().join("repos.manifest.toml"), "version = 1\n").unwrap();
    family
}

#[cfg(target_os = "linux")]
struct ProvenanceFixture {
    family: tempfile::TempDir,
    output: tempfile::TempDir,
    repository: PathBuf,
    commit: String,
    cargo: PathBuf,
    rustc: PathBuf,
}

#[cfg(target_os = "linux")]
impl ProvenanceFixture {
    fn new() -> Self {
        let family = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let repository = family.path().join("bullet-farm");
        fs::create_dir(&repository).unwrap();
        git(&repository, &["init", "--quiet"]);
        let cargo = rustup_tool("cargo");
        let rustc = rustup_tool("rustc");
        let release = version_field(&rustc, "release:");
        fs::create_dir(repository.join("src")).unwrap();
        fs::write(repository.join("Cargo.lock"), b"version = 4\n").unwrap();
        fs::write(
            repository.join("rust-toolchain.toml"),
            format!("[toolchain]\nchannel = \"{release}\"\n"),
        )
        .unwrap();
        fs::write(repository.join("src/lib.rs"), b"pub fn fixture() {}\n").unwrap();
        git(&repository, &["add", "--all"]);
        git(&repository, &["commit", "--quiet", "-m", "fixture"]);
        let commit = git_line(&repository, &["rev-parse", "--verify", "HEAD^{commit}"]);
        fs::write(
            family.path().join("repos.manifest.toml"),
            format!(
                "[[repo]]\nname = \"bullet-farm\"\npath = \"{}\"\n",
                repository.display()
            ),
        )
        .unwrap();
        Self {
            family,
            output,
            repository,
            commit,
            cargo,
            rustc,
        }
    }

    fn args(&self, output: &Path) -> Vec<String> {
        vec![
            "--bootstrap-commit".to_owned(),
            self.commit.clone(),
            "--cargo-bin".to_owned(),
            self.cargo.display().to_string(),
            "--rustc-bin".to_owned(),
            self.rustc.display().to_string(),
            "--output".to_owned(),
            output.display().to_string(),
            "--source-archive-output".to_owned(),
            output.with_extension("tar").display().to_string(),
        ]
    }

    fn produce(&self, name: &str) -> PathBuf {
        let output = self.output.path().join(name);
        run(
            self.family.path(),
            "recovery-provenance",
            &self.args(&output),
        )
        .unwrap();
        output
    }

    fn error(&self, args: &[String]) -> CoordError {
        run(self.family.path(), "recovery-provenance", args).unwrap_err()
    }
}

#[cfg(target_os = "linux")]
fn git(repository: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Bullet Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_NAME", "Bullet Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[cfg(target_os = "linux")]
fn git_line(repository: &Path, args: &[&str]) -> String {
    String::from_utf8(git(repository, args))
        .unwrap()
        .trim_end()
        .to_owned()
}

#[cfg(target_os = "linux")]
fn rustup_tool(name: &str) -> PathBuf {
    let output = Command::new("rustup")
        .args(["which", name])
        .output()
        .unwrap();
    assert!(output.status.success());
    fs::canonicalize(String::from_utf8(output.stdout).unwrap().trim()).unwrap()
}

#[cfg(target_os = "linux")]
fn version_field(tool: &Path, prefix: &str) -> String {
    let output = Command::new(tool)
        .arg("-Vv")
        .env_clear()
        .env("LC_ALL", "C")
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
        .unwrap()
        .to_owned()
}

#[cfg(target_os = "linux")]
fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(target_os = "linux")]
#[test]
fn producer_cli_is_path_only_and_creation_free_before_admission() {
    let family = family();
    let output = family.path().join("plan.json");
    let unknown = run(
        family.path(),
        "recovery-plan",
        &[
            "--output".to_owned(),
            output.display().to_string(),
            "--repo".to_owned(),
            "bullet-kernel".to_owned(),
        ],
    )
    .unwrap_err();
    assert_eq!(unknown.code(), "UNKNOWN_OPTION");
    assert!(!family.path().join(".bullet-family").exists());
    assert!(!output.exists());

    for (action, name) in [
        ("recovery-plan", "output"),
        ("recovery-proof", "plan"),
        ("adopt", "request"),
    ] {
        let error = run(
            family.path(),
            action,
            &[format!("--{name}"), "relative.json".to_owned()],
        )
        .unwrap_err();
        assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
        assert!(!family.path().join(".bullet-family").exists());
    }

    let absent = run(
        family.path(),
        "recovery-plan",
        &["--output".to_owned(), output.display().to_string()],
    )
    .unwrap_err();
    assert_eq!(absent.code(), "COORD_NOT_INITIALIZED");
    assert!(!family.path().join(".bullet-family").exists());
    assert!(!output.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn producer_cli_refuses_unsealed_or_noncanonical_input_before_coord_io() {
    let family = family();
    let document = family.path().join("document.json");
    fs::write(&document, b"{}\n").unwrap();
    let mode_error = run(
        family.path(),
        "recovery-proof",
        &["--plan".to_owned(), document.display().to_string()],
    )
    .unwrap_err();
    assert_eq!(mode_error.code(), "INVALID_RECOVERY_PRODUCTION");

    fs::set_permissions(&document, fs::Permissions::from_mode(0o400)).unwrap();
    let schema_error = run(
        family.path(),
        "recovery-proof",
        &["--plan".to_owned(), document.display().to_string()],
    )
    .unwrap_err();
    assert_eq!(schema_error.code(), "INVALID_RECOVERY_PRODUCTION");
    assert!(!family.path().join(".bullet-family").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn provenance_producer_is_deterministic_complete_and_create_once() {
    let fixture = ProvenanceFixture::new();
    let first = fixture.produce("first.json");
    let second = fixture.produce("second.json");
    let first_bytes = fs::read(&first).unwrap();
    assert_eq!(first_bytes, fs::read(&second).unwrap());
    assert_eq!(first_bytes.last(), Some(&b'\n'));
    assert!(!first_bytes[..first_bytes.len() - 1].contains(&b'\n'));
    let metadata = fs::metadata(&first).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o400);
    assert_eq!(metadata.nlink(), 1);
    let document = bullet_wire::decode_unique_value(&first_bytes).unwrap();
    assert_eq!(document["bootstrap_commit_oid"], fixture.commit);
    let expected_paths = ["Cargo.lock", "rust-toolchain.toml", "src/lib.rs"];
    let paths = document["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|source| source["path"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(paths, expected_paths);
    let lock = fs::read(fixture.repository.join("Cargo.lock")).unwrap();
    assert_eq!(document["cargo_lock_sha256"], sha256(&lock));
    let archive = git(
        &fixture.repository,
        &["archive", "--format=tar", &fixture.commit],
    );
    assert_eq!(fs::read(first.with_extension("tar")).unwrap(), archive);
    assert_eq!(fs::read(second.with_extension("tar")).unwrap(), archive);
    let metadata = fs::metadata(first.with_extension("tar")).unwrap();
    assert_eq!(
        (metadata.permissions().mode() & 0o7777, metadata.nlink()),
        (0o400, 1)
    );
    assert_eq!(document["archive_sha256"], sha256(&archive));
    let executable = fs::read("/proc/self/exe").unwrap();
    assert_eq!(document["executable_sha256"], sha256(&executable));
    assert_eq!(
        document["executable_byte_length"].as_u64(),
        Some(executable.len() as u64)
    );
    assert!(!fixture.family.path().join(".bullet-family").exists());

    let error = fixture.error(&fixture.args(&first));
    assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
    assert_eq!(fs::read(&first).unwrap(), first_bytes);
    let adopted = fixture.output.path().join("adopted.json");
    fs::copy(first.with_extension("tar"), adopted.with_extension("tar")).unwrap();
    fs::set_permissions(
        adopted.with_extension("tar"),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    run(
        fixture.family.path(),
        "recovery-provenance",
        &fixture.args(&adopted),
    )
    .unwrap();
    assert_eq!(fs::read(adopted).unwrap(), first_bytes);
    let differing = fixture.output.path().join("differing.json");
    fs::write(differing.with_extension("tar"), b"different archive").unwrap();
    fs::set_permissions(
        differing.with_extension("tar"),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    assert!(fixture.error(&fixture.args(&differing)).code() == "COORD_SUBJECT_CHANGED");
    assert!(!differing.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn provenance_cli_refuses_option_path_output_and_tool_substitution_before_creation() {
    let fixture = ProvenanceFixture::new();
    let output = fixture.output.path().join("refused.json");

    let missing = run(fixture.family.path(), "recovery-provenance", &[]).unwrap_err();
    assert_eq!(missing.code(), "MISSING_OPTION");

    let mut unknown = fixture.args(&output);
    unknown.extend(["--repo".to_owned(), "bullet-farm".to_owned()]);
    assert_eq!(fixture.error(&unknown).code(), "UNKNOWN_OPTION");

    let mut duplicate = fixture.args(&output);
    duplicate.extend(["--bootstrap-commit".to_owned(), fixture.commit.clone()]);
    assert_eq!(fixture.error(&duplicate).code(), "DUPLICATE_OPTION");

    let mut relative = fixture.args(&output);
    relative[3] = "cargo".to_owned();
    assert_eq!(
        fixture.error(&relative).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );

    let inside = fixture.repository.join("provenance.json");
    assert_eq!(
        fixture.error(&fixture.args(&inside)).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );
    for archive_path in [&output, &fixture.repository.join("archive.tar")] {
        let mut args = fixture.args(&output);
        *args.last_mut().unwrap() = archive_path.display().to_string();
        assert_eq!(fixture.error(&args).code(), "INVALID_RECOVERY_PRODUCTION");
    }
    let link = fixture.output.path().join("cargo-link");
    symlink(&fixture.cargo, &link).unwrap();
    let mut linked_tool = fixture.args(&output);
    linked_tool[3] = link.display().to_string();
    assert_eq!(
        fixture.error(&linked_tool).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );
    let archive_link = fixture.output.path().join("archive-link");
    symlink(&fixture.cargo, &archive_link).unwrap();
    let mut linked_archive = fixture.args(&output);
    *linked_archive.last_mut().unwrap() = archive_link.display().to_string();
    assert_eq!(
        fixture.error(&linked_archive).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );
    assert!(!output.exists());
    let not_elf = fixture.output.path().join("not-elf");
    fs::write(&not_elf, b"#!/bin/false\n").unwrap();
    fs::set_permissions(&not_elf, fs::Permissions::from_mode(0o700)).unwrap();
    let mut invalid_tool = fixture.args(&output);
    invalid_tool[3] = not_elf.display().to_string();
    assert_eq!(
        fixture.error(&invalid_tool).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );

    let privileged = fixture.output.path().join("privileged-rustc");
    fs::copy(&fixture.rustc, &privileged).unwrap();
    fs::set_permissions(&privileged, fs::Permissions::from_mode(0o4755)).unwrap();
    let mut privileged_tool = fixture.args(&output);
    privileged_tool[5] = privileged.display().to_string();
    let custody = fixture.error(&privileged_tool);
    assert_eq!(custody.code(), "INVALID_RECOVERY_PRODUCTION");
    assert!(custody.to_string().contains("non-privileged"));
    assert!(!output.exists());
    assert!(!fixture.family.path().join(".bullet-family").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn provenance_producer_refuses_dirty_mismatched_and_unrepresentable_subjects() {
    let mut fixture = ProvenanceFixture::new();
    let output = fixture.output.path().join("refused.json");
    fs::write(fixture.repository.join("untracked"), b"dirty\n").unwrap();
    let dirty = fixture.error(&fixture.args(&output));
    assert_eq!(dirty.code(), "DIRTY_SOURCE");
    fs::remove_file(fixture.repository.join("untracked")).unwrap();

    let mut mismatch = fixture.args(&output);
    mismatch[1] = "0".repeat(40);
    assert_eq!(
        fixture.error(&mismatch).code(),
        "INVALID_RECOVERY_PRODUCTION"
    );

    fs::write(fixture.repository.join("empty"), b"").unwrap();
    git(&fixture.repository, &["add", "--all"]);
    git(
        &fixture.repository,
        &["commit", "--quiet", "-m", "zero byte"],
    );
    fixture.commit = git_line(
        &fixture.repository,
        &["rev-parse", "--verify", "HEAD^{commit}"],
    );
    let zero = fixture.error(&fixture.args(&output));
    assert_eq!(zero.code(), "INVALID_RECOVERY_PRODUCTION");
    assert!(zero.to_string().contains("zero-byte"));
    assert!(!output.exists());
    assert!(!fixture.family.path().join(".bullet-family").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn provenance_refuses_ambient_attributes_and_export_substitution() {
    let mut fixture = ProvenanceFixture::new();
    let output = fixture.output.path().join("refused.json");
    let ambient = fixture.repository.join(".git/info/attributes");
    fs::write(&ambient, b"* -export-subst\n").unwrap();
    let error = fixture.error(&fixture.args(&output));
    assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
    assert!(error.to_string().contains(".git/info/attributes"));
    fs::remove_file(ambient).unwrap();

    fs::write(
        fixture.repository.join(".gitattributes"),
        b"expanded export-subst\n",
    )
    .unwrap();
    fs::write(fixture.repository.join("expanded"), b"$Format:%H$\n").unwrap();
    git(&fixture.repository, &["add", "--all"]);
    git(
        &fixture.repository,
        &["commit", "--quiet", "-m", "export substitution"],
    );
    fixture.commit = git_line(
        &fixture.repository,
        &["rev-parse", "--verify", "HEAD^{commit}"],
    );
    let error = fixture.error(&fixture.args(&output));
    assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
    assert!(error.to_string().contains("Git blob"));
    assert!(!output.exists());
}

#[cfg(not(target_os = "linux"))]
#[test]
fn producer_cli_refuses_platform_before_options_or_subject_io() {
    let family = family();
    for action in [
        "recovery-provenance",
        "recovery-plan",
        "recovery-proof",
        "recovery-review",
        "recovery-request",
        "adopt",
    ] {
        let error = run(family.path(), action, &[]).unwrap_err();
        assert_eq!(error.code(), "COORD_RECOVERY_PLATFORM_UNSUPPORTED");
        assert!(!family.path().join(".bullet-family").exists());
    }
}
