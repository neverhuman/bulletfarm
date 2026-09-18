#!/usr/bin/env bash
# Development-host-only lane: the real console and the real browser.
#
# This lane exists because two things can only be proved on the machine that
# holds the operator's provider credentials: `bullet tui` driven through a real
# pseudo-terminal against an authenticated daemon, and the Portal driven by a
# real browser against that same daemon. A hosted runner has no such session, so
# a hosted copy of this lane would look like proof while proving something
# weaker. It refuses there by name rather than degrading quietly.
#
# Nothing here spends money, launches a provider, or reads credential bytes.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

# A hosted runner cannot hold the session this lane reads. Refuse before any work.
if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  refuse DEVNODE_LANE_IS_LOCAL_ONLY \
    "this lane needs an authenticated operator session and provider credentials, which exist only on the development host"
  exit 1
fi

DEVNODE_BIN_DIR="${BULLET_BIN_DIR:-$HOME/.local/bin}"
DEVNODE_BULLET="${BULLET_BIN:-$DEVNODE_BIN_DIR/bullet}"
DEVNODE_BIND="${BULLET_DEVNODE_BIND:-127.0.0.1:7420}"
DEVNODE_TUIWRIGHT="${BULLET_TUIWRIGHT_ROOT:-$(cd "$REPO_ROOT/../.." 2>/dev/null && pwd -P)/jankurai}"
DEVNODE_ARTIFACTS="$REPO_ROOT/.ci-artifacts/devnode"

# Every prerequisite refuses by name with the exact remedy. A missing input is
# never a skip: a lane that quietly proves less is worse than one that stops.
require_devnode_input() {
  local kind="$1" subject="$2" remedy="$3"
  refuse "DEVNODE_${kind}" "$subject; remedy: $remedy"
  exit 1
}

[[ -x "$DEVNODE_BULLET" ]] || require_devnode_input BINARY_MISSING \
  "no executable bullet at $DEVNODE_BULLET" "run ./bootstrap.sh, or set BULLET_BIN"

[[ -d "$DEVNODE_TUIWRIGHT/crates/tuiwright" ]] || require_devnode_input TUIWRIGHT_MISSING \
  "no Tuiwright checkout at $DEVNODE_TUIWRIGHT/crates/tuiwright" \
  "clone https://github.com/neverhuman/jankurai beside the family, or set BULLET_TUIWRIGHT_ROOT"

command -v curl >/dev/null 2>&1 || require_devnode_input TOOL_MISSING "curl" "install curl"

# The browser half drives the Portal against a daemon it builds itself, so it
# needs the Portal and Kernel checkouts beside this one, exactly as the Portal's
# own real-farmd lane documents.
DEVNODE_FAMILY="$(cd "$REPO_ROOT/.." && pwd -P)"
DEVNODE_PORTAL="$DEVNODE_FAMILY/bullet-portal"
[[ -f "$DEVNODE_PORTAL/ops/ci/real-farmd.sh" ]] || require_devnode_input PORTAL_MISSING \
  "no bullet-portal checkout at $DEVNODE_PORTAL" "clone bullet-portal beside this repository"
[[ -f "$DEVNODE_FAMILY/bullet-kernel/Cargo.toml" ]] || require_devnode_input KERNEL_MISSING \
  "no bullet-kernel checkout at $DEVNODE_FAMILY/bullet-kernel" "clone bullet-kernel beside this repository"

if ! curl --fail --silent --show-error --max-time 3 "http://$DEVNODE_BIND/health" >/dev/null 2>&1; then
  require_devnode_input DAEMON_UNREACHABLE \
    "nothing answered http://$DEVNODE_BIND/health" \
    "start the daemon first, then rerun this lane"
fi

prepare_ci_directory "$REPO_ROOT" .ci-artifacts/devnode

log "console flows through a real pseudo-terminal"
# Serialised because that is the configuration proved, not because concurrency
# is known to be forbidden. Observed once: of six concurrent consoles, two never
# reached first paint within fifteen seconds; serialised, none failed. The
# concurrency at which first paint starts failing has not been measured.
(
  cd devnode/tui
  BULLET_BIN="$DEVNODE_BULLET" \
  BULLET_DEVNODE_ARTIFACTS="$DEVNODE_ARTIFACTS" \
  CARGO_TARGET_DIR="${BULLET_DEVNODE_TARGET:-$HOME/.cache/bullet-devnode-target}" \
    cargo test --locked --test tui -- --test-threads=1
)

log "the 1080p recorder still builds against its pinned harness"
# Building it, not running it. A recording is a deliberate act with a private
# output directory and a real session; a lane that produced published media as a
# side effect would be a lane that publishes without anyone asking. Building it
# is what stops it rotting silently while the console changes underneath it.
(
  cd devnode/record
  CARGO_TARGET_DIR="${BULLET_DEVNODE_TARGET:-$HOME/.cache/bullet-devnode-target}" \
    cargo build --locked
)

log "browser flows against a real daemon"
# Both halves are mandatory. A lane that can skip its browser half while still
# reporting success is a lane that quietly proves less than it appears to, which
# is the exact failure this lane exists to avoid.
(
  cd "$DEVNODE_PORTAL"
  bash ops/ci/real-farmd.sh
)
(
  cd "$DEVNODE_PORTAL"
  bash ops/ci/real-farmd.sh --packaged
)

log "devnode lane passed"
