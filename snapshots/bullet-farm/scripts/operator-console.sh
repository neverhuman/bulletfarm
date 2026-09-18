#!/usr/bin/env bash
# Unsigned loopback operator console: farm init + farmd + Vite Portal.
# Reuses scripts/dogfood/serve.sh --leave-bootstrap so the one-time token
# stays on disk for `bullet auth login`. Never prints the token, cookie, or CSRF.
set -euo pipefail
umask 077

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
KERNEL="$FAMILY/bullet-kernel"
PORTAL="$FAMILY/bullet-portal"
# shellcheck source=ops/ci/toolchain-pins.sh
source "$HUB/ops/ci/toolchain-pins.sh"

usage() {
  cat <<'EOF'
usage: operator-console.sh --data-dir <abs dir under $HOME>
                           [--bind 127.0.0.1:7420]
                           [--portal-origin http://127.0.0.1:5173]
                           [--bullet <abs>] [--farmd <abs>] [--ready-timeout-s 30]
       operator-console.sh --stop --data-dir <abs dir>
       operator-console.sh --help

Unsigned local console only. Not a trusted installer. HOLD remains.

--data-dir must be an absolute path under $HOME, outside this clone, and
not under /tmp. There is no implicit default.

Next commands (token is never printed):
  bullet auth login --farmd <farmd> --origin <portal-origin> --stdin < bootstrap_file
  bullet tui
  open <portal-origin>  (paste the token only if login has not consumed it)
EOF
}

refuse() {
  printf 'OPERATOR_CONSOLE_%s: %s\n' "$1" "$2" >&2
  exit 1
}

data_dir=""
bind="127.0.0.1:7420"
portal_origin="http://127.0.0.1:5173"
bullet_bin="${BULLET_BIN:-}"
farmd_bin="${BULLET_FARMD_BIN:-}"
stop=0
ready_timeout_s=30

while [[ $# -gt 0 ]]; do
  case "$1" in
    --data-dir|--bind|--portal-origin|--bullet|--farmd|--ready-timeout-s)
      [[ $# -ge 2 && -n "$2" ]] || refuse ARG_VALUE_MISSING "$1" ;;
  esac
  case "$1" in
    --data-dir) data_dir="${2:-}"; shift 2 ;;
    --bind) bind="${2:-}"; shift 2 ;;
    --portal-origin) portal_origin="${2:-}"; shift 2 ;;
    --bullet) bullet_bin="${2:-}"; shift 2 ;;
    --farmd) farmd_bin="${2:-}"; shift 2 ;;
    --ready-timeout-s) ready_timeout_s="$2"; shift 2 ;;
    --stop) stop=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) refuse ARG_UNKNOWN "$1" ;;
  esac
done

