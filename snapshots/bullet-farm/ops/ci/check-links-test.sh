#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
checker="$REPO_ROOT/ops/ci/check-links.sh"
tmp="$(mktemp -d)"
cleanup() {
  local status=$?
  if [[ "$status" == 0 ]]; then rm -rf "$tmp"
  else printf '[ci] link fixture retained: %s\n' "$tmp" >&2; fi
}
trap cleanup EXIT

# Explicit component lane: actual Lychee and an observed loopback HTTP server.
# Ordinary lint retains its network-free shell-tool prerequisites.
if [[ "${1:-}" == --external ]]; then
  command -v lychee >/dev/null
  python3 - "$REPO_ROOT" "$tmp" <<'PY_EXTERNAL'
from pathlib import Path
import http.server
import os
import shutil
import subprocess
import sys
import threading

source, scratch = map(Path, sys.argv[1:])
fixture = scratch / "external repo with spaces"
for name in ("docs", ".github/ISSUE_TEMPLATE", "publication/root/.github", "ops/ci", "home"):
    (fixture / name).mkdir(parents=True, exist_ok=True)
for name in ("external-links.sh", "markdown-inputs.sh", "lib.sh", "artifact-path.sh", "rust-toolchain-boundary.sh"):
    shutil.copy2(source / "ops/ci" / name, fixture / "ops/ci" / name)
observed = []

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        observed.append(self.path)
        self.send_response(404 if self.path.startswith("/missing/") else 200)
        self.end_headers()
        self.wfile.write(b"fixture response")

    def log_message(self, *_args):
        pass

server = http.server.ThreadingHTTPServer(("127.0.0.2", 0), Handler)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
base = f"http://127.0.0.2:{server.server_port}"
paths = ("CONTRIBUTING.md", "SECURITY.md", ".github/PULL_REQUEST_TEMPLATE.md",
         ".github/ISSUE_TEMPLATE/report.md", "publication/root/README.md",
         "publication/root/.github/SUPPORT.md", "docs/guide.md")
(fixture / "README.md").write_text(
    "# Fixture\n[local](docs/guide.md)\n[excluded](http://127.0.0.1:1/excluded)\n")

def write_reference(name, target):
    (fixture / name).write_text(f'[used][reference]\n\n[reference]: <{base}{target}> "Reference"\n')

for index, name in enumerate(paths):
    write_reference(name, f"/valid/{index}")
# These local destinations exist only after aggregate generation. This lane
# accepts no claim about them; the member's local file links still get checked.
with (fixture / "publication/root/README.md").open("a") as file:
    file.write("\n[member](bullet-kernel/README.md)\n")
environment = {"PATH": os.environ["PATH"], "HOME": str(fixture / "home"),
               "LC_ALL": "C", "TZ": "UTC", "NO_PROXY": "127.0.0.2", "no_proxy": "127.0.0.2"}
command = ["bash", str(fixture / "ops/ci/external-links.sh")]

def run(label, required, expect_success):
    observed.clear()
    result = subprocess.run(command, cwd=fixture, env=environment,
                            capture_output=True, text=True, timeout=45)
    (scratch / f"{label}.stdout").write_text(result.stdout)
    (scratch / f"{label}.stderr").write_text(result.stderr)
    print(f"[ci] actual Lychee {label}: exit={result.returncode}, observed={observed}")
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    if expect_success:
        assert result.returncode == 0 and set(required) <= set(observed), label
    else:
        assert result.returncode != 0 and required[0] in observed, label
        assert "404" in result.stdout + result.stderr, label

try:
    run("reference-positive", [f"/valid/{index}" for index in range(len(paths))], True)
    for index, name in enumerate(paths):
        target = f"/missing/{index}"
        original = (fixture / name).read_bytes()
        write_reference(name, target)
        run(f"reference-negative-{index}", [target], False)
        (fixture / name).write_bytes(original)
    (fixture / "README.md").write_text("[missing member file](docs/absent.md)\n")
    result = subprocess.run(command, cwd=fixture, env=environment,
                            capture_output=True, text=True, timeout=45)
    print(result.stdout, end="")
    print(result.stderr, end="", file=sys.stderr)
    assert result.returncode != 0 and "docs/absent.md" in result.stdout + result.stderr
    print("[ci] actual Lychee member-relative refusal passed")
