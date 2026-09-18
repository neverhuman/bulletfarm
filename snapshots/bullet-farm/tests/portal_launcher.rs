const PORTAL_LAUNCHER: &str = include_str!("../scripts/portal.sh");

#[test]
fn portal_launcher_enforces_the_same_origin_vite_proxy() {
    assert!(
        PORTAL_LAUNCHER.contains("unset VITE_BULLET_API"),
        "the development launcher must clear an inherited cross-origin API base"
    );
    assert!(
        !PORTAL_LAUNCHER.contains("export VITE_BULLET_API"),
        "the development launcher must not export a cross-origin API base"
    );
    assert!(
        !PORTAL_LAUNCHER.contains("127.0.0.1:7420"),
        "browser API requests must use Vite's same-origin proxy"
    );
    assert!(
        PORTAL_LAUNCHER
            .contains("npm run dev -- --host 127.0.0.1 --port \"$portal_port\" --strictPort"),
        "the development server must stay bound to loopback on its selected strict port"
    );
    assert!(
        PORTAL_LAUNCHER.contains("BULLET_PORTAL_PORT"),
        "the printed console origin must be able to select the Vite port"
    );
    #[cfg(unix)]
    assert_launcher_runtime();
}

#[cfg(unix)]
fn assert_launcher_runtime() {
    use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

    let root = tempfile::tempdir().unwrap();
    let family = root.path().join("family with spaces");
    let scripts = family.join("bullet-farm/scripts");
    let portal = family.join("bullet-portal");
    let bin = root.path().join("bin");
    for directory in [&scripts, &portal, &bin] {
        fs::create_dir_all(directory).unwrap();
    }
    let launcher = scripts.join("portal.sh");
    fs::write(&launcher, PORTAL_LAUNCHER).unwrap();
    let npm = bin.join("npm");
    fs::write(
        &npm,
        r#"#!/bin/sh
printf '%s\n' "${VITE_BULLET_API-unset}" > "$BULLET_TEST_CAPTURE.env"
printf '%s\n' "$@" > "$BULLET_TEST_CAPTURE.args"
pwd > "$BULLET_TEST_CAPTURE.cwd"
"#,
    )
    .unwrap();
    fs::set_permissions(&npm, fs::Permissions::from_mode(0o700)).unwrap();
    let capture = root.path().join("captured");
    let path =
        std::env::join_paths([bin.as_path(), Path::new("/usr/bin"), Path::new("/bin")]).unwrap();
    let invoke = |port: Option<&str>| {
        let mut command = Command::new("bash");
        command
            .arg(&launcher)
            .env_clear()
            .env("PATH", &path)
            .env("VITE_BULLET_API", "https://cross-origin.invalid")
            .env("BULLET_TEST_CAPTURE", &capture);
        if let Some(port) = port {
            command.env("BULLET_PORTAL_PORT", port);
        }
        command
            .output()
            .expect("actual Portal launcher must execute")
    };
    for (configured, expected) in [
        (None, "5173"),
        (Some(""), "5173"),
        (Some("6201"), "6201"),
        (Some("1"), "1"),
        (Some("65535"), "65535"),
    ] {
        let output = invoke(configured);
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(
            fs::read_to_string(capture.with_extension("env")).unwrap(),
            "unset\n"
        );
        assert_eq!(
            fs::read_to_string(capture.with_extension("args")).unwrap(),
            format!("run\ndev\n--\n--host\n127.0.0.1\n--port\n{expected}\n--strictPort\n")
        );
        assert_eq!(
            Path::new(
                fs::read_to_string(capture.with_extension("cwd"))
                    .unwrap()
                    .trim_end()
            ),
            portal.canonicalize().unwrap()
        );
        for extension in ["env", "args", "cwd"] {
            fs::remove_file(capture.with_extension(extension)).unwrap();
        }
    }
    for invalid in ["0", "65536", "-1", "1 2", "abc", "5173;exit 0"] {
        let output = invoke(Some(invalid));
        assert!(!output.status.success(), "port {invalid} was accepted");
        assert!(String::from_utf8_lossy(&output.stderr).contains("PORTAL_PORT_INVALID"));
        for extension in ["env", "args", "cwd"] {
            assert!(
                !capture.with_extension(extension).exists(),
                "npm ran for {invalid}"
            );
        }
    }
}
