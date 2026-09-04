#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "COMPONENT_ONLY Wave-0 observer awaits independently reviewed claim truth"
    )
)]

#[cfg(test)]
mod tests;

use std::{
    fs::{File, Metadata},
    os::unix::{
        ffi::OsStrExt,
        fs::{FileExt, MetadataExt},
    },
    path::Path,
    time::Duration,
};

use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
use sha2::{Digest, Sha256};

use super::{
    WAVE0_REPOSITORIES,
    repository::{self, FamilyRepository, Wave0FamilyGuard},
};
use crate::{
    checkout::{admit_repository_metadata, verify_exact_worktree},
    coord::{
        CoordError,
        model::{Wave0CleanStateV1, Wave0MemberRoleV1, Wave0MemberV1},
    },
    process::Limits,
};

const fn observation_limits(stdout_bytes: usize) -> Limits {
    Limits {
        timeout: Duration::from_secs(30),
        stdout_bytes,
        stderr_bytes: 4 * 1024,
    }
}
const SUBJECT_LIMITS: Limits = observation_limits(256);
const STATUS_LIMITS: Limits = observation_limits(64 * 1024);
const MAX_LEDGER_BYTES: u64 = 64 * 1024 * 1024;
const ROLES: [Wave0MemberRoleV1; 4] = [
    Wave0MemberRoleV1::Hub,
    Wave0MemberRoleV1::Kernel,
    Wave0MemberRoleV1::BulletGit,
    Wave0MemberRoleV1::Portal,
];
const SAFE_GIT_CONFIG: [&str; 16] = [
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.attributesFile=/dev/null",
    "-c",
    "core.excludesFile=/dev/null",
    "-c",
    "core.filemode=true",
    "-c",
    "core.symlinks=true",
    "-c",
    "core.ignorecase=false",
    "-c",
    "core.autocrlf=false",
    "-c",
    "core.untrackedCache=false",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::coord) struct Wave0MechanicalObservation {
    pub(in crate::coord) members: [Wave0MemberV1; 4],
    pub(in crate::coord) collaboration_log_path_hex: String,
    pub(in crate::coord) collaboration_log_sha256: String,
    pub(in crate::coord) collaboration_log_byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemberPass {
    member: Wave0MemberV1,
    repository: FamilyRepository,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservationSeam {
    AfterSubject,
    AfterStatus,
}

pub(in crate::coord) fn observe_wave0_mechanical(
    family_root: &Path,
) -> Result<Wave0MechanicalObservation, CoordError> {
    observe_wave0_mechanical_with(family_root, |_, _| {})
}

fn observe_wave0_mechanical_with(
    family_root: &Path,
    mut at_seam: impl FnMut(&str, ObservationSeam),
) -> Result<Wave0MechanicalObservation, CoordError> {
    let family = Wave0FamilyGuard::open(family_root)?;
    let ledger = LedgerPrefix::open(&family, family_root)?;
    family.revalidate()?;
    let first = observe_family(&family, family_root, &mut at_seam)?;
    ledger.revalidate(&family)?;
    family.revalidate()?;
    let second = observe_family(&family, family_root, &mut at_seam)?;
    ledger.revalidate(&family)?;
    family.revalidate()?;
    if first != second {
        return Err(changed(
            "family subjects or clean state changed across independent observations",
        ));
    }
    let members = first
        .into_iter()
        .map(|pass| pass.member)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| changed("Wave-0 observation did not contain four members"))?;
    Ok(Wave0MechanicalObservation {
        members,
        collaboration_log_path_hex: ledger.path_hex,
        collaboration_log_sha256: ledger.sha256,
        collaboration_log_byte_length: ledger.byte_length,
    })
}

fn observe_family(
    family: &Wave0FamilyGuard,
    family_root: &Path,
    at_seam: &mut impl FnMut(&str, ObservationSeam),
) -> Result<Vec<MemberPass>, CoordError> {
    WAVE0_REPOSITORIES
        .iter()
        .zip(ROLES)
        .map(|((name, identity), role)| {
            observe_member(family, family_root, name, identity, role, at_seam)
        })
        .collect()
}

fn observe_member(
    family: &Wave0FamilyGuard,
    family_root: &Path,
    name: &str,
    identity: &str,
    role: Wave0MemberRoleV1,
    at_seam: &mut impl FnMut(&str, ObservationSeam),
) -> Result<MemberPass, CoordError> {
    family.revalidate()?;
    let selected = repository::select(family_root, name)?;
    let repo = family_root.join(name);
    verify_clean(&repo)?;
    verify_no_replace_refs(&repo, || {
        revalidate_member(family, family_root, name, &selected)
    })?;
    let subject = run_git(
        &repo,
        &[
            "rev-parse",
            "--show-object-format",
            "HEAD^{commit}",
            "HEAD^{tree}",
        ],
        SUBJECT_LIMITS,
        || revalidate_member(family, family_root, name, &selected),
    )?;
    let subject_tuple = parse_subject(&subject)?;
    at_seam(name, ObservationSeam::AfterSubject);
    let status = run_git(
        &repo,
        &[
            "status",
            "--porcelain=v2",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ],
        STATUS_LIMITS,
        || revalidate_member(family, family_root, name, &selected),
    )?;
    at_seam(name, ObservationSeam::AfterStatus);
    if !status.is_empty() {
        return Err(CoordError::new(
            "DIRTY_CHECKOUT",
            format!("Wave-0 {name} has staged, unstaged, or untracked state"),
        ));
    }
    verify_no_replace_refs(&repo, || {
        revalidate_member(family, family_root, name, &selected)
    })?;
    let repeated = run_git(
        &repo,
        &[
            "rev-parse",
            "--show-object-format",
            "HEAD^{commit}",
            "HEAD^{tree}",
        ],
        SUBJECT_LIMITS,
        || revalidate_member(family, family_root, name, &selected),
    )?;
    if parse_subject(&repeated)? != subject_tuple {
        return Err(changed(format!(
            "Wave-0 {name} subject changed across status observation"
        )));
    }
    revalidate_member(family, family_root, name, &selected)?;
    let (commit_oid, tree_oid) = subject_tuple;
    Ok(MemberPass {
        member: Wave0MemberV1 {
            role,
            repository_identity: identity.to_owned(),
            commit_oid,
            tree_oid,
            index_state: Wave0CleanStateV1::Clean,
            worktree_state: Wave0CleanStateV1::Clean,
            untracked_state: Wave0CleanStateV1::Clean,
        },
        repository: selected,
    })
}

fn verify_clean(repo: &Path) -> Result<(), CoordError> {
    admit_repository_metadata(repo, None)?;
    verify_exact_worktree(repo)
}

fn verify_no_replace_refs(
    repo: &Path,
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<(), CoordError> {
    let refs = run_git(
        repo,
        &[
            "for-each-ref",
            "--count=1",
            "--format=%(refname)",
            "refs/replace",
        ],
        STATUS_LIMITS,
        after_verify,
    )?;
    if refs.is_empty() {
        Ok(())
    } else {
        Err(CoordError::new(
            "UNSAFE_GIT_METADATA",
            "loose or packed Git replacement refs are not admitted for Wave-0",
        ))
    }
}

fn revalidate_member(
    family: &Wave0FamilyGuard,
    family_root: &Path,
    name: &str,
    expected: &FamilyRepository,
) -> Result<(), CoordError> {
    family.revalidate()?;
    if &repository::select(family_root, name)? != expected {
        return Err(changed(format!(
            "Wave-0 {name} repository identity changed"
        )));
    }
    verify_clean(&family_root.join(name))
}

fn run_git(
    repo: &Path,
    args: &[&str],
    limits: Limits,
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<Vec<u8>, CoordError> {
    let mut admitted_args = Vec::with_capacity(SAFE_GIT_CONFIG.len() + args.len());
    admitted_args.extend_from_slice(&SAFE_GIT_CONFIG);
    admitted_args.extend_from_slice(args);
    let output = crate::family_lock::run_admitted_git_after_verify(
        repo,
        &admitted_args,
        limits,
        "Wave-0 mechanical Git observation",
        after_verify,
    )?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(CoordError::new(
            "GIT_VERIFICATION_FAILED",
            "Wave-0 Git observation did not return one quiet successful result",
        ));
    }
    Ok(output.stdout)
}

fn parse_subject(bytes: &[u8]) -> Result<(String, String), CoordError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid_output("Git subject output is not UTF-8"))?;
    let mut lines = text.split_terminator('\n');
    let algorithm = lines.next().unwrap_or_default();
    let commit = lines.next().unwrap_or_default();
    let tree = lines.next().unwrap_or_default();
    if algorithm != "sha1" || lines.next().is_some() || !text.ends_with('\n') {
        return Err(CoordError::new(
            "UNSUPPORTED_GIT_OBJECT_FORMAT",
            "Wave-0 admits exactly one SHA-1 Git subject tuple",
        ));
    }
    if !valid_sha1(commit) || !valid_sha1(tree) {
        return Err(invalid_output("Git returned a malformed SHA-1 subject"));
    }
    Ok((format!("sha1:{commit}"), format!("sha1:{tree}")))
}

