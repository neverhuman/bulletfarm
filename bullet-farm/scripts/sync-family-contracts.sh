#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
MODE="${1:-write}"
CONTRACT_SYNC_STAGING=""

finish() {
  if [[ -n "$CONTRACT_SYNC_STAGING" ]]; then
    rm -f -- "$CONTRACT_SYNC_STAGING"
  fi
}
trap finish EXIT

sync_file() {
  local source="$1"
  local destination="$2"
  if [[ "$MODE" == "check" ]]; then
    if ! cmp -s -- "$source" "$destination"; then
      echo "generated contract drift: $destination" >&2
      exit 1
    fi
    return
  fi
  mkdir -p -- "$(dirname "$destination")"
  CONTRACT_SYNC_STAGING="$(mktemp "${destination}.XXXXXX")"
  cp -- "$source" "$CONTRACT_SYNC_STAGING"
  cmp -s -- "$source" "$CONTRACT_SYNC_STAGING"
  mv -- "$CONTRACT_SYNC_STAGING" "$destination"
  CONTRACT_SYNC_STAGING=""
}

if [[ "$MODE" != "write" && "$MODE" != "check" ]]; then
  echo "usage: sync-family-contracts.sh [write|check]" >&2
  exit 2
fi

sync_file \
  "$HUB/contracts/generated/rust/schema_bundle.rs" \
  "$FAMILY/bullet-kernel/crates/domain/src/schema_bundle.rs"
sync_file \
  "$HUB/policy/v1alpha1/policy.json" \
  "$FAMILY/bullet-kernel/crates/application/tests/fixtures/policy-v1alpha1.json"
sync_file \
  "$HUB/crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json" \
  "$FAMILY/bullet-kernel/crates/application/tests/fixtures/policy-v1alpha2-live-enabled.json"
sync_file \
  "$HUB/contracts/generated/rust/schema_bundle.rs" \
  "$FAMILY/bullet-git/contracts/generated/rust/schema_bundle.rs"
sync_file \
  "$HUB/contracts/generated/typescript/schemaBundle.ts" \
  "$FAMILY/bullet-portal/src/generated/schemaBundle.ts"
sync_file \
  "$FAMILY/bullet-kernel/contracts/generated/api.ts" \
  "$FAMILY/bullet-portal/src/generated/api.ts"
for trace in effect-check-ambiguity effect-third-party lease-fence-reclaim; do
  sync_file \
    "$HUB/formal/traces/${trace}.json" \
    "$FAMILY/bullet-kernel/crates/adapters/tests/fixtures/formal/${trace}.json"
done

echo "family contract consumers $MODE passed"
