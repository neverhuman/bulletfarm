#!/usr/bin/env bash
# Launch (or stop) one loopback bullet-farmd for the dogfood operator loop and
# exchange its one-time bootstrap token for a session cookie + CSRF token.
#
# This wrapper grants no authority: farmd, the peer registry, and the lease
# transport key live under the operator's own 0700 data dir. Secrets (bootstrap
# token, cookie, CSRF) are written only to the 0600 session file and never
# printed.
set -euo pipefail
umask 077

usage() {
  cat <<'EOF'
usage: serve.sh --data-dir <abs 0700 dir> [--bind 127.0.0.1:7420]
                [--portal-origin <exact origin>] [--farmd <bullet-farmd bin>]
                [--session-file <abs>] [--ready-timeout-s 30]
       serve.sh --stop --data-dir <abs dir>
       serve.sh --help

Start: admit/create <data-dir> (0700, self-owned, no symlink), provision the
lease-transport key once, write the peer registry with a deterministic runner
id, launch bullet-farmd under setsid (logs in <data-dir>/logs/, 0600), wait for
/health, obtain the bootstrap token (--bootstrap-token-file when the binary
lists it, else scraped from stdout), exchange it for cookie+csrf via curl with
an exact Origin header, and write <session-file> (0600 JSON). Prints only
non-secret lines: origin=, farmd=, pid=, data_dir=, session_file=,
bootstrap_path=, log=.

Stop: send SIGTERM to farmd's process group (pid from <data-dir>/farmd.pid),
escalate to SIGKILL after 10s, remove the pid file and the session file.

Defaults: --bind 127.0.0.1:7420; --farmd from $BULLET_FARMD_BIN;
--session-file <data-dir>/session.json. The portal origin defaults to farmd's
own http://<bound-address>; pass --portal-origin only when a separate Portal
dev server will hold the browser session.

Refusals are printed as DOGFOOD_OPS_<CODE>: <value> on stderr with exit 1.
EOF
}

refuse() {
  printf 'DOGFOOD_OPS_%s: %s\n' "$1" "$2" >&2
  exit 1
}

data_dir=""
bind="127.0.0.1:7420"
portal_origin=""
farmd_bin="${BULLET_FARMD_BIN:-}"
session_file=""
ready_timeout_s=30
stop=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --data-dir) data_dir="${2:-}"; shift 2 ;;
    --bind) bind="${2:-}"; shift 2 ;;
    --portal-origin) portal_origin="${2:-}"; shift 2 ;;
    --farmd) farmd_bin="${2:-}"; shift 2 ;;
    --session-file) session_file="${2:-}"; shift 2 ;;
    --ready-timeout-s) ready_timeout_s="${2:-}"; shift 2 ;;
    --stop) stop=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) refuse ARG_UNKNOWN "$1" ;;
  esac
done

