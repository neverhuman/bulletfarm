//! Production signed Unix client stays off public `/v1/leases/*`.

use bullet_domain::{AttemptId, AttemptState};
use bullet_harness_core::lease_transport::LeaseTransportSigningKey;
use bullet_runner_core::{LeaseClient, SignedLeaseRpcClient};

#[tokio::test]
async fn signed_rpc_refuses_advance_and_does_not_use_http_leases() {
    let key = LeaseTransportSigningKey::generate("kernel-local", "lease-1").unwrap();
    let client = SignedLeaseRpcClient::new(
        "/tmp/bullet-missing-lease-transport.sock".into(),
        key,
        "http://127.0.0.1:1",
    )
    .expect("client");
    let error = client
        .advance(&AttemptId::from_seed("unused"), AttemptState::Running)
        .await
        .expect_err("advance");
    assert_eq!(
        match &error {
            bullet_runner_core::RunnerError::Lease { code, .. } => code.as_str(),
            other => panic!("expected lease refusal, got {other}"),
        },
        "LEASE_TRANSPORT_UNSUPPORTED"
    );
}

#[test]
fn load_key_reads_exactly_64_secret_bytes() {
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("lease.key");
    let key = LeaseTransportSigningKey::generate("kernel-local", "lease-1").unwrap();
    std::fs::write(&path, key.secret_bytes()).expect("write");
    let loaded = SignedLeaseRpcClient::load_key(&path, "kernel-local", "lease-1").expect("load");
    assert_eq!(loaded.public_hex(), key.public_hex());
}
