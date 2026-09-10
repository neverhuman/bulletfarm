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
      'tuiwright: UNAVAILABLE; a pinned recording CLI is not an executable qualification suite' \
      'all: requires both suites; unavailable is non-passing'
    exit 0 ;;
  all|rendered|tuiwright) ;;
  *) printf 'LOCAL_UI_SELECTION_INVALID: %s\n' "$selection" >&2; exit 2 ;;
esac
if [[ "${CI:-}" == true || "${CI:-}" == 1 || "${GITHUB_ACTIONS:-}" == true \
  || "${GITHUB_ACTIONS:-}" == 1 || "${XBABE2_LOCAL_UI_CI:-}" != 1 \
  || "$(hostname -s)" != xbabe2 ]]; then
  printf 'XBABE2_LOCAL_UI_UNAVAILABLE: explicit xbabe2 local execution outside CI is required\n' >&2
  exit 78
fi
if [[ "$selection" != rendered ]]; then
  printf '%s\n' \
    'TUIWRIGHT_SUITE_UNAVAILABLE: no admitted Rust Tuiwright qualification suite and exact inventory exist yet' \
    'Recording/help/version probes and ordinary PTY component tests cannot satisfy this requirement' >&2
  exit 78
fi
portal="$family/bullet-portal"
if [[ ! -f "$portal/scripts/ci-local.sh" || ! -f "$portal/ops/ci/rendered.sh" ]]; then
  printf 'PLAYWRIGHT_SUITE_UNAVAILABLE: canonical Portal rendered lane is missing\n' >&2
  exit 78
fi
# The Portal dispatcher owns source admission, actual browser execution, exact
# selected/completed identity validation, report retention and final read-back.
printf 'Running rendered component suite only; Tuiwright remains unavailable\n' >&2
exec bash "$portal/scripts/ci-local.sh" rendered
