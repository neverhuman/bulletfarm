#!/usr/bin/env bash
set -euo pipefail
shopt -s inherit_errexit

# Shared data admission for the bounded normalization stage. This does not
# qualify a provider, a recording, Portal state, or a rendered media generation.
LIVE_HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
LIVE_INPUTS=(scripts/readme-live-record.sh scripts/readme-live-render.sh
  scripts/readme-live-check.sh scripts/readme-schema-check.sh)
LIVE_STAGE_FILES=(capture.json manifest.json observation.json stderr.txt stdout.txt transcript.txt)
LIVE_PORTAL_FRAMES=(01-control-tower.png 02-shift-brief.png 03-fleet.png
  04-mission-graph.png 05-control-tower-return.png)

live_die() { printf 'readme-live-check: %s\n' "$*" >&2; exit 1; }

live_start() {
  local tool
  for tool in bash cat cmp cp find grep iconv jq mkdir mktemp mv realpath rm sha256sum sort stat tr; do
    command -v "$tool" >/dev/null 2>&1 || live_die "TOOL_UNAVAILABLE $tool"
  done
  LIVE_TMP="$(mktemp -d)"
}

live_root_subject() {
  local root="$1"
  [[ "$root" == /* && -d "$root" && ! -L "$root" &&
    "$root" == "$(realpath -e -- "$root")" ]] || live_die UNSAFE_ROOT
  stat -c '%d:%i:%f:%u:%g' -- "$root"
}

live_inventory() {
  local root="$1" name limit
  shift
  live_root_subject "$root" >/dev/null
  printf '%s\n' "$@" | LC_ALL=C sort >"$LIVE_TMP/expected-inventory"
  find -P "$root" -mindepth 1 -printf '%P\n' | LC_ALL=C sort >"$LIVE_TMP/actual-inventory"
  cmp -s "$LIVE_TMP/expected-inventory" "$LIVE_TMP/actual-inventory" || live_die INVENTORY_DRIFT
  for name in "$@"; do
    limit=65536
    [[ "$name" != *.png && "$name" != *.gif ]] || limit=3145728
    [[ -f "$root/$name" && ! -L "$root/$name" &&
      "$(stat -c '%h' -- "$root/$name")" == 1 &&
      "$(stat -c '%s' -- "$root/$name")" -le "$limit" ]] || live_die UNSAFE_ARTIFACT
  done
}

live_files() {
  local root="$1" name digest bytes result='[]'
  shift
  for name in "$@"; do
    digest="$(sha256sum -- "$root/$name")"; digest="${digest%% *}"
    bytes="$(stat -c '%s' -- "$root/$name")"
    result="$(jq -c --arg path "$name" --arg sha256 "$digest" --argjson bytes "$bytes" \
      '. + [{path:$path,sha256:$sha256,bytes:$bytes}]' <<<"$result")"
  done
  jq -cS 'sort_by(.path)' <<<"$result"
}

live_sources() {
  local path
  for path in "${LIVE_INPUTS[@]}"; do
    [[ -f "$LIVE_HUB/$path" && ! -L "$LIVE_HUB/$path" &&
      "$(realpath -e -- "$LIVE_HUB/$path")" == "$LIVE_HUB/$path" ]] || live_die UNSAFE_SOURCE
  done
  live_files "$LIVE_HUB" "${LIVE_INPUTS[@]}"
}

live_strict_json() {
  bash "$LIVE_HUB/scripts/readme-schema-check.sh" --strict-json "$@" || live_die INVALID_JSON
}

live_artifacts() {
  local root="$1" manifest="$2" expected actual
  shift 2
  jq -e '.artifact_hashes | type == "array" and length > 0 and all(.[];
    (keys | sort) == ["bytes","path","sha256"] and
    (.path | type == "string") and (.sha256 | test("^[0-9a-f]{64}$")) and
    (.bytes | type == "number" and . >= 0 and . <= 3145728 and floor == .))' \
    "$manifest" >/dev/null || live_die INVALID_ARTIFACT_SET
  # Command substitution propagates parser failure; no process-substitution loop.
  expected="$(jq -cS '.artifact_hashes | sort_by(.path)' "$manifest")"
  actual="$(live_files "$root" "$@")"
  [[ "$expected" == "$actual" ]] || live_die ARTIFACT_DRIFT
}

live_text() {
  local root="$1" file code=0
  local -a streams=("${LIVE_CAPTURE_FILES[@]:1}") paths=()
  for file in "${streams[@]}"; do
    paths+=("$root/$file")
    iconv -f UTF-8 -t UTF-8 "$root/$file" >"$LIVE_TMP/utf8" || live_die INVALID_UTF8
    cmp -s "$root/$file" "$LIVE_TMP/utf8" || live_die INVALID_UTF8
    LC_ALL=C tr -d '\12\40-\176\200-\377' <"$root/$file" >"$LIVE_TMP/control"
    [[ ! -s "$LIVE_TMP/control" ]] || live_die CONTROL_CHARACTER
  done
  [[ -s "$root/stdout.txt" ]] || live_die EMPTY_REPLY
  jq -r '.. | strings' "$root/capture.json" >"$LIVE_TMP/capture-strings"
  LC_ALL=C grep -aEiq \
    '(/home/|/Users/|Authorization:|Bearer[[:space:]]|BEGIN [A-Z ]*PRIVATE KEY|sk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|(AKIA|ASIA)[0-9A-Z]{16}|xox[baprs]-|[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,})' \
    "$LIVE_TMP/capture-strings" "${paths[@]}" || code=$?
  [[ "$code" != 0 ]] || live_die REDACTION_REQUIRED
  [[ "$code" == 1 ]] || live_die SCANNER_FAILED
}

live_capture() {
  local root="$1"
  if [[ "$LIVE_PROFILE" == portal ]]; then live_portal_capture "$root"; return; fi
  live_strict_json "$root/capture.json"
  jq -e '
    (keys | sort) == ["artifact_hashes","classification","command","demo_id",
      "observed_at","release_authority","schema_version"] and
    .schema_version == "bullet.readme-live-capture.v2" and
    .classification == "UNSIGNED_OPERATOR_SUPPLIED_CAPTURE" and .release_authority == false and
    all(.. | strings; test("[\u0000-\u001f\u007f]") | not) and
    (.demo_id == "claude-session" or .demo_id == "codex-session" or .demo_id == "cursor-session") and
    (.observed_at == "UNKNOWN" or (.observed_at | type == "string" and
      test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$") and
      (. as $at | (fromdateiso8601 | strftime("%Y-%m-%dT%H:%M:%SZ")) == $at))) and
    (.command | (keys | sort) == ["argv","exit_code","requested_effort","requested_model","status"] and
      ((.status == "COMPLETED" and .exit_code == 0 and
        (.requested_model | type == "string" and length > 0 and length <= 128) and
        (.requested_effort | type == "string" and length > 0 and length <= 128) and
        (.argv | type == "array" and length > 0 and length <= 32 and
          all(.[]; type == "string" and length > 0 and length <= 4096))) or
        . == {status:"UNKNOWN",exit_code:null,argv:null,requested_model:"UNKNOWN",requested_effort:"UNKNOWN"})) and
    (.command.status == "UNKNOWN" or .command.argv[0] ==
      ({"claude-session":"claude","codex-session":"codex","cursor-session":"cursor-agent"}[.demo_id]))
  ' "$root/capture.json" >/dev/null || live_die UNSUPPORTED_OR_INCOMPLETE_CAPTURE
  live_artifacts "$root" "$root/capture.json" "${LIVE_CAPTURE_FILES[@]:1}"
  live_text "$root"
}

live_expected() {
  local root="$1" output="$2" capture_sha reply_sha
  if [[ "$LIVE_PROFILE" == portal ]]; then live_portal_expected "$root" "$output"; return; fi
  capture_sha="$(sha256sum "$root/capture.json")"; capture_sha="${capture_sha%% *}"
  reply_sha="$(sha256sum "$root/stdout.txt")"; reply_sha="${reply_sha%% *}"
  {
    printf '%s\n' 'Operator-supplied capture; provenance unverified' \
      'Model, account, authentication, files modified: UNKNOWN' \
      'Bullet transaction proof: ABSENT' 'Reply (literal captured text):'
    cat -- "$root/stdout.txt"
  } >"$output/transcript.txt"
  jq -S --arg capture_sha "$capture_sha" --arg reply_sha "$reply_sha" '{
    schema_version:"bullet.readme-live-normalization.v2", document_type:"observation",
    classification:.classification, demo_id:.demo_id, release_authority:false,
    status:"NORMALIZED_CAPTURE_ONLY", capture_verified:false, media_verified:false,
    operator_reported:{observed_at:.observed_at,command:.command},
    observed:{model:"UNKNOWN",effort:"UNKNOWN",account:"UNKNOWN",subscription:"UNKNOWN",
      authentication:"UNKNOWN",files_modified:"UNKNOWN",source_subject:"UNKNOWN",
      operator_cli_execution:"UNKNOWN",bullet_dispatch:"UNKNOWN",ledger_sequence:"UNKNOWN"},
    connected_transaction_proof:"ABSENT", capture_sha256:$capture_sha,
    reply_sha256:$reply_sha, reply_format:"UTF8_PLAINTEXT"
  }' "$root/capture.json" >"$output/observation.json"
}

live_manifest() {
  local root="$1" output="$2" sources="$3" artifacts
  artifacts="$(live_files "$root" "${LIVE_CAPTURE_FILES[@]}" observation.json transcript.txt)"
  jq -nS --argjson sources "$sources" --argjson artifacts "$artifacts" '{
    schema_version:"bullet.readme-live-normalization.v2", document_type:"manifest",
    classification:"UNSIGNED_OPERATOR_SUPPLIED_CAPTURE", release_authority:false,
    status:"NORMALIZED_CAPTURE_ONLY", capture_verified:false, media_verified:false,
    generation:{name:"bullet-readme-data-normalization",version:2,inputs:$sources},
    artifact_hashes:$artifacts
  }' >"$output"
}

live_stage() {
  local root="$1" sources="$2"
  live_profile "$root"
  if [[ "${3:-normalized}" == rendered ]]; then
    LIVE_STAGE_FILES+=("$(jq -er '.demo_id' "$root/capture.json").gif" fallback.png frames.framemd5 layout.json render.json)
  fi
  live_inventory "$root" "${LIVE_STAGE_FILES[@]}"
  live_capture "$root"
  live_strict_json "$root/observation.json" "$root/manifest.json"
  mkdir -p "$LIVE_TMP/expected"
  live_expected "$root" "$LIVE_TMP/expected"
  cmp -s "$LIVE_TMP/expected/transcript.txt" "$root/transcript.txt" || live_die REPLY_DRIFT
  cmp -s "$LIVE_TMP/expected/observation.json" "$root/observation.json" || live_die OBSERVATION_DRIFT
  live_artifacts "$root" "$root/manifest.json" "${LIVE_CAPTURE_FILES[@]}" observation.json transcript.txt
  live_manifest "$root" "$LIVE_TMP/expected/manifest.json" "$sources"
  cmp -s "$LIVE_TMP/expected/manifest.json" "$root/manifest.json" || live_die GENERATION_DRIFT
}

live_snapshot() {
  local root="$1" destination="$2" before after
  shift 2
  live_inventory "$root" "$@"
  before="$(live_files "$root" "$@")"
  mkdir "$destination"
  cp -a --no-dereference -- "$root/." "$destination/"
  live_inventory "$destination" "$@"
  after="$(live_files "$destination" "$@")"
  [[ "$before" == "$after" ]] || live_die INPUT_CHANGED
}

live_recheck() {
  local root="$1" snapshot="$2" subject="$3" sources="$4"
  shift 4
  live_inventory "$root" "$@"
  [[ "$(live_root_subject "$root")" == "$subject" &&
    "$(live_files "$root" "$@")" == "$(live_files "$snapshot" "$@")" &&
    "$(live_sources)" == "$sources" ]] || live_die INPUT_CHANGED
}

live_profile() {
  local root="$1" schema
  live_root_subject "$root" >/dev/null
  [[ -e "$root/capture.json" || -L "$root/capture.json" ]] || live_die INVENTORY_DRIFT
  [[ -f "$root/capture.json" && ! -L "$root/capture.json" &&
    "$(stat -c '%h' "$root/capture.json")" == 1 &&
    "$(stat -c '%s' "$root/capture.json")" -le 65536 ]] || live_die UNSAFE_ARTIFACT
  live_strict_json "$root/capture.json"
  schema="$(jq -er '.schema_version' "$root/capture.json")"
  case "$schema" in
    bullet.readme-live-capture.v2)
      LIVE_PROFILE=cli; LIVE_CAPTURE_FILES=(capture.json stderr.txt stdout.txt)
      if [[ "$(jq -r '.command.status' "$root/capture.json")" == UNKNOWN ]]; then
        LIVE_CAPTURE_FILES=(capture.json stdout.txt)
      fi ;;
    bullet.readme-portal-capture.v2)
      LIVE_PROFILE=portal; LIVE_CAPTURE_FILES=(capture.json "${LIVE_PORTAL_FRAMES[@]}") ;;
    *) live_die UNSUPPORTED_OR_INCOMPLETE_CAPTURE ;;
  esac
  LIVE_STAGE_FILES=("${LIVE_CAPTURE_FILES[@]}" manifest.json observation.json transcript.txt)
}

live_portal_capture() {
  local root="$1"
  jq -e '
    (keys | sort) == ["artifact_hashes","classification","demo_id","observed_at",
      "release_authority","schema_version"] and
    .schema_version == "bullet.readme-portal-capture.v2" and .demo_id == "portal-ui" and
    .classification == "UNSIGNED_OPERATOR_SUPPLIED_CAPTURE" and .release_authority == false and
    (.observed_at == "UNKNOWN" or (.observed_at | type == "string" and
      test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$") and
      (. as $at | (fromdateiso8601 | strftime("%Y-%m-%dT%H:%M:%SZ")) == $at)))
  ' "$root/capture.json" >/dev/null || live_die UNSUPPORTED_OR_INCOMPLETE_CAPTURE
  live_artifacts "$root" "$root/capture.json" "${LIVE_PORTAL_FRAMES[@]}"
}

live_portal_expected() {
  local root="$1" output="$2" capture_sha
  capture_sha="$(sha256sum "$root/capture.json")"; capture_sha="${capture_sha%% *}"
  printf '%s\n' 'Portal: operator-supplied screenshots; provenance unverified' \
    'Backend identity, ledger state and sequence: UNKNOWN' \
    'Source/runtime, authentication and accessibility: UNKNOWN' \
    'Bullet transaction proof: ABSENT' >"$output/transcript.txt"
  jq -S --arg capture_sha "$capture_sha" '{
    schema_version:"bullet.readme-live-normalization.v2",document_type:"observation",
    classification:.classification,demo_id:.demo_id,release_authority:false,
    status:"NORMALIZED_CAPTURE_ONLY",capture_verified:false,media_verified:false,
    operator_reported:{observed_at:.observed_at},
    observed:{backend_identity:"UNKNOWN",ledger_state:"UNKNOWN",ledger_sequence:"UNKNOWN",
      source_subject:"UNKNOWN",runtime_subject:"UNKNOWN",authentication:"UNKNOWN",accessibility:"UNKNOWN"},
    connected_transaction_proof:"ABSENT",capture_sha256:$capture_sha,
    frame_format:"OPERATOR_SUPPLIED_PNG"
  }' "$root/capture.json" >"$output/observation.json"
}

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then return 0; fi
if [[ "$#" == 0 || ( "$#" == 2 && "$1" == --tools-receipt ) ]]; then
  if [[ -f "$LIVE_HUB/docs/readme-live-media/collection.json" ]]; then
    [[ "$#" == 2 ]] || { echo 'readme-live-check: TOOL_RECEIPT_REQUIRED' >&2; exit 78; }
    exec bash "$LIVE_HUB/scripts/readme-live-render.sh" --verify-collection "$LIVE_HUB/docs/readme-live-media" "$@"
  fi
  echo 'readme-live-check: MEDIA_GENERATION_UNQUALIFIED (historical v1 preserved; no normal docs PASS)' >&2
  exit 78
fi
if [[ "$#" == 4 && "$1" == --rendered-collection && "$3" == --tools-receipt ]]; then
  exec bash "$LIVE_HUB/scripts/readme-live-render.sh" --verify-collection "$2" "$3" "$4"
fi
[[ "$#" == 2 && "$1" == --normalized-stage ]] || {
  echo 'usage: readme-live-check.sh --normalized-stage ABSOLUTE_DIRECTORY | [--rendered-collection ABSOLUTE_DIRECTORY] --tools-receipt ABSOLUTE_PRIVATE_FILE' >&2; exit 2;
}
live_start
trap 'rm -rf -- "$LIVE_TMP"' EXIT
sources="$(live_sources)"
subject="$(live_root_subject "$2")"
live_profile "$2"
live_snapshot "$2" "$LIVE_TMP/snapshot" "${LIVE_STAGE_FILES[@]}"
live_stage "$LIVE_TMP/snapshot" "$sources"
live_recheck "$2" "$LIVE_TMP/snapshot" "$subject" "$sources" "${LIVE_STAGE_FILES[@]}"
echo 'readme-live-check: NORMALIZED_CAPTURE_ONLY (capture and media remain unverified)'
