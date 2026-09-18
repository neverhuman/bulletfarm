use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use super::{
    GitProgram, SignatureInputs,
    subject::{PinnedAllowedSigners, PinnedExecutable},
};
use crate::process::Limits;

const TEST_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(20),
    stdout_bytes: 1024 * 1024,
    stderr_bytes: 1024 * 1024,
};

#[test]
fn pinned_git_path_defeats_post_admission_replacement() {
    let fixture = fixture("git-replacement");
    let repository = fixture.join("repository");
    fs::create_dir_all(repository.join(".git")).expect("repository fixture");
    fs::write(
        repository.join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n",
    )
    .expect("repository config");
    let git = executable(
        &fixture,
        "git-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'git version 2.43.0\\n'; exit 0; fi\n",
            "for argument do case \"$argument\" in --work-tree=*) printf admitted > \"${argument#--work-tree=}/git-ran\";; esac; done\n",
        ),
    );
    let program = GitProgram::admit(&git).expect("admit fixture Git");

    let error = program
        .run_after_verify(
            &repository,
            &["rev-parse", "HEAD"],
            None,
            TEST_LIMITS,
            || {
                let original_git = fixture.join("git-admitted");
                fs::rename(&git, original_git).expect("move admitted Git source");
                executable(
                    &fixture,
                    "git-real",
                    "#!/bin/sh\nprintf attacker > git-attacker\n",
                );
                Ok(())
            },
        )
        .expect_err("Git source replacement must fail after the sealed child");
    assert_eq!(error.code(), "GIT_TOOL_CHANGED");
    assert_eq!(
        fs::read_to_string(repository.join("git-ran")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("git-attacker").exists());
    fs::remove_dir_all(fixture).expect("remove replacement fixture");
}

#[test]
fn pinned_repository_path_defeats_post_admission_replacement() {
    let fixture = fixture("repository-replacement");
    let repository = fixture.join("repository");
    let moved = fixture.join("repository-admitted");
    fs::create_dir_all(repository.join(".git")).expect("repository fixture");
    fs::write(
        repository.join(".git/config"),
        "[core]\n\trepositoryformatversion = 0\n",
    )
    .expect("repository config");
    let git = executable(
        &fixture,
        "git-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'git version 2.43.0\\n'; exit 0; fi\n",
            "for argument do case \"$argument\" in --work-tree=*) printf admitted > \"${argument#--work-tree=}/git-ran\";; esac; done\n",
        ),
    );
    let program = GitProgram::admit(&git).expect("admit fixture Git");

    let error = program
        .run_after_verify(
            &repository,
            &["rev-parse", "HEAD"],
            None,
            TEST_LIMITS,
            || {
                fs::rename(&repository, &moved).expect("move admitted repository");
                fs::create_dir_all(repository.join(".git")).expect("replacement repository");
                fs::write(
                    repository.join(".git/config"),
                    "[core]\n\trepositoryformatversion = 0\n",
                )
                .expect("replacement config");
                Ok(())
            },
        )
        .expect_err("repository replacement must fail after the pinned child");
    assert_eq!(error.code(), "GIT_REPOSITORY_CHANGED");
    assert_eq!(
        fs::read_to_string(moved.join("git-ran")).unwrap(),
        "admitted"
    );
    assert!(!repository.join("git-ran").exists());
    fs::remove_dir_all(fixture).expect("remove repository fixture");
}

