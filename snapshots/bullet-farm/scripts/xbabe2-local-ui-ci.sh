#!/usr/bin/env bash
# Local component suites; this entry grants no installed/provider qualification.
set -euo pipefail
hub="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
family="$(cd "$hub/.." && pwd -P)"
selection="${1:-all}"
if (( $# > 1 )); then
  printf 'usage: xbabe2-local-ui-ci.sh [all|rendered|tuiwright|--list]\n' >&2
  exit 2
fi
case "$selection" in
  --list)
    printf '%s\n' \
      'rendered: Portal scripts/ci-local.sh rendered; exact Playwright identities checked by ops/ci/rendered.sh' \
      'tuiwright: Kernel scripts/ci-local.sh operator-tui; exact admitted harness and Bullet binaries required' \
      'all: requires both suites; unavailable is non-passing'
    exit 0 ;;
  all|rendered|tuiwright) ;;
  *) printf 'LOCAL_UI_SELECTION_INVALID: %s\n' "$selection" >&2; exit 2 ;;
esac
if [[ ${CI+x} || ${GITHUB_ACTIONS+x} || "${XBABE2_LOCAL_UI_CI:-}" != 1 \
  || "$(hostname -s)" != xbabe2 || "$(uname -s)" != Linux ]]; then
  printf 'XBABE2_LOCAL_UI_UNAVAILABLE: explicit xbabe2 local execution outside CI is required\n' >&2
  exit 78
fi
kernel="$family/bullet-kernel"
portal="$family/bullet-portal"
if [[ "$selection" != rendered && ( ! -f "$kernel/scripts/ci-local.sh" || ! -f "$kernel/ops/ci/operator-tui.sh" ) ]]; then
  printf 'TUIWRIGHT_SUITE_UNAVAILABLE: canonical Kernel qualification lane is missing\n' >&2
  exit 78
fi
if [[ "$selection" != tuiwright && ( ! -f "$portal/scripts/ci-local.sh" || ! -f "$portal/ops/ci/rendered.sh" ) ]]; then
  printf 'PLAYWRIGHT_SUITE_UNAVAILABLE: canonical Portal rendered lane is missing\n' >&2
  exit 78
fi
# Member dispatchers own admission, execution and selected/completed evidence.
# Forward the exact BULLET_TUIWRIGHT_{CARGO,RUSTC,BULLET}_BIN/SHA256,
# SOURCE_SHA256 and OUTPUT inputs unchanged; Kernel owns their admission.
# Never replace either execution with a saved receipt or a tool version probe.
if [[ "$selection" != rendered ]]; then
  printf 'Running Tuiwright component suite\n' >&2
  bash "$kernel/scripts/ci-local.sh" operator-tui
fi
if [[ "$selection" != tuiwright ]]; then
  printf 'Running rendered component suite\n' >&2
  exec bash "$portal/scripts/ci-local.sh" rendered
fi
