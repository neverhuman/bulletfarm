#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/rust-toolchain-boundary-test.sh"
lib_path="$script_dir/lib.sh"
# shellcheck source=ops/ci/lib.sh
source "$lib_path"
repo_root="$REPO_ROOT"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/bullet-disallowed-methods.XXXXXX")"
trap 'rm -rf -- "$fixture_root"' EXIT

boundary_root="$fixture_root/compiler-boundary"
mkdir -p "$boundary_root/.cargo" "$boundary_root/crate"
printf '%s\n' '[build]' 'rustc-workspace-wrapper = "selective-wrapper"' \
  >"$boundary_root/.cargo/config.toml"
if enforce_rust_compiler_boundary "$boundary_root" >"$fixture_root/config.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: workspace Cargo config was admitted" >&2
  exit 1
fi
grep -Fq CARGO_CONFIG_FORBIDDEN "$fixture_root/config.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: config failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$boundary_root/.cargo/config.toml"

custom_cargo_home="$fixture_root/custom-cargo-home"
mkdir -p "$custom_cargo_home"
if CARGO_HOME="$custom_cargo_home" enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/cargo-home.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: custom CARGO_HOME was admitted" >&2
  exit 1
fi
grep -Fq CARGO_HOME_FORBIDDEN "$fixture_root/cargo-home.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: CARGO_HOME failed for an unrelated reason" >&2
  exit 1
}
if ! CARGO_HOME="${HOME%/}/.cargo" enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/default-cargo-home.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: default CARGO_HOME was refused" >&2
  cat "$fixture_root/default-cargo-home.log" >&2
  exit 1
fi

if CARGO_ALIAS_CLIPPY=metadata enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/cargo-alias.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: Cargo Clippy alias was admitted" >&2
  exit 1
fi
grep -Fq RUST_COMPILER_CONTROL_ENV "$fixture_root/cargo-alias.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: Cargo alias failed for an unrelated reason" >&2
  exit 1
}
if cargo_alias_clippy=metadata \
  enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/cargo-alias-case.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: case-variant Cargo alias was admitted" >&2
  exit 1
fi
grep -Fq RUST_COMPILER_CONTROL_ENV "$fixture_root/cargo-alias-case.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: case-variant alias failed for an unrelated reason" >&2
  exit 1
}
for hostile_name in RuStC RuStFlAgS CaRgO_TaRgEt_X86_64_UnKnOwN_LiNuX_GnU_LiNkEr \
  ClIpPy_CoNf_DiR RuStFmT_ArGs; do
  export "$hostile_name=hostile-control"
  if enforce_rust_compiler_boundary "$repo_root" \
    >"$fixture_root/mixed-control.log" 2>&1; then
    echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: mixed-case $hostile_name was admitted" >&2
    exit 1
  fi
  unset "$hostile_name"
  grep -Fq RUST_COMPILER_CONTROL_ENV "$fixture_root/mixed-control.log" || {
    echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: mixed-case control failed for an unrelated reason" >&2
    exit 1
  }
done

hostile_home="$fixture_root/hostile-home"
mkdir -p "$hostile_home/.cargo"
printf '%s\n' '"build"."rustc-workspace-wrapper" = "selective-wrapper"' \
  >"$hostile_home/.cargo/config.toml"
if HOME="$hostile_home" CARGO_HOME="$hostile_home/.cargo" \
  enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/default-home-config.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: default Cargo-home compiler control was admitted" >&2
  exit 1
fi
grep -Fq CARGO_CONFIG_CONTROL_FORBIDDEN "$fixture_root/default-home-config.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: default Cargo-home config failed for an unrelated reason" >&2
  exit 1
}
printf '%s\n' 'paths = ["../substituted-dependency"]' \
  >"$hostile_home/.cargo/config.toml"
if HOME="$hostile_home" enforce_rust_compiler_boundary "$repo_root" \
  >"$fixture_root/path-override.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: Cargo dependency path override was admitted" >&2
  exit 1
fi
grep -Fq CARGO_CONFIG_CONTROL_FORBIDDEN "$fixture_root/path-override.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: path override failed for an unrelated reason" >&2
  exit 1
}

printf '%s\n' 'fn main() {}' >"$boundary_root/crate/build.rs"
if enforce_rust_compiler_boundary "$boundary_root" >"$fixture_root/build-rs.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: build.rs was admitted" >&2
  exit 1
fi
grep -Fq CARGO_BUILD_SCRIPT_FORBIDDEN "$fixture_root/build-rs.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: build.rs failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$boundary_root/crate/build.rs"

printf '%s\n' 'fn main() {}' >"$boundary_root/crate/probe.rs"
ln -s probe.rs "$boundary_root/crate/BUILD.RS"
if enforce_rust_compiler_boundary "$boundary_root" \
  >"$fixture_root/build-rs-symlink.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: case-variant symlinked build.rs was admitted" >&2
  exit 1
fi
grep -Fq CARGO_BUILD_SCRIPT_FORBIDDEN "$fixture_root/build-rs-symlink.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: symlinked build.rs failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$boundary_root/crate/BUILD.RS" "$boundary_root/crate/probe.rs"

