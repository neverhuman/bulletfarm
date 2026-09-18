use std::io::{Seek, Write};
use std::{collections::BTreeMap, fs, path::Path, process::Command, time::Duration};

use serde::Serialize;

use super::{
    DESTINATION, Result, decode, encode, git, read_manifest, require, scan,
    store::{self, Prepared, Request, Store},
};
use crate::{coord::CoordError, process};

#[derive(Serialize)]
struct Receipt<'a> {
    schema_version: &'static str,
    request_sha256: &'a str,
    aggregate_commit: &'a str,
    destination: &'a str,
    refs: BTreeMap<String, String>,
    integrated: bool,
}

pub(super) fn desired_refs(request: &Request, prepared: &Prepared) -> BTreeMap<String, String> {
    let mut refs = request
        .manifest
        .members
        .values()
        .map(|s| (s.source_ref.clone(), s.commit.clone()))
        .collect::<BTreeMap<_, _>>();
    refs.insert(
        prepared.review_ref.clone(),
        prepared.aggregate_commit.clone(),
    );
    refs
}

pub(super) fn read_remote(store: &Store, destination: &str) -> Result<BTreeMap<String, String>> {
    read_remote_authenticated(store, destination, None)
}

fn read_remote_authenticated(
    store: &Store,
    destination: &str,
    token: Option<&str>,
) -> Result<BTreeMap<String, String>> {
    let mut command = git::command(&store.objects);
    if let Some(token) = token {
        authenticate_git(&mut command, destination, token)?;
    }
    let output = String::from_utf8(git::execute(
        command.args([
            "ls-remote",
            "--refs",
            destination,
            "refs/heads/main",
            "refs/heads/publication/*",
            "refs/tags/bullet-source/v1/*",
        ]),
        None,
    )?)
    .map_err(|_| CoordError::new("PUBLICATION_REMOTE_INVALID", "non-UTF-8 refs"))?;
    let mut refs = BTreeMap::new();
    for line in output.lines() {
        let (commit, name) = line
            .split_once('\t')
            .ok_or_else(|| CoordError::new("PUBLICATION_REMOTE_INVALID", "invalid ref response"))?;
        super::oid(commit)?;
        require(
            refs.insert(name.into(), commit.into()).is_none(),
            "PUBLICATION_REMOTE_INVALID",
        )?;
    }
    Ok(refs)
}

pub(super) fn jeryu_token() -> Result<String> {
    let token = std::env::var("BULLET_PUBLICATION_TOKEN").map_err(|_| {
        CoordError::new(
            "PUBLICATION_JERYU_TOKEN_REQUIRED",
            "provide a JeRyu token authorized for root/bulletfarm",
        )
    })?;
    require(
        !token.is_empty()
            && token.len() <= 4096
            && !token
                .bytes()
                .any(|b| b.is_ascii_whitespace() || b.is_ascii_control()),
        "PUBLICATION_TOKEN_INVALID",
    )?;
    Ok(token)
}

pub(super) fn authenticate_git(
    command: &mut Command,
    destination: &str,
    token: &str,
) -> Result<()> {
    require(
        matches!(destination, DESTINATION | super::JERYU_DESTINATION),
        "PUBLICATION_DESTINATION_MISMATCH",
    )?;
    let header = if destination == super::JERYU_DESTINATION {
        format!("Authorization: Bearer {token}")
    } else {
        format!(
            "Authorization: Basic {}",
            base64(format!("x-access-token:{token}").as_bytes())
        )
    };
    // Exact URL scope, no redirects, and no secret in argv or retained output.
    command
        .env("GIT_CONFIG_COUNT", "2")
        .env(
            "GIT_CONFIG_KEY_0",
            format!("http.{destination}.extraheader"),
        )
        .env("GIT_CONFIG_VALUE_0", header)
        .env("GIT_CONFIG_KEY_1", "http.followRedirects")
        .env("GIT_CONFIG_VALUE_1", "false");
    Ok(())
}

pub(super) fn gh_executable(path: &Path) -> Result<(std::fs::File, std::path::PathBuf)> {
    use nix::{
        fcntl::{FcntlArg, FdFlag, SealFlag, fcntl},
        sys::memfd::{MemFdCreateFlag, memfd_create},
    };
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Write},
        os::{
            fd::AsRawFd,
            unix::fs::{OpenOptionsExt, PermissionsExt},
        },
    };
    require(path.is_absolute(), "PUBLICATION_GH_PATH_INVALID")?;
    let source = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(CoordError::io)?;
    require(
        source.metadata().map_err(CoordError::io)?.is_file(),
        "PUBLICATION_GH_SUBSTITUTED",
    )?;
    let mut bytes = Vec::new();
    source
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(
        bytes.len() <= 64 * 1024 * 1024
            && store::digest(&bytes)
                == "d2330508768dbbaa4c474353c77367e1690b1fe08c81497f787e40f9f53564d4",
        "PUBLICATION_GH_SUBSTITUTED",
    )?;
    let descriptor = memfd_create(c"bullet-publication-gh", MemFdCreateFlag::MFD_ALLOW_SEALING)
        .map_err(|_| CoordError::new("PUBLICATION_GH_PIN_FAILED", "memfd unavailable"))?;
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
    .map_err(|_| CoordError::new("PUBLICATION_GH_PIN_FAILED", "sealing failed"))?;
    let retained =
        File::open(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(CoordError::io)?;
    fcntl(retained.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty())).map_err(|_| {
        CoordError::new("PUBLICATION_GH_PIN_FAILED", "descriptor inheritance failed")
    })?;
    let retained_path = format!("/proc/self/fd/{}", retained.as_raw_fd()).into();
    Ok((retained, retained_path))
}

