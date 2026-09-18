#!/usr/bin/env bash
# Disposable component fixtures exercise the real launchers; no real providers.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 - "$HUB" <<'PY'
import hashlib
import http.client
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time

hub = Path(sys.argv[1])
evidence = Path(tempfile.mkdtemp(prefix="bullet-console-components-"))
# The launcher requires private non-/tmp ancestry. Nested missing directories and
# spaces exercise clean-home creation without changing the operator's HOME.
data_root = Path(tempfile.mkdtemp(prefix=".bullet-console-test-", dir=Path.home()))
fixture = evidence / "family" / "bullet-farm"
portal = fixture.parent / "bullet-portal"
for directory in [fixture / "scripts/dogfood", fixture / "ops/ci", portal / "ops/ci", fixture.parent / "bullet-kernel"]:
    directory.mkdir(parents=True)
inputs = {}
for name in ["scripts/operator-console.sh", "scripts/portal.sh", "scripts/dogfood/serve.sh", "ops/ci/toolchain-pins.sh", ".node-version", ".npm-version"]:
    shutil.copyfile(hub / name, fixture / name)
    inputs[name] = hashlib.sha256((hub / name).read_bytes()).hexdigest()
(evidence / "inputs.json").write_text(json.dumps(inputs, indent=2))
(portal / "ops/ci/preinstall-scan.mjs").write_text("// fixture\n")
bin_dir = evidence / "bin"
bin_dir.mkdir()
# Fake tools implement only their native launcher contracts. The launcher,
# process creation, HTTP requests, Unix socket, and proxy traversal are real.
fixture_tool = r'''#!/usr/bin/env python3
import http.server, json, os, pathlib, signal, socket, subprocess, sys, time, urllib.request
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
scenario = os.environ.get("CONSOLE_TEST_SCENARIO", "success")
marker = pathlib.Path(os.environ["CONSOLE_TEST_MARKER"])
if scenario == "portal-descendant" and name == "npm" and args[:2] == ["run", "dev"] or scenario == "farmd-descendant" and name == "farmd" and args[0] == "--data-dir":
    child = subprocess.Popen([sys.executable, "-c", "import json,os,signal,sys,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); open(sys.argv[1], 'a').write(json.dumps(dict(kind='grandchild',pid=os.getpid(),parent_pid=os.getppid()))+'\\n'); time.sleep(60)", str(marker)])
def record(kind, **values):
    with marker.open("a") as output:
        output.write(json.dumps(dict(kind=kind, pid=os.getpid(), **values)) + "\n")
if name in ("node", "npm") and args == ["--version"]:
    print(os.environ["PINNED_NODE_VERSION"] if name == "node" else os.environ["PINNED_NPM_VERSION"])
    sys.exit(0)
if name == "node" or (name == "npm" and args[0] == "ci"):
    record(name, args=args)
    sys.exit(0)
if name == "cargo":
    raise SystemExit("unexpected cargo invocation")
if name == "bullet":
    directory = pathlib.Path(os.environ["BULLET_DATA_DIR"])
    assert directory.is_dir() and directory.stat().st_mode & 0o777 == 0o700
    record("farm-init", data=str(directory))
    sys.exit(0)
if name == "farmd":
    if args == ["--help"]:
        print("--bootstrap-token-file")
        sys.exit(0)
    if args[0].startswith("--provision-"):
        # Disposable bytes satisfy the fixture file contract, never real keys.
        pathlib.Path(args[1]).write_text("0" * 64)
        if args[0] == "--provision-lease-transport-key":
            print("LEASE_TRANSPORT_KEY_PROVISIONED: " + args[1])
        sys.exit(0)
    record("farmd", args=args)
    if scenario == "farmd-fail":
        print("fixture farmd failed before readiness", file=sys.stderr)
        sys.exit(17)
    unix_socket = socket.socket(socket.AF_UNIX)
    unix_socket.bind(args[args.index("--lease-transport-socket") + 1])
    port = int(args[args.index("--bind") + 1].split(":")[-1])
else:
    record("portal", args=args, proxy=os.environ.get("BULLET_FARMD_TEST_PROXY"), api=os.environ.get("VITE_BULLET_API"))
    if scenario == "portal-exit":
        sys.exit(23)
    port = int(args[args.index("--port") + 1])
    if scenario == "portal-bind-delay":
        time.sleep(0.5)
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if name == "farmd":
            body = b'{"status":"ok"}'
        elif scenario in ("portal-not-ready", "interrupt", "portal-descendant", "farmd-descendant"):
            self.send_error(503)
            return
        elif self.path == "/health" and scenario == "occupied-listener":
            body = b'{"status":"ok"}'
        elif self.path == "/health":
            with urllib.request.urlopen(os.environ["BULLET_FARMD_TEST_PROXY"] + "/health", timeout=1) as response:
                body = response.read()
        else:
            body = b'<html><div id="root"></div></html>'
        self.send_response(200)
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *args):
        pass
server = http.server.HTTPServer(("127.0.0.1", port), Handler)
if name == "farmd":
    print("bullet-farmd listening on 127.0.0.1:" + str(server.server_port), flush=True)
else:
    print("Local: http://127.0.0.1:" + str(server.server_port) + "/", flush=True)
server.serve_forever()
'''
for name in ["node", "npm", "cargo", "bullet", "farmd"]:
    path = bin_dir / name
    path.write_text(fixture_tool)
    path.chmod(0o700)
