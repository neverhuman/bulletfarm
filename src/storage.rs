use crate::{Error, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::mpsc::{self, SyncSender};
use std::time::Duration;

type Request = Box<dyn FnOnce(&mut Connection) + Send>;

/// Forward-only migrations, applied in order inside one transaction. Each file must
/// insert its own version into `schema_migrations`; a file that has run never runs again.
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/001_core.sql")),
    (2, include_str!("../migrations/002_kernel.sql")),
];

/// The only connection and database writer. Saturation rejects admission.
pub struct Database {
    sender: Option<SyncSender<Request>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let path = path.to_owned();
        let (sender, receiver) = mpsc::sync_channel::<Request>(64);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("bf-database".into())
            .spawn(move || match open_connection(&path) {
                Ok(mut conn) => {
                    let _ = ready_tx.send(Ok(()));
                    for request in receiver {
                        request(&mut conn);
                    }
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            })?;
        ready_rx
            .recv()
            .map_err(|_| Error::StorageUnavailable("database worker stopped".into()))??;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
        })
    }

    pub(crate) fn shutdown(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }

    pub fn call<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.sender
            .as_ref()
            .ok_or_else(|| Error::StorageUnavailable("database closed".into()))?
            .try_send(Box::new(move |conn| {
                let _ = tx.send(f(conn));
            }))
            .map_err(|_| {
                Error::StorageUnavailable(
                    "database queue full or closed; retry the same command ID".into(),
                )
            })?;
        rx.recv()
            .map_err(|_| Error::StorageUnavailable("database worker stopped".into()))?
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn open_connection(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    let version: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let source: String = conn.query_row("SELECT sqlite_source_id()", [], |r| r.get(0))?;
    if version != "3.53.2"
        || source
            != "2026-06-03 19:12:13 d6e03d8c777cfa2d35e3b60d8ec3e0187f3e9f99d8e2ee9cac695fd6fcdf1a24"
    {
        return Err(Error::StorageUnavailable(format!(
            "unqualified SQLite runtime {version}"
        )));
    }
    conn.busy_timeout(Duration::from_millis(100))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
    migrate(&mut conn)?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

/// Apply every migration newer than the recorded schema version, atomically. Foreign
/// keys are off while tables are rebuilt (the pragma is a no-op inside a transaction, so
/// it is set here); `PRAGMA foreign_key_check` guards the commit.
fn migrate(conn: &mut Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
    let tx = conn.transaction()?;
    let tracked: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='schema_migrations')",
        [],
        |r| r.get(0),
    )?;
    let current: i64 = if tracked {
        tx.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?
    } else {
        0
    };
    for (version, sql) in MIGRATIONS {
        if *version <= current {
            continue;
        }
        tx.execute_batch(sql)?;
        let recorded: i64 = tx.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version=?1",
            [version],
            |r| r.get(0),
        )?;
        if recorded != 1 {
            return Err(Error::StorageUnavailable(format!(
                "migration {version} did not record its version"
            )));
        }
    }
    let mut violations = 0;
    let mut check = tx.prepare("PRAGMA foreign_key_check")?;
    let mut rows = check.query([])?;
    while rows.next()?.is_some() {
        violations += 1;
    }
    drop(rows);
    drop(check);
    if violations > 0 {
        return Err(Error::StorageUnavailable(format!(
            "migration left {violations} foreign key violation(s)"
        )));
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn describe(path: &Path) -> Result<(Vec<i64>, Vec<String>, Vec<String>)> {
        let db = Database::open(path)?;
        db.call(|c| {
            let versions = c
                .prepare("SELECT version FROM schema_migrations ORDER BY version")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<i64>>>()?;
            let shape = c
                .prepare(
                    "SELECT type||' '||name||': '||sql FROM sqlite_master
                     WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' ORDER BY type,name",
                )?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            let command_key = c
                .prepare("SELECT name FROM pragma_table_info('commands') WHERE pk>0 ORDER BY pk")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            Ok((versions, shape, command_key))
        })
    }

    #[test]
    fn fresh_and_legacy_databases_end_with_the_same_shape() {
        let dir = TempDir::new().unwrap();
        let fresh = dir.path().join("fresh.sqlite");
        let legacy = dir.path().join("legacy.sqlite");

        // A hub.sqlite as today's 001 left it: four tables, one command, one operation.
        {
            let old = Connection::open(&legacy).unwrap();
            old.execute_batch(MIGRATIONS[0].1).unwrap();
            old.execute_batch(
                "INSERT INTO commands VALUES('c-legacy','owner','note','sha','{}','2026-09-18T00:00:00Z');
                 INSERT INTO operations VALUES('op-legacy','c-legacy','owner','accepted','{}');",
            )
            .unwrap();
        }

        let (fresh_versions, fresh_shape, fresh_key) = describe(&fresh).unwrap();
        let (legacy_versions, legacy_shape, legacy_key) = describe(&legacy).unwrap();
        assert_eq!(fresh_versions, vec![1, 2]);
        assert_eq!(legacy_versions, vec![1, 2]);
        assert_eq!(fresh_shape, legacy_shape);
        assert_eq!(fresh_key, vec!["actor_id", "command_id"]);
        assert_eq!(legacy_key, fresh_key);
        assert!(fresh_shape.iter().any(|s| s.starts_with("table events: ")));
        assert!(fresh_shape
            .iter()
            .any(|s| s.starts_with("table principals: ") && s.contains("'runner'")));

        // Legacy rows survive with their actor kind, and the migrated file is usable.
        let db = Database::open(&legacy).unwrap();
        let (kinds, ops): (String, i64) = db
            .call(|c| {
                let kinds = c.query_row(
                    "SELECT c.actor_kind||'/'||o.actor_kind FROM commands c
                     JOIN operations o ON o.actor_id=c.actor_id AND o.command_id=c.command_id
                     WHERE c.command_id='c-legacy'",
                    [],
                    |r| r.get(0),
                )?;
                c.execute("INSERT INTO principals VALUES('runner-1','runner',1)", [])?;
                assert!(c
                    .execute("INSERT INTO principals VALUES('x','bogus',1)", [])
                    .is_err());
                let ops = c.query_row("SELECT COUNT(*) FROM operations", [], |r| r.get(0))?;
                Ok((kinds, ops))
            })
            .unwrap();
        assert_eq!(kinds, "human/human");
        assert_eq!(ops, 1);
        drop(db);

        // Reopening is a no-op: same versions, same shape.
        let (again, shape_again, _) = describe(&legacy).unwrap();
        assert_eq!(again, vec![1, 2]);
        assert_eq!(shape_again, fresh_shape);
    }
}