pub(super) fn api_command(
    executable: &Path,
    config: &Path,
    token: &str,
    method: &str,
    endpoint: &str,
) -> Command {
    let mut command = Command::new(executable);
    command
        .env_clear()
        .current_dir(config)
        .env("PATH", "/usr/bin:/bin")
        .env("GH_CONFIG_DIR", config)
        .env("GH_TOKEN", token)
        .env("GH_PROMPT_DISABLED", "1")
        .args([
            "api",
            "--hostname",
            "github.com",
            "--method",
            method,
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            endpoint,
        ]);
    command
}

pub(super) fn api(
    token: &str,
    method: &str,
    endpoint: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value> {
    let path = std::env::var_os("BULLET_PUBLICATION_GH").ok_or_else(|| {
        CoordError::new(
            "PUBLICATION_GH_REQUIRED",
            "set BULLET_PUBLICATION_GH to the admitted absolute gh2.62.0 executable",
        )
    })?;
    let (_subject, executable) = gh_executable(Path::new(&path))?;
    let config = tempfile::tempdir().map_err(CoordError::io)?;
    let mut command = api_command(&executable, config.path(), token, method, endpoint);
    let limits = process::Limits {
        timeout: Duration::from_secs(30),
        stdout_bytes: 1024 * 1024,
        stderr_bytes: 4096,
    };
    let output = if let Some(body) = body {
        let mut input = tempfile::tempfile().map_err(CoordError::io)?;
        input.write_all(&encode(body)?).map_err(CoordError::io)?;
        input.rewind().map_err(CoordError::io)?;
        command.args(["--input", "-"]);
        process::run_bounded_with_input_file(&mut command, "publication GitHub API", limits, input)?
            .output
    } else {
        process::run_bounded(&mut command, "publication GitHub API", limits)?
    };
    require(
        output.status.success(),
        "PUBLICATION_API_OUTCOME_UNAVAILABLE",
    )?;
    decode(&output.stdout)
}

pub(super) fn app_token() -> Result<String> {
    let token = std::env::var("BULLET_PUBLICATION_TOKEN").map_err(|_| CoordError::new("PUBLICATION_APP_TOKEN_REQUIRED", "provide a repository-scoped GitHub App installation token with contents, workflows and pull_requests write"))?;
    require(
        !token.is_empty()
            && token.len() <= 4096
            && !token
                .bytes()
                .any(|b| b.is_ascii_whitespace() || b.is_ascii_control()),
        "PUBLICATION_TOKEN_INVALID",
    )?;
    let value = api(
        &token,
        "GET",
        "/installation/repositories?per_page=100",
        None,
    )?;
    require(
        value["total_count"] == 1
            && value["repositories"]
                .as_array()
                .is_some_and(|a| a.len() == 1)
            && value["repositories"][0]["full_name"] == "neverhuman/bulletfarm",
        "PUBLICATION_REPOSITORY_SCOPE_REQUIRED",
    )?;
    Ok(token)
}

pub(super) fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let word = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(char::from(ALPHABET[((word >> 18) & 63) as usize]));
        encoded.push(char::from(ALPHABET[((word >> 12) & 63) as usize]));
        encoded.push(if chunk.len() > 1 {
            char::from(ALPHABET[((word >> 6) & 63) as usize])
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            char::from(ALPHABET[(word & 63) as usize])
        } else {
            '='
        });
    }
    encoded
}

