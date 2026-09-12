use super::*;
use std::os::unix::fs::PermissionsExt;

fn submission_socket(
    listener: &std::net::TcpListener,
    selected: &str,
) -> (std::net::TcpStream, String) {
    use std::io::Read;
    // Submission commits and syncs its journal before connecting. Allow bounded
    // setup time for real storage under load; acknowledgement-before-body has
    // its own two-second assertion below and must not inherit this allowance.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut socket = loop {
        match listener.accept() {
            Ok((socket, _)) => break socket,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("accept failed: {e}"),
        }
        assert!(
            std::time::Instant::now() < deadline,
            "caller did not reach {selected}"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();
    socket
        .set_write_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        assert!(bytes.len() < 16_384);
    }
    let request = String::from_utf8(bytes).unwrap();
    assert!(request.starts_with(selected));
    (socket, request)
}

#[test]
fn submission_acknowledgement_precedes_body_and_preserves_journal_after_response_loss() {
    use std::io::{Read, Write};
    use std::time::Duration;
    for outcome in [
        "missing",
        "foreign",
        "duplicate",
        "absent",
        "lost",
        "valid",
        "fresh",
    ] {
        let temp = tempfile::Builder::new()
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir()
            .unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let session = Session {
            directory: temp.path().join("state"),
            credentials: crate::auth::store::Credentials {
                schema_version: 1,
                farmd: endpoint.clone(),
                origin: endpoint,
                cookie: format!("bullet_session=ses_{}", "a".repeat(64)),
                csrf: format!("csrf_{}", "b".repeat(64)),
            },
        };
        let mut payload = super::super::task::payload(
            super::super::task::fixture(),
            "acct",
            "codex",
            "model",
            None,
        )
        .unwrap();
        payload.task.objective = "Preserve café and 日本語\nexactly".into();
        let envelope = serde_json::json!({"idempotency_key":"acknowledged-submit","kind":"run_coding","payload":payload});
        let expected =
            CommandRequest::new("acknowledged-submit", "run_coding", &envelope["payload"]).unwrap();
        let journal = session.directory.join(format!("{}.json", expected.id()));
        // Seed a durable record whose valid wire JSON differs from Value serialization.
        // Retry must preserve whitespace and Unicode escapes, including after restart.
        let reconstructed = format!(
            " \n{}\n",
            serde_json::to_string_pretty(&envelope)
                .unwrap()
                .replace("café", "caf\\u00e9")
        );
        let exact = if outcome == "fresh" {
            serde_json::to_string(&envelope).unwrap()
        } else {
            reconstructed
        };
        if outcome != "fresh" {
            let store = crate::auth::store::CredentialStore::open(&session.directory).unwrap();
            store
                .record_command(
                    &expected.id(),
                    &serde_json::json!({
                        "schema_version":2,"farmd":session.farmd,"origin":session.origin,
                        "envelope":envelope,"request_bytes":exact
                    }),
                )
                .unwrap();
            drop(store);
        }
        let body = serde_json::json!({"id":expected.id().as_str(),"kind":expected.kind,"payload_digest":expected.digest().to_hex(),"status":"PENDING","result":null}).to_string();
        let sid = format!("sid_{}", "c".repeat(64));
        let (sent, received) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            let client = scope.spawn(|| {
                sent.send(super::super::submit(
                    &session,
                    SubmitRequest {
                        payload: &payload,
                        idempotency_key: Some("acknowledged-submit"),
                    },
                ))
                .unwrap();
            });
            let (mut discovery, request) =
                submission_socket(&listener, "GET /api/v1/auth/session HTTP/1.1\r\n");
            let original = std::fs::read(&journal).expect("journal must commit before discovery");
            assert_eq!(
                serde_json::from_slice::<Value>(&original).unwrap()["envelope"],
                envelope
            );
            assert!(!request.contains("x-bullet-expected-session:"));
            assert!(!request.contains("csrf"));
            let view = serde_json::json!({"status":"AUTHENTICATED","operator_id":format!("opr_{}","d".repeat(64)),"session_id":sid,"issued_at":"2026-09-10T00:00:00Z","expires_at":"2026-09-10T08:00:00Z"}).to_string();
            write!(discovery,"HTTP/1.1 200 OK\r\nx-bullet-session-id: {sid}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{view}",view.len()).unwrap();
            drop(discovery);
            let (mut socket, request) =
                submission_socket(&listener, "POST /api/v1/commands HTTP/1.1\r\n");
            assert_eq!(
                request
                    .lines()
                    .filter(|line| line.starts_with("x-bullet-expected-session:"))
                    .collect::<Vec<_>>(),
                vec![format!("x-bullet-expected-session: {sid}")]
            );
            for (name, value) in [
                ("cookie", &session.cookie),
                ("origin", &session.origin),
                ("x-bullet-csrf", &session.csrf),
            ] {
                assert!(request.contains(&format!("\r\n{name}: {value}\r\n")));
            }
            let length: usize = request
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .unwrap()
                .parse()
                .unwrap();
            assert!(length < 16_384);
            let mut submitted = vec![0; length];
            socket.read_exact(&mut submitted).unwrap();
            assert_eq!(
                serde_json::from_slice::<Value>(&submitted).unwrap(),
                envelope
            );
            assert_eq!(
                submitted,
                serde_json::from_slice::<Value>(&original).unwrap()["request_bytes"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
            );
            let acknowledgement = match outcome {
                "missing" | "absent" => String::new(),
                "foreign" => format!("x-bullet-session-id: sid_{}\r\n", "e".repeat(64)),
                "duplicate" => format!("x-bullet-session-id: {sid}\r\n").repeat(2),
                _ => format!("x-bullet-session-id: {sid}\r\n"),
            };
            let result = if outcome == "lost" {
                drop(socket);
                received.recv_timeout(Duration::from_secs(2)).unwrap()
            } else {
                let status = if outcome == "absent" { 404 } else { 202 };
                write!(socket,"HTTP/1.1 {status} Fixture\r\n{acknowledgement}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n",body.len()).unwrap();
                socket.flush().unwrap();
                let early = received.recv_timeout(Duration::from_millis(
                    if matches!(outcome, "valid" | "fresh") {
                        100
                    } else {
                        2000
                    },
                ));
                if matches!(outcome, "valid" | "fresh") {
                    assert!(matches!(
                        early,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                    ));
                    socket.write_all(body.as_bytes()).unwrap();
                    received.recv_timeout(Duration::from_secs(2)).unwrap()
                } else {
                    let _ = socket.write_all(body.as_bytes());
                    early.expect("session refusal must precede body release")
                }
            };
            client.join().unwrap();
            if matches!(outcome, "valid" | "fresh") {
                assert_eq!(result.unwrap()["id"], expected.id().as_str());
            } else {
                let reason = match outcome {
                    "missing" | "absent" => "FARMD_SESSION_ACK_MISSING",
                    "foreign" => "FARMD_SESSION_ACK_MISMATCH",
                    "duplicate" => "FARMD_SESSION_ACK_AMBIGUOUS",
                    _ => "FARMD_REQUEST_FAILED",
                };
                let error = result.unwrap_err();
                assert!(error.starts_with(reason), "{outcome}: {error}");
                assert!(error.contains(expected.id().as_str()));
            }
            assert_eq!(std::fs::read(&journal).unwrap(), original);
        });
        let reopened = Session {
            directory: session.directory.clone(),
            credentials: session.credentials,
        };
        let (retried, _) = reconciliation_body(&reopened, expected.id().as_str())
            .unwrap()
            .unwrap();
        assert_eq!(retried.bytes, exact);
        assert_eq!(retried.provenance, "RECORDED_EXACT");
        assert_eq!(
            reconciliation_request(&reopened, expected.id().as_str())
                .unwrap()
                .unwrap(),
            expected
        );
        assert_eq!(
            std::fs::read_dir(&reopened.directory)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("cmd_"))
                .count(),
            1
        );
    }
}
