# Bullet Farm

This is the primary integration repository for Bullet Farm on
[GitHub](https://github.com/neverhuman/bulletfarm). It preserves all four
supporting source repositories. Their exact
commit, tree, object format, and immutable source refs are recorded in
[`publication.json`](publication.json). The original source objects are retained
under `refs/tags/bullet-source/v1/<member>/<commit>`.

| Member | Role |
| --- | --- |
| [bullet-farm](bullet-farm/README.md) | Hub, installer, contracts, and assurance |
| [bullet-kernel](bullet-kernel/README.md) | Control plane and runtime boundaries |
| [bullet-git](bullet-git/README.md) | Candidate graph, journal, and proof roots |
| [bullet-portal](bullet-portal/README.md) | Operations portal |

Each member retains its existing source authority. The aggregate contains every
member tree and retains the original source objects. The publication tool verifies
those trees; hosted CI and protected integration still require exact-subject
execution and authoritative read-back.

Bullet is not release certified. See the existing
[G1–G18 register](bullet-farm/docs/assurance/product-gaps.md) and
[full-product plan](bullet-farm/docs/assurance/full-product-dogfood-plan.md).
Version-2 root workflows derive from the reviewed 53-job/55-invocation member
inventory. Missing execution profiles remain explicit refusals. Complete member,
family, scheduled and operational certification campaigns remain required.
Human review is required for integration; local fixture checks do not establish
hosted success. JeRyu self-hosting and native forge qualification remain separate
release obligations.

Root files are generated from the Hub's `publication/root/` templates. Change the
source template and republish; direct aggregate edits fail regeneration checks.
