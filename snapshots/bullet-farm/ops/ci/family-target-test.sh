#!/usr/bin/env bash
# Hub target and Gitd inheritance proof using its exact driver and fixture receivers.
# Reproduce the operator default; the family build must tighten its own mask.
set -euo pipefail
umask 0002
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture="$(mktemp -d)"
cleanup() { chmod -R u+rwx -- "$fixture"; rm -rf -- "$fixture"; }
trap cleanup EXIT
for member in bullet-farm bullet-git bullet-kernel bullet-portal; do
  mkdir -p "$fixture/$member/.git" "$fixture/$member/ops/ci" "$fixture/$member/scripts"
  printf 'ref: refs/heads/main\n' >"$fixture/$member/.git/HEAD"
done
farm="$fixture/bullet-farm"
cp "$REPO_ROOT/ops/ci/family.sh" "$farm/ops/ci/family.sh"
cp "$REPO_ROOT/ops/ci/family-custody.sh" "$farm/ops/ci/family-custody.sh"
cp "$REPO_ROOT/ops/ci/toolchain-pins.sh" "$farm/ops/ci/toolchain-pins.sh"
cp "$REPO_ROOT/.node-version" "$REPO_ROOT/.npm-version" "$farm/"
cat >"$farm/ops/ci/lib.sh" <<'LIB'
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WIRE_EXPECTED_TESTS=1
refuse() { printf '%s\n' "$*" >&2; return 1; }
prepare_ci_directory() { mkdir -p "$1/$2"; }
sha256_file() { sha256sum "$1" | cut -d ' ' -f 1; }
log() {
  if [[ "$1" == '4/7 '* ]]; then
    case "$TARGET_MODE" in
      family-dir) export BULLET_CI_CARGO_TARGET_DIR="$TARGET_MARKER" ;;
      family-id) export BULLET_CI_CARGO_TARGET_ID="$TARGET_MARKER" ;;
    esac
  fi
}
LIB
printf 'printf "1\\n"\n' >"$farm/ops/ci/family-report-check.sh"
printf 'FAST_EXPECTED_TESTS=1\nCONTRACT_EXPECTED_TESTS=1\n' >"$fixture/bullet-git/ops/ci/lib.sh"
printf 'EXPECTED_STANDALONE_TESTS=1\nEXPECTED_CONTRACT_TESTS=1\nEXPECTED_FAMILY_TESTS=1\n' \
  >"$fixture/bullet-kernel/ops/ci/inventory.sh"
mkdir "$fixture/bullet-portal/e2e"
printf 'test("fixture", () => {})\n' >"$fixture/bullet-portal/e2e/real-farmd.spec.ts"
printf 'node ops/ci/assert-report.mjs junit "\044reports/playwright.xml" 1\n' \
  >"$fixture/bullet-portal/ops/ci/contract.sh"
