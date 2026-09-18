//! OS-backed admission for release evidence. Repository bytes never choose trust roots.

use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{ErrorKind, Read},
    path::{Component, Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::{
    fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    io::AsRawFd,
};

use super::schema::{
    AdmissionDescriptor, MAX_INPUT_BYTES, MsrvPolicy, RECEIPT_NAMESPACE, TIME_NAMESPACE, digest,
};
use crate::coord::CoordError;

pub(super) const ADMISSION_PATH: &str = "/etc/bullet-farm/release-msrv-1-95-admission.toml";
const POLICY_DIGEST_DOMAIN: &[u8] = b"bullet-farm.release-msrv-policy.v1\0";
const TOOL_MAX_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct AdmittedPolicy {
    pub policy: MsrvPolicy,
    pub policy_digest: String,
}

pub(super) struct OperatorFile {
    pub(super) bytes: Vec<u8>,
    #[cfg(unix)]
    pub(super) device: u64,
    #[cfg(unix)]
    pub(super) inode: u64,
}

pub(super) fn load(hub: &Path) -> Result<Option<AdmittedPolicy>, CoordError> {
    let descriptor_path = Path::new(ADMISSION_PATH);
    match fs::symlink_metadata(descriptor_path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CoordError::io(error)),
        Ok(_) => {}
    }
    let descriptor = operator_file(
        descriptor_path,
        "MSRV admission descriptor",
        MAX_INPUT_BYTES,
        false,
    )?;
    let descriptor = AdmissionDescriptor::parse(&descriptor.bytes)?;
    reject_inside_family(&descriptor.policy_path, hub, "MSRV policy")?;
    let policy_input = operator_file(
        &descriptor.policy_path,
        "MSRV evidence policy",
        MAX_INPUT_BYTES,
        false,
    )?;
    let policy = MsrvPolicy::parse(&policy_input.bytes)?;
    let family = hub
        .parent()
        .ok_or_else(|| invalid("Hub has no family root"))?;

    let evidence = canonical_directory(&policy.evidence_directory, "MSRV evidence directory")?;
    if evidence.starts_with(family) {
        return Err(invalid(
            "MSRV evidence directory must be outside the source family",
        ));
    }
    let source_root = admitted_root(
        &policy.source_allowed_signers_path,
        hub,
        &evidence,
        "source-tag trust root",
    )?;
    let attestor_root = admitted_root(
        &policy.attestor_allowed_signers_path,
        hub,
        &evidence,
        "attestor trust root",
    )?;
    let time_root = admitted_root(
        &policy.trusted_time_allowed_signers_path,
        hub,
        &evidence,
        "trusted-time trust root",
    )?;
    reject_same_inputs(&[&source_root, &attestor_root, &time_root])?;
    validate_key_roles(
        &source_root.bytes,
        &attestor_root.bytes,
        &time_root.bytes,
        &policy,
    )?;

    for (tool, expected, label) in [
        (&policy.rustc.path, &policy.rustc.digest, "admitted rustc"),
        (&policy.cargo.path, &policy.cargo.digest, "admitted cargo"),
    ] {
        reject_inside_family(tool, hub, label)?;
        let input = operator_file(tool, label, TOOL_MAX_BYTES, true)?;
        let actual = format!("blake3:{}", blake3::hash(&input.bytes).to_hex());
        if &actual != expected {
            return Err(invalid(format!(
                "{label} digest differs from the operator policy"
            )));
        }
    }
    Ok(Some(AdmittedPolicy {
        policy,
        policy_digest: digest(POLICY_DIGEST_DOMAIN, &policy_input.bytes),
    }))
}

fn admitted_root(
    path: &Path,
    hub: &Path,
    evidence: &Path,
    label: &str,
) -> Result<OperatorFile, CoordError> {
    reject_inside_family(path, hub, label)?;
    if path.starts_with(evidence) {
        return Err(invalid(format!(
            "{label} must be outside the evidence directory"
        )));
    }
    operator_file(path, label, MAX_INPUT_BYTES, false)
}

