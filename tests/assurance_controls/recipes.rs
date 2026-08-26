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
fn setup_recipe_preserves_literal_arguments() {
    use std::{env, ffi::OsString, os::unix::fs::PermissionsExt};

    let temp = tempfile::tempdir().expect("setup recipe fixture");
    let bin = temp.path().join("bin");
    fs::create_dir(&bin).expect("create fake bin");
    let capture = temp.path().join("argv");
    let marker = temp.path().join("ambient-cargo-executed");
    let cargo = bin.join("cargo");
    fs::write(
        &cargo,
        format!("#!/bin/bash\nprintf executed > '{}'\n", marker.display()),
    )
    .expect("write fake cargo");
    let mut permissions = fs::metadata(&cargo)
        .expect("fake cargo metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&cargo, permissions).expect("make fake cargo executable");

    let setup = temp.path().join("bullet-family");
    fs::write(
        &setup,
        format!(
            "#!/bin/bash\nset -euo pipefail\nprintf '%s\\0' \"$@\" > '{}'\n",
            capture.display()
        ),
    )
    .expect("write fake setup binary");
    let mut permissions = fs::metadata(&setup)
        .expect("fake setup metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&setup, permissions).expect("make fake setup executable");

    let injection_marker = temp.path().join("injected");
    let spaced = temp.path().join("member root");
    let injected = format!("; /usr/bin/touch {}", injection_marker.display());
    let mut path = OsString::from(bin.as_os_str());
    path.push(":");
    path.push(env::var_os("PATH").expect("PATH is set"));

    let refused = Command::new("just")
        .arg("setup")
        .current_dir(root())
        .env("PATH", &path)
        .env_remove("BULLET_SETUP_ADMITTED_BIN")
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env("BULLET_SETUP_NPM_CLI", "/bin/true")
        .output()
        .expect("run setup without admitted bootstrap");
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("operator-pre-admitted bootstrap unavailable"),
        "unexpected refusal: {}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(!marker.exists(), "missing bootstrap executed ambient Cargo");

    // A regular executable placed directly under the family root: invariant under
    // external target directories, removed on drop even when an assertion fails.
    let family_root = root()
        .parent()
        .expect("hub has a family root")
        .canonicalize()
        .expect("canonical family root");
    let in_family_bin = InFamilyExecutable::create(&family_root);
    let in_family = Command::new("just")
        .arg("setup")
        .current_dir(root())
        .env("PATH", &path)
        .env("BULLET_SETUP_ADMITTED_BIN", &in_family_bin.path)
        .env("BULLET_SETUP_CARGO_BIN", "/bin/true")
        .env("BULLET_SETUP_NODE_BIN", "/bin/true")
        .env("BULLET_SETUP_NPM_CLI", "/bin/true")
        .output()
        .expect("run setup with in-family bootstrap");
    assert!(!in_family.status.success());
    assert!(
        String::from_utf8_lossy(&in_family.stderr).contains("outside the source family"),
        "unexpected refusal: {}",
        String::from_utf8_lossy(&in_family.stderr)
    );
    assert!(
        !marker.exists(),
        "in-family bootstrap executed ambient Cargo"
    );

    let status = Command::new("just")
        .args([
            "setup",
            "--root",
            spaced.to_str().expect("UTF-8 fixture path"),
            &injected,
        ])
        .current_dir(root())
        .env("PATH", path)
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
        !marker.exists(),
        "external bootstrap executed ambient Cargo"
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
            "--root".to_owned(),
            spaced.display().to_string(),
            injected,
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
