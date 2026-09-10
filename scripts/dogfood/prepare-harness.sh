#!/usr/bin/env bash
# Derive the full BULLET_HARNESS_* environment a farmd command worker needs to
# spawn `bullet-runner --provider claude`, write it to a 0600 env file, and
# prove the binding with `bullet coding harness-check`.
#
# This wrapper grants no authority and spends nothing: it never launches a
# provider, never reads credential bytes, and never submits a command.
#
# Three of the seventeen REQUIRED names have no producer anywhere in the kernel
# today. This script says so on stdout, marks them PLACEHOLDER_DRY_RUN_ONLY in
# the env file, and refuses to pretend otherwise. See scripts/dogfood/README.md.
set -euo pipefail
umask 077

usage() {
  cat <<'EOF'
usage: prepare-harness.sh --data-dir <abs 0700 dir> --source-repo <abs git repo>
                          --objective "<text>" --gate-id <gat_64hex>
                          --scope <prefix> --preservation-dir <abs, absent>
                          --dogfood-data-dir <abs 0700 dir>
                          --dogfood-executable <abs>
                          [--session-file <abs>] [--env-file <abs>]
                          [--credential <src,target,blake3>]...
                          [--provider claude] [--max-budget-usd 0.75]
                          [--bullet <bullet bin>] [--harness-path /usr/bin:/bin]
                          [--allow-placeholders]
       prepare-harness.sh --help

Reads the 0600 session file serve.sh wrote (lease socket, farmd uid/gid, farmd
origin, lease-transport key path) and derives every BULLET_HARNESS_* name the
command worker's child stage consumes:

  apps/bullet/src/coding/harness.rs                17 REQUIRED + BULLET_HARNESS_PATH
  apps/bullet-runner/.../child/coding.rs           each name -> one bullet-runner flag
  apps/bullet-runner/.../child.rs                  HOME comes from BULLET_HARNESS_HOME

Real inputs are derived from the operator's own state. Names with no producer
are written as PLACEHOLDER_DRY_RUN_ONLY:<value> and reported by name; with
--allow-placeholders the script still exits 0 on a BOUND harness-check, but the
env file is only good for a dry run, never for a billable turn.

The env file is NOT a shell script. Each line is exactly NAME=VALUE with a raw
value; values containing a newline are refused. worker-loop.sh --env-file reads
the same shape.

Refusals are printed as DOGFOOD_OPS_<CODE>: <value> on stderr with exit 1.
An UNBOUND harness-check exits 2, matching `bullet coding harness-check`.
EOF
}

refuse() {
  printf 'DOGFOOD_OPS_%s: %s\n' "$1" "$2" >&2
  exit 1
}

provider="claude"
data_dir=""
session_file=""
source_repo=""
objective=""
preservation_dir=""
dogfood_data_dir=""
dogfood_executable=""
env_file=""
max_budget_usd="0.75"
bullet_bin="${BULLET_BIN:-}"
harness_path="/usr/bin:/bin"
allow_placeholders=0
declare -a gate_ids=() scopes=() credentials=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --provider) provider="${2:-}"; shift 2 ;;
    --data-dir) data_dir="${2:-}"; shift 2 ;;
    --session-file) session_file="${2:-}"; shift 2 ;;
    --source-repo) source_repo="${2:-}"; shift 2 ;;
    --objective) objective="${2:-}"; shift 2 ;;
    --gate-id) gate_ids+=("${2:-}"); shift 2 ;;
    --scope) scopes+=("${2:-}"); shift 2 ;;
    --preservation-dir) preservation_dir="${2:-}"; shift 2 ;;
    --dogfood-data-dir) dogfood_data_dir="${2:-}"; shift 2 ;;
    --dogfood-executable) dogfood_executable="${2:-}"; shift 2 ;;
    --credential) credentials+=("${2:-}"); shift 2 ;;
    --env-file) env_file="${2:-}"; shift 2 ;;
    --max-budget-usd) max_budget_usd="${2:-}"; shift 2 ;;
    --bullet) bullet_bin="${2:-}"; shift 2 ;;
    --harness-path) harness_path="${2:-}"; shift 2 ;;
    --allow-placeholders) allow_placeholders=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) refuse ARG_UNKNOWN "$1" ;;
  esac
done

