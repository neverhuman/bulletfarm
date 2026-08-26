//! Exact-subject and semantic verification after operator admission.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    admission::AdmittedPolicy,
    schema::{
        CommandKind, CommandObservation, EnvironmentBinding, FAMILY, GATE_ID, MAX_INPUT_BYTES,
        MsrvReceipt, MsrvSubject, RECEIPT_NAMESPACE, TIME_NAMESPACE, TOOLCHAIN,
        TrustedTimeObservation, identity_fingerprint,
    },
};
use crate::{
    check::{
        executor::{RepositorySet, git, git_bytes},
        subject::RepositorySubject,
    },
    coord::CoordError,
    family_lock,
    release::verify_detached,
};

const REQUIRED_REPOSITORIES: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];
const RUST_REPOSITORIES: [&str; 3] = ["bullet-farm", "bullet-git", "bullet-kernel"];
const RECEIPT_FILE: &str = "rust-msrv-1-95.toml";
const RECEIPT_SIGNATURE: &str = "rust-msrv-1-95.toml.sig";
const TIME_FILE: &str = "rust-msrv-1-95.trusted-time.toml";
const TIME_SIGNATURE: &str = "rust-msrv-1-95.trusted-time.toml.sig";

pub(super) struct VerifiedMsrv {
    pub detail: String,
    pub subjects: Vec<RepositorySubject>,
}

pub(super) struct ExactFamily {
    pub(super) family_lock_digest: String,
    pub(super) receipt_subjects: Vec<MsrvSubject>,
    pub(super) gate_subjects: Vec<RepositorySubject>,
}

pub(super) fn evaluate(
    repositories: &RepositorySet,
    admitted: &AdmittedPolicy,
) -> Result<VerifiedMsrv, CoordError> {
    let exact = exact_family(repositories, admitted)?;
    reject_non_independent_identities(&exact, admitted)?;
    let evidence = &admitted.policy.evidence_directory;
    let receipt_subject = verify_detached(
        &evidence.join(RECEIPT_FILE),
        &evidence.join(RECEIPT_SIGNATURE),
        &admitted.policy.attestor_allowed_signers_path,
        &admitted.policy.attestor_identity,
        RECEIPT_NAMESPACE,
        "MSRV semantic receipt signature",
        MAX_INPUT_BYTES,
    )?;
    let receipt = MsrvReceipt::parse(&receipt_subject.bytes)?;
    verify_receipt(&receipt, &exact, admitted)?;

    let time_subject = verify_detached(
        &evidence.join(TIME_FILE),
        &evidence.join(TIME_SIGNATURE),
        &admitted.policy.trusted_time_allowed_signers_path,
        &admitted.policy.trusted_time_identity,
        TIME_NAMESPACE,
        "MSRV trusted-time signature",
        MAX_INPUT_BYTES,
    )?;
    let time = TrustedTimeObservation::parse(&time_subject.bytes)?;
    verify_time(
        &receipt,
        &time,
        &receipt_subject.digest,
        admitted,
        now_unix_ms()?,
    )?;
    Ok(VerifiedMsrv {
        detail: format!(
            "independently attested Rust 1.95 build/test receipt {} binds the current signed family",
            receipt_subject.digest
        ),
        subjects: exact.gate_subjects,
    })
}

