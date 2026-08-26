use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bullet-setup-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    let hub = root.join("bullet-farm");
    fs::create_dir_all(hub.join("scripts")).expect("fixture directories");
    fs::create_dir_all(hub.join("release")).expect("release directory");
    fs::write(
        hub.join("Cargo.toml"),
        "[package]\nname='bullet-family'\nversion='0.0.0'\n",
    )
    .expect("Cargo fixture");
    fs::write(hub.join("scripts/setup.sh"), "#!/bin/sh\nexit 1\n").expect("setup fixture");
    fs::write(
        hub.join("repos.manifest.toml"),
        concat!(
            "schema_version = \"1.2.0\"\n",
            "family = \"bullet-farm\"\n",
            "umbrella_repo = \"bullet-farm\"\n",
            "required_repos = [\"bullet-farm\", \"bullet-kernel\", \"bullet-git\", \"bullet-portal\"]\n",
        ),
    )
    .expect("manifest fixture");
    fs::write(
        hub.join("family.lock"),
        "schema_version = \"2\"\nfamily = \"bullet-farm\"\n",
    )
    .expect("legacy lock fixture");
    fs::write(hub.join("release/allowed_signers"), "fixture\n").expect("signer fixture");
    root
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).expect("read fixture") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).expect("relative").to_path_buf(),
                    fs::read(path).expect("file bytes"),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn legacy_lock_blocks_setup_before_mutation() {
    let root = fixture("legacy");
    let hub = root.join("bullet-farm");
    let before = snapshot(&root);
    let error = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("setup"),
            OsString::from("--root"),
            root.clone().into_os_string(),
            OsString::from("--source"),
            OsString::from("jeryu"),
        ],
        Ok(hub),
    )
    .expect_err("schema 2 cannot authorize installation");
    assert_eq!(error.code(), "UNSUPPORTED_SCHEMA");
    assert_eq!(snapshot(&root), before, "blocked setup changed its input");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn checkout_verify_rejects_legacy_lock() {
    let root = fixture("checkout-legacy");
    let hub = root.join("bullet-farm");
    let error = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("--root"),
            root.clone().into_os_string(),
            OsString::from("checkout"),
            OsString::from("verify"),
        ],
        Ok(hub),
    )
    .expect_err("schema 2 cannot verify an install");
    assert_eq!(error.code(), "UNSUPPORTED_SCHEMA");
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn setup_rejects_a_symlink_root() {
    use std::os::unix::fs::symlink;

    let root = fixture("symlink-root");
    let hub = root.join("bullet-farm");
    let link = root
        .parent()
        .expect("temporary parent")
        .join(format!("bullet-setup-link-{}", std::process::id()));
    if link.exists() || link.symlink_metadata().is_ok() {
        fs::remove_file(&link).expect("remove stale link");
    }
    symlink(&root, &link).expect("create root link");
    let error = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("setup"),
            OsString::from("--root"),
            link.clone().into_os_string(),
            OsString::from("--source"),
            OsString::from("jeryu"),
        ],
        Ok(hub),
    )
    .expect_err("symlink root must fail before lock parsing");
    assert_eq!(error.code(), "INVALID_CHECKOUT");
    fs::remove_file(link).expect("remove link");
    fs::remove_dir_all(root).expect("remove fixture");
}
