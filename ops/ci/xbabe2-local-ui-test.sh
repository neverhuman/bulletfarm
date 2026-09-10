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
  env PATH="$fixture/bin:$PATH" CI= GITHUB_ACTIONS= XBABE2_LOCAL_UI_CI=1 \
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
printf '%s\n' "$*" >"$FIXTURE_CHILD_LOG"
exit "${FIXTURE_CHILD_STATUS:-0}"
CHILD
run_case 24 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=24
[[ "$(cat "$fixture/selected")" == rendered ]]
run_case 0 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=0
run_case 0 --list
grep -Fq 'tuiwright: UNAVAILABLE' "$fixture/result.log"
run_case 2 invalid
printf '[ci] local UI dispatcher: 11 refusal/routing cases passed; no rendered qualification\n'
