#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="$HUB/docs/readme-media"
case_name="${1:-all}"
if [[ "$#" -gt 1 || ! "$case_name" =~ ^(all|checker|publisher)$ ]]; then
  echo "usage: $0 [all|checker|publisher]" >&2
  exit 2
fi

for tool in cmp cp docker find grep ln mktemp mv sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-custody-test: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

test_tmp="$(mktemp -d)"
cleanup() {
  if [[ -n "${checker_pid:-}" ]]; then
    kill "$checker_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "${publisher_pid:-}" ]]; then
    kill "$publisher_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$test_tmp"
}
trap cleanup EXIT

copy_media() {
  local destination="$1"
  mkdir -p "$destination"
  cp -a --no-dereference "$MEDIA/." "$destination/"
}

wait_for_marker() {
  local marker="$1" pid="$2" label="$3"
  for _ in {1..4800}; do
    [[ -e "$marker" ]] && return 0
    kill -0 "$pid" >/dev/null 2>&1 || {
      printf 'readme-custody-test: %s exited before the attack window\n' "$label" >&2
      return 1
    }
    sleep 0.05
  done
  printf 'readme-custody-test: timed out waiting for %s attack window\n' "$label" >&2
  return 1
}

write_docker_gate() {
  local path="$1"
  # shellcheck disable=SC2016 # These variables expand when the generated wrapper executes.
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    'for argument in "$@"; do' \
    '  if [[ "$argument" == *"${README_GATE_MATCH}"* ]]; then' \
    '    if (set -o noclobber; : >"$README_GATE_MARKER") 2>/dev/null; then' \
    '      for attempt in {1..4800}; do' \
    '        [[ -e "$README_GATE_RELEASE" ]] && break' \
    '        sleep 0.05' \
    '      done' \
    '      [[ -e "$README_GATE_RELEASE" ]] || exit 97' \
    '    fi' \
    '    break' \
    '  fi' \
    'done' \
    'exec "$README_REAL_DOCKER" "$@"' >"$path"
  chmod 700 "$path"
}

write_mktemp_gate() {
  local path="$1"
  # shellcheck disable=SC2016 # These variables expand when the generated wrapper executes.
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'set -euo pipefail' \
    'for argument in "$@"; do' \
    '  if [[ "$argument" == *.fallback.png.publish.* ]]; then' \
    '    if (set -o noclobber; : >"$README_GATE_MARKER") 2>/dev/null; then' \
    '      for attempt in {1..4800}; do' \
    '        [[ -e "$README_GATE_RELEASE" ]] && break' \
    '        sleep 0.05' \
    '      done' \
    '      [[ -e "$README_GATE_RELEASE" ]] || exit 97' \
    '    fi' \
    '    break' \
    '  fi' \
    'done' \
    'exec "$README_REAL_MKTEMP" "$@"' >"$path"
  chmod 700 "$path"
}

checker_substitution_is_refused() {
  local root="$test_tmp/checker-media"
  local outside="$test_tmp/outside-manifest.json"
  local manifest="$root/component-preview/manifest.json"
  local marker="$test_tmp/checker-marker" release="$test_tmp/checker-release"
  local fake_bin="$test_tmp/checker-bin" log="$test_tmp/checker.log"
  local real_docker outside_sha checker_code
  real_docker="$(command -v docker)"
  copy_media "$root"
  cp "$manifest" "$outside"
  outside_sha="$(sha256sum "$outside" | awk '{print $1}')"
  mkdir -p "$fake_bin"
  write_docker_gate "$fake_bin/docker"

  set +e
  README_REAL_DOCKER="$real_docker" README_GATE_MATCH='--version' \
    README_GATE_MARKER="$marker" README_GATE_RELEASE="$release" \
    PATH="$fake_bin:$PATH" bash "$HUB/scripts/readme-check.sh" \
    --staged-root "$root" >"$log" 2>&1 &
  checker_pid=$!
  set -e
  wait_for_marker "$marker" "$checker_pid" checker
  mv "$manifest" "$manifest.pre-substitution"
  ln -s "$outside" "$manifest"
  : >"$release"
  set +e
  wait "$checker_pid"
  checker_code=$?
  set -e
  checker_pid=""

  [[ "$checker_code" -ne 0 ]] || {
    echo "readme-custody-test: checker accepted an after-preflight outside manifest symlink" >&2
    return 1
  }
  grep -Fq 'readme-check: media source gained a symlink during validation' "$log" || {
    echo "readme-custody-test: checker refused outside the source-custody path" >&2
    return 1
  }
  [[ "$(sha256sum "$outside" | awk '{print $1}')" == "$outside_sha" ]] || {
    echo "readme-custody-test: checker mutated the outside manifest" >&2
    return 1
  }
}

