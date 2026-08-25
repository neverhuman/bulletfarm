//! Durable signed lease-transport grant index.

use super::{from_json, store};
use bullet_application::{LeaseGrant, LedgerError};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

pub(super) fn put_grant(
    conn: &mut Connection,
    idempotency_digest: &str,
    grant: &LeaseGrant,
) -> Result<(), LedgerError> {
    let json = serde_json::to_string(grant).map_err(store)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(store)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT grant_json FROM lease_transport_grants WHERE idempotency_digest = ?1",
            params![idempotency_digest],
            |row| row.get(0),
        )
        .optional()
        .map_err(store)?;
    if let Some(existing) = existing {
        let stored: LeaseGrant = from_json(&existing)?;
        if stored != *grant {
            return Err(bullet_domain::DomainError::Conflict(format!(
                "lease transport grant {idempotency_digest} differs from the stored row"
            ))
            .into());
        }
        tx.commit().map_err(store)?;
        return Ok(());
    }
    tx.execute(
        "INSERT INTO lease_transport_grants (idempotency_digest, grant_json, recorded_at)
         VALUES (?1, ?2, datetime('now'))",
        params![idempotency_digest, json],
    )
    .map_err(store)?;
    tx.commit().map_err(store)?;
    Ok(())
}

pub(super) fn get_grant(
    conn: &Connection,
    idempotency_digest: &str,
) -> Result<Option<LeaseGrant>, LedgerError> {
    let json: Option<String> = conn
        .query_row(
            "SELECT grant_json FROM lease_transport_grants WHERE idempotency_digest = ?1",
            params![idempotency_digest],
            |row| row.get(0),
        )
        .optional()
        .map_err(store)?;
    json.map(|text| from_json(&text)).transpose()
}

#[cfg(test)]
mod tests {
    use crate::sqlite::SqliteLedger;
    use bullet_application::lease_transport::{
        sign_runner_permit, SignedAcquireBody, SignedLeaseService,
    };
    use bullet_application::{
        materialize_plan, LeaseTransportOperation, LeaseTransportSigningKey, PlanInput,
    };
    use bullet_domain::{RunnerId, TaskClass};

    const AT: &str = "2026-01-01T00:00:00.000Z";

    #[test]
    fn sqlite_reopen_returns_the_same_grant() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("grants.sqlite");
        let key = LeaseTransportSigningKey::generate("kernel-local", "lease-1").unwrap();
        let body;
        let first;
        {
            let mut ledger = SqliteLedger::open(&path).expect("open");
            let graph = materialize_plan(
                &mut ledger,
                "sqlite-lease-transport",
                &PlanInput {
                    title: "durable grant".into(),
                    objective: "survive reopen".into(),
                    packages: vec![("one".into(), TaskClass::MechanicalCodeEdit)],
                },
                AT,
            )
            .expect("plan");
            body = SignedAcquireBody {
                work_package_id: graph.packages[0].id.clone(),
                runner_id: RunnerId::from_seed("signed-runner"),
                runner_epoch: 1,
                idempotency_key: "acquire-once".into(),
                ttl_seconds: 15,
            };
            let mut service = SignedLeaseService::new(key.verification_key().unwrap());
            let now = 1_700_000_000_000;
            let permit = sign_runner_permit(
                &key,
                LeaseTransportOperation::Acquire,
                &body.runner_id,
                body.runner_epoch,
                body.work_package_id.as_str(),
                &body.idempotency_key,
                &body,
                now,
            )
            .unwrap();
            first = service.acquire(&mut ledger, &permit, &body, now).unwrap();
        }
        let ledger = SqliteLedger::open(&path).expect("reopen");
        let mut service = SignedLeaseService::new(key.verification_key().unwrap());
        let now = 1_700_000_000_000;
        let permit = sign_runner_permit(
            &key,
            LeaseTransportOperation::Readback,
            &body.runner_id,
            body.runner_epoch,
            body.work_package_id.as_str(),
            &body.idempotency_key,
            &body,
            now,
        )
        .unwrap();
        let second = service.readback(&ledger, &permit, &body, now).unwrap();
        assert_eq!(first.attempt.id, second.attempt.id);
        assert_eq!(first.lease.fence, second.lease.fence);
    }
}
