#!/usr/bin/env bash
# Compile the complete workspace and prove that setup refuses before mutation
# on every non-Linux platform advertised by scheduled CI.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
[[ "$(uname -s)" != Linux ]] \
  || { refuse PLATFORM_LANE_MISROUTED "run this lane on macOS or Windows"; exit 1; }
cargo check --locked --workspace --all-targets
cargo test --locked -p bullet-wire --test canonical_hostile
cargo clippy --locked -p bullet-family --lib --bins --no-deps -- \
  -D warnings -F clippy::disallowed_methods
cargo clippy --locked -p bullet-wire --lib --bins --no-deps -- \
  -D warnings -D clippy::disallowed_methods
selected="$(cargo test --locked --test ci_controls -- --list \
  | grep -Ec '^non_linux_setup_refuses_before_mutation: test$' || true)"
[[ "$selected" -eq 1 ]] \
  || { refuse TYPED_REFUSAL_TEST_DRIFT "selected $selected tests; expected 1"; exit 1; }
cargo test --locked --test ci_controls non_linux_setup_refuses_before_mutation -- --exact
recovery_selected="$(cargo test --locked -p bullet-family --test coord_rollover -- --list \
  | grep -Ec '^unsupported_platform_refuses_before_subject_io_or_coord_creation: test$' || true)"
[[ "$recovery_selected" -eq 1 ]] \
  || { refuse TYPED_RECOVERY_REFUSAL_TEST_DRIFT "selected $recovery_selected tests; expected 1"; exit 1; }
cargo test --locked -p bullet-family --test coord_rollover \
  unsupported_platform_refuses_before_subject_io_or_coord_creation -- --exact
log "non-Linux compile, strict decoder policy, and typed mutation refusal passed"