fn exact_family(
    repositories: &RepositorySet,
    admitted: &AdmittedPolicy,
) -> Result<ExactFamily, CoordError> {
    let hub = repositories.path("bullet-farm")?;
    let lock = family_lock::load(&hub.join("family.lock"))?;
    let required = REQUIRED_REPOSITORIES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    lock.validate_required_members(&required)?;
    let hub_signer =
        family_lock::verify_hub_checkout(&lock, hub, &admitted.policy.source_allowed_signers_path)?;
    for member in &lock.member {
        family_lock::verify_locked_checkout(
            member,
            repositories.path(&member.name)?,
            &admitted.policy.source_allowed_signers_path,
        )?;
    }
    let tagged_lock = git_bytes(hub, &["show", &format!("{}:family.lock", lock.tag)])?;
    if family_lock::parse(&tagged_lock)? != lock {
        return Err(invalid("signed Hub tag and admitted family.lock differ"));
    }
    let family_lock_digest = format!("blake3:{}", blake3::hash(&tagged_lock).to_hex());
    let (hub_commit, hub_tree) = current_subject(hub)?;
    let hub_lockfile = tagged_regular_file_digest(hub, &lock.tag, "Cargo.lock")?;

    let mut receipt_subjects = vec![MsrvSubject {
        repository: "bullet-farm".to_owned(),
        tag: lock.tag.clone(),
        commit_oid: hub_commit.clone(),
        tree_oid: hub_tree.clone(),
        lockfile_path: "Cargo.lock".to_owned(),
        lockfile_digest: hub_lockfile,
        release_signing_identity: hub_signer,
    }];
    let mut gate_subjects =
        vec![RepositorySubject::new("bullet-farm", hub_commit, hub_tree).map_err(model_error)?];
    for name in REQUIRED_REPOSITORIES.iter().skip(1) {
        let member = lock
            .member(name)
            .ok_or_else(|| invalid(format!("family.lock omits {name}")))?;
        if member.lockfile.len() != 1 {
            return Err(invalid(format!(
                "{name} does not bind exactly one dependency lock"
            )));
        }
        receipt_subjects.push(MsrvSubject {
            repository: (*name).to_owned(),
            tag: member.tag.clone(),
            commit_oid: member.commit_oid.clone(),
            tree_oid: member.tree_oid.clone(),
            lockfile_path: member.lockfile[0].path.clone(),
            lockfile_digest: member.lockfile[0].digest.clone(),
            release_signing_identity: member.release_signing_identity.clone(),
        });
        gate_subjects.push(
            RepositorySubject::new(*name, &member.commit_oid, &member.tree_oid)
                .map_err(model_error)?,
        );
    }
    Ok(ExactFamily {
        family_lock_digest,
        receipt_subjects,
        gate_subjects,
    })
}

