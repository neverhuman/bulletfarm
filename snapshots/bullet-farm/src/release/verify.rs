//! Exact byte and detached-signature verification for release bundles.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};

use super::{ReleaseFile, ReleaseManifest, SignedReleaseFile, signature};
use crate::{coord::CoordError, family_lock, process::InputFileOutput};

const MANIFEST: &str = "release-manifest.toml";
const MANIFEST_SIGNATURE: &str = "release-manifest.toml.sig";
const SIGNATURE_NAMESPACE: &str = "bullet-farm-release";
const REQUIRED_FAMILY_MEMBERS: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];

pub(super) struct VerificationReceipt {
    pub(super) manifest: ReleaseManifest,
}

pub(super) fn verify(
    bundle: &Path,
    allowed_signers: &Path,
) -> Result<VerificationReceipt, CoordError> {
    let bundle = admitted_directory(bundle, "release bundle")?;
    let mut allowed_signers =
        admitted_external_file(allowed_signers, "allowed signers", 64 * 1024)?;
    let allowed_signers = immutable_snapshot(&mut allowed_signers, "allowed signers", 64 * 1024)?;
    signature::admit_verifier()?;
    let manifest_path = bundle_file(&bundle, MANIFEST)?;
    let mut manifest_input = open_bounded_file(&manifest_path, "release manifest", 1024 * 1024)?;
    let manifest_bytes = read_open_bounded(&mut manifest_input, 1024 * 1024)?;
    let manifest = ReleaseManifest::parse(&manifest_bytes)?;
    let manifest_signature = bundle_file(&bundle, MANIFEST_SIGNATURE)?;
    let mut manifest_signature =
        open_bounded_file(&manifest_signature, "release manifest signature", 64 * 1024)?;
    let manifest_signature = immutable_snapshot(
        &mut manifest_signature,
        "release manifest signature",
        64 * 1024,
    )?;
    let streamed_manifest = verify_signature(
        &manifest,
        &manifest_signature.file,
        &allowed_signers.file,
        manifest_input.try_clone().map_err(CoordError::io)?,
    )?;
    if streamed_manifest.byte_count != manifest_bytes.len() as u64
        || streamed_manifest.digest != *blake3::hash(&manifest_bytes).as_bytes()
        || read_open_bounded(&mut manifest_input, 1024 * 1024)? != manifest_bytes
        || read_path_bounded(&manifest_path, 1024 * 1024)? != manifest_bytes
    {
        return Err(invalid_bundle(
            "release manifest changed during signature verification",
        ));
    }

    let lock_path = bundle_file(&bundle, &manifest.family_lock.path)?;
    let mut lock_input = open_bounded_file(&lock_path, "family.lock", manifest.family_lock.size)?;
    let lock_bytes = read_open_bounded(&mut lock_input, manifest.family_lock.size)?;
    verify_bytes(&manifest.family_lock, &lock_bytes)?;
    let lock = family_lock::parse(&lock_bytes)?;
    verify_release_signing_subject(&manifest, &lock, &allowed_signers)?;
    if lock.schema_version != manifest.family_lock_schema_version || lock.tag != manifest.tag {
        return Err(invalid_bundle(
            "included family.lock does not bind the manifest schema and tag",
        ));
    }
    let required_members = REQUIRED_FAMILY_MEMBERS.map(str::to_owned);
    lock.validate_required_members(&required_members)?;

    for package in &manifest.package {
        for signed in [
            &package.archive,
            &package.checksums,
            &package.cyclonedx_sbom,
            &package.spdx_sbom,
            &package.provenance,
        ] {
            verify_signed_file(&bundle, &manifest, signed, &allowed_signers.file)?;
        }
    }
    Ok(VerificationReceipt { manifest })
}

fn verify_release_signing_subject(
    manifest: &ReleaseManifest,
    lock: &family_lock::FamilyLock,
    allowed_signers: &ImmutableSnapshot,
) -> Result<(), CoordError> {
    if manifest.release_signing_identity != lock.hub.release_signing_identity {
        return Err(invalid_bundle(
            "release manifest signer does not match the locked Hub signing identity",
        ));
    }
    let allowed_signers_digest = format!("blake3:{}", allowed_signers.digest.to_hex());
    if allowed_signers_digest != lock.external.release_signing.allowed_signers_digest {
        return Err(invalid_bundle(
            "admitted allowed-signers bytes do not match the locked external subject",
        ));
    }

    // There is no trusted-time input at this component boundary. The lock parser proves that the
    // interval is structurally ordered, but not_before_unix_ms/not_after_unix_ms remain
    // UNADJUDICATED here and must not be reported as an enforced release property.
    Ok(())
}

