#!/usr/bin/env bash
# Run the one-shot bullet-command-worker in a supervised loop against a farmd
# started by serve.sh.
#
# bullet-command-worker claims at most one command and exits, printing
# COMMAND_UNKNOWN (a command was claimed and settled UNKNOWN) or NO_COMMAND
# (nothing to claim). This wrapper only re-invokes it; it grants no authority,
# never kills a child that is still working, and never restarts one mid-flight.
set -euo pipefail
umask 077

usage() {
  cat <<'EOF'
usage: worker-loop.sh --data-dir <abs 0700 dir> --manifest <abs json>
                      [--session-file <abs>] [--interval 2] [--max-idle N]
                      [--worker <bullet-command-worker>] [--deadline-ms 600000]
                      [--env-file <abs prepare-harness env file>]
                      [--transaction-offline <bin>] [--farmd <bin>]
                      [--runner <bin>] [--gitd <bin>] [--verifier <bin>]
       worker-loop.sh --help

Builds <manifest> when it is absent, using the same
`bullet.command-worker-binary-manifest.v1` schema and SHA-256 subjects as
bullet-kernel/ops/ci/proof-public-command-component.sh: canonical sorted JSON
with no terminal newline, mode 0600. Binary paths come from the flags above or
from $BULLET_TRANSACTION_OFFLINE_BIN, $BULLET_FARMD_BIN, $BULLET_RUNNER_BIN,
$BULLET_GITD_BIN and $BULLET_VERIFIER_FIXTURE_BIN; farmd also falls back to the
binary recorded in the session file.

Each iteration logs one line to stdout and to <data-dir>/logs/worker-loop-*.log:
  <utc> iter=<n> exit=<code> result=<token> elapsed_ms=<n>

NO_COMMAND is idle: the sleep doubles from --interval up to 8x. Any other
result resets the backoff. --max-idle N exits 0 after N consecutive idle
iterations (0, the default, loops until signalled). SIGTERM/SIGINT stop the
loop after the running child finishes on its own; the child is started in its
own session so a terminal signal cannot reach it.

--env-file loads the BULLET_HARNESS_* bindings prepare-harness.sh wrote into the
worker's environment; run_coding needs them, run_demo does not.

A nonzero worker exit is a typed refusal, not a transient: the loop prints the
worker's stderr verbatim and stops with exit 1.

Refusals are printed as DOGFOOD_OPS_<CODE>: <value> on stderr with exit 1.
EOF
}

refuse() {
  printf 'DOGFOOD_OPS_%s: %s\n' "$1" "$2" >&2
  exit 1
}

data_dir=""
session_file=""
manifest=""
interval=2
max_idle=0
worker_bin="${BULLET_COMMAND_WORKER_BIN:-}"
deadline_ms=600000
env_file=""
transaction_offline_bin="${BULLET_TRANSACTION_OFFLINE_BIN:-}"
farmd_bin="${BULLET_FARMD_BIN:-}"
runner_bin="${BULLET_RUNNER_BIN:-}"
gitd_bin="${BULLET_GITD_BIN:-}"
verifier_bin="${BULLET_VERIFIER_FIXTURE_BIN:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --data-dir) data_dir="${2:-}"; shift 2 ;;
    --session-file) session_file="${2:-}"; shift 2 ;;
    --manifest) manifest="${2:-}"; shift 2 ;;
    --interval) interval="${2:-}"; shift 2 ;;
    --max-idle) max_idle="${2:-}"; shift 2 ;;
    --worker) worker_bin="${2:-}"; shift 2 ;;
    --deadline-ms) deadline_ms="${2:-}"; shift 2 ;;
    --env-file) env_file="${2:-}"; shift 2 ;;
    --transaction-offline) transaction_offline_bin="${2:-}"; shift 2 ;;
    --farmd) farmd_bin="${2:-}"; shift 2 ;;
    --runner) runner_bin="${2:-}"; shift 2 ;;
    --gitd) gitd_bin="${2:-}"; shift 2 ;;
    --verifier) verifier_bin="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) refuse ARG_UNKNOWN "$1" ;;
  esac
done

