use std::{
    fs::{self, File, hard_link},
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
};

use super::{EvidenceCrash, test_crash_at, write_or_verify_sealed};

const CODE: &str = "COORD_EVIDENCE_TEST_UNKNOWN";
const NAME: &str = "evidence.json";
const BYTES: &[u8] = b"{\"kind\":\"evidence\",\"schema_version\":2}\n";

fn fixture() -> (tempfile::TempDir, File) {
    let root = tempfile::tempdir().unwrap();
    let parent = File::open(root.path()).unwrap();
    (root, parent)
}

fn assert_exact(root: &tempfile::TempDir, expected: &[u8]) -> (u64, u64) {
    let path = root.path().join(NAME);
    let metadata = fs::symlink_metadata(&path).unwrap();
    assert!(metadata.is_file());
    assert_eq!(metadata.mode() & 0o7777, 0o400);
    assert_eq!(metadata.nlink(), 1);
    let mut observed = Vec::new();
    File::open(path)
        .unwrap()
        .read_to_end(&mut observed)
        .unwrap();
    assert_eq!(observed, expected);
    (metadata.dev(), metadata.ino())
}

#[test]
fn every_anonymous_evidence_write_offset_is_absent_and_retryable() {
    for offset in 0..=BYTES.len() {
        let (root, parent) = fixture();
        test_crash_at(EvidenceCrash::WriteOffset(offset));
        assert!(write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).is_err());
        assert!(!root.path().join(NAME).exists(), "offset {offset}");
        write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).unwrap();
        let identity = assert_exact(&root, BYTES);
        write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).unwrap();
        assert_eq!(assert_exact(&root, BYTES), identity);
    }
}

#[test]
fn every_link_boundary_is_absent_or_exact_and_retryable() {
    for point in [
        EvidenceCrash::AfterDataSync,
        EvidenceCrash::AfterSeal,
        EvidenceCrash::AfterLink,
    ] {
        let (root, parent) = fixture();
        test_crash_at(point);
        assert!(write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).is_err());
        if point == EvidenceCrash::AfterLink {
            let identity = assert_exact(&root, BYTES);
            write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).unwrap();
            assert_eq!(assert_exact(&root, BYTES), identity);
        } else {
            assert!(!root.path().join(NAME).exists(), "point {point:?}");
            write_or_verify_sealed(&parent, NAME, BYTES, true, CODE).unwrap();
            assert_exact(&root, BYTES);
        }
    }
}

#[test]
fn existing_exact_reconciles_and_divergent_or_unsafe_names_never_change() {
    let (exact_root, exact_parent) = fixture();
    fs::write(exact_root.path().join(NAME), BYTES).unwrap();
    fs::set_permissions(
        exact_root.path().join(NAME),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    let identity = assert_exact(&exact_root, BYTES);
    write_or_verify_sealed(&exact_parent, NAME, BYTES, true, CODE).unwrap();
    assert_eq!(assert_exact(&exact_root, BYTES), identity);

    let (different_root, different_parent) = fixture();
    fs::write(different_root.path().join(NAME), b"different\n").unwrap();
    fs::set_permissions(
        different_root.path().join(NAME),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    let before = fs::read(different_root.path().join(NAME)).unwrap();
    assert!(write_or_verify_sealed(&different_parent, NAME, BYTES, true, CODE).is_err());
    assert_eq!(fs::read(different_root.path().join(NAME)).unwrap(), before);

    let (unsafe_root, unsafe_parent) = fixture();
    fs::write(unsafe_root.path().join("target"), BYTES).unwrap();
    fs::set_permissions(
        unsafe_root.path().join("target"),
        fs::Permissions::from_mode(0o400),
    )
    .unwrap();
    hard_link(
        unsafe_root.path().join("target"),
        unsafe_root.path().join(NAME),
    )
    .unwrap();
    assert!(write_or_verify_sealed(&unsafe_parent, NAME, BYTES, true, CODE).is_err());
    fs::remove_file(unsafe_root.path().join(NAME)).unwrap();
    symlink("target", unsafe_root.path().join(NAME)).unwrap();
    assert!(write_or_verify_sealed(&unsafe_parent, NAME, BYTES, true, CODE).is_err());
}

#[test]
fn closed_evidence_bounds_refuse_without_publication() {
    for bytes in [&[][..], &vec![b'x'; 16 * 1024 + 1][..]] {
        let (root, parent) = fixture();
        assert!(write_or_verify_sealed(&parent, NAME, bytes, true, CODE).is_err());
        assert!(!root.path().join(NAME).exists());
    }
}
