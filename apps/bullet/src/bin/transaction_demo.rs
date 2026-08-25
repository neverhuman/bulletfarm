//! Offline five-plane TRANSACTION_PROOF saga. `run_demo` stays the
//! projection seeder. This binary is what `just demo` drives.

use bullet_adapters::SqliteLedger;
use bullet_application::{materialize_plan, CommandRequest, EffectState, Ledger, PlanInput};
use bullet_domain::{
    AttemptState, Digest, GateOutcome, RunnerId, TaskClass, REASON_ZERO_TESTS, REPOSITORY_GATE_ID,
};
use bullet_effects_core::{
    authorize, dispatch, propose, reconcile, IntentInput, LocalBareForge, LossMode,
    LostResponseForge, ReconcileOutcome, ZERO_OID,
};
use bullet_harness_core::lease_transport::LeaseTransportSigningKey;
use bullet_harness_core::proposal::{PatchMutation, PatchOperation, PatchProposal, Preimage};
use bullet_harness_core::transaction_proof::{
    TransactionProofSigningKey, TransactionProofSubject, TRANSACTION_PROOF_CLASS,
    TRANSACTION_PROOF_SCHEMA_VERSION,
};
use bullet_runner_core::lease::{AcquireRequest, HeartbeatCall, LeaseClient, ReleaseCall};
use bullet_runner_core::{gitd_fixture_binary, GitdSession, SignedLeaseRpcClient};
use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const FIXTURE_KEY: [u8; 32] = [0x5a; 32];
const ISSUER: &str = "kernel-local";
const KEY_ID: &str = "lease-1";

fn fail(message: impl Into<String>) -> String {
    message.into()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn now_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis(),
    )
    .expect("millis")
}

fn data_dir() -> PathBuf {
    std::env::var("BULLET_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./target/demo"))
}

fn kernel_bin(name: &str) -> PathBuf {
    let env = format!("BULLET_{}_BIN", name.to_ascii_uppercase().replace('-', "_"));
    if let Some(path) = std::env::var_os(&env) {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug")
        .join(name)
}

fn private_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| fail(format!("create {}: {err}", path.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|err| fail(format!("chmod {}: {err}", path.display())))?;
    }
    Ok(())
}

fn frame(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    buf.extend_from_slice(bytes);
}

#[derive(Serialize)]
struct FixturePermitClaims {
    schema_version: String,
    attempt_id: String,
    attempt_fence: u64,
    workspace_nonce: String,
    workspace_generation: u64,
    fixture_root: String,
    expires_at_unix_ms: u64,
}

fn fixture_mac(key: &[u8; 32], claims: &FixturePermitClaims) -> String {
    let payload = serde_json::to_vec(claims).expect("claims");
    let mut buf = Vec::new();
    frame(&mut buf, b"bullet-gitd.fixture-permit.mac.v1");
    frame(&mut buf, key);
    frame(&mut buf, &payload);
    Digest::of(&buf).to_hex()
}

fn signed_token(token: &Value, fixture_root: &Path) -> Value {
    let claims = FixturePermitClaims {
        schema_version: "v1".into(),
        attempt_id: token["attempt_id"].as_str().expect("attempt").into(),
        attempt_fence: token["attempt_fence"].as_u64().expect("fence"),
        workspace_nonce: hex_encode(
            &serde_json::from_value::<Vec<u8>>(token["workspace_nonce"].clone())
                .expect("nonce bytes"),
        ),
        workspace_generation: 1,
        fixture_root: fixture_root.display().to_string(),
        expires_at_unix_ms: now_ms().saturating_add(30_000),
    };
    let mac = fixture_mac(&FIXTURE_KEY, &claims);
    let mut out = token.clone();
    out["fixture_permit"] = json!({ "claims": claims, "mac": mac });
    out
}

