use bullet_wire::{
    Blake3Digest, IPC_PROTOCOL_VERSION, IpcHelloAck, IpcService, JsonRpcFailure, JsonRpcSuccess,
    MAX_IPC_FRAME_BYTES, MAX_IPC_IN_FLIGHT, RpcInbound, RpcRequestId, RpcServerSession,
    encode_jsonl,
};
use serde_json::{Value, json};

const NOW: u64 = 1_000_000;

fn id(byte: u8) -> RpcRequestId {
    RpcRequestId::from_digest(Blake3Digest::from_bytes([byte; 32]))
}

fn frame(value: Value) -> Vec<u8> {
    bullet_wire::canonical_json(&value).expect("canonical frame")
}

fn hello_with(request_id: RpcRequestId, versions: Value, limit: u64) -> Vec<u8> {
    frame(json!({
        "id": request_id,
        "jsonrpc": "2.0",
        "method": "bullet.hello",
        "params": {
            "features": ["cancellation", "deadlines"],
            "max_frame_bytes": limit,
            "protocol": "bullet.ipc",
            "service": "runner",
            "versions": versions
        }
    }))
}

fn hello(session: &mut RpcServerSession) -> RpcRequestId {
    let request_id = id(1);
    let accepted = session
        .accept_frame(
            &hello_with(
                request_id.clone(),
                json!([IPC_PROTOCOL_VERSION]),
                MAX_IPC_FRAME_BYTES as u64,
            ),
            NOW,
        )
        .expect("hello accepted");
    assert!(matches!(
        accepted,
        RpcInbound::Hello {
            request_id: ref seen,
            peer: IpcService::Runner,
            ..
        } if seen == &request_id
    ));
    request_id
}

fn call_frame(request_id: RpcRequestId, deadline: u64, body: Value) -> Vec<u8> {
    frame(json!({
        "id": request_id,
        "jsonrpc": "2.0",
        "method": "bullet.workspace.apply",
        "params": {"body": body, "deadline_unix_ms": deadline}
    }))
}

fn cancel_frame(request_id: RpcRequestId) -> Vec<u8> {
    frame(json!({
        "jsonrpc": "2.0",
        "method": "bullet.cancel",
        "params": {"request_id": request_id}
    }))
}

#[test]
fn hello_call_cancel_and_finish_are_exact() {
    let mut session = RpcServerSession::new(IpcService::BulletGitd);
    hello(&mut session);
    assert_eq!(session.negotiated_frame_bytes(), Some(MAX_IPC_FRAME_BYTES));

    let request_id = id(2);
    let accepted = session
        .accept_frame(
            &call_frame(request_id.clone(), NOW + 10_000, json!({"path": "a"})),
            NOW,
        )
        .expect("call accepted");
    assert!(matches!(
        accepted,
        RpcInbound::Call {
            request_id: ref seen,
            ref method,
            deadline_unix_ms,
            ref body,
        } if seen == &request_id
            && method == "bullet.workspace.apply"
            && deadline_unix_ms == NOW + 10_000
            && body == &json!({"path": "a"})
    ));
    assert!(!session.request_is_cancelled(&request_id).unwrap());

    assert_eq!(
        session
            .accept_frame(&cancel_frame(request_id.clone()), NOW)
            .unwrap(),
        RpcInbound::Cancel {
            request_id: request_id.clone()
        }
    );
    assert!(session.request_is_cancelled(&request_id).unwrap());
    session.finish_request(&request_id).unwrap();
    assert_eq!(
        session
            .request_is_cancelled(&request_id)
            .unwrap_err()
            .code(),
        "IPC_REQUEST_UNKNOWN"
    );
}

#[test]
fn hello_is_mandatory_versioned_and_negotiates_the_lower_frame_limit() {
    let mut missing = RpcServerSession::new(IpcService::Verifier);
    let error = missing
        .accept_frame(&call_frame(id(2), NOW + 1, json!({})), NOW)
        .unwrap_err();
    assert_eq!(error.code(), "IPC_HELLO_REQUIRED");
    assert_eq!(
        missing
            .accept_frame(&hello_with(id(1), json!([1]), 2048), NOW)
            .unwrap_err()
            .code(),
        "IPC_SESSION_CLOSED"
    );

    for versions in [json!([]), json!([1, 1]), json!([2])] {
        let mut session = RpcServerSession::new(IpcService::Verifier);
        assert!(
            session
                .accept_frame(&hello_with(id(1), versions, 2048), NOW)
                .is_err()
        );
    }

    let mut session = RpcServerSession::new(IpcService::Verifier);
    let accepted = session
        .accept_frame(&hello_with(id(1), json!([1, 2]), 2048), NOW)
        .unwrap();
    assert!(matches!(
        accepted,
        RpcInbound::Hello {
            acknowledgement: IpcHelloAck {
                selected_version: 1,
                max_frame_bytes: 2048,
                ..
            },
            ..
        }
    ));
    assert_eq!(session.negotiated_frame_bytes(), Some(2048));
}