printf '%s\n' '[package]' 'name = "hostile-build"' 'version = "0.0.0"' \
  'build = "selective.rs"' >"$boundary_root/crate/Cargo.toml"
if enforce_rust_compiler_boundary "$boundary_root" >"$fixture_root/build-key.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: manifest build key was admitted" >&2
  exit 1
fi
grep -Fq CARGO_MANIFEST_BUILD_SCRIPT_FORBIDDEN "$fixture_root/build-key.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: manifest build key failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$boundary_root/crate/Cargo.toml"

for package_build in \
  $'[package]\n"build" = "selective.rs"\n' \
  $'package.build = "selective.rs"\n'; do
  printf '%s' "$package_build" >"$boundary_root/crate/Cargo.toml"
  if enforce_rust_compiler_boundary "$boundary_root" \
    >"$fixture_root/build-key-toml.log" 2>&1; then
    echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: TOML package.build was admitted" >&2
    exit 1
  fi
  grep -Fq CARGO_MANIFEST_BUILD_SCRIPT_FORBIDDEN \
    "$fixture_root/build-key-toml.log" || {
    echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: TOML build key failed for an unrelated reason" >&2
    exit 1
  }
done
rm -f -- "$boundary_root/crate/Cargo.toml"

wrapper_marker="$fixture_root/selective-wrapper-ran"
# shellcheck disable=SC2016 # literal source for the hostile wrapper fixture
printf '%s\n' '#!/usr/bin/env bash' \
  'compiler="$1"' 'shift' \
  'case " $* " in *" bullet_wire "*) : >"${BULLET_WRAPPER_MARKER:?}"; exec "$compiler" "$@" --cap-lints allow ;; esac' \
  'exec "$compiler" "$@"' >"$boundary_root/selective-wrapper"
chmod +x "$boundary_root/selective-wrapper"
if RUSTC_WORKSPACE_WRAPPER="$boundary_root/selective-wrapper" \
  BULLET_WRAPPER_MARKER="$wrapper_marker" \
  enforce_rust_compiler_boundary "$repo_root" >"$fixture_root/wrapper.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: selective workspace wrapper was admitted" >&2
  exit 1
fi
grep -Fq RUST_COMPILER_CONTROL_ENV "$fixture_root/wrapper.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: wrapper failed for an unrelated reason" >&2
  exit 1
}
[[ ! -e "$wrapper_marker" ]] || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: selective workspace wrapper executed" >&2
  exit 1
}

formatter_marker="$fixture_root/formatter-ran"
# shellcheck disable=SC2016 # literal source for the hostile formatter fixture
printf '%s\n' '#!/usr/bin/env bash' \
  ': >"${BULLET_FORMATTER_MARKER:?}"' >"$boundary_root/hostile-rustfmt"
chmod +x "$boundary_root/hostile-rustfmt"
if RUSTFMT="$boundary_root/hostile-rustfmt" \
  BULLET_FORMATTER_MARKER="$formatter_marker" \
  enforce_rust_compiler_boundary "$repo_root" >"$fixture_root/rustfmt.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: formatter override was admitted" >&2
  exit 1
fi
grep -Fq RUST_COMPILER_CONTROL_ENV "$fixture_root/rustfmt.log" || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: formatter override failed for an unrelated reason" >&2
  exit 1
}
[[ ! -e "$formatter_marker" ]] || {
  echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: hostile formatter executed" >&2
  exit 1
}

bash "$script_dir/rust-build-subject-test.sh"

serde_json_version="$({
  awk '
    $0 == "name = \"serde_json\"" { found = 1; next }
    found && /^version = / {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "$repo_root/Cargo.lock"
})"
[[ -n "$serde_json_version" ]] || {
  echo "DISALLOWED_METHODS_CANARY_INVALID: serde_json is absent from Cargo.lock" >&2
  exit 1
}

mkdir -p "$fixture_root/src"
cp "$repo_root/clippy.toml" "$fixture_root/clippy.toml"
cat >"$fixture_root/Cargo.toml" <<EOF
[package]
name = "bullet-disallowed-methods-canary"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
serde_json = "=$serde_json_version"

[workspace]
EOF
run_case() {
  local case_name="$1"
  case "$case_name" in
    resolved-alias)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub use serde_json::from_slice as parse_json;
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
use bullet_disallowed_methods_canary::parse_json;
fn main() { let _ = parse_json::<serde_json::Value>(br#"{}"#); }
EOF
      ;;
    from-str)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub fn decoder(text: &str) { let _ = serde_json::from_str::<serde_json::Value>(text); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder("{}"); }
EOF
      ;;
    from-reader)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub fn decoder(bytes: &[u8]) { let _ = serde_json::from_reader::<_, serde_json::Value>(bytes); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    deserializer-from-str)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub fn decoder(text: &str) { let _ = serde_json::Deserializer::from_str(text); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder("{}"); }
EOF
      ;;
    deserializer-from-slice)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub fn decoder(bytes: &[u8]) { let _ = serde_json::Deserializer::from_slice(bytes); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    deserializer-from-reader)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub fn decoder(bytes: &[u8]) { let _ = serde_json::Deserializer::from_reader(bytes); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    deserializer-new)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub use serde_json::de::{Deserializer as JsonDecoder, SliceRead as JsonSlice};
