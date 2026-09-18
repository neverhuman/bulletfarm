use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use super::{CommandSpec, SetupEnvironment, ToolIdentity, Toolchain};
use crate::{family_lock::ToolchainSubject, setup::transaction::AdmittedRoot};

#[test]
fn node_and_npm_versions_are_exact_and_fail_closed() {
    assert!(crate::setup::supported_node_version("v22.23.2"));
    assert!(crate::setup::supported_npm_version("10.9.8"));
    for refused in [
        "v21.99.99",
        "v22.23.1",
        "v26.1.0",
        "v22",
        "v22.0",
        "v22.0.0-rc.1",
        "v18446744073709551616.0.0",
        "22.0.0",
        "node v22.0.0",
    ] {
        assert!(!crate::setup::supported_node_version(refused), "{refused}");
    }
    for refused in ["10.9.7", "11.13.0", "10.9", "v10.9.8"] {
        assert!(!crate::setup::supported_npm_version(refused), "{refused}");
    }
    assert_eq!(
        ToolIdentity::Cargo.normalized_version("cargo 1.97.1"),
        Some("1.97.1")
    );
    for refused in [
        "cargo 1.97",
        "cargo 1.97.1.2",
        "cargo 1.97.1-rc.1",
        "cargo 1.x.1",
    ] {
        assert_eq!(
            ToolIdentity::Cargo.normalized_version(refused),
            None,
            "{refused}"
        );
    }
}