publisher_temps_preserve_outside_targets() {
  local root="$test_tmp/publisher-media" outside_dir="$test_tmp/publisher-outside"
  local marker="$test_tmp/publisher-marker" release="$test_tmp/publisher-release"
  local fake_bin="$test_tmp/publisher-bin" log="$test_tmp/publisher.log"
  local real_docker demo file sentinel expected publisher_code
  real_docker="$(command -v docker)"
  copy_media "$root"
  mkdir -p "$outside_dir" "$fake_bin"
  write_docker_gate "$fake_bin/docker"

  set +e
  README_REAL_DOCKER="$real_docker" README_GATE_MATCH=':/media:ro' \
    README_GATE_MARKER="$marker" README_GATE_RELEASE="$release" \
    README_MEDIA_ROOT="$root" PATH="$fake_bin:$PATH" \
    bash "$HUB/scripts/readme-render.sh" >"$log" 2>&1 &
  publisher_pid=$!
  set -e
  wait_for_marker "$marker" "$publisher_pid" publisher
  for demo in component-preview provider-safety; do
    for file in fallback.png "$demo.gif" frames.framemd5 manifest.json; do
      sentinel="$outside_dir/$demo-${file//\//_}"
      printf 'outside sentinel: %s/%s\n' "$demo" "$file" >"$sentinel"
      expected="$(sha256sum "$sentinel" | awk '{print $1}')"
      printf '%s\n' "$expected" >"$sentinel.sha256"
      ln -s "$sentinel" "$root/$demo/$file.tmp.$publisher_pid"
    done
  done
  : >"$release"
  set +e
  wait "$publisher_pid"
  publisher_code=$?
  set -e
  publisher_pid=""

  for demo in component-preview provider-safety; do
    for file in fallback.png "$demo.gif" frames.framemd5 manifest.json; do
      sentinel="$outside_dir/$demo-${file//\//_}"
      expected="$(<"$sentinel.sha256")"
      [[ "$(sha256sum "$sentinel" | awk '{print $1}')" == "$expected" ]] || {
        echo "readme-custody-test: publisher overwrote an outside target" >&2
        return 1
      }
      [[ -f "$root/$demo/$file" && ! -L "$root/$demo/$file" ]] || {
        echo "readme-custody-test: publisher installed a symlink" >&2
        return 1
      }
    done
  done
  [[ "$publisher_code" -eq 0 ]] || {
    printf 'readme-custody-test: repaired publisher refused safe random-temp publication (exit %s)\n' \
      "$publisher_code" >&2
    return 1
  }
}

publisher_parent_substitution_is_refused() {
  local root="$test_tmp/parent-media" outside_dir="$test_tmp/parent-outside"
  local retained_dir="$test_tmp/component-preview-retained"
  local marker="$test_tmp/parent-marker" release="$test_tmp/parent-release"
  local fake_bin="$test_tmp/parent-bin" log="$test_tmp/parent.log"
  local real_mktemp file publisher_code
  local -A outside_shas=()
  real_mktemp="$(command -v mktemp)"
  copy_media "$root"
  mkdir -p "$outside_dir" "$fake_bin"
  write_mktemp_gate "$fake_bin/mktemp"
  for file in fallback.png component-preview.gif frames.framemd5 manifest.json; do
    printf 'outside parent sentinel: %s\n' "$file" >"$outside_dir/$file"
    outside_shas["$file"]="$(sha256sum "$outside_dir/$file" | awk '{print $1}')"
  done

  set +e
  README_REAL_MKTEMP="$real_mktemp" README_GATE_MARKER="$marker" \
    README_GATE_RELEASE="$release" README_MEDIA_ROOT="$root" PATH="$fake_bin:$PATH" \
    bash "$HUB/scripts/readme-render.sh" >"$log" 2>&1 &
  publisher_pid=$!
  set -e
  wait_for_marker "$marker" "$publisher_pid" parent-publisher
  mv "$root/component-preview" "$retained_dir"
  ln -s "$outside_dir" "$root/component-preview"
  : >"$release"
  set +e
  wait "$publisher_pid"
  publisher_code=$?
  set -e
  publisher_pid=""

  [[ "$publisher_code" -ne 0 ]] || {
    echo "readme-custody-test: publisher accepted a substituted parent directory" >&2
    return 1
  }
  grep -Fq 'readme-render: publication directory identity changed:' "$log" || {
    echo "readme-custody-test: publisher refused outside the directory-custody path" >&2
    return 1
  }
  for file in fallback.png component-preview.gif frames.framemd5 manifest.json; do
    [[ "$(sha256sum "$outside_dir/$file" | awk '{print $1}')" == "${outside_shas[$file]}" ]] || {
      printf 'readme-custody-test: substituted parent changed outside %s\n' "$file" >&2
      return 1
    }
  done
  [[ -L "$root/component-preview" ]] || {
    echo "readme-custody-test: substituted public parent was unexpectedly replaced" >&2
    return 1
  }
  if find "$retained_dir" -maxdepth 1 -name '.*.publish.*' -print -quit | grep -q .; then
    echo "readme-custody-test: descriptor-relative cleanup left a publication temporary" >&2
    return 1
  fi
}

if [[ "$case_name" == all || "$case_name" == checker ]]; then
  checker_substitution_is_refused
fi
if [[ "$case_name" == all || "$case_name" == publisher ]]; then
  publisher_temps_preserve_outside_targets
  publisher_parent_substitution_is_refused
fi
echo "readme-custody-test: PASS ($case_name)"
