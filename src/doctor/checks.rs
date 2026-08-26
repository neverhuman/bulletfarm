use std::{path::Path, process::Command, time::Duration};

use super::model::{DoctorCheck, DoctorFamilyLock};
use crate::process::{Limits, run_bounded};

const GIT_BIN: &str = "/usr/bin/git";
const VERSION_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(10),
    stdout_bytes: 64 * 1024,
    stderr_bytes: 64 * 1024,
};
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 16 * 1024 * 1024,
};

pub(super) fn check_hub_checkout(hub_root: &Path) -> DoctorCheck {
    let dot_git = hub_root.join(".git");
    if dot_git.is_dir() {
        DoctorCheck::pass(
            "hub_checkout",
            "hub is an ordinary Git checkout (.git is a directory)",
        )
    } else if dot_git.is_file() {
        DoctorCheck::blocked(
            "hub_checkout",
            "hub is a linked Git worktree (.git is a file)",
            "clone the hub as an ordinary repository; Bullet Farm forbids worktrees",
        )
    } else {
        DoctorCheck::blocked(
            "hub_checkout",
            "hub has no .git directory",
            "start from an ordinary clone of the signed Bullet Farm hub tag",
        )
    }
}

pub(super) fn check_tools() -> DoctorCheck {
    let probes = [
        ("git", GIT_BIN, &["--version"] as &[&str]),
        ("rustc", "rustc", &["--version"]),
        ("cargo", "cargo", &["--version"]),
        ("rustup", "rustup", &["--version"]),
        ("rustfmt", "rustfmt", &["--version"]),
        ("clippy", "cargo", &["clippy", "--version"]),
        ("node", "node", &["--version"]),
        ("npm", "npm", &["--version"]),
        ("just", "just", &["--version"]),
    ];
    let mut versions = Vec::new();
    let mut missing = Vec::new();
    for (label, command, args) in probes {
        match command_version(command, args).and_then(|version| admit_tool_version(label, version))
        {
            Ok(version) => versions.push(format!("{label}={version}")),
            Err(reason) => missing.push(format!("{label} ({reason})")),
        }
    }
    if missing.is_empty() {
        DoctorCheck::pass("toolchain", versions.join("; "))
    } else {
        DoctorCheck::blocked(
            "toolchain",
            format!("missing or unusable tools: {}", missing.join(", ")),
            "install the pinned Rust and Node toolchains plus git, rustup, rustfmt, clippy, npm, and just; rerun doctor",
        )
    }
}

pub(super) fn check_source_metadata(lock: &DoctorFamilyLock) -> DoctorCheck {
    if !lock.installable_schema {
        return DoctorCheck::blocked(
            "source_metadata",
            format!(
                "family.lock schema {} is diagnostic-only and lacks the complete install authority",
                lock.schema_version
            ),
            "restore authenticated Jeryu sources, publish signed member tags, and generate schema 3 with exact trees, lockfiles, and artifact checksums",
        );
    }
    let missing = lock
        .member
        .iter()
        .filter(|member| member.name != "bullet-farm")
        .filter(|member| member.jeryu_url.as_deref().is_none_or(str::is_empty))
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>();
    let missing_slugs = lock
        .member
        .iter()
        .filter(|member| member.name != "bullet-farm")
        .filter(|member| member.jeryu_slug.as_deref().is_none_or(str::is_empty))
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>();
    if missing.is_empty() && missing_slugs.is_empty() {
        DoctorCheck::pass(
            "source_metadata",
            "every non-hub member has an immutable Jeryu source URL and slug",
        )
    } else {
        DoctorCheck::blocked(
            "source_metadata",
            format!(
                "family.lock lacks clone URLs for [{}] and Jeryu slugs for [{}]",
                missing.join(", "),
                missing_slugs.join(", ")
            ),
            "publish signed member tags, add immutable Jeryu source URLs/slugs and artifact checksums to the lock, then regenerate and verify it",
        )
    }
}

pub(super) fn check_family_layout(
    hub_root: &Path,
    family_root: Option<&Path>,
    lock: &DoctorFamilyLock,
) -> Vec<DoctorCheck> {
    let Some(family_root) = family_root else {
        return vec![DoctorCheck::blocked(
            "family_layout",
            "only the hub checkout is present; no outer repos.manifest.toml was found",
            "publish a signed schema-3 lock with authenticated Jeryu sources, then run bullet-family setup; the current diagnostic lock cannot authorize member creation",
        )];
    };
    let mut absent = Vec::new();
    let mut worktrees = Vec::new();
    let mut wrong_heads = Vec::new();
    let mut dirty = Vec::new();
    let mut unsafe_metadata = Vec::new();
    if !lock
        .member
        .iter()
        .any(|member| member.name == "bullet-farm")
    {
        match crate::checkout::admit_repository_metadata(hub_root, None) {
            Ok(()) => match crate::checkout::verify_exact_worktree(hub_root) {
                Ok(()) => {}
                Err(error) => dirty.push(format!("bullet-farm ({error})")),
            },
            Err(error) => unsafe_metadata.push(format!("bullet-farm ({error})")),
        }
    }
    for member in &lock.member {
        let repo = if member.name == "bullet-farm" {
            hub_root.to_path_buf()
        } else {
            family_root.join(&member.name)
        };
        if !repo.join(".git").exists() {
            absent.push(member.name.clone());
            continue;
        }
        if !repo.join(".git").is_dir() {
            worktrees.push(member.name.clone());
            continue;
        }
        if let Err(error) =
            crate::checkout::admit_repository_metadata(&repo, member.jeryu_url.as_deref())
        {
            unsafe_metadata.push(format!("{} ({error})", member.name));
            continue;
        }
        if member.name != "bullet-farm" {
            match git(&repo, &["rev-parse", "HEAD"]) {
                Ok(head) if head.trim() == member.commit_oid => {}
                Ok(head) => wrong_heads.push(format!(
                    "{} expected {} found {}",
                    member.name,
                    member.commit_oid,
                    head.trim()
                )),
                Err(reason) => wrong_heads.push(format!("{} ({reason})", member.name)),
            }
        }
        match crate::checkout::verify_exact_worktree(&repo) {
            Ok(()) => {}
            Err(error) => dirty.push(format!("{} ({error})", member.name)),
        }
    }
    vec![
        layout_result(&absent, &worktrees, &unsafe_metadata),
        oid_result(&wrong_heads),
        cleanliness_result(&dirty),
    ]
}

