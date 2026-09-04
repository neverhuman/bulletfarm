#!/usr/bin/env bash
# One process-tree offline component bridge: durable farmd UDS, product
# runner, production gitd, verifier fixture, and LocalBareForge. This command
# deliberately cannot emit or admit TRANSACTION_PROOF.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
KERNEL="$FAMILY/bullet-kernel"
GIT="$FAMILY/bullet-git"

if [[ -n "${BULLET_DATA_DIR:-}" ]]; then
  DATA="$BULLET_DATA_DIR"
  if [[ "$DATA" != /* || "$DATA" == "/" || ! -d "$DATA" || -L "$DATA" ]]; then
    echo "BULLET_DATA_DIR must name an existing absolute real directory" >&2
    exit 1
  fi
else
  DATA="$(mktemp -d /tmp/bullet-txn-offline.XXXXXX)"
  chmod 700 "$DATA"
fi

RECEIPT="${TRANSACTION_OFFLINE_RECEIPT:-$DATA/COMPONENT_PROOF.receipt.json}"
if [[ "$RECEIPT" != /* ]]; then
  echo "TRANSACTION_OFFLINE_RECEIPT must be an absolute path" >&2
  exit 1
fi

echo "== Bullet Farm offline transaction proof =="
echo "kernel: $KERNEL"
echo "data:   $DATA"

(cd "$GIT" && cargo build --locked -q -p bullet-gitd --bin bullet-gitd)
(cd "$KERNEL" && cargo build --locked -q -p bullet-farmd --bin bullet-farmd)
(cd "$KERNEL" && cargo build --locked -q -p bullet-runner --bin bullet-runner)
(cd "$KERNEL" && cargo build --locked -q -p bullet --bin transaction_offline)
(cd "$KERNEL" && cargo build --locked -q -p bullet-verifier --features fixture-executor --bin bullet-verifier-fixture)

VERIFIER_FIXTURE_BUILD_BIN="$KERNEL/target/debug/bullet-verifier-fixture"
VERIFIER_FIXTURE_STAGE="$(mktemp -d /tmp/bullet-verifier-fixture.XXXXXX)"
chmod 700 "$VERIFIER_FIXTURE_STAGE"
BULLET_VERIFIER_FIXTURE_BIN="$VERIFIER_FIXTURE_STAGE/bullet-verifier-fixture"
cp --reflink=never -- "$VERIFIER_FIXTURE_BUILD_BIN" "$BULLET_VERIFIER_FIXTURE_BIN"

export BULLET_GITD_BIN="$GIT/target/debug/bullet-gitd"
BULLET_GITD_SHA256="$(sha256sum -- "$BULLET_GITD_BIN")"
export BULLET_GITD_SHA256="${BULLET_GITD_SHA256%% *}"
export BULLET_FARMD_BIN="$KERNEL/target/debug/bullet-farmd"
export BULLET_RUNNER_BIN="$KERNEL/target/debug/bullet-runner"
BULLET_VERIFIER_FIXTURE_SHA256="$(sha256sum -- "$BULLET_VERIFIER_FIXTURE_BIN")"
export BULLET_VERIFIER_FIXTURE_SHA256="${BULLET_VERIFIER_FIXTURE_SHA256%% *}"
exec {BULLET_VERIFIER_FIXTURE_FD}<"$BULLET_VERIFIER_FIXTURE_BIN"
export BULLET_VERIFIER_FIXTURE_FD
export TRANSACTION_OFFLINE_RECEIPT="$RECEIPT"
export BULLET_DATA_DIR="$DATA"

umask 077
"$KERNEL/target/debug/transaction_offline" | tee "$DATA/transaction_offline.stdout"
if [[ ! -f "$RECEIPT" ]]; then
  python3 - "$DATA/transaction_offline.stdout" "$RECEIPT" <<'PY'
import json, sys
text = open(sys.argv[1], encoding="utf-8").read()
start = text.find("{")
end = text.rfind("}")
if start < 0 or end < 0:
    raise SystemExit("offline saga stdout had no JSON receipt")
open(sys.argv[2], "w", encoding="utf-8").write(text[start : end + 1] + "\n")
PY
fi
if grep -Fq '"evidence_class": "TRANSACTION_PROOF"' "$RECEIPT"; then
  echo "component bridge illegally emitted TRANSACTION_PROOF" >&2
  exit 1
fi
if ! grep -Fq '"evidence_class": "COMPONENT_PROOF"' "$RECEIPT"; then
  echo "offline component bridge receipt is not COMPONENT_PROOF" >&2
  exit 1
fi
for required in \
  '"transaction_gate_eligible": false' \
  '"independent_evidence_eligible": false' \
  '"verifier_outcome": "PASS"' \
  '"product_runner_gate_passed": true' \
  '"product_runner_outcome": "CANDIDATE_BINDING_REFUSED"'; do
  if ! grep -Fq "$required" "$RECEIPT"; then
    echo "offline component bridge receipt lacks $required" >&2
    exit 1
  fi
done
echo "== offline COMPONENT_PROOF written: $RECEIPT =="