fn verify_signed_file(
    bundle: &Path,
    manifest: &ReleaseManifest,
    signed: &SignedReleaseFile,
    allowed_signers: &File,
) -> Result<(), CoordError> {
    let mut payload = open_verified_file(bundle, &signed.file)?;
    let mut admitted_signature = open_verified_file(bundle, &signed.signature)?;
    let signature = immutable_snapshot(
        &mut admitted_signature,
        &signed.signature.path,
        signed.signature.size,
    )?;
    verify_snapshot_subject(&signature, &signed.signature)?;
    let streamed_payload = verify_signature(
        manifest,
        &signature.file,
        allowed_signers,
        payload.try_clone().map_err(CoordError::io)?,
    )?;
    if streamed_payload.byte_count != signed.file.size
        || format!(
            "blake3:{}",
            blake3::Hash::from_bytes(streamed_payload.digest).to_hex()
        ) != signed.file.digest
    {
        return Err(invalid_bundle(format!(
            "{} streamed signature subject differs from the signed manifest",
            signed.file.path
        )));
    }
    verify_open_file(&mut payload, &signed.file)?;
    verify_open_file(&mut admitted_signature, &signed.signature)?;
    verify_file(bundle, &signed.file)?;
    verify_file(bundle, &signed.signature)
}

fn verify_file(bundle: &Path, expected: &ReleaseFile) -> Result<(), CoordError> {
    let mut input = open_verified_file(bundle, expected)?;
    verify_open_file(&mut input, expected)
}

fn open_verified_file(bundle: &Path, expected: &ReleaseFile) -> Result<File, CoordError> {
    let path = bundle_file(bundle, &expected.path)?;
    let mut input = open_bounded_file(&path, &expected.path, expected.size)?;
    verify_open_file(&mut input, expected)?;
    Ok(input)
}

fn verify_open_file(input: &mut File, expected: &ReleaseFile) -> Result<(), CoordError> {
    let metadata = input.metadata().map_err(CoordError::io)?;
    if metadata.len() != expected.size {
        return Err(invalid_bundle(format!(
            "{} byte size differs from the signed manifest",
            expected.path
        )));
    }
    let digest = digest_open_file(input)?;
    if digest != expected.digest {
        return Err(invalid_bundle(format!(
            "{} digest differs from the signed manifest",
            expected.path
        )));
    }
    Ok(())
}

fn verify_bytes(expected: &ReleaseFile, bytes: &[u8]) -> Result<(), CoordError> {
    if u64::try_from(bytes.len()).ok() != Some(expected.size)
        || format!("blake3:{}", blake3::hash(bytes).to_hex()) != expected.digest
    {
        return Err(invalid_bundle(format!(
            "{} bytes differ from the signed manifest",
            expected.path
        )));
    }
    Ok(())
}

fn verify_snapshot_subject(
    snapshot: &ImmutableSnapshot,
    expected: &ReleaseFile,
) -> Result<(), CoordError> {
    if snapshot.byte_count != expected.size
        || format!("blake3:{}", snapshot.digest.to_hex()) != expected.digest
    {
        return Err(invalid_bundle(format!(
            "{} sealed signature subject differs from the signed manifest",
            expected.path
        )));
    }
    Ok(())
}

fn verify_signature(
    manifest: &ReleaseManifest,
    signature: &File,
    allowed_signers: &File,
    payload: File,
) -> Result<InputFileOutput, CoordError> {
    let (principal, fingerprint) = manifest.signer_parts();
    signature::verify(
        signature,
        allowed_signers,
        payload,
        principal,
        fingerprint,
        SIGNATURE_NAMESPACE,
        "release signature verification",
    )
}

fn admitted_directory(path: &Path, label: &str) -> Result<PathBuf, CoordError> {
    if !path.is_absolute() {
        return Err(invalid_bundle(format!("{label} path must be absolute")));
    }
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(invalid_bundle(format!(
            "{label} must be a regular non-symlink directory"
        )));
    }
    let canonical = path.canonicalize().map_err(CoordError::io)?;
    if canonical != path {
        return Err(invalid_bundle(format!(
            "{label} path must already be canonical"
        )));
    }
    Ok(canonical)
}

pub(super) fn admitted_external_file(
    path: &Path,
    label: &str,
    maximum: u64,
) -> Result<File, CoordError> {
    if !path.is_absolute() {
        return Err(invalid_bundle(format!("{label} path must be absolute")));
    }
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(invalid_bundle(format!(
            "{label} must be a bounded regular non-symlink file"
        )));
    }
    let canonical = path.canonicalize().map_err(CoordError::io)?;
    if canonical != path {
        return Err(invalid_bundle(format!(
            "{label} path must already be canonical"
        )));
    }
    open_bounded_file(&canonical, label, maximum)
}