env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"],
           PINNED_NODE_VERSION="v" + (hub / ".node-version").read_text().strip(),
           PINNED_NPM_VERSION=(hub / ".npm-version").read_text().strip(),
           VITE_BULLET_API="https://ambient.invalid", BULLET_FARMD_TEST_PROXY="http://127.0.0.1:1")
# The pins script exports its own canonical values; the fixture Node prints v.
fixture_tool = fixture_tool.replace('print(os.environ["PINNED_NODE_VERSION"] if name == "node"', 'print("v" + os.environ["PINNED_NODE_VERSION"] if name == "node"')
(bin_dir / "node").write_text(fixture_tool)
owned = []
results = []
def alive(pid):
    try:
        raw = Path(f"/proc/{pid}/stat").read_text().rsplit(") ", 1)[1]
        return raw.split()[0] != "Z"
    except FileNotFoundError:
        return False

def records(marker):
    return [json.loads(line) for line in marker.read_text().splitlines()] if marker.exists() else []

def stop_fixture(pid):
    # Test-owned fixture processes only; no PID read from user state.
    if alive(pid):
        os.kill(pid, signal.SIGTERM)
    for _ in range(100):
        if not alive(pid):
            return
        time.sleep(0.02)
    raise AssertionError(f"fixture survived cleanup: {pid}")

def run_case(label, args, want, scenario="success", interrupt=False, script="operator-console.sh"):
    marker = evidence / (label + ".markers.jsonl")
    stdout = (evidence / (label + ".stdout")).open("w")
    stderr = (evidence / (label + ".stderr")).open("w")
    command = ["bash", str(fixture / "scripts" / script), *args]
    process = subprocess.Popen(command, env=dict(env, CONSOLE_TEST_SCENARIO=scenario, CONSOLE_TEST_MARKER=str(marker)), stdout=stdout, stderr=stderr)
    try:
        if interrupt:
            for _ in range(300):
                observed = records(marker)
                expected_parent = "farmd" if scenario == "farmd-descendant" else "portal"
                parents = {row["pid"] for row in observed if row["kind"] == expected_parent}
                descendant_ready = scenario not in ("portal-descendant", "farmd-descendant") or any(
                    row["kind"] == "grandchild" and row["parent_pid"] in parents for row in observed)
                if any(row["kind"] == "portal" for row in observed) and descendant_ready:
                    break
                assert process.poll() is None, "launcher exited before interruption stage"
                time.sleep(0.02)
            else:
                raise AssertionError("portal child never started")
            process.send_signal(signal.SIGTERM)
        status = process.wait(timeout=20)
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=10)
        stdout.close()
        stderr.close()
        children = [row["pid"] for row in records(marker) if row["kind"] in ("portal", "farmd", "grandchild")]
        owned.extend(children)
    output = (evidence / (label + ".stdout")).read_text() + (evidence / (label + ".stderr")).read_text()
    assert "unbound variable" not in output, output
    if scenario in ("portal-descendant", "farmd-descendant"):
        assert any(row["kind"] == "grandchild" for row in records(marker)), "grandchild never started"
    if interrupt:
        assert status == 143, (label, status, output)
        assert all(not alive(pid) for pid in children), (label, "interruption leaked a child", children)
        assert "next=bullet tui" not in output
    elif want:
        assert status != 0 and want in output, (label, status, output)
        assert "next=bullet tui" not in output
        assert all(not alive(pid) for pid in children), (label, "startup leaked a child", children)
    else:
        assert status == 0 and "next=bullet tui" in output, (label, status, output)
    results.append(dict(case=label, exit=status, outcome="PASS", children=children))
    print("operator-console-test:", label, "PASS", flush=True)
    return output, records(marker)