#[cfg(unix)]
#[test]
fn locked_tool_subjects_bind_every_file_field_before_execution() {
    let fixture = fixture_root("locked-tool-subjects");
    let probe_marker = fixture.join("cargo-probed");
    let node_probe_marker = fixture.join("node-probed");
    let npm_probe_marker = fixture.join("npm-probed");
    let cargo = executable(
        &fixture,
        "cargo-real",
        &format!(
            "#!/bin/sh\nif [ \"${{1-}}\" = --version ]; then printf probed > '{}'; printf 'cargo 1.97.1\\n'; exit 0; fi\nprintf work > '{}'\n",
            probe_marker.display(),
            fixture.join("cargo-work").display()
        ),
    );
    let node = executable(
        &fixture,
        "node-real",
        &format!(
            concat!(
                "#!/bin/sh\n",
                "if [ \"${{1-}}\" = --version ]; then printf probed > '{}'; printf 'v22.23.2\\n'; exit 0; fi\n",
                "if [ \"${{2-}}\" = --version ]; then printf probed > '{}'; printf '10.9.8\\n'; exit 0; fi\n",
                "exit 99\n",
            ),
            node_probe_marker.display(),
            npm_probe_marker.display(),
        ),
    );
    let npm_cli = fixture.join("npm-cli.js");
    fs::write(&npm_cli, "fixture npm cli\n").unwrap();
    let cargo_manifest = fixture.join("cargo.manifest");
    let node_manifest = fixture.join("node.manifest");
    let npm_manifest = fixture.join("npm.manifest");
    fs::write(&cargo_manifest, "cargo manifest\n").unwrap();
    fs::write(&node_manifest, "node manifest\n").unwrap();
    fs::write(&npm_manifest, "npm manifest\n").unwrap();

    let subjects = vec![
        tool_subject("cargo", "1.97.1", &cargo, &cargo_manifest),
        tool_subject("node", "22.23.2", &node, &node_manifest),
        tool_subject("npm-cli", "10.9.8", &npm_cli, &npm_manifest),
    ];
    let toolchain = Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), &subjects)
        .expect("exact locked tools");
    assert!(probe_marker.exists(), "exact sealed Cargo was not probed");
    assert!(
        node_probe_marker.exists(),
        "exact sealed Node was not probed"
    );
    assert!(npm_probe_marker.exists(), "exact sealed npm was not probed");

    let missing = &subjects[1..];
    for marker in [&probe_marker, &node_probe_marker, &npm_probe_marker] {
        fs::remove_file(marker).unwrap();
    }
    let error = Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), missing)
        .expect_err("missing Cargo subject");
    assert_eq!(error.code(), "SETUP_TOOL_SUBJECT_MISSING");
    assert!(!probe_marker.exists(), "missing subject executed Cargo");
    assert!(!node_probe_marker.exists(), "missing subject executed Node");
    assert!(!npm_probe_marker.exists(), "missing subject executed npm");

    let mut duplicated = subjects.clone();
    duplicated.push(subjects[0].clone());
    let error = Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), &duplicated)
        .expect_err("duplicate Cargo subject");
    assert_eq!(error.code(), "SETUP_TOOL_SUBJECT_MISMATCH");

    for field in [
        "install-path",
        "binary-digest",
        "size",
        "manifest-path",
        "manifest-digest",
        "npm-manifest-digest",
    ] {
        let mut hostile = subjects.clone();
        match field {
            "install-path" => hostile[0].install_path.push_str(".other"),
            "binary-digest" => {
                hostile[0].binary_digest = format!("blake3:{}", "0".repeat(64));
            }
            "size" => hostile[0].size_bytes += 1,
            "manifest-path" => {
                hostile[0].manifest_path = node_manifest.display().to_string();
            }
            "manifest-digest" => {
                hostile[0].manifest_digest = format!("blake3:{}", "1".repeat(64));
            }
            "npm-manifest-digest" => {
                hostile[2].manifest_digest = format!("blake3:{}", "2".repeat(64));
            }
            _ => unreachable!(),
        }
        let _ = fs::remove_file(&probe_marker);
        let error =
            match Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), &hostile) {
                Ok(_) => panic!("changed locked file subject was admitted: {field}"),
                Err(error) => error,
            };
        assert_eq!(error.code(), "SETUP_TOOL_SUBJECT_MISMATCH");
        assert!(!probe_marker.exists(), "mismatched subject executed Cargo");
        assert!(
            !node_probe_marker.exists(),
            "mismatched subject executed Node"
        );
        assert!(
            !npm_probe_marker.exists(),
            "mismatched subject executed npm"
        );
    }

    let cargo_manifest_alias = fixture.join("cargo.manifest.alias");
    fs::hard_link(&cargo_manifest, &cargo_manifest_alias).unwrap();
    let mut hostile = subjects.clone();
    hostile[1].manifest_path = cargo_manifest_alias.display().to_string();
    hostile[1].manifest_digest = hostile[0].manifest_digest.clone();
    let error = Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), &hostile)
        .expect_err("hard-linked cross-tool manifest alias");
    assert_eq!(error.code(), "SETUP_TOOL_SUBJECT_MISMATCH");
    assert!(!probe_marker.exists(), "aliased subject executed Cargo");
    assert!(!node_probe_marker.exists(), "aliased subject executed Node");
    assert!(!npm_probe_marker.exists(), "aliased subject executed npm");

    let mut hostile = subjects.clone();
    hostile[0].version = "1.97.0".into();
    let error = Toolchain::admit_locked(Some(&cargo), Some(&node), Some(&npm_cli), &hostile)
        .expect_err("changed locked version");
    assert_eq!(error.code(), "SETUP_TOOL_SUBJECT_MISMATCH");
    assert!(probe_marker.exists(), "Cargo version was not probed");
    assert!(!node_probe_marker.exists(), "Cargo mismatch executed Node");
    assert!(!npm_probe_marker.exists(), "Cargo mismatch executed npm");

    let root = AdmittedRoot::open(&fixture).unwrap();
    let environment = SetupEnvironment::create(&root, &toolchain).unwrap();
    let original_manifest = fixture.join("cargo.manifest.original");
    fs::rename(&cargo_manifest, &original_manifest).unwrap();
    fs::write(&cargo_manifest, "attacker manifest\n").unwrap();
    let error = toolchain
        .run_cargo(&fixture, &["fetch", "--locked", "--offline"], &environment)
        .expect_err("post-admission manifest swap");
    assert_eq!(error.code(), "SETUP_TOOL_CHANGED");
    assert!(!fixture.join("cargo-work").exists());
    environment.finish().unwrap();

    fs::remove_dir_all(fixture).unwrap();
}

