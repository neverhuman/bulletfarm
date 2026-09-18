//! Scan every reachable object's raw contents; synthetic scan objects stay local.
//! Receipts record bounded pinned-rule execution and confer no release authority.
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
mod objects;
#[cfg(test)]
use objects::{Object, batch, hash_blob};
use objects::{inventory, synthetic, validate_limits};

use super::store::{Prepared, Request, Store, digest, persist};
use super::{Result, decode, encode, git, oid, require};
use crate::{coord::CoordError, process};

const SCANNER_SHA256: &str = "50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359";
const CONFIG_PATH: &str = "publication/gitleaks.toml";
const MAX_OBJECTS: usize = 100_000;
const MAX_OBJECT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const BATCH_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ScanSubject {
    schema_version: String,
    request_sha256: String,
    aggregate_commit: String,
    scanner_sha256: String,
    config_sha256: String,
    inventory_sha256: String,
    raw_objects_sha256: String,
    object_count: usize,
    object_bytes: u64,
    synthetic_commit: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    subject: ScanSubject,
    attempt: String,
    report_sha256: String,
    log_sha256: String,
}

fn validate_config(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_CONFIG_INVALID", "UTF-8 required"))?;
    // Rule and allowlist authority is the exact reviewed Hub commit, never ambient scanner policy.
    let value: toml::Table = toml::from_str(text)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_CONFIG_INVALID", "invalid TOML"))?;
    let extension = value.get("extend").and_then(toml::Value::as_table);
    require(
        extension.is_some_and(|e| {
            e.len() == 1 && e.get("useDefault").and_then(toml::Value::as_bool) == Some(true)
        }),
        "PUBLICATION_SCAN_CONFIG_EXTERNAL_RULES",
    )
}

fn bounded_file(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    require(
        fs::symlink_metadata(path)
            .map_err(CoordError::io)?
            .is_file(),
        "PUBLICATION_SCAN_FILE_INVALID",
    )?;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(CoordError::io)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(bytes.len() as u64 <= maximum, "PUBLICATION_SCAN_FILE_LIMIT")?;
    Ok(bytes)
}

struct Scanner {
    _file: File,
    path: PathBuf,
}

#[cfg(target_os = "linux")]
fn pin_scanner(path: &Path) -> Result<Scanner> {
    use nix::{
        fcntl::{FcntlArg, FdFlag, SealFlag, fcntl},
        sys::memfd::{MemFdCreateFlag, memfd_create},
    };
    use std::os::{fd::AsRawFd, unix::fs::PermissionsExt};
    require(path.is_absolute(), "PUBLICATION_SCANNER_PATH_INVALID")?;
    let bytes = bounded_file(path, 128 * 1024 * 1024)?;
    require(
        digest(&bytes) == SCANNER_SHA256,
        "PUBLICATION_SCANNER_SUBSTITUTED",
    )?;
    let descriptor = memfd_create(
        c"bullet-publication-gitleaks",
        MemFdCreateFlag::MFD_ALLOW_SEALING,
    )
    .map_err(|_| CoordError::new("PUBLICATION_SCANNER_PIN_FAILED", "memfd unavailable"))?;
    let mut file = File::from(descriptor);
    file.write_all(&bytes).map_err(CoordError::io)?;
    file.set_permissions(fs::Permissions::from_mode(0o500))
        .map_err(CoordError::io)?;
    fcntl(
        file.as_raw_fd(),
        FcntlArg::F_ADD_SEALS(
            SealFlag::F_SEAL_WRITE
                | SealFlag::F_SEAL_GROW
                | SealFlag::F_SEAL_SHRINK
                | SealFlag::F_SEAL_SEAL,
        ),
    )
    .map_err(|_| CoordError::new("PUBLICATION_SCANNER_PIN_FAILED", "sealing failed"))?;
    let retained =
        File::open(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(CoordError::io)?;
    fcntl(retained.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty())).map_err(|_| {
        CoordError::new(
            "PUBLICATION_SCANNER_PIN_FAILED",
            "descriptor inheritance failed",
        )
    })?;
    Ok(Scanner {
        path: PathBuf::from(format!("/proc/self/fd/{}", retained.as_raw_fd())),
        _file: retained,
    })
}

#[cfg(not(target_os = "linux"))]
fn pin_scanner(_path: &Path) -> Result<Scanner> {
    Err(CoordError::new(
        "PUBLICATION_SCAN_PLATFORM_UNSUPPORTED",
        "sealed Linux scanner execution required",
    ))
}

fn scanner_command(
    scanner: &Scanner,
    repo: &Path,
    commit: &str,
    config: &Path,
    report: &Path,
    ignore: &Path,
) -> Command {
    let mut command = Command::new(&scanner.path);
    for secret in [
        "BULLET_PUBLICATION_TOKEN",
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "GH_ENTERPRISE_TOKEN",
        "GITHUB_ENTERPRISE_TOKEN",
    ] {
        command.env_remove(secret);
    }
    for (key, _) in std::env::vars_os() {
        let key_text = key.to_string_lossy();
        if key_text.starts_with("GIT_") || key_text.starts_with("GITLEAKS_") {
            command.env_remove(key);
        }
    }
    command
        .current_dir(ignore)
        .env_remove("GITLEAKS_CONFIG")
        .env_remove("GITLEAKS_CONFIG_TOML")
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .args(["git", "--config"])
        .arg(config)
        .arg("--gitleaks-ignore-path")
        .arg(ignore)
        .args([
            "--ignore-gitleaks-allow",
            "--redact=100",
            "--no-color",
            "--no-banner",
            "--report-format=json",
            "--report-path",
        ])
        .arg(report)
        .arg("--log-opts")
        .arg(format!(
            "--root --text --no-renames --no-ext-diff --no-textconv {commit} --"
        ))
        .arg(repo);
    command
}

