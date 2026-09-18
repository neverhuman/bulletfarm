#!/usr/bin/env bash
# Native harness/Portal observations only. Raw runs remain private for review.
set -euo pipefail
umask 077

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
FAMILY="$(cd "$HUB/.." && pwd -P)"
PORTAL="$FAMILY/bullet-portal"
FARMD_BIN="${BULLET_FARMD_BIN:-$FAMILY/bullet-kernel/target/release/bullet-farmd}"
DATA_ROOT="${BULLET_DEMO_GIF_CACHE:-$HOME/.cache/bullet-demo-gif}"
PTY_RECORD="$HUB/scripts/lib/demo-gif-pty-record.py"
PORTAL_TOUR="$HUB/scripts/lib/demo-gif-portal-tour.mjs"
PROMPT='Explain Bullet Farm in six short lines: name the four member repos (bullet-farm, bullet-kernel, bullet-git, bullet-portal), the fenced Attempt, exact Candidate identity, independent Evidence, and why UNKNOWN is a first-class outcome. Do not use tools. Do not modify files.'

for tool in claude codex cursor-agent curl jq node npm python3; do
  command -v "$tool" >/dev/null || { echo "demo-gif-record: missing $tool" >&2; exit 1; }
done
[[ -x "$FARMD_BIN" && -f "$PORTAL/package.json" && -d "$PORTAL/node_modules/playwright" && -d "$PORTAL/dist" ]] || {
  echo 'demo-gif-record: farmd, Portal dist and Playwright are required' >&2
  exit 1
}

# Refuse symlinked/shared storage. Each run gets a fresh DB; old runs are never deleted.
RUN_ROOT="$(python3 - "$DATA_ROOT" "$FAMILY" <<'PY'
import os, stat, sys, tempfile
from pathlib import Path
p, family = map(Path, sys.argv[1:])
if not p.is_absolute() or p.resolve() != p or p.is_relative_to(family):
    raise SystemExit('demo-gif-record: private external canonical storage required')
p.mkdir(mode=0o700, parents=True, exist_ok=True)
s = p.stat()
if not stat.S_ISDIR(s.st_mode) or s.st_uid != os.getuid() or s.st_mode & 0o077:
    raise SystemExit('demo-gif-record: storage owner/mode refused')
print(tempfile.mkdtemp(prefix='run-', dir=p))
PY
)"
DATA_DIR="$RUN_ROOT/farmd-data"
mkdir "$DATA_DIR"
services=()
active_recorder=""
campaign_status=0
running_job() {
  local owned
  for owned in $(jobs -pr); do [[ "$owned" == "$1" ]] && return 0; done
  return 1
}
cleanup() {
  local original="$1" pid alive=0
  trap - EXIT INT TERM HUP
  for pid in "${services[@]}" "$active_recorder"; do
    if [[ -n "$pid" ]] && running_job "$pid"; then kill -TERM "$pid" 2>/dev/null || true; fi
  done
  # PTY owners have a bounded group teardown. Never wait indefinitely for one.
  for _ in {1..80}; do
    alive=0
    for pid in "${services[@]}" "$active_recorder"; do
      if [[ -n "$pid" ]] && running_job "$pid"; then alive=1; fi
    done
    [[ "$alive" == 0 ]] && break
    sleep 0.05
  done
  if [[ "$alive" != 0 ]]; then
    echo 'demo-gif-record: recorder teardown unverified; retain owned run for reconciliation' >&2
    original=1
  fi
  for pid in farmd preview; do
    if [[ -d "$RUN_ROOT/$pid" ]] && ! jq -e '.child_started == true and .owned_group_gone == true
        and .stop_reason == "INTERRUPTED" and .recorder_exit == 143' \
      "$RUN_ROOT/$pid/session.cast.result.json" >/dev/null 2>&1; then
      original=1
      echo "demo-gif-record: $pid teardown evidence unavailable" >&2
    fi
  done
  printf '%s\n' "$original" >"$RUN_ROOT/campaign.exit"
  echo "demo-gif-record: private raw attempt evidence retained at $RUN_ROOT"
  exit "$original"
}
trap 'cleanup "$?"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

