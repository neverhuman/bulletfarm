#!/usr/bin/env bash
# Disposable hosted publication proof. Never publishes refs or changes GITHUB_SHA.
set -euo pipefail

refuse() {
  printf '%s\n' "$1" >&2
  exit 1
}

test_inventory() {
  local log="$1" names digest bytes
  [[ -f "$log" && ! -L "$log" ]] || refuse PUBLICATION_TEST_INVENTORY_INVALID
  bytes="$(wc -c <"$log")"
  [[ "$bytes" -gt 0 && "$bytes" -le 16777216 ]] || refuse PUBLICATION_TEST_INVENTORY_INVALID
  # Compare completed identities, including duplicates, against the independently
  # reviewed publication subset of the Hub inventory. Count changes require review.
  if ! names="$(LC_ALL=C awk '
    /^test result:/ {
      summaries++
      if ($0 !~ /^test result: ok\. 34 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in [0-9]+([.][0-9]+)?s$/) bad = 1
      next
    }
    /^test / {
      tests++
      if ($0 !~ /^test publication::[A-Za-z0-9_:]+ \.\.\. ok$/) bad = 1
      else print $2
    }
    END { if (bad || tests != 34 || summaries != 1) exit 1 }
  ' "$log" | LC_ALL=C sort)"; then
    refuse PUBLICATION_TEST_INVENTORY_INVALID
  fi
  digest="$(printf '%s\n' "$names" | sha256sum | cut -d ' ' -f 1)"
  [[ "$digest" == f309efec38c41eed248b39a9c64ccb1e466ba02aabc5c89ac978ac12cb5fa35d ]] \
    || refuse PUBLICATION_TEST_INVENTORY_INVALID
}

wrapper_inventory() {
  local log="$1" names digest bytes
  [[ -f "$log" && ! -L "$log" ]] || refuse PUBLICATION_WRAPPER_INVENTORY_INVALID
  bytes="$(wc -c <"$log")"
  [[ "$bytes" -gt 0 && "$bytes" -le 16777216 ]] || refuse PUBLICATION_WRAPPER_INVENTORY_INVALID
  # These 48 names map to the independently reviewed wrapper cases, including actual v2 generation.
  # Successful exit and a count alone cannot admit missing or changed fixtures.
  if ! names="$(LC_ALL=C awk '
    /^publication wrapper fixtures:/ {
      summaries++
      if ($0 != "publication wrapper fixtures: 48 passed; 0 failed; 0 skipped") bad = 1
      next
    }
    /^publication wrapper case:/ {
      tests++
      if ($0 !~ /^publication wrapper case: [a-z0-9_]+ \.\.\. ok$/) bad = 1
      else print $4
      next
    }
    /^publication wrapper/ {bad = 1}
    END {if (bad || tests != 48 || summaries != 1) exit 1}
  ' "$log" | LC_ALL=C sort)"; then
    refuse PUBLICATION_WRAPPER_INVENTORY_INVALID
  fi
  digest="$(printf '%s\n' "$names" | sha256sum | cut -d ' ' -f 1)"
  [[ "$digest" == 1a7cee84d811c3dc2c22af09d6d9843e454794d73b5eec4e99a8b739ec5baf6a ]] \
    || refuse PUBLICATION_WRAPPER_INVENTORY_INVALID
}

