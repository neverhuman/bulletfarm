# Historical design corpus

Status: **historical provenance; non-authoritative**  
Last reviewed: 2026-08-25

These documents explain the design lineage that became Bullet Farm. They do
not authorize a mutation, define the current wire format, prove a feature, or
change release readiness. Current sources are indexed in [`../README.md`](../README.md).

| Source | Role |
| --- | --- |
| `CENTERRAIL_FINAL_ADAPTIVE_MULTI_FRONTIER_ENGINEERING_SPEC.md` | Original comprehensive Centerrail engineering design |
| `GASTOWN_OPEN_ISSUES_RISK_AUDIT_FOR_CENTERRAIL.md` | Historical negative-requirements and incident review |
| `git_role.md` | Historical BulletGit design note |
| `paper.md` | Summary and recorded metadata for the separately produced IEEE paper; sanitized copy, see below |
| `nightshift.md` | Release-truth UX note ("display the exact claim that remains unproved"); byte-identical mirror of the family-root `docs/nightshift.md` |
| `POTENTIAL_DRAFT.md` | Adjudicated red-team additions A1–A7 and the doctrine layer; byte-identical mirror of the family-root `docs/POTENTIAL_DRAFT.md` |

`HISTORICAL_ARTIFACTS.sha256` binds the tracked Markdown bytes in this
directory (six entries). Check it with `sha256sum -c docs/spec/HISTORICAL_ARTIFACTS.sha256`
from the hub root. It does not attest to the external PDF, TeX, bibliography,
or ZIP mentioned in `paper.md`; those files are not distributed by this
repository.

`paper.md` here is a **sanitized copy** that differs from the family-root
`docs/paper.md`: the root copy (sha256 `1b34ae64…`) carries four machine-local
`sandbox:/mnt/data/...` links, which were removed to produce this copy
(sha256 `1e1a8bd8…`). Cite this copy, not the root one. `nightshift.md` and
`POTENTIAL_DRAFT.md` are mirrored without modification, so their root and hub
hashes are identical.

The public product name is **Bullet Farm**. “Centerrail” is retained here only
to preserve the historical record and must not be used as current product or
runtime authority.