cat >"$fixture/member-lane.sh" <<'MEMBER'
#!/usr/bin/env bash
set -euo pipefail
member="${PWD##*/}"
[[ "${CARGO_TARGET_DIR:-}" == "$HUB_TARGET" ]] || exit 90
printf '%s %s\n' "$member" "$1" >>"$TARGET_TRACE"
case "$member:$1" in
  bullet-portal:required)
    [[ ! ${BULLET_GITD_BIN+x} && ! ${BULLET_GITD_SHA256+x} ]] || exit 98
    if [[ "$TARGET_MODE" == portal-before-change ]]; then
      IFS= read -r gitd_bin <"$TARGET_GITD_SUBJECT"
      printf '\n# changed before Portal family\n' >>"$gitd_bin"
    fi
    ;;
  bullet-portal:family)
    mapfile -t expected <"$TARGET_GITD_SUBJECT"
    [[ "${BULLET_GITD_BIN:-}" == "${expected[0]}" \
      && "${BULLET_GITD_SHA256:-}" == "${expected[1]}" \
      && "$BULLET_GITD_BIN" == /* && -f "$BULLET_GITD_BIN" \
      && -x "$BULLET_GITD_BIN" && ! -L "$BULLET_GITD_BIN" \
      && "$(stat -Lc '%a' -- "$BULLET_GITD_BIN")" == 700 \
      && "$(sha256sum "$BULLET_GITD_BIN" | cut -d ' ' -f 1)" == "$BULLET_GITD_SHA256" ]] || exit 98
    printf 'portal received exact Gitd\n' >>"$TARGET_TRACE"
    if [[ "$TARGET_MODE" == portal-after-change ]]; then
      printf '\n# changed during Portal family\n' >>"$BULLET_GITD_BIN"
    fi
    ;;
esac
MEMBER
cp "$fixture/member-lane.sh" "$fixture/bullet-git/scripts/ci-local.sh"
cp "$fixture/member-lane.sh" "$fixture/bullet-portal/scripts/ci-local.sh"
cat >"$fixture/bullet-kernel/scripts/ci-local.sh" <<'KERNEL'
#!/usr/bin/env bash
set -euo pipefail
[[ ! ${CARGO_TARGET_DIR+x} ]] || exit 91
case "$TARGET_MODE:$1" in
  required-dir:required|family-dir:family)
    [[ "${BULLET_CI_CARGO_TARGET_DIR:-}" == "$TARGET_MARKER" \
      && ! ${BULLET_CI_CARGO_TARGET_ID+x} ]] || exit 96
    printf 'fixture receiver preserved DIR bytes\n' >&2
    exit 75
    ;;
  required-id:required|family-id:family)
    [[ "${BULLET_CI_CARGO_TARGET_ID:-}" == "$TARGET_MARKER" \
      && ! ${BULLET_CI_CARGO_TARGET_DIR+x} ]] || exit 96
    printf 'fixture receiver preserved ID bytes\n' >&2
    exit 75
    ;;
  *) [[ ! ${BULLET_CI_CARGO_TARGET_DIR+x} && ! ${BULLET_CI_CARGO_TARGET_ID+x} ]] || exit 96 ;;
esac
printf 'kernel %s\n' "$1" >>"$TARGET_TRACE"
if [[ "$1" == family ]]; then
  printf '%s\n%s\n' "$BULLET_GITD_BIN" "$BULLET_GITD_SHA256" >"$TARGET_GITD_SUBJECT"
fi
KERNEL
cat >"$farm/scripts/sync-family-contracts.sh" <<'SYNC'
[[ "$CARGO_TARGET_DIR" == "$HUB_TARGET" ]] || exit 92
SYNC
cat >"$farm/ops/ci/contract.sh" <<'CONTRACT'
[[ "$CARGO_TARGET_DIR" == "$HUB_TARGET" ]] || exit 93
[[ ! ${BULLET_GITD_BIN+x} && ! ${BULLET_GITD_SHA256+x} ]] || exit 98
printf 'hub complete\n' >>"$TARGET_TRACE"
exit 97
CONTRACT
mkdir "$fixture/tools"
cat >"$fixture/tools/tool" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
case "${0##*/}:$*" in
  node:--version) printf 'v22.23.2\n' ;;
  npm:--version) printf '10.9.8\n' ;;
  b3sum:--version) printf 'b3sum 1.8.2\n' ;;
  rustc:--version) printf 'rustc 1.95.0 fixture\n' ;;
  cargo:--version) printf 'cargo 1.95.0 fixture\n' ;;
  rustup:--version) printf 'rustup 1.29.0 fixture\n' ;;
  'rustup:run 1.97.1 rustc --version') printf 'rustc 1.97.1 fixture\n' ;;
  'rustup:run 1.97.1 cargo --version') printf 'cargo 1.97.1 fixture\n' ;;
  'rustup:run 1.97.1 cargo build --locked -p bullet-gitd --bin bullet-gitd')
    mkdir -p "$CARGO_TARGET_DIR/debug"
    printf '#!/usr/bin/env bash\nexit 0\n' >"$CARGO_TARGET_DIR/debug/bullet-gitd"
    chmod +x "$CARGO_TARGET_DIR/debug/bullet-gitd"
    ;;
  git:*)
    case "$*" in
      *'rev-parse --verify HEAD') printf '%040d\n' 1 ;;
      *'rev-parse --verify HEAD^{tree}') printf '%040d\n' 2 ;;
      *'status --porcelain=v1 --untracked-files=all') : ;;
      *) exit 94 ;;
    esac
    ;;
  *) exit 95 ;;
