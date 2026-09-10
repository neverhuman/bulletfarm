#!/usr/bin/env bash
# Prepare a root-owned immutable Rust toolchain for /usr/lib/bullet/toolchains,
# mirroring how /usr/lib/bullet/providers/claude/<version> is staged.
#
# This script NEVER runs a privileged step. It builds the whole tree under
# $HOME, writes a passport manifest beside it, and PRINTS the exact sudo
# commands for the operator to review and run. It calls no sudo, no install
# into /usr, and no provider CLI.
set -euo pipefail
umask 022

usage() {
  cat <<'EOF'
usage: stage-rust-toolchain.sh [--toolchain 1.97.1] [--host x86_64-unknown-linux-gnu]
                               [--toolchain-root <abs rustup toolchain dir>]
                               [--staging-root <abs, absent>]
                               [--install-root /usr/lib/bullet/toolchains/rust]
                               [--member <abs cargo workspace>]...
                               [--skip-cargo-fetch] [--cargo-home-max-gib 2]
       stage-rust-toolchain.sh --help

Stages <install-root>/<toolchain>/{bin,cargo-home,rustup-home} under $HOME:

  rustup-home/toolchains/<toolchain>-<host>/   the one real copy of the toolchain
  bin/{cargo,rustc,rustdoc,rustfmt,cargo-clippy,clippy-driver}
                                               tiny exec wrappers that pin
                                               RUSTUP_HOME and RUSTUP_TOOLCHAIN
  cargo-home/                                  offline registry populated by
                                               `cargo fetch --locked` for each
                                               --member (default: the three
                                               family Rust workspaces)

Writes <staging-root>/<toolchain>.passport.json (file count, aggregate bytes,
sha256 of bin/cargo and bin/rustc and of the real toolchain cargo/rustc, and
the exact `rustc --version` output), mirroring
/usr/lib/bullet/providers/claude/2.1.266.passport.json, which sits beside the
version directory rather than inside it.

Then prints the exact `sudo /usr/bin/cp -a`, `sudo /usr/bin/chown -R root:root`
and `sudo /usr/bin/chmod -R a-w` commands. Nothing privileged is executed.

cargo-home is included only when `cargo fetch --locked` succeeds for every
member and the result is at most --cargo-home-max-gib. `cargo fetch` needs the
network; when it fails the tree is still staged, cargo-home is reported as
excluded, and the printed commands leave it out.

Refusals are printed as DOGFOOD_OPS_<CODE>: <value> on stderr with exit 1.
EOF
}

refuse() {
  printf 'DOGFOOD_OPS_%s: %s\n' "$1" "$2" >&2
  exit 1
}

toolchain="1.97.1"
host="x86_64-unknown-linux-gnu"
toolchain_root=""
staging_root=""
install_root="/usr/lib/bullet/toolchains/rust"
skip_cargo_fetch=0
cargo_home_max_gib=2
declare -a members=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --toolchain) toolchain="${2:-}"; shift 2 ;;
    --host) host="${2:-}"; shift 2 ;;
    --toolchain-root) toolchain_root="${2:-}"; shift 2 ;;
    --staging-root) staging_root="${2:-}"; shift 2 ;;
    --install-root) install_root="${2:-}"; shift 2 ;;
    --member) members+=("${2:-}"); shift 2 ;;
    --skip-cargo-fetch) skip_cargo_fetch=1; shift ;;
    --cargo-home-max-gib) cargo_home_max_gib="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) refuse ARG_UNKNOWN "$1" ;;
  esac
done

for tool in jq sha256sum stat realpath cp find du date; do
  command -v "$tool" >/dev/null 2>&1 || refuse TOOL_MISSING "$tool"
done