fn tool_subject(id: &str, version: &str, binary: &Path, manifest: &Path) -> ToolchainSubject {
    let binary_bytes = fs::read(binary).unwrap();
    let manifest_bytes = fs::read(manifest).unwrap();
    ToolchainSubject {
        id: id.into(),
        version: version.into(),
        install_path: binary.display().to_string(),
        binary_digest: format!("blake3:{}", blake3::hash(&binary_bytes).to_hex()),
        manifest_path: manifest.display().to_string(),
        manifest_digest: format!("blake3:{}", blake3::hash(&manifest_bytes).to_hex()),
        size_bytes: binary_bytes.len() as u64,
    }
}

#[cfg(unix)]
#[test]
fn tool_admission_rejects_missing_relative_noncanonical_and_mismatched_inputs() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let fixture = fixture_root("tool-admission");
    let cargo = executable(
        &fixture,
        "cargo-real",
        "#!/bin/sh\nprintf 'cargo 1.97.1\\n'\n",
    );
    let node = executable(&fixture, "node-real", "#!/bin/sh\nprintf 'v22.23.2\\n'\n");
    let npm_cli = fixture.join("npm-cli.js");
    fs::write(&npm_cli, "fixture\n").expect("npm fixture");

    let old_node = executable(&fixture, "node-old", "#!/bin/sh\nprintf 'v21.99.99\\n'\n");
    let error = Toolchain::admit(Some(&cargo), Some(&old_node), Some(&npm_cli))
        .expect_err("Node below the minimum major must fail closed");
    assert_eq!(error.code(), "SETUP_TOOL_IDENTITY_MISMATCH");

    let error = Toolchain::admit(None, Some(&node), Some(&npm_cli))
        .expect_err("missing Cargo path must fail closed");
    assert_eq!(error.code(), "SETUP_TOOL_MISSING");

    let error = Toolchain::admit(Some(Path::new("cargo")), Some(&node), Some(&npm_cli))
        .expect_err("relative Cargo path must fail closed");
    assert_eq!(error.code(), "SETUP_TOOL_PATH_NOT_ABSOLUTE");

    let cargo_link = fixture.join("cargo-link");
    symlink(&cargo, &cargo_link).expect("Cargo symlink");
    let error = Toolchain::admit(Some(&cargo_link), Some(&node), Some(&npm_cli))
        .expect_err("non-canonical Cargo path must fail closed");
    assert_eq!(error.code(), "SETUP_TOOL_PATH_NOT_CANONICAL");

    let not_executable = fixture.join("cargo-data");
    fs::write(&not_executable, "cargo 1.97.1\n").expect("Cargo data");
    fs::set_permissions(&not_executable, fs::Permissions::from_mode(0o644))
        .expect("Cargo data permissions");
    let error = Toolchain::admit(Some(&not_executable), Some(&node), Some(&npm_cli))
        .expect_err("non-executable Cargo path must fail closed");
    assert_eq!(error.code(), "SETUP_TOOL_NOT_EXECUTABLE");

    let ambient_marker = fixture.join("ambient-cargo-executed");
    executable(
        &fixture,
        "cargo",
        &format!(
            "#!/bin/sh\nprintf executed > '{}'\n",
            ambient_marker.display()
        ),
    );
    let wrong = fs::canonicalize("/usr/bin/false").expect("canonical false executable");
    for attempt in 0..64 {
        let error = Toolchain::admit(Some(&wrong), Some(&node), Some(&npm_cli))
            .expect_err("wrong Cargo identity must fail closed");
        assert_eq!(
            error.code(),
            "SETUP_TOOL_IDENTITY_MISMATCH",
            "wrong-identity attempt {attempt} returned: {error}"
        );
    }
    assert!(!ambient_marker.exists(), "ambient Cargo executable ran");

    fs::remove_dir_all(fixture).expect("remove tool admission fixture");
}

