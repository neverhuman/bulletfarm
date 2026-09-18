//! Read-only onboarding diagnostics that work from a hub-only clone.

mod checks;
mod discovery;
mod model;

use std::path::Path;

use crate::coord::CoordError;
use checks::{
    check_exact_family_authority, check_family_layout, check_hub_checkout, check_source_metadata,
    check_tools,
};
use discovery::{discover_hub as resolve_hub, read_lock};
use model::{CheckStatus, DoctorReport, DoctorStatus};

const USAGE: &str = "usage: bullet-family [--root PATH] doctor --json";

/// One doctor run: the machine-parsable report and the exit code that reports
/// the same verdict to a shell. `--json` output is identical on both paths.
pub struct DoctorExecution {
    output: String,
    exit_code: u8,
}

impl DoctorExecution {
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }

    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }

    #[must_use]
    pub fn into_output(self) -> String {
        self.output
    }
}

/// Diagnose the hub and report the verdict in both the JSON body and the exit
/// code. A blocked hub exits 3; it never signals success.
///
/// # Errors
///
/// Typed `CoordError` for usage, discovery, and lock-decoding failures. Those
/// keep their own exit codes; 3 means "diagnosed, and not usable".
pub fn execute(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<DoctorExecution, CoordError> {
    if args != ["--json"] {
        return Err(CoordError::new("USAGE", USAGE));
    }
    let hub_root = discover_hub(current_dir, explicit_root)?;
    let family_root = hub_root
        .parent()
        .filter(|parent| parent.join("repos.manifest.toml").is_file())
        .map(Path::to_path_buf);
    let lock = read_lock(&hub_root)?;
    let mut checks = vec![check_hub_checkout(&hub_root), check_tools()];
    checks.push(check_source_metadata(&lock));
    checks.extend(check_family_layout(
        &hub_root,
        family_root.as_deref(),
        &lock,
    ));
    checks.push(check_exact_family_authority(
        &hub_root,
        family_root.as_deref(),
        &lock,
    ));
    let status = if checks
        .iter()
        .any(|check| check.status == CheckStatus::Blocked)
    {
        DoctorStatus::Blocked
    } else {
        DoctorStatus::Ready
    };
    let output = serde_json::to_string_pretty(&DoctorReport {
        schema_version: 1,
        command: "doctor",
        status: status.as_str(),
        hub_root: display_path(&hub_root),
        family_root: family_root.as_deref().map(display_path),
        checks,
    })
    .map_err(CoordError::json)?;
    Ok(DoctorExecution {
        output,
        exit_code: status.exit_code(),
    })
}

/// The report body alone, for callers that already treat a refusal as failure.
///
/// # Errors
///
/// The same typed `CoordError`s as [`execute`].
pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    execute(current_dir, explicit_root, args).map(DoctorExecution::into_output)
}

pub(crate) fn discover_hub(
    current_dir: &Path,
    explicit_root: Option<&str>,
) -> Result<std::path::PathBuf, CoordError> {
    resolve_hub(current_dir, explicit_root)
}

/// Read-only summary of the checked-in family lock for in-process projections.
/// It carries no installation authority; installers use the strict schema-3 path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LockSummary {
    pub(crate) schema_version: String,
    pub(crate) tag: String,
    pub(crate) installable: bool,
    /// `(member name, algorithm-free commit OID)` in lock order.
    pub(crate) members: Vec<(String, String)>,
}

pub(crate) fn lock_summary(hub_root: &Path) -> Result<LockSummary, CoordError> {
    let lock = read_lock(hub_root)?;
    Ok(LockSummary {
        schema_version: lock.schema_version,
        tag: lock.tag,
        installable: lock.installable_schema,
        members: lock
            .member
            .into_iter()
            .map(|member| (member.name, member.commit_oid))
            .collect(),
    })
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
