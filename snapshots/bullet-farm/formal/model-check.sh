#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMAL="$ROOT/formal"
LOCK="$FORMAL/toolchain.lock.json"
MODEL_LOCK="$FORMAL/model-lock.json"
CACHE="$ROOT/target/tlc/v1.7.4"
JAR="${TLA2TOOLS_JAR:-$CACHE/tla2tools.jar}"
MODE="${1:-check}"
SEED=20260824
WORKERS=1

if [[ "$#" -gt 1 || ("$MODE" != "check" && "$MODE" != "write") ]]; then
  echo "formal-check: usage: bash formal/model-check.sh [check|write]" >&2
  exit 2
fi

mkdir -p "$CACHE"

for required in awk chmod curl find grep java jq mktemp mv rm sed sha1sum sha256sum sort tee; do
  command -v "$required" >/dev/null 2>&1 || {
    echo "formal-check: missing required tool $required" >&2
    exit 2
  }
done

expected_sha="$(jq -er '.tlc_jar_sha256' "$LOCK")"
expected_sha1="$(jq -er '.tlc_jar_sha1' "$LOCK")"
url="$(jq -er '.url' "$LOCK")"
expected_java="$(jq -er '.java_major' "$LOCK")"

if [[ ! -f "$JAR" ]]; then
  [[ -z "${TLA2TOOLS_JAR:-}" ]] || {
    echo "formal-check: explicit TLA2TOOLS_JAR does not exist: $JAR" >&2
    exit 2
  }
  partial="$CACHE/tla2tools.jar.partial"
  curl --proto '=https' --tlsv1.2 -fLSs "$url" -o "$partial"
  echo "$expected_sha  $partial" | sha256sum --check --status || {
    echo "formal-check: downloaded TLC jar failed SHA-256 pin" >&2
    exit 2
  }
  mv "$partial" "$JAR"
fi

echo "$expected_sha  $JAR" | sha256sum --check --status || {
  echo "formal-check: TLC jar SHA-256 does not match toolchain lock" >&2
  exit 2
}
actual_sha1="$(sha1sum "$JAR" | awk '{print $1}')"
[[ "$actual_sha1" == "$expected_sha1" ]] || {
  echo "formal-check: TLC jar SHA-1 does not match upstream release checksum" >&2
  exit 2
}
java_major="$(java -version 2>&1 | sed -nE '1s/.*version "([0-9]+).*/\1/p')"
[[ "$java_major" == "$expected_java" ]] || {
  echo "formal-check: Java major $java_major does not match pinned major $expected_java" >&2
  exit 2
}

validate_model_lock() {
  local path="$1"
  jq -e --argjson seed "$SEED" --argjson workers "$WORKERS" '
    def nonnegative_integer:
      type == "number" and . >= 0 and floor == .;
    def sha256:
      type == "string" and test("^[0-9a-f]{64}$");
    type == "object"
    and (keys == ["models", "schema_version", "seed", "workers"])
    and .schema_version == "v1alpha1"
    and .seed == $seed
    and .workers == $workers
    and (.models | type == "array" and length == 2)
    and ([.models[].module] == ["EffectCheck.tla", "LeaseFence.tla"])
    and all(.models[];
      . as $model
      | type == "object"
      and (keys == ["config", "config_sha256", "depth", "distinct_states",
                    "generated_states", "module", "module_sha256"])
      and ($model.module | type == "string" and test("^[A-Za-z][A-Za-z0-9]*\\.tla$"))
      and ($model.config | type == "string" and test("^[A-Za-z][A-Za-z0-9]*\\.cfg$"))
      and ($model.config == ($model.module | sub("\\.tla$"; ".cfg")))
      and ($model.module_sha256 | sha256)
      and ($model.config_sha256 | sha256)
      and ($model.generated_states | nonnegative_integer)
      and ($model.distinct_states | nonnegative_integer)
      and ($model.depth | nonnegative_integer)
    )
  ' "$path" >/dev/null 2>&1 || {
    echo "formal-check: model lock has an invalid or incomplete exact shape" >&2
    exit 2
  }
}

if [[ -L "$MODEL_LOCK" ]]; then
  echo "formal-check: model lock must not be a symlink" >&2
  exit 2
fi
if [[ "$MODE" == "check" ]]; then
  [[ -f "$MODEL_LOCK" ]] || {
    echo "formal-check: model lock is missing" >&2
    exit 2
  }
  validate_model_lock "$MODEL_LOCK"
fi

mapfile -t modules < <(find "$FORMAL" -maxdepth 1 -type f -name '*.tla' -printf '%f\n' | sort)
[[ "${#modules[@]}" -eq 2 ]] || {
  echo "formal-check: exactly two TLA+ modules are permitted; found ${#modules[@]}" >&2
  exit 2
}
[[ "${modules[*]}" == "EffectCheck.tla LeaseFence.tla" ]] || {
  echo "formal-check: exact model inventory must be EffectCheck.tla and LeaseFence.tla" >&2
  exit 2
}

