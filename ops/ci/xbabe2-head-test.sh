#!/usr/bin/env bash
# Component refusal tests only: no terminal, provider or credentialed execution.
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
umask 077
fixture="$(mktemp -d)"
cleanup() {
  local status=$?
  if (( status == 0 )); then rm -rf -- "$fixture";
  else printf '[ci] retained failed Head-refusal fixture: %s\n' "$fixture" >&2; fi
}
trap cleanup EXIT
hub="$fixture/family/bullet-farm"
mkdir -p "$hub/scripts" "$hub/ops/ci" "$fixture/bin" "$fixture/state/receipts"
cp "$repo/scripts/xbabe2-head.sh" "$hub/scripts/"
cp "$repo/ops/ci/lib.sh" "$repo/ops/ci/artifact-path.sh" \
  "$repo/ops/ci/rust-toolchain-boundary.sh" "$hub/ops/ci/"
cat >"$fixture/bin/hostname" <<'HOST'
#!/usr/bin/env bash
printf '%s\n' "${FIXTURE_HOST:-xbabe2}"
HOST
chmod 700 "$fixture/bin/hostname"
# These are the old cache-key inputs for absent member checkouts. No candidate
# or transaction is created; the deliberately fabricated file must never pass.
key="$(printf 'missing\nmissing\nhead-doors\n27\nnone' | sha256sum)"
key="${key%% *}"
receipt="$fixture/state/receipts/$key"
run_case() {
  local name="$1" expected="$2" status=0
  shift 2
  env PATH="$fixture/bin:$PATH" CI= GITHUB_ACTIONS= \
    JANKURAI_CHECKOUT="$fixture/absent-jankurai" \
    BULLET_XBABE2_HEAD_STATE="$fixture/state" "$@" \
    bash "$hub/scripts/xbabe2-head.sh" >"$fixture/$name.log" 2>&1 || status=$?
  [[ "$status" == 78 ]] || {
    printf '[ci] HEAD_REFUSAL_REGRESSION: %s exit=%s\n' "$name" "$status" >&2
    cat "$fixture/$name.log" >&2
    return 1
  }
  grep -Fq "$expected" "$fixture/$name.log"
  ! grep -Eq 'skip registry hit|: PASS' "$fixture/$name.log"
}
run_case absent-receipt HEAD_RUNTIME_BINDING_REQUIRED
printf 'this is not an executed or admitted proof\n' >"$receipt"
chmod 600 "$receipt"
before="$(sha256sum "$receipt")"
run_case forged-private-receipt HEAD_RUNTIME_BINDING_REQUIRED
[[ "$(sha256sum "$receipt")" == "$before" ]]
mv "$receipt" "$fixture/retained-receipt"
ln -s "$fixture/retained-receipt" "$receipt"
run_case symlink-receipt HEAD_RUNTIME_BINDING_REQUIRED
[[ -L "$receipt" ]]
[[ "$(cat "$fixture/retained-receipt")" == 'this is not an executed or admitted proof' ]]
run_case ci-refusal XBABE2_PROVIDER_PROOF_UNAVAILABLE CI=true
run_case actions-refusal XBABE2_PROVIDER_PROOF_UNAVAILABLE GITHUB_ACTIONS=true
run_case other-host XBABE2_PROVIDER_PROOF_UNAVAILABLE FIXTURE_HOST=foreign
printf '[ci] Head proof refuses absent/forged/symlink receipts, CI and other hosts (6 cases); no live qualification\n'