fn valid_sha1(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn head_reference(bytes: &[u8]) -> Result<Option<&str>, CoordError> {
    let text = std::str::from_utf8(bytes).map_err(|_| changed("Wave-0 HEAD is not UTF-8"))?;
    let value = text
        .strip_suffix('\n')
        .ok_or_else(|| changed("Wave-0 HEAD lacks its terminal newline"))?;
    if let Some(reference) = value.strip_prefix("ref: ") {
        if reference.len() > 512
            || !reference.starts_with("refs/heads/")
            || !reference.is_ascii()
            || reference.contains('\\')
            || reference.bytes().any(|byte| byte.is_ascii_control())
            || reference
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(changed("Wave-0 HEAD contains an unsafe symbolic ref"));
        }
        Ok(Some(reference))
    } else if valid_sha1(value) {
        Ok(None)
    } else {
        Err(changed("Wave-0 detached HEAD is not an exact SHA-1 OID"))
    }
}

struct LedgerPrefix {
    file: File,
    identity: LedgerIdentity,
    bytes: Vec<u8>,
    path_hex: String,
    sha256: String,
    byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LedgerIdentity {
    device: u64,
    inode: u64,
    mode: u32,
    owner_uid: u32,
    owner_gid: u32,
    links: u64,
}

impl LedgerPrefix {
    fn open(family: &Wave0FamilyGuard, family_root: &Path) -> Result<Self, CoordError> {
        let file = open_ledger(family)?;
        let metadata = file.metadata().map_err(CoordError::io)?;
        validate_ledger_metadata(&metadata, 1)?;
        let byte_length = metadata.len();
        let first = read_prefix(&file, byte_length)?;
        let second = read_prefix(&file, byte_length)?;
        let final_metadata = file.metadata().map_err(CoordError::io)?;
        if first != second || metadata_tuple(&metadata) != metadata_tuple(&final_metadata) {
            return Err(changed("collaboration log changed during prefix admission"));
        }
        if first.last() != Some(&b'\n') {
            return Err(changed(
                "collaboration log prefix lacks its terminal newline",
            ));
        }
        let path = family_root.join("AGENT_CHAT.md");
        let path_bytes = path.as_os_str().as_bytes();
        if path_bytes.len() > 4096 {
            return Err(changed("collaboration log path exceeds 4,096 bytes"));
        }
        Ok(Self {
            identity: LedgerIdentity::from(&metadata),
            sha256: format!("sha256:{:x}", Sha256::digest(&first)),
            path_hex: path_bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            byte_length,
            bytes: first,
            file,
        })
    }