#[test]
fn pinned_signature_helper_overrides_changed_local_config_and_path() {
    let fixture = fixture("signature-helper-replacement");
    let repository = signed_repository(&fixture);
    let git = copied_executable(Path::new("/usr/bin/git"), &fixture, "git-real");
    let helper = executable(
        &fixture,
        "ssh-keygen-real",
        &format!(
            "#!/bin/sh\nprintf admitted > '{}'\nexec /usr/bin/ssh-keygen \"$@\"\n",
            fixture.join("helper-admitted").display()
        ),
    );
    let attacker = executable(
        &fixture,
        "ssh-keygen-attacker",
        &format!(
            "#!/bin/sh\nprintf attacker > '{}'\nexit 99\n",
            fixture.join("helper-attacker").display()
        ),
    );
    let program = GitProgram::admit(&git).expect("admit real Git copy");
    let helper =
        PinnedExecutable::admit("fixture SSH verifier", &helper).expect("admit fixture helper");
    let allowed = fixture.join("allowed_signers");
    let arguments = ["-c", "gpg.format=ssh", "verify-tag", "--raw", "v1.0.0"];

    let error = program
        .run_after_verify(
            &repository,
            &arguments,
            Some(SignatureInputs {
                helper: &helper,
                allowed_signers: &allowed,
            }),
            TEST_LIMITS,
            || {
                let config = repository.join(".git/config");
                let mut config = OpenOptions::new()
                    .append(true)
                    .open(config)
                    .expect("open local config");
                writeln!(config, "[gpg \"ssh\"]\n\tprogram = {}", attacker.display())
                    .expect("publish hostile helper config");
                Ok(())
            },
        )
        .expect_err("local config mutation must fail after verification");
    assert_eq!(error.code(), "GIT_REPOSITORY_CHANGED");
    assert_eq!(
        fs::read_to_string(fixture.join("helper-admitted")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("helper-attacker").exists());
    fs::remove_dir_all(fixture).expect("remove config fixture");
}

#[test]
fn replaced_signature_helper_path_never_executes_replacement_bytes() {
    let fixture = fixture("signature-helper-path");
    let repository = signed_repository(&fixture);
    let git = copied_executable(Path::new("/usr/bin/git"), &fixture, "git-real");
    let helper_path = executable(
        &fixture,
        "ssh-keygen-real",
        &format!(
            "#!/bin/sh\nprintf admitted > '{}'\nexec /usr/bin/ssh-keygen \"$@\"\n",
            fixture.join("helper-admitted").display()
        ),
    );
    let program = GitProgram::admit(&git).expect("admit real Git copy");
    let helper = PinnedExecutable::admit("fixture SSH verifier", &helper_path)
        .expect("admit fixture helper");
    let allowed = fixture.join("allowed_signers");
    let arguments = ["-c", "gpg.format=ssh", "verify-tag", "--raw", "v1.0.0"];

    let error = program
        .run_after_verify(
            &repository,
            &arguments,
            Some(SignatureInputs {
                helper: &helper,
                allowed_signers: &allowed,
            }),
            TEST_LIMITS,
            || {
                fs::rename(&helper_path, fixture.join("ssh-keygen-admitted"))
                    .expect("move admitted helper source");
                executable(
                    &fixture,
                    "ssh-keygen-real",
                    &format!(
                        "#!/bin/sh\nprintf attacker > '{}'\nexit 99\n",
                        fixture.join("helper-attacker").display()
                    ),
                );
                Ok(())
            },
        )
        .expect_err("helper source replacement must be reported after verification");
    assert_eq!(error.code(), "GIT_TOOL_CHANGED");
    assert_eq!(
        fs::read_to_string(fixture.join("helper-admitted")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("helper-attacker").exists());
    fs::remove_dir_all(fixture).expect("remove helper fixture");
}

#[test]
fn replaced_allowed_signers_path_cannot_change_signature_subject() {
    let fixture = fixture("allowed-signers-path");
    let repository = signed_repository(&fixture);
    let git = copied_executable(Path::new("/usr/bin/git"), &fixture, "git-real");
    let program = GitProgram::admit(&git).expect("admit real Git copy");
    let helper_path = recording_helper(&fixture);
    let helper = PinnedExecutable::admit("fixture SSH verifier", &helper_path)
        .expect("admit fixture helper");
    let allowed = fixture.join("allowed_signers");
    let admitted = fixture.join("allowed_signers-admitted");
    let arguments = ["-c", "gpg.format=ssh", "verify-tag", "--raw", "v1.0.0"];

    let error = program
        .run_after_verify(
            &repository,
            &arguments,
            Some(SignatureInputs {
                helper: &helper,
                allowed_signers: &allowed,
            }),
            TEST_LIMITS,
            || {
                fs::rename(&allowed, &admitted).expect("move admitted allowed-signers");
                fs::write(&allowed, "attacker ssh-ed25519 invalid\n")
                    .expect("publish replacement allowed-signers");
                Ok(())
            },
        )
        .expect_err("allowed-signers replacement must be reported after verification");
    assert_eq!(error.code(), "ALLOWED_SIGNERS_CHANGED");
    assert_sealed_signer_subject(&fixture, &allowed);
    fs::remove_dir_all(fixture).expect("remove signer-path fixture");
}

#[test]
fn mutated_allowed_signers_inode_cannot_change_signature_subject() {
    let fixture = fixture("allowed-signers-in-place");
    let repository = signed_repository(&fixture);
    let git = copied_executable(Path::new("/usr/bin/git"), &fixture, "git-real");
    let program = GitProgram::admit(&git).expect("admit real Git copy");
    let helper_path = recording_helper(&fixture);
    let helper = PinnedExecutable::admit("fixture SSH verifier", &helper_path)
        .expect("admit fixture helper");
    let allowed = fixture.join("allowed_signers");
    let arguments = ["-c", "gpg.format=ssh", "verify-tag", "--raw", "v1.0.0"];

    let error = program
        .run_after_verify(
            &repository,
            &arguments,
            Some(SignatureInputs {
                helper: &helper,
                allowed_signers: &allowed,
            }),
            TEST_LIMITS,
            || {
                fs::write(&allowed, "attacker ssh-ed25519 invalid\n")
                    .expect("mutate admitted allowed-signers inode");
                Ok(())
            },
        )
        .expect_err("allowed-signers mutation must be reported after verification");
    assert_eq!(error.code(), "ALLOWED_SIGNERS_CHANGED");
    assert_sealed_signer_subject(&fixture, &allowed);
    fs::remove_dir_all(fixture).expect("remove signer-mutation fixture");
}

#[test]
fn allowed_signers_symlink_is_never_an_admitted_subject() {
    use std::os::unix::fs::symlink;

    let fixture = fixture("allowed-signers-symlink");
    let target = fixture.join("allowed_signers-target");
    let link = fixture.join("allowed_signers-link");
    fs::write(&target, "release@bullet.farm ssh-ed25519 fixture\n")
        .expect("allowed-signers target");
    symlink(&target, &link).expect("allowed-signers symlink");

    let error = PinnedAllowedSigners::admit(&link)
        .expect_err("allowed-signers symlink must fail before signature verification");
    assert_eq!(error.code(), "INVALID_ALLOWED_SIGNERS");
    fs::remove_dir_all(fixture).expect("remove signer-symlink fixture");
}

fn recording_helper(fixture: &Path) -> PathBuf {
    executable(
        fixture,
        "ssh-keygen-recording",
        &format!(
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$@\" >> '{}'\n",
                "/usr/bin/ssh-keygen \"$@\"\n",
                "status=$?\n",
                "printf '%s\\n' \"$status\" > '{}'\n",
                "exit \"$status\"\n",
            ),
            fixture.join("helper-arguments").display(),
            fixture.join("helper-status").display(),
        ),
    )
}

fn assert_sealed_signer_subject(fixture: &Path, original_path: &Path) {
    assert_eq!(
        fs::read_to_string(fixture.join("helper-status"))
            .expect("recorded ssh-keygen status")
            .trim(),
        "0",
        "ssh-keygen did not accept the sealed original signer subject"
    );
    let arguments =
        fs::read_to_string(fixture.join("helper-arguments")).expect("recorded ssh-keygen args");
    assert!(arguments.contains("/proc/self/fd/"), "{arguments}");
    assert!(
        !arguments.contains(&original_path.to_string_lossy().into_owned()),
        "ssh-keygen reopened the allowed-signers pathname: {arguments}"
    );
}

fn signed_repository(fixture: &Path) -> PathBuf {
    let repository = fixture.join("repository");
    fs::create_dir(&repository).expect("repository fixture");
    let key = fixture.join("signing");
    run(Command::new("/usr/bin/ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key));
    for arguments in [
        vec!["init", "--initial-branch=main"],
        vec!["config", "user.name", "Bullet Fixture"],
        vec!["config", "user.email", "release@bullet.farm"],
        vec!["config", "gpg.format", "ssh"],
    ] {
        run(git(&repository).args(arguments));
    }
    run(git(&repository)
        .args(["config", "user.signingkey"])
        .arg(&key));
    fs::write(repository.join("subject"), "exact subject\n").expect("subject file");
    run(git(&repository).args(["add", "subject"]));
    run(git(&repository).args(["commit", "-m", "subject"]));
    run(git(&repository).args(["tag", "-s", "v1.0.0", "-m", "signed fixture"]));
    let public = fs::read_to_string(key.with_extension("pub")).expect("public key");
    let mut fields = public.split_whitespace();
    let algorithm = fields.next().expect("public-key algorithm");
    let body = fields.next().expect("public-key body");
    fs::write(
        fixture.join("allowed_signers"),
        format!("release@bullet.farm {algorithm} {body}\n"),
    )
    .expect("allowed signers");
    repository
}

fn git(repository: &Path) -> Command {
    let mut command = Command::new("/usr/bin/git");
    command
        .arg("-C")
        .arg(repository)
        .env_clear()
        .env("HOME", "/")
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    command
}

fn run(command: &mut Command) {
    let output = command.output().expect("run fixture command");
    assert!(
        output.status.success(),
        "fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn copied_executable(source: &Path, root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    fs::copy(source, &path).expect("copy executable fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("fixture permissions");
    path
}

fn executable(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create executable fixture");
    file.write_all(contents.as_bytes())
        .expect("write executable fixture");
    file.set_permissions(fs::Permissions::from_mode(0o755))
        .expect("fixture permissions");
    file.sync_all().expect("sync executable fixture");
    path
}

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-family-lock-command-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("fixture root");
    root
}