observations='[]'
run_root="$(mktemp -d "$CACHE/run.XXXXXX")"
generated_lock=""
cleanup() {
  if [[ -n "$generated_lock" ]]; then
    rm -f -- "$generated_lock"
  fi
  rm -rf -- "$run_root"
}
trap cleanup EXIT

for module in "${modules[@]}"; do
  name="${module%.tla}"
  config="$name.cfg"
  module_sha256="$(sha256sum "$FORMAL/$module" | awk '{print $1}')"
  config_sha256="$(sha256sum "$FORMAL/$config" | awk '{print $1}')"

  if [[ "$MODE" == "check" ]]; then
    expected_module="$(jq -er --arg module "$module" '.models[] | select(.module == $module) | .module_sha256' "$MODEL_LOCK")"
    expected_config="$(jq -er --arg module "$module" '.models[] | select(.module == $module) | .config_sha256' "$MODEL_LOCK")"
    [[ "$module_sha256" == "$expected_module" ]] || {
      echo "formal-check: $module differs from model lock" >&2
      exit 2
    }
    [[ "$config_sha256" == "$expected_config" ]] || {
      echo "formal-check: $config differs from model lock" >&2
      exit 2
    }
  fi

  model_run_root="$run_root/$name"
  mkdir -m 0700 -- "$model_run_root"
  java_tmp_root="$model_run_root/java-tmp"
  mkdir -m 0700 -- "$java_tmp_root"
  metadata_root="$model_run_root/metadata"
  log="$model_run_root/$name.log"
  (
    cd "$FORMAL"
    java -Djava.io.tmpdir="$java_tmp_root" -XX:+UseParallelGC \
      -cp "$JAR" tlc2.TLC -workers "$WORKERS" \
      -metadir "$metadata_root" -seed "$SEED" -fp 0 -config "$config" "$module"
  ) | tee "$log"
  grep -Fq "Model checking completed. No error has been found." "$log" || {
    echo "formal-check: $name did not complete cleanly" >&2
    exit 2
  }
  counts="$(sed -nE 's/^([0-9]+) states generated, ([0-9]+) distinct states found.*/\1 \2/p' "$log")"
  [[ "$counts" =~ ^[0-9]+\ [0-9]+$ ]] || {
    echo "formal-check: $name emitted an invalid or ambiguous state-count line" >&2
    exit 2
  }
  read -r generated distinct <<< "$counts"
  depth="$(sed -nE 's/^The depth of the complete state graph search is ([0-9]+)\./\1/p' "$log")"
  [[ "$depth" =~ ^[0-9]+$ ]] || {
    echo "formal-check: $name emitted an invalid or ambiguous depth line" >&2
    exit 2
  }

  if [[ "$MODE" == "check" ]]; then
    expected_generated="$(jq -er --arg module "$module" '.models[] | select(.module == $module) | .generated_states' "$MODEL_LOCK")"
    expected_distinct="$(jq -er --arg module "$module" '.models[] | select(.module == $module) | .distinct_states' "$MODEL_LOCK")"
    expected_depth="$(jq -er --arg module "$module" '.models[] | select(.module == $module) | .depth' "$MODEL_LOCK")"
    [[ "$generated" == "$expected_generated" && "$distinct" == "$expected_distinct" && "$depth" == "$expected_depth" ]] || {
      echo "formal-check: $name states drifted: generated=$generated distinct=$distinct depth=$depth" >&2
      exit 2
    }
  else
    observations="$(jq -cn \
      --argjson models "$observations" \
      --arg config "$config" \
      --arg config_sha256 "$config_sha256" \
      --argjson depth "$depth" \
      --argjson distinct_states "$distinct" \
      --argjson generated_states "$generated" \
      --arg module "$module" \
      --arg module_sha256 "$module_sha256" \
      '$models + [{config: $config, config_sha256: $config_sha256, depth: $depth,
                   distinct_states: $distinct_states, generated_states: $generated_states,
                   module: $module, module_sha256: $module_sha256}]')"
  fi
done

if [[ "$MODE" == "write" ]]; then
  generated_lock="$(mktemp "$FORMAL/.model-lock.json.tmp.XXXXXX")"
  jq -cn \
    --argjson models "$observations" \
    --argjson seed "$SEED" \
    --argjson workers "$WORKERS" \
    '{models: $models, schema_version: "v1alpha1", seed: $seed, workers: $workers}' \
    > "$generated_lock"
  validate_model_lock "$generated_lock"
  chmod 0644 "$generated_lock"
  mv "$generated_lock" "$MODEL_LOCK"
  generated_lock=""
  echo "formal-check: wrote model-lock.json from 2/2 pinned TLC observations"
else
  echo "formal-check: 2/2 models match pinned tool, exact lock shape, source, config, states, and depth"
fi