record_cli() {
  local provider="$1" attempt="$2" status=0
  shift 2
  local dest="$RUN_ROOT/$provider-$attempt"
  mkdir "$dest"
  python3 "$PTY_RECORD" --cast "$dest/session.cast" --transcript "$dest/transcript.txt" \
    --cols 140 --rows 38 --title "$provider/$attempt" "$@" &
  active_recorder=$!
  wait "$active_recorder" || status=$?
  active_recorder=""
  printf '%s\n' "$status" >"$dest/producer.exit"
  if ! jq -e '.child_started == false or .owned_group_gone == true' \
    "$dest/session.cast.result.json" >/dev/null 2>&1; then
    echo 'demo-gif-record: outcome/teardown unverified; no secondary invocation' >&2
    exit 1
  fi
  if jq -e '.stop_reason == "INTERRUPTED" or (.stop_reason == "CHILD_EXIT" and
      (.child_signal == 1 or .child_signal == 2 or .child_signal == 15 or
       .child_exit == 129 or .child_exit == 130 or .child_exit == 143))' \
      "$dest/session.cast.result.json" >/dev/null; then
    echo 'demo-gif-record: capture cancelled; no secondary invocation' >&2
    exit "$status"
  fi
  return "$status"
}

owned_services_running() {
  local pid
  for pid in "${services[@]}"; do
    if ! running_job "$pid"; then
      echo 'demo-gif-record: owned service exited; listener source unverified' >&2
      return 1
    fi
  done
}

# No status/auth probe is account qualification. Echoed prompts prove nothing.
cd "$FAMILY"
if ! record_cli claude primary --idle-quit 14 --min-runtime 55 --max-seconds 180 -- \
  claude --model sonnet --permission-mode dontAsk "$PROMPT"; then
  record_cli claude secondary --max-seconds 120 -- \
    claude -p "$PROMPT" --output-format stream-json --verbose --permission-mode dontAsk --model sonnet \
    || campaign_status=1
fi
if ! record_cli codex primary --inject-delay 2.6 --inject '1\r' \
  --idle-quit 14 --min-runtime 55 --max-seconds 180 -- codex --sandbox read-only "$PROMPT"; then
  record_cli codex secondary --max-seconds 120 -- \
    codex exec --skip-git-repo-check --sandbox read-only "$PROMPT" || campaign_status=1
fi
if ! record_cli cursor primary --idle-quit 14 --min-runtime 55 --max-seconds 180 -- \
  cursor-agent --mode plan --trust "$PROMPT"; then
  record_cli cursor secondary --max-seconds 120 -- \
    cursor-agent -p --output-format stream-json --mode plan --trust "$PROMPT" || campaign_status=1
fi

FARMD_BIND="${BULLET_DEMO_FARMD_BIND:-127.0.0.1:7421}"
PORTAL_BIND="${BULLET_DEMO_PORTAL_BIND:-127.0.0.1:5174}"
for bind in "$FARMD_BIND" "$PORTAL_BIND"; do
  [[ "$bind" =~ ^127\.0\.0\.1:[0-9]{1,5}$ ]] || { echo 'loopback binding required' >&2; exit 1; }
done
PORTAL_ORIGIN="http://$PORTAL_BIND"
# Refuse existing listeners before sending a bootstrap token. This check does not
# authenticate a listener that races a later bind; lifetime checks remain required.
python3 - "$FARMD_BIND" "$PORTAL_BIND" <<'PY'
import socket, sys
sockets = []
try:
    for address in sys.argv[1:]:
        host, port = address.rsplit(':', 1)
        if not 1 <= int(port) <= 65535:
            raise ValueError('port outside range')
        sock = socket.socket()
        sockets.append(sock)
        sock.bind((host, int(port)))
        sock.listen(1)
except (OSError, ValueError):
    raise SystemExit('demo-gif-record: loopback listen address unavailable')
finally:
    for sock in sockets:
        sock.close()
