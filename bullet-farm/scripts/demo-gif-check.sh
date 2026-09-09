#!/usr/bin/env bash
# Explicit private media bootstrap; no install, provider call, or public export.
set -euo pipefail
[[ $# -ge 3 ]] || { echo 'usage: PYTHON PYTHON_SHA256 IMPLEMENTATION_SHA256 [options]' >&2; exit 2; }
python="$1"; python_sha="$2"; implementation_sha="$3"; shift 3
implementation="$(cd "${BASH_SOURCE[0]%/*}/lib" && pwd -P)/demo-gif-render.py"
[[ "$python" == /* && -f "$python" && ! -L "$python" && -x "$python" &&
   "$python_sha" =~ ^[0-9a-f]{64}$ && "$implementation_sha" =~ ^[0-9a-f]{64}$ ]] || exit 2
actual="$(/usr/bin/sha256sum -- "$python")"
[[ "${actual%% *}" == "$python_sha" ]] || { echo PYTHON_HASH_MISMATCH >&2; exit 2; }
actual="$(/usr/bin/sha256sum -- "$implementation")"
[[ "${actual%% *}" == "$implementation_sha" ]] || { echo SOURCE_HASH_MISMATCH >&2; exit 2; }
exec "$python" -I "$implementation" check "$@"
