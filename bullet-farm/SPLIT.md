# bullet-farm

Status: pre-transaction split-family hub; not release-ready
Owner: Bullet Farm maintainers
Last reviewed: 2026-09-09
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

- Primary integration and delivery aggregate:
  `https://github.com/neverhuman/bulletfarm`.
- Four independent member repositories retain their source origins and exact
  commit/tree identities. The Hub retains its JeRyu identity `root/bullet-farm`.
- New publication requests select the destination from the committed
  `publication/config.json`; retained requests keep their original destination
  and identities. Historical workflow-free snapshots remain historical evidence.
- Hosted CI and protected integration require exact-subject execution and
  authoritative read-back.
- Release tag pattern: `bullet-farm-v0.1.0-split.0`

## Split Rules

- Implement changes in canonical member checkouts and generate the primary
  aggregate from their reviewed subjects; preserve original source objects.
- JeRyu self-hosting and native forge qualification remain release obligations.
- Release builds depend on immutable tags, not branches.
- Local development uses `scripts/fuse.sh` output under `.fusion/`.
- Committed manifests must not depend on sibling checkout paths.
- Generated outputs are regenerated from their source contracts or build commands.

## Required Local Check

```bash
bash scripts/ci-local.sh required
```
