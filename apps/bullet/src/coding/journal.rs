//! Durable request identity precedes all submission network effects.

use super::{credentials::Session, SubmitRequest};
use bullet_application::CommandRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    farmd: String,
    origin: String,
    envelope: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    request_bytes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub(super) struct RequestBody {
    pub(super) bytes: String,
    pub(super) provenance: &'static str,
}

impl Journal {
    fn body(&self) -> Result<RequestBody, String> {
        let (bytes, provenance) = match (self.schema_version, &self.request_bytes) {
            (1, None) => (
                serde_json::to_string(&self.envelope).map_err(|_| "COMMAND_JOURNAL_CORRUPT")?,
                "RECONSTRUCTED_V1",
            ),
            (2, Some(bytes)) => (bytes.clone(), "RECORDED_EXACT"),
            _ => return Err("COMMAND_JOURNAL_CORRUPT".into()),
        };
        let decoded = serde_json::from_str::<bullet_harness_core::strict_json::StrictJson>(&bytes)
            .map_err(|_| "COMMAND_JOURNAL_CORRUPT")?
            .0;
        if decoded != self.envelope {
            return Err("COMMAND_JOURNAL_CORRUPT".into());
        }
        Ok(RequestBody { bytes, provenance })
    }
}

#[cfg(unix)]
pub(super) fn prepare(
    session: &Session,
    input: &SubmitRequest<'_>,
) -> Result<(RequestBody, CommandRequest), String> {
    use crate::auth::store::CredentialStore;
    let key = match input.idempotency_key {
        Some(key) => key.to_owned(),
        None => format!("cli_{}", super::random_hex(16)?),
    };
    if key.is_empty() || key.len() > 256 || key.chars().any(char::is_control) {
        return Err("COMMAND_KEY_INVALID: require 1..=256 non-control UTF-8 bytes".into());
    }
    let id = bullet_domain::CommandId::from_seed(&key);
    let store = CredentialStore::open(&session.directory)?;
    let record = match store.load_command(&id)? {
        Some(value) => {
            let recorded: Journal =
                serde_json::from_value(value).map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
            validate_retry(&recorded, session, input, &key)?;
            recorded
        }
        None => {
            input
                .payload
                .validate()
                .map_err(|_| "COMMAND_TASK_INVALID")?;
            let proposed = serde_json::json!({"idempotency_key":key,"kind":"run_coding","payload":input.payload});
            CommandRequest::new(&key, "run_coding", &proposed["payload"])
                .map_err(|_| "COMMAND_REQUEST_INVALID")?;
            let record = Journal {
                schema_version: 2,
                farmd: session.farmd.clone(),
                origin: session.origin.clone(),
                request_bytes: Some(
                    serde_json::to_string(&proposed)
                        .map_err(|_| "COMMAND_JOURNAL_ENCODING_FAILED")?,
                ),
                envelope: proposed,
            };
            store.record_command(
                &id,
                &serde_json::to_value(&record).map_err(|_| "COMMAND_JOURNAL_ENCODING_FAILED")?,
            )?;
            record
        }
    };
    let request = CommandRequest::new(&key, "run_coding", &record.envelope["payload"])
        .map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
    Ok((record.body()?, request))
}

#[cfg(not(unix))]
pub(super) fn prepare(
    _session: &Session,
    _input: &SubmitRequest<'_>,
) -> Result<(RequestBody, CommandRequest), String> {
    Err("AUTH_PRIVATE_STORE_UNSUPPORTED".into())
}

#[cfg(unix)]
fn validate_retry(
    record: &Journal,
    session: &Session,
    input: &SubmitRequest<'_>,
    key: &str,
) -> Result<(), String> {
    let _: crate::client::models::CommandEnvelope =
        crate::client::decode(&record.envelope).map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
    record.body()?;
    if record.farmd != session.farmd
        || record.origin != session.origin
        || record.envelope["idempotency_key"] != key
        || record.envelope["kind"] != "run_coding"
        || record.envelope["payload"]
            != serde_json::to_value(input.payload).map_err(|_| "COMMAND_TASK_INVALID")?
    {
        return Err(
            "IDEMPOTENCY_CONFLICT: this key already binds another request or endpoint".into(),
        );
    }
    Ok(())
}

pub(super) fn correlate(request: &CommandRequest, response: &Value) -> Result<(), String> {
    if response["id"] != request.id().as_str()
        || response["kind"] != request.kind
        || response["payload_digest"] != request.digest().to_hex()
    {
        return Err("FARMD_COMMAND_SUBJECT_MISMATCH: preserve the original journal and reconcile its command id".into());
    }
    Ok(())
}

#[cfg(unix)]
pub(super) fn reconciliation_body(
    session: &Session,
    id: &str,
) -> Result<Option<(RequestBody, CommandRequest)>, String> {
    let id = bullet_domain::CommandId::parse(id).map_err(|_| "COMMAND_ID_INVALID")?;
    let store = crate::auth::store::CredentialStore::open(&session.directory)?;
    let Some(value) = store.load_command(&id)? else {
        return Ok(None);
    };
    let record: Journal = serde_json::from_value(value).map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
    let envelope: crate::client::models::CommandEnvelope =
        crate::client::decode(&record.envelope).map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
    let body = record.body()?;
    if record.farmd != session.farmd || record.origin != session.origin {
        return Err("COMMAND_JOURNAL_ENDPOINT_MISMATCH".into());
    }
    let request = CommandRequest::new(envelope.idempotency_key, envelope.kind, &envelope.payload)
        .map_err(|_| "COMMAND_JOURNAL_CORRUPT")?;
    if request.id() != id {
        return Err("COMMAND_JOURNAL_SUBJECT_MISMATCH".into());
    }
    Ok(Some((body, request)))
}

#[cfg(unix)]
pub(super) fn reconciliation_request(
    session: &Session,
    id: &str,
) -> Result<Option<CommandRequest>, String> {
    Ok(reconciliation_body(session, id)?.map(|(_, request)| request))
}

#[cfg(not(unix))]
pub(super) fn reconciliation_body(
    _session: &Session,
    _id: &str,
) -> Result<Option<(RequestBody, CommandRequest)>, String> {
    Err("AUTH_PRIVATE_STORE_UNSUPPORTED".into())
}

#[cfg(not(unix))]
pub(super) fn reconciliation_request(
    _session: &Session,
    _id: &str,
) -> Result<Option<CommandRequest>, String> {
    Err("AUTH_PRIVATE_STORE_UNSUPPORTED".into())
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(all(test, unix))]
mod submission_tests;
