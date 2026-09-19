use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use bf::api::{router, AppState};
use bf::Hub;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

const ORIGIN: &str = "http://127.0.0.1:9";

fn app(hub: &Arc<Hub>) -> Router {
    router(AppState::new(
        hub.clone(),
        ORIGIN.into(),
        "boot-secret".into(),
    ))
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn bootstrap(app: &Router) -> String {
    let (status, body) = send(
        app,
        Request::post(format!("{ORIGIN}/v3/bootstrap"))
            .header("host", "127.0.0.1:9")
            .header("authorization", "Bearer boot-secret")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["token"].as_str().unwrap().to_owned()
}

async fn post_command(app: &Router, token: &str, raw: &[u8]) -> (StatusCode, Value) {
    send(
        app,
        Request::post(format!("{ORIGIN}/v3/commands"))
            .header("host", "127.0.0.1:9")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(raw.to_vec()))
            .unwrap(),
    )
    .await
}

fn command(id: &str, kind: &str, text: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema_version": 3, "command_id": id, "kind": kind, "payload": {"text": text}
    }))
    .unwrap()
}

/// Rows in `table` for `command_id`, read through a second connection to the hub file.
fn rows(dir: &Path, table: &str, command_id: &str) -> i64 {
    let conn = rusqlite::Connection::open(dir.join("hub.sqlite")).unwrap();
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE command_id=?1"),
        [command_id],
        |r| r.get(0),
    )
    .unwrap()
}

fn assert_error(body: &Value, code: &str) {
    assert_eq!(body["error"], code, "{body}");
    assert!(body["message"].is_string(), "{body}");
    let id = body["correlation_id"].as_str().expect("correlation_id");
    assert_eq!(id.len(), 36, "{id}");
    assert!(id.bytes().enumerate().all(|(i, b)| match i {
        8 | 13 | 18 | 23 => b == b'-',
        _ => b.is_ascii_hexdigit(),
    }));
}

#[test]
fn second_hub_is_refused() {
    let dir = TempDir::new().unwrap();
    let first = Hub::open(dir.path()).unwrap();
    let err = Hub::open(dir.path()).err().expect("second hub");
    assert_eq!(err.code(), "RESOURCE_CONFLICT");
    drop(first);
}

#[test]
fn doctor_reports_pinned_sqlite() {
    let dir = TempDir::new().unwrap();
    let hub = Hub::open(dir.path()).unwrap();
    let doctor = hub.doctor();
    assert_eq!(doctor["ok"], true);
    assert_eq!(doctor["sqlite"], "3.53.2");
}

