use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use tempfile::TempDir;

use super::*;

const PRINCIPAL: &str = "release@bullet.invalid";

struct Key {
    path: PathBuf,
    public_key: String,
    identity: String,
}

struct Fixture {
    root: TempDir,
    key: Key,
    receipt_path: PathBuf,
    signature_path: PathBuf,
    policy_path: PathBuf,
    receipt: ReleaseReceipt,
    policy: ReleaseReceiptPolicy,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("bullet-release-receipt-")
            .tempdir()
            .expect("temporary receipt root");
        let key = create_key(root.path(), "release-key", PRINCIPAL);
        let policy = policy(&key, vec![ReleaseReceiptKind::RustMsrv195]);
        let policy_path = root.path().join("policy.toml");
        fs::write(&policy_path, policy.canonical_bytes().unwrap()).unwrap();
        let mut receipt = receipt(&key.identity);
        receipt.policy_digest = policy.digest().unwrap();
        let receipt_path = root.path().join("receipt.toml");
        fs::write(&receipt_path, receipt.canonical_bytes().unwrap()).unwrap();
        let signature_path = PathBuf::from(format!("{}.sig", receipt_path.display()));
        sign(root.path(), &key.path, &receipt_path, SIGNATURE_NAMESPACE);
        Self {
            root,
            key,
            receipt_path,
            signature_path,
            policy_path,
            receipt,
            policy,
        }
    }

    fn write_receipt(&self, receipt: &ReleaseReceipt, key: &Path, namespace: &str) {
        fs::write(&self.receipt_path, receipt.canonical_bytes().unwrap()).unwrap();
        if self.signature_path.exists() {
            fs::remove_file(&self.signature_path).unwrap();
        }
        sign(self.root.path(), key, &self.receipt_path, namespace);
    }
}

#[test]
fn canonical_schema_goldens_round_trip_exactly() {
    let receipt = receipt("release@bullet.invalid|ed25519|SHA256:abcdefghijklmnop");
    let receipt_golden = concat!(
        "release_receipt_schema_version = \"1\"\n",
        "receipt_kind = \"rust-msrv-1-95\"\n",
        "family = \"bullet-farm\"\n",
        "tag = \"v1.0.0\"\n",
        "hub_commit_oid = \"sha1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
        "hub_tree_oid = \"sha1:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\n",
        "tool_name = \"rustc\"\n",
        "tool_version = \"1.95.0\"\n",
        "tool_digest = \"blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\"\n",
        "profile = \"release-msrv\"\n",
        "configuration_digest = \"blake3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\"\n",
        "subject_digest = \"blake3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\"\n",
        "result = \"VERIFIED\"\n",
        "result_digest = \"blake3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"\n",
        "started_at_unix_ms = 1700000000000\n",
        "completed_at_unix_ms = 1700000001000\n",
        "expires_at_unix_ms = 1700100000000\n",
        "policy_digest = \"blake3:1111111111111111111111111111111111111111111111111111111111111111\"\n",
        "release_signing_identity = \"release@bullet.invalid|ed25519|SHA256:abcdefghijklmnop\"\n",
    );
    assert_eq!(
        receipt.canonical_bytes().unwrap(),
        receipt_golden.as_bytes()
    );
    assert_eq!(
        ReleaseReceipt::parse(receipt_golden.as_bytes()).unwrap(),
        receipt
    );

    let key = fake_key();
    let policy = policy(&key, vec![ReleaseReceiptKind::RustMsrv195]);
    let policy_golden = format!(
        concat!(
            "release_receipt_policy_schema_version = \"1\"\n",
            "family = \"bullet-farm\"\n",
            "signature_namespace = \"bullet-farm-release-receipt-v1\"\n\n",
            "[[signer]]\n",
            "release_signing_identity = \"release@bullet.invalid|ed25519|SHA256:abcdefghijklmnop\"\n",
            "public_key = \"ssh-ed25519 {}\"\n",
            "receipt_kind = [\"rust-msrv-1-95\"]\n",
            "valid_from_unix_ms = 1699999999000\n",
            "valid_until_unix_ms = 1700200000000\n",
        ),
        "A".repeat(68)
    );
    assert_eq!(policy.canonical_bytes().unwrap(), policy_golden.as_bytes());
    assert_eq!(
        ReleaseReceiptPolicy::parse(policy_golden.as_bytes()).unwrap(),
        policy
    );
}