fn empty_report(bytes: &[u8]) -> Result<()> {
    let value = bullet_wire::decode_unique_value_bounded(bytes, 1024 * 1024)
        .map_err(|_| CoordError::new("PUBLICATION_SCAN_REPORT_INVALID", "malformed report"))?;
    require(
        value.as_array().is_some_and(Vec::is_empty),
        "PUBLICATION_HISTORY_SECRET_FINDINGS",
    )
}

pub(super) fn scan(store: &Store, request: &Request, prepared: &Prepared) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(1200);
    let config = git::blob(
        &store.objects,
        &request.manifest.members["bullet-farm"].commit,
        CONFIG_PATH,
    )?;
    validate_config(&config)?;
    let scanner_path = std::env::var_os("BULLET_PUBLICATION_GITLEAKS")
        .map(PathBuf::from)
        .ok_or_else(|| {
            CoordError::new(
                "PUBLICATION_SCANNER_REQUIRED",
                "set BULLET_PUBLICATION_GITLEAKS to the pinned absolute executable",
            )
        })?;
    let scanner = pin_scanner(&scanner_path)?;
    let mut heads = request
        .manifest
        .members
        .values()
        .map(|s| s.commit.clone())
        .collect::<Vec<_>>();
    heads.push(prepared.aggregate_commit.clone());
    let objects = inventory(&store.objects, &heads)?;
    let inventory_bytes = encode(&objects)?;
    let (synthetic_commit, raw_objects_sha256) = synthetic(&store.objects, &objects, deadline)?;
    let subject = ScanSubject {
        schema_version: "bullet.publication-scan.v1".into(),
        request_sha256: prepared.request_sha256.clone(),
        aggregate_commit: prepared.aggregate_commit.clone(),
        scanner_sha256: SCANNER_SHA256.into(),
        config_sha256: digest(&config),
        inventory_sha256: digest(&inventory_bytes),
        raw_objects_sha256,
        object_count: objects.len(),
        object_bytes: validate_limits(&objects)?,
        synthetic_commit,
    };
    let id = &request.request_id;
    let receipt_path = store.path(id, "scan");
    if receipt_path.try_exists().map_err(CoordError::io)? {
        let receipt: Receipt = decode(&bounded_file(&receipt_path, 1024 * 1024)?)?;
        require(receipt.subject == subject, "PUBLICATION_SCAN_RECEIPT_DRIFT")?;
        require(
            bounded_file(&store.path(id, "scan-inventory"), 16 * 1024 * 1024)? == inventory_bytes,
            "PUBLICATION_SCAN_ARTIFACT_DRIFT",
        )?;
        require(
            receipt.attempt.starts_with("scan-attempt-")
                && receipt
                    .attempt
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
            "PUBLICATION_SCAN_ARTIFACT_DRIFT",
        )?;
        let attempt = store.root.join(&receipt.attempt);
        git::canonical_directory(&attempt)?;
        let report = bounded_file(&attempt.join("report.json"), 1024 * 1024)?;
        let log = bounded_file(&attempt.join("execution.json"), 4 * 1024 * 1024)?;
        require(
            digest(&report) == receipt.report_sha256 && digest(&log) == receipt.log_sha256,
            "PUBLICATION_SCAN_ARTIFACT_DRIFT",
        )?;
        empty_report(&report)?;
        return Ok(());
    }
    persist(&store.path(id, "scan-inventory"), &inventory_bytes)?;
    // This ref is local custody only; transport pushes an exact source/review allowlist.
    git::bytes(
        &store.objects,
        &[
            "update-ref",
            &format!("refs/publication-scans/{id}"),
            &subject.synthetic_commit,
        ],
    )?;
    let attempt = tempfile::Builder::new()
        .prefix("scan-attempt-")
        .tempdir_in(&store.root)
        .map_err(CoordError::io)?
        .into_path();
    let config_path = attempt.join("gitleaks.toml");
    persist(&config_path, &config)?;
    let ignore = attempt.join("empty-ignore");
    fs::create_dir(&ignore).map_err(CoordError::io)?;
    let report_path = attempt.join("report.json");
    let mut command = scanner_command(
        &scanner,
        &store.objects,
        &subject.synthetic_commit,
        &config_path,
        &report_path,
        &ignore,
    );
    require(Instant::now() < deadline, "PUBLICATION_SCAN_DEADLINE")?;
    let output = process::run_bounded(
        &mut command,
        "publication secret scan",
        process::Limits {
            timeout: deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(300)),
            stdout_bytes: 1024 * 1024,
            stderr_bytes: 1024 * 1024,
        },
    )?;
    let log = encode(
        &serde_json::json!({"exit_code":output.status.code(),"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)}),
    )?;
    persist(&attempt.join("execution.json"), &log)?;
    require(output.status.success(), "PUBLICATION_HISTORY_SCAN_REFUSED")?;
    let report = bounded_file(&report_path, 1024 * 1024)?;
    empty_report(&report)?;
    File::open(&report_path)
        .and_then(|file| file.sync_all())
        .map_err(CoordError::io)?;
    File::open(&attempt)
        .and_then(|file| file.sync_all())
        .map_err(CoordError::io)?;
    File::open(&store.root)
        .and_then(|file| file.sync_all())
        .map_err(CoordError::io)?;
    let attempt = attempt
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CoordError::new("PUBLICATION_SCAN_ARTIFACT_DRIFT", "invalid attempt path"))?
        .to_owned();
    let receipt = Receipt {
        subject,
        attempt,
        report_sha256: digest(&report),
        log_sha256: digest(&log),
    };
    persist(&receipt_path, &encode(&receipt)?)
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
