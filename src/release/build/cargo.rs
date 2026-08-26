//! Locked release compilation of every packaged Rust binary.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use super::{BINARIES, BuildPlan, failed, portal::PortalOutput, subject::Toolchain};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const CARGO_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(3 * 3600),
    stdout_bytes: 64 * 1024 * 1024,
    stderr_bytes: 64 * 1024 * 1024,
};

/// One release binary staged for the archive.
#[derive(Clone, Debug)]
pub(super) struct BuiltBinary {
    pub(super) name: String,
    pub(super) path: PathBuf,
}

/// One exact child invocation, recorded verbatim for the provenance record.
#[derive(Clone, Debug)]
pub(super) struct RecordedCommand {
    pub(super) program: String,
    pub(super) args: Vec<String>,
    pub(super) cwd: String,
    pub(super) env: Vec<(String, String)>,
}

pub(super) fn build_binaries(
    plan: &BuildPlan,
    portal: &PortalOutput,
    commands: &mut Vec<RecordedCommand>,
) -> Result<Vec<BuiltBinary>, CoordError> {
    let portal_dist = portal.dist.to_str().ok_or_else(|| {
        failed("the Portal dist path is not UTF-8 and cannot be passed to the build")
    })?;
    invoke(
        plan,
        "bullet-kernel",
        &[
            "build",
            "--locked",
            "--release",
            "-p",
            "bullet-farmd",
            "--features",
            "embedded-portal",
        ],
        &[("BULLET_PORTAL_DIST", portal_dist)],
        commands,
    )?;
    invoke(
        plan,
        "bullet-kernel",
        &[
            "build",
            "--locked",
            "--release",
            "-p",
            "bullet",
            "-p",
            "bullet-effects",
            "-p",
            "bullet-mcpd",
            "-p",
            "bullet-runner",
            "-p",
            "bullet-verifier",
        ],
        &[],
        commands,
    )?;
    invoke(
        plan,
        "bullet-git",
        &["build", "--locked", "--release", "-p", "bullet-gitd"],
        &[],
        commands,
    )?;
    invoke(
        plan,
        "bullet-farm",
        &["build", "--locked", "--release", "-p", "bullet-family"],
        &[],
        commands,
    )?;

    let mut binaries = Vec::with_capacity(BINARIES.len());
    for (member, _, name) in BINARIES {
        let path = target_dir(plan, member).join("release").join(name);
        let metadata = std::fs::symlink_metadata(&path).map_err(CoordError::io)?;
        if !metadata.file_type().is_file() || metadata.len() == 0 {
            return Err(failed(format!(
                "{name} was not produced as a nonempty regular file at {}",
                path.display()
            )));
        }
        binaries.push(BuiltBinary {
            name: name.to_owned(),
            path,
        });
    }
    binaries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(binaries)
}

/// `cargo metadata --locked` for one member, as raw JSON bytes.
pub(super) fn metadata(
    plan: &BuildPlan,
    member: &str,
    commands: &mut Vec<RecordedCommand>,
) -> Result<Vec<u8>, CoordError> {
    let mut args = vec![
        "metadata".to_owned(),
        "--locked".to_owned(),
        "--format-version".to_owned(),
        "1".to_owned(),
        "--filter-platform".to_owned(),
        super::SUPPORTED_TARGET.to_owned(),
    ];
    if plan.offline {
        args.push("--offline".to_owned());
    }
    let output = run(plan, member, &args, &[], commands)?;
    Ok(output)
}

fn invoke(
    plan: &BuildPlan,
    member: &str,
    args: &[&str],
    extra_env: &[(&str, &str)],
    commands: &mut Vec<RecordedCommand>,
) -> Result<(), CoordError> {
    let mut owned = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    if plan.offline {
        owned.push("--offline".to_owned());
    }
    run(plan, member, &owned, extra_env, commands).map(|_| ())
}

fn run(
    plan: &BuildPlan,
    member: &str,
    args: &[String],
    extra_env: &[(&str, &str)],
    commands: &mut Vec<RecordedCommand>,
) -> Result<Vec<u8>, CoordError> {
    let subject = plan.member(member)?;
    let target = target_dir(plan, member);
    std::fs::create_dir_all(&target).map_err(CoordError::io)?;
    let tools = &plan.tools;
    let mut command = Command::new(&tools.cargo);
    command.args(args).current_dir(&subject.path);
    let mut env = base_env(tools, &target)?;
    for (name, value) in extra_env {
        env.push(((*name).to_owned(), (*value).to_owned()));
    }
    command.env_clear();
    for (name, value) in &env {
        command.env(name, value);
    }
    commands.push(RecordedCommand {
        program: tools.cargo.display().to_string(),
        args: args.to_vec(),
        cwd: subject.path.display().to_string(),
        env: env.clone(),
    });
    let output = run_bounded(&mut command, "release build cargo", CARGO_LIMITS)?;
    if !output.status.success() {
        return Err(failed(format!(
            "cargo {} in {member} exited {:?}: {}",
            args.join(" "),
            output.status.code(),
            tail(&[output.stderr.as_slice(), output.stdout.as_slice()].concat())
        )));
    }
    Ok(output.stdout)
}

fn base_env(tools: &Toolchain, target: &Path) -> Result<Vec<(String, String)>, CoordError> {
    let cargo_bin = tools
        .cargo
        .parent()
        .ok_or_else(|| failed("the admitted cargo has no parent directory"))?;
    let mut path = OsString::from(cargo_bin);
    path.push(":/usr/bin:/bin");
    let mut env = vec![
        (
            "PATH".to_owned(),
            path.to_str()
                .ok_or_else(|| failed("the build PATH is not UTF-8"))?
                .to_owned(),
        ),
        ("LC_ALL".to_owned(), "C".to_owned()),
        ("CARGO_TERM_COLOR".to_owned(), "never".to_owned()),
        (
            "CARGO_TARGET_DIR".to_owned(),
            target
                .to_str()
                .ok_or_else(|| failed("the build target directory is not UTF-8"))?
                .to_owned(),
        ),
        ("SOURCE_DATE_EPOCH".to_owned(), "0".to_owned()),
    ];
    for name in ["HOME", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN"] {
        if let Some(value) = std::env::var_os(name).and_then(|value| value.into_string().ok()) {
            env.push((name.to_owned(), value));
        }
    }
    env.sort();
    Ok(env)
}

fn target_dir(plan: &BuildPlan, member: &str) -> PathBuf {
    plan.cache.join("cargo-target").join(member)
}

/// The last bounded, control-stripped slice of a failed child's stderr.
pub(super) fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut kept = text
        .trim_end()
        .chars()
        .filter(|character| !character.is_control() || *character == '\n')
        .rev()
        .take(2048)
        .collect::<Vec<_>>();
    kept.reverse();
    kept.into_iter().collect()
}