#[test]
fn malformed_noncanonical_and_oversized_frames_close_the_session() {
    let cases: Vec<(&str, Vec<u8>, &str)> = vec![
        ("empty", Vec::new(), "EMPTY_DOCUMENT"),
        (
            "newline",
            b"{\"jsonrpc\":\"2.0\"}\n".to_vec(),
            "IPC_FRAME_BOUNDARY_INVALID",
        ),
        (
            "noncanonical",
            b"{ \"jsonrpc\": \"2.0\" }".to_vec(),
            "NON_CANONICAL_JSON",
        ),
        (
            "duplicate",
            b"{\"id\":\"rpc_0000000000000000000000000000000000000000000000000000000000000000\",\"id\":\"rpc_1111111111111111111111111111111111111111111111111111111111111111\",\"jsonrpc\":\"2.0\",\"method\":\"bullet.hello\",\"params\":{}}".to_vec(),
            "DUPLICATE_JSON_KEY",
        ),
        (
            "oversized",
            vec![b'x'; MAX_IPC_FRAME_BYTES + 1],
            "IPC_FRAME_TOO_LARGE",
        ),
    ];
    for (name, bytes, code) in cases {
        let mut session = RpcServerSession::new(IpcService::Effects);
        assert_eq!(
            session.accept_frame(&bytes, NOW).unwrap_err().code(),
            code,
            "{name}"
        );
        assert_eq!(
            session
                .accept_frame(&hello_with(id(1), json!([1]), 2048), NOW)
                .unwrap_err()
                .code(),
            "IPC_SESSION_CLOSED",
            "{name}"
        );
    }
}

#[test]
fn deadlines_duplicate_ids_and_bad_cancellation_fail_closed() {
    for deadline in [NOW - 1, NOW, NOW + 60_001] {
        let mut session = RpcServerSession::new(IpcService::BulletGitd);
        hello(&mut session);
        let error = session
            .accept_frame(&call_frame(id(2), deadline, json!({})), NOW)
            .unwrap_err();
        assert!(matches!(
            error.code(),
            "IPC_DEADLINE_EXPIRED" | "IPC_DEADLINE_TOO_FAR"
        ));
    }

    let mut duplicate = RpcServerSession::new(IpcService::BulletGitd);
    hello(&mut duplicate);
    let request_id = id(2);
    duplicate
        .accept_frame(&call_frame(request_id.clone(), NOW + 1, json!({})), NOW)
        .unwrap();
    duplicate.finish_request(&request_id).unwrap();
    assert_eq!(
        duplicate
            .accept_frame(&call_frame(request_id, NOW + 1, json!({})), NOW)
            .unwrap_err()
            .code(),
        "IPC_REQUEST_ID_REUSED"
    );

    let mut unknown = RpcServerSession::new(IpcService::BulletGitd);
    hello(&mut unknown);
    assert_eq!(
        unknown
            .accept_frame(&cancel_frame(id(9)), NOW)
            .unwrap_err()
            .code(),
        "IPC_CANCEL_UNKNOWN"
    );

    let mut repeated = RpcServerSession::new(IpcService::BulletGitd);
    hello(&mut repeated);
    let request_id = id(2);
    repeated
        .accept_frame(&call_frame(request_id.clone(), NOW + 1, json!({})), NOW)
        .unwrap();
    repeated
        .accept_frame(&cancel_frame(request_id.clone()), NOW)
        .unwrap();
    assert_eq!(
        repeated
            .accept_frame(&cancel_frame(request_id), NOW)
            .unwrap_err()
            .code(),
        "IPC_CANCEL_DUPLICATE"
    );
}

