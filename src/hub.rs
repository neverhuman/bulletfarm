use crate::digest::sha256_hex;
use crate::storage::Database;
use crate::{Error, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub struct Hub {
    pub data_dir: PathBuf,
    pub(crate) db: Database,
    pub epoch: String,
    _instance: File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub status: String,
    pub result: Value,
}

impl Hub {
    pub fn open(data_dir: &Path) -> Result<Self> {
        if data_dir
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err(Error::PolicyDenied(
                "hub directory cannot be a symlink".into(),
            ));
        }
        fs::create_dir_all(data_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))?;
        }
        let instance = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(data_dir.join("hub.lock"))?;
        instance.try_lock().map_err(|_| {
            Error::ResourceConflict("hub already running; reconnect using endpoint.json".into())
        })?;
        let hub = Self {
            data_dir: fs::canonicalize(data_dir)?,
            db: Database::open(&data_dir.join("hub.sqlite"))?,
            epoch: format!("boot-{}", Uuid::new_v4()),
            _instance: instance,
        };
        Ok(hub)
    }

    pub fn sqlite_version(&self) -> Result<String> {
        self.db
            .call(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?))
    }

    /// Principal kinds the hub can hold (migrations/002_kernel.sql CHECK).
    pub const PRINCIPAL_KINDS: &'static [&'static str] =
        &["human", "agent", "runner", "verifier", "publisher"];

    /// Mint a session for an existing active principal. Bootstrap uses this for `owner`.
    pub fn ensure_session(&self, principal: &str) -> Result<String> {
        let principal = principal.to_owned();
        let (token, digest) = new_token();
        self.db.call(move |c| {
            let active: i64 = c.query_row(
                "SELECT COUNT(*) FROM principals WHERE id=?1 AND active=1",
                [&principal],
                |r| r.get(0),
            )?;
            if active == 0 {
                return Err(Error::AuthRequired);
            }
            insert_session(c, &digest, &principal)
        })?;
        Ok(token)
    }

    /// Hub-internal: create `principal` with `kind` if absent (a present principal must
    /// already have that kind), then mint a session. Not reachable over HTTP; enrollment
    /// (BF3-004-AC03) will replace it.
    pub fn ensure_session_kind(&self, principal: &str, kind: &str) -> Result<String> {
        if !Self::PRINCIPAL_KINDS.contains(&kind) {
            return Err(Error::InvalidContract(format!(
                "unknown principal kind {kind}"
            )));
        }
        let principal = principal.to_owned();
        let kind = kind.to_owned();
        let (token, digest) = new_token();
        self.db.call(move |c| {
            let tx = c.transaction()?;
            let existing: Option<(String, i64)> = tx
                .query_row(
                    "SELECT kind, active FROM principals WHERE id=?1",
                    [&principal],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            match existing {
                None => {
                    tx.execute(
                        "INSERT INTO principals(id,kind,active) VALUES(?1,?2,1)",
                        params![principal, kind],
                    )?;
                }
                Some((found, _)) if found != kind => {
                    return Err(Error::PolicyDenied(format!(
                        "principal {principal} is a {found}, not a {kind}"
                    )));
                }
                Some((_, 0)) => return Err(Error::AuthRequired),
                Some(_) => {}
            }
            insert_session(&tx, &digest, &principal)?;
            tx.commit()?;
            Ok(())
        })?;
        Ok(token)
    }

    /// Hub-internal: activate or deactivate a principal. Existing sessions stop
    /// authenticating and historical replays stop resolving while inactive.
    pub fn set_principal_active(&self, principal: &str, active: bool) -> Result<()> {
        let principal = principal.to_owned();
        self.db.call(move |c| {
            let changed = c.execute(
                "UPDATE principals SET active=?1 WHERE id=?2",
                params![active as i64, principal],
            )?;
            if changed == 0 {
                return Err(Error::AuthRequired);
            }
            Ok(())
        })
    }

    pub fn lookup_session(&self, token: &str) -> Result<String> {
        let hash = sha256_hex(token.as_bytes());
        self.db.call(move |c| {
            c.query_row(
                "SELECT s.principal_id FROM sessions s JOIN principals p ON p.id=s.principal_id
                 WHERE s.token=?1 AND s.revoked=0 AND p.active=1 AND julianday(s.expires_at)>julianday('now')",
                [hash],
                |r| r.get(0),
            )
            .map_err(|_| Error::AuthRequired)
        })
    }

    pub fn revoke_session(&self, token: &str) -> Result<()> {
        let hash = sha256_hex(token.as_bytes());
        self.db.call(move |c| {
            c.execute("UPDATE sessions SET revoked=1 WHERE token=?1", [hash])?;
            Ok(())
        })
    }

    pub fn doctor(&self) -> Value {
        json!({
            "ok": true,
            "sqlite": self.sqlite_version().ok(),
            "clis": {
                "claude": probe("claude"),
                "codex": probe("codex"),
                "cursor": probe("cursor-agent"),
                "grok": probe("grok"),
            },
            "live_executor": null,
            "note": "Core hub only. Agent discovery, board, and TUI land in later PRs."
        })
    }

    pub fn operation(&self, actor: &str, id: &str) -> Result<Operation> {
        crate::commands::get(self, actor, id)
    }

    pub fn command_bytes(&self, actor: &str, raw: &[u8]) -> Result<Operation> {
        crate::commands::submit(self, actor, raw)
    }
}

impl Drop for Hub {
    fn drop(&mut self) {
        self.db.shutdown();
        let _ = self._instance.unlock();
    }
}

fn new_token() -> (String, String) {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let digest = sha256_hex(token.as_bytes());
    (token, digest)
}

fn insert_session(c: &Connection, digest: &str, principal: &str) -> Result<()> {
    c.execute(
        "INSERT INTO sessions(token,principal_id,created_at,expires_at) VALUES(?1,?2,?3,?4)",
        params![
            digest,
            principal,
            Utc::now().to_rfc3339(),
            (Utc::now() + Duration::hours(8)).to_rfc3339()
        ],
    )?;
    Ok(())
}

fn probe(bin: &str) -> Value {
    match Command::new(bin)
        .arg("--version")
        .env("TERM", "dumb")
        .output()
    {
        Ok(out) if out.status.success() => {
            json!(String::from_utf8_lossy(&out.stdout)
                .trim()
                .lines()
                .next()
                .unwrap_or(""))
        }
        _ => Value::Null,
    }
}
