#![cfg(target_os = "linux")]

#[path = "support/demo_gif_readiness.rs"]
mod readiness;

use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{Duration, Instant},
};

use serde_json::Value;

fn hub() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn private_tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir()
        .unwrap()
}

fn recorder(root: &Path, source: &str, options: &[&str]) -> Command {
    let mut command = Command::new("timeout");
    command
        .args(["--kill-after=1s", "7s", "python3"])
        .arg(hub().join("scripts/lib/demo-gif-pty-record.py"))
        .arg("--cast")
        .arg(root.join("session.cast"))
        .arg("--transcript")
        .arg(root.join("transcript.txt"))
        .args(["--max-seconds", "2"])
        .args(options)
        .args(["--", "python3", "-c", source]);
    command
}

fn report(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(root.join("session.cast.result.json")).unwrap()).unwrap()
}

fn run(source: &str, options: &[&str]) -> (tempfile::TempDir, Output, Value) {
    let root = private_tempdir();
    let result = recorder(root.path(), source, options).output().unwrap();
    assert_ne!(
        result.status.code(),
        Some(137),
        "outer deadline killed capture"
    );
    let receipt = report(root.path());
    (root, result, receipt)
}

#[test]
fn child_exit_and_signal_are_preserved_without_completion_credit() {
    for code in [0, 1, 129, 130, 143] {
        let (_, output, receipt) = run(&format!("print('OUTPUT', flush=True);exit({code})"), &[]);
        assert_eq!(output.status.code(), Some(code));
        assert_eq!(receipt["child_exit"], code);
        assert_eq!(receipt["stop_reason"], "CHILD_EXIT");
        assert_eq!(receipt["completion"], "UNVERIFIED");
        assert_eq!(receipt["bullet_live_admission"], false);
        assert_eq!(receipt["owned_group_gone"], true);
    }
    let root = private_tempdir();
    let missing = root.path().join("does-not-exist");
    let output = Command::new("timeout")
        .args(["--kill-after=1s", "7s", "python3"])
        .arg(hub().join("scripts/lib/demo-gif-pty-record.py"))
        .arg("--cast")
        .arg(root.path().join("session.cast"))
        .arg("--transcript")
        .arg(root.path().join("transcript.txt"))
        .args(["--max-seconds", "2", "--"])
        .arg(&missing)
        .output()
        .unwrap();
    assert!(!missing.exists());
    assert_eq!(output.status.code(), Some(127));
    let receipt = report(root.path());
    assert_eq!(receipt["child_started"], true); // Fork occurred; requested exec did not.
    assert_eq!(receipt["child_exit"], 127);
    assert_eq!(receipt["owned_group_gone"], true);
    assert_eq!(receipt["completion"], "UNVERIFIED");
    for signal in [1, 2, 15] {
        let (_, output, receipt) = run(
            &format!(
                "import os,signal;signal.signal({signal},signal.SIG_DFL);print('signal',flush=True);os.kill(os.getpid(),{signal})"
            ),
            &[],
        );
        assert_eq!(output.status.code(), Some(128 + signal));
        assert_eq!(receipt["child_signal"], signal);
        assert!(receipt["child_exit"].is_null());
        assert_eq!(receipt["owned_group_gone"], true);
    }
}

#[test]
fn timeout_escalates_ignored_signals_with_an_independent_bound() {
    let started = Instant::now();
    // Child signal dispositions must be acknowledged before the parent's
    // capture loop can time out; interpreter startup is not the escalation test.
    let root = private_tempdir();
    let output = readiness::capture(
        root.path(),
        &hub().join("scripts/lib/demo-gif-pty-record.py"),
    );
    let receipt = report(root.path());
    assert!(started.elapsed() < Duration::from_secs(3));
    assert_eq!(output.status.code(), Some(124));
    assert_eq!(receipt["stop_reason"], "TIMEOUT");
    assert_eq!(receipt["child_signal"], 9);
    assert_eq!(receipt["owned_group_gone"], true);
    assert_eq!(receipt["capture_limit_seconds"], 0.2);
    assert_eq!(receipt["cleanup_poll_allowance_seconds"], 1.4);
    assert_eq!(receipt["filesystem_io_deadline"], "NOT_GUARANTEED");
}

#[test]
fn terminal_eof_does_not_certify_a_still_running_child() {
    let (_, output, receipt) = run(
        "import os,signal,time;signal.alarm(5);print('closing',flush=True);os.close(0);os.close(1);os.close(2);time.sleep(10)",
        &[],
    );
    assert!(!output.status.success());
    assert_eq!(receipt["stop_reason"], "PTY_CLOSED_BEFORE_EXIT");
    assert_eq!(receipt["owned_group_gone"], true);
}