[[ -n "$data_dir" ]] || refuse DATA_DIR_REQUIRED "--data-dir is required; there is no default"
[[ "$data_dir" == /* ]] || refuse DATA_DIR_NOT_ABSOLUTE "$data_dir"
case "$data_dir" in
  /tmp|/tmp/*) refuse DATA_DIR_TMP "$data_dir" ;;
esac
[[ "$data_dir" == "$HOME"/* ]] || refuse DATA_DIR_NOT_UNDER_HOME "$data_dir"
[[ "$data_dir" != "$HOME" ]] || refuse DATA_DIR_UNTRUSTED "$data_dir"
case "$data_dir" in
  "$FAMILY"|"$FAMILY"/*|"$HUB"|"$HUB"/*) refuse DATA_DIR_INSIDE_CLONE "$data_dir" ;;
esac

# A persisted PID is an observation, not permission to signal a process. Safe
# cross-invocation stop requires the installed Rust supervisor's process custody.
if [[ "$stop" -eq 1 ]]; then
  refuse STOP_CUSTODY_UNAVAILABLE "persisted PIDs cannot authorize signals; state retained for supervised recovery"
fi

[[ "$bind" =~ ^127\.0\.0\.1:[0-9]+$ ]] || refuse BIND_NOT_LOOPBACK "$bind"
[[ "$portal_origin" =~ ^http://(127\.0\.0\.1|localhost):[0-9]+$ ]] \
  || refuse PORTAL_ORIGIN_NOT_LOOPBACK "$portal_origin"
bind_port="${bind##*:}"
portal_port="${portal_origin##*:}"
[[ ${#bind_port} -le 5 && $((10#$bind_port)) -le 65535 ]] || refuse BIND_PORT_INVALID "$bind"
[[ ${#portal_port} -le 5 && $((10#$portal_port)) -ge 1 && $((10#$portal_port)) -le 65535 ]] \
  || refuse PORTAL_PORT_INVALID "$portal_origin"
[[ "$ready_timeout_s" =~ ^[1-9][0-9]?$ ]] || refuse READY_TIMEOUT_INVALID "$ready_timeout_s (want 1..99)"
[[ -d "$KERNEL" && -d "$PORTAL" ]] || refuse FAMILY_LAYOUT "need bullet-kernel and bullet-portal siblings"

for tool in cargo curl node npm setsid realpath stat flock jq; do
  command -v "$tool" >/dev/null 2>&1 || refuse TOOL_MISSING "$tool"
done
[[ "$(node --version)" == "v$PINNED_NODE_VERSION" ]] \
  || refuse NODE_PIN "expected Node v$PINNED_NODE_VERSION, found $(node --version)"
[[ "$(npm --version)" == "$PINNED_NPM_VERSION" ]] \
  || refuse NPM_PIN "expected npm $PINNED_NPM_VERSION, found $(npm --version)"

if [[ -z "$farmd_bin" || -z "$bullet_bin" ]]; then
  ( cd "$KERNEL" && cargo build --locked -p bullet --bin bullet -p bullet-farmd --bin bullet-farmd ) \
    || refuse BUILD_FAILED "cargo build --locked -p bullet --bin bullet -p bullet-farmd"
  [[ -n "$bullet_bin" ]] || bullet_bin="$KERNEL/target/debug/bullet"
  [[ -n "$farmd_bin" ]] || farmd_bin="$KERNEL/target/debug/bullet-farmd"
fi
[[ "$bullet_bin" == /* && -x "$bullet_bin" ]] || refuse BULLET_BIN_INVALID "$bullet_bin"
[[ "$farmd_bin" == /* && -x "$farmd_bin" ]] || refuse FARMD_BIN_INVALID "$farmd_bin"
bullet_bin="$(realpath -e -- "$bullet_bin")"
farmd_bin="$(realpath -e -- "$farmd_bin")"

# Validate existing ancestry before mkdir or farm init can follow a symlink.
[[ "$(realpath -m -- "$data_dir")" == "$data_dir" ]] || refuse DATA_DIR_NOT_CANONICAL "$data_dir"
ancestor="$data_dir"
while [[ "$ancestor" != / ]]; do
  if [[ -e "$ancestor" || -L "$ancestor" ]]; then
    [[ -d "$ancestor" && ! -L "$ancestor" ]] || refuse DATA_DIR_UNTRUSTED "$ancestor"
    perm="$(stat -Lc '%a' -- "$ancestor")"
    [[ $((8#$perm & 8#002)) -eq 0 ]] || refuse DATA_DIR_UNTRUSTED "$ancestor (world writable)"
  fi
  ancestor="$(dirname -- "$ancestor")"
done
mkdir -p -- "$data_dir"
[[ "$(stat -Lc '%u:%a:%F' -- "$data_dir")" == "$(id -u):700:directory" ]] \
  || refuse DATA_DIR_UNTRUSTED "$data_dir (want self-owned 0700)"
path="$data_dir/logs"
[[ ! -L "$path" ]] || refuse DATA_DIR_UNTRUSTED "$path"
[[ -e "$path" ]] || mkdir -m 0700 -- "$path"
[[ "$(stat -Lc '%u:%a:%F' -- "$path")" == "$(id -u):700:directory" ]] \
  || refuse DATA_DIR_UNTRUSTED "$path"
[[ ! -L "$data_dir/operator-console.lock" ]] || refuse DATA_DIR_UNTRUSTED "console lock symlink"
exec {console_lock}>"$data_dir/operator-console.lock"
flock -n "$console_lock" || refuse ALREADY_STARTING "$data_dir"
for path in "$data_dir/portal.pid" "$data_dir/farmd.pid"; do
  [[ ! -e "$path" && ! -L "$path" ]] || refuse PID_STATE_UNRECONCILED "$path; preserved without signaling"
done
export BULLET_DATA_DIR="$data_dir"
"$bullet_bin" farm init || refuse FARM_INIT_FAILED "$data_dir"

(
  cd "$PORTAL"
  node ops/ci/preinstall-scan.mjs
  npm ci --ignore-scripts --no-audit --no-fund
)

# Keep serve.sh alive as farmd's owner until both interfaces are ready. On
# failure or interruption only this invocation's children receive signals.
startup_dir="$(mktemp -d "$data_dir/logs/console-startup.XXXXXXXX")"
serve_out="$startup_dir/serve.stdout"
: >"$serve_out"
serve_pid=""
portal_pid=""
startup_complete=0
cleanup() {
  local status=$?
  trap - EXIT INT TERM
  if [[ "$startup_complete" -eq 0 ]]; then
    if [[ -n "$portal_pid" ]]; then
      kill -TERM -- "-$portal_pid" 2>/dev/null || true
      for ((shutdown_tick=0; shutdown_tick<40; shutdown_tick++)); do
        kill -0 -- "-$portal_pid" 2>/dev/null || break
        sleep 0.05
      done
      if [[ "$shutdown_tick" -eq 40 ]]; then kill -KILL -- "-$portal_pid" 2>/dev/null || true; fi
      wait "$portal_pid" 2>/dev/null || true
      if kill -0 -- "-$portal_pid" 2>/dev/null; then
        printf 'OPERATOR_CONSOLE_CLEANUP_UNRESOLVED: portal group %s; retained %s\n' "$portal_pid" "$data_dir/portal.pid" >&2
      else
        rm -f -- "$data_dir/portal.pid"
      fi
    fi
    if [[ -n "$serve_pid" ]]; then
      kill -TERM -- "-$serve_pid" 2>/dev/null || true
      for ((shutdown_tick=0; shutdown_tick<100; shutdown_tick++)); do
        kill -0 "$serve_pid" 2>/dev/null || break
        sleep 0.05
      done
      if kill -0 "$serve_pid" 2>/dev/null; then
        printf 'OPERATOR_CONSOLE_CLEANUP_UNRESOLVED: launcher %s; retained %s\n' "$serve_pid" "$startup_dir/serve.pid" >&2
      else
        wait "$serve_pid" 2>/dev/null || true
      fi
    fi
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
setsid bash "$HUB/scripts/dogfood/serve.sh" \
  --data-dir "$data_dir" --bind "$bind" --portal-origin "$portal_origin" \
  --farmd "$farmd_bin" --leave-bootstrap --ready-timeout-s "$ready_timeout_s" \
  --startup-ack-file "$startup_dir/ack" >"$serve_out" 2>"$startup_dir/serve.stderr" {console_lock}>&- &
serve_pid=$!
printf '%s\n' "$serve_pid" >"$startup_dir/serve.pid"
for ((tick=0; tick<ready_timeout_s*20; tick++)); do
  if grep -qx 'startup=ready' "$serve_out"; then break; fi
  if ! kill -0 "$serve_pid" 2>/dev/null; then
    wait "$serve_pid" 2>/dev/null || true
    serve_pid=""
    refuse FARMD_START_FAILED "see $startup_dir/serve.stderr"
  fi
  sleep 0.05
done
grep -qx 'startup=ready' "$serve_out" || refuse FARMD_START_FAILED "see $startup_dir/serve.stderr"
farmd_url="$(sed -n 's/^farmd=//p' "$serve_out")"
[[ "$farmd_url" =~ ^http://127\.0\.0\.1:[0-9]+$ ]] || refuse FARMD_REPORT_INVALID "$serve_out"

BULLET_PORTAL_PORT="$portal_port" BULLET_FARMD_TEST_PROXY="$farmd_url" \
  setsid bash "$HUB/scripts/portal.sh" >"$startup_dir/portal.log" 2>&1 </dev/null {console_lock}>&- &
portal_pid=$!
printf '%s\n' "$portal_pid" >"$data_dir/portal.pid"
portal_ready=0
for ((tick=0; tick<ready_timeout_s*10; tick++)); do
  kill -0 "$portal_pid" 2>/dev/null || refuse PORTAL_EXITED "see $startup_dir/portal.log"
  kill -0 "$serve_pid" 2>/dev/null || refuse FARMD_EXITED "see $startup_dir/serve.stderr"
  # Vite prints Local only after its own strict-port bind succeeds. A preexisting
  # listener must not qualify this invocation while npm is still starting.
  if grep -Eq "Local:.*http://127[.]0[.]0[.]1:$portal_port/" "$startup_dir/portal.log" \
    && curl --fail --silent --max-time 1 "$portal_origin/" >"$startup_dir/portal.html" 2>/dev/null \
    && grep -q 'id="root"' "$startup_dir/portal.html" \
    && curl --fail --silent --max-time 1 "$portal_origin/health" >"$startup_dir/proxy-health.json" 2>/dev/null \
    && jq -e '.status == "ok"' "$startup_dir/proxy-health.json" >/dev/null 2>&1; then
    portal_ready=1
    break
  fi
  sleep 0.1
done
[[ "$portal_ready" -eq 1 ]] || refuse PORTAL_NOT_READY "see $startup_dir"
kill -0 "$portal_pid" 2>/dev/null || refuse PORTAL_EXITED "see $startup_dir/portal.log"
printf 'ready\n' >"$startup_dir/ack"
wait "$serve_pid" || refuse FARMD_START_FAILED "see $startup_dir/serve.stderr"
serve_pid=""
startup_complete=1
cat "$serve_out"
printf 'portal=%s\n' "$portal_origin"
printf 'next=bullet auth login --farmd %s --origin %s --stdin < bootstrap_file\n' \
  "$farmd_url" "$portal_origin"
printf 'next=bullet tui\n'
printf 'note=just dev cannot create a session; this wrapper can.\n'
printf 'note=unsigned local console; HOLD remains; not VERIFIED.\n'
printf 'note=cross-invocation stop requires qualified process custody; PID files are diagnostic only.\n'
