#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/toolchain-pins.sh
source "$(dirname "${BASH_SOURCE[0]}")/toolchain-pins.sh"
cd "$REPO_ROOT"

test_root="$(mktemp -d)"
cleanup() { rm -rf -- "$test_root"; }
trap cleanup EXIT
fake_bin="$test_root/bin"
mkdir -p "$fake_bin"
dev_log="$test_root/dev.log"

printf '%s\n' '#!/bin/sh' 'sleep 0.5' 'exit 0' >"$fake_bin/cargo"
npm_marker="$test_root/npm-installed"
# The generated child script must receive these parameter expansions literally.
# shellcheck disable=SC2016
printf '#!/bin/sh\nif [ "${1:-}" = "--version" ]; then echo %s; exit 0; fi\nprintf "npm:%%s\\n" "$*" >>"%s"\nif [ ! -e "%s" ]; then : >"%s"; exit 0; fi\nsleep 30\n' \
  "$PINNED_NPM_VERSION" "$dev_log" "$npm_marker" "$npm_marker" >"$fake_bin/npm"
# The generated child script must receive these parameter expansions literally.
# shellcheck disable=SC2016
printf '#!/bin/sh\nif [ "${1:-}" = "--version" ]; then echo v%s; exit 0; fi\nprintf "node:%%s\\n" "$*" >>"%s"\n' \
  "$PINNED_NODE_VERSION" "$dev_log" >"$fake_bin/node"
curl_log="$test_root/curl.log"
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >>"%s"\nexit 0\n' "$curl_log" >"$fake_bin/curl"
chmod 0700 "$fake_bin/cargo" "$fake_bin/node" "$fake_bin/npm" "$fake_bin/curl"

set +e
output="$(PATH="$fake_bin:$PATH" bash scripts/dev.sh 2>&1)"
status=$?
set -e
[[ "$status" -eq 1 && "$output" == *'unexpected supervised child exit is a failure'* ]] \
  || { refuse DEV_SUPERVISOR_ZERO_EXIT_ACCEPTED "status=$status output=$output"; exit 1; }
grep -Fq 'http://127.0.0.1:7420/health' "$curl_log" \
  || { refuse DEV_FARMD_READINESS_MISSING "$curl_log"; exit 1; }
grep -Fq 'http://127.0.0.1:5173/' "$curl_log" \
  || { refuse DEV_PORTAL_READINESS_MISSING "$curl_log"; exit 1; }
mapfile -t install_order <"$dev_log"
[[ "${install_order[0]:-}" == 'node:ops/ci/preinstall-scan.mjs' ]] \
  || { refuse DEV_PREINSTALL_SCAN_MISSING "${install_order[*]:-empty}"; exit 1; }
[[ "${install_order[1]:-}" == 'npm:ci --ignore-scripts --no-audit --no-fund' ]] \
  || { refuse DEV_PREINSTALL_ORDER_INVALID "${install_order[*]:-empty}"; exit 1; }
log "dev supervisor rejects an unexpected zero child exit and cleans its peer"

: >"$dev_log"
# The generated child script must receive its first argument expansion literally.
# shellcheck disable=SC2016
printf '%s\n' '#!/bin/sh' 'if [ "${1:-}" = "--version" ]; then echo v99.0.0; exit 0; fi' \
  'printf "node:%s\\n" "$*" >>"'"$dev_log"'"' >"$fake_bin/node"
chmod 0700 "$fake_bin/node"
set +e
output="$(PATH="$fake_bin:$PATH" bash scripts/dev.sh 2>&1)"
status=$?
set -e
[[ "$status" -eq 1 && "$output" == *"expected Node v$PINNED_NODE_VERSION, found v99.0.0"* ]] \
  || { refuse DEV_NODE_VERSION_ACCEPTED "status=$status output=$output"; exit 1; }
[[ ! -s "$dev_log" ]] \
  || { refuse DEV_NODE_VERSION_SIDE_EFFECT "$dev_log"; exit 1; }

