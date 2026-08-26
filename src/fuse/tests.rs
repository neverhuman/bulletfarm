use std::{collections::BTreeMap, fs, path::PathBuf};

use super::{Source, parse_args, publish};

#[test]
fn arguments_are_exact() {
    assert_eq!(
        parse_args(&["--source".into(), "local".into()]).unwrap(),
        Source::Local
    );
    assert_eq!(
        parse_args(&["--source".into(), "lock".into()]).unwrap(),
        Source::Lock
    );
    for denied in [
        vec![],
        vec!["local".into()],
        vec!["--source".into()],
        vec!["--source".into(), "other".into()],
        vec!["--source".into(), "local".into(), "--all".into()],
        vec![
            "--source".into(),
            "local".into(),
            "--source".into(),
            "lock".into(),
        ],
    ] {
        assert_eq!(parse_args(&denied).unwrap_err().code(), "USAGE");
    }
}

#[test]
fn publication_is_complete_replace_safe_and_byte_idempotent() {
    let hub = fixture("publication");
    let first = files(b"first\n");
    publish::publish(&hub, &first).unwrap();
    let first_bytes = snapshot(&hub.join(".fusion"));
    publish::publish(&hub, &first).unwrap();
    assert_eq!(snapshot(&hub.join(".fusion")), first_bytes);

    let second = files(b"second\n");
    publish::publish(&hub, &second).unwrap();
    let second_bytes = snapshot(&hub.join(".fusion"));
    assert_eq!(
        second_bytes.get("manifest.toml"),
        Some(&b"second\n".to_vec())
    );
    assert_ne!(second_bytes, first_bytes);
    assert!(staging_paths(&hub).is_empty());
    fs::remove_dir_all(hub).unwrap();
}

#[cfg(unix)]
#[test]
fn hostile_destination_symlink_and_unknown_content_are_preserved() {
    use std::os::unix::fs::symlink;

    let hub = fixture("hostile");
    let external = hub.with_extension("external");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("sentinel"), "preserve\n").unwrap();
    symlink(&external, hub.join(".fusion")).unwrap();
    let error = publish::publish(&hub, &files(b"new\n")).unwrap_err();
    assert_eq!(error.code(), "FUSION_DESTINATION_CONFLICT");
    assert_eq!(
        fs::read_to_string(external.join("sentinel")).unwrap(),
        "preserve\n"
    );
    fs::remove_file(hub.join(".fusion")).unwrap();

    fs::create_dir(hub.join(".fusion")).unwrap();
    fs::write(hub.join(".fusion/sentinel"), "preserve\n").unwrap();
    let error = publish::publish(&hub, &files(b"new\n")).unwrap_err();
    assert_eq!(error.code(), "FUSION_DESTINATION_CONFLICT");
    assert_eq!(
        fs::read_to_string(hub.join(".fusion/sentinel")).unwrap(),
        "preserve\n"
    );
    fs::remove_dir_all(hub).unwrap();
    fs::remove_dir_all(external).unwrap();
}

fn files(manifest: &[u8]) -> Vec<(&str, &[u8], bool)> {
    vec![
        ("manifest.toml", manifest, false),
        ("source", b"local\n", false),
        ("dev.sh", b"#!/bin/sh\n", true),
    ]
}

fn fixture(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("bullet-fuse-unit-{name}-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir(&path).unwrap();
    path
}

fn snapshot(root: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}

fn staging_paths(hub: &std::path::Path) -> Vec<PathBuf> {
    fs::read_dir(hub)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".fusion.stage."))
                .then_some(path)
        })
        .collect()
}
