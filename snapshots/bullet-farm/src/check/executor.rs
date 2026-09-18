//! Bounded execution bound to clean, unchanged ordinary Git checkouts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use serde::Deserialize;

use super::{
    catalog::{self, CommandGate, SubjectScope},
    model::{CheckReport, CheckTier, GateClass, GateResult},
    prerequisites,
    profiles::ReleaseProfile,
    subject::RepositorySubject,
};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const GIT_BIN: &str = "/usr/bin/git";
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(20),
    stdout_bytes: 2 * 1024 * 1024,
    stderr_bytes: 2 * 1024 * 1024,
};
const COMMAND_OUTPUT_BYTES: usize = 32 * 1024 * 1024;
const FAST_PROFILE_LIMIT: Duration = Duration::from_secs(60);
const REPOSITORIES: &[&str] = &[
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];

pub(super) fn report(hub: &Path, tier: CheckTier) -> Result<CheckReport, CoordError> {
    if tier == CheckTier::Release {
        return prerequisites::report_release_with_evidence(hub).map_err(model_error);
    }
    let repositories =
        match RepositorySet::discover(hub) {
            Ok(repositories) => repositories,
            Err(error) => {
                return CheckReport::new(
                    tier,
                    vec![GateResult::blocked(
                    "catalog.family-layout",
                    GateClass::Component,
                    error.to_string(),
                    "restore ordinary clean sibling checkouts under the discovered family root",
                )
                .map_err(model_error)?],
                )
                .map_err(model_error);
            }
        };
    if let Err(error) = repositories.capture_family() {
        return CheckReport::new(
            tier,
            vec![GateResult::blocked(
                "catalog.exact-subjects",
                GateClass::Component,
                error.to_string(),
                "finish and hand off active claims, then rerun from four clean ordinary exact checkouts",
            )
            .map_err(model_error)?],
        )
        .map_err(model_error);
    }
    let started = Instant::now();
    let mut gates = Vec::new();
    for gate in catalog::commands(tier) {
        let timeout = if tier == CheckTier::Fast {
            gate.timeout
                .min(FAST_PROFILE_LIMIT.saturating_sub(started.elapsed()))
        } else {
            gate.timeout
        };
        if timeout.is_zero() {
            gates.push(bind_if_clean(
                GateResult::fail(
                    gate.id,
                    gate.class,
                    "the 60 second fast profile deadline expired before this fixed command could start",
                    "warm the pinned dependency caches, remove unintended work, and rerun the unchanged catalog",
                )
                .map_err(model_error)?,
                repositories.capture(gate.scope),
            )?);
        } else {
            gates.push(execute(gate, &repositories, timeout)?);
        }
    }
    if tier == CheckTier::Fast {
        let elapsed = started.elapsed();
        let subjects = repositories.capture_family();
        let gate = if elapsed <= FAST_PROFILE_LIMIT {
            GateResult::pass(
                "fast.wall-clock",
                GateClass::Component,
                "all admitted fast commands completed within the 60 second profile ceiling",
            )
        } else {
            GateResult::fail(
                "fast.wall-clock",
                GateClass::Component,
                "the admitted fast catalog exceeded the 60 second profile ceiling",
                "warm the pinned dependency caches, remove unintended work, and rerun the unchanged catalog",
            )
        }
        .map_err(model_error)?;
        gates.push(bind_if_clean(gate, subjects)?);
        gates.push(bind_if_clean(
            GateResult::pass(
                "fast.affected-path-routing",
                GateClass::Component,
                "the conservative affected-path route executed a nonempty fast lane for every family member plus generated drift",
            )
            .map_err(model_error)?,
            repositories.capture_family(),
        )?);
    } else {
        gates.extend(prerequisites::required_blockers().map_err(model_error)?);
    }
    CheckReport::new(tier, gates).map_err(model_error)
}

pub(super) fn report_profile(
    hub: &Path,
    profile: ReleaseProfile,
    receipts: &Path,
) -> Result<CheckReport, CoordError> {
    prerequisites::report_release_profile_for_hub(hub, profile, receipts).map_err(model_error)
}

