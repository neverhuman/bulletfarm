#!/usr/bin/env bash
# DF-R5: native macos-15 / windows-2025 compile-and-refusal evidence.
# This host may only claim that evidence when it is that platform.
# Linux records the honest unproved hold and never synthesizes a native receipt.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

kernel="$(uname -s)"
scheduled="$(<"$REPO_ROOT/.github/workflows/scheduled.yml")"
[[ "$scheduled" == *macos-15* && "$scheduled" == *windows-2025* ]] \
  || { refuse PLATFORM_LANE_DRIFT "scheduled.yml must name macos-15 and windows-2025"; exit 1; }
require_file ops/ci/platform-refusal.sh

if [[ "$kernel" == Linux ]]; then
  [[ "$mode" == --self-test || "$mode" == check ]] || exit 2
  refuse NATIVE_PLATFORM_EVIDENCE_UNAVAILABLE \
    "this Linux host cannot admit macos-15 or windows-2025 compile/refusal; run ops/ci/platform-refusal.sh on those runners"
  exit 1
fi

exec bash "$REPO_ROOT/ops/ci/platform-refusal.sh"