#[test]
fn output_and_event_limits_stop_capture_and_retain_bounded_bytes() {
    let (root, output, receipt) = run(
        "import os,signal;signal.alarm(5)\nwhile True: os.write(1,b'x'*8192)",
        &["--max-bytes", "1024"],
    );
    assert!(!output.status.success());
    assert_eq!(receipt["stop_reason"], "OUTPUT_LIMIT");
    assert_eq!(
        fs::metadata(root.path().join("session.cast.raw"))
            .unwrap()
            .len(),
        1024
    );
    let (_, output, receipt) = run(
        "import signal,time;signal.alarm(5)\nfor x in range(20): print(x,flush=True);time.sleep(0.05)",
        &["--max-events", "1"],
    );
    assert!(!output.status.success());
    assert_eq!(receipt["stop_reason"], "CAPTURE_ERROR");
    assert_eq!(receipt["error_class"], "OverflowError");
}

#[test]
fn invalid_bounds_and_existing_outputs_refuse_before_child_start() {
    for bound in ["nan", "inf", "-1", "0"] {
        let root = private_tempdir();
        let output = recorder(
            root.path(),
            "raise Exception('must not start')",
            &["--max-seconds", bound],
        )
        .output()
        .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
    let root = private_tempdir();
    let existing = root.path().join("session.cast");
    fs::write(&existing, "PRIOR ATTEMPT").unwrap();
    let marker = root.path().join("started");
    let source = format!("open({:?},'w').write('bad')", marker.to_str().unwrap());
    let output = recorder(root.path(), &source, &[]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(existing).unwrap(), "PRIOR ATTEMPT");
    assert!(!marker.exists());
}

#[test]
fn symlinks_and_shared_parents_cannot_redirect_private_capture() {
    let root = private_tempdir();
    let target = root.path().join("target");
    fs::write(&target, "PRIOR").unwrap();
    symlink(&target, root.path().join("session.cast")).unwrap();
    assert_eq!(
        recorder(root.path(), "print('bad')", &[])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fs::read_to_string(target).unwrap(), "PRIOR");
    let shared = root.path().join("shared");
    fs::create_dir(&shared).unwrap();
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        recorder(&shared, "print('bad')", &[])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fs::read_dir(shared).unwrap().count(), 0);
}

