#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
grep -qx 'schema_version = "2"' family.lock \
  || { refuse SETUP_REFUSAL_SUBJECT_DRIFT "expected diagnostic schema-2 family.lock"; exit 1; }

schema_output="$(mktemp)"
shim_root="$(mktemp -d)"
cleanup() { rm -f -- "$schema_output" "$shim_root/cargo" "$shim_root/marker"; rmdir "$shim_root" 2>/dev/null || true; }
trap cleanup EXIT
status=0
cargo run --locked --quiet --bin bullet-family -- setup \
  --root "${REPO_ROOT%/*}" --source jeryu --offline >"$schema_output" 2>&1 || status=$?
[[ "$status" -ne 0 ]] || { refuse LEGACY_SETUP_AUTHORIZED "schema 2 returned success"; exit 1; }
grep -q 'UNSUPPORTED_SCHEMA' "$schema_output" \
  || { refuse LEGACY_SETUP_REASON_DRIFT "UNSUPPORTED_SCHEMA absent"; exit 1; }

dollar='$'
printf '%s\n' '#!/bin/sh' ": > \"${dollar}{BULLET_SETUP_CARGO_MARKER:?}\"" 'exit 99' >"$shim_root/cargo"
chmod 700 "$shim_root/cargo"
status=0
(cd /tmp && PATH="$shim_root:$PATH" BULLET_SETUP_CARGO_MARKER="$shim_root/marker" \
  "$REPO_ROOT/scripts/setup.sh" --offline) >"$schema_output" 2>&1 || status=$?
[[ "$status" -ne 0 ]] || { refuse SOURCE_WRAPPER_UNEXPECTED_SUCCESS "bootstrap refusal returned zero"; exit 1; }
grep -q 'operator-pre-admitted bootstrap unavailable' "$schema_output" \
  || { refuse SOURCE_WRAPPER_REASON_DRIFT "bootstrap refusal reason absent"; exit 1; }
[[ ! -e "$shim_root/marker" ]] \
  || { refuse AMBIENT_CARGO_EXECUTED "source wrapper ran the PATH shim"; exit 1; }
log "schema-2 and no-ambient-Cargo setup refusals passed"
