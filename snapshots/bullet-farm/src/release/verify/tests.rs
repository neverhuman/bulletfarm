use std::{
    fs::{self, File},
    sync::atomic::{AtomicU64, Ordering},
};

use nix::{sys::stat::Mode, unistd::mkfifo};

use super::{
    ReleaseFile, immutable_snapshot, open_bounded_file, read_open_bounded, verify_open_file,
    verify_snapshot_subject,
};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn sealed_signature_receipt_rejects_in_place_substitution_after_admission() {
    let path = fixture_path("signature");
    let admitted = b"first-signature";
    let substituted = b"other-signature";
    assert_eq!(admitted.len(), substituted.len());
    fs::write(&path, admitted).expect("write admitted signature");
    let mut input = open_bounded_file(&path, "signature", 64 * 1024).expect("open signature");
    let expected = ReleaseFile {
        path: "artifact.sig".to_owned(),
        size: admitted.len() as u64,
        digest: format!("blake3:{}", blake3::hash(admitted).to_hex()),
    };
    verify_open_file(&mut input, &expected).expect("admit initial subject");

    fs::write(&path, substituted).expect("rewrite admitted inode");
    let snapshot = immutable_snapshot(&mut input, "signature", 64 * 1024).expect("snapshot");
    assert_eq!(snapshot.byte_count, substituted.len() as u64);
    assert_eq!(snapshot.digest, blake3::hash(substituted));
    assert_eq!(
        verify_snapshot_subject(&snapshot, &expected)
            .expect_err("substituted snapshot must fail")
            .code(),
        "INVALID_RELEASE_BUNDLE"
    );
    fs::remove_file(path).expect("remove signature fixture");
}

#[test]
fn bounded_read_rejects_more_than_maximum_even_when_metadata_reports_zero() {
    let mut input = File::open("/proc/self/cmdline").expect("open proc command line");
    assert_eq!(input.metadata().expect("command line metadata").len(), 0);
    assert_eq!(
        read_open_bounded(&mut input, 1)
            .expect_err("maximum plus one byte must fail")
            .code(),
        "INVALID_RELEASE_BUNDLE"
    );
}

#[test]
fn fifo_final_component_fails_without_waiting_for_a_writer() {
    let path = fixture_path("fifo");
    mkfifo(&path, Mode::S_IRUSR | Mode::S_IWUSR).expect("create FIFO");
    assert_eq!(
        open_bounded_file(&path, "release input", 64)
            .expect_err("FIFO must fail admission")
            .code(),
        "INVALID_RELEASE_BUNDLE"
    );
    fs::remove_file(path).expect("remove FIFO fixture");
}

fn fixture_path(label: &str) -> std::path::PathBuf {
    let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "bullet-release-verify-{label}-{}-{sequence}",
        std::process::id()
    ))
}
