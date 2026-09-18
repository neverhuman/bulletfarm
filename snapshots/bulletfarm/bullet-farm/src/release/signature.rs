//! Shared bounded OpenSSH detached-signature verification.

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[cfg(unix)]
use std::os::{fd::AsRawFd, unix::fs::PermissionsExt};

use crate::{
    coord::CoordError,
    process::{InputFileOutput, Limits, run_bounded_with_input_file},
};

#[cfg(unix)]
const SSH_KEYGEN: &str = "/usr/bin/ssh-keygen";
#[cfg(windows)]
const SSH_KEYGEN: &str = r"C:\Windows\System32\OpenSSH\ssh-keygen.exe";
const SIGNATURE_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(10),
    stdout_bytes: 64 * 1024,
    stderr_bytes: 64 * 1024,
};

pub(super) fn admit_verifier() -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(SSH_KEYGEN).map_err(CoordError::io)?;
    let invalid = metadata.file_type().is_symlink() || !metadata.file_type().is_file();
    #[cfg(unix)]
    let invalid = invalid || metadata.permissions().mode() & 0o111 == 0;
    if invalid {
        return Err(CoordError::new(
            "RELEASE_VERIFIER_UNAVAILABLE",
            "the fixed OpenSSH verifier is unavailable",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    signature: &File,
    allowed_signers: &File,
    payload: File,
    principal: &str,
    fingerprint: &str,
    namespace: &str,
    label: &str,
) -> Result<InputFileOutput, CoordError> {
    let pinned_signature = PinnedPath::new(signature, "detached signature")?;
    let pinned_signers = PinnedPath::new(allowed_signers, "allowed signers")?;
    let output = run_bounded_with_input_file(
        Command::new(SSH_KEYGEN)
            .args(["-Y", "verify", "-f"])
            .arg(pinned_signers.path())
            .args(["-I", principal, "-n", namespace, "-s"])
            .arg(pinned_signature.path())
            .env_clear()
            .env("HOME", "/")
            .env("LC_ALL", "C")
            .env("PATH", "/usr/bin"),
        label,
        SIGNATURE_LIMITS,
        payload,
    )?;
    if !output.output.status.success() {
        return Err(CoordError::new(
            "RELEASE_SIGNATURE_INVALID",
            "detached release signature did not verify",
        ));
    }
    let mut status = output.output.stdout.clone();
    status.extend_from_slice(&output.output.stderr);
    let status = std::str::from_utf8(&status).map_err(|_| {
        CoordError::new(
            "INVALID_SIGNATURE_OUTPUT",
            "ssh-keygen returned non-UTF-8 signature status",
        )
    })?;
    let expected =
        format!("Good \"{namespace}\" signature for {principal} with ED25519 key {fingerprint}");
    if status.lines().filter(|line| *line == expected).count() != 1 {
        return Err(CoordError::new(
            "RELEASE_SIGNER_IDENTITY_MISMATCH",
            "signature status did not bind the exact admitted Ed25519 signer",
        ));
    }
    Ok(output)
}

struct PinnedPath {
    path: PathBuf,
    #[cfg(unix)]
    descriptor: std::os::fd::RawFd,
}

impl PinnedPath {
    #[cfg(unix)]
    fn new(file: &File, label: &str) -> Result<Self, CoordError> {
        let descriptor = nix::unistd::dup(file.as_raw_fd()).map_err(|error| {
            CoordError::new(
                "RELEASE_INPUT_PIN_FAILED",
                format!("could not pin {label}: {error}"),
            )
        })?;
        #[cfg(target_os = "linux")]
        let path = PathBuf::from(format!("/proc/self/fd/{descriptor}"));
        #[cfg(not(target_os = "linux"))]
        let path = PathBuf::from(format!("/dev/fd/{descriptor}"));
        Ok(Self { path, descriptor })
    }

    #[cfg(not(unix))]
    fn new(_file: &File, label: &str) -> Result<Self, CoordError> {
        Err(CoordError::new(
            "RELEASE_VERIFICATION_PLATFORM_UNSUPPORTED",
            format!("{label} cannot be descriptor-pinned on this platform"),
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PinnedPath {
    fn drop(&mut self) {
        #[cfg(unix)]
        let _ = nix::unistd::close(self.descriptor);
    }
}