# The generated child script must receive these parameter expansions literally.
# shellcheck disable=SC2016
printf '#!/bin/sh\nif [ "${1:-}" = "--version" ]; then echo v%s; exit 0; fi\nprintf "node:%%s\\n" "$*" >>"%s"\n' \
  "$PINNED_NODE_VERSION" "$dev_log" >"$fake_bin/node"
# The generated child script must receive its first argument expansion literally.
# shellcheck disable=SC2016
printf '%s\n' '#!/bin/sh' 'if [ "${1:-}" = "--version" ]; then echo 99.0.0; exit 0; fi' \
  'printf "npm:%s\\n" "$*" >>"'"$dev_log"'"' >"$fake_bin/npm"
chmod 0700 "$fake_bin/node" "$fake_bin/npm"
set +e
output="$(PATH="$fake_bin:$PATH" bash scripts/dev.sh 2>&1)"
status=$?
set -e
[[ "$status" -eq 1 && "$output" == *"expected npm $PINNED_NPM_VERSION, found 99.0.0"* ]] \
  || { refuse DEV_NPM_VERSION_ACCEPTED "status=$status output=$output"; exit 1; }
[[ ! -s "$dev_log" ]] \
  || { refuse DEV_NPM_VERSION_SIDE_EFFECT "$dev_log"; exit 1; }
log "dev source admission pins Node/npm and precedes dependency installation"

pin_fixture="$test_root/pin-fixture"
mkdir -p "$pin_fixture/ops/ci"
cp ops/ci/toolchain-pins.sh "$pin_fixture/ops/ci/toolchain-pins.sh"
assert_pin_refused() {
  local case_name="$1" output status
  set +e
  output="$(bash -c 'source "$1"' _ "$pin_fixture/ops/ci/toolchain-pins.sh" 2>&1)"
  status=$?
  set -e
  [[ "$status" -ne 0 && "$output" == *'TOOLCHAIN_PIN_INVALID:'* ]] \
    || { refuse DEV_TOOLCHAIN_PIN_ACCEPTED "$case_name status=$status output=$output"; exit 1; }
}
printf '%s\n' "$PINNED_NODE_VERSION" >"$pin_fixture/.node-version"
printf '%s\n' "$PINNED_NPM_VERSION" >"$pin_fixture/.npm-version"
bash -c 'source "$1"' _ "$pin_fixture/ops/ci/toolchain-pins.sh"
printf '%s' "$PINNED_NODE_VERSION" >"$pin_fixture/.node-version"
assert_pin_refused missing-lf
printf '%s\r\n' "$PINNED_NODE_VERSION" >"$pin_fixture/.node-version"
assert_pin_refused crlf
printf '%s\nextra\n' "$PINNED_NODE_VERSION" >"$pin_fixture/.node-version"
assert_pin_refused multiple-lines
printf '%s\n' "$PINNED_NODE_VERSION" >"$pin_fixture/node-target"
ln -sf node-target "$pin_fixture/.node-version"
assert_pin_refused symlink
log "toolchain pin reader rejects malformed, multiline, CRLF, and symlink subjects"

portal_args="$test_root/portal.args"
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >"%s"\n' "$portal_args" >"$fake_bin/npm"
chmod 0700 "$fake_bin/npm"
PATH="$fake_bin:$PATH" bash scripts/portal.sh
[[ "$(<"$portal_args")" == 'run dev -- --host 127.0.0.1 --port 5173 --strictPort' ]] \
  || { refuse DEV_PORTAL_STRICT_PORT_MISSING "$(<"$portal_args")"; exit 1; }

farmd_args="$test_root/farmd.args"
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >"%s"\n' "$farmd_args" >"$fake_bin/cargo"
chmod 0700 "$fake_bin/cargo"
PATH="$fake_bin:$PATH" BULLET_DATA_DIR="$test_root/data" bash scripts/farmd.sh
[[ "$(<"$farmd_args")" == 'run --locked -p bullet-farmd -- --data-dir '*'/data --bind 127.0.0.1:7420 --portal-origin http://127.0.0.1:5173' ]] \
  || { refuse DEV_FARMD_LOCKFILE_MISSING "$(<"$farmd_args")"; exit 1; }
log "dev launchers pin the lockfile and refuse Vite port fallback"