pub fn decoder(bytes: &[u8]) { let _ = JsonDecoder::new(JsonSlice::new(bytes)); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    stream-new)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
pub use serde_json::{Value as JsonValue, de::{SliceRead as JsonSlice, StreamDeserializer as JsonStream}};
pub fn decoder(bytes: &[u8]) { let _ = JsonStream::<_, JsonValue>::new(JsonSlice::new(bytes)); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    generic-parse)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
use std::str::FromStr;
pub fn parse<T: FromStr>(text: &str) -> Result<T, T::Err> { text.parse() }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { let _ = bullet_disallowed_methods_canary::parse::<serde_json::Value>("{}"); }
EOF
      ;;
    generic-from-str)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
use std::str::FromStr;
pub fn parse<T: FromStr>(text: &str) -> Result<T, T::Err> { <T as FromStr>::from_str(text) }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { let _ = bullet_disallowed_methods_canary::parse::<serde_json::Value>("{}"); }
EOF
      ;;
    macro-parse)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
use std::str::FromStr;
macro_rules! dispatch { ($text:expr) => { $text.parse() } }
pub fn parse<T: FromStr>(text: &str) -> Result<T, T::Err> { dispatch!(text) }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { let _ = bullet_disallowed_methods_canary::parse::<serde_json::Value>("{}"); }
EOF
      ;;
    macro-from-str)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
use std::str::FromStr;
macro_rules! dispatch { ($type:ty, $text:expr) => { <$type as FromStr>::from_str($text) } }
pub fn parse<T: FromStr>(text: &str) -> Result<T, T::Err> { dispatch!(T, text) }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { let _ = bullet_disallowed_methods_canary::parse::<serde_json::Value>("{}"); }
EOF
      ;;
    local-allow)
      cat >"$fixture_root/src/lib.rs" <<'EOF'
#[allow(clippy::disallowed_methods)]
pub fn decoder(bytes: &[u8]) { let _ = serde_json::from_slice::<serde_json::Value>(bytes); }
EOF
      cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() { bullet_disallowed_methods_canary::decoder(br#"{}"#); }
EOF
      ;;
    *)
      echo "DISALLOWED_METHODS_CANARY_INVALID: unknown case $case_name" >&2
      exit 1
      ;;
  esac

  set +e
  CARGO_TARGET_DIR="$fixture_root/target" cargo clippy \
    --offline \
    --manifest-path "$fixture_root/Cargo.toml" \
    -- \
    -F clippy::disallowed_methods >"$fixture_root/$case_name.log" 2>&1
  status=$?
  set -e
  [[ "$status" -ne 0 ]] || {
    echo "DISALLOWED_METHODS_CANARY_FAILED: $case_name passed Clippy" >&2
    exit 1
  }
  diagnostic_pattern="use of a disallowed method"
  [[ "$case_name" == "local-allow" ]] \
    && diagnostic_pattern="incompatible with previous forbid"
  grep -Fq "$diagnostic_pattern" "$fixture_root/$case_name.log" || {
    echo "DISALLOWED_METHODS_CANARY_INVALID: $case_name failed for an unrelated reason" >&2
    sed -n '1,120p' "$fixture_root/$case_name.log" >&2
    exit 1
  }
}

for case_name in \
  from-str \
  from-reader \
  resolved-alias \
  deserializer-from-str \
  deserializer-from-slice \
  deserializer-from-reader \
  deserializer-new \
  stream-new \
  generic-parse \
  generic-from-str \
  macro-parse \
  macro-from-str \
  local-allow; do
  run_case "$case_name"
done

cat >"$fixture_root/src/lib.rs" <<'EOF'
use std::str::FromStr;

pub struct ExactScalar(u64);

impl ExactScalar {
    pub fn parse_checked(text: &str) -> Option<Self> {
        text.bytes().try_fold(0_u64, |number, byte| {
            byte.is_ascii_digit()
                .then_some(byte - b'0')
                .and_then(|digit| number.checked_mul(10)?.checked_add(u64::from(digit)))
        }).map(Self)
    }

    pub fn value(&self) -> u64 { self.0 }
}

impl FromStr for ExactScalar {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse_checked(text).ok_or(())
    }
}
EOF
cat >"$fixture_root/src/main.rs" <<'EOF'
fn main() {
    let value = bullet_disallowed_methods_canary::ExactScalar::parse_checked("17").unwrap();
    assert_eq!(value.value(), 17);
}
EOF
CARGO_TARGET_DIR="$fixture_root/target" cargo clippy \
  --offline \
  --manifest-path "$fixture_root/Cargo.toml" \
  -- \
  -D clippy::disallowed_methods >"$fixture_root/typed-constructor.log" 2>&1 || {
  echo "DISALLOWED_METHODS_CANARY_INVALID: exact typed constructor was rejected" >&2
  sed -n '1,120p' "$fixture_root/typed-constructor.log" >&2
  exit 1
}

echo "disallowed methods canary: PASS"
