# Bullet Farm primary aggregate

This repository publishes exact reviewed subjects from four independent source
repositories. Read `publication.json` for their commits, trees and immutable
source refs. The primary integration destination is
`https://github.com/neverhuman/bulletfarm`, with Git transport at
`https://github.com/neverhuman/bulletfarm.git`. New requests capture this selected
destination; retained requests keep their original destinations and identities.

Root files are generated from the Hub's `publication/root/` templates according
to `bullet-farm/publication/config.json`. Edit the canonical Hub templates and
regenerate through `bullet-publish`; do not hand-edit aggregate copies.

Implement product changes in the claimed canonical source checkout. Follow its
`AGENTS.md`, family manifest and coordination log. Do not create Git worktrees.
Source origins remain authoritative; aggregate integration requires human review.

The retained GitHub publication bootstrap template defines aggregate integrity, publication
unit tests and exact source reconstruction into ordinary disposable CI checkouts.
It also defines execution of the exact Hub source-scan lane and validation of completed artifacts
against the aggregate event, member subjects, run and attempt. Its success does
not establish full member/family CI, operational evidence or release certification.
Version-2 generated roots add the reviewed member job inventory; unsupported
execution profiles remain explicit failures. Complete campaigns and tool admission
remain required. JeRyu self-hosting, runner activation, native reconstruction and
native PR transport remain separate release obligations. A workflow file or a
projected check does not prove execution.

CI uses the real aggregate event SHA and retains distinct member commit/tree
subjects. Never substitute a member SHA for `GITHUB_SHA`, infer a missing check
as success, or run privileged provider/release operations from an ordinary PR.