finally:
    server.shutdown()
    server.server_close()
    thread.join(timeout=5)
print("[ci] actual external reference matrix passed (1 positive, 7 observed HTTP 404 refusals, 1 member-relative refusal)")
PY_EXTERNAL
  exit 0
fi
[[ "$#" == 0 ]] || { printf 'usage: check-links-test.sh [--external]\n' >&2; exit 2; }


make_fixture() {
  local fixture="$1"
  mkdir -p "$fixture/repo/docs" "$fixture/repo/images"
  printf 'png\n' >"$fixture/repo/images/example.png"
  printf '%s\n' \
    '# Home' \
    '' \
    '[inline](docs/guide.md#details)' \
    '![image](images/example.png)' \
    '[reference][guide]' \
    '[same](#home)' \
    '[external](https://example.invalid/not-fetched)' \
    '' \
    '[guide]: docs/guide.md#details "Guide"' >"$fixture/repo/README.md"
  printf '%s\n' \
    '# Guide' \
    '' \
    '## Details' \
    '' \
    '[up](../README.md#home)' >"$fixture/repo/docs/guide.md"
}

expect_failure() {
  local name="$1" fixture="$2"
  if bash "$checker" --root "$fixture/repo" README.md docs/guide.md >/dev/null 2>&1; then
    printf '[ci] CHECK_LINKS_NEGATIVE_MISSED: %s\n' "$name" >&2
    exit 1
  fi
}

valid="$tmp/valid"
make_fixture "$valid"
bash "$checker" --root "$valid/repo" README.md docs/guide.md >/dev/null

missing_target="$tmp/missing-target"
make_fixture "$missing_target"
printf '\n[missing](docs/absent.md)\n' >>"$missing_target/repo/README.md"
expect_failure missing-target "$missing_target"

missing_fragment="$tmp/missing-fragment"
make_fixture "$missing_fragment"
printf '\n[missing](docs/guide.md#absent)\n' >>"$missing_fragment/repo/README.md"
expect_failure missing-fragment "$missing_fragment"

escape="$tmp/escape"
make_fixture "$escape"
printf '# Outside\n' >"$escape/outside.md"
printf '\n[outside](../outside.md)\n' >>"$escape/repo/README.md"
expect_failure parent-escape "$escape"

symlink_escape="$tmp/symlink-escape"
make_fixture "$symlink_escape"
printf '# Outside\n' >"$symlink_escape/outside.md"
ln -s "$symlink_escape/outside.md" "$symlink_escape/repo/docs/outside.md"
printf '\n[outside](docs/outside.md)\n' >>"$symlink_escape/repo/README.md"
expect_failure symlink-escape "$symlink_escape"

absolute="$tmp/absolute"
make_fixture "$absolute"
printf '\n[absolute](/etc/passwd)\n' >>"$absolute/repo/README.md"
expect_failure absolute-path "$absolute"

brand_claim="$tmp/brand-claim"
make_fixture "$brand_claim"
mkdir -p "$brand_claim/repo/docs/brand/mascots"
printf '# Brief\n\nGas Town is worse.\n' \
  >"$brand_claim/repo/docs/brand/mascots/01-hostile.md"
if bash "$checker" --root "$brand_claim/repo" >/dev/null 2>&1; then
  printf '[ci] BRAND_COMPETITOR_CLAIM_ACCEPTED\n' >&2
  exit 1
fi

# Default inventory must include root community and hidden GitHub Markdown.
for name in CONTRIBUTING.md SECURITY.md '.github/PULL_REQUEST_TEMPLATE.md' '.github/ISSUE_TEMPLATE/report.md'; do
  fixture="$tmp/inventory-${name//\//-}"
  make_fixture "$fixture"
  mkdir -p "$(dirname "$fixture/repo/$name")"
  printf '[used][ref]\n\n[ref]: docs/absent.md\n' >"$fixture/repo/$name"
  if bash "$checker" --root "$fixture/repo" >"$tmp/inventory.stdout" 2>"$tmp/inventory.stderr"; then
    printf '[ci] MARKDOWN_INVENTORY_OMISSION: %s\n' "$name" >&2
    exit 1
  fi
  rg -q 'BROKEN_RELATIVE_LINK:' "$tmp/inventory.stderr"
done