def options(label):
    with socket.socket() as reserve:
        reserve.bind(("127.0.0.1", 0))
        port = reserve.getsockname()[1]
    return ["--data-dir", str(data_root / label / "nested state"), "--bind", "127.0.0.1:0",
            "--portal-origin", f"http://127.0.0.1:{port}", "--bullet", str(bin_dir / "bullet"),
            "--farmd", str(bin_dir / "farmd"), "--ready-timeout-s", "2"]

passed = False
try:
    for label, args, want in [
        ("missing-data", [], "DATA_DIR_REQUIRED"),
        ("missing-value", ["--data-dir"], "ARG_VALUE_MISSING"),
        ("relative-data", ["--data-dir", "relative/console"], "DATA_DIR_NOT_ABSOLUTE"),
        ("tmp-data", ["--data-dir", "/tmp/bullet-console"], "DATA_DIR_TMP"),
        ("bad-port", options("bad-port") + ["--portal-origin", "http://127.0.0.1:65536"], "PORTAL_PORT_INVALID"),
    ]:
        run_case(label, args, want)
    success_args = options("success")
    output, rows = run_case("alternate-ports-clean-home", success_args, None)
    portal_row = next(row for row in rows if row["kind"] == "portal")
    actual_farmd = next(line.split("=", 1)[1] for line in output.splitlines() if line.startswith("farmd="))
    assert portal_row["proxy"] == actual_farmd and not actual_farmd.endswith(":0")
    assert portal_row["api"] is None
    assert portal_row["args"][-4:] == ["127.0.0.1", "--port", success_args[5].split(":")[-1], "--strictPort"]
    for child in [row["pid"] for row in rows if row["kind"] in ("portal", "farmd", "grandchild")]:
        stop_fixture(child)
    for scenario, want in [("farmd-fail", "FARMD_START_FAILED"), ("portal-exit", "PORTAL_EXITED"), ("portal-not-ready", "PORTAL_NOT_READY")]:
        run_case(scenario, options(scenario), want, scenario)
    run_case("interrupt", options("interrupt") + ["--ready-timeout-s", "10"], "", "interrupt", True)
    for scenario in ["portal-descendant", "farmd-descendant"]:
        run_case(scenario, options(scenario) + ["--ready-timeout-s", "10"], "", scenario, True)
    # An unrelated server responds with the expected page and health while the
    # new npm process delays its bind. Only the launched Vite readiness counts.
    collision_args = options("occupied-port")
    collision_port = collision_args[5].split(":")[-1]
    collision_marker = evidence / "occupied-listener.markers.jsonl"
    listener_log = (evidence / "occupied-listener.log").open("w")
    listener = subprocess.Popen([str(bin_dir / "npm"), "run", "dev", "--", "--port", collision_port],
        env=dict(env, CONSOLE_TEST_MARKER=str(collision_marker), CONSOLE_TEST_SCENARIO="occupied-listener"), stdout=listener_log, stderr=listener_log)
    owned.append(listener.pid)
    for _ in range(100):
        if "Local:" in (evidence / "occupied-listener.log").read_text():
            break
        time.sleep(0.02)
    else:
        raise AssertionError("occupied listener did not bind")
    connection = http.client.HTTPConnection("127.0.0.1", int(collision_port), timeout=2)
    for target in ["/", "/health"]:
        connection.request("GET", target)
        response = connection.getresponse()
        body = response.read()
        assert response.status == 200
        assert b'id="root"' in body if target == "/" else json.loads(body) == {"status": "ok"}
    connection.close()
    original = (fixture / "scripts/operator-console.sh").read_text()
    guard = '  if grep -Eq "Local:.*http://127[.]0[.]0[.]1:$portal_port/" "$startup_dir/portal.log" \\\n    && curl'
    assert original.count(guard) == 1
    mutant = original.replace(guard, '  if curl')
    (fixture / "scripts/operator-console-negative-control.sh").write_text(mutant)
    (evidence / "occupied-port-negative-control.json").write_text(json.dumps({
        "source_sha256": hashlib.sha256(original.encode()).hexdigest(),
        "mutant_sha256": hashlib.sha256(mutant.encode()).hexdigest(),
        "change": "remove only fresh Vite bind-log readiness condition"}, indent=2))
    negative_args = options("occupied-negative-control")
    negative_args[5] = collision_args[5]
    # This explicit negative control must reproduce unsafe success; it is never
    # counted as accepted launcher readiness. Its bytes and outputs are retained.
    _, negative_rows = run_case("occupied-negative-control-unsafe-success", negative_args, None,
        "portal-bind-delay", script="operator-console-negative-control.sh")
    for row in negative_rows:
        if row["kind"] in ("portal", "farmd"):
            stop_fixture(row["pid"])
    run_case("occupied-port", collision_args, "PORTAL_EXITED", "portal-bind-delay")
    assert alive(listener.pid), "collision cleanup signaled the foreign listener"
    stop_fixture(listener.pid)
    listener.wait(timeout=5)
    listener_log.close()
    foreign = subprocess.Popen(["sleep", "60"])
    owned.append(foreign.pid)
    stop_dir = data_root / "persisted-state"
    stop_dir.mkdir(mode=0o700)
    for label, pid in [("foreign", str(foreign.pid)), ("stale", "99999999"), ("invalid", "0")]:
        for kind in ["portal", "farmd"]:
            (stop_dir / (kind + ".pid")).write_text(pid + "\n")
        before = {path.name: path.read_bytes() for path in stop_dir.iterdir()}
        run_case("stop-" + label, ["--stop", "--data-dir", str(stop_dir)], "STOP_CUSTODY_UNAVAILABLE")
        command = ["bash", str(fixture / "scripts/dogfood/serve.sh"), "--stop", "--data-dir", str(stop_dir)]
        result = subprocess.run(command, capture_output=True, env=env, timeout=10)
        (evidence / ("serve-stop-" + label + ".stderr")).write_bytes(result.stderr)
        assert result.returncode != 0 and b"DOGFOOD_OPS_STOP_CUSTODY_UNAVAILABLE" in result.stderr
        assert alive(foreign.pid), "stop signaled an unrelated process"
        assert before == {path.name: path.read_bytes() for path in stop_dir.iterdir()}, "stop destroyed reconciliation state"
    stop_fixture(foreign.pid)
    foreign.wait(timeout=5)
    target = data_root / "symlink-target"
    target.mkdir(mode=0o700)
    link = data_root / "linked"
    link.symlink_to(target, target_is_directory=True)
    run_case("symlink-ancestor", ["--data-dir", str(link / "state"), "--bullet", str(bin_dir / "bullet"), "--farmd", str(bin_dir / "farmd")], "DATA_DIR_NOT_CANONICAL")
    assert not list(target.iterdir())
    passed = True
finally:
    for pid in set(owned):
        stop_fixture(pid)
    (evidence / "results.json").write_text(json.dumps(dict(passed=passed, cases=results), indent=2))
    shutil.copytree(data_root, evidence / "runtime", ignore=shutil.ignore_patterns("*.sock"), dirs_exist_ok=True)
    shutil.rmtree(data_root)
    print("operator-console-test: evidence=" + str(evidence), flush=True)
PY
