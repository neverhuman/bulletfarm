#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="${1:-$HUB/docs/readme-media}"
SCHEMA="$HUB/docs/schemas/bullet.readme-demo.v1.schema.json"
SNAPSHOT="$MEDIA/snapshot.json"
VHS_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
SOURCE_EPOCH=1787616000

if [[ "${1:-}" == "--strict-json" ]]; then
  shift
  (( "$#" > 0 )) || {
    echo "readme-schema-check: --strict-json requires at least one file" >&2
    exit 2
  }
  if command -v python3 >/dev/null 2>&1; then
    strict_python=python3
  elif command -v python >/dev/null 2>&1; then
    strict_python=python
  else
    echo "readme-schema-check: Python 3.12 is required" >&2
    exit 1
  fi
  [[ "$("$strict_python" --version 2>&1)" == "Python 3.12."* ]] || {
    echo "readme-schema-check: Python 3.12 is required" >&2
    exit 1
  }
  exec "$strict_python" -I -S - "$@" <<'PY'
import json
import math
import sys
from decimal import Decimal, InvalidOperation

MAX_DOCUMENT_BYTES = 4 * 1024 * 1024
MAX_SAFE_INTEGER = 9_007_199_254_740_991


def reject_duplicate_members(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON object member")
        result[key] = value
    return result


def reject_non_json_constant(_value):
    raise ValueError("non-JSON numeric constant")


def parse_strict_integer(value):
    parsed = int(value)
    if abs(parsed) > MAX_SAFE_INTEGER:
        raise ValueError("integer exceeds the interoperable safe range")
    return parsed


def parse_strict_float(value):
    try:
        exact = Decimal(value)
        binary = float(value)
        round_trip = Decimal(repr(binary))
    except (InvalidOperation, OverflowError):
        raise ValueError("JSON number is outside the interoperable range") from None
    if not math.isfinite(binary) or exact != round_trip:
        raise ValueError("JSON number loses precision")
    if exact == exact.to_integral_value() and abs(exact) > MAX_SAFE_INTEGER:
        raise ValueError("integer exceeds the interoperable safe range")
    return exact


def reject_non_finite_numbers(value):
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError("non-finite JSON number")
    if isinstance(value, dict):
        for member in value.values():
            reject_non_finite_numbers(member)
    elif isinstance(value, list):
        for member in value:
            reject_non_finite_numbers(member)


for path in sys.argv[1:]:
    try:
        with open(path, "rb") as handle:
            encoded = handle.read(MAX_DOCUMENT_BYTES + 1)
        if len(encoded) > MAX_DOCUMENT_BYTES:
            raise ValueError("JSON document exceeds the size bound")
        document = json.loads(
            encoded.decode("utf-8"),
            object_pairs_hook=reject_duplicate_members,
            parse_constant=reject_non_json_constant,
            parse_float=parse_strict_float,
            parse_int=parse_strict_integer,
        )
        reject_non_finite_numbers(document)
    except (OSError, UnicodeError, ValueError, RecursionError, MemoryError):
        raise SystemExit(1)
PY
fi

[[ "$MEDIA" == /* && -d "$MEDIA" && ! -L "$MEDIA" ]] || {
  echo "readme-schema-check: media root must be an absolute ordinary directory" >&2
  exit 2
}
for tool in grep jq sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-schema-check: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
if command -v python3 >/dev/null 2>&1; then
  JSONSCHEMA_PYTHON=python3
elif command -v python >/dev/null 2>&1; then
  JSONSCHEMA_PYTHON=python
else
  echo "readme-schema-check: Python 3.12 is required" >&2
  exit 1
fi
[[ "$($JSONSCHEMA_PYTHON --version 2>&1)" == "Python 3.12."* ]] || {
  echo "readme-schema-check: Python 3.12 is required" >&2
  exit 1
}
[[ "$($JSONSCHEMA_PYTHON -c 'from importlib.metadata import version; print(version("jsonschema"))')" == 4.26.0 ]] || {
  echo "readme-schema-check: jsonschema 4.26.0 is required" >&2
  exit 1
}

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

bash "$HUB/scripts/readme-schema-check.sh" --strict-json "$SCHEMA" "$SNAPSHOT"
for hostile in duplicate nan infinity negative-infinity overflow underflow trailing deep oversize; do
  case "$hostile" in
    duplicate) printf '{"status":"FAIL","status":"PASS"}\n' >"$tmp/$hostile.json" ;;
    nan) printf '{"number":NaN}\n' >"$tmp/$hostile.json" ;;
    infinity) printf '{"number":Infinity}\n' >"$tmp/$hostile.json" ;;
    negative-infinity) printf '{"number":-Infinity}\n' >"$tmp/$hostile.json" ;;
    overflow) printf '{"number":1e9999}\n' >"$tmp/$hostile.json" ;;
    underflow) printf '{"number":1e-9999}\n' >"$tmp/$hostile.json" ;;
    trailing) printf '{"status":"PASS"}\n{"status":"FAIL"}\n' >"$tmp/$hostile.json" ;;
    deep) "$JSONSCHEMA_PYTHON" -I -S -c 'print("[" * 2000 + "0" + "]" * 2000)' \
      >"$tmp/$hostile.json" ;;
    oversize) "$JSONSCHEMA_PYTHON" -I -S -c 'import sys; sys.stdout.write("0" * (4 * 1024 * 1024 + 1))' \
      >"$tmp/$hostile.json" ;;
  esac
  if bash "$HUB/scripts/readme-schema-check.sh" --strict-json "$tmp/$hostile.json" \
    >/dev/null 2>&1; then
    printf 'readme-schema-check: strict parser admitted %s JSON\n' "$hostile" >&2
    exit 1
  fi
done

snapshot_is_valid() {
  jq -e '
    (keys | sort) == (["classification","observed_at","release_authority","repositories",
      "schema_version","source_date_epoch","subject_committer_epochs"] | sort)
    and .schema_version == "bullet.readme-snapshot.v1"
    and (.observed_at | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"))
    and (.observed_at as $at | ($at | fromdateiso8601 | strftime("%Y-%m-%dT%H:%M:%SZ")) == $at)
    and (.subject_committer_epochs | keys | sort) == (.repositories | keys | sort)
    and all(.subject_committer_epochs[]; type == "number" and . > 0)
    and ((.observed_at | fromdateiso8601) >= ([.subject_committer_epochs[]] | max))
    and .source_date_epoch == 1787616000
    and .classification == "UNSIGNED_COMPONENT_OBSERVATION"
    and .release_authority == false
    and (.repositories | keys | sort == ["bullet-farm", "bullet-git", "bullet-kernel", "bullet-portal"])
    and all(.repositories[];
      (keys | sort) == ["commit_oid","tree_oid"]
      and (.commit_oid | test("^[0-9a-f]{40}$"))
      and (.tree_oid | test("^[0-9a-f]{40}$")))
  ' "$1" >/dev/null
}
snapshot_is_valid "$SNAPSHOT"
jq '.unexpected = "must-refuse"' "$SNAPSHOT" >"$tmp/snapshot-unknown.json"
if snapshot_is_valid "$tmp/snapshot-unknown.json"; then
  echo "readme-schema-check: snapshot admitted an unknown field" >&2
  exit 1
fi
jq '.observed_at = "2026-02-30T12:00:00Z"' "$SNAPSHOT" >"$tmp/snapshot-impossible-date.json"
if snapshot_is_valid "$tmp/snapshot-impossible-date.json"; then
  echo "readme-schema-check: snapshot admitted an impossible calendar date" >&2
  exit 1
fi

jq -e '
  ."$schema" == "https://json-schema.org/draft/2020-12/schema" and
  .type == "object" and .additionalProperties == false and
  (.required | sort) == (["classification","demo_id","document_type","live_provider_spawned",
    "network","observed_at","release_authority","schema_version"] | sort) and
  .properties.schema_version.const == "bullet.readme-demo.v1" and
  .properties.subjects.additionalProperties == false and
  (.properties.subjects.required | sort) == (["bullet-farm","bullet-git","bullet-kernel","bullet-portal"] | sort) and
  .properties.renderer.additionalProperties == false and
  .properties.generation.additionalProperties == false and
  .properties.generation.properties.command.const == "bash scripts/readme-render.sh" and
  .properties.generation.properties.inputs.minItems == 8 and
  .properties.generation.properties.inputs.maxItems == 8 and
  .properties.generation.properties.inputs.uniqueItems == true and
  .properties.artifact_hashes.minItems == 1 and
  .properties.artifact_hashes.uniqueItems == true and
  .properties.artifact_hashes.items.additionalProperties == false and
  .properties.artifact_hashes.items.properties.path.pattern == "^(?!\\.\\.?$)[^/\\\\]+$"
' "$SCHEMA" >/dev/null
artifact_pattern="$(jq -r '.properties.artifact_hashes.items.properties.path.pattern' "$SCHEMA")"
printf 'artifact.gif\n' | grep -Pqx "$artifact_pattern"
for invalid in 'nested/artifact.gif' 'nested\artifact.gif' '..'; do
  if printf '%s\n' "$invalid" | grep -Pqx "$artifact_pattern"; then
    printf 'readme-schema-check: artifact pattern admitted %s\n' "$invalid" >&2
    exit 1
  fi
done

schema_document_is_valid() {
  bash "$HUB/scripts/readme-schema-check.sh" --strict-json "$SCHEMA" "$1" || return 1
  "$JSONSCHEMA_PYTHON" - "$SCHEMA" "$1" <<'PY'
import json
import sys
from datetime import datetime
from jsonschema import Draft202012Validator, FormatChecker

with open(sys.argv[1], encoding="utf-8") as handle:
    schema = json.load(handle)
with open(sys.argv[2], encoding="utf-8") as handle:
    instance = json.load(handle)
format_checker = FormatChecker()

@format_checker.checks("date-time", raises=(TypeError, ValueError))
def canonical_utc_date_time(value):
    if not isinstance(value, str):
        return False
    parsed = datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    return parsed.strftime("%Y-%m-%dT%H:%M:%SZ") == value

Draft202012Validator.check_schema(schema)
Draft202012Validator(schema, format_checker=format_checker).validate(instance)
PY
}

document_is_valid() {
  schema_document_is_valid "$1" || return 1
  jq -e '.observed_at as $at |
    ($at | fromdateiso8601 | strftime("%Y-%m-%dT%H:%M:%SZ")) == $at' "$1" >/dev/null
}

expect_schema_rejection() {
  local document="$1" label="$2"
  if document_is_valid "$document" >/dev/null 2>&1; then
    printf 'readme-schema-check: contract admitted hostile instance: %s\n' "$label" >&2
    exit 1
  fi
}

snapshot_sha="$(sha256sum "$SNAPSHOT" | awk '{print $1}')"
playback_sha="$(sha256sum "$MEDIA/playback.sh" | awk '{print $1}')"
schema_sha="$(sha256sum "$SCHEMA" | awk '{print $1}')"
checker_sha="$(sha256sum "$HUB/scripts/readme-check.sh" | awk '{print $1}')"
input_checker_sha="$(sha256sum "$HUB/scripts/readme-input-check.sh" | awk '{print $1}')"
recorder_sha="$(sha256sum "$HUB/scripts/readme-record.sh" | awk '{print $1}')"
renderer_sha="$(sha256sum "$HUB/scripts/readme-render.sh" | awk '{print $1}')"
schema_checker_sha="$(sha256sum "$HUB/scripts/readme-schema-check.sh" | awk '{print $1}')"
observed_at="$(jq -er '.observed_at' "$SNAPSHOT")"

for demo in component-preview provider-safety; do
  observation="$MEDIA/$demo/observation.json"
  manifest="$MEDIA/$demo/manifest.json"
  document_is_valid "$observation"
  document_is_valid "$manifest"
  jq -e --arg demo "$demo" --arg observed_at "$observed_at" --slurpfile snapshot "$SNAPSHOT" '
    .schema_version == "bullet.readme-demo.v1" and .document_type == "observation" and
    .demo_id == $demo and .observed_at == $observed_at and
    .classification == "UNSIGNED_COMPONENT_OBSERVATION" and .release_authority == false and
    .live_provider_spawned == false and .network == "isolated" and
    .subjects == $snapshot[0].repositories and
    (keys | sort) == (["classification","demo_id","document_type","live_provider_spawned",
      "network","observed_at","outcomes","release_authority","schema_version","subjects"] | sort) and
    all(.subjects[]; (keys | sort) == ["commit_oid","tree_oid"]) and
    all(.outcomes[]; (keys | sort) == ["exit_code","name","status"])
  ' "$observation" >/dev/null
  jq -e --arg demo "$demo" --arg observed_at "$observed_at" --arg image "$VHS_IMAGE" \
    --arg snapshot_sha "$snapshot_sha" --arg playback_sha "$playback_sha" \
    --arg schema_sha "$schema_sha" --arg checker_sha "$checker_sha" \
    --arg input_checker_sha "$input_checker_sha" \
    --arg recorder_sha "$recorder_sha" \
    --arg renderer_sha "$renderer_sha" --arg schema_checker_sha "$schema_checker_sha" \
    --argjson source_epoch "$SOURCE_EPOCH" '
    .schema_version == "bullet.readme-demo.v1" and .document_type == "manifest" and
    .demo_id == $demo and .observed_at == $observed_at and
    .classification == "UNSIGNED_COMPONENT_OBSERVATION" and .release_authority == false and
    .live_provider_spawned == false and .network == "disabled-during-render" and
    .renderer == {name:"VHS",version:"0.11.0",image:$image,ffmpeg_version:"7.1.3-0+deb13u1",
      canonicalization:"committed-transcript-drawtext",width:1200,height:675,frames_per_second:12,
      maximum_duration_seconds:30,maximum_gif_bytes:3145728,font:"JetBrains Mono",locale:"C.UTF-8",
      timezone:"UTC",source_date_epoch:$source_epoch,cursor_blink:false} and
    .generation == {name:"bullet-farm-readme-render",version:1,command:"bash scripts/readme-render.sh",inputs:[
      {path:"docs/readme-media/playback.sh",sha256:$playback_sha},
      {path:"docs/readme-media/snapshot.json",sha256:$snapshot_sha},
      {path:"docs/schemas/bullet.readme-demo.v1.schema.json",sha256:$schema_sha},
      {path:"scripts/readme-check.sh",sha256:$checker_sha},
      {path:"scripts/readme-input-check.sh",sha256:$input_checker_sha},
      {path:"scripts/readme-record.sh",sha256:$recorder_sha},
      {path:"scripts/readme-render.sh",sha256:$renderer_sha},
      {path:"scripts/readme-schema-check.sh",sha256:$schema_checker_sha}]} and
    ([.artifact_hashes[].path] | sort) ==
      ([($demo + ".gif"),($demo + ".tape"),"fallback.png","frames.framemd5","observation.json","transcript.txt"] | sort) and
    ([.artifact_hashes[].path] | unique | length) == 6 and
    (keys | sort) == (["artifact_hashes","classification","demo_id","document_type","generation",
      "live_provider_spawned","network","observed_at","release_authority","renderer","schema_version"] | sort)
  ' "$manifest" >/dev/null
done

jq -e '.outcomes == [
  {name:"doctor",status:"BLOCKED",exit_code:3},
  {name:"hub-component-checks",status:"PASS",exit_code:0},
  {name:"materialize-replay",status:"IDEMPOTENT",exit_code:0},
  {name:"fence",status:"1 -> 2",exit_code:0},
  {name:"stale-authority",status:"REFUSED",exit_code:0},
  {name:"lost-response-effect",status:"UNKNOWN",exit_code:0}
]' "$MEDIA/component-preview/observation.json" >/dev/null
jq -e '.outcomes == [
  {name:"claude-offline-contract",status:"PASS",exit_code:0},
  {name:"codex-offline-contract",status:"PASS",exit_code:0},
  {name:"cursor-offline-contract",status:"PASS",exit_code:0},
  {name:"antigravity-offline-contract",status:"PASS",exit_code:0},
  {name:"claude-live-admission",status:"POLICY_LIVE_ADMISSION_DISABLED",exit_code:78},
  {name:"codex-live-admission",status:"POLICY_LIVE_ADMISSION_DISABLED",exit_code:78},
  {name:"cursor-live-admission",status:"POLICY_LIVE_ADMISSION_DISABLED",exit_code:78},
  {name:"antigravity-live-admission",status:"POLICY_LIVE_ADMISSION_DISABLED",exit_code:78},
  {name:"provider-spawn-count",status:"0",exit_code:0},
  {name:"live-provider-proof",status:"ABSENT",exit_code:0}
]' "$MEDIA/provider-safety/observation.json" >/dev/null

component_manifest="$MEDIA/component-preview/manifest.json"
component_observation="$MEDIA/component-preview/observation.json"
jq '.generation.unexpected="must-refuse"' "$component_manifest" >"$tmp/unknown-generation.json"
expect_schema_rejection "$tmp/unknown-generation.json" unknown-generation-field
jq '.generation.inputs[1].path=.generation.inputs[0].path' "$component_manifest" >"$tmp/duplicate-input.json"
expect_schema_rejection "$tmp/duplicate-input.json" duplicate-generation-path
jq '.artifact_hashes[1].path=.artifact_hashes[0].path' "$component_manifest" >"$tmp/duplicate-artifact.json"
expect_schema_rejection "$tmp/duplicate-artifact.json" duplicate-artifact-path
jq '.artifact_hashes[0].path=".."' "$component_manifest" >"$tmp/dot-artifact.json"
expect_schema_rejection "$tmp/dot-artifact.json" dot-artifact-path
jq '.network="isolated"' "$component_manifest" >"$tmp/manifest-network.json"
expect_schema_rejection "$tmp/manifest-network.json" swapped-manifest-network
jq '.subjects={}' "$component_manifest" >"$tmp/manifest-cross-field.json"
expect_schema_rejection "$tmp/manifest-cross-field.json" manifest-observation-field
jq '.network="disabled-during-render"' "$component_observation" >"$tmp/observation-network.json"
expect_schema_rejection "$tmp/observation-network.json" swapped-observation-network
jq '.renderer={}' "$component_observation" >"$tmp/observation-cross-field.json"
expect_schema_rejection "$tmp/observation-cross-field.json" observation-manifest-field
jq '.outcomes[1].name=.outcomes[0].name' "$component_observation" >"$tmp/duplicate-outcome.json"
expect_schema_rejection "$tmp/duplicate-outcome.json" duplicate-outcome-name
jq '.outcomes[0]={name:"doctor",status:"PASS",exit_code:0}' "$component_observation" >"$tmp/wrong-outcome.json"
expect_schema_rejection "$tmp/wrong-outcome.json" wrong-outcome-semantics
jq '.observed_at="2026-02-30T12:00:00Z"' "$component_observation" >"$tmp/impossible-date.json"
if schema_document_is_valid "$tmp/impossible-date.json" >/dev/null 2>&1; then
  echo "readme-schema-check: registered date-time checker admitted an impossible calendar date" >&2
  exit 1
fi
expect_schema_rejection "$tmp/impossible-date.json" impossible-calendar-date
sed '0,/"release_authority": false/{s//"release_authority": true, "release_authority": false/}' \
  "$component_observation" >"$tmp/duplicate-member.json"
expect_schema_rejection "$tmp/duplicate-member.json" duplicate-json-member
sed '0,/"source_date_epoch": 1787616000/{s//"source_date_epoch": 1e9999/}' \
  "$component_manifest" >"$tmp/nonfinite-number.json"
expect_schema_rejection "$tmp/nonfinite-number.json" exponent-overflow
{
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '%s\n' "$line"
  done <"$component_observation"
  printf '%s\n' '{}'
} >"$tmp/trailing-document.json"
expect_schema_rejection "$tmp/trailing-document.json" trailing-document

echo "readme-schema-check: PASS (Draft 2020-12 plus semantic hostiles)"
