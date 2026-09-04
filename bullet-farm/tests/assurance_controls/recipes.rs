use std::{fs, process::Command};

use super::root;

#[cfg(unix)]
#[test]
fn parameterized_recipes_preserve_literal_arguments() {
    use std::{env, ffi::OsString, os::unix::fs::PermissionsExt};

    let temp = tempfile::tempdir().expect("parameterized recipe fixture");
    let bin = temp.path().join("bin");
    fs::create_dir(&bin).expect("create fake bin");
    let cargo = bin.join("cargo");
    fs::write(
        &cargo,
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\0' \"$@\" >\"${BULLET_TEST_CAPTURE:?}\"\n",
    )
    .expect("write fake cargo");
    let mut permissions = fs::metadata(&cargo)
        .expect("fake cargo metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&cargo, permissions).expect("make fake cargo executable");

    let capture = temp.path().join("argv");
    let marker = temp.path().join("injected");
    let injected = format!("invalid; /usr/bin/touch {}", marker.display());
    let mut path = OsString::from(bin.as_os_str());
    path.push(":");
    path.push(env::var_os("PATH").expect("PATH is set"));

    let invoke = |recipe: &str, arguments: &[&str]| {
        let status = Command::new("just")
            .arg(recipe)
            .args(arguments)
            .current_dir(root())
            .env("PATH", &path)
            .env("BULLET_TEST_CAPTURE", &capture)
            .status()
            .unwrap_or_else(|error| panic!("run {recipe} recipe: {error}"));
        assert!(status.success(), "{recipe} recipe failed");
        assert!(!marker.exists(), "{recipe} executed caller text as shell");
        fs::read(&capture)
            .expect("captured Cargo argv")
            .split(|byte| *byte == 0)
            .filter(|argument| !argument.is_empty())
            .map(|argument| String::from_utf8(argument.to_vec()).expect("UTF-8 argument"))
            .collect::<Vec<_>>()
    };

    let coord = invoke("coord", &["--agent", &injected]);
    assert!(
        coord.ends_with(&["coord".to_owned(), "--agent".to_owned(), injected.clone()]),
        "coord did not preserve its literal argv tail: {coord:?}"
    );

    let subject_injected = format!(
        "{}/subjects; /usr/bin/touch {}",
        temp.path().display(),
        marker.display()
    );
    let generated = invoke("lock-generate", &[&injected, &subject_injected]);
    assert!(
        generated.ends_with(&[
            "lock".to_owned(),
            "generate".to_owned(),
            "--tag".to_owned(),
            injected.clone(),
            "--subjects".to_owned(),
            subject_injected,
        ]),
        "lock-generate did not preserve its literal argv tail: {generated:?}"
    );

    let verified = invoke("lock-verify", &[&injected]);
    assert!(
        verified.ends_with(&[
            "lock".to_owned(),
            "verify".to_owned(),
            "--tag".to_owned(),
            injected.clone(),
        ]),
        "lock-verify did not preserve its literal argv tail: {verified:?}"
    );

    let doctor = Command::new("just")
        .args(["ci-doctor", &injected])
        .current_dir(root())
        .env("PATH", path)
        .status()
        .expect("run ci-doctor recipe");
    assert!(!doctor.success(), "invalid doctor lane was accepted");
    assert!(!marker.exists(), "ci-doctor executed caller text as shell");
}

#[cfg(unix)]
#[test]
fn setup_recipe_has_a_closed_argument_and_launcher_surface() {
    use std::{env, os::unix::fs::PermissionsExt};

    let temp = tempfile::tempdir().expect("setup recipe fixture");
    let bin = temp.path().join("bin");
    fs::create_dir(&bin).expect("create fake bin");
    let capture = temp.path().join("argv");
    let cargo_marker = temp.path().join("ambient-cargo-executed");
    let launcher_marker = temp.path().join("ambient-shell-executed");
    let injection_marker = temp.path().join("injected");

    let make_executable = |path: &std::path::Path, body: &str| {
        fs::write(path, body).expect("write executable fixture");
        let mut permissions = fs::metadata(path).expect("fixture metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("make fixture executable");
    };

    make_executable(
        &bin.join("cargo"),
        &format!(
            "#!/bin/bash\nprintf executed > '{}'\n",
            cargo_marker.display()
        ),
    );
    for name in ["bash", "sh"] {
        make_executable(
            &bin.join(name),
            &format!(
                "#!/bin/sh\nprintf executed > '{}'\nexit 97\n",
                launcher_marker.display()
            ),
        );
    }

    let setup = temp.path().join("bullet-family");
    make_executable(
        &setup,
        &format!(
            "#!/bin/bash\nset -euo pipefail\nprintf '%s\\0' \"$@\" > '{}'\n",
            capture.display()
        ),
    );

    let just = env::split_paths(&env::var_os("PATH").expect("PATH is set"))
        .map(|directory| directory.join("just"))
        .find(|candidate| candidate.is_file())
        .expect("just is installed at an absolute PATH entry");
    assert!(just.is_absolute(), "just fixture path must be absolute");

    let assert_bootstrap_unavailable = |output: std::process::Output| {
        assert_eq!(
            output.status.code(),
            Some(4),
            "unexpected refusal status; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("setup: SETUP_BOOTSTRAP_UNAVAILABLE:"),
            "unexpected refusal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    let direct_without_path = Command::new(root().join("scripts/setup.sh"))
        .arg("--offline")
        .env_clear()
        .env("PATH", "/definitely-missing")
        .env("LC_ALL", "C")
        .output()
        .expect("run direct setup wrapper without PATH");
    assert_bootstrap_unavailable(direct_without_path);

    let just_without_path = Command::new(&just)
        .arg("setup")
        .current_dir(root())
        .env_clear()
        .env("PATH", "/definitely-missing")
        .env("LC_ALL", "C")
        .output()
        .expect("run just setup without PATH");
    assert_bootstrap_unavailable(just_without_path);

    let refused = Command::new(&just)
        .arg("setup")
        .current_dir(root())
        .env("PATH", &bin)
        .env_remove("BULLET_SETUP_ADMITTED_BIN")
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env("BULLET_SETUP_NPM_CLI", "/bin/true")
        .output()
        .expect("run setup without admitted bootstrap");
    assert_bootstrap_unavailable(refused);
    assert!(
        !cargo_marker.exists(),
        "missing bootstrap executed ambient Cargo"
    );
    assert!(
        !launcher_marker.exists(),
        "setup selected an ambient shell launcher"
    );

    // A regular executable placed directly under the family root: invariant under
    // external target directories, removed on drop even when an assertion fails.
    let family_root = root()
        .parent()
        .expect("hub has a family root")
        .canonicalize()
        .expect("canonical family root");
    let in_family_bin = InFamilyExecutable::create(&family_root);
    let in_family = Command::new(&just)
        .arg("setup")
        .current_dir(root())
        .env("PATH", &bin)
        .env("BULLET_SETUP_ADMITTED_BIN", &in_family_bin.path)
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env("BULLET_SETUP_NPM_CLI", "/bin/true")
        .output()
        .expect("run setup with in-family bootstrap");
    assert_eq!(in_family.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&in_family.stderr).contains("setup: SETUP_BOOTSTRAP_INVALID:"),
        "unexpected refusal: {}",
        String::from_utf8_lossy(&in_family.stderr)
    );
    assert!(
        String::from_utf8_lossy(&in_family.stderr).contains("outside the source family"),
        "unexpected refusal detail: {}",
        String::from_utf8_lossy(&in_family.stderr)
    );

    let missing_tool = Command::new(&just)
        .arg("setup")
        .current_dir(root())
        .env("PATH", &bin)
        .env("BULLET_SETUP_ADMITTED_BIN", &setup)
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env_remove("BULLET_SETUP_NPM_CLI")
        .output()
        .expect("run setup without explicit npm authority");
    assert_eq!(missing_tool.status.code(), Some(4));
    assert!(
        String::from_utf8_lossy(&missing_tool.stderr).contains("setup: SETUP_TOOL_PATH_INVALID:"),
        "unexpected refusal: {}",
        String::from_utf8_lossy(&missing_tool.stderr)
    );
    assert!(
        !capture.exists(),
        "tool-path refusal executed the selected bootstrap"
    );

    let injected = format!("; /usr/bin/touch {}", injection_marker.display());
    for arguments in [
        vec!["--root"],
        vec!["--offline", "--offline"],
        vec![injected.as_str()],
    ] {
        let refused = Command::new(&just)
            .arg("--")
            .arg("setup")
            .args(arguments)
            .current_dir(root())
            .env("PATH", &bin)
            .env("BULLET_SETUP_ADMITTED_BIN", &setup)
            .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
            .env("BULLET_SETUP_NODE_BIN", "/bin/true")
            .env("BULLET_SETUP_NPM_CLI", "/bin/true")
            .output()
            .expect("run setup with invalid argument tail");
        assert_eq!(refused.status.code(), Some(4));
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("setup: SETUP_ARGUMENT_INVALID:"),
            "unexpected refusal: {}",
            String::from_utf8_lossy(&refused.stderr)
        );
        assert!(
            !capture.exists(),
            "argument refusal executed the selected bootstrap"
        );
    }

    let status = Command::new(&just)
        .args(["--", "setup", "--offline"])
        .current_dir(root())
        .env("PATH", &bin)
        .env("BULLET_SETUP_ADMITTED_BIN", &setup)
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env("BULLET_SETUP_NPM_CLI", "/bin/true")
        .status()
        .expect("run setup recipe");

    assert!(status.success());
    assert!(
        !injection_marker.exists(),
        "recipe executed an interpolated shell command"
    );
    assert!(
        !cargo_marker.exists(),
        "external bootstrap executed ambient Cargo"
    );
    assert!(
        !launcher_marker.exists(),
        "setup selected an ambient shell launcher"
    );
    let captured = fs::read(&capture).expect("captured setup argv");
    let arguments = captured
        .split(|byte| *byte == 0)
        .filter(|argument| !argument.is_empty())
        .map(|argument| String::from_utf8(argument.to_vec()).expect("UTF-8 argument"))
        .collect::<Vec<_>>();
    assert_eq!(
        arguments,
        [
            "setup".to_owned(),
            "--root".to_owned(),
            root().parent().unwrap().display().to_string(),
            "--source".to_owned(),
            "jeryu".to_owned(),
            "--cargo-bin".to_owned(),
            "/bin/true".to_owned(),
            "--node-bin".to_owned(),
            "/bin/true".to_owned(),
            "--npm-cli".to_owned(),
            "/bin/true".to_owned(),
            "--offline".to_owned(),
        ]
    );
}

/// Temporary regular executable under the family root for the in-family refusal case.
struct InFamilyExecutable {
    path: std::path::PathBuf,
}

impl InFamilyExecutable {
    fn create(family_root: &std::path::Path) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let path = family_root.join(format!(".setup-recipe-in-family-{}", std::process::id()));
        fs::write(&path, "#!/bin/sh\nexit 99\n").expect("write in-family executable");
        let mut permissions = fs::metadata(&path)
            .expect("in-family executable metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make in-family executable runnable");
        Self { path }
    }
}

impl Drop for InFamilyExecutable {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
