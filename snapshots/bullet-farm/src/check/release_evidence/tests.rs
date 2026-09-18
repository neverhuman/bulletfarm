use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use super::{
    admission::{
        AdmittedPolicy, OperatorFile, reject_same_inputs, secure_ancestors, validate_key_roles,
    },
    schema::{
        CommandKind, FAMILY, GATE_ID, MsrvPolicy, MsrvReceipt, MsrvSubject, POLICY_SCHEMA,
        RECEIPT_NAMESPACE, RECEIPT_SCHEMA, TIME_SCHEMA, TOOLCHAIN, ToolSubject,
        TrustedTimeObservation,
    },
    verify::{ExactFamily, expected_commands, verify_receipt, verify_time},
};

const ATTESTOR: &str = "attestor@bullet.invalid|ed25519|SHA256:abcdefghijklmnop";
const TIME: &str = "time@bullet.invalid|ed25519|SHA256:ponmlkjihgfedcba";

#[test]
fn semantic_receipt_requires_exact_commands_counts_subjects_and_fresh_time() {
    let admitted = admitted_policy();
    let exact = exact_family();
    let receipt = receipt(&admitted, &exact);
    verify_receipt(&receipt, &exact, &admitted).unwrap();
    let receipt_digest = digest(&receipt.canonical_bytes().unwrap());
    let time = trusted_time(&receipt, &admitted, &receipt_digest);
    verify_time(&receipt, &time, &receipt_digest, &admitted, 2_000_000).unwrap();

    let mut wrong = receipt.clone();
    wrong.subject[0].tree_oid = oid('9');
    assert!(verify_receipt(&wrong, &exact, &admitted).is_err());

    let mut wrong = receipt.clone();
    wrong.command[0].argv.push("--release".to_owned());
    assert!(verify_receipt(&wrong, &exact, &admitted).is_err());

    let mut wrong = receipt.clone();
    wrong.command[1].tests_passed = 0;
    assert!(verify_receipt(&wrong, &exact, &admitted).is_err());

    let mut wrong = receipt.clone();
    wrong.command[1].tests_skipped = 1;
    assert!(verify_receipt(&wrong, &exact, &admitted).is_err());

    let mut wrong = receipt.clone();
    wrong.cargo.digest = tagged_digest('9');
    assert!(verify_receipt(&wrong, &exact, &admitted).is_err());

    let mut stale = time.clone();
    stale.observed_at_unix_ms = 1_800_000;
    stale.valid_until_unix_ms = 2_100_000;
    assert!(verify_time(&receipt, &stale, &receipt_digest, &admitted, 2_000_000).is_err());

    let mut future = time.clone();
    future.observed_at_unix_ms = 2_100_001;
    assert!(verify_time(&receipt, &future, &receipt_digest, &admitted, 2_000_000).is_err());

    let mut replay = time;
    replay.receipt_digest = tagged_digest('8');
    assert!(verify_time(&receipt, &replay, &receipt_digest, &admitted, 2_000_000).is_err());
}

#[test]
fn canonical_schemas_reject_unknown_and_noncanonical_fields() {
    let admitted = admitted_policy();
    let exact = exact_family();
    let receipt = receipt(&admitted, &exact);
    let canonical = String::from_utf8(receipt.canonical_bytes().unwrap()).unwrap();
    let hostile = canonical.replacen(
        "family = \"bullet-farm\"",
        "family = \"bullet-farm\"\nunknown = true",
        1,
    );
    assert_eq!(
        MsrvReceipt::parse(hostile.as_bytes()).unwrap_err().code(),
        "INVALID_MSRV_RELEASE_EVIDENCE"
    );
    assert!(MsrvReceipt::parse(format!("\n{canonical}").as_bytes()).is_err());

    let policy_bytes = admitted.policy.canonical_bytes().unwrap();
    let mut hostile = String::from_utf8(policy_bytes).unwrap();
    hostile.push_str("legacy = true\n");
    assert!(MsrvPolicy::parse(hostile.as_bytes()).is_err());

    let receipt_digest = digest(&receipt.canonical_bytes().unwrap());
    let time = trusted_time(&receipt, &admitted, &receipt_digest);
    let canonical = String::from_utf8(time.canonical_bytes().unwrap()).unwrap();
    let hostile = canonical.replacen(
        "family = \"bullet-farm\"",
        "family = \"bullet-farm\"\nunknown = true",
        1,
    );
    assert!(TrustedTimeObservation::parse(hostile.as_bytes()).is_err());
}

