# Historical BulletFarm archive

This branch preserves superseded implementations and historical evidence. Its instructions,
plans, grants and reviews do not govern current development or constitute current approvals.
Development occurs only on `neverhuman/bulletfarm/main` in `/home/ubuntu/bulletfarm`.

## Find original code, PRs and checks

- `snapshots/<repository>/`: six byte-identical source main trees at inventory.
- `ref-index.json`: all 1,475 original remote/local/reflog refs, original object IDs and
  their immutable `archive/<repository>/...` tags. Git authorship is unchanged.
- `pr-index.json`: all 68 historical PR numbers, original head SHAs, source tags and exports.
- `records/<repository>/pulls/<number>/`: descriptions, reviews, commits and timelines.
  Repository-level issue-comments and review-comments files preserve discussions/replies.
- `records/<repository>/checks/` and `statuses/`: historical PR-head check results.
- `records/<repository>/runs/`: job/check results and all available attempt logs (ZIP).
- `records/<repository>/artifact-bytes/`: all 1,766 available CI artifact ZIPs.
- `evidence-manifest.json`: SHA-256 and byte count for every exported evidence file.
  `evidence-summary.json` records repository counts and unavailable material explicitly.

The capture includes 368 workflow runs. No inventoried artifact was expired, no available
artifact failed capture, and no requested workflow attempt log was unavailable. Wikis were
unavailable (recorded per repository); all six repositories had Discussions disabled and no
releases at inventory. These statements describe the captured inventory, not future changes.

The pending `bf` PR 12 head is `b9bd7227ab9ce3dde8959dd0082a3848ed778673` at
`archive/bf/pull/12/head`. Both exact-head review exchanges, discussion, and subsequent
supersession/closure evidence are retained. Historical reviews are not newly imported native
GitHub approvals. The successor lives in the active repository.

No private local Git configuration, credential store, coordination database or launcher is
published. Private mirrors, Git objects, working state, ignored evidence, coordination history
and launchers are backed up outside the development checkout. Build/dependency caches are
excluded from private working-state archives; their paths are inventoried. Original Git source
objects are preserved unchanged, including deliberate secret-redaction test canaries. Exported
record/ZIP contents passed the documented credential-pattern scan with no findings.

Logs/artifacts are compressed; no exported file exceeds 50 MiB. Larger private backup archives
are split into ordered parts with SHA-256 manifests. A fresh GitHub mirror verifies all original
ref object IDs and passes Git integrity checks. See `verification/` for evidence and `tools/`
for the capture/verification implementation. Tools are historical diagnostics, not live authority.

This archive does not by itself authorize deletion. Both migration PRs, independent reviews,
installation/data/PR-discovery checks, independent-backup restoration, final source recheck
and operator-assisted deletion authentication must pass the active migration plan's gates.
