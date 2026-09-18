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