fn execute(
    gate: &CommandGate,
    repositories: &RepositorySet,
    timeout: Duration,
) -> Result<GateResult, CoordError> {
    let before = match repositories.capture(gate.scope) {
        Ok(subjects) => subjects,
        Err(error) => {
            return GateResult::blocked(
                gate.id,
                gate.class,
                error.to_string(),
                "finish active claims and restore clean ordinary checkouts; never reset another agent's work",
            )
            .map_err(model_error);
        }
    };
    let repository = repositories.path(gate.repository)?;
    let script = match admit_script(repository, gate.script) {
        Ok(script) => script,
        Err(error) => {
            return GateResult::blocked(
                gate.id,
                gate.class,
                error.to_string(),
                "restore the exact tracked non-symlink catalog script and rerun from a clean checkout",
            )
            .map_err(model_error)?
            .with_subjects(before)
            .map_err(model_error);
        }
    };
    let outcome = run_bounded(
        Command::new(catalog::BASH_BIN)
            .current_dir(repository)
            .arg(&script)
            .args(gate.arguments),
        gate.id,
        Limits {
            timeout,
            stdout_bytes: COMMAND_OUTPUT_BYTES,
            stderr_bytes: COMMAND_OUTPUT_BYTES,
        },
    );
    let after = match repositories.capture(gate.scope) {
        Ok(subjects) => subjects,
        Err(error) => {
            return GateResult::unknown(
                gate.id,
                gate.class,
                format!("the command returned but its exact subjects could not be reconstructed: {error}"),
                "preserve the checkouts, inspect the command boundary, and rerun only after exact subjects are recoverable",
            )
            .map_err(model_error)
            .and_then(|result| result.with_subjects(before).map_err(model_error));
        }
    };
    if before != after {
        return GateResult::fail(
            gate.id,
            gate.class,
            "the fixed command changed an exact Git subject or checkout state",
            "inspect and preserve the mutation; proof commands must leave every bound checkout clean and unchanged",
        )
        .map_err(model_error)?
        .with_subjects(before)
        .map_err(model_error);
    }
    let result = match outcome {
        Ok(output) if output.status.success() => GateResult::pass(
            gate.id,
            gate.class,
            format!(
                "{} {} completed on clean unchanged subjects",
                catalog::BASH_BIN, gate.script
            ),
        ),
        Ok(output) => GateResult::fail(
            gate.id,
            gate.class,
            format!(
                "{} {} exited {:?} (stdout {} bytes, stderr {} bytes)",
                catalog::BASH_BIN,
                gate.script,
                output.status.code(),
                output.stdout.len(),
                output.stderr.len()
            ),
            "run the named local lane directly, repair its first failure, and rerun this exact catalog",
        ),
        Err(error) => GateResult::fail(
            gate.id,
            gate.class,
            format!("the bounded command failed: {error}"),
            "restore the admitted local toolchain, inspect timeout/output/process-tree failure, and rerun",
        ),
    }
    .map_err(model_error)?;
    result.with_subjects(before).map_err(model_error)
}

fn bind_if_clean(
    gate: GateResult,
    subjects: Result<Vec<RepositorySubject>, CoordError>,
) -> Result<GateResult, CoordError> {
    match subjects {
        Ok(subjects) => gate.with_subjects(subjects).map_err(model_error),
        Err(error) => GateResult::unknown(
            gate.id().to_owned(),
            gate.class(),
            format!(
                "{}; exact post-catalog subjects are unavailable: {error}",
                gate.detail()
            ),
            "preserve the checkouts and rerun only after every exact subject is clean and reconstructible",
        )
        .map_err(model_error),
    }
}

fn admit_script(repository: &Path, relative: &str) -> Result<PathBuf, CoordError> {
    let path = repository.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        CoordError::new(
            "CHECK_SCRIPT_UNAVAILABLE",
            format!("{}: {error}", path.display()),
        )
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(CoordError::new(
            "CHECK_SCRIPT_REFUSED",
            format!("{} is not a regular non-symlink file", path.display()),
        ));
    }
    let canonical = path.canonicalize().map_err(CoordError::io)?;
    if !canonical.starts_with(repository) {
        return Err(CoordError::new(
            "CHECK_SCRIPT_REFUSED",
            format!("{} escapes its repository", path.display()),
        ));
    }
    Ok(canonical)
}

pub(super) struct RepositorySet {
    paths: BTreeMap<&'static str, PathBuf>,
}

impl RepositorySet {
    pub(super) fn discover(hub: &Path) -> Result<Self, CoordError> {
        let hub = hub.canonicalize().map_err(CoordError::io)?;
        let family = hub.parent().ok_or_else(|| {
            CoordError::new("FAMILY_LAYOUT_UNAVAILABLE", "hub has no family parent")
        })?;
        if !family.join("repos.manifest.toml").is_file() {
            return Err(CoordError::new(
                "FAMILY_LAYOUT_UNAVAILABLE",
                "outer repos.manifest.toml is absent",
            ));
        }
        admit_family_manifest(family)?;
        let mut paths = BTreeMap::new();
        for &name in REPOSITORIES {
            let path = if name == "bullet-farm" {
                hub.clone()
            } else {
                family.join(name).canonicalize().map_err(|error| {
                    CoordError::new(
                        "FAMILY_LAYOUT_UNAVAILABLE",
                        format!("{name} cannot be resolved: {error}"),
                    )
                })?
            };
            let git_metadata = fs::symlink_metadata(path.join(".git")).map_err(|error| {
                CoordError::new(
                    "FAMILY_LAYOUT_UNAVAILABLE",
                    format!("{name} Git metadata cannot be inspected: {error}"),
                )
            })?;
            if !git_metadata.file_type().is_dir() {
                return Err(CoordError::new(
                    "FORBIDDEN_WORKTREE_LAYOUT",
                    format!("{name} is not an ordinary Git checkout"),
                ));
            }
            paths.insert(name, path);
        }
        Ok(Self { paths })
    }

