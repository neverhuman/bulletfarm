//! Descriptor-pinned Git execution for family-lock verification.

mod subject;

#[cfg(test)]
mod tests;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
};

use self::subject::{PinnedAllowedSigners, PinnedExecutable, PinnedRepository};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const GIT_BIN: &str = "/usr/bin/git";
const SSH_KEYGEN_BIN: &str = "/usr/bin/ssh-keygen";

static GIT: OnceLock<GitProgram> = OnceLock::new();
static SSH_KEYGEN: OnceLock<PinnedExecutable> = OnceLock::new();

pub(super) fn run(
    repo: &Path,
    args: &[&str],
    allowed_signers: Option<&Path>,
    limits: Limits,
) -> Result<Output, CoordError> {
    let signature = match allowed_signers {
        Some(allowed_signers) => Some(SignatureInputs {
            helper: admitted_ssh_keygen()?,
            allowed_signers,
        }),
        None => None,
    };
    admitted_git()?.run(repo, args, signature, limits)
}

pub(super) fn run_labeled_after_verify(
    repo: &Path,
    args: &[&str],
    limits: Limits,
    label: &str,
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<Output, CoordError> {
    admitted_git()?.run_labeled_after_verify(repo, args, None, limits, label, after_verify)
}

#[derive(Debug)]
struct GitProgram {
    executable: PinnedExecutable,
}

#[derive(Clone, Copy)]
struct SignatureInputs<'a> {
    helper: &'a PinnedExecutable,
    allowed_signers: &'a Path,
}

impl GitProgram {
    fn admit(path: &Path) -> Result<Self, CoordError> {
        let executable = PinnedExecutable::admit("Git family-lock verification", path)?;
        let program = Self { executable };
        program.probe_identity()?;
        Ok(program)
    }

    fn probe_identity(&self) -> Result<(), CoordError> {
        self.executable.verify()?;
        let output = run_bounded(
            Command::new(self.executable.execution_path())
                .arg("--version")
                .env_clear()
                .env("LC_ALL", "C")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_TERMINAL_PROMPT", "0"),
            "Git family-lock identity probe",
            super::GIT_LIMITS,
        );
        self.executable.verify()?;
        let output = output?;
        let version = std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|text| text.lines().next())
            .unwrap_or_default();
        if !output.status.success() || !valid_git_version(version) {
            return Err(CoordError::new(
                "GIT_IDENTITY_MISMATCH",
                "bounded --version probe did not identify Git",
            ));
        }
        Ok(())
    }

    fn run(
        &self,
        repo: &Path,
        args: &[&str],
        signature: Option<SignatureInputs<'_>>,
        limits: Limits,
    ) -> Result<Output, CoordError> {
        self.run_after_verify(repo, args, signature, limits, || Ok(()))
    }

    fn run_after_verify(
        &self,
        repo: &Path,
        args: &[&str],
        signature: Option<SignatureInputs<'_>>,
        limits: Limits,
        after_verify: impl FnOnce() -> Result<(), CoordError>,
    ) -> Result<Output, CoordError> {
        self.run_labeled_after_verify(
            repo,
            args,
            signature,
            limits,
            "Git family-lock verification",
            after_verify,
        )
    }

    fn run_labeled_after_verify(
        &self,
        repo: &Path,
        args: &[&str],
        signature: Option<SignatureInputs<'_>>,
        limits: Limits,
        label: &str,
        after_verify: impl FnOnce() -> Result<(), CoordError>,
    ) -> Result<Output, CoordError> {
        let repository = PinnedRepository::admit(repo)?;
        let allowed_signers = signature
            .map(|signature| signature.allowed_signers)
            .map(PinnedAllowedSigners::admit)
            .transpose()?;
        self.executable.verify()?;
        repository.verify()?;
        if let Some(signature) = signature {
            signature.helper.verify()?;
        }
        if let Some(allowed_signers) = &allowed_signers {
            allowed_signers.verify()?;
        }
        after_verify()?;

        let work_tree_path = repository.work_tree_path();
        let mut command = Command::new(self.executable.execution_path());
        command
            .current_dir(&work_tree_path)
            .arg(format!("--git-dir={}", repository.git_dir_path().display()))
            .arg(format!("--work-tree={}", work_tree_path.display()));
        if let Some(signature) = signature {
            command.arg("-c").arg(format!(
                "gpg.ssh.program={}",
                signature.helper.execution_path().display()
            ));
        }
        if let Some(allowed_signers) = &allowed_signers {
            command.arg("-c").arg(format!(
                "gpg.ssh.allowedSignersFile={}",
                allowed_signers.subject_path().display()
            ));
        }
        let output = run_bounded(
            command
                .args(args)
                .env_clear()
                .env("LC_ALL", "C")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_NO_REPLACE_OBJECTS", "1")
                .env("GIT_OPTIONAL_LOCKS", "0")
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_NO_LAZY_FETCH", "1"),
            label,
            limits,
        );

        repository.verify()?;
        self.executable.verify()?;
        if let Some(signature) = signature {
            signature.helper.verify()?;
        }
        if let Some(allowed_signers) = &allowed_signers {
            allowed_signers.verify()?;
        }
        output
    }
}

fn admitted_git() -> Result<&'static GitProgram, CoordError> {
    if let Some(git) = GIT.get() {
        return Ok(git);
    }
    let canonical = canonical_tool(GIT_BIN, "Git family-lock verification")?;
    let candidate = GitProgram::admit(&canonical)?;
    let _ = GIT.set(candidate);
    Ok(GIT
        .get()
        .expect("Git is initialized after successful admission"))
}

fn admitted_ssh_keygen() -> Result<&'static PinnedExecutable, CoordError> {
    if let Some(helper) = SSH_KEYGEN.get() {
        return Ok(helper);
    }
    let canonical = canonical_tool(SSH_KEYGEN_BIN, "SSH signature verifier")?;
    let candidate = PinnedExecutable::admit("SSH signature verifier", &canonical)?;
    let _ = SSH_KEYGEN.set(candidate);
    Ok(SSH_KEYGEN
        .get()
        .expect("SSH helper is initialized after successful admission"))
}

fn canonical_tool(path: &str, label: &str) -> Result<PathBuf, CoordError> {
    fs::canonicalize(path).map_err(|error| {
        CoordError::new(
            "GIT_TOOL_UNAVAILABLE",
            format!("{label}: {path} cannot be resolved: {error}"),
        )
    })
}

fn valid_git_version(value: &str) -> bool {
    value.strip_prefix("git version ").is_some_and(|version| {
        !version.is_empty()
            && version.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_')
            })
            && version
                .split('.')
                .take(2)
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    })
}
