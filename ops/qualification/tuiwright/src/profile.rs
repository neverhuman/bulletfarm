//! Explicit component context validation; never live or signed admission.
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

fn environment(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("HOSTED_SUBJECT_MISSING: {name}"))
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    ensure!(output.status.success(), "HOSTED_SOURCE_READ_FAILED");
    Ok(String::from_utf8(output.stdout)?)
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn positive_decimal(value: &str) -> bool {
    !value.starts_with('0') && !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

pub fn admit(args: &[String]) -> Result<Value> {
    ensure!(cfg!(target_os = "linux"), "TUIWRIGHT_PLATFORM_UNAVAILABLE");
    let mode = args.first().map(String::as_str).unwrap_or_default();
    if mode == "hosted-component" {
        ensure!(
            environment("CI")? == "true"
                && environment("GITHUB_ACTIONS")? == "true"
                && environment("RUNNER_OS")? == "Linux",
            "TUIWRIGHT_HOSTED_CONTEXT_INVALID"
        );
    } else {
        ensure!(
            std::env::var_os("CI").is_none() && std::env::var_os("GITHUB_ACTIONS").is_none(),
            "TUIWRIGHT_HOSTED_RUN_REFUSED"
        );
        ensure!(
            std::fs::read_to_string("/proc/sys/kernel/hostname")?.trim() == "xbabe2",
            "TUIWRIGHT_HOST_NOT_ADMITTED"
        );
    }
    ensure!(args.len() == 7 && matches!(mode, "component" | "hosted-component")
        && args[1] == "--bullet" && args[3] == "--sha256" && args[5] == "--output",
        "USAGE: component|hosted-component --bullet /absolute/bullet --sha256 SHA256 --output /absolute/new-directory");
    if mode == "component" {
        return Ok(json!({"profile":mode}));
    }
    let commit = environment("GITHUB_SHA")?;
    let workflow_commit = environment("GITHUB_WORKFLOW_SHA")?;
    let workflow_digest = environment("BULLET_TUIWRIGHT_WORKFLOW_SHA256")?;
    let event = environment("GITHUB_EVENT_NAME")?;
    let run = environment("GITHUB_RUN_ID")?;
    let attempt = environment("GITHUB_RUN_ATTEMPT")?;
    let repository = environment("GITHUB_REPOSITORY")?;
    let reference = environment("GITHUB_REF")?;
    let workflow_ref = environment("GITHUB_WORKFLOW_REF")?;
    let repository_parts = repository.split('/').collect::<Vec<_>>();
    ensure!(
        hex(&commit, 40)
            && hex(&workflow_commit, 40)
            && hex(&workflow_digest, 64)
            && matches!(event.as_str(), "push" | "pull_request" | "merge_group")
            && positive_decimal(&run)
            && positive_decimal(&attempt)
            && repository_parts.len() == 2
            && repository_parts.iter().all(|part| !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-')))
            && reference.starts_with("refs/")
            && workflow_ref == format!("{repository}/.github/workflows/ci.yml@{reference}"),
        "TUIWRIGHT_HOSTED_SUBJECT_INVALID"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()?;
    let workspace = environment("GITHUB_WORKSPACE")?;
    let job = environment("GITHUB_JOB")?;
    ensure!(
        job == "operator-tui"
            && Path::new(&workspace) == root
            && Path::new(&workspace).canonicalize()? == root,
        "TUIWRIGHT_HOSTED_WORKSPACE_INVALID"
    );
    ensure!(
        git(&root, &["rev-parse", "HEAD"])?.trim() == commit
            && git(&root, &["status", "--porcelain", "--untracked-files=all"])?.is_empty(),
        "TUIWRIGHT_HOSTED_SOURCE_CHANGED"
    );
    let workflow = root.join(".github/workflows/ci.yml");
    ensure!(
        !workflow.symlink_metadata()?.file_type().is_symlink()
            && crate::evidence::hash(&workflow)? == workflow_digest
            && git(
                &root,
                &[
                    "show",
                    &format!("{workflow_commit}:.github/workflows/ci.yml")
                ]
            )?
            .as_bytes()
                == std::fs::read(&workflow)?,
        "TUIWRIGHT_HOSTED_WORKFLOW_CHANGED"
    );
    Ok(
        json!({"profile":mode,"commit":commit,"tree":git(&root, &["rev-parse", "HEAD^{tree}"])?.trim(),
        "workflow_sha256":workflow_digest,"workflow_commit":workflow_commit,"workflow_ref":workflow_ref,
        "workspace":workspace,"job":job,"repository":repository,"event":event,"run_id":run,"run_attempt":attempt,
        "profile_source_sha256":crate::evidence::hash(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/profile.rs"))?}),
    )
}