PY
mkdir "$RUN_ROOT/farmd" "$RUN_ROOT/preview"
python3 "$PTY_RECORD" --cast "$RUN_ROOT/farmd/session.cast" \
  --transcript "$RUN_ROOT/farmd/transcript.txt" --max-seconds 180 -- \
  "$FARMD_BIN" --data-dir "$DATA_DIR" --bind "$FARMD_BIND" --portal-origin "$PORTAL_ORIGIN" &
services+=("$!")
(
  cd "$PORTAL"
  unset VITE_BULLET_API
  export BULLET_FARMD_TEST_PROXY="http://$FARMD_BIND"
  exec python3 "$PTY_RECORD" --cast "$RUN_ROOT/preview/session.cast" \
    --transcript "$RUN_ROOT/preview/transcript.txt" --max-seconds 180 -- \
    npm run preview -- --host 127.0.0.1 --port "${PORTAL_BIND##*:}" --strictPort
) &
services+=("$!")
for _ in {1..80}; do
  owned_services_running
  if curl -fsS --max-time 1 "http://$FARMD_BIND/health" >/dev/null \
    && curl -fsS --max-time 1 "$PORTAL_ORIGIN/" >/dev/null; then break; fi
  sleep 0.25
done
owned_services_running
curl -fsS --max-time 1 "http://$FARMD_BIND/health" >/dev/null
curl -fsS --max-time 1 "$PORTAL_ORIGIN/" >/dev/null

# Raw log is private and streamed before service exit. Never export its token.
bootstrap="$(python3 - "$RUN_ROOT/farmd/session.cast.raw" <<'PY'
import re, sys
from pathlib import Path
match = re.search(rb'Bullet Farm one-time bootstrap: (boot_[0-9a-f]+)', Path(sys.argv[1]).read_bytes())
if not match:
    raise SystemExit('demo-gif-record: bootstrap unavailable')
print(match[1].decode())
PY
)"
owned_services_running
PORTAL_PACKAGE_JSON="$PORTAL/package.json" PORTAL_ORIGIN="$PORTAL_ORIGIN" \
  PORTAL_CAPTURE_DIR="$RUN_ROOT/portal" BULLET_BOOTSTRAP_TOKEN="$bootstrap" \
  record_cli portal capture --max-seconds 120 -- node "$PORTAL_TOUR" || campaign_status=1
unset bootstrap
owned_services_running
if ! jq -e '.capture_status == "CAPTURED" and .product_completion == "UNVERIFIED" and .bullet_live_admission == false' \
  "$RUN_ROOT/portal/observation.json" >/dev/null 2>&1; then campaign_status=1; fi

python3 - "$RUN_ROOT" "$PTY_RECORD" "$PORTAL_TOUR" <<'PY'
import hashlib, json, sys, time
from pathlib import Path
root = Path(sys.argv[1])
attempts = []
for result in sorted(root.glob('*-*/session.cast.result.json')):
    if result.parent.name.split('-')[0] not in ('claude', 'codex', 'cursor'):
        continue
    data = result.read_bytes()
    row = json.loads(data)
    attempts.append({'path': str(result.relative_to(root)), 'sha256': hashlib.sha256(data).hexdigest(),
                     'child_started': row['child_started'], 'recorder_exit': row['recorder_exit'],
                     'completion': row['completion']})
manifest = {'schema_version': 'bullet.native-recording.v1', 'observed_at_unix': time.time(),
            'classification': 'PRIVATE_NATIVE_CAPTURE', 'release_authority': False,
            'native_capture_child_started': any(a['child_started'] for a in attempts),
            'provider_execution': 'UNVERIFIED',
            'bullet_live_admission': False, 'account_qualification': 'UNVERIFIED',
            'listener_identity': 'UNVERIFIED',
            'completion': 'UNVERIFIED', 'public_export_review': 'REQUIRED', 'attempts': attempts,
            'sources': {str(Path(p).name): hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sys.argv[2:]}}
with (root / 'capture.json').open('x') as out:
    json.dump(manifest, out, indent=2)
    out.write('\n')
PY
cleanup "$campaign_status"
