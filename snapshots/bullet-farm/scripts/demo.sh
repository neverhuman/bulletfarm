#!/usr/bin/env bash
# Drive the offline five-plane fixture saga and print its self-signed component receipt.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
FAMILY="$(cd "$HUB/.." && pwd -P)"
KERNEL="$FAMILY/bullet-kernel"
GIT="$FAMILY/bullet-git"
PORTAL="$FAMILY/bullet-portal"
data_ancestors_valid() {
  local ancestor="$1" identity owner mode
  [[ "$ancestor" == /* && "$ancestor" != / && -d "$ancestor" && ! -L "$ancestor" \
    && "$(realpath -e -- "$ancestor")" == "$ancestor" ]] || return 1
  while :; do
    [[ -d "$ancestor" && ! -L "$ancestor" ]] || return 1
    identity="$(stat -c '%u:%a' -- "$ancestor")" || return 1
    owner="${identity%%:*}"; mode="${identity#*:}"
    [[ "$owner" == 0 || "$owner" == "$(id -u)" ]] || return 1
    [[ "$mode" =~ ^[0-7]{3,4}$ ]] || return 1
    if (( (8#$mode & 8#22) != 0 )); then
      [[ "$owner" == 0 ]] && (( (8#$mode & 8#1000) != 0 )) || return 1
    fi
    [[ "$ancestor" != / ]] || break
    ancestor="${ancestor%/*}"
    [[ -n "$ancestor" ]] || ancestor=/
  done
}

if [[ -n "${BULLET_DATA_DIR:-}" ]]; then
  DATA="$BULLET_DATA_DIR"
else
  DATA="$(mktemp -d /tmp/bullet-txn.XXXXXX)"
fi
# This preflight avoids an expensive build for a path SQLite will refuse. The
# ledger independently revalidates custody before it opens or creates any data.
if ! data_ancestors_valid "$DATA" \
  || [[ "$(stat -c '%u:%a' -- "$DATA")" != "$(id -u):700" ]]; then
  echo "DEMO_DATA_INVALID: use a canonical caller-owned 0700 directory; ancestors must be root/caller-owned without group/other write, except root-owned sticky directories" >&2
  exit 1
fi

echo "== Bullet Farm demo =="
echo "evidence_class: COMPONENT_PROOF"
echo "verifier_fixture_trust: UNSIGNED_FIXTURE"
echo "independent_verification_eligible: false"
echo "release_gate_eligible: false"
echo "transaction_proof: absent"
echo "kernel: $KERNEL"
echo "data:   $DATA"

if [[ ! -f "$KERNEL/Cargo.toml" ]]; then
  echo "bullet-kernel checkout missing at $KERNEL" >&2
  exit 1
fi
if [[ ! -f "$GIT/Cargo.toml" ]]; then
  echo "bullet-git checkout missing at $GIT" >&2
  exit 1
fi

if [[ ${CARGO_TARGET_DIR+x} ]]; then
  DEMO_TARGET="$CARGO_TARGET_DIR"
else
  DEMO_TARGET="$(mktemp -d /tmp/bullet-demo-target.XXXXXX)"
fi
if [[ "$DEMO_TARGET" != /* || "$DEMO_TARGET" == / || ! -d "$DEMO_TARGET" \
  || -L "$DEMO_TARGET" || "$(realpath -e -- "$DEMO_TARGET")" != "$DEMO_TARGET" \
  || "$(stat -c '%u:%a' -- "$DEMO_TARGET")" != "$(id -u):700" \
  || "$DEMO_TARGET/" == "$FAMILY/"* ]]; then
  echo "DEMO_TARGET_INVALID: use an existing canonical caller-owned 0700 target outside the family" >&2
  exit 1
fi
export CARGO_TARGET_DIR="$DEMO_TARGET"
unset BULLET_VERIFIER_FIXTURE_FD BULLET_VERIFIER_FIXTURE_BIN
umask 077
BUILD_REPORTS="$(mktemp -d "$DEMO_TARGET/demo-build.XXXXXX")"
echo "build target: $DEMO_TARGET"
echo "retained build reports: $BUILD_REPORTS"
declare -A BUILT_PATHS BUILT_SHA256 BUILT_IDENTITIES

executable_identity() {
  stat -c '%d:%i:%u:%g:%f:%h:%s:%y:%z' -- "$1"
}

verify_built_binaries() {
  local binary artifact before after digest
  for binary in bullet-gitd bullet-gitd-fixture bullet-farmd transaction_demo bullet-verifier-fixture; do
    artifact="${BUILT_PATHS[$binary]}"
    if [[ -f "$artifact" && -x "$artifact" && ! -L "$artifact" \
      && "$(realpath -e -- "$artifact")" == "$artifact" ]] \
      && before="$(executable_identity "$artifact")" \
      && digest="$(sha256sum -- "$artifact")" \
      && after="$(executable_identity "$artifact")" \
      && [[ "$before" == "${BUILT_IDENTITIES[$binary]}" && "$after" == "$before" \
        && "${digest%% *}" == "${BUILT_SHA256[$binary]}" ]]; then
      continue
    fi
    echo "DEMO_BUILD_SUBJECT_CHANGED: $binary; raw output retained in $BUILD_REPORTS" >&2
    return 1
  done
}

build_binary() {
  local repo="$1" package="$2" binary="$3" feature="$4" destination="$5"
  local artifact digest identity status
  local stdout="$BUILD_REPORTS/$binary.jsonl" stderr="$BUILD_REPORTS/$binary.stderr"
  local -a args=(build --locked -q -p "$package" --bin "$binary" --message-format=json-render-diagnostics)
  [[ -z "$feature" ]] || args+=(--features "$feature")
  if (cd "$repo" && cargo "${args[@]}") >"$stdout" 2>"$stderr"; then
    cat -- "$stderr" >&2
  else
    status="$?"
    cat -- "$stderr" >&2
    echo "DEMO_BUILD_FAILED: $binary; raw output retained in $BUILD_REPORTS" >&2
    return "$status"
  fi
  if ! artifact="$(jq -ers --arg binary "$binary" --arg repo "$repo/" --arg feature "$feature" '
    if length == 0 or any(.[]; type != "object") then error("invalid Cargo stream") else . end
    | if ([.[] | select(.reason == "build-finished")] | length) != 1
        or .[-1] != {reason:"build-finished",success:true} then error("build not completed") else . end
    | [.[] | select(.reason == "compiler-artifact" and .target.name == $binary
        and .target.kind == ["bin"] and .profile.test == false)]
    | if length != 1 then error("expected one binary artifact") else .[0] end
    | if (.target.src_path | type) != "string" or (.target.src_path | startswith($repo) | not)
        or (.features | type) != "array"
        or ($feature != "" and (.features | index($feature)) == null)
        or ($feature == "" and any(.features[]; . == "fixture-authority" or . == "fixture-executor"))
        then error("artifact subject differs") else . end
    | .executable
    | if type != "string" or length == 0 or (explode | any(. < 32 or . == 127))
        then error("invalid executable path") else . end
  ' "$stdout")"; then
    echo "DEMO_BUILD_ARTIFACT_INVALID: $binary; raw output retained in $BUILD_REPORTS" >&2
    return 1
  fi
  if [[ "$artifact" != "$DEMO_TARGET/"* || ! -f "$artifact" || ! -x "$artifact" \
    || -L "$artifact" || "$(realpath -e -- "$artifact")" != "$artifact" \
    || "$(stat -c '%u' -- "$artifact")" != "$(id -u)" ]]; then
    echo "DEMO_BUILD_ARTIFACT_INVALID: $binary executable is outside its admitted target or substituted" >&2
    return 1
  fi
  local mode
  mode="$(stat -c '%a' -- "$artifact")"
  if (( (8#$mode & 8#22) != 0 )); then
    echo "DEMO_BUILD_ARTIFACT_INVALID: $binary executable is writable by another principal" >&2
    return 1
  fi
  identity="$(executable_identity "$artifact")"
  digest="$(sha256sum -- "$artifact")"
  digest="${digest%% *}"
  if [[ "$(executable_identity "$artifact")" != "$identity" ]]; then
    echo "DEMO_BUILD_SUBJECT_CHANGED: $binary changed during build observation" >&2
    return 1
  fi
  BUILT_PATHS[$binary]="$artifact"
  BUILT_SHA256[$binary]="$digest"
  BUILT_IDENTITIES[$binary]="$identity"
  jq -cn --arg repository "$repo" --arg package "$package" --arg binary "$binary" \
    --arg feature "$feature" --arg executable "$artifact" --arg sha256 "$digest" --arg identity "$identity" \
    '{repository:$repository,package:$package,binary:$binary,feature:$feature,executable:$executable,sha256:$sha256,identity:$identity}' \
    >>"$BUILD_REPORTS/binary-subjects.jsonl"
  printf -v "$destination" '%s' "$artifact"
}

build_binary "$GIT" bullet-gitd bullet-gitd "" BULLET_GITD_BIN
build_binary "$GIT" bullet-gitd bullet-gitd-fixture fixture-authority BULLET_GITD_FIXTURE_BIN
build_binary "$KERNEL" bullet-farmd bullet-farmd "" BULLET_FARMD_BIN
build_binary "$KERNEL" bullet transaction_demo "" TRANSACTION_DEMO_BIN
build_binary "$KERNEL" bullet-verifier bullet-verifier-fixture fixture-executor VERIFIER_FIXTURE_BUILD_BIN
VERIFIER_FIXTURE_STAGE="$(mktemp -d /tmp/bullet-verifier-fixture.XXXXXX)"
if [[ "$VERIFIER_FIXTURE_STAGE" != /* || "$VERIFIER_FIXTURE_STAGE" == "/" || ! -d "$VERIFIER_FIXTURE_STAGE" || -L "$VERIFIER_FIXTURE_STAGE" ]]; then
  echo "verifier fixture stage must be an absolute real directory" >&2
  exit 1
fi
CANONICAL_VERIFIER_FIXTURE_STAGE="$(realpath -e -- "$VERIFIER_FIXTURE_STAGE")"
if [[ "$CANONICAL_VERIFIER_FIXTURE_STAGE" != "$VERIFIER_FIXTURE_STAGE" ]]; then
  echo "verifier fixture stage must be normalized and contain no symlinked ancestor" >&2
  exit 1
fi
if [[ "$(stat -c '%u:%a' -- "$VERIFIER_FIXTURE_STAGE")" != "$(id -u):700" ]]; then
  echo "verifier fixture stage must be caller-owned with exact mode 0700" >&2
  exit 1
fi
BULLET_VERIFIER_FIXTURE_BIN="$VERIFIER_FIXTURE_STAGE/bullet-verifier-fixture"
cp --reflink=never -- "$VERIFIER_FIXTURE_BUILD_BIN" "$BULLET_VERIFIER_FIXTURE_BIN"
CANONICAL_VERIFIER_FIXTURE_BIN="$(realpath -e -- "$BULLET_VERIFIER_FIXTURE_BIN")"
if [[ "$CANONICAL_VERIFIER_FIXTURE_BIN" != "$BULLET_VERIFIER_FIXTURE_BIN" || -L "$BULLET_VERIFIER_FIXTURE_BIN" || ! -f "$BULLET_VERIFIER_FIXTURE_BIN" || ! -x "$BULLET_VERIFIER_FIXTURE_BIN" ]]; then
  echo "staged verifier fixture must be a canonical regular executable" >&2
  exit 1
fi
if [[ "$(stat -c '%u:%h' -- "$BULLET_VERIFIER_FIXTURE_BIN")" != "$(id -u):1" ]]; then
  echo "staged verifier fixture must be caller-owned and single-link" >&2
  exit 1
fi
if ! cmp -s -- "$VERIFIER_FIXTURE_BUILD_BIN" "$BULLET_VERIFIER_FIXTURE_BIN"; then
  echo "staged verifier fixture differs from the built fixture" >&2
  exit 1
fi

export BULLET_GITD_BIN BULLET_GITD_FIXTURE_BIN
BULLET_GITD_SHA256="${BUILT_SHA256[bullet-gitd]}"
export BULLET_GITD_SHA256
BULLET_GITD_FIXTURE_SHA256="${BUILT_SHA256[bullet-gitd-fixture]}"
export BULLET_GITD_FIXTURE_SHA256
export BULLET_FARMD_BIN
BULLET_VERIFIER_FIXTURE_SHA256="$(sha256sum -- "$BULLET_VERIFIER_FIXTURE_BIN")"
BULLET_VERIFIER_FIXTURE_SHA256="${BULLET_VERIFIER_FIXTURE_SHA256%% *}"
if [[ "$BULLET_VERIFIER_FIXTURE_SHA256" != "${BUILT_SHA256[bullet-verifier-fixture]}" ]]; then
  echo "DEMO_BUILD_SUBJECT_CHANGED: staged verifier differs from its captured build subject" >&2
  exit 1
fi
export BULLET_VERIFIER_FIXTURE_SHA256

verify_built_binaries
exec {BULLET_VERIFIER_FIXTURE_FD}<"$BULLET_VERIFIER_FIXTURE_BIN"
export BULLET_VERIFIER_FIXTURE_FD
DEMO_STATUS=0
(cd "$KERNEL" && BULLET_DATA_DIR="$DATA" "$TRANSACTION_DEMO_BIN") \
  >"$BUILD_REPORTS/transaction_demo.stdout" 2>"$BUILD_REPORTS/transaction_demo.stderr" || DEMO_STATUS="$?"
exec {BULLET_VERIFIER_FIXTURE_FD}<&-
unset BULLET_VERIFIER_FIXTURE_FD
cat -- "$BUILD_REPORTS/transaction_demo.stdout"
cat -- "$BUILD_REPORTS/transaction_demo.stderr" >&2
if (( DEMO_STATUS != 0 )); then
  echo "DEMO_COMPONENT_FAILED: raw output retained in $BUILD_REPORTS" >&2
  exit "$DEMO_STATUS"
fi
verify_built_binaries

if [[ "${BULLET_DEMO_PORTAL:-0}" == "1" ]]; then
  echo "== portal smoke =="
  (cd "$PORTAL" && npm test --silent)
fi

echo "== component demo complete (TRANSACTION_PROOF remains absent) =="
