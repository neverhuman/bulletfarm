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
                           [--bullet <abs>] [--farmd <abs>]
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --data-dir) data_dir="${2:-}"; shift 2 ;;
    --bind) bind="${2:-}"; shift 2 ;;
    --portal-origin) portal_origin="${2:-}"; shift 2 ;;
    --bullet) bullet_bin="${2:-}"; shift 2 ;;
    --farmd) farmd_bin="${2:-}"; shift 2 ;;
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

if [[ "$stop" -eq 1 ]]; then
  portal_pid_file="$data_dir/portal.pid"
  if [[ -f "$portal_pid_file" ]]; then
    portal_pid="$(<"$portal_pid_file")"
    if [[ "$portal_pid" =~ ^[0-9]+$ ]] && kill -0 "$portal_pid" 2>/dev/null; then
      kill -TERM -- "-$portal_pid" 2>/dev/null || kill -TERM "$portal_pid" 2>/dev/null || true
    fi
    rm -f -- "$portal_pid_file"
  fi
  exec bash "$HUB/scripts/dogfood/serve.sh" --stop --data-dir "$data_dir"
fi

[[ "$bind" =~ ^127\.0\.0\.1:[0-9]+$ ]] || refuse BIND_NOT_LOOPBACK "$bind"
[[ "$portal_origin" =~ ^http://(127\.0\.0\.1|localhost):[0-9]+$ ]] \
  || refuse PORTAL_ORIGIN_NOT_LOOPBACK "$portal_origin"
[[ -d "$KERNEL" && -d "$PORTAL" ]] || refuse FAMILY_LAYOUT "need bullet-kernel and bullet-portal siblings"

for tool in cargo curl node npm setsid; do
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

if [[ ! -e "$data_dir" ]]; then
  mkdir -m 0700 -- "$data_dir"
fi
if [[ ! -e "$data_dir/logs" ]]; then
  mkdir -m 0700 -- "$data_dir/logs"
fi
export BULLET_DATA_DIR="$data_dir"
"$bullet_bin" farm init || refuse FARM_INIT_FAILED "$data_dir"

(
  cd "$PORTAL"
  node ops/ci/preinstall-scan.mjs
  npm ci --ignore-scripts --no-audit --no-fund
)

# serve.sh admits the 0700 data dir and starts farmd with the token left in place.
serve_out="$(mktemp)"
if ! bash "$HUB/scripts/dogfood/serve.sh" \
  --data-dir "$data_dir" \
  --bind "$bind" \
  --portal-origin "$portal_origin" \
  --farmd "$farmd_bin" \
  --leave-bootstrap >"$serve_out"; then
  kill -TERM -- "-$portal_pid" 2>/dev/null || true
  rm -f -- "$data_dir/portal.pid"
  cat "$serve_out" >&2 || true
  rm -f -- "$serve_out"
  refuse FARMD_START_FAILED "see $data_dir/logs"
fi
cat "$serve_out"
rm -f -- "$serve_out"

setsid bash "$HUB/scripts/portal.sh" >"$data_dir/logs/portal.log" 2>&1 &
portal_pid=$!
printf '%s\n' "$portal_pid" >"$data_dir/portal.pid"

farmd_url="http://${bind}"
printf 'portal=%s\n' "$portal_origin"
printf 'next=bullet auth login --farmd %s --origin %s --stdin < bootstrap_file\n' \
  "$farmd_url" "$portal_origin"
printf 'next=bullet tui\n'
printf 'note=just dev cannot create a session; this wrapper can.\n'
printf 'note=unsigned local console; HOLD remains; not VERIFIED.\n'