[[ "$toolchain" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]] || refuse TOOLCHAIN_INVALID "$toolchain"
[[ "$host" =~ ^[a-z0-9_]+-[a-z0-9_]+-[a-z0-9_]+(-[a-z0-9_]+)?$ ]] || refuse HOST_INVALID "$host"
[[ "$install_root" == /* ]] || refuse INSTALL_ROOT_NOT_ABSOLUTE "$install_root"
[[ "$cargo_home_max_gib" =~ ^[0-9]+$ && "$cargo_home_max_gib" -ge 1 ]] \
  || refuse CARGO_HOME_MAX_INVALID "$cargo_home_max_gib"

toolchain_name="$toolchain-$host"
[[ -n "$toolchain_root" ]] || toolchain_root="$HOME/.rustup/toolchains/$toolchain_name"
[[ "$toolchain_root" == /* ]] || refuse TOOLCHAIN_ROOT_NOT_ABSOLUTE "$toolchain_root"
[[ -d "$toolchain_root" && ! -L "$toolchain_root" ]] || refuse TOOLCHAIN_ROOT_MISSING "$toolchain_root"
[[ "$(realpath -e -- "$toolchain_root")" == "$toolchain_root" ]] \
  || refuse TOOLCHAIN_ROOT_NOT_CANONICAL "$toolchain_root"
for required in bin/cargo bin/rustc lib; do
  [[ -e "$toolchain_root/$required" ]] || refuse TOOLCHAIN_ROOT_INCOMPLETE "$toolchain_root/$required"
done

if [[ ${#members[@]} -eq 0 ]]; then
  for candidate in /home/ubuntu/bullet/bullet-kernel /home/ubuntu/bullet/bullet-farm /home/ubuntu/bullet/bullet-git; do
    [[ -f "$candidate/Cargo.toml" ]] && members+=("$candidate")
  done
fi
for member in ${members[@]+"${members[@]}"}; do
  [[ "$member" == /* ]] || refuse MEMBER_NOT_ABSOLUTE "$member"
  [[ -f "$member/Cargo.toml" ]] || refuse MEMBER_NOT_A_WORKSPACE "$member/Cargo.toml"
  [[ -f "$member/Cargo.lock" ]] || refuse MEMBER_LOCK_MISSING "$member/Cargo.lock (--locked needs it)"
done

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
[[ -n "$staging_root" ]] || staging_root="$HOME/bullet-toolchain-stage-$stamp"
[[ "$staging_root" == /* ]] || refuse STAGING_ROOT_NOT_ABSOLUTE "$staging_root"
[[ "$staging_root" != /home/ubuntu/bullet/* ]] \
  || refuse STAGING_ROOT_IN_FAMILY_TREE "$staging_root must live outside the family repos"
[[ ! -e "$staging_root" && ! -L "$staging_root" ]] || refuse STAGING_ROOT_EXISTS "$staging_root"
staging_parent="$(dirname -- "$staging_root")"
[[ -d "$staging_parent" ]] || refuse STAGING_PARENT_MISSING "$staging_parent"

mkdir -m 0700 -- "$staging_root"
deployment_root="$staging_root/$toolchain"
mkdir -m 0755 -- "$deployment_root" "$deployment_root/bin" "$deployment_root/rustup-home" \
  "$deployment_root/rustup-home/toolchains"

printf 'staging_root=%s\n' "$staging_root"
printf 'toolchain_root=%s\n' "$toolchain_root"
printf 'copying toolchain (this reads ~1 GiB)...\n'
cp -a -- "$toolchain_root" "$deployment_root/rustup-home/toolchains/$toolchain_name"

# rustup resolves a toolchain through settings.toml; pin the staged one.
cat >"$deployment_root/rustup-home/settings.toml" <<TOML
version = "12"
default_toolchain = "$toolchain_name"

[overrides]
TOML

installed_root="$install_root/$toolchain"
real_bin="$installed_root/rustup-home/toolchains/$toolchain_name/bin"
for entry in cargo rustc rustdoc rustfmt cargo-clippy clippy-driver cargo-fmt; do
  [[ -x "$toolchain_root/bin/$entry" ]] || continue
  cat >"$deployment_root/bin/$entry" <<WRAPPER
#!/bin/sh
# Immutable staged Rust $toolchain. CARGO_HOME defaults to the read-only
# offline cache staged beside this wrapper; a gate that needs to write (cargo
# takes a package-cache lock even with --offline) must export its own writable
# CARGO_HOME seeded from that directory.
set -eu
RUSTUP_HOME="$installed_root/rustup-home"
RUSTUP_TOOLCHAIN="$toolchain_name"
CARGO_HOME="\${CARGO_HOME:-$installed_root/cargo-home}"
export RUSTUP_HOME RUSTUP_TOOLCHAIN CARGO_HOME
exec "$real_bin/$entry" "\$@"
WRAPPER
  chmod 0555 -- "$deployment_root/bin/$entry"
done

# --- offline cargo home -----------------------------------------------------
cargo_home="$deployment_root/cargo-home"
cargo_home_state="EXCLUDED"
cargo_home_reason="not attempted"
cargo_home_bytes=0
if [[ "$skip_cargo_fetch" -eq 1 ]]; then
  cargo_home_reason="--skip-cargo-fetch"
elif ! command -v cargo >/dev/null 2>&1; then
  cargo_home_reason="cargo is not on PATH"
else
  mkdir -m 0755 -- "$cargo_home"
  fetch_log="$staging_root/cargo-fetch.log"
  : >"$fetch_log"
  fetch_ok=1
  for member in ${members[@]+"${members[@]}"}; do
    # --locked must not rewrite the member's lock file; the family repos are
    # other agents' working trees and this lane never writes them.
    before="$(sha256sum -- "$member/Cargo.lock" | cut -d' ' -f1)"
    printf '=== cargo fetch --locked %s\n' "$member" >>"$fetch_log"
    if ! CARGO_HOME="$cargo_home" RUSTUP_HOME="$HOME/.rustup" RUSTUP_TOOLCHAIN="$toolchain_name" \
      cargo fetch --locked --manifest-path "$member/Cargo.toml" >>"$fetch_log" 2>&1; then
      fetch_ok=0
      cargo_home_reason="cargo fetch --locked failed for $member (see $fetch_log)"
      printf 'DOGFOOD_OPS_CARGO_FETCH_FAILED: %s\n' "$member" >&2
    fi
    after="$(sha256sum -- "$member/Cargo.lock" | cut -d' ' -f1)"
    [[ "$before" == "$after" ]] || refuse MEMBER_LOCK_MUTATED "$member/Cargo.lock changed during fetch"
    [[ "$fetch_ok" -eq 1 ]] || break
  done
  if [[ "$fetch_ok" -eq 1 ]]; then
    cargo_home_bytes="$(du -sb -- "$cargo_home" | cut -f1)"
    if [[ "$cargo_home_bytes" -gt $((cargo_home_max_gib * 1024 * 1024 * 1024)) ]]; then
      cargo_home_state="EXCLUDED"
      cargo_home_reason="$cargo_home_bytes bytes exceeds ${cargo_home_max_gib} GiB; not installed by default"
    else
      cargo_home_state="INCLUDED"
      cargo_home_reason="cargo fetch --locked for ${#members[@]} member(s)"
    fi
  fi
fi
[[ -d "$cargo_home" ]] && cargo_home_bytes="$(du -sb -- "$cargo_home" | cut -f1)"

# --- passport manifest ------------------------------------------------------
file_count="$(find "$deployment_root" -type f | wc -l)"
aggregate_bytes="$(du -sb -- "$deployment_root" | cut -f1)"
wrapper_cargo_sha="$(sha256sum -- "$deployment_root/bin/cargo" | cut -d' ' -f1)"
wrapper_rustc_sha="$(sha256sum -- "$deployment_root/bin/rustc" | cut -d' ' -f1)"
real_cargo="$deployment_root/rustup-home/toolchains/$toolchain_name/bin/cargo"
real_rustc="$deployment_root/rustup-home/toolchains/$toolchain_name/bin/rustc"
real_cargo_sha="$(sha256sum -- "$real_cargo" | cut -d' ' -f1)"
real_rustc_sha="$(sha256sum -- "$real_rustc" | cut -d' ' -f1)"
rustc_version="$("$toolchain_root/bin/rustc" --version 2>/dev/null || true)"
[[ -n "$rustc_version" ]] || refuse RUSTC_VERSION_UNAVAILABLE "$toolchain_root/bin/rustc"

passport="$staging_root/$toolchain.passport.json"
jq -S -n \
  --arg schema "bullet.rust-toolchain-staging.v1" \
  --arg toolchain "$toolchain" --arg host "$host" --arg name "$toolchain_name" \
  --arg source "$toolchain_root" --arg deployment_root "$installed_root" \
  --arg staged_at "$staging_root" \
  --argjson file_count "$file_count" --argjson aggregate_size_bytes "$aggregate_bytes" \
  --arg wrapper_cargo "$wrapper_cargo_sha" --arg wrapper_rustc "$wrapper_rustc_sha" \
  --arg real_cargo "$real_cargo_sha" --arg real_rustc "$real_rustc_sha" \
  --arg rustc_version "$rustc_version" \
  --arg cargo_home_state "$cargo_home_state" --arg cargo_home_reason "$cargo_home_reason" \
  --argjson cargo_home_bytes "$cargo_home_bytes" \
  --argjson members "$(printf '%s\n' ${members[@]+"${members[@]}"} | jq -R . | jq -cs .)" '
  {schema_version:$schema, kind:"rust-toolchain", toolchain:$toolchain, host:$host,
   toolchain_name:$name, source_root:$source, deployment_root:$deployment_root,
   staged_root:$staged_at, entrypoints:["bin/cargo","bin/rustc"],
   aggregate_file_count:$file_count, aggregate_size_bytes:$aggregate_size_bytes,
   sha256:{"bin/cargo":$wrapper_cargo,"bin/rustc":$wrapper_rustc,
     "rustup-home/bin/cargo":$real_cargo,"rustup-home/bin/rustc":$real_rustc},
   rustc_version:$rustc_version,
   cargo_home:{state:$cargo_home_state, reason:$cargo_home_reason,
     size_bytes:$cargo_home_bytes, members:$members},
   privileged_install:"NOT_PERFORMED_BY_THIS_SCRIPT",
   evidence_class:"OPERATOR_STAGING", release_gate_eligible:false}
' >"$passport"
chmod 0444 -- "$passport"

printf '\npassport=%s\n' "$passport"
printf 'deployment_root=%s\n' "$deployment_root"
printf 'aggregate_file_count=%s\n' "$file_count"
printf 'aggregate_size_bytes=%s\n' "$aggregate_bytes"
printf 'rustc_version=%s\n' "$rustc_version"
printf 'cargo_home=%s size_bytes=%s reason=%s\n' "$cargo_home_state" "$cargo_home_bytes" "$cargo_home_reason"

if [[ "$cargo_home_state" != INCLUDED && -d "$cargo_home" ]]; then
  printf '\nNOTE: cargo-home stays under %s and is NOT in the printed install commands.\n' "$cargo_home"
fi

printf '\n--- operator commands (review, then run; this script ran none of them) ---\n'
printf 'sudo /usr/bin/install -d -m 0555 -o root -g root %s\n' "$(dirname -- "$install_root")"
printf 'sudo /usr/bin/install -d -m 0555 -o root -g root %s\n' "$install_root"
if [[ "$cargo_home_state" == INCLUDED ]]; then
  printf 'sudo /usr/bin/cp -a %s %s\n' "$deployment_root" "$installed_root"
else
  printf 'sudo /usr/bin/cp -a %s %s\n' "$deployment_root" "$installed_root"
  printf 'sudo /usr/bin/rm -rf %s\n' "$installed_root/cargo-home"
fi
printf 'sudo /usr/bin/cp -a %s %s.passport.json\n' "$passport" "$installed_root"
printf 'sudo /usr/bin/chown -R root:root %s %s.passport.json\n' "$installed_root" "$installed_root"
printf 'sudo /usr/bin/chmod -R a-w %s %s.passport.json\n' "$installed_root" "$installed_root"
printf 'sudo /usr/bin/find %s -type d -exec /usr/bin/chmod 0555 {} +\n' "$installed_root"
printf 'sudo /usr/bin/find %s -type f -perm -u+x -exec /usr/bin/chmod 0555 {} +\n' "$installed_root"
printf 'sudo /usr/bin/chmod 0444 %s.passport.json\n' "$installed_root"
printf '\nverify afterwards:\n'
printf '  /usr/bin/stat -c "%%n %%U:%%G %%a" %s %s.passport.json\n' "$installed_root" "$installed_root"
printf '  %s/bin/rustc --version   # expect: %s\n' "$installed_root" "$rustc_version"
printf '\nnot proven by this staging: that any gate runs against it, that the\n'
printf 'toolchain bytes are reproducible, or that cargo can take its package-cache\n'
printf 'lock inside a read-only CARGO_HOME.\n'