#[cfg(unix)]
#[test]
fn admitted_tools_ignore_ambient_path_and_environment() {
    const CHILD: &str = "BULLET_SETUP_TOOL_TEST_CHILD";
    const ROOT: &str = "BULLET_SETUP_TOOL_TEST_ROOT";
    if std::env::var_os(CHILD).is_some() {
        admitted_tool_child(Path::new(
            &std::env::var_os(ROOT).expect("child fixture root"),
        ));
        return;
    }

    let fixture = fixture_root("tool-environment");
    let shim = fixture.join("shim");
    fs::create_dir(&shim).expect("shim directory");
    let shim_marker = fixture.join("shim-executed");
    for name in ["cargo", "node", "npm"] {
        executable(
            &shim,
            name,
            &format!(
                "#!/bin/sh\nprintf shim > '{}'\nexit 99\n",
                shim_marker.display()
            ),
        );
    }
    tool_fixtures(&fixture);
    let repository_target = fixture.join("target");
    let repository_target_debug = repository_target.join("debug");
    fs::create_dir_all(&repository_target_debug).expect("ignored repository target");
    let repository_target_marker = fixture.join("repository-target-executed");
    let hostile_contract = executable(
        &repository_target_debug,
        "bullet-contract",
        &format!(
            "#!/bin/sh\nprintf executed > '{}'\n",
            repository_target_marker.display()
        ),
    );
    let ambient_target = fixture.join("ambient-target");
    fs::create_dir(&ambient_target).expect("ambient target");
    let ambient_sentinel = ambient_target.join("sentinel");
    fs::write(&ambient_sentinel, "preserve ambient target\n").expect("ambient target sentinel");

    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "setup::command::tests::admitted_tools_ignore_ambient_path_and_environment",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env(ROOT, &fixture)
        .env("BULLET_INSTALL_CANARY_SECRET", "must-not-leak")
        .env("CARGO_TARGET_DIR", &ambient_target)
        .env("RUSTFLAGS", "--cfg hostile_rustflags")
        .env("RUSTC_WRAPPER", shim.join("cargo"))
        .env("RUSTC_WORKSPACE_WRAPPER", shim.join("node"))
        .env("CARGO_BUILD_TARGET", "hostile-target-triple")
        .env("PATH", &shim)
        .output()
        .expect("spawn isolated test child");
    assert!(
        output.status.success(),
        "isolated child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!shim_marker.exists(), "ambient PATH shim executed");
    assert!(
        hostile_contract.exists(),
        "ignored repository target changed"
    );
    assert!(
        !repository_target_marker.exists(),
        "ignored repository target executable ran"
    );
    assert_eq!(
        fs::read_to_string(ambient_sentinel).unwrap(),
        "preserve ambient target\n"
    );
    fs::remove_dir_all(fixture).expect("remove tool environment fixture");
}

#[cfg(unix)]
fn admitted_tool_child(fixture: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let toolchain = Toolchain::admit(
        Some(&fixture.join("cargo-real")),
        Some(&fixture.join("node-real")),
        Some(&fixture.join("npm-cli.js")),
    )
    .expect("admit exact fixture tools");
    let root = AdmittedRoot::open(fixture).expect("admit command fixture root");
    let environment = SetupEnvironment::create(&root, &toolchain).expect("isolated setup HOME");
    let home = environment.home_path().to_path_buf();
    let cargo_target = environment.cargo_target_path().to_path_buf();
    assert_eq!(
        fs::metadata(&cargo_target).unwrap().permissions().mode() & 0o777,
        0o700,
        "Cargo target must be private"
    );
    toolchain
        .run_cargo(fixture, &["fetch", "--locked", "--offline"], &environment)
        .expect("run admitted Cargo");
    toolchain
        .run_npm(fixture, &["ci", "--offline"], &environment)
        .expect("run admitted npm");

    for observation in ["cargo-observed", "npm-observed"] {
        let observed = fs::read_to_string(fixture.join(observation)).expect("tool observation");
        assert!(observed.contains("canary=unset\n"), "{observed}");
        assert!(observed.contains("child=unset\n"), "{observed}");
        assert!(observed.contains("--offline"), "{observed}");
        assert!(!observed.contains("/shim"), "{observed}");
        assert!(observed.contains(&format!("home={}\n", home.display())));
    }
    let cargo_observed = fs::read_to_string(fixture.join("cargo-observed")).unwrap();
    assert!(
        cargo_observed.contains(&format!("cargo_target={}\n", cargo_target.display())),
        "{cargo_observed}"
    );
    for variable in [
        "rustflags",
        "rustc_wrapper",
        "rustc_workspace_wrapper",
        "cargo_build_target",
    ] {
        assert!(
            cargo_observed.contains(&format!("{variable}=unset\n")),
            "{cargo_observed}"
        );
    }
    environment.finish().expect("remove ephemeral setup HOME");
    assert!(!home.exists(), "ephemeral setup HOME survived drop");
    assert!(
        !cargo_target.exists(),
        "ephemeral Cargo target survived finish"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn sealed_subjects_defeat_post_verification_path_swaps() {
    let fixture = fixture_root("tool-path-swap");
    let cargo = executable(
        &fixture,
        "cargo-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'cargo 1.97.1\\n'; exit 0; fi\n",
            "printf admitted > cargo-admitted\n",
        ),
    );
    let node = executable(
        &fixture,
        "node-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'v22.23.2\\n'; exit 0; fi\n",
            "exec /bin/sh \"$@\"\n",
        ),
    );
    let npm_cli = fixture.join("npm-cli.js");
    fs::write(
        &npm_cli,
        concat!(
            "if [ \"${1-}\" = --version ]; then printf '10.9.8\\n'; exit 0; fi\n",
            "printf admitted > npm-admitted\n",
        ),
    )
    .expect("npm CLI fixture");
    let toolchain = Toolchain::admit(Some(&cargo), Some(&node), Some(&npm_cli))
        .expect("admit exact fixture tools");
    let git = executable(
        &fixture,
        "git-real",
        &format!(
            concat!(
                "#!/bin/sh\n",
                "if [ \"${{1-}}\" = --version ]; then printf 'git version 2.43.0\\n'; exit 0; fi\n",
                "printf admitted > '{}'\n",
            ),
            fixture.join("git-admitted").display()
        ),
    );
    let git_command = CommandSpec::admit(ToolIdentity::Git, &git, Vec::new(), Vec::new())
        .expect("admit exact fixture Git");
    let root = AdmittedRoot::open(&fixture).expect("admit command fixture root");
    let environment = SetupEnvironment::create(&root, &toolchain).expect("isolated setup HOME");

    let original_cargo = fixture.join("cargo-original");
    let error = toolchain
        .cargo
        .run_after_verify(&fixture, &["fetch"], &environment, || {
            fs::rename(&cargo, &original_cargo).expect("move verified Cargo path");
            executable(
                &fixture,
                "cargo-real",
                "#!/bin/sh\nprintf attacker > cargo-attacker\n",
            );
            Ok(())
        })
        .expect_err("post-verification Cargo swap must be reported");
    assert_eq!(error.code(), "SETUP_TOOL_CHANGED");
    assert_eq!(
        fs::read_to_string(fixture.join("cargo-admitted")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("cargo-attacker").exists());

    let original_npm = fixture.join("npm-original.js");
    let error = toolchain
        .npm
        .run_after_verify(&fixture, &["ci"], &environment, || {
            fs::rename(&npm_cli, &original_npm).expect("move verified npm CLI path");
            fs::write(&npm_cli, "printf attacker > npm-attacker\n")
                .expect("publish attacker npm CLI");
            Ok(())
        })
        .expect_err("post-verification npm companion swap must be reported");
    assert_eq!(error.code(), "SETUP_TOOL_CHANGED");
    assert_eq!(
        fs::read_to_string(fixture.join("npm-admitted")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("npm-attacker").exists());

    let original_git = fixture.join("git-original");
    let error = git_command
        .run_git_after_verify(Some(&fixture), &[OsStr::new("status")], || {
            fs::rename(&git, &original_git).expect("move verified Git path");
            executable(
                &fixture,
                "git-real",
                &format!(
                    "#!/bin/sh\nprintf attacker > '{}'\n",
                    fixture.join("git-attacker").display()
                ),
            );
            Ok(())
        })
        .expect_err("post-verification Git swap must be reported after the child succeeds");
    assert_eq!(error.code(), "SETUP_TOOL_CHANGED");
    assert_eq!(
        fs::read_to_string(fixture.join("git-admitted")).unwrap(),
        "admitted"
    );
    assert!(!fixture.join("git-attacker").exists());

    environment.finish().expect("remove ephemeral setup HOME");
    fs::remove_dir_all(fixture).expect("remove path-swap fixture");
}

#[cfg(unix)]
#[test]
fn setup_environment_detects_root_replacement_and_cleans_only_the_pinned_root() {
    let fixture = fixture_root("environment-root-replacement");
    let moved = fixture.with_extension("original");
    let _ = fs::remove_dir_all(&moved);
    tool_fixtures(&fixture);
    let cargo = fixture.join("cargo-real");
    let node = fixture.join("node-real");
    let npm_cli = fixture.join("npm-cli.js");
    let admit = |attempt| {
        Toolchain::admit(Some(&cargo), Some(&node), Some(&npm_cli)).unwrap_or_else(|error| {
            panic!("admit exact fixture tools on attempt {attempt}: {error}")
        })
    };
    let mut toolchain = admit(0);
    for attempt in 1..64 {
        toolchain = admit(attempt);
    }
    let root = AdmittedRoot::open(&fixture).expect("admit command fixture root");
    let environment = SetupEnvironment::create(&root, &toolchain).expect("isolated setup HOME");

    fs::rename(&fixture, &moved).expect("move admitted root");
    fs::create_dir(&fixture).expect("replacement root");
    fs::write(fixture.join("sentinel"), "preserve replacement\n").expect("replacement sentinel");
    let error = environment
        .verify()
        .expect_err("replaced environment root must fail closed");
    assert_eq!(error.code(), "SETUP_ROOT_REPLACED");
    drop(environment);
    assert_eq!(
        fs::read_to_string(fixture.join("sentinel")).unwrap(),
        "preserve replacement\n"
    );
    assert!(fs::read_dir(&moved).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(crate::setup::STAGING_PREFIX)
    }));
    fs::remove_dir_all(fixture).unwrap();
    fs::remove_dir_all(moved).unwrap();
}

#[cfg(unix)]
fn tool_fixtures(root: &Path) {
    executable(
        root,
        "cargo-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'cargo 1.97.1\\n'; exit 0; fi\n",
            "printf 'canary=%s\\nchild=%s\\nargs=%s\\npath=%s\\nhome=%s\\ncargo_target=%s\\nrustflags=%s\\nrustc_wrapper=%s\\nrustc_workspace_wrapper=%s\\ncargo_build_target=%s\\n' \\\n",
            "  \"${BULLET_INSTALL_CANARY_SECRET-unset}\" \\\n",
            "  \"${BULLET_SETUP_TOOL_TEST_CHILD-unset}\" \"$*\" \"$PATH\" \"$HOME\" \\\n",
            "  \"${CARGO_TARGET_DIR-unset}\" \"${RUSTFLAGS-unset}\" \"${RUSTC_WRAPPER-unset}\" \\\n",
            "  \"${RUSTC_WORKSPACE_WRAPPER-unset}\" \"${CARGO_BUILD_TARGET-unset}\" \\\n",
            "  > cargo-observed\n",
        ),
    );
    executable(
        root,
        "node-real",
        concat!(
            "#!/bin/sh\n",
            "if [ \"${1-}\" = --version ]; then printf 'v22.23.2\\n'; exit 0; fi\n",
            "if [ \"${2-}\" = --version ]; then printf '10.9.8\\n'; exit 0; fi\n",
            "printf 'canary=%s\\nchild=%s\\nargs=%s\\npath=%s\\nhome=%s\\n' \\\n",
            "  \"${BULLET_INSTALL_CANARY_SECRET-unset}\" \\\n",
            "  \"${BULLET_SETUP_TOOL_TEST_CHILD-unset}\" \"$*\" \"$PATH\" \"$HOME\" \\\n",
            "  > npm-observed\n",
        ),
    );
    fs::write(root.join("npm-cli.js"), "fixture\n").expect("npm CLI fixture");
}

#[cfg(unix)]
fn executable(root: &Path, name: &str, contents: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join(name);
    let publishing = root.join(format!(".{name}.publishing"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&publishing)
        .expect("create private executable fixture");
    file.write_all(contents.as_bytes())
        .expect("write executable fixture");
    file.set_permissions(fs::Permissions::from_mode(0o755))
        .expect("fixture permissions");
    file.sync_all().expect("sync executable fixture");
    drop(file);
    fs::hard_link(&publishing, &path).expect("publish executable fixture without replacement");
    fs::remove_file(&publishing).expect("remove private executable fixture name");
    fs::File::open(root)
        .expect("open executable fixture directory")
        .sync_all()
        .expect("sync executable fixture directory");
    path
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-setup-command-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("command fixture root");
    root
}