#[tokio::test]
async fn http_auth_and_command_replay() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let origin = "http://127.0.0.1:9";
    let bootstrap = "boot-secret";
    let app = router(AppState::new(hub, origin.into(), bootstrap.into()));
    let health = app
        .clone()
        .oneshot(
            Request::get("http://127.0.0.1:9/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let denied = app
        .clone()
        .oneshot(
            Request::get("http://127.0.0.1:9/v3/doctor")
                .header("host", "127.0.0.1:9")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

    let login = app
        .clone()
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/bootstrap")
                .header("host", "127.0.0.1:9")
                .header("authorization", "Bearer boot-secret")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let token =
        serde_json::from_slice::<Value>(&login.into_body().collect().await.unwrap().to_bytes())
            .unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();

    let body = json!({"schema_version":3,"command_id":"n1","kind":"note","payload":{"text":"hi"}});
    let raw = serde_json::to_vec(&body).unwrap();
    let first = app
        .clone()
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/commands")
                .header("host", "127.0.0.1:9")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(raw.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::ACCEPTED);
    let one: Value =
        serde_json::from_slice(&first.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let second = app
        .oneshot(
            Request::post("http://127.0.0.1:9/v3/commands")
                .header("host", "127.0.0.1:9")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(raw))
                .unwrap(),
        )
        .await
        .unwrap();
    let two: Value =
        serde_json::from_slice(&second.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(one["id"], two["id"]);
}

#[tokio::test]
async fn replay_is_scoped_to_the_actor() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);
    let owner = bootstrap(&app).await;
    let agent = hub.ensure_session_kind("agent-a", "agent").unwrap();

    let raw = command("c1", "note", "hi");
    let (status, first) = post_command(&app, &owner, &raw).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let (status, again) = post_command(&app, &owner, &raw).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(first["id"], again["id"]);

    let (status, conflict) = post_command(&app, &owner, &command("c1", "note", "changed")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&conflict, "RESOURCE_CONFLICT");
    assert_eq!(rows(dir.path(), "operations", "c1"), 1);
    assert_eq!(rows(dir.path(), "commands", "c1"), 1);

    // Another principal owns its own c1: different bytes, no conflict, no leak.
    let (status, theirs) = post_command(&app, &agent, &command("c1", "note", "mine")).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{theirs}");
    assert_ne!(theirs["id"], first["id"]);
    assert_eq!(rows(dir.path(), "operations", "c1"), 2);
    // And the owner's replay is untouched by it.
    let (_, still) = post_command(&app, &owner, &raw).await;
    assert_eq!(still["id"], first["id"]);
}

#[test]
fn failed_storage_rolls_back_command_event_and_operation() {
    let dir = TempDir::new().unwrap();
    let hub = Hub::open(dir.path()).unwrap();
    let raw = command("c-fail", "note", "hi");
    let injector = rusqlite::Connection::open(dir.path().join("hub.sqlite")).unwrap();
    injector
        .execute_batch(
            "CREATE TRIGGER injected BEFORE INSERT ON operations
             BEGIN SELECT RAISE(ABORT, 'injected constraint'); END;",
        )
        .unwrap();

    let err = hub.command_bytes("owner", &raw).unwrap_err();
    assert_eq!(err.code(), "STORAGE_UNAVAILABLE");
    assert!(err.to_string().contains("injected"), "{err}");
    for table in ["commands", "events", "operations"] {
        assert_eq!(rows(dir.path(), table, "c-fail"), 0, "{table}");
    }

    // The id was not stranded: once storage recovers the same command lands whole.
    injector.execute_batch("DROP TRIGGER injected").unwrap();
    let op = hub.command_bytes("owner", &raw).unwrap();
    assert_eq!(op.status, "accepted");
    for table in ["commands", "events", "operations"] {
        assert_eq!(rows(dir.path(), table, "c-fail"), 1, "{table}");
    }
}

#[tokio::test]
async fn agents_and_runners_cannot_exercise_human_authority() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);
    let owner = bootstrap(&app).await;
    let agent = hub.ensure_session_kind("agent-a", "agent").unwrap();
    let runner = hub.ensure_session_kind("runner-1", "runner").unwrap();

    let grant = command("g1", "grant_allowance", "self-approve");
    let (status, denied) = post_command(&app, &agent, &grant).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&denied, "POLICY_DENIED");
    for table in ["commands", "events", "operations"] {
        assert_eq!(rows(dir.path(), table, "g1"), 0, "{table}");
    }
    for kind in ["resolve_decision", "take", "submit_human"] {
        let (status, body) = post_command(&app, &agent, &command("h1", kind, "x")).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{kind}: {body}");
        assert_error(&body, "POLICY_DENIED");
    }

    let (status, accepted) = post_command(&app, &owner, &grant).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{accepted}");
    assert_eq!(accepted["status"], "accepted");
    assert_eq!(accepted["result"]["error"], "NOT_IMPLEMENTED");
    assert_eq!(rows(dir.path(), "operations", "g1"), 1);

    let (status, agent_note) = post_command(&app, &agent, &command("n1", "note", "ok")).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{agent_note}");

    let (status, runner_note) = post_command(&app, &runner, &command("n2", "note", "no")).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{runner_note}");
    assert_error(&runner_note, "POLICY_DENIED");
    assert_eq!(rows(dir.path(), "commands", "n2"), 0);

    // A principal's kind is fixed: minting it under another kind is refused.
    let err = hub.ensure_session_kind("agent-a", "human").unwrap_err();
    assert_eq!(err.code(), "POLICY_DENIED");
}

