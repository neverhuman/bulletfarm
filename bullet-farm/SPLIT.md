# bullet-farm

Status: pre-transaction split-family hub; not release-ready
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-26
Applies to: bullet-farm

## Role

Public hub, installer, family manifest and lock, fusion script, onboarding,
frozen wire contracts (`crates/bullet-wire`), canonical policy (`policy/`),
release verification (`bullet-family release verify|extract|receipt-verify`),
family coordination (`bullet-family coord`), the fail-closed explicit-profile
release decision (`bullet-family check release --profile <profile> --receipts
<absolute-registry> --json`; `just release-truth` renders the diagnostic page), and
historical Centerrail design provenance.

## Repositories

- Initial source authority: Jeryu repository `root/bullet-farm`
- Public GitHub index: `https://github.com/neverhuman/bulletfarm` (not
  `neverhuman/bullet-farm`); discovery/PR mirror only, never source authority.
  The first public snapshot omits `.github/workflows/*` because the publication
  credential lacks GitHub `workflow` scope; local Hub main keeps those files.
- Release tag pattern: `bullet-farm-v0.1.0-split.0`

## Split Rules

- Jeryu is the initial source forge; GitHub is a configurable effect adapter,
  not source authority.
- Release builds depend on immutable tags, not branches.
- Local development uses `scripts/fuse.sh` output under `.fusion/`.
- Committed manifests must not depend on sibling checkout paths.
- Generated outputs are regenerated from their source contracts or build commands.

## Required Local Check

```bash
bash scripts/ci-local.sh required
```