pub(super) fn publish_to(
    store: &Store,
    request: &Request,
    prepared: &Prepared,
    destination: &str,
    token: Option<&str>,
) -> Result<Vec<u8>> {
    let desired = desired_refs(request, prepared);
    let remote = read_remote_authenticated(store, destination, token)?;
    for (name, commit) in &desired {
        require(
            remote.get(name).is_none_or(|actual| actual == commit),
            "PUBLICATION_REF_CONFLICT",
        )?;
    }
    let complete = desired
        .iter()
        .all(|(name, commit)| remote.get(name) == Some(commit));
    if !complete {
        require(
            if request.bootstrap()? {
                !remote.contains_key("refs/heads/main")
            } else {
                remote.get("refs/heads/main") == Some(&request.expected_main)
            },
            "PUBLICATION_BASE_CHANGED",
        )?;
        let mut command = git::command(&store.objects);
        command.args(["push", "--atomic", "--porcelain"]);
        for name in desired.keys() {
            command.arg(format!(
                "--force-with-lease={name}:{}",
                remote.get(name).map(String::as_str).unwrap_or_default()
            ));
        }
        if let Some(token) = token {
            authenticate_git(&mut command, destination, token)?;
        }
        command.arg(destination);
        for (name, commit) in &desired {
            command.arg(format!("{commit}:{name}"));
        }
        // A lost response does not authorize a new request. Read remote truth even on failure.
        let outcome = git::execute(&mut command, None);
        let observed = read_remote_authenticated(store, destination, token)?;
        if !desired
            .iter()
            .all(|(name, commit)| observed.get(name) == Some(commit))
        {
            return Err(CoordError::new(
                "PUBLICATION_OUTCOME_UNKNOWN",
                format!(
                    "exact request retained; push acknowledged={}, reconcile original request",
                    outcome.is_ok()
                ),
            ));
        }
    }
    let observed = read_remote_authenticated(store, destination, token)?;
    require(
        desired
            .iter()
            .all(|(name, commit)| observed.get(name) == Some(commit)),
        "PUBLICATION_REMOTE_DRIFT",
    )?;
    encode(&Receipt {
        schema_version: "bullet.publication-receipt.v1",
        request_sha256: &prepared.request_sha256,
        aggregate_commit: &prepared.aggregate_commit,
        destination,
        refs: desired,
        integrated: false,
    })
}

pub(super) fn push(path: &Path, id: &str) -> Result<String> {
    let store = Store::open(path)?;
    let (request, prepared) = store.load(id)?;
    scan::scan(&store, &request, &prepared)?;
    let destination = &request.manifest.tool_config.destination;
    let token = if destination == super::JERYU_DESTINATION {
        jeryu_token()?
    } else {
        app_token()?
    };
    let receipt = publish_to(&store, &request, &prepared, destination, Some(&token))?;
    store::persist(&store.path(id, "receipt"), &receipt)?;
    String::from_utf8(receipt)
        .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}

pub(super) fn reconstruct_from(aggregate: &Path, root: &Path, destination: &str) -> Result<String> {
    let manifest = read_manifest(aggregate)?;
    let aggregate_commit = git::text(aggregate, &["rev-parse", "HEAD"])?;
    let parent = root
        .parent()
        .ok_or_else(|| CoordError::new("PUBLICATION_ROOT_INVALID", "parent required"))?;
    git::canonical_directory(parent)?;
    require(
        !root.exists() && fs::symlink_metadata(root).is_err(),
        "PUBLICATION_ROOT_EXISTS",
    )?;
    fs::create_dir(root).map_err(CoordError::io)?;
    // This is an admitted disposable CI root. Partial reconstructions are retained
    // for diagnosis; retries use a fresh runner root and never replace checkouts.
    for (name, subject) in &manifest.members {
        let repo = root.join(name);
        git::bytes(root, &["init", "--template=", name])?;
        git::bytes(
            &repo,
            &["fetch", "--no-tags", destination, &subject.source_ref],
        )?;
        require(
            git::text(&repo, &["rev-parse", "FETCH_HEAD"])? == subject.commit,
            "PUBLICATION_SOURCE_REF_DRIFT",
        )?;
        require(
            git::text(&repo, &["rev-parse", "--show-object-format"])? == subject.object_format,
            "PUBLICATION_OBJECT_FORMAT_MISMATCH",
        )?;
        git::bytes(&repo, &["fsck", "--full", "--strict", "--no-dangling"])?;
        git::bytes(&repo, &["checkout", "--detach", &subject.commit])?;
        git::checkout(&repo)?;
        require(
            git::text(&repo, &["rev-parse", "HEAD^{tree}"])? == subject.tree,
            "PUBLICATION_TREE_MISMATCH",
        )?;
    }
    let portable = git::blob(aggregate, "HEAD", "repos.manifest.toml")?;
    store::persist(&root.join("repos.manifest.toml"), &portable)?;
    crate::family_lock::validate_publication_family_root(root)?;
    require(
        read_manifest(aggregate)? == manifest
            && git::text(aggregate, &["rev-parse", "HEAD"])? == aggregate_commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    Ok(format!(
        "reconstructed aggregate {aggregate_commit} into {}",
        root.display()
    ))
}

pub(super) fn reconstruct(aggregate: &Path, root: &Path) -> Result<String> {
    require(
        read_manifest(aggregate)?.tool_config.destination == DESTINATION,
        "PUBLICATION_JERYU_RECONSTRUCTION_UNAVAILABLE",
    )?;
    require(
        std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true"),
        "PUBLICATION_DISPOSABLE_CI_REQUIRED",
    )?;
    let runner = std::env::var_os("RUNNER_TEMP").ok_or_else(|| {
        CoordError::new("PUBLICATION_DISPOSABLE_CI_REQUIRED", "RUNNER_TEMP required")
    })?;
    let runner = Path::new(&runner);
    git::canonical_directory(runner)?;
    require(
        root.parent() == Some(runner),
        "PUBLICATION_RUNNER_ROOT_INVALID",
    )?;
    reconstruct_from(aggregate, root, DESTINATION)
}
