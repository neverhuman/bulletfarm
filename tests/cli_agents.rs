use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn bf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bf"))
}

fn write_proc(proc: &Path, pid: i64, ppid: i64, exe: &str, cmdline: &str, comm: &str, cwd: &str) {
    let dir = proc.join(pid.to_string());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("status"),
        format!("Name:\t{comm}\nState:\tS (sleeping)\nPPid:\t{ppid}\nVmRSS:\t  4096 kB\n"),
    )
    .unwrap();
    fs::write(dir.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
    fs::write(dir.join("comm"), comm).unwrap();
    let _ = fs::remove_file(dir.join("exe"));
    symlink(exe, dir.join("exe")).unwrap();
    fs::create_dir_all(cwd).unwrap();
    let _ = fs::remove_file(dir.join("cwd"));
    symlink(cwd, dir.join("cwd")).unwrap();
}

#[test]
fn lists_live_sessions_and_drops_dead_registry_rows() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let proc = root.path().join("proc");
    let work = root.path().join("work");
    fs::create_dir_all(home.join(".claude/sessions")).unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::create_dir_all(home.join(".grok")).unwrap();

    write_proc(
        &proc,
        100,
        1,
        "/home/u/.local/bin/claude",
        "claude --resume abc",
        "claude",
        work.join("claude").to_str().unwrap(),
    );
    write_proc(
        &proc,
        200,
        1,
        "/usr/bin/codex",
        "codex --yolo resume codex-1",
        "codex",
        work.join("codex").to_str().unwrap(),
    );
    write_proc(
        &proc,
        300,
        1,
        "/usr/bin/node",
        "/home/u/.local/bin/agent --use-system-ca /home/u/.local/share/cursor-agent/versions/2026.09.10/index.js --resume=sess-cursor",
        "MainThread",
        work.join("cursor").to_str().unwrap(),
    );
    write_proc(
        &proc,
        400,
        1,
        "/home/u/.grok/downloads/grok-linux-x86_64",
        "grok",
        "agent",
        work.join("grok").to_str().unwrap(),
    );
    // Mention of cursor-agent in a shell argv must not become a Cursor row.
    write_proc(
        &proc,
        500,
        1,
        "/bin/bash",
        "bash -c pgrep -f cursor-agent/versions",
        "bash",
        work.join("bash").to_str().unwrap(),
    );

    fs::write(
        home.join(".claude/sessions/100.json"),
        r#"{"pid":100,"sessionId":"live-claude","cwd":"/work/claude","name":"fix tests","status":"busy","startedAt":1700000000000,"updatedAt":1700000001000}"#,
    )
    .unwrap();
    fs::write(
        home.join(".claude/sessions/999.json"),
        r#"{"pid":999,"sessionId":"dead-claude","cwd":"/gone","name":"stale","status":"idle"}"#,
    )
    .unwrap();
    fs::write(
        home.join(".codex/session_index.jsonl"),
        "{\"id\":\"codex-1\",\"thread_name\":\"Implement prune\",\"updated_at\":\"2026-09-18T00:00:00Z\"}\n",
    )
    .unwrap();
    fs::write(
        home.join(".grok/active_sessions.json"),
        r#"[{"session_id":"grok-1","pid":400,"cwd":"/work/grok","opened_at":"2026-09-18T00:00:00Z"}]"#,
    )
    .unwrap();

    let out = bf()
        .env("BF_HOME", &home)
        .env("BF_PROC", &proc)
        .args([
            "--data-dir",
            root.path().join("data").to_str().unwrap(),
            "agents",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let table = String::from_utf8_lossy(&out.stdout);
    assert!(table.contains("claude"), "{table}");
    assert!(table.contains("codex"), "{table}");
    assert!(table.contains("cursor"), "{table}");
    assert!(table.contains("grok"), "{table}");
    assert!(table.contains("fix tests"), "{table}");
    assert!(table.contains("Implement prune"), "{table}");
    assert!(!table.contains("dead-claude"), "{table}");
    assert!(!table.contains(" 500 "), "{table}");
}

#[test]
fn codex_helpers_are_not_sessions() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let proc = root.path().join("proc");
    let work = root.path().join("work");
    fs::create_dir_all(home.join(".codex")).unwrap();
    let vendor = "/home/u/.npm-global/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin";
    write_proc(
        &proc,
        10,
        1,
        "/home/u/.nvm/versions/node/v26.1.0/bin/node",
        "node /home/u/.npm-global/bin/codex --yolo resume sess-1",
        "node-MainThread",
        work.join("n").to_str().unwrap(),
    );
    write_proc(
        &proc,
        11,
        10,
        &format!("{vendor}/codex"),
        "codex --yolo resume sess-1",
        "codex",
        work.join("c").to_str().unwrap(),
    );
    write_proc(
        &proc,
        12,
        11,
        &format!("{vendor}/codex-code-mode-host"),
        "codex-code-mode-host",
        "codex-code-mode",
        work.join("h").to_str().unwrap(),
    );
    write_proc(
        &proc,
        13,
        11,
        &format!("{vendor}/../codex-path/rg"),
        "rg --json pattern",
        "rg",
        work.join("r").to_str().unwrap(),
    );
    fs::write(
        home.join(".codex/session_index.jsonl"),
        "{\"id\":\"sess-1\",\"thread_name\":\"One session\",\"updated_at\":\"2026-09-18T00:00:00Z\"}\n",
    )
    .unwrap();
    let env = bf::agents::Env { home, proc };
    let agents = bf::agents::discover(&env).unwrap();
    let codex: Vec<_> = agents.iter().filter(|a| a.provider == "codex").collect();
    assert_eq!(
        codex.len(),
        1,
        "wrapper + CLI + code-mode + rg must be one row, got {agents:?}"
    );
    assert_eq!(codex[0].pid, 11);
    assert_eq!(codex[0].session_id.as_deref(), Some("sess-1"));
    assert_eq!(codex[0].title.as_deref(), Some("One session"));
}

#[test]
fn two_hundred_pids_under_500ms() {
    let root = tempfile::tempdir().unwrap();
    let proc = root.path().join("proc");
    let cwd = root.path().join("w");
    fs::create_dir_all(&cwd).unwrap();
    for pid in 1..=200 {
        write_proc(
            &proc,
            pid,
            1,
            "/usr/bin/codex",
            "codex --yolo",
            "codex",
            cwd.to_str().unwrap(),
        );
    }
    let start = Instant::now();
    let env = bf::agents::Env {
        home: root.path().join("empty-home"),
        proc,
    };
    let agents = bf::agents::discover(&env).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(agents.len(), 200);
    assert!(
        elapsed.as_millis() < 500,
        "discover 200 pids took {}ms",
        elapsed.as_millis()
    );
}
