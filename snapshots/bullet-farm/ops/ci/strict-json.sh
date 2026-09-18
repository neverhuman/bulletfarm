#!/usr/bin/env bash
# Parse security- and evidence-sensitive JSON with the Python 3.12 standard
# library before jq sees it. Duplicate members and non-finite numbers refuse.
set -euo pipefail

(( "$#" > 0 )) || {
  echo '[ci] STRICT_JSON_PATH_MISSING' >&2
  exit 2
}
if command -v python3 >/dev/null 2>&1; then
  python_bin=python3
elif command -v python >/dev/null 2>&1; then
  python_bin=python
else
  echo '[ci] STRICT_JSON_PYTHON_MISSING' >&2
  exit 1
fi
python_version="$("$python_bin" --version 2>&1)"
[[ "$python_version" == "Python 3.12."* ]] || {
  echo '[ci] STRICT_JSON_PYTHON_VERSION_INVALID' >&2
  exit 1
}

exec "$python_bin" -I -S - "$@" <<'PY'
import json
import math
import sys
from decimal import Decimal, InvalidOperation

MAX_DOCUMENT_BYTES = 4 * 1024 * 1024
MAX_SAFE_INTEGER = 9_007_199_254_740_991


def reject_duplicate_members(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON object member")
        result[key] = value
    return result


def reject_non_json_constant(_value):
    raise ValueError("non-JSON numeric constant")


def parse_strict_integer(value):
    parsed = int(value)
    if abs(parsed) > MAX_SAFE_INTEGER:
        raise ValueError("integer exceeds the interoperable safe range")
    return parsed


def parse_strict_float(value):
    try:
        exact = Decimal(value)
        binary = float(value)
        round_trip = Decimal(repr(binary))
    except (InvalidOperation, OverflowError):
        raise ValueError("JSON number is outside the interoperable range") from None
    if not math.isfinite(binary) or exact != round_trip:
        raise ValueError("JSON number loses precision")
    if exact == exact.to_integral_value() and abs(exact) > MAX_SAFE_INTEGER:
        raise ValueError("integer exceeds the interoperable safe range")
    return exact


def reject_non_finite_numbers(value):
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError("non-finite JSON number")
    if isinstance(value, dict):
        for member in value.values():
            reject_non_finite_numbers(member)
    elif isinstance(value, list):
        for member in value:
            reject_non_finite_numbers(member)


for path in sys.argv[1:]:
    try:
        with open(path, "rb") as handle:
            encoded = handle.read(MAX_DOCUMENT_BYTES + 1)
        if len(encoded) > MAX_DOCUMENT_BYTES:
            raise ValueError("JSON document exceeds the size bound")
        document = json.loads(
            encoded.decode("utf-8"),
            object_pairs_hook=reject_duplicate_members,
            parse_constant=reject_non_json_constant,
            parse_float=parse_strict_float,
            parse_int=parse_strict_integer,
        )
        reject_non_finite_numbers(document)
    except (OSError, UnicodeError, ValueError, RecursionError, MemoryError):
        # Do not echo parser details: a rejected document can contain secrets.
        print(f"[ci] STRICT_JSON_INVALID: {path}", file=sys.stderr)
        raise SystemExit(1)
PY
