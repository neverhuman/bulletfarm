# 0002 — Jeryu as the Bullet Farm effect target: requirements and do-not-disturb rules

Status: Accepted target; credentialed access quarantined pending Wave 8
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: bullet-kernel (effects), bullet-farm (hub)

## Decision

Local Jeryu is the default future production forge. Wave 4 closes its pinned
capability contract, Wave 5 closes the forge semantic port offline, and Wave 8
requires a separately approved protected live transaction. A local bare
repository (`LocalBareForge`) backs offline component and fault tests behind the
same `ForgeEffects` port. GitHub certifies separately. Wave 0 performs no
authenticated probe, repository creation, push, check, pull request, or
integration.

## Do-not-disturb rules (hard)

Many repo families depend on this Jeryu instance. Therefore:
- NEVER stop, restart, upgrade, or reconfigure the `jeryu serve` process (pid-owned by the operator).
- NEVER edit the operator's Jeryu configuration/data, its `--split-manifest` files, or the canonical
  `jeryu-split` source family as part of Bullet Farm work. The family `AGENTS.md` identifies the
  permitted checkout; no substitute family may be created.
- Probe only an operator-named, immutable, signed Jeryu build after its exact
  tag, manifest, binary/API/capability digests, SBOM, provenance, and signature
  match the family lock. No arbitrary `root/bullet-*` repository or hostname is
  authorized. Missing semantics return `UNSUPPORTED_BY_ADAPTER`; they are never
  worked around by touching the forge or reported green.
- Existing host tokens are outside the trust boundary and do not authorize Bullet Farm. Re-login is
  not requested during quarantine.

## Feature requirements to verify against the running forge (fill in with probe results)

| Feature (git_role.md / spec §23.3) | Needed for | Jeryu status |
|---|---|---|
| Designate or create the exact protected test repository through authorized REST | Wave-8 protected transaction | UNPROBED — QUARANTINED |
| Push branch `refs/heads/bullet/candidate/<id>` | Candidate export | UNPROBED — QUARANTINED |
| Expected-old-OID push semantics (or --force-with-lease honored) | exact-subject delivery | UNPROBED — QUARANTINED |
| Read ref back via REST | Effect receipt | UNPROBED — QUARANTINED |
| Create/update PR idempotently | protected delivery | UNPROBED — QUARANTINED |
| Check-run bound to exact SHA (`Bullet Farm / Proof Complete`, proof_root payload) | proof surface | UNPROBED — QUARANTINED |
| Branch protection / required checks | protected integration | UNPROBED — QUARANTINED |
| Merge queue / merge-group subject | Phase-later integration | UNPROBED — QUARANTINED |
| Immutable annotated tags | family release pins | UNPROBED — QUARANTINED |

Gaps become a reviewed proposal and new signed Jeryu release in the independent
`/home/ubuntu/jain-split/jeryu-split` family, not a patch to Bullet or the
running forge. Deployment to any hostname is a separate operator decision;
`git.neverhuman.org` is not currently ratified for Bullet.

## Probe results (2026-08-24, unauthenticated GET)

The forge is healthy: `/` serves the SPA (200) and `/api/v3` answers with GitHub-style JSON
(`401 {"documentation_url":"/docs/rest","message":"Requires authentication"}`), as does the git
smart-HTTP endpoint (`/git/<org>/<repo>.git/info/refs` → 401). Every REST capability probe in the
table above would require authenticated admission. This historical observation is not a current
capability receipt. The effects lane remains on `LocalBareForge` and the table stays quarantined.
