#!/usr/bin/env bash
# Canonical security-lane wrapper for bullet-farm.
#
# The executable lane is ops/ci/security.sh; this wrapper is the stable entry
# point that names the tools the lane runs so a reader (human or agent) does not
# have to reconstruct them from the workflow. It adds no policy of its own and
# it never downgrades a failure: it execs the lane and inherits its exit code.
#
# Tools, all fail-closed. `scripts/ci-doctor.sh security` pins the exact
# versions and the lane refuses a missing or wrong-version tool rather than
# reducing itself to the checks that happen to be runnable:
#   gitleaks detect --source . --no-git --redact --no-banner   (gitleaks 8.21.2)
#   cargo deny --locked check licenses advisories bans sources  (cargo-deny 0.19.8,
#     against the committed deny.toml, after an independent freshness proof of
#     the RustSec advisory database)
#   zizmor .                                                    (zizmor 1.25.2,
#     workflow linting over the committed workflow bytes)
#   cargo audit                    (not a second lane: cargo deny advisories
#     already consumes the RustSec DB this wrapper would otherwise name)
#   npm audit                      (not applicable: this repo has no product
#     package.json graph)
#   sbom                           (release build can emit one; this lane does
#     not publish an SBOM because no release artifact exists)
# This wrapper is tools/security-lane.sh.
#
# Declared in agent/security-policy.toml. Do not add `|| true` to any step.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$repo_root/ops/ci/security.sh"
