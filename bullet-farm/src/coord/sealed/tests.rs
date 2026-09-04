use std::{fs, os::unix::fs::PermissionsExt};

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    value: String,
}

#[test]
fn anonymous_output_and_independent_reads_are_exact() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = root.path().join("sealed.json");
    let expected = Document {
        value: "exact".to_owned(),
    };
    write(&path, &expected).unwrap();
    assert_eq!(read::<Document>(&path).unwrap(), expected);
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.is_file());
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o400);
    assert_eq!(metadata.nlink(), 1);
    assert!(write(&path, &expected).is_err());
}

#[test]
fn raw_output_preserves_nul_and_has_no_implicit_lf() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = root.path().join("signing-message.bin");
    let expected = b"bullet.recovery.signature.v1\0{\"exact\":true}";

    write_raw(&path, expected, expected.len() as u64).unwrap();

    assert_eq!(read_raw(&path, expected.len() as u64).unwrap(), expected);
    assert_ne!(expected.last(), Some(&b'\n'));
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o400);
    assert_eq!(metadata.nlink(), 1);
}

#[test]
fn raw_output_refuses_empty_oversize_and_existing_subjects() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let empty = root.path().join("empty.bin");
    let zero_bound = root.path().join("zero-bound.bin");
    let oversize = root.path().join("oversize.bin");

    assert!(write_raw(&empty, b"", 1).is_err());
    assert!(write_raw(&zero_bound, b"x", 0).is_err());
    assert!(write_raw(&oversize, b"four", 3).is_err());
    for path in [&empty, &zero_bound, &oversize] {
        assert!(!path.exists(), "a refused raw output was published");
    }

    let exact = root.path().join("exact.bin");
    write_raw(&exact, b"four", 4).unwrap();
    assert!(write_raw(&exact, b"other", 5).is_err());
    assert_eq!(read_raw(&exact, 4).unwrap(), b"four");
}

#[test]
fn parent_mode_file_mode_and_link_count_are_closed() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let path = root.path().join("sealed.json");
    let expected = Document {
        value: "exact".to_owned(),
    };
    write(&path, &expected).unwrap();

    fs::set_permissions(&path, fs::Permissions::from_mode(0o440)).unwrap();
    assert!(read::<Document>(&path).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    let second = root.path().join("second.json");
    fs::hard_link(&path, &second).unwrap();
    assert!(read::<Document>(&path).is_err());
    fs::remove_file(&second).unwrap();

    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o750)).unwrap();
    assert!(read::<Document>(&path).is_err());
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn symlinked_parent_and_noncanonical_bytes_refuse() {
    let root = tempfile::tempdir().unwrap();
    let private = root.path().join("private");
    fs::create_dir(&private).unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
    let path = private.join("sealed.json");
    fs::write(&path, b"{\"value\":\"exact\"}\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    let alias = root.path().join("alias");
    std::os::unix::fs::symlink(&private, &alias).unwrap();
    assert!(read::<Document>(&alias.join("sealed.json")).is_err());

    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"{ \"value\": \"exact\" }\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(read::<Document>(&path).is_err());
}

#[test]
fn frozen_legacy_parent_admits_only_exact_0700_or_0775() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("events.jsonl");
    fs::write(&path, b"frozen\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();

    for mode in [0o700, 0o775] {
        fs::set_permissions(root.path(), fs::Permissions::from_mode(mode)).unwrap();
        assert_eq!(read_raw_legacy_live(&path, 64).unwrap(), b"frozen\n");
        assert_eq!(read_raw(&path, 64).is_ok(), mode == 0o700);
    }
    for mode in [0o750, 0o770, 0o777, 0o755] {
        fs::set_permissions(root.path(), fs::Permissions::from_mode(mode)).unwrap();
        assert!(read_raw_legacy_live(&path, 64).is_err());
    }

    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o775)).unwrap();
    let alias_root = tempfile::tempdir().unwrap();
    let alias = alias_root.path().join("coord");
    std::os::unix::fs::symlink(root.path(), &alias).unwrap();
    assert!(read_raw_legacy_live(&alias.join("events.jsonl"), 64).is_err());

    let runtime_parent = ParentIdentity {
        device: 1,
        inode: 1,
        owner_uid: 0,
        owner_gid: 0,
        mode: 0o755,
    };
    assert!(
        runtime_parent
            .validate(ParentAdmission::RootRuntime)
            .is_ok()
    );
    for (owner_uid, owner_gid, mode) in [(1, 0, 0o755), (0, 1, 0o755), (0, 0, 0o700)] {
        assert!(
            ParentIdentity {
                owner_uid,
                owner_gid,
                mode,
                ..runtime_parent
            }
            .validate(ParentAdmission::RootRuntime)
            .is_err()
        );
    }
    let runtime_file = Identity {
        device: 1,
        inode: 1,
        owner_uid: 0,
        owner_gid: 0,
        mode: 0o444,
        links: 1,
        length: 1,
        mtime_seconds: 1,
        mtime_nanoseconds: 0,
        ctime_seconds: 1,
        ctime_nanoseconds: 0,
    };
    assert!(
        runtime_file
            .validate_file(1, 64, ParentAdmission::RootRuntime)
            .is_ok()
    );
    for (owner_uid, owner_gid, mode, links) in [
        (1, 0, 0o444, 1),
        (0, 1, 0o444, 1),
        (0, 0, 0o400, 1),
        (0, 0, 0o444, 2),
    ] {
        assert!(
            Identity {
                owner_uid,
                owner_gid,
                mode,
                links,
                ..runtime_file
            }
            .validate_file(1, 64, ParentAdmission::RootRuntime)
            .is_err()
        );
    }
}

#[test]
fn root_runtime_mode_and_response_loss_adoption_are_exact() {
    assert_eq!(ParentAdmission::RootRuntime.file_mode().bits(), 0o444);
    assert!(runtime::exact_existing_bytes(
        b"exact\n", b"exact\n", b"exact\n"
    ));
    assert!(!runtime::exact_existing_bytes(
        b"exact\n",
        b"different\n",
        b"different\n"
    ));
    assert!(!runtime::exact_existing_bytes(
        b"exact\n",
        b"exact\n",
        b"different\n"
    ));
}
