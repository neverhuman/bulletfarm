//! Fail-closed installation of the exact family described by `family.lock`.

mod args;
mod command;
mod transaction;
mod validate;

use std::{ffi::OsStr, path::Path};

use crate::{
    coord::CoordError,
    family_lock::{self, LockedMember},
};
use args::parse_args;
use command::{SetupEnvironment, Toolchain, run_git};
use transaction::{AdmittedRoot, NoFault, install_transaction};
use validate::SetupValidator;

#[cfg(test)]
use transaction::ExactOnlyValidator;
#[cfg(test)]
use transaction::{admitted_root_for_test, publish_staged_for_test};

const GIT_BIN: &str = "/usr/bin/git";
const BASH_BIN: &str = "/bin/bash";
const STAGING_PREFIX: &str = ".bullet-family-setup.";
pub(crate) fn supported_node_version(value: &str) -> bool {
    crate::toolchain_pins::matches_node(value)
}

pub(crate) fn supported_npm_version(value: &str) -> bool {
    crate::toolchain_pins::matches_npm(value)
}

pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    let options = parse_args(explicit_root, args)?;
    require_setup_containment()?;
    let family_root = AdmittedRoot::open(&options.root)?;
    let hub_root = crate::doctor::discover_hub(current_dir, None)?;
    family_root.ensure_path_identity()?;
    let expected_hub = family_root.path().join("bullet-farm");
    if expected_hub.canonicalize().map_err(CoordError::io)?
        != hub_root.canonicalize().map_err(CoordError::io)?
    {
        return Err(CoordError::new(
            "HUB_LOCATION_MISMATCH",
            format!(
                "setup root must contain the invoking hub at {}",
                expected_hub.display()
            ),
        ));
    }
    family_root.ensure_path_identity()?;
    let lock = family_lock::load(&hub_root.join("family.lock"))?;
    let allowed_signers = hub_root.join("release/allowed_signers");
    crate::checkout::verify_hub(&lock, &hub_root, family_root.path(), &allowed_signers)?;
    family_root.ensure_path_identity()?;
    let toolchain = Toolchain::admit_locked(
        options.cargo_bin.as_deref(),
        options.node_bin.as_deref(),
        options.npm_cli.as_deref(),
        &lock.external.toolchain,
    )?;
    let environment = SetupEnvironment::create(&family_root, &toolchain)?;
    {
        let validator = SetupValidator::new(options.offline, &toolchain, &environment);
        install_transaction(
            &family_root,
            &hub_root,
            &lock,
            options.offline,
            &JeryuTransport,
            &validator,
            &NoFault,
        )?;
    }
    environment.finish()?;
    Ok(format!(
        "setup complete: {} members at {}",
        lock.member.len() + 1,
        lock.tag
    ))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn require_setup_containment() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn require_setup_containment() -> Result<(), CoordError> {
    Err(CoordError::new(
        "UNSUPPORTED_PLATFORM_CONTAINMENT",
        "setup cannot mutate on this platform until process-tree termination and atomic no-replace publication have equivalent release evidence",
    ))
}

trait CloneTransport {
    fn clone_member(
        &self,
        member: &LockedMember,
        destination: &Path,
        offline: bool,
    ) -> Result<(), CoordError>;
}

struct JeryuTransport;

impl CloneTransport for JeryuTransport {
    fn clone_member(
        &self,
        member: &LockedMember,
        destination: &Path,
        offline: bool,
    ) -> Result<(), CoordError> {
        if offline {
            return Err(CoordError::new(
                "OFFLINE_SOURCE_UNAVAILABLE",
                format!(
                    "{} is absent and no verified local object cache is configured",
                    member.name
                ),
            ));
        }
        let source = member.jeryu_url.as_deref().ok_or_else(|| {
            CoordError::new(
                "SOURCE_METADATA_MISSING",
                format!("{} has no authenticated Jeryu URL", member.name),
            )
        })?;
        clone_repository(OsStr::new(source), destination, false)
    }
}

#[cfg(test)]
fn install(
    family_root: &Path,
    hub_root: &Path,
    lock: &family_lock::FamilyLock,
    offline: bool,
    transport: &dyn CloneTransport,
) -> Result<(), CoordError> {
    let family_root = AdmittedRoot::open(family_root)?;
    install_transaction(
        &family_root,
        hub_root,
        lock,
        offline,
        transport,
        &ExactOnlyValidator,
        &NoFault,
    )
}

fn checkout_locked_commit(member: &LockedMember, repo: &Path) -> Result<(), CoordError> {
    let commit = member
        .commit_oid
        .split_once(':')
        .map(|(_, oid)| oid)
        .ok_or_else(|| CoordError::new("INVALID_FAMILY_LOCK", "commit OID lacks algorithm"))?;
    run_git(
        Some(repo),
        &[
            OsStr::new("-c"),
            OsStr::new("core.hooksPath=/dev/null"),
            OsStr::new("-c"),
            OsStr::new("core.autocrlf=false"),
            OsStr::new("checkout"),
            OsStr::new("--detach"),
            OsStr::new("--force"),
            OsStr::new(commit),
        ],
    )
}

fn clone_repository(
    source: &OsStr,
    destination: &Path,
    permit_local: bool,
) -> Result<(), CoordError> {
    let local_policy = if permit_local { "always" } else { "never" };
    run_git(
        None,
        &[
            OsStr::new("-c"),
            OsStr::new(if permit_local {
                "protocol.file.allow=always"
            } else {
                "protocol.file.allow=never"
            }),
            OsStr::new("clone"),
            OsStr::new("--no-checkout"),
            OsStr::new("--no-local"),
            OsStr::new("--config"),
            OsStr::new("core.hooksPath=/dev/null"),
            OsStr::new("--config"),
            OsStr::new("core.autocrlf=false"),
            OsStr::new(source),
            destination.as_os_str(),
        ],
    )
    .map_err(|_| {
        CoordError::new(
            "SOURCE_CLONE_FAILED",
            format!("Git refused the locked source (file protocol {local_policy})"),
        )
    })
}

#[cfg(test)]
mod tests;
