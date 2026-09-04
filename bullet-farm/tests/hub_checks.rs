use std::{ffi::OsString, fs, path::PathBuf};

fn command(command: &[&str], current: PathBuf) -> Result<String, bullet_family::coord::CoordError> {
    let args =
        std::iter::once(OsString::from("bullet-family")).chain(command.iter().map(OsString::from));
    bullet_family::cli::run(args, Ok(current))
}

fn fixture(name: &str, cargo: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bullet-hub-check-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    fs::create_dir_all(root.join("scripts")).expect("fixture directories");
    fs::write(root.join("Cargo.toml"), cargo).expect("Cargo fixture");
    fs::write(root.join("family.lock"), "schema_version = \"2\"\n").expect("lock fixture");
    fs::write(root.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").expect("setup fixture");
    root
}

#[test]
fn rust_hub_check_accepts_the_real_onboarding_surface() {
    let hub = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(command(&["hub", "check"], hub).unwrap(), "hub-check: ok");
}

#[test]
fn dependency_check_allows_intra_repo_parent_paths_and_rejects_escape() {
    let root = fixture(
        "path-deps",
        "[package]\nname='root'\nversion='0.0.0'\n[workspace]\nmembers=['crates/a','crates/b']\n",
    );
    for name in ["a", "b"] {
        fs::create_dir_all(root.join("crates").join(name)).expect("crate directory");
    }
    fs::write(
        root.join("crates/a/Cargo.toml"),
        "[package]\nname='a'\nversion='0.0.0'\n[dependencies]\nb={path='../b'}\n",
    )
    .expect("inside dependency");
    fs::write(
        root.join("crates/b/Cargo.toml"),
        "[package]\nname='b'\nversion='0.0.0'\n",
    )
    .expect("crate b");
    assert!(command(&["deps", "check"], root.clone()).is_ok());
    fs::write(
        root.join("crates/a/Cargo.toml"),
        "[package]\nname='a'\nversion='0.0.0'\n[dependencies]\noutside={path='../../../outside'}\n",
    )
    .expect("escape dependency");
    let error = command(&["deps", "check"], root.clone()).expect_err("escape refused");
    assert_eq!(error.code(), "FORBIDDEN_PATH_DEPENDENCY");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn dependency_check_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root = fixture(
        "symlink-deps",
        "[package]\nname='root'\nversion='0.0.0'\n[dependencies]\noutside={path='linked/not-created'}\n",
    );
    let outside = root
        .parent()
        .expect("fixture parent")
        .join(format!("bullet-hub-check-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).expect("outside directory");
    symlink(&outside, root.join("linked")).expect("dependency symlink");
    let error = command(&["deps", "check"], root.clone()).expect_err("symlink refused");
    assert_eq!(error.code(), "FORBIDDEN_PATH_DEPENDENCY");

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='root'\nversion='0.0.0'\n[dependencies]\noutside={path='C:/outside'}\n",
    )
    .expect("Windows drive dependency");
    let error = command(&["deps", "check"], root.clone()).expect_err("drive path refused");
    assert_eq!(error.code(), "FORBIDDEN_PATH_DEPENDENCY");
    fs::remove_dir_all(root).expect("remove fixture");
    fs::remove_dir_all(outside).expect("remove outside fixture");
}
