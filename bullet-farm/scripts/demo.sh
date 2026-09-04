#!/usr/bin/env bash
# Drive the offline five-plane fixture saga and print its self-signed component receipt.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
KERNEL="$FAMILY/bullet-kernel"
GIT="$FAMILY/bullet-git"
PORTAL="$FAMILY/bullet-portal"
if [[ -n "${BULLET_DATA_DIR:-}" ]]; then
  DATA="$BULLET_DATA_DIR"
  if [[ "$DATA" != /* || "$DATA" == "/" || ! -d "$DATA" || -L "$DATA" ]]; then
    echo "BULLET_DATA_DIR must name an existing absolute real directory" >&2
    exit 1
  fi
  CANONICAL_DATA="$(realpath -e -- "$DATA")"
  if [[ "$CANONICAL_DATA" != "$DATA" ]]; then
    echo "BULLET_DATA_DIR must be normalized and contain no symlinked ancestor" >&2
    exit 1
  fi
  if [[ "$(stat -c '%u:%a' -- "$DATA")" != "$(id -u):700" ]]; then
    echo "BULLET_DATA_DIR must be caller-owned with exact mode 0700" >&2
    exit 1
  fi
else
  DATA="$(mktemp -d /tmp/bullet-txn.XXXXXX)"
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

(cd "$GIT" && cargo build --locked -q -p bullet-gitd --bin bullet-gitd)
(cd "$GIT" && cargo build --locked -q -p bullet-gitd --features fixture-authority --bin bullet-gitd-fixture)
(cd "$KERNEL" && cargo build --locked -q -p bullet-farmd --bin bullet-farmd)
(cd "$KERNEL" && cargo build --locked -q -p bullet --bin transaction_demo)
(cd "$KERNEL" && cargo build --locked -q -p bullet-verifier --features fixture-executor --bin bullet-verifier-fixture)

VERIFIER_FIXTURE_BUILD_BIN="$KERNEL/target/debug/bullet-verifier-fixture"
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

export BULLET_GITD_BIN="$GIT/target/debug/bullet-gitd"
export BULLET_GITD_FIXTURE_BIN="$GIT/target/debug/bullet-gitd-fixture"
BULLET_GITD_SHA256="$(sha256sum -- "$BULLET_GITD_BIN")"
BULLET_GITD_SHA256="${BULLET_GITD_SHA256%% *}"
export BULLET_GITD_SHA256
BULLET_GITD_FIXTURE_SHA256="$(sha256sum -- "$BULLET_GITD_FIXTURE_BIN")"
BULLET_GITD_FIXTURE_SHA256="${BULLET_GITD_FIXTURE_SHA256%% *}"
export BULLET_GITD_FIXTURE_SHA256
export BULLET_FARMD_BIN="$KERNEL/target/debug/bullet-farmd"
BULLET_VERIFIER_FIXTURE_SHA256="$(sha256sum -- "$BULLET_VERIFIER_FIXTURE_BIN")"
BULLET_VERIFIER_FIXTURE_SHA256="${BULLET_VERIFIER_FIXTURE_SHA256%% *}"
export BULLET_VERIFIER_FIXTURE_BIN
export BULLET_VERIFIER_FIXTURE_SHA256

(cd "$KERNEL" && BULLET_DATA_DIR="$DATA" cargo run --locked -q -p bullet --bin transaction_demo)

if [[ "${BULLET_DEMO_PORTAL:-0}" == "1" ]]; then
  echo "== portal smoke =="
  (cd "$PORTAL" && npm test --silent)
fi

echo "== component demo complete (TRANSACTION_PROOF remains absent) =="
