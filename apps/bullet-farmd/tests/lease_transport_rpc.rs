//! farmd-internal Unix signed lease transport. Public `/v1/leases/*` stays
//! absent.

use bullet_adapters::SqliteLedger;
use bullet_application::{materialize_plan, LeaseTransportSigningKey, PlanInput};
use bullet_domain::{AttemptState, RunnerId, TaskClass, WorkPackageId};
use bullet_runner_core::{
    AcquireRequest, HeartbeatCall, LeaseClient, ReleaseCall, SignedLeaseRpcClient,
};
use std::path::Path;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;

fn seed_graph(db: &Path) -> WorkPackageId {
    let mut ledger = SqliteLedger::open(db).expect("open ledger");
    let graph = materialize_plan(
        &mut ledger,
        "unix-lease",
        &PlanInput {
            title: "signed unix lease".into(),
            objective: "runner signs, farmd verifies".into(),
            packages: vec![("one".into(), TaskClass::MechanicalCodeEdit)],
        },
        "2026-01-01T00:00:00.000Z",
    )
    .expect("plan");
    graph.packages[0].id.clone()
}

async fn start_rpc(
    db: &Path,
    socket: &Path,
    key: &LeaseTransportSigningKey,
) -> tokio::task::JoinHandle<()> {
    let state = bullet_farmd::lease_transport_rpc::LeaseTransportState::open(
        db,
        key.issuer(),
        key.key_id(),
        key.public_hex(),
    )
    .expect("state");
    let serve_path = socket.to_path_buf();
    let task = tokio::spawn(async move {
        bullet_farmd::lease_transport_rpc::serve(serve_path, state)
            .await
            .expect("serve");
    });
    for _ in 0..50 {
        if UnixStream::connect(socket).await.is_ok() {
            return task;
        }
        sleep(Duration::from_millis(20)).await;
    }
    panic!("lease-transport socket did not appear");
}

fn reload_key(key: &LeaseTransportSigningKey) -> LeaseTransportSigningKey {
    LeaseTransportSigningKey::from_bytes(key.issuer(), key.key_id(), key.secret_bytes())
        .expect("reload key")
}

fn client(socket: &Path, key: &LeaseTransportSigningKey) -> SignedLeaseRpcClient {
    SignedLeaseRpcClient::new(socket.to_path_buf(), reload_key(key), "http://127.0.0.1:1")
        .expect("client")
}

#[tokio::test]
async fn unix_rpc_acquire_readback_release_and_replay() {
    let directory = tempfile::tempdir().expect("tempdir");
    let db = directory.path().join("ledger.sqlite");
    let socket = directory.path().join("lease-transport.sock");
    let package = seed_graph(&db);
    let key = LeaseTransportSigningKey::generate("kernel-local", "lease-1").unwrap();
    let _server = start_rpc(&db, &socket, &key).await;
    let client = client(&socket, &key);
    let request = AcquireRequest {
        work_package_id: package,
        runner_id: RunnerId::from_seed("unix-runner"),
        runner_epoch: 1,
        idempotency_key: "acquire-once".into(),
        ttl_seconds: 15,
    };
    let first = client.acquire(&request).await.expect("acquire");
    assert_eq!(first.lease.fence, 1);
    let heartbeat = HeartbeatCall::for_grant(&first).expect("heartbeat");
    client.heartbeat(&heartbeat).await.expect("heartbeat");
    let again = client.acquire(&request).await.expect("idempotent acquire");
    assert_eq!(first.attempt.id, again.attempt.id);
    client
        .release(&ReleaseCall {
            attempt_id: first.attempt.id.clone(),
            outcome: AttemptState::Failed,
            requeue: false,
        })
        .await
        .expect("release");
}

#[tokio::test]
async fn unix_rpc_survives_farmd_restart_readback() {
    let directory = tempfile::tempdir().expect("tempdir");
    let db = directory.path().join("ledger.sqlite");
    let socket = directory.path().join("lease-transport.sock");
    let package = seed_graph(&db);
    let key = LeaseTransportSigningKey::generate("kernel-local", "lease-1").unwrap();
    let first_grant;
    {
        let server = start_rpc(&db, &socket, &key).await;
        let client = client(&socket, &key);
        first_grant = client
            .acquire(&AcquireRequest {
                work_package_id: package.clone(),
                runner_id: RunnerId::from_seed("unix-runner"),
                runner_epoch: 1,
                idempotency_key: "acquire-once".into(),
                ttl_seconds: 15,
            })
            .await
            .expect("acquire");
        server.abort();
        let _ = server.await;
        let _ = std::fs::remove_file(&socket);
    }
    let _server = start_rpc(&db, &socket, &key).await;
    let client = client(&socket, &key);
    let replay = client
        .acquire(&AcquireRequest {
            work_package_id: package,
            runner_id: RunnerId::from_seed("unix-runner"),
            runner_epoch: 1,
            idempotency_key: "acquire-once".into(),
            ttl_seconds: 15,
        })
        .await
        .expect("post-restart acquire");
    assert_eq!(first_grant.attempt.id, replay.attempt.id);
    assert_eq!(first_grant.lease.fence, replay.lease.fence);
}

#[tokio::test]
async fn public_lease_routes_stay_absent() {
    let directory = tempfile::tempdir().expect("tempdir");
    let db = directory.path().join("ledger.sqlite");
    let _ = seed_graph(&db);
    let app = bullet_farmd::api::router(&db).expect("router");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    stream
        .write_all(
            b"POST /v1/leases/acquire HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
        )
        .await
        .expect("write");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.expect("read");
    let response = String::from_utf8_lossy(&bytes);
    assert!(response.contains("404"), "{response}");
    assert!(
        !response.contains("LEASE_GRANTED"),
        "public acquire must stay unmounted"
    );
}
