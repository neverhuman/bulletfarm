use crate::digest::{parse_strict_json, require_utf8, sha256_hex};
use crate::hub::{Hub, Operation};
use crate::{Error, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use uuid::Uuid;

/// Every kind the endpoint accepts. Semantics land in later PRs: each records a durable
/// operation whose result is NOT_IMPLEMENTED once the actor passes the per-kind policy.
const KINDS: &[&str] = &[
    "note",
    "release",
    "stop",
    "grant_allowance",
    "resolve_decision",
    "take",
    "submit_human",
];

/// Spec §20.2 human-only set. Checked on the authenticated actor's kind, never on whoever
/// initiated it (CF06, HF17), before any row is written.
const HUMAN_ONLY: &[&str] = &[
    "grant_allowance",
    "resolve_decision",
    "take",
    "submit_human",
];

fn authorize(actor_kind: &str, kind: &str) -> Result<()> {
    match actor_kind {
        "human" => Ok(()),
        "agent" if !HUMAN_ONLY.contains(&kind) => Ok(()),
        "agent" => Err(Error::PolicyDenied(format!(
            "{kind} requires a human actor"
        ))),
        other => Err(Error::PolicyDenied(format!(
            "{other} principals cannot submit commands yet"
        ))),
    }
}

/// The actor's kind if the principal is still active; rechecked inside every command
/// transaction so a historical replay by a deactivated principal fails (BF3-004-AC02).
fn active_kind(c: &Connection, actor: &str) -> Result<String> {
    c.query_row(
        "SELECT kind FROM principals WHERE id=?1 AND active=1",
        [actor],
        |r| r.get(0),
    )
    .optional()?
    .ok_or(Error::AuthRequired)
}

pub(crate) fn submit(hub: &Hub, actor: &str, raw: &[u8]) -> Result<Operation> {
    let text = require_utf8(raw)?;
    let body = parse_strict_json(text)?;
    let command_id = body["command_id"]
        .as_str()
        .ok_or_else(|| Error::InvalidContract("command_id required".into()))?
        .to_owned();
    if command_id.is_empty() || body.get("schema_version") != Some(&json!(3)) {
        return Err(Error::InvalidContract(
            "schema_version 3 and command_id required".into(),
        ));
    }
    let kind = body["kind"]
        .as_str()
        .ok_or_else(|| Error::InvalidContract("kind required".into()))?
        .to_owned();
    if !KINDS.contains(&kind.as_str()) {
        return Err(Error::InvalidContract(format!("unsupported kind {kind}")));
    }
    let digest = sha256_hex(raw);
    let actor = actor.to_owned();
    let raw_text = text.to_owned();
    hub.db.call(move |c| {
        // One transaction: authorize -> replay lookup -> command -> event -> operation.
        // Any error drops `tx`, rolling back every row written so far.
        let tx = c.transaction()?;
        let actor_kind = active_kind(&tx, &actor)?;
        authorize(&actor_kind, &kind)?;
        let existing: Option<(String, String, String, String)> = tx
            .query_row(
                "SELECT cmd.body_sha, o.id, o.status, o.result_json FROM commands cmd
                 JOIN operations o ON o.actor_id=cmd.actor_id AND o.command_id=cmd.command_id
                 WHERE cmd.actor_id=?1 AND cmd.command_id=?2",
                params![actor, command_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        if let Some((stored, id, status, result_json)) = existing {
            if stored != digest {
                return Err(Error::CommandConflict);
            }
            return Ok(Operation {
                id,
                status,
                result: serde_json::from_str(&result_json).unwrap_or(Value::Null),
            });
        }
        let op_id = format!("op-{}", Uuid::new_v4());
        let result = json!({
            "ok": false,
            "error": "NOT_IMPLEMENTED",
            "kind": kind,
            "note": "command semantics land in later PRs; this records the command for replay"
        });
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO commands(actor_id,command_id,actor_kind,kind,body_sha,raw_json,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![actor, command_id, actor_kind, kind, digest, raw_text, now],
        )?;
        tx.execute(
            "INSERT INTO events(at,actor_id,kind,command_id,operation_id,payload_json)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                now,
                actor,
                kind,
                command_id,
                op_id,
                json!({"actor_kind": actor_kind, "status": "accepted", "result": result})
                    .to_string()
            ],
        )?;
        tx.execute(
            "INSERT INTO operations(id,actor_id,command_id,actor_kind,status,result_json)
             VALUES(?1,?2,?3,?4,'accepted',?5)",
            params![op_id, actor, command_id, actor_kind, result.to_string()],
        )?;
        tx.commit()?;
        Ok(Operation {
            id: op_id,
            status: "accepted".into(),
            result,
        })
    })
}

pub(crate) fn get(hub: &Hub, actor: &str, id: &str) -> Result<Operation> {
    let actor = actor.to_owned();
    let id = id.to_owned();
    hub.db.call(move |c| {
        c.query_row(
            "SELECT id,status,result_json FROM operations WHERE id=?1 AND actor_id=?2",
            params![id, actor],
            |r| {
                let raw: String = r.get(2)?;
                Ok(Operation {
                    id: r.get(0)?,
                    status: r.get(1)?,
                    result: serde_json::from_str(&raw).unwrap_or(Value::Null),
                })
            },
        )
        .optional()?
        .ok_or(Error::AuthRequired)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn replay_same_body_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let hub = Hub::open(dir.path()).unwrap();
        let body =
            br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"hi"}}"#;
        let a = hub.command_bytes("owner", body).unwrap();
        let b = hub.command_bytes("owner", body).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.status, "accepted");
    }

    #[test]
    fn changed_body_conflicts() {
        let dir = TempDir::new().unwrap();
        let hub = Hub::open(dir.path()).unwrap();
        hub.command_bytes(
            "owner",
            br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"a"}}"#,
        )
        .unwrap();
        let err = hub
            .command_bytes(
                "owner",
                br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"b"}}"#,
            )
            .unwrap_err();
        assert!(matches!(err, Error::CommandConflict));
        assert_eq!(err.code(), "RESOURCE_CONFLICT");
    }

    #[test]
    fn policy_is_per_principal_kind() {
        assert!(authorize("human", "grant_allowance").is_ok());
        assert!(authorize("agent", "note").is_ok());
        for kind in HUMAN_ONLY {
            assert_eq!(
                authorize("agent", kind).unwrap_err().code(),
                "POLICY_DENIED"
            );
        }
        for actor in ["runner", "verifier", "publisher"] {
            for kind in KINDS {
                assert_eq!(authorize(actor, kind).unwrap_err().code(), "POLICY_DENIED");
            }
        }
    }
}