#[tokio::test]
async fn inactive_principal_cannot_replay_a_known_id() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);
    let owner = bootstrap(&app).await;
    let raw = command("c1", "note", "hi");
    let (status, _) = post_command(&app, &owner, &raw).await;
    assert_eq!(status, StatusCode::ACCEPTED);

    hub.set_principal_active("owner", false).unwrap();
    let (status, body) = post_command(&app, &owner, &raw).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "AUTH_REQUIRED");
    // The kernel rechecks on its own, independent of session validation.
    let err = hub.command_bytes("owner", &raw).unwrap_err();
    assert_eq!(err.code(), "AUTH_REQUIRED");

    hub.set_principal_active("owner", true).unwrap();
    let (status, _) = post_command(&app, &owner, &raw).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn error_bodies_carry_a_fresh_correlation_id() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);
    let request = || {
        Request::get(format!("{ORIGIN}/v3/doctor"))
            .header("host", "127.0.0.1:9")
            .body(Body::empty())
            .unwrap()
    };
    let (status, first) = send(&app, request()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&first, "AUTH_REQUIRED");
    let (_, second) = send(&app, request()).await;
    assert_error(&second, "AUTH_REQUIRED");
    assert_ne!(first["correlation_id"], second["correlation_id"]);

    let owner = bootstrap(&app).await;
    let (status, bad) = post_command(&app, &owner, b"{\"schema_version\":3}").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&bad, "INVALID_CONTRACT");
}

async fn get_authed(app: &Router, token: &str, path: &str) -> axum::http::Response<Body> {
    app.clone()
        .oneshot(
            Request::get(format!("{ORIGIN}{path}"))
                .header("host", "127.0.0.1:9")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn page_reads_are_empty_authenticated_lists_not_404() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);

    let (status, body) = send(
        &app,
        Request::get(format!("{ORIGIN}/v3/projects"))
            .header("host", "127.0.0.1:9")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "AUTH_REQUIRED");

    let owner = bootstrap(&app).await;
    for path in ["/v3/projects", "/v3/drafts", "/v3/work"] {
        let response = get_authed(&app, &owner, path).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["items"], json!([]), "{path}: {value}");
    }
    let response = get_authed(&app, &owner, "/v3/drafts/missing").await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["conversation"], json!([]));
    assert_eq!(value["jobs"], json!([]));

    let response = get_authed(&app, &owner, "/v3/events").await;
    assert_eq!(response.status(), StatusCode::OK);
    let ctype = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ctype.starts_with("text/event-stream"),
        "content-type {ctype}"
    );
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.contains("data: {\"cursor\":0}"), "sse body:\n{text}");

    let doctor = get_authed(&app, &owner, "/v3/doctor").await;
    let bytes = doctor.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(value["provider"].is_object(), "{value}");
}

#[tokio::test]
async fn page_command_kinds_are_accepted_as_not_implemented() {
    let dir = TempDir::new().unwrap();
    let hub = Arc::new(Hub::open(dir.path()).unwrap());
    let app = app(&hub);
    let owner = bootstrap(&app).await;
    for kind in [
        "run",
        "remember_project",
        "edit_draft",
        "start_work",
        "retry_task",
        "create_mission",
        "pause",
        "stop",
        "cancel",
        "resume",
    ] {
        let (status, body) =
            post_command(&app, &owner, &command(&format!("pg-{kind}"), kind, "x")).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{kind}: {body}");
        assert_eq!(body["result"]["error"], "NOT_IMPLEMENTED", "{kind}: {body}");
    }
}