#[test]
fn negotiated_and_in_flight_bounds_are_enforced() {
    let mut small = RpcServerSession::new(IpcService::BulletGitd);
    small
        .accept_frame(&hello_with(id(1), json!([1]), 1024), NOW)
        .unwrap();
    let oversized_body = "x".repeat(1024);
    assert_eq!(
        small
            .accept_frame(
                &call_frame(id(2), NOW + 1, json!({"value": oversized_body})),
                NOW,
            )
            .unwrap_err()
            .code(),
        "IPC_FRAME_TOO_LARGE"
    );

    let mut full = RpcServerSession::new(IpcService::BulletGitd);
    hello(&mut full);
    for index in 0..MAX_IPC_IN_FLIGHT {
        let request_id = id(u8::try_from(index + 2).unwrap());
        full.accept_frame(&call_frame(request_id, NOW + 1, json!({})), NOW)
            .unwrap();
    }
    assert_eq!(
        full.accept_frame(&call_frame(id(100), NOW + 1, json!({})), NOW)
            .unwrap_err()
            .code(),
        "IPC_IN_FLIGHT_LIMIT"
    );
}

#[test]
fn response_encoding_is_canonical_bounded_jsonl() {
    let success = JsonRpcSuccess::new(id(1), json!({"selected_version": 1}));
    let encoded = encode_jsonl(&success, 1024).unwrap();
    assert_eq!(encoded.last(), Some(&b'\n'));
    assert_eq!(
        std::str::from_utf8(&encoded).unwrap(),
        format!(
            "{{\"id\":\"{}\",\"jsonrpc\":\"2.0\",\"result\":{{\"selected_version\":1}}}}\n",
            id(1)
        )
    );

    let failure = JsonRpcFailure::new(id(2), -32_000, "refused");
    let encoded = encode_jsonl(&failure, 1024).unwrap();
    let decoded: Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded["error"]["code"], -32_000);
    assert_eq!(
        encode_jsonl(&failure, 100).unwrap_err().code(),
        "IPC_FRAME_LIMIT_INVALID"
    );
}

#[test]
fn wrong_versions_unknown_fields_and_unadmitted_methods_poison_the_session() {
    let cases = [
        json!({
            "id": id(1), "jsonrpc": "1.0", "method": "bullet.hello",
            "params": {"features": ["cancellation", "deadlines"],
                "max_frame_bytes": 2048, "protocol": "bullet.ipc",
                "service": "runner", "versions": [1]}
        }),
        json!({
            "extra": true, "id": id(1), "jsonrpc": "2.0", "method": "bullet.hello",
            "params": {"features": ["cancellation", "deadlines"],
                "max_frame_bytes": 2048, "protocol": "bullet.ipc",
                "service": "runner", "versions": [1]}
        }),
        json!({
            "id": id(1), "jsonrpc": "2.0", "method": "bullet.hello",
            "params": {"extra": true, "features": ["cancellation", "deadlines"],
                "max_frame_bytes": 2048, "protocol": "bullet.ipc",
                "service": "runner", "versions": [1]}
        }),
        json!({
            "id": id(1), "jsonrpc": "2.0", "method": "bullet.hello",
            "params": {"features": ["deadlines"], "max_frame_bytes": 2048,
                "protocol": "bullet.ipc", "service": "runner", "versions": [1]}
        }),
    ];
    for value in cases {
        let mut session = RpcServerSession::new(IpcService::BulletGitd);
        assert!(session.accept_frame(&frame(value), NOW).is_err());
        assert_eq!(
            session
                .accept_frame(&hello_with(id(9), json!([1]), 2048), NOW)
                .unwrap_err()
                .code(),
            "IPC_SESSION_CLOSED"
        );
    }

    for value in [
        json!({
            "id": id(2), "jsonrpc": "2.0", "method": "Bullet.Bad",
            "params": {"body": {}, "deadline_unix_ms": NOW + 1}
        }),
        json!({
            "jsonrpc": "2.0", "method": "bullet.unexpected", "params": {}
        }),
        json!({
            "id": id(2), "jsonrpc": "2.0", "method": "bullet.cancel",
            "params": {"request_id": id(2)}
        }),
    ] {
        let mut session = RpcServerSession::new(IpcService::BulletGitd);
        hello(&mut session);
        assert!(session.accept_frame(&frame(value), NOW).is_err());
        assert_eq!(
            session
                .accept_frame(&call_frame(id(3), NOW + 1, json!({})), NOW)
                .unwrap_err()
                .code(),
            "IPC_SESSION_CLOSED"
        );
    }
}
