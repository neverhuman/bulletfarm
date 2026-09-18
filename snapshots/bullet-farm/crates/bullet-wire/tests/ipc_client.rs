use bullet_wire::{
    Blake3Digest, IpcHelloAck, IpcService, JsonRpcFailure, JsonRpcSuccess, RpcClientSession,
    RpcInbound, RpcRequestId, RpcResponse, RpcServerSession, encode_jsonl,
};
use serde_json::json;

const NOW: u64 = 1_000_000;

fn id(byte: u8) -> RpcRequestId {
    RpcRequestId::from_digest(Blake3Digest::from_bytes([byte; 32]))
}

fn frame(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\n").expect("encoded JSONL line")
}

fn connect(client: &mut RpcClientSession, server: &mut RpcServerSession, frame_limit: usize) {
    let hello_id = id(1);
    let line = client
        .start_hello(hello_id.clone(), frame_limit)
        .expect("client hello");
    let RpcInbound::Hello {
        acknowledgement, ..
    } = server
        .accept_frame(frame(&line), NOW)
        .expect("server hello")
    else {
        panic!("expected hello");
    };
    let line = encode_jsonl(&JsonRpcSuccess::new(hello_id, acknowledgement), frame_limit)
        .expect("hello response");
    assert!(matches!(
        client
            .accept_response(frame(&line))
            .expect("client accepts acknowledgement"),
        RpcResponse::Success { .. }
    ));
}

#[test]
fn client_and_server_round_trip_identical_hello_call_cancel_and_response_bytes() {
    let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
    let mut server = RpcServerSession::new(IpcService::BulletGitd);
    connect(&mut client, &mut server, 2048);
    assert_eq!(client.negotiated_frame_bytes(), Some(2048));
    assert_eq!(server.negotiated_frame_bytes(), Some(2048));

    let request_id = id(2);
    let line = client
        .start_call(
            request_id.clone(),
            "bullet.workspace.apply",
            NOW + 100,
            NOW,
            json!({"path": "src/lib.rs"}),
        )
        .expect("client call");
    assert!(matches!(
        server.accept_frame(frame(&line), NOW).expect("server call"),
        RpcInbound::Call {
            request_id: ref seen,
            ref body,
            ..
        } if seen == &request_id && body == &json!({"path": "src/lib.rs"})
    ));

    let line = client.cancel(&request_id).expect("client cancellation");
    assert_eq!(
        server
            .accept_frame(frame(&line), NOW)
            .expect("server cancellation"),
        RpcInbound::Cancel {
            request_id: request_id.clone()
        }
    );
    assert!(server.request_is_cancelled(&request_id).unwrap());

    let line = encode_jsonl(
        &JsonRpcSuccess::new(request_id.clone(), json!({"outcome": "cancelled"})),
        2048,
    )
    .unwrap();
    server.finish_request(&request_id).unwrap();
    assert_eq!(
        client
            .accept_response(frame(&line))
            .expect("correlated terminal response"),
        RpcResponse::Success {
            request_id,
            result: json!({"outcome": "cancelled"})
        }
    );
}

#[test]
fn client_rejects_hostile_acknowledgements_and_hello_errors() {
    let cases = [
        IpcHelloAck {
            protocol: "other".into(),
            service: IpcService::BulletGitd,
            selected_version: 1,
            max_frame_bytes: 2048,
            features: vec!["cancellation".into(), "deadlines".into()],
        },
        IpcHelloAck {
            protocol: "bullet.ipc".into(),
            service: IpcService::Verifier,
            selected_version: 1,
            max_frame_bytes: 2048,
            features: vec!["cancellation".into(), "deadlines".into()],
        },
        IpcHelloAck {
            protocol: "bullet.ipc".into(),
            service: IpcService::BulletGitd,
            selected_version: 2,
            max_frame_bytes: 2048,
            features: vec!["cancellation".into(), "deadlines".into()],
        },
        IpcHelloAck {
            protocol: "bullet.ipc".into(),
            service: IpcService::BulletGitd,
            selected_version: 1,
            max_frame_bytes: 4096,
            features: vec!["cancellation".into(), "deadlines".into()],
        },
    ];
    for acknowledgement in cases {
        let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
        client.start_hello(id(1), 2048).unwrap();
        let line = encode_jsonl(&JsonRpcSuccess::new(id(1), acknowledgement), 2048).unwrap();
        assert_eq!(
            client.accept_response(frame(&line)).unwrap_err().code(),
            "IPC_HELLO_ACK_INVALID"
        );
        assert_eq!(
            client.start_hello(id(2), 2048).unwrap_err().code(),
            "IPC_SESSION_CLOSED"
        );
    }

    let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
    client.start_hello(id(1), 2048).unwrap();
    let line = encode_jsonl(&JsonRpcFailure::new(id(1), -32_601, "no version"), 2048).unwrap();
    assert_eq!(
        client.accept_response(frame(&line)).unwrap_err().code(),
        "IPC_HELLO_REFUSED"
    );
}

