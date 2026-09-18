//! One durable App-authored review request; human integration stays external.
use std::{fs::OpenOptions, io::Read, os::unix::fs::OpenOptionsExt, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::store::{Prepared, Request, Store, digest, persist};
use super::{DESTINATION, Result, decode, encode, require, scan, transport};
use crate::coord::CoordError;

const REPOSITORY: &str = "neverhuman/bulletfarm";
const PULLS: &str = "/repos/neverhuman/bulletfarm/pulls";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema_version: String,
    request_sha256: String,
    aggregate_commit: String,
    head: String,
    base: String,
    title: String,
    body: String,
    draft: bool,
    maintainer_can_modify: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: String,
    request_sha256: String,
    intent_sha256: String,
    number: u64,
    url: String,
    head_sha: String,
    actor_id: u64,
    actor_login: String,
    repository_id: u64,
}

fn intent(request: &Request, prepared: &Prepared) -> Intent {
    Intent {
        schema_version: "bullet.publication-pr-intent.v1".into(),
        request_sha256: prepared.request_sha256.clone(),
        aggregate_commit: prepared.aggregate_commit.clone(),
        head: format!("publication/{}", request.request_id),
        base: "main".into(),
        draft: false,
        maintainer_can_modify: false,
        title: format!("Publish reviewed Bullet family: {}", request.request_id),
        body: format!(
            "Publish exact member sources and generated aggregate files for human review.\n\nAggregate: `{}`\n\nThis request grants no integration or release authority.\n\n<!-- bullet-publication-request-sha256:{} -->\n",
            prepared.aggregate_commit, prepared.request_sha256
        ),
    }
}

fn read_record(path: &Path) -> Result<Option<Vec<u8>>> {
    let file = match OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CoordError::io(error)),
    };
    require(
        file.metadata().map_err(CoordError::io)?.is_file(),
        "PUBLICATION_PR_RECORD_INVALID",
    )?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(bytes.len() <= 1024 * 1024, "PUBLICATION_PR_RECORD_INVALID")?;
    Ok(Some(bytes))
}

fn verify_publication(store: &Store, request: &Request, prepared: &Prepared) -> Result<()> {
    let expected = transport::desired_refs(request, prepared);
    let bytes = read_record(&store.path(&request.request_id, "receipt"))?.ok_or_else(|| {
        CoordError::new(
            "PUBLICATION_PR_PUBLICATION_REQUIRED",
            "publish and retain the exact ref receipt first",
        )
    })?;
    let receipt: Value = decode(&bytes)?;
    require(
        receipt
            == serde_json::json!({
                "schema_version":"bullet.publication-receipt.v1", "request_sha256":prepared.request_sha256,
                "aggregate_commit":prepared.aggregate_commit, "destination":DESTINATION, "refs":expected, "integrated":false
            }),
        "PUBLICATION_PR_PUBLICATION_DRIFT",
    )?;
    let remote = transport::read_remote(store, DESTINATION)?;
    require(
        expected
            .iter()
            .all(|(name, oid)| remote.get(name) == Some(oid)),
        "PUBLICATION_PR_REMOTE_DRIFT",
    )
}

fn validate_pull(value: &Value, intent: &Intent) -> Result<Receipt> {
    let number = value["number"].as_u64().filter(|n| *n > 0).ok_or_else(|| {
        CoordError::new(
            "PUBLICATION_PR_RESPONSE_INVALID",
            "positive PR number required",
        )
    })?;
    let actor_id = value["user"]["id"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            CoordError::new(
                "PUBLICATION_PR_AUTHOR_INVALID",
                "positive Bot identity required",
            )
        })?;
    let actor_login = value["user"]["login"].as_str().unwrap_or_default();
    require(
        value["user"]["type"] == "Bot"
            && actor_login.ends_with("[bot]")
            && (6..=100).contains(&actor_login.len()),
        "PUBLICATION_PR_AUTHOR_INVALID",
    )?;
    let repository_id = value["base"]["repo"]["id"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            CoordError::new(
                "PUBLICATION_PR_REPOSITORY_INVALID",
                "repository identity required",
            )
        })?;
    require(
        value["head"]["repo"]["id"] == repository_id
            && value["head"]["repo"]["full_name"] == REPOSITORY
            && value["base"]["repo"]["full_name"] == REPOSITORY,
        "PUBLICATION_PR_REPOSITORY_INVALID",
    )?;
    require(
        value["head"]["ref"] == intent.head
            && value["head"]["sha"] == intent.aggregate_commit
            && value["base"]["ref"] == intent.base,
        "PUBLICATION_PR_SUBJECT_DRIFT",
    )?;
    require(
        value["body"] == intent.body && value["title"] == intent.title,
        "PUBLICATION_PR_REQUEST_CONFLICT",
    )?;
    // Draft/maintainer settings are creation defaults; later human changes do not change the subject.
    require(
        value["state"] == "open" || value["state"] == "closed",
        "PUBLICATION_PR_RESPONSE_INVALID",
    )?;
    let url = format!("https://github.com/{REPOSITORY}/pull/{number}");
    require(value["html_url"] == url, "PUBLICATION_PR_RESPONSE_INVALID")?;
    Ok(Receipt {
        schema_version: "bullet.publication-pr-receipt.v1".into(),
        request_sha256: intent.request_sha256.clone(),
        intent_sha256: digest(&encode(intent)?),
        number,
        url,
        head_sha: intent.aggregate_commit.clone(),
        actor_id,
        actor_login: actor_login.into(),
        repository_id,
    })
}

