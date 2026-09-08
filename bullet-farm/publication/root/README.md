# Bullet Farm

This public index preserves four independent source repositories. Their exact
commit, tree, object format, and immutable source refs are recorded in
[`publication.json`](publication.json). The original source objects are retained
under `refs/tags/bullet-source/v1/<member>/<commit>`.

| Member | Role |
| --- | --- |
| [bullet-farm](bullet-farm/README.md) | Hub, installer, contracts, and assurance |
| [bullet-kernel](bullet-kernel/README.md) | Control plane and runtime boundaries |
| [bullet-git](bullet-git/README.md) | Candidate graph, journal, and proof roots |
| [bullet-portal](bullet-portal/README.md) | Operations portal |

The aggregate is a publication destination. Each member retains its existing
source authority. CI reconstructs ordinary split checkouts from the recorded
source refs and verifies their trees against this aggregate.

Bullet is not release certified. See the existing
[G1–G18 register](bullet-farm/docs/assurance/product-gaps.md) and
[full-product plan](bullet-farm/docs/assurance/full-product-dogfood-plan.md).
The root bootstrap also executes one Hub source scan with exact artifact and
run-attempt checks. The full 53-job/55-invocation member inventory, family,
scheduled and operational certification campaigns remain required. Human review
is required for integration; local fixture checks do not establish hosted success.

Root files are generated from the Hub's `publication/root/` templates. Change the
source template and republish; direct aggregate edits fail regeneration checks.