#[test]
fn client_rejects_unknown_late_and_reused_request_ids_for_connection_lifetime() {
    let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
    let mut server = RpcServerSession::new(IpcService::BulletGitd);
    connect(&mut client, &mut server, 2048);
    let line = encode_jsonl(&JsonRpcSuccess::new(id(9), json!({})), 2048).unwrap();
    assert_eq!(
        client.accept_response(frame(&line)).unwrap_err().code(),
        "IPC_RESPONSE_ID_UNKNOWN"
    );
    assert_eq!(
        client
            .start_call(id(2), "bullet.x", NOW + 1, NOW, json!({}))
            .unwrap_err()
            .code(),
        "IPC_SESSION_CLOSED"
    );

    let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
    let mut server = RpcServerSession::new(IpcService::BulletGitd);
    connect(&mut client, &mut server, 2048);
    let request_id = id(2);
    client
        .start_call(request_id.clone(), "bullet.x", NOW + 1, NOW, json!({}))
        .unwrap();
    let line = encode_jsonl(
        &JsonRpcSuccess::new(request_id.clone(), json!({"ok": true})),
        2048,
    )
    .unwrap();
    client
        .accept_response(frame(&line))
        .expect("first response");
    assert_eq!(
        client.accept_response(frame(&line)).unwrap_err().code(),
        "IPC_RESPONSE_ID_UNKNOWN"
    );

    let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
    let mut server = RpcServerSession::new(IpcService::BulletGitd);
    connect(&mut client, &mut server, 2048);
    let request_id = id(2);
    client
        .start_call(request_id.clone(), "bullet.x", NOW + 1, NOW, json!({}))
        .unwrap();
    let line = encode_jsonl(
        &JsonRpcSuccess::new(request_id.clone(), json!({"ok": true})),
        2048,
    )
    .unwrap();
    client.accept_response(frame(&line)).unwrap();
    assert_eq!(
        client
            .start_call(request_id, "bullet.x", NOW + 1, NOW, json!({}))
            .unwrap_err()
            .code(),
        "IPC_REQUEST_ID_REUSED"
    );
}

#[test]
fn client_rejects_ambiguous_malformed_and_mismatched_hello_responses() {
    let values = [
        json!({
            "error": {"code": -1, "message": "x"}, "id": id(1),
            "jsonrpc": "2.0", "result": {}
        }),
        json!({"id": id(1), "jsonrpc": "2.0"}),
        json!({"extra": true, "id": id(1), "jsonrpc": "2.0", "result": {}}),
        json!({"id": id(1), "jsonrpc": "1.0", "result": {}}),
        json!({
            "id": id(9), "jsonrpc": "2.0",
            "result": {"features": ["cancellation", "deadlines"],
                "max_frame_bytes": 2048, "protocol": "bullet.ipc",
                "selected_version": 1, "service": "bullet-gitd"}
        }),
    ];
    for value in values {
        let mut client = RpcClientSession::new(IpcService::Runner, IpcService::BulletGitd);
        client.start_hello(id(1), 2048).unwrap();
        let encoded = bullet_wire::canonical_json(&value).unwrap();
        assert!(client.accept_response(&encoded).is_err());
        assert_eq!(
            client.start_hello(id(2), 2048).unwrap_err().code(),
            "IPC_SESSION_CLOSED"
        );
    }
}