admit_runner() {
  [[ "${GITHUB_ACTIONS:-}" == true ]] || refuse PUBLICATION_DISPOSABLE_CI_REQUIRED
  [[ "${RUNNER_TEMP:-}" == /* && -d "$RUNNER_TEMP" && ! -L "$RUNNER_TEMP" ]] \
    || refuse PUBLICATION_RUNNER_TEMP_INVALID
  [[ "$(realpath -e "$RUNNER_TEMP")" == "$RUNNER_TEMP" ]] \
    || refuse PUBLICATION_RUNNER_TEMP_INVALID
  [[ "${GITHUB_WORKSPACE:-}" == /* && -d "$GITHUB_WORKSPACE/.git" \
    && ! -L "$GITHUB_WORKSPACE" && ! -L "$GITHUB_WORKSPACE/.git" ]] \
    || refuse PUBLICATION_CHECKOUT_INVALID
  [[ "$(realpath -e "$GITHUB_WORKSPACE")" == "$GITHUB_WORKSPACE" ]] \
    || refuse PUBLICATION_CHECKOUT_INVALID
  [[ "${GITHUB_SHA:-}" =~ ^[0-9a-f]{40}$ ]] || refuse PUBLICATION_EVENT_SHA_INVALID
  [[ "$(git -C "$GITHUB_WORKSPACE" rev-parse HEAD)" == "$GITHUB_SHA" ]] \
    || refuse PUBLICATION_EVENT_SHA_MISMATCH
  [[ -z "$(git -C "$GITHUB_WORKSPACE" status --porcelain=v1 --untracked-files=all)" ]] \
    || refuse PUBLICATION_CHECKOUT_DIRTY
  [[ "$(git -C "$GITHUB_WORKSPACE" rev-parse --is-shallow-repository)" == false ]] \
    || refuse PUBLICATION_SHALLOW_CHECKOUT
  report="$RUNNER_TEMP/bullet-publication-report"
  private="$RUNNER_TEMP/bullet-publication-private"
}

source_scan() {
  admit_runner
  umask 077
  mkdir "$report" "$private"
  printf 'publication bootstrap started\n' >"$report/status.txt"
  local scanner
  scanner="$(command -v gitleaks)"
  [[ "$scanner" == /* && -f "$scanner" && ! -L "$scanner" ]] \
    || refuse PUBLICATION_SCANNER_INVALID
  printf '%s  %s\n' \
    '50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359' "$scanner" \
    | sha256sum --check --status || refuse PUBLICATION_SCANNER_IDENTITY
  [[ "$("$scanner" version)" == 8.21.2 ]] || refuse PUBLICATION_SCANNER_VERSION
  local policy="$GITHUB_WORKSPACE/bullet-farm/publication/gitleaks.toml"
  [[ -f "$policy" && ! -L "$policy" ]] || refuse PUBLICATION_SCANNER_POLICY_REQUIRED
  cp "$policy" "$private/gitleaks.toml"
  : >"$private/gitleaks.ignore"
  if ! "$scanner" detect --source "$GITHUB_WORKSPACE" --no-git --redact=100 \
    --no-banner --config "$private/gitleaks.toml" \
    --gitleaks-ignore-path "$private/gitleaks.ignore" --ignore-gitleaks-allow \
    --report-format json --report-path "$private/source-scan.json" \
    >"$private/source-scan.log" 2>&1; then
    printf 'source scan failed; raw scanner output remains private\n' >"$report/status.txt"
    refuse PUBLICATION_SOURCE_SCAN_FAILED
  fi
  # Successful gitleaks emits only []; never publish finding payloads or snippets.
  [[ "$(tr -d '[:space:]' <"$private/source-scan.json")" == '[]' ]] \
    || refuse PUBLICATION_SCANNER_REPORT_INVALID
  printf '[]\n' >"$report/source-scan.json"
  printf 'gitleaks 8.21.2\nscanner_sha256 %s\nscanner_policy_sha256 %s\n' \
    '50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359' \
    "$(sha256sum "$private/gitleaks.toml" | cut -d ' ' -f 1)" >"$report/toolchain.txt"
  printf 'aggregate current-source scan passed\n' >"$report/status.txt"
}

prove() {
  admit_runner
  [[ -d "$report" && ! -L "$report" && -d "$private" && ! -L "$private" \
    && -f "$report/source-scan.json" && ! -L "$report/source-scan.json" \
    && "$(cat "$report/source-scan.json")" == '[]' ]] \
    || refuse PUBLICATION_SOURCE_SCAN_REQUIRED
  umask 077
  local cargo_home="$RUNNER_TEMP/bullet-publication-cargo"
  local target="$RUNNER_TEMP/bullet-publication-target"
  local build_root="$RUNNER_TEMP/bullet-publication-build"
  local split_root="$RUNNER_TEMP/bullet-publication-family"
  local cargo_bin rustc_bin rust_bin config_parent
  mkdir "$cargo_home" "$target" "$build_root"
  printf '[build]\njobs = 2\n[net]\nretry = 2\n' >"$cargo_home/config.toml"
  cp "$cargo_home/config.toml" "$report/cargo-config.toml"
  # Cargo reads configuration from the invocation directory and its ancestors.
  config_parent="$build_root"
  while :; do
    [[ ! -e "$config_parent/.cargo/config" && ! -L "$config_parent/.cargo/config" \
      && ! -e "$config_parent/.cargo/config.toml" && ! -L "$config_parent/.cargo/config.toml" ]] \
      || refuse PUBLICATION_AMBIENT_CARGO_CONFIG
    [[ "$config_parent" != / ]] || break
    config_parent="$(dirname "$config_parent")"
  done
  cargo_bin="$(rustup which --toolchain 1.95.0 cargo)"
  rustc_bin="$(rustup which --toolchain 1.95.0 rustc)"
  [[ -f "$cargo_bin" && -x "$cargo_bin" && -f "$rustc_bin" && -x "$rustc_bin" ]] \
    || refuse PUBLICATION_RUST_TOOLCHAIN_INVALID
  rust_bin="$(dirname "$cargo_bin")"
  "$cargo_bin" --version >>"$report/toolchain.txt"
  "$rustc_bin" --version >>"$report/toolchain.txt"
  [[ "$("$cargo_bin" --version)" == 'cargo 1.95.0 '* \
    && "$("$rustc_bin" --version)" == 'rustc 1.95.0 '* ]] \
    || refuse PUBLICATION_RUST_TOOLCHAIN_INVALID
  local -a cargo_env=(env -i "HOME=${HOME:?}" "PATH=$rust_bin:/usr/bin:/bin"
    "LC_ALL=C" "TZ=UTC" "CARGO_HOME=$cargo_home" "CARGO_TARGET_DIR=$target"
    "RUSTC=$rustc_bin" "BULLET_PUBLICATION_GITLEAKS=$(command -v gitleaks)")
  cd "$build_root"
  printf 'publication verifier build started; bootstrap incomplete\n' >"$report/status.txt"
  "${cargo_env[@]}" "$cargo_bin" build --locked \
    --manifest-path "$GITHUB_WORKSPACE/bullet-farm/Cargo.toml" --bin bullet-publish \
    >"$private/build.log" 2>&1 || refuse PUBLICATION_BUILD_FAILED
  printf 'publication tests started; bootstrap incomplete\n' >"$report/status.txt"
  "${cargo_env[@]}" "$cargo_bin" test --locked \
    --manifest-path "$GITHUB_WORKSPACE/bullet-farm/Cargo.toml" -p bullet-family \
    --lib publication:: -- --test-threads=1 --format=pretty --color=never \
    >"$private/publication-tests.log" 2>&1 \
    || refuse PUBLICATION_TESTS_FAILED
  test_inventory "$private/publication-tests.log"
  cp "$private/publication-tests.log" "$report/publication-tests.log"
  BULLET_PUBLICATION_TEST_BIN="$target/debug/bullet-publish" \
    bash "$GITHUB_WORKSPACE/bullet-farm/publication/ci-tests.sh" \
    >"$private/wrapper-tests.log" 2>&1 || refuse PUBLICATION_WRAPPER_TESTS_FAILED
  wrapper_inventory "$private/wrapper-tests.log"
  cp "$private/wrapper-tests.log" "$report/wrapper-tests.log"
  printf 'aggregate verification started; bootstrap incomplete\n' >"$report/status.txt"
  "$target/debug/bullet-publish" verify "$GITHUB_WORKSPACE" >"$report/verify.log"
  printf 'exact split reconstruction started; bootstrap incomplete\n' >"$report/status.txt"
  "$target/debug/bullet-publish" reconstruct "$GITHUB_WORKSPACE" "$split_root" \
    >"$report/reconstruct.log"
  printf 'publication proof completed; typed observation pending; full member/family campaigns remain required\n' \
    >"$report/status.txt"
  "$target/debug/bullet-publish" ci-observe "$GITHUB_WORKSPACE" "$split_root" "$report"
  [[ -f "${GITHUB_OUTPUT:-}" && ! -L "$GITHUB_OUTPUT" ]] || refuse PUBLICATION_OUTPUT_INVALID
  printf 'bootstrap_completion_sha256=%s\n' \
    "$(sha256sum "$report/observation.json" | cut -d ' ' -f 1)" >>"$GITHUB_OUTPUT"
}

case "${1:-}" in
  test-inventory) [[ "$#" == 2 ]] || refuse PUBLICATION_CI_USAGE; test_inventory "$2" ;;
  wrapper-inventory) [[ "$#" == 2 ]] || refuse PUBLICATION_CI_USAGE; wrapper_inventory "$2" ;;
  source-scan) [[ "$#" == 1 ]] || refuse PUBLICATION_CI_USAGE; source_scan ;;
  prove) [[ "$#" == 1 ]] || refuse PUBLICATION_CI_USAGE; prove ;;
  member-source-scan)
    [[ "$#" == 1 ]] || refuse PUBLICATION_CI_USAGE
    bash "$(dirname "${BASH_SOURCE[0]}")/ci-required.sh" member-run
    ;;
  *) refuse PUBLICATION_CI_USAGE ;;
esac