esac
TOOL
chmod +x "$fixture/tools/tool"
for tool in node npm b3sum rustc cargo rustup git; do
  ln -s tool "$fixture/tools/$tool"
done
# Use real custody in fixture directories only. The child sees this process as
# its owning parent, exactly as in the canonical family launcher.
# shellcheck source=ops/ci/family-custody.sh
source "$REPO_ROOT/ops/ci/family-custody.sh"
marker=$'spoofed marker\nwith spaces : data'
for mode in normal required-dir required-id family-dir family-id portal-before-change portal-after-change; do
  : >"$fixture/trace"
  : >"$fixture/gitd-subject"
  record=''
  ci_proof_acquire "$farm" bullet-farm family family record
  extra=()
  case "$mode" in
    required-dir) extra=("BULLET_CI_CARGO_TARGET_DIR=$marker") ;;
    required-id) extra=("BULLET_CI_CARGO_TARGET_ID=$marker") ;;
  esac
  set +e
  env -u BULLET_CI_CARGO_TARGET_DIR -u BULLET_CI_CARGO_TARGET_ID \
    -u BULLET_GITD_BIN -u BULLET_GITD_SHA256 \
    PATH="$fixture/tools:$PATH" TARGET_MODE="$mode" TARGET_TRACE="$fixture/trace" TARGET_MARKER="$marker" \
    TARGET_GITD_SUBJECT="$fixture/gitd-subject" \
    HUB_TARGET="$fixture/hub-target" CARGO_TARGET_DIR="$fixture/hub-target" \
    BULLET_CI_PROOF_CUSTODY="$record" "${extra[@]}" \
    bash "$farm/ops/ci/family.sh" >"$fixture/run.log" 2>&1
  status=$?
  set -e
  ci_proof_release "$farm" bullet-farm "$record" family
  grep -Fxq 'bullet-git required' "$fixture/trace"
  case "$mode" in
    normal)
      [[ "$status" == 97 ]]
      [[ "$(grep -c '^kernel ' "$fixture/trace")" == 2 ]]
      grep -Fxq 'kernel required' "$fixture/trace"
      grep -Fxq 'kernel family' "$fixture/trace"
      grep -Fxq 'bullet-portal required' "$fixture/trace"
      grep -Fxq 'bullet-portal family' "$fixture/trace"
      grep -Fxq 'portal received exact Gitd' "$fixture/trace"
      grep -Fxq 'hub complete' "$fixture/trace"
      ;;
    required-*)
      [[ "$status" == 75 ]]
      [[ "$(grep -c '^kernel ' "$fixture/trace" || true)" == 0 ]]
      grep -Eq '^fixture receiver preserved (DIR|ID) bytes$' "$fixture/run.log"
      ;;
    family-*)
      [[ "$status" == 75 ]]
      [[ "$(grep -c '^kernel ' "$fixture/trace")" == 1 ]]
      grep -Eq '^fixture receiver preserved (DIR|ID) bytes$' "$fixture/run.log"
      ;;
    portal-*)
      [[ "$status" == 1 ]]
      grep -Fxq 'bullet-portal required' "$fixture/trace"
      if grep -Fxq 'hub complete' "$fixture/trace"; then exit 1; fi
      if [[ "$mode" == portal-before-change ]]; then
        if grep -Fxq 'bullet-portal family' "$fixture/trace"; then exit 1; fi
        grep -Fxq 'BULLET_GITD_BIN_CHANGED before-portal-family' "$fixture/run.log"
      else
        grep -Fxq 'portal received exact Gitd' "$fixture/trace"
        grep -Fxq 'BULLET_GITD_BIN_CHANGED after-portal-family' "$fixture/run.log"
      fi
      ;;
  esac
  [[ -z "$(find "$fixture" -type d -name bullet-ci.lock.d -print)" ]]
done
echo '[ci] family Kernel target and Portal Gitd boundaries: 7 fixture scenarios passed'
