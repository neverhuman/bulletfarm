#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOLS_ROOT="$REPO_ROOT/.ci-tools"
VENV="$TOOLS_ROOT/readme-jsonschema"
REQUIREMENTS="$REPO_ROOT/.config/readme-jsonschema-requirements.txt"

for path in "$TOOLS_ROOT" "$VENV"; do
  [[ ! -L "$path" ]] || {
    printf 'install-readme-jsonschema: symlinked tool path refused: %s\n' "$path" >&2
    exit 1
  }
done
command -v python3 >/dev/null 2>&1 || {
  echo 'install-readme-jsonschema: Python 3.12 is required' >&2
  exit 1
}
[[ "$(python3 --version)" == "Python 3.12."* ]] || {
  printf 'install-readme-jsonschema: expected Python 3.12, found %s\n' "$(python3 --version)" >&2
  exit 1
}
[[ -f "$REQUIREMENTS" && ! -L "$REQUIREMENTS" ]] || {
  echo 'install-readme-jsonschema: hash-locked requirements are missing or unsafe' >&2
  exit 1
}

mkdir -p "$TOOLS_ROOT"
python3 -m venv --clear "$VENV"
"$VENV/bin/python3" -m pip install \
  --disable-pip-version-check --only-binary=:all: --require-hashes \
  --requirement "$REQUIREMENTS"
version="$("$VENV/bin/python3" -c 'from importlib.metadata import version; print(version("jsonschema"))')"
[[ "$version" == 4.26.0 ]] || {
  printf 'install-readme-jsonschema: expected jsonschema 4.26.0, found %s\n' "$version" >&2
  exit 1
}
printf 'install-readme-jsonschema: installed pinned validator at %s\n' "$VENV/bin/python3"