fn lookup(
    api: &mut impl FnMut(&str, &str, Option<&Value>) -> Result<Value>,
    intent: &Intent,
) -> Result<Option<Value>> {
    // More than one match is a conflict. An accepted 0/1 result is below this page's 100-item limit.
    let endpoint = format!(
        "{PULLS}?state=all&head=neverhuman%3A{}&base=main&per_page=100&page=1",
        intent.head.replace('/', "%2F")
    );
    let value = api("GET", &endpoint, None)?;
    let values = value
        .as_array()
        .ok_or_else(|| CoordError::new("PUBLICATION_PR_RESPONSE_INVALID", "array required"))?;
    require(values.len() <= 1, "PUBLICATION_PR_MULTIPLE_MATCHES")?;
    Ok(values.first().cloned())
}

fn read_back(
    api: &mut impl FnMut(&str, &str, Option<&Value>) -> Result<Value>,
    candidate: &Value,
    intent: &Intent,
) -> Result<Receipt> {
    let observed = validate_pull(candidate, intent)?;
    let current = api("GET", &format!("{PULLS}/{}", observed.number), None)?;
    require(
        validate_pull(&current, intent)? == observed,
        "PUBLICATION_PR_READBACK_DRIFT",
    )?;
    Ok(observed)
}

fn perform(
    store: &Store,
    request: &Request,
    prepared: &Prepared,
    mut api: impl FnMut(&str, &str, Option<&Value>) -> Result<Value>,
    mut verify_remote: impl FnMut() -> Result<()>,
) -> Result<Vec<u8>> {
    let intent = intent(request, prepared);
    let id = &request.request_id;
    let intent_bytes = encode(&intent)?;
    persist(&store.path(id, "pr-intent"), &intent_bytes)?;
    verify_remote()?;
    let receipt_path = store.path(id, "pr-receipt");
    if let Some(bytes) = read_record(&receipt_path)? {
        let saved: Receipt = decode(&bytes)?;
        require(
            saved.schema_version == "bullet.publication-pr-receipt.v1"
                && saved.intent_sha256 == digest(&intent_bytes),
            "PUBLICATION_PR_RECEIPT_DRIFT",
        )?;
        let current = api("GET", &format!("{PULLS}/{}", saved.number), None)?;
        require(
            validate_pull(&current, &intent)? == saved,
            "PUBLICATION_PR_RECEIPT_DRIFT",
        )?;
        verify_remote()?;
        return Ok(bytes);
    }
    let attempt_bytes = encode(
        &serde_json::json!({"schema_version":"bullet.publication-pr-attempt.v1","intent_sha256":digest(&intent_bytes)}),
    )?;
    let attempted = read_record(&store.path(id, "pr-attempt"))?;
    if let Some(bytes) = &attempted {
        require(*bytes == attempt_bytes, "PUBLICATION_PR_REQUEST_CONFLICT")?;
    }
    let candidate = match lookup(&mut api, &intent)? {
        Some(value) => value,
        None => {
            require(attempted.is_none(), "PUBLICATION_PR_OUTCOME_UNKNOWN")?;
            verify_remote()?;
            // After this fsync, every retry is read-only, including death before the POST started.
            persist(&store.path(id, "pr-attempt"), &attempt_bytes)?;
            let body = serde_json::json!({"title":intent.title,"body":intent.body,"head":intent.head,"base":intent.base,"draft":intent.draft,"maintainer_can_modify":intent.maintainer_can_modify});
            match api("POST", PULLS, Some(&body)) {
                Ok(value) => value,
                Err(_) => lookup(&mut api, &intent).ok().flatten().ok_or_else(|| {
                    CoordError::new(
                        "PUBLICATION_PR_OUTCOME_UNKNOWN",
                        "creation may have occurred; only reconcile the original request",
                    )
                })?,
            }
        }
    };
    let receipt = read_back(&mut api, &candidate, &intent)?;
    verify_remote()?;
    let bytes = encode(&receipt)?;
    persist(&receipt_path, &bytes)?;
    Ok(bytes)
}

pub(super) fn run(path: &Path, id: &str) -> Result<String> {
    let store = Store::open(path)?;
    let (request, prepared) = store.load(id)?;
    scan::scan(&store, &request, &prepared)?;
    let token = transport::app_token()?;
    let bytes = perform(
        &store,
        &request,
        &prepared,
        |method, endpoint, body| transport::api(&token, method, endpoint, body),
        || verify_publication(&store, &request, &prepared),
    )?;
    String::from_utf8(bytes).map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}

#[cfg(test)]
mod tests;