#[test]
fn schemas_reject_duplicate_unknown_noncanonical_legacy_and_oversized_inputs() {
    let canonical = String::from_utf8(
        receipt("release@bullet.invalid|ed25519|SHA256:abcdefghijklmnop")
            .canonical_bytes()
            .unwrap(),
    )
    .unwrap();
    for hostile in [
        canonical.replacen(
            "family = \"bullet-farm\"",
            "family = \"bullet-farm\"\nlegacy = true",
            1,
        ),
        format!("{canonical}family = \"bullet-farm\"\n"),
        format!("\n{canonical}"),
        canonical.replace("result = \"VERIFIED\"", "result = \"PASS\""),
        canonical.replace("hub_commit_oid = \"sha1:", "hub_commit_oid = \""),
        canonical.replace(&digest('c'), &digest('A')),
    ] {
        assert_eq!(
            ReleaseReceipt::parse(hostile.as_bytes())
                .unwrap_err()
                .code(),
            "INVALID_RELEASE_RECEIPT"
        );
    }
    assert_eq!(
        ReleaseReceipt::parse(&vec![b'x'; MAX_RECEIPT_BYTES as usize + 1])
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_RECEIPT"
    );

    let key = fake_key();
    let canonical = String::from_utf8(
        policy(&key, vec![ReleaseReceiptKind::RustMsrv195])
            .canonical_bytes()
            .unwrap(),
    )
    .unwrap();
    let key_line = format!("public_key = {:?}", key.public_key);
    for hostile in [
        canonical.replacen(&key_line, &format!("{key_line}\n{key_line}"), 1),
        canonical.replacen(
            "family = \"bullet-farm\"",
            "family = \"bullet-farm\"\nlegacy = true",
            1,
        ),
        format!("\n{canonical}"),
    ] {
        assert_eq!(
            ReleaseReceiptPolicy::parse(hostile.as_bytes())
                .unwrap_err()
                .code(),
            "INVALID_RELEASE_RECEIPT_POLICY"
        );
    }
    let unsorted = policy(
        &key,
        vec![
            ReleaseReceiptKind::TransactionDemo,
            ReleaseReceiptKind::BackupRestore,
        ],
    );
    assert_eq!(
        unsorted.canonical_bytes().unwrap_err().code(),
        "INVALID_RELEASE_RECEIPT_POLICY"
    );
}

#[test]
fn exact_signed_receipt_verifies_read_only_without_clearing_a_gate() {
    let fixture = Fixture::new();
    let before = snapshot(&fixture);
    let verified = verify(
        &fixture.receipt_path,
        &fixture.signature_path,
        &fixture.policy_path,
    )
    .unwrap();
    assert_eq!(verified.receipt, fixture.receipt);
    assert_eq!(verified.policy_digest, fixture.policy.digest().unwrap());
    let output = crate::release::run(&[
        "receipt-verify".to_owned(),
        "--receipt".to_owned(),
        text(&fixture.receipt_path).to_owned(),
        "--signature".to_owned(),
        text(&fixture.signature_path).to_owned(),
        "--policy".to_owned(),
        text(&fixture.policy_path).to_owned(),
    ])
    .unwrap();
    assert!(output.contains("contract only; no release gate cleared"));
    assert_eq!(snapshot(&fixture), before);
}

#[test]
fn authorization_and_signature_mutations_fail_closed() {
    let fixture = Fixture::new();
    let mut changed = fixture.receipt.clone();
    changed.profile = "changed-profile".to_owned();
    fs::write(&fixture.receipt_path, changed.canonical_bytes().unwrap()).unwrap();
    assert_eq!(verify_fixture(&fixture), "RELEASE_SIGNATURE_INVALID");

    let fixture = Fixture::new();
    fixture.write_receipt(&fixture.receipt, &fixture.key.path, "wrong-namespace");
    assert_eq!(verify_fixture(&fixture), "RELEASE_SIGNATURE_INVALID");

    let fixture = Fixture::new();
    let mut policy = fixture.policy.clone();
    policy.signer[0].receipt_kind = vec![
        ReleaseReceiptKind::ProviderCodex,
        ReleaseReceiptKind::RustMsrv195,
    ];
    fs::write(&fixture.policy_path, policy.canonical_bytes().unwrap()).unwrap();
    assert_eq!(verify_fixture(&fixture), "RELEASE_RECEIPT_POLICY_MISMATCH");

    let fixture = Fixture::new();
    let mut changed = fixture.receipt.clone();
    changed.receipt_kind = ReleaseReceiptKind::ProviderCodex;
    fixture.write_receipt(&changed, &fixture.key.path, SIGNATURE_NAMESPACE);
    assert_eq!(verify_fixture(&fixture), "RELEASE_RECEIPT_NOT_AUTHORIZED");

    let fixture = Fixture::new();
    let mut changed = fixture.receipt.clone();
    changed.started_at_unix_ms = fixture.policy.signer[0].valid_from_unix_ms - 1;
    fixture.write_receipt(&changed, &fixture.key.path, SIGNATURE_NAMESPACE);
    assert_eq!(verify_fixture(&fixture), "RELEASE_RECEIPT_NOT_AUTHORIZED");

    let fixture = Fixture::new();
    let rogue = create_key(fixture.root.path(), "rogue-key", "rogue@bullet.invalid");
    let mut changed = fixture.receipt.clone();
    changed.release_signing_identity = rogue.identity.clone();
    fixture.write_receipt(&changed, &rogue.path, SIGNATURE_NAMESPACE);
    assert_eq!(
        verify_fixture(&fixture),
        "RELEASE_RECEIPT_SIGNER_NOT_ALLOWED"
    );
}