#[test]
fn detached_verifier_binds_exact_payload_namespace_and_attestor() {
    let root = tempfile::Builder::new()
        .prefix("bullet-msrv-detached-")
        .tempdir()
        .unwrap();
    let key = root.path().join("attestor");
    run(
        root.path(),
        ["-q", "-t", "ed25519", "-N", "", "-f", text(&key)],
    );
    let public = fs::read_to_string(key.with_extension("pub")).unwrap();
    let fingerprint = String::from_utf8(
        run_output(
            root.path(),
            ["-lf", text(&key.with_extension("pub")), "-E", "sha256"],
        )
        .stdout,
    )
    .unwrap()
    .split_whitespace()
    .nth(1)
    .unwrap()
    .to_owned();
    let identity = format!("attestor@bullet.invalid|ed25519|{fingerprint}");
    let allowed = root.path().join("allowed_signers");
    fs::write(
        &allowed,
        format!(
            "attestor@bullet.invalid namespaces=\"{RECEIPT_NAMESPACE}\" {}\n",
            public
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
        ),
    )
    .unwrap();
    let payload = root.path().join("receipt.toml");
    fs::write(&payload, b"canonical semantic receipt\n").unwrap();
    run(
        root.path(),
        [
            "-Y",
            "sign",
            "-f",
            text(&key),
            "-n",
            RECEIPT_NAMESPACE,
            text(&payload),
        ],
    );
    let signature = PathBuf::from(format!("{}.sig", payload.display()));
    let verified = crate::release::verify_detached(
        &payload,
        &signature,
        &allowed,
        &identity,
        RECEIPT_NAMESPACE,
        "fixture semantic receipt",
        4096,
    )
    .unwrap();
    assert_eq!(verified.bytes, b"canonical semantic receipt\n");
    fs::write(&payload, b"substituted receipt\n").unwrap();
    assert!(
        crate::release::verify_detached(
            &payload,
            &signature,
            &allowed,
            &identity,
            RECEIPT_NAMESPACE,
            "fixture semantic receipt",
            4096,
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn mutable_ancestors_hardlinks_and_copied_role_keys_are_rejected() {
    let root = tempfile::Builder::new()
        .prefix("bullet-msrv-hostile-admission-")
        .tempdir()
        .unwrap();
    let leaf = root.path().join("operator-policy");
    fs::write(&leaf, "policy\n").unwrap();
    assert!(secure_ancestors(Path::new("/etc/passwd"), "root-owned control").is_ok());
    assert!(secure_ancestors(&leaf, "hostile policy").is_err());

    let linked = root.path().join("linked-policy");
    fs::hard_link(&leaf, &linked).unwrap();
    let metadata = fs::metadata(&leaf).unwrap();
    let first = OperatorFile {
        bytes: Vec::new(),
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    let metadata = fs::metadata(&linked).unwrap();
    let second = OperatorFile {
        bytes: Vec::new(),
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    assert!(reject_same_inputs(&[&first, &second]).is_err());

    let policy = admitted_policy().policy;
    let source_key = "A".repeat(68);
    let distinct_attestor = "B".repeat(68);
    let distinct_time = "C".repeat(68);
    let source = format!("source@bullet.invalid ssh-ed25519 {source_key}\n");
    let attestor = format!(
        "attestor@bullet.invalid namespaces=\"{RECEIPT_NAMESPACE}\" ssh-ed25519 {distinct_attestor}\n"
    );
    let time = format!(
        "time@bullet.invalid namespaces=\"{}\" ssh-ed25519 {distinct_time}\n",
        super::schema::TIME_NAMESPACE
    );
    validate_key_roles(
        source.as_bytes(),
        attestor.as_bytes(),
        time.as_bytes(),
        &policy,
    )
    .unwrap();
    let copied = format!(
        "time@bullet.invalid namespaces=\"{}\" ssh-ed25519 {distinct_attestor}\n",
        super::schema::TIME_NAMESPACE
    );
    assert!(
        validate_key_roles(
            source.as_bytes(),
            attestor.as_bytes(),
            copied.as_bytes(),
            &policy,
        )
        .is_err()
    );
    let copied_source = format!(
        "attestor@bullet.invalid namespaces=\"{RECEIPT_NAMESPACE}\" ssh-ed25519 {source_key}\n"
    );
    assert!(
        validate_key_roles(
            source.as_bytes(),
            copied_source.as_bytes(),
            time.as_bytes(),
            &policy,
        )
        .is_err()
    );

    let mut same_fingerprint = policy;
    same_fingerprint.trusted_time_identity =
        "different-principal@bullet.invalid|ed25519|SHA256:abcdefghijklmnop".to_owned();
    assert!(same_fingerprint.canonical_bytes().is_err());
}

fn admitted_policy() -> AdmittedPolicy {
    AdmittedPolicy {
        policy: MsrvPolicy {
            release_msrv_policy_schema_version: POLICY_SCHEMA.to_owned(),
            family: FAMILY.to_owned(),
            gate_id: GATE_ID.to_owned(),
            evidence_directory: PathBuf::from("/var/lib/bullet-farm/evidence"),
            source_allowed_signers_path: PathBuf::from("/etc/bullet-farm/source-signers"),
            attestor_allowed_signers_path: PathBuf::from("/etc/bullet-farm/attestor-signers"),
            trusted_time_allowed_signers_path: PathBuf::from("/etc/bullet-farm/time-signers"),
            attestor_identity: ATTESTOR.to_owned(),
            trusted_time_identity: TIME.to_owned(),
            rustc: ToolSubject {
                path: PathBuf::from("/opt/rust-1.95/bin/rustc"),
                version: "rustc 1.95.0 (fixture)".to_owned(),
                digest: tagged_digest('a'),
            },
            cargo: ToolSubject {
                path: PathBuf::from("/opt/rust-1.95/bin/cargo"),
                version: "cargo 1.95.0 (fixture)".to_owned(),
                digest: tagged_digest('b'),
            },
            maximum_run_duration_ms: 1_000_000,
            maximum_receipt_age_ms: 100_000,
            maximum_time_observation_age_ms: 100_000,
            maximum_future_skew_ms: 100_000,
        },
        policy_digest: tagged_digest('c'),
    }
}

fn exact_family() -> ExactFamily {
    let receipt_subjects = [
        "bullet-farm",
        "bullet-git",
        "bullet-kernel",
        "bullet-portal",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, repository)| MsrvSubject {
        repository: repository.to_owned(),
        tag: "v1.0.0".to_owned(),
        commit_oid: oid(char::from_digit((index + 1) as u32, 10).unwrap()),
        tree_oid: oid(char::from_digit((index + 5) as u32, 10).unwrap()),
        lockfile_path: if repository == "bullet-portal" {
            "package-lock.json".to_owned()
        } else {
            "Cargo.lock".to_owned()
        },
        lockfile_digest: tagged_digest(char::from_digit((index + 1) as u32, 16).unwrap()),
        release_signing_identity: format!(
            "source-{index}@bullet.invalid|ed25519|SHA256:abcdefghijklmnop"
        ),
    })
    .collect();
    ExactFamily {
        family_lock_digest: tagged_digest('d'),
        receipt_subjects,
        gate_subjects: Vec::new(),
    }
}

fn receipt(admitted: &AdmittedPolicy, exact: &ExactFamily) -> MsrvReceipt {
    let mut command = expected_commands(&admitted.policy);
    for observation in &mut command {
        observation.output_digest = tagged_digest('e');
        match observation.kind {
            CommandKind::Build => observation.build_units = 1,
            CommandKind::Test => observation.tests_passed = 1,
        }
    }
    MsrvReceipt {
        release_msrv_receipt_schema_version: RECEIPT_SCHEMA.to_owned(),
        family: FAMILY.to_owned(),
        gate_id: GATE_ID.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        evidence_nonce: "1".repeat(64),
        policy_digest: admitted.policy_digest.clone(),
        family_lock_digest: exact.family_lock_digest.clone(),
        rustc: admitted.policy.rustc.clone(),
        cargo: admitted.policy.cargo.clone(),
        subject: exact.receipt_subjects.clone(),
        command,
        started_at_unix_ms: 1_900_000,
        completed_at_unix_ms: 1_950_000,
        expires_at_unix_ms: 2_200_000,
        attestor_identity: admitted.policy.attestor_identity.clone(),
    }
}

fn trusted_time(
    receipt: &MsrvReceipt,
    admitted: &AdmittedPolicy,
    receipt_digest: &str,
) -> TrustedTimeObservation {
    TrustedTimeObservation {
        trusted_time_schema_version: TIME_SCHEMA.to_owned(),
        family: FAMILY.to_owned(),
        gate_id: GATE_ID.to_owned(),
        evidence_nonce: receipt.evidence_nonce.clone(),
        receipt_digest: receipt_digest.to_owned(),
        policy_digest: admitted.policy_digest.clone(),
        observed_at_unix_ms: 1_990_000,
        valid_until_unix_ms: 2_100_000,
        trusted_time_identity: admitted.policy.trusted_time_identity.clone(),
    }
}

fn tagged_digest(value: char) -> String {
    format!("blake3:{}", value.to_string().repeat(64))
}

fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn oid(value: char) -> String {
    format!("sha1:{}", value.to_string().repeat(40))
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

fn text(path: &Path) -> &str {
    path.to_str().expect("fixture path is UTF-8")
}