fn bundle_file(bundle: &Path, relative: &str) -> Result<PathBuf, CoordError> {
    // Observed symlinks are rejected, but intermediate directories remain unpinned against races;
    // hostile-directory safety requires descriptor-relative traversal such as openat2.
    let mut cursor = bundle.to_path_buf();
    for component in relative.split('/') {
        cursor.push(component);
        let metadata = fs::symlink_metadata(&cursor).map_err(CoordError::io)?;
        if metadata.file_type().is_symlink() {
            return Err(invalid_bundle(format!(
                "release bundle path traverses a symlink: {relative}"
            )));
        }
    }
    Ok(cursor)
}

fn open_bounded_file(path: &Path, label: &str, maximum: u64) -> Result<File, CoordError> {
    #[cfg(unix)]
    let input = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(CoordError::io)?;
    #[cfg(not(unix))]
    let input = {
        let _ = path;
        return Err(CoordError::new(
            "RELEASE_VERIFICATION_PLATFORM_UNSUPPORTED",
            format!("{label} cannot be descriptor-pinned on this platform"),
        ));
    };
    let metadata = input.metadata().map_err(CoordError::io)?;
    if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > maximum {
        return Err(invalid_bundle(format!(
            "{label} must be a bounded regular file"
        )));
    }
    Ok(input)
}

fn read_path_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, CoordError> {
    let mut input = open_bounded_file(path, "release input", maximum)?;
    read_open_bounded(&mut input, maximum)
}

pub(super) fn read_open_bounded(input: &mut File, maximum: u64) -> Result<Vec<u8>, CoordError> {
    let size = usize::try_from(input.metadata().map_err(CoordError::io)?.len().min(maximum))
        .map_err(|_| invalid_bundle("release input is too large for this platform"))?;
    let read_limit = maximum
        .checked_add(1)
        .ok_or_else(|| invalid_bundle("release input byte limit cannot be represented"))?;
    input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut bytes = Vec::with_capacity(size);
    Read::by_ref(input)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    if u64::try_from(bytes.len())
        .ok()
        .is_none_or(|size| size > maximum)
    {
        return Err(invalid_bundle("release input exceeds its byte limit"));
    }
    Ok(bytes)
}

fn digest_open_file(input: &mut File) -> Result<String, CoordError> {
    input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer).map_err(CoordError::io)?;
        if count == 0 {
            input.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
            return Ok(format!("blake3:{}", hasher.finalize().to_hex()));
        }
        hasher.update(&buffer[..count]);
    }
}

pub(super) struct ImmutableSnapshot {
    pub(super) file: File,
    pub(super) byte_count: u64,
    pub(super) digest: blake3::Hash,
}

#[cfg(target_os = "linux")]
pub(super) fn immutable_snapshot(
    input: &mut File,
    label: &str,
    maximum: u64,
) -> Result<ImmutableSnapshot, CoordError> {
    use nix::{
        fcntl::{FcntlArg, SealFlag, fcntl},
        sys::memfd::{MemFdCreateFlag, memfd_create},
    };

    let bytes = read_open_bounded(input, maximum)?;
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| invalid_bundle(format!("{label} is too large for this platform")))?;
    let digest = blake3::hash(&bytes);
    let descriptor = memfd_create(
        c"bullet-release-input",
        MemFdCreateFlag::MFD_ALLOW_SEALING | MemFdCreateFlag::MFD_CLOEXEC,
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_INPUT_PIN_FAILED",
            format!("could not snapshot {label}: {error}"),
        )
    })?;
    let mut snapshot = File::from(descriptor);
    snapshot.write_all(&bytes).map_err(CoordError::io)?;
    snapshot.flush().map_err(CoordError::io)?;
    fcntl(
        snapshot.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|error| {
        CoordError::new(
            "RELEASE_INPUT_PIN_FAILED",
            format!("could not seal {label}: {error}"),
        )
    })?;
    snapshot.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(ImmutableSnapshot {
        file: snapshot,
        byte_count,
        digest,
    })
}

#[cfg(not(target_os = "linux"))]
pub(super) fn immutable_snapshot(
    _input: &mut File,
    label: &str,
    _maximum: u64,
) -> Result<ImmutableSnapshot, CoordError> {
    Err(CoordError::new(
        "RELEASE_VERIFICATION_PLATFORM_UNSUPPORTED",
        format!("{label} cannot be sealed on this platform"),
    ))
}

fn invalid_bundle(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_BUNDLE", reason)
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
