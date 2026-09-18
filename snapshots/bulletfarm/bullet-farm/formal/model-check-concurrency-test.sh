#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMAL="$ROOT/formal"
CACHE="$ROOT/target/tlc/v1.7.4"
TEST_ROOT="$(mktemp -d "$ROOT/target/formal-concurrency.XXXXXX")"
STATES_ROOT="$FORMAL/states"
states_created=false
states_sentinel=""
pid_a=""
pid_b=""

cleanup() {
  for pid in "$pid_a" "$pid_b"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
  if [[ -n "$states_sentinel" ]]; then
    rm -f -- "$states_sentinel"
  fi
  if [[ "$states_created" == true ]]; then
    rm -rf -- "$STATES_ROOT"
  fi
  rm -rf -- "$TEST_ROOT"
}
trap cleanup EXIT

[[ ! -L "$STATES_ROOT" ]] || {
  echo "formal-concurrency-test: default metadata root must not be a symlink" >&2
  exit 1
}
if [[ -e "$STATES_ROOT" && ! -d "$STATES_ROOT" ]]; then
  echo "formal-concurrency-test: default metadata root must be a directory" >&2
  exit 1
fi
if [[ -d "$STATES_ROOT" ]] \
  && find "$STATES_ROOT" -mindepth 1 -print -quit | grep -q .; then
  echo "formal-concurrency-test: default metadata root must start empty" >&2
  exit 1
fi
if [[ ! -e "$STATES_ROOT" ]]; then
  mkdir -m 0700 -- "$STATES_ROOT"
  states_created=true
fi
states_sentinel="$(mktemp "$STATES_ROOT/.private-run-sentinel.XXXXXX")"
states_sentinel_name="${states_sentinel##*/}"
printf 'private model checks must not touch this default root\n' >"$states_sentinel"
states_sentinel_hash="$(sha256sum "$states_sentinel" | awk '{print $1}')"

bash "$FORMAL/model-check.sh" check >"$TEST_ROOT/a.log" 2>&1 &
pid_a=$!
bash "$FORMAL/model-check.sh" check >"$TEST_ROOT/b.log" 2>&1 &
pid_b=$!

status_a=0
status_b=0
wait "$pid_a" || status_a=$?
pid_a=""
wait "$pid_b" || status_b=$?
pid_b=""
[[ "$status_a" -eq 0 && "$status_b" -eq 0 ]] || {
  echo "formal-concurrency-test: overlapping checks failed: a=$status_a b=$status_b" >&2
  sed -n '1,240p' "$TEST_ROOT/a.log" >&2
  sed -n '1,240p' "$TEST_ROOT/b.log" >&2
  exit 1
}

for log in "$TEST_ROOT/a.log" "$TEST_ROOT/b.log"; do
  [[ "$(grep -Fc 'Model checking completed. No error has been found.' "$log")" -eq 2 ]] || {
    echo "formal-concurrency-test: incomplete model inventory in $log" >&2
    exit 1
  }
  [[ "$(grep -Fc '1585 states generated, 378 distinct states found' "$log")" -eq 1 ]] || {
    echo "formal-concurrency-test: EffectCheck counts drifted in $log" >&2
    exit 1
  }
  [[ "$(grep -Fc '578478 states generated, 141963 distinct states found' "$log")" -eq 1 ]] || {
    echo "formal-concurrency-test: LeaseFence counts drifted in $log" >&2
    exit 1
  }
  [[ "$(grep -Fc 'The depth of the complete state graph search is 18.' "$log")" -eq 1 ]] || {
    echo "formal-concurrency-test: EffectCheck depth drifted in $log" >&2
    exit 1
  }
  [[ "$(grep -Fc 'The depth of the complete state graph search is 27.' "$log")" -eq 1 ]] || {
    echo "formal-concurrency-test: LeaseFence depth drifted in $log" >&2
    exit 1
  }
  [[ "$(grep -Fc 'formal-check: 2/2 models match pinned tool, exact lock shape, source, config, states, and depth' "$log")" -eq 1 ]] || {
    echo "formal-concurrency-test: pinned completion summary missing from $log" >&2
    exit 1
  }
  for standard_module in Naturals FiniteSets TLC Sequences; do
    mapfile -t standard_paths < <(grep -F "/java-tmp/$standard_module.tla" "$log")
    [[ "${#standard_paths[@]}" -eq 2 ]] || {
      echo "formal-concurrency-test: $standard_module was not extracted once per private model run in $log" >&2
      exit 1
    }
    for standard_path in "${standard_paths[@]}"; do
      [[ "$standard_path" == "Parsing file $CACHE/run."*"/java-tmp/$standard_module.tla" ]] || {
        echo "formal-concurrency-test: $standard_module escaped the private JVM scratch root in $log" >&2
        exit 1
      }
    done
  done
  if grep -Fq 'No such file or directory' "$log"; then
    echo "formal-concurrency-test: metadata disappeared during an overlapping check" >&2
    exit 1
  fi
done

[[ "$(sha256sum "$states_sentinel" | awk '{print $1}')" == "$states_sentinel_hash" ]] || {
  echo "formal-concurrency-test: a check touched the default metadata root" >&2
  exit 1
}
states_inventory="$(find "$STATES_ROOT" -mindepth 1 -printf '%P:%y\n' | LC_ALL=C sort)"
[[ "$states_inventory" == "$states_sentinel_name:f" ]] || {
  echo "formal-concurrency-test: a check wrote into the default metadata root" >&2
  exit 1
}
if find "$CACHE" -mindepth 1 -maxdepth 1 -type d -name 'run.*' -print -quit | grep -q .; then
  echo "formal-concurrency-test: a private run root survived cleanup" >&2
  exit 1
fi

echo "formal-concurrency-test: two overlapping pinned checks passed with private metadata and logs"