inventory="$tmp/complete-inventory"
make_fixture "$inventory"
mkdir -p "$inventory/repo/.github/ISSUE_TEMPLATE" "$inventory/repo/publication/root/.github"
printf '# Security\n' >"$inventory/repo/SECURITY.md"
printf '# Template\n' >"$inventory/repo/.github/ISSUE_TEMPLATE/with spaces.md"
printf '# Template\n' >"$inventory/repo/.github/ISSUE_TEMPLATE/with"$'\n'"newline.md"
printf '[member](bullet-kernel/README.md)\n' >"$inventory/repo/publication/root/README.md"
printf '# Support\n' >"$inventory/repo/publication/root/.github/SUPPORT.md"
# shellcheck source=ops/ci/markdown-inputs.sh
source "$REPO_ROOT/ops/ci/markdown-inputs.sh"
collect_markdown_inputs "$inventory/repo" member
[[ "${#MARKDOWN_INPUTS[@]}" == 5 ]]
[[ "${MARKDOWN_INPUTS[0]}" == './README.md' ]]
[[ "${MARKDOWN_INPUTS[1]}" == './SECURITY.md' ]]
[[ "${MARKDOWN_INPUTS[2]}" == '.github/ISSUE_TEMPLATE/with'$'\n''newline.md' ]]
[[ "${MARKDOWN_INPUTS[3]}" == '.github/ISSUE_TEMPLATE/with spaces.md' ]]
[[ "${MARKDOWN_INPUTS[4]}" == 'docs/guide.md' ]]
collect_markdown_inputs "$inventory/repo" templates
[[ "${#MARKDOWN_INPUTS[@]}" == 2 ]]
[[ "${MARKDOWN_INPUTS[0]}" == 'publication/root/.github/SUPPORT.md' ]]
[[ "${MARKDOWN_INPUTS[1]}" == 'publication/root/README.md' ]]
bash "$checker" --root "$inventory/repo" >/dev/null

# No partial inventory may hide a failed enumeration or escape into another root.
mv "$inventory/repo/docs" "$inventory/repo/docs-saved"
if collect_markdown_inputs "$inventory/repo" member >"$tmp/invalid.stdout" 2>"$tmp/invalid.stderr"; then
  printf '[ci] MISSING_INVENTORY_ROOT_ACCEPTED\n' >&2; exit 1
fi
rg -q MARKDOWN_INVENTORY_FAILED "$tmp/invalid.stderr"
mv "$inventory/repo/docs-saved" "$inventory/repo/docs"
ln -s "$tmp/valid/repo/README.md" "$inventory/repo/.github/foreign.md"
if collect_markdown_inputs "$inventory/repo" member >"$tmp/escape.stdout" 2>"$tmp/escape.stderr"; then
  printf '[ci] FOREIGN_MARKDOWN_INPUT_ACCEPTED\n' >&2; exit 1
fi
rg -q MARKDOWN_INPUT_ESCAPES_ROOT "$tmp/escape.stderr"
[[ "${#MARKDOWN_INPUTS[@]}" == 0 ]]
rm "$inventory/repo/.github/foreign.md"
ln -s "$tmp/valid/repo/docs" "$inventory/repo/.github/linked-directory"
if collect_markdown_inputs "$inventory/repo" member >"$tmp/directory.stdout" 2>"$tmp/directory.stderr"; then
  printf '[ci] OMITTED_DIRECTORY_SYMLINK_ACCEPTED\n' >&2; exit 1
fi
rg -q MARKDOWN_DIRECTORY_SYMLINK "$tmp/directory.stderr"
rm "$inventory/repo/.github/linked-directory"
mkdir "$tmp/failing-bin"
printf '#!/usr/bin/env bash\nprintf "./README.md\\0"\nexit 43\n' >"$tmp/failing-bin/find"
chmod +x "$tmp/failing-bin/find"
if PATH="$tmp/failing-bin:$PATH" collect_markdown_inputs "$inventory/repo" member \
  >"$tmp/partial.stdout" 2>"$tmp/partial.stderr"; then
  printf '[ci] PARTIAL_ENUMERATION_ACCEPTED\n' >&2; exit 1
fi
rg -q MARKDOWN_INVENTORY_FAILED "$tmp/partial.stderr"
[[ "${#MARKDOWN_INPUTS[@]}" == 0 ]]
printf '[ci] Markdown links, references, complete inventories and containment matrix passed\n'