[[ -n "$data_dir" ]] || refuse DATA_DIR_REQUIRED "--data-dir"
[[ "$data_dir" == /* ]] || refuse DATA_DIR_NOT_ABSOLUTE "$data_dir"
[[ "$data_dir" != "$HOME" && "$data_dir" != / ]] || refuse DATA_DIR_UNTRUSTED "$data_dir"
[[ -z "$session_file" ]] && session_file="$data_dir/session.json"
[[ "$session_file" == /* ]] || refuse SESSION_FILE_NOT_ABSOLUTE "$session_file"

for tool in curl jq setsid sha256sum stat realpath sed; do
  command -v "$tool" >/dev/null 2>&1 || refuse TOOL_MISSING "$tool"
done

uid="$(id -u)"
gid="$(id -g)"
pid_file="$data_dir/farmd.pid"

# ---------------------------------------------------------------------------
# --stop
# ---------------------------------------------------------------------------
if [[ "$stop" -eq 1 ]]; then
  [[ -f "$pid_file" ]] || refuse NOT_RUNNING "$pid_file absent"
  pid="$(<"$pid_file")"
  [[ "$pid" =~ ^[0-9]+$ ]] || refuse PID_FILE_INVALID "$pid_file"
  if kill -0 "$pid" 2>/dev/null; then
    kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
    stopped=0
    for _ in $(seq 1 200); do
      if ! kill -0 "$pid" 2>/dev/null; then stopped=1; break; fi
      sleep 0.05
    done
    if [[ "$stopped" -eq 0 ]]; then
      kill -KILL -- "-$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
      sleep 0.2
      printf 'stop=killed pid=%s\n' "$pid"
    else
      printf 'stop=terminated pid=%s\n' "$pid"
    fi
  else
    printf 'stop=already-exited pid=%s\n' "$pid"
  fi
  rm -f -- "$pid_file"
  if [[ -f "$session_file" ]]; then
    rm -f -- "$session_file"
    printf 'session_file=removed %s\n' "$session_file"
  fi
  exit 0
fi

# ---------------------------------------------------------------------------
# start: admit inputs
# ---------------------------------------------------------------------------
[[ -n "$farmd_bin" ]] || refuse FARMD_BIN_REQUIRED "--farmd or BULLET_FARMD_BIN"
[[ "$farmd_bin" == /* && -f "$farmd_bin" && -x "$farmd_bin" ]] || refuse FARMD_BIN_INVALID "$farmd_bin"
[[ "$(realpath -e -- "$farmd_bin")" == "$farmd_bin" ]] || refuse FARMD_BIN_NOT_CANONICAL "$farmd_bin"
[[ "$bind" =~ ^127\.0\.0\.1:[0-9]+$ ]] || refuse BIND_NOT_LOOPBACK "$bind"
[[ "$ready_timeout_s" =~ ^[0-9]+$ && "$ready_timeout_s" -ge 1 ]] || refuse READY_TIMEOUT_INVALID "$ready_timeout_s"
if [[ -n "$portal_origin" ]]; then
  [[ "$portal_origin" =~ ^http://(127\.0\.0\.1|localhost):[0-9]+$ ]] || refuse PORTAL_ORIGIN_NOT_LOOPBACK "$portal_origin"
fi

# The kernel refuses proof roots with world-writable ancestors (e.g. /tmp), so
# admit the ancestry here rather than discovering it from a farmd refusal.
ancestor="$(dirname -- "$data_dir")"
while [[ "$ancestor" != / ]]; do
  [[ -e "$ancestor" ]] || refuse DATA_DIR_ANCESTOR_MISSING "$ancestor"
  [[ ! -L "$ancestor" ]] || refuse DATA_DIR_ANCESTOR_SYMLINK "$ancestor"
  perm="$(stat -Lc '%a' -- "$ancestor")"
  [[ $((8#$perm & 8#002)) -eq 0 ]] || refuse DATA_DIR_ANCESTOR_WORLD_WRITABLE "$ancestor ($perm)"
  ancestor="$(dirname -- "$ancestor")"
done

if [[ -e "$data_dir" || -L "$data_dir" ]]; then
  [[ ! -L "$data_dir" ]] || refuse DATA_DIR_SYMLINK "$data_dir"
  [[ "$(stat -Lc '%u:%a:%F' -- "$data_dir")" == "$uid:700:directory" ]] \
    || refuse DATA_DIR_UNTRUSTED "$(stat -Lc '%u:%a:%F' -- "$data_dir") (want $uid:700:directory)"
else
  mkdir -m 0700 -- "$data_dir"
fi
[[ "$(realpath -e -- "$data_dir")" == "$data_dir" ]] || refuse DATA_DIR_NOT_CANONICAL "$data_dir"

if [[ -f "$pid_file" ]]; then
  old_pid="$(<"$pid_file")"
  if [[ "$old_pid" =~ ^[0-9]+$ ]] && kill -0 "$old_pid" 2>/dev/null; then
    refuse ALREADY_RUNNING "pid $old_pid ($pid_file)"
  fi
  rm -f -- "$pid_file"
fi

for sub in logs custody; do
  if [[ -e "$data_dir/$sub" ]]; then
    [[ ! -L "$data_dir/$sub" && "$(stat -Lc '%u:%a:%F' -- "$data_dir/$sub")" == "$uid:700:directory" ]] \
      || refuse SUBDIR_UNTRUSTED "$data_dir/$sub"
  else
    mkdir -m 0700 -- "$data_dir/$sub"
  fi
done
socket_dir="$data_dir/socket"
if [[ -e "$socket_dir" ]]; then
  [[ ! -L "$socket_dir" && "$(stat -Lc '%u:%a:%F' -- "$socket_dir")" == "$uid:710:directory" ]] \
    || refuse SOCKET_DIR_UNTRUSTED "$socket_dir"
else
  mkdir -m 0710 -- "$socket_dir"
fi
lease_socket="$socket_dir/lease.sock"
[[ ${#lease_socket} -lt 100 ]] || refuse SOCKET_PATH_TOO_LONG "${#lease_socket} bytes"
# Nothing is listening (pid file admitted above), so a leftover socket is stale.
rm -f -- "$lease_socket"

# ---------------------------------------------------------------------------
# lease transport key: create once, then only verify custody
# ---------------------------------------------------------------------------
key="$data_dir/custody/lease-transport.key"
if [[ ! -e "$key" ]]; then
  provision_out="$data_dir/logs/key-provision.stdout"
  provision_err="$data_dir/logs/key-provision.stderr"
  "$farmd_bin" --provision-lease-transport-key "$key" >"$provision_out" 2>"$provision_err" \
    || refuse KEY_PROVISION_FAILED "see $provision_err"
  [[ "$(<"$provision_out")" == "LEASE_TRANSPORT_KEY_PROVISIONED: $key" && ! -s "$provision_err" ]] \
    || refuse KEY_PROVISION_INVALID "$provision_out"
fi
[[ ! -L "$key" && "$(stat -Lc '%u:%a:%s:%F' -- "$key")" == "$uid:600:64:regular file" ]] \
  || refuse KEY_CUSTODY_INVALID "$(stat -Lc '%u:%a:%s:%F' -- "$key")"

# ---------------------------------------------------------------------------
# peer registry: deterministic runner id, epoch 1, service uid = farmd uid
# ---------------------------------------------------------------------------
registry="$data_dir/custody/peer-registry.json"
runner_id="run_$(printf 'bullet-dogfood-ops:%s' "$data_dir" | sha256sum | cut -c1-64)"
runner_epoch=1
if [[ -e "$registry" ]]; then
  [[ ! -L "$registry" && "$(stat -Lc '%u:%a:%F' -- "$registry")" == "$uid:600:regular file" ]] \
    || refuse REGISTRY_CUSTODY_INVALID "$registry"
  jq -e --argjson uid "$uid" --argjson gid "$gid" --arg runner "$runner_id" '
    .farmd_uid == $uid and .socket_gid == $gid and
    (.runners | length == 1) and .runners[0].runner_id == $runner and
    .runners[0].runner_epoch == 1 and .runners[0].service_uid == $uid
  ' "$registry" >/dev/null || refuse REGISTRY_MISMATCH "$registry does not match uid=$uid gid=$gid runner=$runner_id"
else
  jq -cS -n --argjson uid "$uid" --argjson gid "$gid" --arg runner "$runner_id" '
    {farmd_uid:$uid,socket_gid:$gid,runners:[{runner_id:$runner,runner_epoch:1,service_uid:$uid}]}
  ' >"$registry"
  chmod 0600 -- "$registry"
fi

# ---------------------------------------------------------------------------
# bootstrap token path: file-based when the binary supports it, else stdout
# ---------------------------------------------------------------------------
help_text="$("$farmd_bin" --help 2>&1 || true)"
bootstrap_path="stdout"
token_file=""
if grep -q -- '--bootstrap-token-file' <<<"$help_text"; then
  bootstrap_path="token_file"
  token_file="$data_dir/custody/bootstrap.token"
  rm -f -- "$token_file"
  "$farmd_bin" --provision-bootstrap-token "$token_file" \
    >"$data_dir/logs/bootstrap-provision.stdout" 2>"$data_dir/logs/bootstrap-provision.stderr" \
    || refuse BOOTSTRAP_PROVISION_FAILED "see $data_dir/logs/bootstrap-provision.stderr"
  [[ ! -L "$token_file" && "$(stat -Lc '%u:%a:%F' -- "$token_file")" == "$uid:600:regular file" ]] \
    || refuse BOOTSTRAP_TOKEN_CUSTODY_INVALID "$token_file"
fi

# ---------------------------------------------------------------------------
# launch under setsid; the log file is 0600 (umask 077)
# ---------------------------------------------------------------------------
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
log_file="$data_dir/logs/farmd-$(date -u +%Y%m%dT%H%M%SZ).log"
farmd_args=(--data-dir "$data_dir" --bind "$bind"
  --lease-transport-socket "$lease_socket" --lease-peer-registry "$registry"
  --lease-transport-key "$key")
[[ -n "$portal_origin" ]] && farmd_args+=(--portal-origin "$portal_origin")
[[ -n "$token_file" ]] && farmd_args+=(--bootstrap-token-file "$token_file")

setsid "$farmd_bin" "${farmd_args[@]}" >"$log_file" 2>&1 </dev/null &
farmd_pid=$!
printf '%s\n' "$farmd_pid" >"$pid_file"

fail_launch() {
  # Never leak the bootstrap line from the log into the operator's terminal.
  sed -e 's/^Bullet Farm one-time bootstrap: .*/Bullet Farm one-time bootstrap: <redacted>/' \
    -n -e '1,40p' "$log_file" >&2 || true
  kill -TERM -- "-$farmd_pid" 2>/dev/null || true
  rm -f -- "$pid_file"
  [[ -n "$token_file" ]] && rm -f -- "$token_file"
  refuse "$1" "$2"
}

farmd_base=""
ticks=$((ready_timeout_s * 20))
for _ in $(seq 1 "$ticks"); do
  bound="$(sed -n 's/.*bullet-farmd listening on \(127\.0\.0\.1:[0-9][0-9]*\)$/\1/p' "$log_file" | tail -n 1)"
  if [[ "$bound" =~ ^127\.0\.0\.1:[0-9]+$ ]]; then
    farmd_base="http://$bound"
    if curl --fail --silent --max-time 2 "$farmd_base/health" >"$data_dir/logs/health.json" 2>/dev/null; then
      break
    fi
    farmd_base=""
  fi
  kill -0 "$farmd_pid" 2>/dev/null || fail_launch FARMD_EXITED "log=$log_file"
  sleep 0.05
done
[[ -n "$farmd_base" ]] || fail_launch FARMD_NOT_READY "no /health within ${ready_timeout_s}s (log=$log_file)"
[[ -S "$lease_socket" ]] || fail_launch LEASE_SOCKET_MISSING "$lease_socket"
jq -e '.status == "ok"' "$data_dir/logs/health.json" >/dev/null \
  || fail_launch HEALTH_INVALID "$data_dir/logs/health.json"

origin="$farmd_base"
[[ -n "$portal_origin" ]] && origin="$portal_origin"

# ---------------------------------------------------------------------------
# obtain the bootstrap token without echoing it
# ---------------------------------------------------------------------------
bootstrap_token=""
if [[ "$bootstrap_path" == token_file ]]; then
  bootstrap_token="$(<"$token_file")"
  rm -f -- "$token_file"
else
  for _ in $(seq 1 100); do
    bootstrap_token="$(sed -n 's/^Bullet Farm one-time bootstrap: //p' "$log_file" | head -n 1)"
    [[ -n "$bootstrap_token" ]] && break
    sleep 0.05
  done
fi
[[ "$bootstrap_token" =~ ^boot_[0-9a-f]{64}$ ]] || fail_launch BOOTSTRAP_TOKEN_INVALID "path=$bootstrap_path"

# ---------------------------------------------------------------------------
# exchange for cookie + csrf; the token travels via a 0600 file, not argv
# ---------------------------------------------------------------------------
cookie_jar="$data_dir/custody/session.cookies"
exchange_body="$data_dir/custody/bootstrap-exchange.json"
exchange_req="$data_dir/custody/bootstrap-request.json"
rm -f -- "$cookie_jar" "$exchange_body" "$exchange_req"
jq -n --arg token "$bootstrap_token" '{bootstrap_token:$token}' >"$exchange_req"
bootstrap_token=""
http_code="$(curl --silent --show-error -c "$cookie_jar" -o "$exchange_body" -w '%{http_code}' \
  --max-time 10 -H "Origin: $origin" -H 'content-type: application/json' \
  --data "@$exchange_req" "$farmd_base/api/v1/auth/bootstrap" 2>"$data_dir/logs/bootstrap-exchange.stderr" || true)"
rm -f -- "$exchange_req"
[[ "$http_code" == 200 ]] || fail_launch BOOTSTRAP_EXCHANGE_FAILED "HTTP ${http_code:-none} origin=$origin (see $data_dir/logs/bootstrap-exchange.stderr)"
csrf="$(jq -r '.csrf_token // empty' "$exchange_body")"
[[ "$csrf" =~ ^csrf_[0-9a-f]{64}$ ]] || fail_launch CSRF_INVALID "$exchange_body"
cookie_value="$(awk '$6 == "bullet_session" { print $7 }' "$cookie_jar" | tail -n 1)"
[[ -n "$cookie_value" ]] || fail_launch SESSION_COOKIE_MISSING "$cookie_jar"
rm -f -- "$exchange_body"

# ---------------------------------------------------------------------------
# session file (0600) and the non-secret report
# ---------------------------------------------------------------------------
session_tmp="$session_file.tmp.$$"
jq -n --arg origin "$origin" --arg farmd "$farmd_base" --arg bind "$bind" \
  --arg cookie "bullet_session=$cookie_value" --arg csrf "$csrf" \
  --argjson farmd_pid "$farmd_pid" --arg lease_socket "$lease_socket" \
  --arg registry "$registry" --arg key "$key" --arg started_at "$started_at" \
  --arg data_dir "$data_dir" --arg runner_id "$runner_id" --argjson runner_epoch "$runner_epoch" \
  --argjson farmd_uid "$uid" --argjson socket_gid "$gid" --arg bootstrap_path "$bootstrap_path" \
  --arg log "$log_file" --arg farmd_bin "$farmd_bin" '
  {schema_version:"bullet.dogfood-ops-session.v1",
   origin:$origin, farmd:$farmd, bind:$bind, cookie:$cookie, csrf:$csrf,
   farmd_pid:$farmd_pid, farmd_bin:$farmd_bin, lease_socket:$lease_socket,
   registry:$registry, key:$key, started_at:$started_at, data_dir:$data_dir,
   runner_id:$runner_id, runner_epoch:$runner_epoch, farmd_uid:$farmd_uid,
   socket_gid:$socket_gid, bootstrap_path:$bootstrap_path, log:$log}
' >"$session_tmp"
chmod 0600 -- "$session_tmp"
mv -f -- "$session_tmp" "$session_file"
cookie_value=""
csrf=""

printf 'origin=%s\n' "$origin"
printf 'farmd=%s\n' "$farmd_base"
printf 'pid=%s\n' "$farmd_pid"
printf 'data_dir=%s\n' "$data_dir"
printf 'session_file=%s\n' "$session_file"
printf 'bootstrap_path=%s\n' "$bootstrap_path"
printf 'log=%s\n' "$log_file"