fn sh(dir: &Path, script: &str) -> Result<(), String> {
    let out = Command::new("sh")
        .arg("-ec")
        .arg(script)
        .current_dir(dir)
        .output()
        .map_err(|err| fail(format!("spawn git: {err}")))?;
    if !out.status.success() {
        return Err(fail(format!(
            "git: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

fn init_source(root: &Path) -> Result<(PathBuf, String), String> {
    let src = root.join("source");
    fs::create_dir_all(src.join("src")).map_err(|err| fail(err.to_string()))?;
    fs::write(src.join("src").join("lib.rs"), "pub fn seed() {}\n")
        .map_err(|err| fail(err.to_string()))?;
    sh(
        &src,
        "git init -q -b main . && git config user.name bullet && git config user.email bullet@test && git add . && git commit -qm seed",
    )?;
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&src)
        .output()
        .map_err(|err| fail(err.to_string()))?;
    let hex = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((src, format!("sha1:{hex}")))
}

fn wait_for(path: &Path, tries: u32) -> Result<(), String> {
    for _ in 0..tries {
        if path.exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(fail(format!("timed out waiting for {}", path.display())))
}

fn free_port() -> Result<u16, String> {
    Ok(TcpListener::bind("127.0.0.1:0")
        .map_err(|err| fail(err.to_string()))?
        .local_addr()
        .map_err(|err| fail(err.to_string()))?
        .port())
}

fn spawn_farmd(
    data: &Path,
    socket: &Path,
    verify_hex: &str,
    port: u16,
) -> Result<std::process::Child, String> {
    let bin = kernel_bin("bullet-farmd");
    if !bin.is_file() {
        return Err(fail(format!(
            "bullet-farmd missing at {} (build -p bullet-farmd)",
            bin.display()
        )));
    }
    Command::new(bin)
        .arg("--data-dir")
        .arg(data)
        .arg("--bind")
        .arg(format!("127.0.0.1:{port}"))
        .arg("--lease-transport-socket")
        .arg(socket)
        .arg("--lease-transport-verify-hex")
        .arg(verify_hex)
        .arg("--lease-transport-issuer")
        .arg(ISSUER)
        .arg("--lease-transport-key-id")
        .arg(KEY_ID)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| fail(format!("spawn farmd: {err}")))
}

fn run_verifier(
    workspace: &Path,
    base: &str,
    head: &str,
    tree: &str,
    attempt: &str,
    overlap: bool,
) -> Result<(i32, Value), String> {
    let bin = kernel_bin("bullet-verifier");
    if !bin.is_file() {
        return Err(fail(format!(
            "bullet-verifier missing at {}",
            bin.display()
        )));
    }
    let request = json!({
        "workspace_repo_path": workspace.display().to_string(),
        "base_sha": base,
        "head_sha": head,
        "tree_sha": tree,
        "gate_id": REPOSITORY_GATE_ID,
        "author_attempt_id": attempt,
    });
    let mut cmd = Command::new(bin);
    cmd.arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if overlap {
        cmd.env("BULLET_VERIFIER_AUTHOR_OVERLAP", "1");
    }
    let mut child = cmd
        .spawn()
        .map_err(|err| fail(format!("spawn verifier: {err}")))?;
    use std::io::Write as _;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| fail("verifier stdin"))?
        .write_all(request.to_string().as_bytes())
        .map_err(|err| fail(err.to_string()))?;
    let out = child
        .wait_with_output()
        .map_err(|err| fail(err.to_string()))?;
    let text = if out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stderr).into_owned()
    } else {
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let value = serde_json::from_str(text.trim()).unwrap_or(json!({ "raw": text.trim() }));
    Ok((out.status.code().unwrap_or(1), value))
}

fn strip_oid(oid: &str) -> &str {
    oid.rsplit(':').next().unwrap_or(oid)
}

async fn run() -> Result<(), String> {
    let data = data_dir();
    private_dir(&data)?;
    let db = data.join("ledger.sqlite");
    let mut ledger = SqliteLedger::open(&db).map_err(|err| fail(err.to_string()))?;
    let now = Utc::now().to_rfc3339();
    let graph = materialize_plan(
        &mut ledger,
        "txn-proof-demo",
        &PlanInput {
            title: "Offline TRANSACTION_PROOF".into(),
            objective: "Five-plane signed demo without live providers.".into(),
            packages: vec![(
                "signed offline transaction".into(),
                TaskClass::BoundedBugFix,
            )],
        },
        &now,
    )
    .map_err(|err| fail(err.to_string()))?;
    let package = graph
        .packages
        .first()
        .ok_or_else(|| fail("demo graph has no package"))?;
    let command = CommandRequest::new(
        "txn-proof-demo",
        "run_demo",
        &json!({ "evidence_class": TRANSACTION_PROOF_CLASS }),
    )
    .map_err(|err| fail(err.to_string()))?;
    let command = ledger
        .submit_command(&command)
        .map_err(|err| fail(err.to_string()))?;
    drop(ledger);

    let lease_key =
        LeaseTransportSigningKey::generate(ISSUER, KEY_ID).map_err(|err| fail(err.to_string()))?;
    let socket = data.join("lease-transport.sock");
    let port = free_port()?;
    let mut farmd = spawn_farmd(&data, &socket, lease_key.public_hex(), port)?;
    wait_for(&socket, 80)?;

    let farmd_url = format!("http://127.0.0.1:{port}");
    let client = SignedLeaseRpcClient::new(socket.clone(), lease_key, &farmd_url)
        .map_err(|err| fail(err.to_string()))?;
    let runner = RunnerId::from_seed("txn-demo-runner");
    let first = client
        .acquire(&AcquireRequest {
            work_package_id: package.id.clone(),
            runner_id: runner.clone(),
            runner_epoch: 1,
            idempotency_key: "txn-demo-a1".into(),
            ttl_seconds: 15,
        })
        .await
        .map_err(|err| fail(err.to_string()))?;
    if first.attempt.fence != 1 {
        let _ = farmd.kill();
        return Err(fail(format!(
            "first fence was {}, expected 1",
            first.attempt.fence
        )));
    }

    let fixture_bin = gitd_fixture_binary();
    if !fixture_bin.is_file() {
        let _ = farmd.kill();
        return Err(fail(format!(
            "bullet-gitd-fixture missing at {} (build --features fixture-authority)",
            fixture_bin.display()
        )));
    }
    let scratch = std::env::temp_dir().join(format!("bullet-txn-{}", std::process::id()));
    private_dir(&scratch)?;
    let source_root = scratch.join("src-root");
    private_dir(&source_root)?;
    let (source, base) = init_source(&source_root)?;
    let fixture_root = scratch.join("farm");
    private_dir(&fixture_root)?;
    let token =
        serde_json::to_value(&first.authority_token).map_err(|err| fail(err.to_string()))?;
    let signed = signed_token(&token, &fixture_root);
    let mut gitd = GitdSession::spawn_with(
        fixture_bin,
        [
            "--fixture-root",
            fixture_root.to_str().ok_or_else(|| fail("root utf8"))?,
            "--fixture-key-hex",
            &hex_encode(&FIXTURE_KEY),
        ],
        signed.clone(),
    )
    .await
    .map_err(|err| fail(err.to_string()))?;

    let workspace = gitd
        .clone_workspace(&source, &base, &fixture_root, &["src".into()])
        .await
        .map_err(|err| fail(err.to_string()))?;
    let cleanup_root = scratch.join("cleanup-neg");
    private_dir(&cleanup_root)?;
    let mut cleanup_gitd = GitdSession::spawn_with(
        gitd_fixture_binary(),
        [
            "--fixture-root",
            cleanup_root.to_str().ok_or_else(|| fail("root utf8"))?,
            "--fixture-key-hex",
            &hex_encode(&FIXTURE_KEY),
        ],
        signed_token(&token, &cleanup_root),
    )
    .await
    .map_err(|err| fail(err.to_string()))?;
    cleanup_gitd
        .clone_workspace(&source, &base, &cleanup_root, &["src".into()])
        .await
        .map_err(|err| fail(err.to_string()))?;
    let cleanup_before = cleanup_gitd
        .invoke(
            "cleanup",
            json!({
                "preservation_receipt": "not-a-receipt",
                "deleted_at": now,
            }),
        )
        .await;
    let _ = cleanup_gitd.kill();
    if cleanup_before.is_ok() {
        let _ = gitd.kill();
        let _ = farmd.kill();
        return Err(fail("cleanup-before-preserve succeeded"));
    }

    let proposal = PatchProposal {
        schema_version: 1,
        proposal_id: format!("cnt_{}", Digest::of(b"txn-demo-proposal").to_hex()),
        producing_attempt_id: first.attempt.id.to_string(),
        base_checkpoint_id: workspace.base_checkpoint_id.clone(),
        base_checkpoint_digest: workspace.base_checkpoint_digest.clone(),
        operations: vec![PatchOperation {
            path: "src/lib.rs".into(),
            preimage: Preimage::Absent,
            mutation: PatchMutation::Write {
                content_utf8: "pub fn demo() {}\n".into(),
            },
        }],
        gate_ids: vec![REPOSITORY_GATE_ID.into()],
        intent_summary: String::new(),
        claims: Vec::new(),
        uncertainties: Vec::new(),
        done: true,
    };
    // src/lib.rs exists in the seed; preimage must match.
    let lib_digest = Digest::of(b"pub fn seed() {}\n").to_hex();
    let mut proposal = proposal;
    proposal.operations[0].preimage = Preimage::Digest { digest: lib_digest };
    let applied = gitd
        .apply_proposal(&proposal)
        .await
        .map_err(|err| fail(err.to_string()))?;
    let checkpoint = gitd
        .checkpoint()
        .await
        .map_err(|err| fail(err.to_string()))?;
    let checkpoint_id = checkpoint
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(&applied.checkpoint.id)
        .to_string();

    let prepare = gitd
        .invoke(
            "prepare_candidate",
            json!({
                "change": {
                    "id": format!("chg_{}", Digest::of(b"txn-demo-change").to_hex()),
                    "mission": "txn-proof-demo",
                    "acceptance_root": Digest::of(b"acc").to_hex()
                },
                "provenance": {
                    "schema_version": 1,
                    "repository_id": first.authority_token.repository_id.to_string(),
                    "producing_attempt_id": first.attempt.id.to_string(),
                    "attempt_fence": first.attempt.fence,
                    "work_package_id": package.id.to_string(),
                    "variant_id": first.authority_token.variant_id.to_string(),
                    "plan_revision_id": first.authority_token.plan_revision_id.to_string(),
                    "graph_revision_id": format!("grf_{}", Digest::of(b"txn-demo-graph").to_hex()),
                    "base_checkpoint_id": checkpoint_id,
                    "base_commit": workspace.base_sha,
                    "parent_candidate_ids": [],
                    "granted_scope": ["src"],
                    "context_capsule_id": format!("cnt_{}", Digest::of(b"ctx").to_hex()),
                    "configuration_snapshot_id": format!("cnt_{}", Digest::of(b"cfg").to_hex()),
                    "policy_snapshot_id": format!("cnt_{}", Digest::of(b"pol").to_hex()),
                    "routing_snapshot_id": format!("cnt_{}", Digest::of(b"rte").to_hex()),
                    "environment_digest": Digest::of(b"env").to_hex(),
                    "toolchain_digest": Digest::of(b"tool").to_hex()
                }
            }),
        )
        .await
        .map_err(|err| fail(format!("prepare_candidate: {err}")))?;
    let candidate_id = prepare
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("candidate id missing: {prepare}")))?
        .to_string();
    let head = prepare
        .get("head_commit")
        .or_else(|| prepare.pointer("/manifest/head_commit"))
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("head missing: {prepare}")))?
        .to_string();
    let tree = prepare
        .get("tree_hash")
        .or_else(|| prepare.get("tree_oid"))
        .or_else(|| prepare.pointer("/manifest/tree_oid"))
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("tree missing: {prepare}")))?
        .to_string();
    let repo_dir = workspace.repo_dir.clone();

    let (writer_code, writer_body) = run_verifier(
        &repo_dir,
        strip_oid(&workspace.base_sha),
        strip_oid(&head),
        strip_oid(&tree),
        first.attempt.id.as_str(),
        true,
    )?;
    let writer_proof_refused = writer_code != 0
        && writer_body
            .get("reason_code")
            .and_then(Value::as_str)
            .is_some_and(|code| code == "VERIFIER_IS_AUTHOR");
    let (verifier_code, verifier_body) = run_verifier(
        &repo_dir,
        strip_oid(&workspace.base_sha),
        strip_oid(&head),
        strip_oid(&tree),
        first.attempt.id.as_str(),
        false,
    )?;
    let verifier_outcome = verifier_body
        .get("outcome")
        .or_else(|| verifier_body.get("result"))
        .and_then(Value::as_str)
        .unwrap_or(if verifier_code == 0 { "PASS" } else { "FAIL" })
        .to_string();
    let _ = (
        verifier_outcome.clone(),
        GateOutcome::Fail,
        REASON_ZERO_TESTS,
    );

    let effects_root = data.join("effects");
    fs::create_dir_all(&effects_root).map_err(|err| fail(err.to_string()))?;
    let workspace_git = effects_root.join("workspace");
    fs::create_dir_all(&workspace_git).map_err(|err| fail(err.to_string()))?;
    sh(
        &workspace_git,
        "git init -q -b main . && git config user.name bullet && git config user.email bullet@test && echo demo > f && git add . && git commit -qm demo",
    )?;
    let head_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&workspace_git)
        .output()
        .map_err(|err| fail(err.to_string()))?;
    let effect_head = String::from_utf8_lossy(&head_out.stdout).trim().to_string();
    let mut effects_ledger = SqliteLedger::open(&db).map_err(|err| fail(err.to_string()))?;
    let token_ref = &first.authority_token;
    let forge = LocalBareForge::init(&effects_root.join("target.git"))
        .map_err(|err| fail(err.to_string()))?;
    let intent = IntentInput {
        provider: "local-bare".into(),
        logical_effect_key: format!("push:txn:{}", first.attempt.fence),
        target_ref: "refs/heads/bullet/candidate/txn-demo".into(),
        new_oid: effect_head.clone(),
        expected_old_oid: ZERO_OID.into(),
        attempt_id: token_ref.attempt_id.clone(),
        fence: token_ref.attempt_fence,
        policy_version: "policy-v1".into(),
        provider_idempotency_key: None,
    };
    let (row, _) =
        propose(&mut effects_ledger, &intent, &now).map_err(|err| fail(err.to_string()))?;
    let (_authorized, seq) = authorize(&mut effects_ledger, &row.id, token_ref, &now)
        .map_err(|err| fail(err.to_string()))?;
    let mut lossy = LostResponseForge::new(forge);
    lossy.lose_next(LossMode::AfterPush);
    let unknown = dispatch(
        &mut effects_ledger,
        &mut lossy,
        &row.id,
        &workspace_git,
        Some(seq),
        &now,
    )
    .map_err(|err| fail(err.to_string()))?;
    if unknown != EffectState::OutcomeUnknown {
        let _ = gitd.kill();
        let _ = farmd.kill();
        return Err(fail(format!("lost response was {unknown:?}, not UNKNOWN")));
    }
    let adopted = reconcile(
        &mut effects_ledger,
        &mut lossy,
        &row.id,
        &workspace_git,
        Some(seq),
        &now,
    )
    .map_err(|err| fail(err.to_string()))?;
    if adopted != ReconcileOutcome::Adopted {
        let _ = gitd.kill();
        let _ = farmd.kill();
        return Err(fail(format!("expected Adopted, got {adopted:?}")));
    }
    let settled = effects_ledger
        .get_effect_intent_by_id(&row.id)
        .map_err(|err| fail(err.to_string()))?
        .map(|record| record.state)
        .ok_or_else(|| fail("settled intent missing"))?;
    drop(effects_ledger);

    let preserve_parent = scratch.join("preserve-parent");
    private_dir(&preserve_parent)?;
    let preserve_to = preserve_parent.join("artifact");
    gitd.invoke(
        "preserve",
        json!({ "destination": preserve_to.display().to_string() }),
    )
    .await
    .map_err(|err| fail(format!("preserve: {err}")))?;
    gitd.kill().map_err(|err| fail(err.to_string()))?;
    client
        .release(&ReleaseCall {
            attempt_id: first.attempt.id.clone(),
            outcome: AttemptState::Superseded,
            requeue: true,
        })
        .await
        .map_err(|err| fail(err.to_string()))?;

    let second = client
        .acquire(&AcquireRequest {
            work_package_id: package.id.clone(),
            runner_id: runner.clone(),
            runner_epoch: 1,
            idempotency_key: "txn-demo-a2".into(),
            ttl_seconds: 15,
        })
        .await
        .map_err(|err| fail(err.to_string()))?;
    if second.attempt.fence != 2 {
        let _ = farmd.kill();
        return Err(fail(format!(
            "successor fence was {}, expected 2",
            second.attempt.fence
        )));
    }
    let stale = client
        .heartbeat(&HeartbeatCall {
            variant_id: first.lease.variant_id.clone(),
            attempt_id: first.attempt.id.clone(),
            fence: first.attempt.fence,
            runner_id: runner,
            runner_epoch: 1,
            workspace_nonce: first.authority_token.workspace_nonce,
            ttl_seconds: 15,
        })
        .await;
    let stale_refused = stale.is_err();
    if !stale_refused {
        let _ = farmd.kill();
        return Err(fail("stale fence heartbeat was accepted"));
    }

    let subject = TransactionProofSubject {
        schema_version: TRANSACTION_PROOF_SCHEMA_VERSION.into(),
        evidence_class: TRANSACTION_PROOF_CLASS.into(),
        fence_first: first.attempt.fence,
        fence_second: second.attempt.fence,
        attempt_first: first.attempt.id.to_string(),
        attempt_second: second.attempt.id.to_string(),
        candidate_id,
        verifier_outcome,
        writer_proof_refused,
        effect_unknown: unknown.as_str().into(),
        effect_settled: settled.as_str().into(),
        stale_refused,
        gitd_fixture: true,
        command_id: command.id.to_string(),
        command_phase: command.phase.as_str().into(),
    };
    if !writer_proof_refused {
        let _ = farmd.kill();
        return Err(fail(format!("writer proof was not refused: {writer_body}")));
    }
    let proof_key = TransactionProofSigningKey::generate("kernel-demo", "txn-proof-1")
        .map_err(|err| fail(err.to_string()))?;
    let proof = proof_key
        .sign(&subject)
        .map_err(|err| fail(err.to_string()))?;
    let proof_json = serde_json::to_string_pretty(&proof).map_err(|err| fail(err.to_string()))?;
    let proof_path = data.join("TRANSACTION_PROOF.json");
    fs::write(&proof_path, &proof_json).map_err(|err| fail(err.to_string()))?;
    let projection = json!({
        "command_id": command.id,
        "command_phase": command.phase.as_str(),
        "effect_unknown": unknown.as_str(),
        "painted_as_success": false,
        "farmd": farmd_url,
    });
    fs::write(
        data.join("projection.json"),
        serde_json::to_string_pretty(&projection).map_err(|err| fail(err.to_string()))?,
    )
    .map_err(|err| fail(err.to_string()))?;
    println!("{proof_json}");
    println!("TRANSACTION_PROOF: {}", proof_path.display());
    let _ = farmd.kill();
    Ok(())
}

fn main() -> ExitCode {
    match tokio::runtime::Runtime::new()
        .expect("tokio")
        .block_on(run())
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("bullet-transaction-demo: {message}");
            ExitCode::FAILURE
        }
    }
}