#[test]
fn symlink_policy_is_never_a_trust_root() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    fs::remove_file(&fixture.policy_path).unwrap();
    symlink(&fixture.receipt_path, &fixture.policy_path).unwrap();
    assert_eq!(verify_fixture(&fixture), "INVALID_RELEASE_BUNDLE");
}

fn receipt(identity: &str) -> ReleaseReceipt {
    ReleaseReceipt {
        release_receipt_schema_version: RELEASE_RECEIPT_SCHEMA_VERSION.to_owned(),
        receipt_kind: ReleaseReceiptKind::RustMsrv195,
        family: FAMILY.to_owned(),
        tag: "v1.0.0".to_owned(),
        hub_commit_oid: format!("sha1:{}", "a".repeat(40)),
        hub_tree_oid: format!("sha1:{}", "b".repeat(40)),
        tool_name: "rustc".to_owned(),
        tool_version: "1.95.0".to_owned(),
        tool_digest: digest('c'),
        profile: "release-msrv".to_owned(),
        configuration_digest: digest('d'),
        subject_digest: digest('e'),
        result: ReleaseReceiptResult::Verified,
        result_digest: digest('f'),
        started_at_unix_ms: 1_700_000_000_000,
        completed_at_unix_ms: 1_700_000_001_000,
        expires_at_unix_ms: 1_700_100_000_000,
        policy_digest: digest('1'),
        release_signing_identity: identity.to_owned(),
    }
}

fn policy(key: &Key, receipt_kind: Vec<ReleaseReceiptKind>) -> ReleaseReceiptPolicy {
    ReleaseReceiptPolicy {
        release_receipt_policy_schema_version: RELEASE_RECEIPT_POLICY_SCHEMA_VERSION.to_owned(),
        family: FAMILY.to_owned(),
        signature_namespace: SIGNATURE_NAMESPACE.to_owned(),
        signer: vec![ReleaseReceiptSigner {
            release_signing_identity: key.identity.clone(),
            public_key: key.public_key.clone(),
            receipt_kind,
            valid_from_unix_ms: 1_699_999_999_000,
            valid_until_unix_ms: 1_700_200_000_000,
        }],
    }
}

fn fake_key() -> Key {
    Key {
        path: PathBuf::new(),
        public_key: format!("ssh-ed25519 {}", "A".repeat(68)),
        identity: "release@bullet.invalid|ed25519|SHA256:abcdefghijklmnop".to_owned(),
    }
}

fn create_key(root: &Path, name: &str, principal: &str) -> Key {
    let path = root.join(name);
    run(root, ["-q", "-t", "ed25519", "-N", "", "-f", text(&path)]);
    let public_key = fs::read_to_string(path.with_extension("pub")).unwrap();
    let public_key = public_key
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");
    let public_path = path.with_extension("pub");
    let output = run_output(root, ["-lf", text(&public_path), "-E", "sha256"]);
    let fingerprint = String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_owned();
    Key {
        path,
        public_key,
        identity: format!("{principal}|ed25519|{fingerprint}"),
    }
}

fn sign(root: &Path, key: &Path, receipt: &Path, namespace: &str) {
    run(
        root,
        [
            "-Y",
            "sign",
            "-f",
            text(key),
            "-n",
            namespace,
            text(receipt),
        ],
    );
}

fn run<'a>(root: &Path, args: impl IntoIterator<Item = &'a str>) {
    let output = run_output(root, args);
    assert!(
        output.status.success(),
        "ssh-keygen failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_output<'a>(root: &Path, args: impl IntoIterator<Item = &'a str>) -> Output {
    Command::new("/usr/bin/ssh-keygen")
        .current_dir(root)
        .args(args)
        .env_clear()
        .env("HOME", "/")
        .env("LC_ALL", "C")
        .env("PATH", "/usr/bin")
        .output()
        .unwrap()
}

fn verify_fixture(fixture: &Fixture) -> &'static str {
    verify(
        &fixture.receipt_path,
        &fixture.signature_path,
        &fixture.policy_path,
    )
    .unwrap_err()
    .code()
}

fn snapshot(fixture: &Fixture) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = fs::read_dir(fixture.root.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file())
        .map(|path| (path.clone(), fs::read(path).unwrap()))
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn digest(byte: char) -> String {
    format!("blake3:{}", byte.to_string().repeat(64))
}

fn text(path: &Path) -> &str {
    path.to_str().unwrap()
}