fn operator_file(
    path: &Path,
    label: &str,
    maximum: u64,
    executable: bool,
) -> Result<OperatorFile, CoordError> {
    validate_absolute_path(path, label)?;
    secure_ancestors(path, label)?;
    #[cfg(unix)]
    let mut input = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK | nix::libc::O_CLOEXEC)
        .open(path)
        .map_err(CoordError::io)?;
    #[cfg(not(unix))]
    {
        let _ = (path, label, maximum, executable);
        return Err(CoordError::new(
            "MSRV_RELEASE_ADMISSION_UNSUPPORTED",
            "operator-owned MSRV evidence admission is implemented only on Unix",
        ));
    }
    #[cfg(unix)]
    {
        let metadata = input.metadata().map_err(CoordError::io)?;
        if !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > maximum
            || metadata.uid() != 0
            || metadata.permissions().mode() & 0o022 != 0
            || metadata.nlink() != 1
        {
            return Err(invalid(format!(
                "{label} must be bounded, root-owned, single-link, and not group/other writable"
            )));
        }
        if executable && metadata.permissions().mode() & 0o111 == 0 {
            return Err(invalid(format!("{label} must be executable")));
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        input
            .by_ref()
            .take(maximum.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(CoordError::io)?;
        let after = input.metadata().map_err(CoordError::io)?;
        let stat = nix::sys::stat::fstat(input.as_raw_fd())
            .map_err(|error| CoordError::new("MSRV_ADMISSION_FSTAT_FAILED", error.to_string()))?;
        if bytes.len() as u64 != metadata.len()
            || bytes.len() as u64 > maximum
            || after.dev() != metadata.dev()
            || after.ino() != metadata.ino()
            || after.len() != metadata.len()
            || after.nlink() != 1
            || stat.st_nlink != 1
        {
            return Err(invalid(format!("{label} changed during descriptor read")));
        }
        Ok(OperatorFile {
            bytes,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
}

pub(super) fn secure_ancestors(path: &Path, label: &str) -> Result<(), CoordError> {
    #[cfg(unix)]
    {
        let parent = path
            .parent()
            .ok_or_else(|| invalid(format!("{label} has no parent directory")))?;
        let mut cursor = PathBuf::from("/");
        validate_secure_directory(&cursor, label)?;
        for component in parent.components() {
            match component {
                Component::RootDir => continue,
                Component::Normal(part) => cursor.push(part),
                _ => return Err(invalid(format!("{label} path is not normalized"))),
            }
            validate_secure_directory(&cursor, label)?;
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (path, label);
        Err(CoordError::new(
            "MSRV_RELEASE_ADMISSION_UNSUPPORTED",
            "secure ancestor admission is implemented only on Unix",
        ))
    }
}

#[cfg(unix)]
fn validate_secure_directory(path: &Path, label: &str) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_dir()
        || metadata.uid() != 0
        || metadata.permissions().mode() & 0o022 != 0
    {
        return Err(invalid(format!(
            "{label} has a mutable, linked, or non-root-owned ancestor: {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_absolute_path(path: &Path, label: &str) -> Result<(), CoordError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid(format!(
            "{label} path must be absolute and lexically normalized"
        )));
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, CoordError> {
    validate_absolute_path(path, label)?;
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(invalid(format!("{label} must be a non-symlink directory")));
    }
    let canonical = path.canonicalize().map_err(CoordError::io)?;
    if canonical != path {
        return Err(invalid(format!("{label} path must already be canonical")));
    }
    Ok(canonical)
}

pub(super) fn reject_same_inputs(inputs: &[&OperatorFile]) -> Result<(), CoordError> {
    #[cfg(unix)]
    for (index, left) in inputs.iter().enumerate() {
        if inputs[index + 1..]
            .iter()
            .any(|right| left.device == right.device && left.inode == right.inode)
        {
            return Err(invalid(
                "operator trust roots must not be hardlinks to one file",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_key_roles(
    source_bytes: &[u8],
    attestor_bytes: &[u8],
    time_bytes: &[u8],
    policy: &MsrvPolicy,
) -> Result<(), CoordError> {
    let source = parse_source_keys(source_bytes)?;
    let attestor = parse_role_key(attestor_bytes, &policy.attestor_identity, RECEIPT_NAMESPACE)?;
    let time = parse_role_key(time_bytes, &policy.trusted_time_identity, TIME_NAMESPACE)?;
    if attestor == time || source.contains(&attestor) || source.contains(&time) {
        return Err(invalid(
            "source-tag, attestor, and trusted-time roles must use distinct Ed25519 keys",
        ));
    }
    Ok(())
}

fn parse_source_keys(bytes: &[u8]) -> Result<BTreeSet<String>, CoordError> {
    let text = admitted_utf8(bytes, "source-tag trust root")?;
    let mut keys = BTreeSet::new();
    for line in text.lines() {
        if line.is_empty() {
            return Err(invalid("source-tag trust root contains an empty line"));
        }
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        let (principal, algorithm, blob) = match fields.as_slice() {
            [principal, algorithm, blob] => (*principal, *algorithm, *blob),
            [principal, namespace, algorithm, blob] if *namespace == "namespaces=\"git\"" => {
                (*principal, *algorithm, *blob)
            }
            _ => return Err(invalid("source-tag trust root has an unsupported entry")),
        };
        validate_principal(principal)?;
        validate_key(algorithm, blob)?;
        if !keys.insert(blob.to_owned()) {
            return Err(invalid("source-tag trust root repeats one Ed25519 key"));
        }
    }
    if keys.is_empty() || keys.len() > 64 {
        return Err(invalid(
            "source-tag trust root requires 1..=64 Ed25519 keys",
        ));
    }
    Ok(keys)
}

fn parse_role_key(bytes: &[u8], identity: &str, namespace: &str) -> Result<String, CoordError> {
    let text = admitted_utf8(bytes, "role trust root")?;
    let mut lines = text.lines();
    let line = lines
        .next()
        .ok_or_else(|| invalid("role trust root is empty"))?;
    if lines.next().is_some() {
        return Err(invalid("role trust root must contain exactly one entry"));
    }
    let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
    let [principal, admitted_namespace, algorithm, blob] = fields.as_slice() else {
        return Err(invalid("role trust root has an unsupported entry"));
    };
    let expected_principal = identity
        .split_once("|ed25519|")
        .map(|parts| parts.0)
        .ok_or_else(|| invalid("role signing identity is malformed"))?;
    if *principal != expected_principal
        || *admitted_namespace != format!("namespaces=\"{namespace}\"")
    {
        return Err(invalid(
            "role trust root principal or namespace differs from policy",
        ));
    }
    validate_principal(principal)?;
    validate_key(algorithm, blob)?;
    Ok((*blob).to_owned())
}

fn admitted_utf8<'a>(bytes: &'a [u8], label: &str) -> Result<&'a str, CoordError> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid(format!("{label} is not UTF-8")))?;
    if text.is_empty()
        || !text.ends_with('\n')
        || text
            .bytes()
            .any(|byte| byte == 0 || (byte < 0x20 && byte != b'\n'))
    {
        return Err(invalid(format!("{label} is not canonical line text")));
    }
    Ok(text.strip_suffix('\n').expect("checked line ending"))
}

fn validate_principal(value: &str) -> Result<(), CoordError> {
    if value.is_empty()
        || value.len() > 128
        || value.bytes().any(|byte| {
            !byte.is_ascii_graphic() || matches!(byte, b',' | b'*' | b'?' | b'!' | b'|')
        })
    {
        return Err(invalid("allowed-signers principal is malformed"));
    }
    Ok(())
}

fn validate_key(algorithm: &str, blob: &str) -> Result<(), CoordError> {
    if algorithm != "ssh-ed25519"
        || !(40..=256).contains(&blob.len())
        || blob
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')))
    {
        return Err(invalid(
            "allowed-signers key must be one bounded Ed25519 key",
        ));
    }
    Ok(())
}

fn reject_inside_family(path: &Path, hub: &Path, label: &str) -> Result<(), CoordError> {
    let family = hub
        .parent()
        .ok_or_else(|| invalid("Hub has no family root"))?;
    if path.starts_with(family) {
        return Err(invalid(format!(
            "{label} cannot be selected from repository bytes"
        )));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_MSRV_RELEASE_ADMISSION", reason)
}