    pub(super) fn path(&self, name: &str) -> Result<&Path, CoordError> {
        self.paths.get(name).map(PathBuf::as_path).ok_or_else(|| {
            CoordError::new(
                "INVALID_CHECK_CATALOG",
                format!("unknown repository {name}"),
            )
        })
    }

    fn capture(&self, scope: SubjectScope) -> Result<Vec<RepositorySubject>, CoordError> {
        match scope {
            SubjectScope::Repository(name) => Ok(vec![capture_subject(name, self.path(name)?)?]),
            SubjectScope::Family => self.capture_family(),
        }
    }

    pub(super) fn capture_family(&self) -> Result<Vec<RepositorySubject>, CoordError> {
        REPOSITORIES
            .iter()
            .map(|&name| capture_subject(name, self.path(name)?))
            .collect()
    }
}

#[derive(Deserialize)]
struct FamilyManifest {
    required_repos: Vec<String>,
}

fn admit_family_manifest(family: &Path) -> Result<(), CoordError> {
    let path = family.join("repos.manifest.toml");
    let metadata = fs::symlink_metadata(&path).map_err(CoordError::io)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 1024 * 1024
    {
        return Err(CoordError::new(
            "INVALID_FAMILY_MANIFEST",
            "outer repos.manifest.toml must be a regular non-symlink file no larger than 1 MiB",
        ));
    }
    let text = fs::read_to_string(&path).map_err(CoordError::io)?;
    let manifest: FamilyManifest = toml::from_str(&text)
        .map_err(|error| CoordError::new("INVALID_FAMILY_MANIFEST", error.to_string()))?;
    let actual = manifest
        .required_repos
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = REPOSITORIES.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected || actual.len() != manifest.required_repos.len() {
        return Err(CoordError::new(
            "INVALID_FAMILY_MANIFEST",
            "required_repos must contain each canonical family member exactly once",
        ));
    }
    Ok(())
}

fn capture_subject(name: &str, repository: &Path) -> Result<RepositorySubject, CoordError> {
    let top = git(repository, &["rev-parse", "--show-toplevel"])?;
    let top = PathBuf::from(top).canonicalize().map_err(CoordError::io)?;
    if top != repository {
        return Err(CoordError::new(
            "CHECK_SUBJECT_MISMATCH",
            format!(
                "{name} resolves to unexpected Git top-level {}",
                top.display()
            ),
        ));
    }
    let algorithm = git(repository, &["rev-parse", "--show-object-format"])?;
    if !matches!(algorithm.as_str(), "sha1" | "sha256") {
        return Err(CoordError::new(
            "CHECK_SUBJECT_MISMATCH",
            format!("{name} uses unsupported Git object format {algorithm:?}"),
        ));
    }
    let head = git(repository, &["rev-parse", "--verify", "HEAD"])?;
    let tree = git(repository, &["rev-parse", "--verify", "HEAD^{tree}"])?;
    if !git_bytes(
        repository,
        &["status", "--porcelain=v2", "-z", "--untracked-files=all"],
    )?
    .is_empty()
    {
        return Err(CoordError::new(
            "CHECK_SUBJECT_DIRTY",
            format!("{name} has tracked, untracked, or index changes"),
        ));
    }
    RepositorySubject::new(
        name,
        format!("{algorithm}:{head}"),
        format!("{algorithm}:{tree}"),
    )
    .map_err(model_error)
}

pub(super) fn git(repository: &Path, arguments: &[&str]) -> Result<String, CoordError> {
    let bytes = git_bytes(repository, arguments)?;
    let text = String::from_utf8(bytes).map_err(|_| {
        CoordError::new("CHECK_GIT_FAILED", "Git emitted non-UTF-8 identity output")
    })?;
    Ok(text.trim().to_owned())
}

pub(super) fn git_bytes(repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, CoordError> {
    let output = run_bounded(
        Command::new(GIT_BIN)
            .arg("-C")
            .arg(repository)
            .args([
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.attributesFile=/dev/null",
                "-c",
                "core.excludesFile=/dev/null",
            ])
            .args(arguments)
            .env_clear()
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_TERMINAL_PROMPT", "0"),
        "Git check subject",
        GIT_LIMITS,
    )?;
    if !output.status.success() {
        return Err(CoordError::new(
            "CHECK_GIT_FAILED",
            format!("Git exited {:?}", output.status.code()),
        ));
    }
    Ok(output.stdout)
}

fn model_error(error: impl std::fmt::Display) -> CoordError {
    CoordError::new("INVALID_CHECK_REPORT", error.to_string())
}
