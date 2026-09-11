#!/usr/bin/env bash
# Dispatcher component tests use fake children only; no browser/TUI qualification.
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
umask 077
fixture="$(mktemp -d)"
cleanup() {
  local status=$?
  if (( status == 0 )); then rm -rf -- "$fixture";
  else printf '[ci] retained failed local UI fixture: %s\n' "$fixture" >&2; fi
}
trap cleanup EXIT
mkdir -p "$fixture/family/bullet-farm/scripts" "$fixture/bin"
cp "$repo/scripts/xbabe2-local-ui-ci.sh" "$fixture/family/bullet-farm/scripts/"
cat >"$fixture/bin/hostname" <<'HOST'
#!/usr/bin/env bash
printf '%s\n' "${FIXTURE_HOST:-xbabe2}"
HOST
cat >"$fixture/bin/tuiwright" <<'TOOL'
#!/usr/bin/env bash
printf 'unqualified tool probe\n' >>"$FIXTURE_TOOL_LOG"
exit 0
TOOL
chmod 700 "$fixture/bin/hostname" "$fixture/bin/tuiwright"
run_case() {
  local expected="$1" selector="$2"; shift 2
  local status=0
  env -u CI -u GITHUB_ACTIONS PATH="$fixture/bin:$PATH" XBABE2_LOCAL_UI_CI=1 \
    XBABE2_ALLOW_ANY_HOST=1 FIXTURE_TOOL_LOG="$fixture/tool-called" "$@" \
    bash "$fixture/family/bullet-farm/scripts/xbabe2-local-ui-ci.sh" "$selector" \
    >"$fixture/result.log" 2>&1 || status=$?
  [[ "$status" == "$expected" ]] || { cat "$fixture/result.log" >&2; return 1; }
  [[ ! -e "$fixture/tool-called" ]]
  ! grep -Eq ': PASS|skip registry hit' "$fixture/result.log"
}
run_case 78 all
grep -Fq TUIWRIGHT_SUITE_UNAVAILABLE "$fixture/result.log"
run_case 78 tuiwright
run_case 78 all CI=true
run_case 78 all GITHUB_ACTIONS=1
run_case 78 all FIXTURE_HOST=not-xbabe2
run_case 78 all XBABE2_LOCAL_UI_CI=0
run_case 78 rendered
grep -Fq PLAYWRIGHT_SUITE_UNAVAILABLE "$fixture/result.log"
portal="$fixture/family/bullet-portal"
mkdir -p "$portal/scripts" "$portal/ops/ci"
touch "$portal/ops/ci/rendered.sh"
cat >"$portal/scripts/ci-local.sh" <<'CHILD'
#!/usr/bin/env bash
[[ "$*" == rendered ]] || exit 91
printf '%s\n' "$*" >>"$FIXTURE_CHILD_LOG"
exit "${FIXTURE_CHILD_STATUS:-0}"
CHILD
run_case 24 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=24
[[ "$(cat "$fixture/selected")" == rendered ]]
run_case 0 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=0
run_case 0 --list
grep -Fq 'tuiwright: Kernel scripts/ci-local.sh tuiwright' "$fixture/result.log"
run_case 2 invalid
run_case 78 all CI=false
run_case 78 all CI=
run_case 78 all GITHUB_ACTIONS=false
run_case 78 all GITHUB_ACTIONS=
kernel="$fixture/family/bullet-kernel"
mkdir -p "$kernel/scripts" "$kernel/ops/ci"
touch "$kernel/ops/ci/tuiwright.sh"
cat >"$kernel/scripts/ci-local.sh" <<'CHILD'
#!/usr/bin/env bash
[[ "$*" == tuiwright ]] || exit 92
printf '%s\n' "$*" >>"$FIXTURE_CHILD_LOG"
exit "${FIXTURE_TUI_STATUS:-0}"
CHILD
run_case 78 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-unavailable" FIXTURE_TUI_STATUS=78
[[ "$(cat "$fixture/tui-unavailable")" == tuiwright ]]
run_case 31 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-failed" FIXTURE_TUI_STATUS=31
run_case 0 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-only"
[[ "$(cat "$fixture/tui-only")" == tuiwright ]]
run_case 31 all FIXTURE_CHILD_LOG="$fixture/all-tui-failed" FIXTURE_TUI_STATUS=31
[[ "$(cat "$fixture/all-tui-failed")" == tuiwright ]]
run_case 24 all FIXTURE_CHILD_LOG="$fixture/all-rendered-failed" FIXTURE_CHILD_STATUS=24
[[ "$(cat "$fixture/all-rendered-failed")" == $'tuiwright\nrendered' ]]
run_case 0 all FIXTURE_CHILD_LOG="$fixture/all-complete"
[[ "$(cat "$fixture/all-complete")" == $'tuiwright\nrendered' ]]
printf '[ci] local UI dispatcher: 21 refusal/routing cases passed; no browser or terminal qualification\n'