    fn revalidate(&self, family: &Wave0FamilyGuard) -> Result<(), CoordError> {
        let reopened = open_ledger(family)?;
        for file in [&self.file, &reopened] {
            let metadata = file.metadata().map_err(CoordError::io)?;
            validate_ledger_metadata(&metadata, self.byte_length)?;
            if LedgerIdentity::from(&metadata) != self.identity
                || read_prefix(file, self.byte_length)? != self.bytes
            {
                return Err(changed(
                    "collaboration log prefix or pathname identity changed",
                ));
            }
        }
        Ok(())
    }
}

impl From<&Metadata> for LedgerIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            owner_uid: metadata.uid(),
            owner_gid: metadata.gid(),
            links: metadata.nlink(),
        }
    }
}

fn open_ledger(family: &Wave0FamilyGuard) -> Result<File, CoordError> {
    openat2(
        &family.root,
        Path::new("AGENT_CHAT.md"),
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        ResolveFlags::BENEATH
            | ResolveFlags::NO_SYMLINKS
            | ResolveFlags::NO_MAGICLINKS
            | ResolveFlags::NO_XDEV,
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain collaboration log: {error}")))
}

fn validate_ledger_metadata(metadata: &Metadata, minimum: u64) -> Result<(), CoordError> {
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.len() < minimum
        || metadata.len() > MAX_LEDGER_BYTES
    {
        return Err(changed(
            "collaboration log is not a bounded single-link regular file",
        ));
    }
    Ok(())
}

fn metadata_tuple(metadata: &Metadata) -> (u64, u64, u64, i64, i64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

fn read_prefix(file: &File, length: u64) -> Result<Vec<u8>, CoordError> {
    let length = usize::try_from(length)
        .map_err(|_| changed("collaboration log prefix does not fit this host"))?;
    let mut bytes = vec![0_u8; length];
    let mut offset = 0;
    while offset < bytes.len() {
        let read = file
            .read_at(&mut bytes[offset..], offset as u64)
            .map_err(CoordError::io)?;
        if read == 0 {
            return Err(changed("collaboration log truncated during prefix read"));
        }
        offset += read;
    }
    Ok(bytes)
}

fn invalid_output(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_GIT_OUTPUT", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("WAVE0_OBSERVATION_CHANGED", reason)
}