pub(super) fn verify_receipt(
    receipt: &MsrvReceipt,
    exact: &ExactFamily,
    admitted: &AdmittedPolicy,
) -> Result<(), CoordError> {
    let policy = &admitted.policy;
    if receipt.family != FAMILY
        || receipt.gate_id != GATE_ID
        || receipt.toolchain != TOOLCHAIN
        || receipt.policy_digest != admitted.policy_digest
        || receipt.family_lock_digest != exact.family_lock_digest
        || receipt.rustc != policy.rustc
        || receipt.cargo != policy.cargo
        || receipt.attestor_identity != policy.attestor_identity
        || receipt.subject != exact.receipt_subjects
    {
        return Err(invalid(
            "MSRV receipt differs from the admitted policy or exact signed family subjects",
        ));
    }
    if receipt.completed_at_unix_ms - receipt.started_at_unix_ms > policy.maximum_run_duration_ms {
        return Err(invalid(
            "MSRV receipt run duration exceeds the admitted bound",
        ));
    }
    let expected = expected_commands(policy);
    if receipt.command.len() != expected.len() {
        return Err(invalid("MSRV receipt command inventory is incomplete"));
    }
    for (actual, expected) in receipt.command.iter().zip(expected) {
        if actual.repository != expected.repository
            || actual.kind != expected.kind
            || actual.program != expected.program
            || actual.argv != expected.argv
            || actual.environment != expected.environment
            || actual.exit_code != 0
            || actual.tests_failed != 0
            || actual.tests_skipped != 0
        {
            return Err(invalid(
                "MSRV receipt command, environment, or outcome is not admitted",
            ));
        }
        match actual.kind {
            CommandKind::Build if actual.build_units == 0 || actual.tests_passed != 0 => {
                return Err(invalid(
                    "MSRV build observation is empty or claims test results",
                ));
            }
            CommandKind::Test if actual.tests_passed == 0 => {
                return Err(invalid("MSRV test observation contains zero passing tests"));
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn verify_time(
    receipt: &MsrvReceipt,
    time: &TrustedTimeObservation,
    receipt_digest: &str,
    admitted: &AdmittedPolicy,
    now: u64,
) -> Result<(), CoordError> {
    let policy = &admitted.policy;
    if time.family != FAMILY
        || time.gate_id != GATE_ID
        || time.evidence_nonce != receipt.evidence_nonce
        || time.receipt_digest != receipt_digest
        || time.policy_digest != admitted.policy_digest
        || time.trusted_time_identity != policy.trusted_time_identity
    {
        return Err(invalid(
            "trusted-time observation does not bind this exact receipt, nonce, and policy",
        ));
    }
    if time.observed_at_unix_ms > now.saturating_add(policy.maximum_future_skew_ms)
        || now > time.valid_until_unix_ms
        || now.saturating_sub(time.observed_at_unix_ms) > policy.maximum_time_observation_age_ms
        || receipt.completed_at_unix_ms > time.observed_at_unix_ms
        || time
            .observed_at_unix_ms
            .saturating_sub(receipt.completed_at_unix_ms)
            > policy.maximum_receipt_age_ms
        || now >= receipt.expires_at_unix_ms
        || time.valid_until_unix_ms > receipt.expires_at_unix_ms
    {
        return Err(invalid(
            "MSRV receipt or trusted-time observation is stale, future, or expired",
        ));
    }
    Ok(())
}

fn reject_non_independent_identities(
    exact: &ExactFamily,
    admitted: &AdmittedPolicy,
) -> Result<(), CoordError> {
    for subject in &exact.receipt_subjects {
        let source = identity_fingerprint(&subject.release_signing_identity);
        if source == identity_fingerprint(&admitted.policy.attestor_identity)
            || source == identity_fingerprint(&admitted.policy.trusted_time_identity)
        {
            return Err(invalid(
                "source-tag, build-attestor, and trusted-time identities must be independent",
            ));
        }
    }
    Ok(())
}

pub(super) fn expected_commands(policy: &super::schema::MsrvPolicy) -> Vec<CommandObservation> {
    let environment = vec![
        EnvironmentBinding {
            name: "CARGO_INCREMENTAL".to_owned(),
            value: "0".to_owned(),
        },
        EnvironmentBinding {
            name: "CARGO_NET_OFFLINE".to_owned(),
            value: "true".to_owned(),
        },
        EnvironmentBinding {
            name: "RUSTC".to_owned(),
            value: policy.rustc.path.display().to_string(),
        },
        EnvironmentBinding {
            name: "RUSTUP_TOOLCHAIN".to_owned(),
            value: TOOLCHAIN.to_owned(),
        },
    ];
    let mut commands = Vec::with_capacity(6);
    for repository in RUST_REPOSITORIES {
        commands.push(CommandObservation {
            repository: repository.to_owned(),
            kind: CommandKind::Build,
            program: policy.cargo.path.clone(),
            argv: ["build", "--workspace", "--all-targets", "--locked"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            environment: environment.clone(),
            build_units: 0,
            tests_passed: 0,
            tests_failed: 0,
            tests_skipped: 0,
            exit_code: 0,
            output_digest: format!("blake3:{}", "0".repeat(64)),
        });
        commands.push(CommandObservation {
            repository: repository.to_owned(),
            kind: CommandKind::Test,
            program: policy.cargo.path.clone(),
            argv: [
                "test",
                "--workspace",
                "--all-targets",
                "--locked",
                "--no-fail-fast",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            environment: environment.clone(),
            build_units: 0,
            tests_passed: 0,
            tests_failed: 0,
            tests_skipped: 0,
            exit_code: 0,
            output_digest: format!("blake3:{}", "0".repeat(64)),
        });
    }
    commands
}

fn current_subject(repository: &Path) -> Result<(String, String), CoordError> {
    let algorithm = git(repository, &["rev-parse", "--show-object-format"])?;
    if !matches!(algorithm.as_str(), "sha1" | "sha256") {
        return Err(invalid(
            "current repository uses an unsupported Git object format",
        ));
    }
    let commit = git(repository, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let tree = git(repository, &["rev-parse", "--verify", "HEAD^{tree}"])?;
    Ok((
        format!("{algorithm}:{commit}"),
        format!("{algorithm}:{tree}"),
    ))
}

fn tagged_regular_file_digest(
    repository: &Path,
    tag: &str,
    path: &str,
) -> Result<String, CoordError> {
    let entry = git(repository, &["ls-tree", tag, "--", path])?;
    let (metadata, actual_path) = entry
        .split_once('\t')
        .ok_or_else(|| invalid(format!("{tag}:{path} is absent or malformed")))?;
    if actual_path != path
        || !metadata.starts_with("100")
        || metadata.split_whitespace().nth(1) != Some("blob")
    {
        return Err(invalid(format!("{tag}:{path} is not one regular Git blob")));
    }
    let bytes = git_bytes(repository, &["show", &format!("{tag}:{path}")])?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn now_unix_ms() -> Result<u64, CoordError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid("system time precedes the Unix epoch"))?
        .as_millis();
    u64::try_from(millis).map_err(|_| invalid("system time exceeds the exact integer range"))
}

fn model_error(error: impl std::fmt::Display) -> CoordError {
    CoordError::new("INVALID_MSRV_RELEASE_SUBJECT", error.to_string())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_MSRV_RELEASE_EVIDENCE", reason)
}