for tool in jq sha256sum stat realpath setsid timeout od truncate; do
  command -v "$tool" >/dev/null 2>&1 || refuse TOOL_MISSING "$tool"
done

[[ -n "$data_dir" ]] || refuse DATA_DIR_REQUIRED "--data-dir"
[[ "$data_dir" == /* ]] || refuse DATA_DIR_NOT_ABSOLUTE "$data_dir"
[[ -d "$data_dir" && ! -L "$data_dir" ]] || refuse DATA_DIR_MISSING "$data_dir"
[[ "$(stat -Lc '%u:%a' -- "$data_dir")" == "$(id -u):700" ]] \
  || refuse DATA_DIR_UNTRUSTED "$(stat -Lc '%u:%a' -- "$data_dir")"
[[ -z "$session_file" ]] && session_file="$data_dir/session.json"
[[ -f "$session_file" && ! -L "$session_file" ]] || refuse SESSION_FILE_MISSING "$session_file"
jq -e '.schema_version == "bullet.dogfood-ops-session.v1"' "$session_file" >/dev/null \
  || refuse SESSION_FILE_INVALID "$session_file"

[[ -n "$manifest" ]] || refuse MANIFEST_REQUIRED "--manifest"
[[ "$manifest" == /* ]] || refuse MANIFEST_NOT_ABSOLUTE "$manifest"
[[ "$interval" =~ ^[0-9]+$ && "$interval" -ge 1 ]] || refuse INTERVAL_INVALID "$interval"
[[ "$max_idle" =~ ^[0-9]+$ ]] || refuse MAX_IDLE_INVALID "$max_idle"
[[ "$deadline_ms" =~ ^[0-9]+$ && "$deadline_ms" -ge 1000 ]] || refuse DEADLINE_MS_INVALID "$deadline_ms"

lease_socket="$(jq -r '.lease_socket // empty' "$session_file")"
farmd_uid="$(jq -r '.farmd_uid // empty' "$session_file")"
socket_gid="$(jq -r '.socket_gid // empty' "$session_file")"
runner_id="$(jq -r '.runner_id // empty' "$session_file")"
runner_epoch="$(jq -r '.runner_epoch // empty' "$session_file")"
[[ -S "$lease_socket" ]] || refuse LEASE_SOCKET_MISSING "$lease_socket (is serve.sh running?)"
[[ "$runner_id" =~ ^run_[0-9a-f]{64}$ ]] || refuse RUNNER_ID_INVALID "$runner_id"
[[ "$runner_epoch" =~ ^[0-9]+$ && "$farmd_uid" =~ ^[0-9]+$ && "$socket_gid" =~ ^[0-9]+$ ]] \
  || refuse SESSION_IDENTITY_INVALID "$runner_epoch/$farmd_uid/$socket_gid"

[[ -n "$worker_bin" ]] || refuse WORKER_BIN_REQUIRED "--worker or BULLET_COMMAND_WORKER_BIN"
[[ "$worker_bin" == /* && -f "$worker_bin" && -x "$worker_bin" ]] || refuse WORKER_BIN_INVALID "$worker_bin"

state_dir="$data_dir/worker"
if [[ -e "$state_dir" ]]; then
  [[ ! -L "$state_dir" && "$(stat -Lc '%u:%a:%F' -- "$state_dir")" == "$(id -u):700:directory" ]] \
    || refuse STATE_DIR_UNTRUSTED "$state_dir"
else
  mkdir -m 0700 -- "$state_dir"
fi
log_dir="$data_dir/logs"
[[ -d "$log_dir" ]] || mkdir -m 0700 -- "$log_dir"

# --- environment bindings ---------------------------------------------------
declare -a env_names=() env_values=()
if [[ -n "$env_file" ]]; then
  [[ "$env_file" == /* ]] || refuse ENV_FILE_NOT_ABSOLUTE "$env_file"
  [[ -f "$env_file" && ! -L "$env_file" ]] || refuse ENV_FILE_MISSING "$env_file"
  [[ "$(stat -Lc '%u:%a' -- "$env_file")" == "$(id -u):600" ]] \
    || refuse ENV_FILE_UNTRUSTED "$(stat -Lc '%u:%a' -- "$env_file")"
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    name="${line%%=*}"
    value="${line#*=}"
    [[ "$name" =~ ^BULLET_HARNESS_[A-Z0-9_]+$ ]] || refuse ENV_FILE_NAME_INVALID "$name"
    env_names+=("$name")
    env_values+=("$value")
  done <"$env_file"
  [[ ${#env_names[@]} -gt 0 ]] || refuse ENV_FILE_EMPTY "$env_file"
fi

# --- binary manifest --------------------------------------------------------
admit_subject() {
  local label="$1" path="$2"
  [[ -n "$path" ]] || refuse "${label}_BIN_REQUIRED" "no path for $label"
  [[ "$path" == /* ]] || refuse "${label}_BIN_NOT_ABSOLUTE" "$path"
  [[ -f "$path" && -x "$path" ]] || refuse "${label}_BIN_INVALID" "$path"
  [[ "$(realpath -e -- "$path")" == "$path" ]] || refuse "${label}_BIN_NOT_CANONICAL" "$path"
  local perm
  perm="$(stat -Lc '%a' -- "$path")"
  [[ $((8#$perm & 8#022)) -eq 0 ]] || refuse "${label}_BIN_UNPROTECTED" "$path mode $perm"
}

if [[ ! -e "$manifest" ]]; then
  [[ -n "$farmd_bin" ]] || farmd_bin="$(jq -r '.farmd_bin // empty' "$session_file")"
  admit_subject TRANSACTION_OFFLINE "$transaction_offline_bin"
  admit_subject FARMD "$farmd_bin"
  admit_subject RUNNER "$runner_bin"
  admit_subject GITD "$gitd_bin"
  admit_subject VERIFIER "$verifier_bin"
  manifest_tmp="$manifest.tmp.$$"
  jq -cS -n \
    --arg transaction_path "$transaction_offline_bin" \
    --arg transaction_sha "$(sha256sum -- "$transaction_offline_bin" | cut -d' ' -f1)" \
    --arg farmd_path "$farmd_bin" \
    --arg farmd_sha "$(sha256sum -- "$farmd_bin" | cut -d' ' -f1)" \
    --arg runner_path "$runner_bin" \
    --arg runner_sha "$(sha256sum -- "$runner_bin" | cut -d' ' -f1)" \
    --arg gitd_path "$gitd_bin" \
    --arg gitd_sha "$(sha256sum -- "$gitd_bin" | cut -d' ' -f1)" \
    --arg verifier_path "$verifier_bin" \
    --arg verifier_sha "$(sha256sum -- "$verifier_bin" | cut -d' ' -f1)" '
    {schema_version:"bullet.command-worker-binary-manifest.v1",
     transaction_offline:{path:$transaction_path,sha256:$transaction_sha},
     farmd:{path:$farmd_path,sha256:$farmd_sha},
     runner:{path:$runner_path,sha256:$runner_sha},
     gitd:{path:$gitd_path,sha256:$gitd_sha},
     verifier:{path:$verifier_path,sha256:$verifier_sha}}
  ' >"$manifest_tmp"
  # The worker compares the file bytes with canonical JSON, which has no
  # terminal newline. Strip exactly the one jq appended.
  last_byte="$(tail -c 1 "$manifest_tmp" | od -An -tuC | tr -d ' \n')"
  [[ "$last_byte" == 10 ]] || refuse MANIFEST_ENCODER_INVALID "missing terminal LF"
  truncate -s -1 "$manifest_tmp"
  chmod 0600 -- "$manifest_tmp"
  mv -f -- "$manifest_tmp" "$manifest"
fi
[[ -f "$manifest" && ! -L "$manifest" ]] || refuse MANIFEST_MISSING "$manifest"
manifest_perm="$(stat -Lc '%a' -- "$manifest")"
[[ $((8#$manifest_perm & 8#022)) -eq 0 ]] || refuse MANIFEST_UNPROTECTED "$manifest mode $manifest_perm"
jq -e '.schema_version == "bullet.command-worker-binary-manifest.v1"' "$manifest" >/dev/null \
  || refuse MANIFEST_SCHEMA_INVALID "$manifest"
manifest_sha="$(sha256sum -- "$manifest" | cut -d' ' -f1)"

# --- loop -------------------------------------------------------------------
stop_requested=0
stop_reason=""
trap 'stop_requested=1; stop_reason=SIGTERM' TERM
trap 'stop_requested=1; stop_reason=SIGINT' INT

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
loop_log="$log_dir/worker-loop-$stamp.log"
: >"$loop_log"
chmod 0600 -- "$loop_log"

emit() {
  printf '%s\n' "$1"
  printf '%s\n' "$1" >>"$loop_log"
}

emit "$(printf '%s start worker=%s manifest=%s manifest_sha256=%s runner_id=%s epoch=%s bindings=%d' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$worker_bin" "$manifest" "$manifest_sha" \
  "$runner_id" "$runner_epoch" "${#env_names[@]}")"

declare -a env_prefix=()
if [[ ${#env_names[@]} -gt 0 ]]; then
  env_prefix=(env)
  for index in "${!env_names[@]}"; do
    env_prefix+=("${env_names[$index]}=${env_values[$index]}")
  done
fi

declare -a worker_args=(--lease-socket "$lease_socket" --farmd-uid "$farmd_uid"
  --socket-gid "$socket_gid" --runner-id "$runner_id" --runner-epoch "$runner_epoch"
  --state-dir "$state_dir" --binary-manifest "$manifest" --deadline-ms "$deadline_ms")

iteration=0
idle_streak=0
backoff="$interval"
max_backoff=$((interval * 8))
exit_code=0
timeout_s=$(((deadline_ms / 1000) + 60))

while [[ "$stop_requested" -eq 0 ]]; do
  iteration=$((iteration + 1))
  out_file="$state_dir/iter-$iteration.stdout"
  err_file="$state_dir/iter-$iteration.stderr"
  started_ms="$(date -u +%s%3N)"
  # setsid keeps the child out of this loop's session, so a terminal SIGINT
  # cannot reach a claim that is already in flight.
  setsid --wait timeout --signal=TERM --kill-after=5s "${timeout_s}s" \
    "${env_prefix[@]}" "$worker_bin" "${worker_args[@]}" \
    >"$out_file" 2>"$err_file" &
  child=$!
  child_status=0
  while :; do
    if wait "$child"; then child_status=0; else child_status=$?; fi
    kill -0 "$child" 2>/dev/null || break
  done
  finished_ms="$(date -u +%s%3N)"
  result="$(head -n 1 "$out_file" | tr -d '\r')"
  [[ -n "$result" ]] || result="<none>"
  emit "$(printf '%s iter=%d exit=%d result=%s elapsed_ms=%d' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$iteration" "$child_status" "$result" \
    "$((finished_ms - started_ms))")"

  if [[ "$child_status" -ne 0 ]]; then
    emit "$(printf '%s refusal iter=%d stderr=%s' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$iteration" "$err_file")"
    if [[ -s "$err_file" ]]; then
      while IFS= read -r refusal_line; do emit "  $refusal_line"; done <"$err_file"
    fi
    exit_code=1
    break
  fi

  if [[ "$result" == NO_COMMAND ]]; then
    idle_streak=$((idle_streak + 1))
    if [[ "$max_idle" -gt 0 && "$idle_streak" -ge "$max_idle" ]]; then
      emit "$(printf '%s stop reason=max-idle idle=%d' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$idle_streak")"
      break
    fi
  else
    idle_streak=0
    backoff="$interval"
  fi

  [[ "$stop_requested" -eq 0 ]] || break
  sleep "$backoff"
  if [[ "$idle_streak" -gt 0 && "$backoff" -lt "$max_backoff" ]]; then
    backoff=$((backoff * 2))
    [[ "$backoff" -le "$max_backoff" ]] || backoff="$max_backoff"
  fi
done

if [[ "$stop_requested" -eq 1 ]]; then
  emit "$(printf '%s stop reason=%s iterations=%d' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$stop_reason" "$iteration")"
fi
emit "$(printf '%s end iterations=%d exit=%d log=%s' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$iteration" "$exit_code" "$loop_log")"
exit "$exit_code"
