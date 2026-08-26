#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KERNEL="$(cd "$HUB/../bullet-kernel" && pwd)"
cd "$HUB"

command -v jq >/dev/null 2>&1 || {
  echo "preview: missing jq" >&2
  exit 1
}

doctor_report="$(mktemp)"
release_report="$(mktemp)"
demo_data="$(mktemp -d)"
receipt_registry="$(mktemp -d)"
cleanup() {
  rm -f "$doctor_report" "$release_report"
  rm -rf "$demo_data" "$receipt_registry"
}
trap cleanup EXIT

echo "== tool diagnostics =="
bash scripts/ci-doctor.sh fast

echo "== family diagnosis (BLOCKED is the expected alpha state) =="
doctor_exit=0
cargo run --locked --quiet --bin bullet-family -- doctor --json >"$doctor_report" || doctor_exit=$?
cat "$doctor_report"
if [[ "$doctor_exit" -ne 3 ]] || ! jq -e '.status == "BLOCKED"' "$doctor_report" >/dev/null; then
  echo "preview: doctor must report BLOCKED with exit 3 for this alpha snapshot" >&2
  exit 1
fi

echo "== Hub component lane =="
bash scripts/ci-local.sh fast

echo "== deterministic credential-free component demo =="
(cd "$KERNEL" && BULLET_DATA_DIR="$demo_data" cargo run --locked --quiet -p bullet --bin bullet -- demo)
jq -e '
  .materialize_idempotent == true
  and .stale_refused == true
  and .fence_first == 1
  and .fence_second == 2
  and .candidate_head == "NOT_PRODUCED"
  and .evidence_result == "NOT_RUN"
  and .effect_outcome == "NOT_DISPATCHED"
  and .effect_unknown_outcome == "NOT_DISPATCHED"
' "$demo_data/receipts.json" >/dev/null

echo "== release diagnosis (still BLOCKED) =="
release_exit=0
cargo run --locked --quiet --bin bullet-family -- check release \
  --profile self-hosted-v1 --receipts "$receipt_registry" --json >"$release_report" || release_exit=$?
cat "$release_report"
if [[ "$release_exit" -ne 3 ]] || ! jq -e '
  .profile == "self-hosted-v1"
  and .status == "BLOCKED"
  and (.gates | length) == 27
  and all(.gates[]; .status == "BLOCKED")
  and ([.gates[] | select(.status == "PASS")] | length) == 0
' "$release_report" >/dev/null; then
  echo "preview: self-hosted-v1 must remain 27/27 BLOCKED with exit 3 and zero PASS" >&2
  exit 1
fi
if ! jq -e '
  ([.gates[] | select(.id == "release.transaction-demo")] | length) == 1
  and all(.gates[] | select(.id == "release.transaction-demo"); .status == "BLOCKED")
  and ([.gates[] | select(.id == "release.profile.self-hosted-v1")] | length) == 1
  and all(.gates[] | select(.id == "release.profile.self-hosted-v1"); .status == "BLOCKED")
' "$release_report" >/dev/null; then
  echo "preview: self-hosted condition and transaction-demo must each remain uniquely BLOCKED" >&2
  exit 1
fi

echo "preview: PASS (component behavior only; public install and release remain BLOCKED)"