#[test]
fn capture_is_private_and_redaction_handles_chunk_boundaries() {
    let (root, output, receipt) = run(
        "import os,time;os.write(1,b'person@');time.sleep(0.1);os.write(1,b'example.invalid\\n')",
        &[],
    );
    assert!(output.status.success());
    assert!(
        fs::read_to_string(root.path().join("transcript.txt"))
            .unwrap()
            .contains("[redacted-email]")
    );
    assert!(
        fs::read_to_string(root.path().join("session.cast.raw"))
            .unwrap()
            .contains("person@example.invalid")
    );
    for name in [
        "session.cast",
        "session.cast.raw",
        "transcript.txt",
        "session.cast.result.json",
    ] {
        assert_eq!(
            fs::metadata(root.path().join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(receipt["raw_private"], true);
    assert_eq!(receipt["public_export_review"], "REQUIRED");
    assert_eq!(receipt["ownership"], "OWNED_PROCESS_GROUP_ONLY");
}

#[test]
fn recorder_interruption_retains_failure_and_terminates_its_group() {
    let root = private_tempdir();
    let child = recorder(
        root.path(),
        "import signal,time;signal.alarm(5);print('ready',flush=True);time.sleep(10)",
        &[],
    )
    .spawn()
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while fs::metadata(root.path().join("session.cast.raw")).map_or(true, |m| m.len() == 0) {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(i32::try_from(child.id()).unwrap()),
        nix::sys::signal::Signal::SIGTERM,
    )
    .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let receipt = report(root.path());
    assert_eq!(receipt["stop_reason"], "INTERRUPTED");
    assert_eq!(receipt["owned_group_gone"], true);
}

fn node(source: &str) -> Output {
    Command::new("timeout")
        .args([
            "--kill-after=1s",
            "8s",
            "node",
            "--input-type=module",
            "-e",
            source,
        ])
        .output()
        .unwrap()
}

fn portal_module() -> String {
    format!(
        "file://{}",
        hub().join("scripts/lib/demo-gif-portal-tour.mjs").display()
    )
}

#[test]
fn portal_observations_preserve_unknown_and_reject_conflicting_identity() {
    let source = format!(
        r#"
import assert from 'node:assert/strict';
import {{commandObservation,validateOrigin}} from '{}';
const input={{id:'cmd_example',kind:'run_demo',payload_digest:'a'.repeat(64),status:'UNKNOWN'}};
assert.equal(commandObservation(input).ledger_status,'UNKNOWN');
assert.equal(commandObservation(input).product_completion,'UNVERIFIED');
assert.equal(commandObservation({{...input,status:'VERIFIED'}}).product_completion,'UNVERIFIED');
for(const bad of [null,{{}},{{...input,status:'DONE'}},{{...input,kind:'other'}},{{...input,payload_digest:'bad'}}]) assert.throws(()=>commandObservation(bad));
assert.throws(()=>commandObservation(input,'another'));
assert.equal(validateOrigin('http://127.0.0.1:7421'),'http://127.0.0.1:7421');
for(const bad of ['https://example.com','http://user:pass@127.0.0.1:7421','http://127.0.0.1:7421/path','http://127.0.0.1:7421?secret']) assert.throws(()=>validateOrigin(bad));
"#,
        portal_module()
    );
    let output = node(&source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn portal_failure_retains_observation_and_closes_browser_without_auth_calls() {
    let root = private_tempdir();
    let package = root.path().join("package.json");
    fs::write(&package, "{}").unwrap();
    let module = root.path().join("node_modules/playwright");
    fs::create_dir_all(&module).unwrap();
    fs::write(module.join("index.js"), format!(r#"
const fs=require('node:fs');
exports.chromium={{launch:async()=>({{
  newContext:async()=>({{newPage:async()=>{{throw Error('synthetic failure before navigation')}},close:async()=>fs.writeFileSync({:?},'closed')}}),
  close:async()=>fs.writeFileSync({:?},'closed')
}})}};
"#, root.path().join("context.closed"), root.path().join("browser.closed"))).unwrap();
    let out = root.path().join("capture");
    let source = format!(
        r#"
import assert from 'node:assert/strict';
import {{capturePortal}} from '{}';
const env={{PORTAL_ORIGIN:'http://127.0.0.1:7421',PORTAL_CAPTURE_DIR:{:?},PORTAL_PACKAGE_JSON:{:?},BULLET_BOOTSTRAP_TOKEN:'synthetic-not-a-credential'}};
await assert.rejects(capturePortal(env));
await assert.rejects(capturePortal(env));
"#,
        portal_module(),
        out,
        package
    );
    let output = node(&source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value =
        serde_json::from_slice(&fs::read(out.join("observation.json")).unwrap()).unwrap();
    assert_eq!(receipt["capture_status"], "FAILED");
    assert_eq!(receipt["product_completion"], "UNVERIFIED");
    assert_eq!(receipt["steps"].as_array().unwrap().len(), 0);
    assert!(root.path().join("context.closed").exists());
    assert!(root.path().join("browser.closed").exists());
    assert!(
        !fs::read_to_string(out.join("observation.json"))
            .unwrap()
            .contains("synthetic-not-a-credential")
    );
}

#[test]
fn synthetic_campaign_retains_failed_attempts_and_requires_portal_observation() {
    let root = private_tempdir();
    let source = r#"
import json,os,shutil,socket,subprocess,sys
from pathlib import Path
source,root=map(Path,sys.argv[1:])
family=root/'family'; hub=family/'bullet-farm'; portal=family/'bullet-portal'
(hub/'scripts/lib').mkdir(parents=True)
for name in ['demo-gif-record.sh','lib/demo-gif-pty-record.py','lib/demo-gif-portal-tour.mjs']:
    shutil.copyfile(source/'scripts'/name,hub/'scripts'/name)
for path in [portal/'dist',portal/'node_modules/playwright']: path.mkdir(parents=True)
(portal/'package.json').write_text('{}')
bin=root/'bin';bin.mkdir()
# These exact fixture executables shadow every provider, HTTP and browser command.
fixture='''#!/usr/bin/env python3
import json,os,signal,sys,time
from pathlib import Path
signal.alarm(10)
name=Path(sys.argv[0]).name
if name in ('claude','codex','cursor-agent'):
    print('echoed prompt is not completion',flush=True)
    if os.environ['SYNTHETIC_PORTAL_RESULT']=='cancel_signal':
        signal.signal(signal.SIGINT,signal.SIG_DFL);os.kill(os.getpid(),signal.SIGINT)
    if os.environ['SYNTHETIC_PORTAL_RESULT']=='cancel_exit': sys.exit(130)
    sys.exit(0 if '-p' in sys.argv or 'exec' in sys.argv else 1)
if name in ('farmd','npm'):
    print('Bullet Farm one-time bootstrap: boot_'+'a'*32,flush=True)
    if name=='npm' and os.environ['SYNTHETIC_PORTAL_RESULT']=='early': sys.exit(1)
    time.sleep(10)
elif name=='curl':
    paths=list(Path(os.environ['BULLET_DEMO_GIF_CACHE']).glob('run-*/farmd/session.cast.raw'))
    sys.exit(0 if paths and all(p.stat().st_size for p in paths) else 1)
elif name=='node':
    if os.environ['SYNTHETIC_PORTAL_RESULT'] in ('present','early'):
        out=Path(os.environ['PORTAL_CAPTURE_DIR']);out.mkdir(mode=0o700)
        (out/'observation.json').write_text(json.dumps({
            'capture_status':'CAPTURED','product_completion':'UNVERIFIED','bullet_live_admission':False}))
    print('synthetic browser fixture only',flush=True)
'''
for name in ['claude','codex','cursor-agent','farmd','npm','curl','node']:
    path=bin/name;path.write_text(fixture);path.chmod(0o700)
cache=root/'cache';cache.mkdir(mode=0o700)
old=cache/'farmd-data';old.mkdir()
for name in ['ledger.sqlite','ledger.sqlite-wal','ledger.sqlite-shm']: (old/name).write_text('PRIOR DATABASE')
env=os.environ|{'PATH':str(bin)+os.pathsep+os.environ['PATH'],'HOME':str(root),
    'BULLET_DEMO_GIF_CACHE':str(cache),'BULLET_FARMD_BIN':str(bin/'farmd')}
ports=[socket.socket(),socket.socket()]
for sock in ports: sock.bind(('127.0.0.1',0))
env['BULLET_DEMO_FARMD_BIND']='127.0.0.1:'+str(ports[0].getsockname()[1])
env['BULLET_DEMO_PORTAL_BIND']='127.0.0.1:'+str(ports[1].getsockname()[1])
for sock in ports: sock.close()
prior=set()
for portal_result,expected in [('present',0),('absent',1),('early',1),('cancel_signal',130),('cancel_exit',130),('occupied',1)]:
    occupied=None
    if portal_result=='occupied':
        occupied=socket.socket();occupied.bind(('127.0.0.1',0));occupied.listen(1)
        env['BULLET_DEMO_PORTAL_BIND']='127.0.0.1:'+str(occupied.getsockname()[1])
    result=subprocess.run(['timeout','--kill-after=1s','15s','bash',str(hub/'scripts/demo-gif-record.sh')],
        env=env|{'SYNTHETIC_PORTAL_RESULT':portal_result},capture_output=True,timeout=18)
    if occupied: occupied.close()
    assert result.returncode==expected,(result.returncode,result.stdout,result.stderr)
    runs=set(cache.glob('run-*'));added=runs-prior;assert len(added)==1
    run=added.pop();prior=runs
    assert (run/'campaign.exit').read_text().strip()==str(expected)
    for path in old.iterdir(): assert path.read_text()=='PRIOR DATABASE'
    if portal_result.startswith('cancel'):
        assert not (run/'claude-secondary').exists() and not (run/'codex-primary').exists()
        receipt=json.loads((run/'claude-primary/session.cast.result.json').read_text())
        assert receipt['owned_group_gone'] is True and receipt['recorder_exit']==130
        assert not (run/'capture.json').exists()
        continue
    if portal_result=='early':
        failed=json.loads((run/'preview/session.cast.result.json').read_text())
        assert failed['stop_reason']=='CHILD_EXIT' and failed['child_exit']==1
    elif portal_result=='occupied':
        assert not (run/'farmd').exists() and not (run/'portal-capture').exists()
        continue
    else:
        record=json.loads((run/'capture.json').read_text())
        assert record['native_capture_child_started'] is True and record['bullet_live_admission'] is False
        assert record['provider_execution']=='UNVERIFIED' and 'native_provider_spawned' not in record
        assert record['completion']=='UNVERIFIED' and record['account_qualification']=='UNVERIFIED'
        assert record['public_export_review']=='REQUIRED' and len(record['attempts'])==6
    for provider in ['claude','codex','cursor']:
        for attempt,status in [('primary',1),('secondary',0)]:
            dest=run/(provider+'-'+attempt)
            receipt=json.loads((dest/'session.cast.result.json').read_text())
            assert receipt['child_exit']==status and receipt['owned_group_gone'] is True
            assert (dest/'producer.exit').read_text().strip()==str(status)
            assert (dest/'session.cast.raw').stat().st_mode&0o777==0o600
    for service in ['farmd','preview']:
        assert json.loads((run/service/'session.cast.result.json').read_text())['owned_group_gone'] is True
    assert (run/'farmd-data').is_dir() and not (run/'farmd-data/ledger.sqlite').exists()
assert not (hub/'docs/media').exists()
"#;
    let output = Command::new("timeout")
        .args(["--kill-after=1s", "40s", "python3", "-c", source])
        .arg(hub())
        .arg(root.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
