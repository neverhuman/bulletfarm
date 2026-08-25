#!/usr/bin/env bash
# Drive the offline five-plane TRANSACTION_PROOF saga and print the signed proof.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
KERNEL="$FAMILY/bullet-kernel"
GIT="$FAMILY/bullet-git"
PORTAL="$FAMILY/bullet-portal"
if [[ -n "${BULLET_DATA_DIR:-}" ]]; then
  DATA="$BULLET_DATA_DIR"
  mkdir -p "$DATA"
else
  DEMO_ROOT="$KERNEL/target/demo"
  mkdir -p "$DEMO_ROOT"
  DATA="$(mktemp -d "$DEMO_ROOT/run.XXXXXXXX")"
fi

echo "== Bullet Farm demo =="
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

(cd "$GIT" && cargo build -q -p bullet-gitd --bin bullet-gitd)
(cd "$GIT" && cargo build -q -p bullet-gitd --features fixture-authority --bin bullet-gitd-fixture)
(cd "$KERNEL" && cargo build -q -p bullet-farmd -p bullet-verifier -p bullet --bin transaction_demo)

export BULLET_GITD_BIN="${BULLET_GITD_BIN:-$GIT/target/debug/bullet-gitd}"
export BULLET_GITD_FIXTURE_BIN="${BULLET_GITD_FIXTURE_BIN:-$GIT/target/debug/bullet-gitd-fixture}"
export BULLET_FARMD_BIN="${BULLET_FARMD_BIN:-$KERNEL/target/debug/bullet-farmd}"
export BULLET_VERIFIER_BIN="${BULLET_VERIFIER_BIN:-$KERNEL/target/debug/bullet-verifier}"

(cd "$KERNEL" && BULLET_DATA_DIR="$DATA" cargo run -q -p bullet --bin transaction_demo)

if [[ "${BULLET_DEMO_PORTAL:-0}" == "1" ]]; then
  echo "== portal smoke =="
  (cd "$PORTAL" && npm test --silent)
fi

echo "== demo complete =="