pub(super) fn check_exact_family_authority(
    hub_root: &Path,
    family_root: Option<&Path>,
    lock: &DoctorFamilyLock,
) -> DoctorCheck {
    let Some(current) = &lock.current else {
        return DoctorCheck::blocked(
            "exact_family_authority",
            "the diagnostic schema-2 lock cannot authenticate an install",
            "publish signed non-hub subjects, generate schema 3, commit it, and sign the exact hub tag",
        );
    };
    let Some(family_root) = family_root else {
        return DoctorCheck::blocked(
            "exact_family_authority",
            "the complete family is not installed",
            "run bullet-family setup from the signed hub after schema-3 source authority is published",
        );
    };
    match crate::checkout::verify_family(family_root, hub_root, current) {
        Ok(()) => DoctorCheck::pass(
            "exact_family_authority",
            "hub and members match the signed schema-3 lock, exact subjects, and clean checkouts",
        ),
        Err(error) => DoctorCheck::blocked(
            "exact_family_authority",
            error.to_string(),
            "restore clean ordinary clones at the signed exact subjects; never reset a dirty shared checkout",
        ),
    }
}

fn layout_result(
    absent: &[String],
    worktrees: &[String],
    unsafe_metadata: &[String],
) -> DoctorCheck {
    if absent.is_empty() && worktrees.is_empty() && unsafe_metadata.is_empty() {
        DoctorCheck::pass(
            "family_layout",
            "all locked members are ordinary sibling checkouts",
        )
    } else {
        DoctorCheck::blocked(
            "family_layout",
            format!(
                "missing members [{}]; forbidden worktrees [{}]; unsafe Git metadata [{}]",
                absent.join(", "),
                worktrees.join(", "),
                unsafe_metadata.join("; ")
            ),
            "create ordinary canonical clones for missing members; never create Git worktrees",
        )
    }
}

fn oid_result(wrong_heads: &[String]) -> DoctorCheck {
    if wrong_heads.is_empty() {
        DoctorCheck::pass(
            "member_oids",
            "every present non-hub member is at its locked commit OID",
        )
    } else {
        DoctorCheck::blocked(
            "member_oids",
            wrong_heads.join("; "),
            "recreate each clean canonical clone at the exact locked OID; never reset a dirty shared checkout",
        )
    }
}

fn cleanliness_result(dirty: &[String]) -> DoctorCheck {
    if dirty.is_empty() {
        DoctorCheck::pass("clean_checkouts", "every present family checkout is clean")
    } else {
        DoctorCheck::blocked(
            "clean_checkouts",
            format!("dirty checkouts: {}", dirty.join(", ")),
            "finish and hand off active claims; do not clean, reset, or stage another agent's changes",
        )
    }
}

fn command_version(command: &str, args: &[&str]) -> Result<String, String> {
    let output = run_bounded(Command::new(command).args(args), command, VERSION_LIMITS)
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("exit {:?}", output.status.code()));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .next()
        .unwrap_or("version unavailable")
        .trim();
    Ok(line.chars().take(160).collect())
}

fn admit_tool_version(label: &str, version: String) -> Result<String, String> {
    if label == "node" && !crate::setup::supported_node_version(&version) {
        return Err(format!(
            "requires exact Node v{}; found {version}",
            crate::toolchain_pins::node()
        ));
    }
    if label == "npm" && !crate::setup::supported_npm_version(&version) {
        return Err(format!(
            "requires exact npm {}; found {version}",
            crate::toolchain_pins::npm()
        ));
    }
    Ok(version)
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = run_bounded(
        Command::new(GIT_BIN)
            .arg("-C")
            .arg(repo)
            .args([
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.attributesFile=/dev/null",
                "-c",
                "core.excludesFile=/dev/null",
            ])
            .args(args)
            .env_clear()
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_TERMINAL_PROMPT", "0"),
        "Git doctor check",
        GIT_LIMITS,
    )
    .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("git exited {:?}", output.status.code()));
    }
    String::from_utf8(output.stdout).map_err(|_| "git emitted non-UTF-8 output".to_owned())
}

#[cfg(test)]
mod tests {
    use super::admit_tool_version;

    #[test]
    fn doctor_blocks_unsupported_node_versions() {
        assert_eq!(
            admit_tool_version("node", "v22.23.2".into()).unwrap(),
            "v22.23.2"
        );
        assert!(
            admit_tool_version("node", "v22.23.1".into())
                .unwrap_err()
                .contains("Node v22.23.2")
        );
        assert!(admit_tool_version("node", "v26.1.0".into()).is_err());
        assert!(admit_tool_version("node", "v22.23".into()).is_err());
        assert_eq!(
            admit_tool_version("npm", "10.9.8".into()).unwrap(),
            "10.9.8"
        );
        assert!(admit_tool_version("npm", "11.13.0".into()).is_err());
        assert_eq!(
            admit_tool_version("cargo", "cargo 1.97.1".into()).unwrap(),
            "cargo 1.97.1"
        );
    }
}
