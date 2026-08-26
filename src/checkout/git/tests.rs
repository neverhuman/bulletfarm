use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use super::{run_after_verify, verify_exact_worktree};

#[test]
fn descriptor_work_tree_preserves_relative_hash_object_semantics() {
    let fixture = fixture("relative-work-tree");
    let repository = fixture.join("repository");
    repository_fixture(&repository);

    verify_exact_worktree(&repository).expect("clean exact work tree");
    fs::remove_dir_all(fixture).expect("remove relative work-tree fixture");
}

#[test]
fn repository_path_replacement_cannot_redirect_checkout_git() {
    let fixture = fixture("repository-replacement");
    let repository = fixture.join("repository");
    let moved = fixture.join("repository-admitted");
    repository_fixture(&repository);

    let error = run_after_verify(&repository, &["status", "--porcelain=v2"], || {
        fs::rename(&repository, &moved).expect("move admitted repository");
        repository_fixture(&repository);
        Ok(())
    })
    .expect_err("repository replacement must fail after the pinned child");
    assert_eq!(error.code(), "GIT_REPOSITORY_CHANGED");
    assert_eq!(
        fs::read_to_string(moved.join("subject")).unwrap(),
        "exact subject\n"
    );
    assert_eq!(
        fs::read_to_string(repository.join("subject")).unwrap(),
        "exact subject\n"
    );
    fs::remove_dir_all(fixture).expect("remove repository fixture");
}

#[test]
fn config_mutation_cannot_launch_a_checkout_helper() {
    let fixture = fixture("config-mutation");
    let repository = fixture.join("repository");
    let canary = fixture.join("fsmonitor-canary");
    repository_fixture(&repository);
    let helper = executable(
        &fixture,
        "fsmonitor-attacker",
        &format!("#!/bin/sh\nprintf attacker > '{}'\n", canary.display()),
    );

    let error = run_after_verify(&repository, &["status", "--porcelain=v2"], || {
        let mut config = OpenOptions::new()
            .append(true)
            .open(repository.join(".git/config"))
            .expect("open local config");
        writeln!(config, "[core]\n\tfsmonitor = {}", helper.display())
            .expect("publish hostile fsmonitor");
        Ok(())
    })
    .expect_err("config mutation must fail after the pinned child");
    assert_eq!(error.code(), "GIT_REPOSITORY_CHANGED");
    assert!(!canary.exists(), "checkout Git launched hostile fsmonitor");
    fs::remove_dir_all(fixture).expect("remove config fixture");
}

fn repository_fixture(repository: &Path) {
    fs::create_dir(repository).expect("repository fixture");
    for arguments in [
        &["init", "--initial-branch=main"][..],
        &["config", "user.name", "Bullet Fixture"],
        &["config", "user.email", "fixture@bullet.farm"],
    ] {
        run(git(repository).args(arguments));
    }
    fs::write(repository.join("subject"), "exact subject\n").expect("subject file");
    run(git(repository).args(["add", "subject"]));
    run(git(repository).args(["commit", "-m", "subject"]));
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
    let output = command.output().expect("run fixture Git");
    assert!(
        output.status.success(),
        "fixture Git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    path
}

fn fixture(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bullet-checkout-git-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("fixture root");
    root
}