for tool in jq git od stat realpath sha256sum b3sum; do
  command -v "$tool" >/dev/null 2>&1 || refuse TOOL_MISSING "$tool"
done

# The worker's Claude path is the only one wired to the dogfood admission flags.
[[ "$provider" == claude ]] || refuse PROVIDER_UNSUPPORTED "$provider (only claude derives BULLET_HARNESS_DOGFOOD_*)"

abs_existing_dir() {
  local label="$1" path="$2" mode="$3"
  [[ "$path" == /* ]] || refuse "${label}_NOT_ABSOLUTE" "$path"
  [[ -d "$path" && ! -L "$path" ]] || refuse "${label}_NOT_DIRECTORY" "$path"
  [[ "$(realpath -e -- "$path")" == "$path" ]] || refuse "${label}_NOT_CANONICAL" "$path"
  [[ "$(stat -Lc '%u:%a' -- "$path")" == "$(id -u):$mode" ]] \
    || refuse "${label}_UNTRUSTED" "$(stat -Lc '%u:%a' -- "$path") (want $(id -u):$mode)"
}

private_subdir() {
  local path="$1"
  if [[ -e "$path" || -L "$path" ]]; then
    [[ ! -L "$path" && "$(stat -Lc '%u:%a:%F' -- "$path")" == "$(id -u):700:directory" ]] \
      || refuse HARNESS_SUBDIR_UNTRUSTED "$path"
  else
    mkdir -m 0700 -- "$path"
  fi
}

[[ -n "$data_dir" ]] || refuse DATA_DIR_REQUIRED "--data-dir"
abs_existing_dir DATA_DIR "$data_dir" 700
[[ -z "$session_file" ]] && session_file="$data_dir/session.json"
[[ "$session_file" == /* ]] || refuse SESSION_FILE_NOT_ABSOLUTE "$session_file"
[[ -f "$session_file" && ! -L "$session_file" ]] || refuse SESSION_FILE_MISSING "$session_file"
[[ "$(stat -Lc '%u:%a:%F' -- "$session_file")" == "$(id -u):600:regular file" ]] \
  || refuse SESSION_FILE_UNTRUSTED "$(stat -Lc '%u:%a:%F' -- "$session_file")"
jq -e '.schema_version == "bullet.dogfood-ops-session.v1"' "$session_file" >/dev/null \
  || refuse SESSION_FILE_INVALID "$session_file"

lease_socket="$(jq -r '.lease_socket // empty' "$session_file")"
farmd_uid="$(jq -r '.farmd_uid // empty' "$session_file")"
socket_gid="$(jq -r '.socket_gid // empty' "$session_file")"
farmd_origin="$(jq -r '.farmd // empty' "$session_file")"
lease_key="$(jq -r '.key // empty' "$session_file")"
[[ -S "$lease_socket" ]] || refuse LEASE_SOCKET_MISSING "$lease_socket (is serve.sh running?)"
[[ "$farmd_uid" =~ ^[0-9]+$ && "$socket_gid" =~ ^[0-9]+$ ]] || refuse SESSION_IDENTITY_INVALID "$farmd_uid:$socket_gid"
[[ "$farmd_origin" =~ ^http://127\.0\.0\.1:[0-9]+$ ]] || refuse SESSION_ORIGIN_INVALID "$farmd_origin"

# --- source repository and base commit -------------------------------------
[[ -n "$source_repo" ]] || refuse SOURCE_REPO_REQUIRED "--source-repo"
[[ "$source_repo" == /* ]] || refuse SOURCE_REPO_NOT_ABSOLUTE "$source_repo"
[[ -d "$source_repo" ]] || refuse SOURCE_REPO_MISSING "$source_repo"
[[ "$(realpath -e -- "$source_repo")" == "$source_repo" ]] || refuse SOURCE_REPO_NOT_CANONICAL "$source_repo"
git -C "$source_repo" rev-parse --git-dir >/dev/null 2>&1 || refuse SOURCE_REPO_NOT_GIT "$source_repo"
base_sha="$(git -C "$source_repo" rev-parse HEAD 2>/dev/null || true)"
[[ "$base_sha" =~ ^[0-9a-f]{40}$ ]] || refuse BASE_SHA_UNAVAILABLE "$source_repo"

# --- objective, gate, scope -------------------------------------------------
[[ -n "$objective" ]] || refuse OBJECTIVE_REQUIRED "--objective"
[[ "$objective" != *$'\n'* ]] || refuse OBJECTIVE_MULTILINE "objective must be one line"
[[ ${#objective} -le 4096 ]] || refuse OBJECTIVE_TOO_LONG "${#objective} bytes"

[[ ${#gate_ids[@]} -ge 1 ]] || refuse GATE_ID_REQUIRED "--gate-id"
# The worker maps BULLET_HARNESS_GATE_ID to exactly one --gate-id flag, so more
# than one gate cannot be carried through this seam. Refuse rather than drop.
[[ ${#gate_ids[@]} -eq 1 ]] \
  || refuse GATE_ID_MULTIPLE_UNBINDABLE "${#gate_ids[@]} gates; BULLET_HARNESS_GATE_ID binds exactly one"
gate_id="${gate_ids[0]}"
[[ "$gate_id" =~ ^gat_[0-9a-f]{64}$ ]] || refuse GATE_ID_INVALID "$gate_id"

[[ ${#scopes[@]} -ge 1 ]] || refuse SCOPE_REQUIRED "--scope"
[[ ${#scopes[@]} -eq 1 ]] \
  || refuse SCOPE_MULTIPLE_UNBINDABLE "${#scopes[@]} scopes; BULLET_HARNESS_SCOPE binds exactly one"
scope="${scopes[0]}"
[[ -n "$scope" && "$scope" != /* && "$scope" != *".."* && "$scope" != *$'\n'* ]] \
  || refuse SCOPE_INVALID "$scope (relative prefix, no .., single line)"

# --- preservation destination ----------------------------------------------
[[ -n "$preservation_dir" ]] || refuse PRESERVATION_DIR_REQUIRED "--preservation-dir"
[[ "$preservation_dir" == /* ]] || refuse PRESERVATION_DIR_NOT_ABSOLUTE "$preservation_dir"
[[ ! -e "$preservation_dir" && ! -L "$preservation_dir" ]] \
  || refuse PRESERVATION_DIR_EXISTS "$preservation_dir must be an exact new directory"
preservation_parent="$(dirname -- "$preservation_dir")"
abs_existing_dir PRESERVATION_PARENT "$preservation_parent" 700

# --- dogfood admission inputs ----------------------------------------------
[[ -n "$dogfood_data_dir" ]] || refuse DOGFOOD_DATA_DIR_REQUIRED "--dogfood-data-dir"
abs_existing_dir DOGFOOD_DATA_DIR "$dogfood_data_dir" 700
dogfood_policy="$dogfood_data_dir/policy/policy.json"
dogfood_binding="$dogfood_data_dir/policy/binding.json"
dogfood_enrollment="$dogfood_data_dir/policy/enrollments/$provider.json"
for required_file in "$dogfood_policy" "$dogfood_binding" "$dogfood_enrollment"; do
  [[ -f "$required_file" && ! -L "$required_file" ]] || refuse DOGFOOD_INPUT_MISSING "$required_file"
  [[ "$(stat -Lc '%u:%a' -- "$required_file")" == "$(id -u):600" ]] \
    || refuse DOGFOOD_INPUT_UNTRUSTED "$required_file $(stat -Lc '%u:%a' -- "$required_file")"
done

[[ -n "$dogfood_executable" ]] || refuse DOGFOOD_EXECUTABLE_REQUIRED "--dogfood-executable"
[[ "$dogfood_executable" == /* ]] || refuse DOGFOOD_EXECUTABLE_NOT_ABSOLUTE "$dogfood_executable"
[[ -f "$dogfood_executable" && -x "$dogfood_executable" ]] || refuse DOGFOOD_EXECUTABLE_INVALID "$dogfood_executable"
[[ "$(realpath -e -- "$dogfood_executable")" == "$dogfood_executable" ]] \
  || refuse DOGFOOD_EXECUTABLE_NOT_CANONICAL "$dogfood_executable"
executable_perm="$(stat -Lc '%a' -- "$dogfood_executable")"
[[ $((8#$executable_perm & 8#022)) -eq 0 ]] \
  || refuse DOGFOOD_EXECUTABLE_WRITABLE "$dogfood_executable mode $executable_perm"

[[ "$max_budget_usd" =~ ^[0-9]+(\.[0-9]+)?$ ]] || refuse MAX_BUDGET_INVALID "$max_budget_usd"
[[ "$max_budget_usd" != 0 && "$max_budget_usd" != 0.0* ]] || refuse MAX_BUDGET_NOT_POSITIVE "$max_budget_usd"

# Credential grants are admitted by shape and by the existence of the source
# only. Their bytes are never read here: recomputing the digest would mean
# reading a credential, which this lane must not do.
for grant in ${credentials[@]+"${credentials[@]}"}; do
  IFS=, read -r cred_src cred_target cred_digest <<<"$grant"
  [[ -n "${cred_src:-}" && -n "${cred_target:-}" && -n "${cred_digest:-}" ]] \
    || refuse CREDENTIAL_SHAPE_INVALID "$grant (want source,target,blake3)"
  [[ "$cred_src" == /* ]] || refuse CREDENTIAL_SOURCE_NOT_ABSOLUTE "$cred_src"
  [[ -f "$cred_src" ]] || refuse CREDENTIAL_SOURCE_MISSING "$cred_src"
  [[ "$cred_target" != /* && "$cred_target" != *".."* ]] || refuse CREDENTIAL_TARGET_INVALID "$cred_target"
  [[ "$cred_digest" =~ ^[0-9a-f]{64}$ ]] || refuse CREDENTIAL_DIGEST_INVALID "<redacted digest shape>"
done

# --- bullet binary ----------------------------------------------------------
[[ -n "$bullet_bin" ]] || bullet_bin="$(command -v bullet 2>/dev/null || true)"
[[ -n "$bullet_bin" ]] || refuse BULLET_BIN_REQUIRED "--bullet or BULLET_BIN"
[[ "$bullet_bin" == /* && -f "$bullet_bin" && -x "$bullet_bin" ]] || refuse BULLET_BIN_INVALID "$bullet_bin"

# --- derived private tree ---------------------------------------------------
harness_root="$data_dir/harness"
private_subdir "$harness_root"
harness_home="$harness_root/home"
workspace_root="$harness_root/workspace"
recovery_dir="$harness_root/recovery"
receipt_dir="$harness_root/receipts"
for sub in "$harness_home" "$workspace_root" "$recovery_dir" "$receipt_dir"; do
  private_subdir "$sub"
done
lease_recovery="$recovery_dir/acquire.json"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
dogfood_receipt="$receipt_dir/$provider-$stamp.json"
[[ ! -e "$dogfood_receipt" ]] || refuse DOGFOOD_RECEIPT_EXISTS "$dogfood_receipt is create-once"
[[ -z "$env_file" ]] && env_file="$harness_root/harness.env"
[[ "$env_file" == /* ]] || refuse ENV_FILE_NOT_ABSOLUTE "$env_file"

# --- Candidate verification key: REAL, derived from farmd's own key ---------
# apps/bullet-farmd/src/main/launch.rs derives its Candidate-preparation
# signing key from the very bytes of --lease-transport-key with the fixed
# labels kernel-local / candidate-preparation-1. A PASETO v4.public secret key
# is seed||public, so the public half this record must carry is the last 32
# bytes of that file. Nothing is invented here and no secret is emitted.
[[ -f "$lease_key" && ! -L "$lease_key" ]] || refuse LEASE_KEY_MISSING "$lease_key"
[[ "$(stat -Lc '%u:%a:%s' -- "$lease_key")" == "$(id -u):600:64" ]] \
  || refuse LEASE_KEY_CUSTODY_INVALID "$(stat -Lc '%u:%a:%s' -- "$lease_key")"
candidate_public_hex="$(od -An -v -tx1 -j32 -N32 -- "$lease_key" | tr -d ' \n')"
[[ "$candidate_public_hex" =~ ^[0-9a-f]{64}$ ]] || refuse CANDIDATE_KEY_DERIVATION_FAILED "public half"
[[ "$candidate_public_hex" != "$(printf '0%.0s' $(seq 1 64))" ]] \
  || refuse CANDIDATE_KEY_DERIVATION_FAILED "all-zero public half"
candidate_key_file="$data_dir/custody/candidate-verification-key.json"
candidate_tmp="$candidate_key_file.tmp.$$"
jq -cS -n --arg public "$candidate_public_hex" '
  {schema_version:"v1alpha1",issuer:"kernel-local",key_id:"candidate-preparation-1",
   public_key_hex:$public}' >"$candidate_tmp"
printf '%s' "$(cat "$candidate_tmp")" >"$candidate_tmp.strip"
mv -f -- "$candidate_tmp.strip" "$candidate_tmp"
chmod 0600 -- "$candidate_tmp"
mv -f -- "$candidate_tmp" "$candidate_key_file"
[[ "$(stat -Lc '%u:%a:%h' -- "$candidate_key_file")" == "$(id -u):600:1" ]] \
  || refuse CANDIDATE_KEY_CUSTODY_INVALID "$(stat -Lc '%u:%a:%h' -- "$candidate_key_file")"

# --- names with no producer in the kernel today -----------------------------
# WorkPackageId::from_seed is blake3("wpk:<seed>") (crates/domain/src/ids.rs),
# so the shape below is exactly what the Kernel would mint -- but no Kernel
# selection, ledger row, or CLI produced it. It is a placeholder.
placeholder_seed="dogfood-ops:$data_dir:$base_sha:$gate_id:$scope"
work_package_id="wpk_$(printf 'wpk:%s' "$placeholder_seed" | b3sum --no-names | tr -d ' \n')"
candidate_request_digest="$(printf 'candidate-request:%s' "$placeholder_seed" | sha256sum | cut -c1-64)"
idempotency_key="dfk_$(printf 'idem:%s' "$placeholder_seed" | sha256sum | cut -c1-32)"
declare -a placeholder_names=(
  BULLET_HARNESS_WORK_PACKAGE_ID
  BULLET_HARNESS_CANDIDATE_REQUEST_DIGEST
  BULLET_HARNESS_IDEMPOTENCY_KEY
)

# --- assemble ---------------------------------------------------------------
declare -a names=() values=()
put() { names+=("$1"); values+=("$2"); }
put BULLET_HARNESS_HOME "$harness_home"
put BULLET_HARNESS_WORK_PACKAGE_ID "PLACEHOLDER_DRY_RUN_ONLY:$work_package_id"
put BULLET_HARNESS_CANDIDATE_REQUEST_DIGEST "PLACEHOLDER_DRY_RUN_ONLY:$candidate_request_digest"
put BULLET_HARNESS_CANDIDATE_VERIFICATION_KEY "$candidate_key_file"
put BULLET_HARNESS_WORKSPACE_ROOT "$workspace_root"
put BULLET_HARNESS_SOURCE_REPO "$source_repo"
put BULLET_HARNESS_BASE_SHA "$base_sha"
put BULLET_HARNESS_PRESERVATION "$preservation_dir"
put BULLET_HARNESS_OBJECTIVE "$objective"
put BULLET_HARNESS_GATE_ID "$gate_id"
put BULLET_HARNESS_SCOPE "$scope"
put BULLET_HARNESS_IDEMPOTENCY_KEY "PLACEHOLDER_DRY_RUN_ONLY:$idempotency_key"
put BULLET_HARNESS_LEASE_SOCKET "$lease_socket"
put BULLET_HARNESS_FARMD_UID "$farmd_uid"
put BULLET_HARNESS_SOCKET_GID "$socket_gid"
put BULLET_HARNESS_LEASE_RECOVERY "$lease_recovery"
put BULLET_HARNESS_EXECUTABLE "$dogfood_executable"
put BULLET_HARNESS_PATH "$harness_path"
put BULLET_HARNESS_FARMD "$farmd_origin"
put BULLET_HARNESS_DOGFOOD_DATA_DIR "$dogfood_data_dir"
put BULLET_HARNESS_DOGFOOD_POLICY "$dogfood_policy"
put BULLET_HARNESS_DOGFOOD_BINDING "$dogfood_binding"
put BULLET_HARNESS_DOGFOOD_ENROLLMENT "$dogfood_enrollment"
put BULLET_HARNESS_DOGFOOD_ISSUER "${BULLET_DOGFOOD_ISSUER:-dogfood-local}"
put BULLET_HARNESS_DOGFOOD_KEY_ID "${BULLET_DOGFOOD_KEY_ID:-dogfood-runner-1}"
put BULLET_HARNESS_DOGFOOD_RECEIPT "$dogfood_receipt"
put BULLET_HARNESS_DOGFOOD_MAX_BUDGET_USD "$max_budget_usd"
if [[ ${#credentials[@]} -gt 0 ]]; then
  # Recorded for the operator only. The worker's Claude argv builder
  # (child/coding.rs claude_dogfood_args) emits no --dogfood-credential flag,
  # so this value reaches nothing today.
  put BULLET_HARNESS_DOGFOOD_CREDENTIALS "NOT_CONSUMED_BY_WORKER:$(IFS=';'; printf '%s' "${credentials[*]}")"
fi

env_tmp="$env_file.tmp.$$"
: >"$env_tmp"
chmod 0600 -- "$env_tmp"
for index in "${!names[@]}"; do
  value="${values[$index]}"
  [[ "$value" != *$'\n'* ]] || refuse ENV_VALUE_MULTILINE "${names[$index]}"
  printf '%s=%s\n' "${names[$index]}" "$value" >>"$env_tmp"
done
mv -f -- "$env_tmp" "$env_file"

# --- harness-check ----------------------------------------------------------
declare -a env_args=(-i "PATH=$harness_path" "HOME=$harness_home")
for index in "${!names[@]}"; do
  env_args+=("${names[$index]}=${values[$index]}")
done
check_json="$harness_root/harness-check.json"
check_status=0
env "${env_args[@]}" "$bullet_bin" coding harness-check --json >"$check_json" 2>"$harness_root/harness-check.stderr" \
  || check_status=$?
jq -e '.outcome' "$check_json" >/dev/null 2>&1 \
  || refuse HARNESS_CHECK_UNREADABLE "$check_json (exit $check_status; see $harness_root/harness-check.stderr)"
outcome="$(jq -r .outcome "$check_json")"

printf 'env_file=%s\n' "$env_file"
printf 'candidate_verification_key=%s (derived from the farmd lease-transport key)\n' "$candidate_key_file"
printf 'base_sha=%s\n' "$base_sha"
printf 'preservation=%s (absent, created by the runner)\n' "$preservation_dir"
printf 'lease_recovery=%s (absent, created on first acquire)\n' "$lease_recovery"
printf 'dogfood_receipt=%s (create-once)\n' "$dogfood_receipt"
printf 'harness_check=%s exit=%s json=%s\n' "$outcome" "$check_status" "$check_json"

printf '\nrequired bindings (bullet coding harness-check):\n'
jq -r '.bindings[] | "  \(.name) \(.state)"' "$check_json"

# harness-check treats the eight Claude names as optional, so report them here.
printf '\nclaude dogfood bindings (optional to harness-check, required by the worker):\n'
for name in BULLET_HARNESS_DOGFOOD_DATA_DIR BULLET_HARNESS_DOGFOOD_POLICY \
  BULLET_HARNESS_DOGFOOD_BINDING BULLET_HARNESS_DOGFOOD_ENROLLMENT \
  BULLET_HARNESS_DOGFOOD_ISSUER BULLET_HARNESS_DOGFOOD_KEY_ID \
  BULLET_HARNESS_DOGFOOD_RECEIPT BULLET_HARNESS_DOGFOOD_MAX_BUDGET_USD; do
  state=ABSENT
  for index in "${!names[@]}"; do
    if [[ "${names[$index]}" == "$name" && -n "${values[$index]}" ]]; then state=PRESENT; fi
  done
  printf '  %s %s\n' "$name" "$state"
done

printf '\nno producer in the kernel today (values are dry-run placeholders):\n'
for name in "${placeholder_names[@]}"; do
  printf '  %s PLACEHOLDER_DRY_RUN_ONLY\n' "$name"
done
printf '  BULLET_HARNESS_DOGFOOD_CREDENTIALS is recorded but never reaches bullet-runner:\n'
printf '    child/coding.rs claude_dogfood_args() emits no --dogfood-credential flag.\n'

[[ "$outcome" == BOUND ]] || { printf 'DOGFOOD_OPS_HARNESS_UNBOUND: %s\n' "$check_json" >&2; exit 2; }
if [[ "$allow_placeholders" -eq 0 ]]; then
  printf 'DOGFOOD_OPS_HARNESS_PLACEHOLDERS_PRESENT: %s\n' "${placeholder_names[*]}" >&2
  printf 'this env file is dry-run only; pass --allow-placeholders to acknowledge\n' >&2
  exit 1
fi
printf '\nBOUND (dry-run only: %s carry placeholders)\n' "${placeholder_names[*]}"
