# Consolidation into neverhuman/bulletfarm

User-adopted plan, 2026-09-18. Keep the existing GitHub repository and its history.
Develop only in `/home/ubuntu/bulletfarm`: one Rust package, embedded React frontend,
ordinary supporting directories, shared documentation and CI. No submodules or worktrees.
The browser/controller implementation plan continues after consolidation; Operating HOLD
is unchanged. This migration does not qualify managed execution or grant live authority.

The current user instruction explicitly adopts this plan and says:

> This migration changes repository identity and command naming. The browser/controller roadmap continues afterward, with Operating HOLD unchanged.

## Preservation and historical lookup

The [historical branch](https://github.com/neverhuman/bulletfarm/tree/archive/history)
contains `snapshots/<old-repository>/`, `records/<old-repository>/`, and a complete
`ref-index.json`. Historical instructions and reviews have no current authority.

| Original repository | Preserved main tag |
| --- | --- |
| `neverhuman/bulletfarm` | `archive/bulletfarm/heads/main` |
| `neverhuman/bf` | `archive/bf/heads/main` |
| `neverhuman/bullet-farm` | `archive/bullet-farm/heads/main` |
| `neverhuman/bullet-kernel` | `archive/bullet-kernel/heads/main` |
| `neverhuman/bullet-git` | `archive/bullet-git/heads/main` |
| `neverhuman/bullet-portal` | `archive/bullet-portal/heads/main` |

Each original `refs/<name>` maps to `refs/tags/archive/<old-repository>/<name>` at
its original object ID. Available PR heads use `archive/<old-repository>/pull/<n>/head`.
`local/` namespaces preserve local branches, remote-tracking refs and reflog commits.
Use the archive's `ref-index.json` for exact source revisions; exported PR records retain
their original numbers, titles, bodies, head SHAs, reviews, comments and check results.
GitHub URLs in those records are provenance and will stop resolving after deletion.
These exports are historical evidence, not new native GitHub approvals.

The pending documentation proposal is preserved at `archive/bf/pull/12/head`, exact
head `b9bd7227ab9ce3dde8959dd0082a3848ed778673`. Its two review exchanges are preserved
under `records/bf/pulls/12/`, with discussion in `records/bf/issue-comments.json`.
PR A carries its implementation plan, current user instructions and canonical specification
forward with repository references updated. The specification remains byte-for-byte:
SHA-256 `1d8064b40041e9383b66b790eb8eaa41924657db5f5d1bb0a5373fc5c501b9eb`.
Close the original PR as superseded only after its capture is verified.

Private mirrors and local working-state backups are retained outside this checkout.
Local configuration, credentials, databases and launchers are never published. Readable
JSON manifests identify exported records; compressed logs and artifacts retain their bytes
and SHA-256 checksums. Split archive files exceeding 50 MiB into ordered indexed parts.
Record expired or unavailable material explicitly. Failure to capture anything currently
available blocks retirement. The initial inventory contains 1,766 unexpired CI artifacts.

## Serialized changes

**PR A — establish this repository.** Start from the existing `bulletfarm/main`, preserve
all six source histories and browsable snapshots, then import the frozen application main
tree. Keep runtime source unchanged; record every repository-identity/documentation
exception in `source-import.json`. Replace the obsolete split manifest and private-plan
instructions. Preserve working CI, toolchain pins, dependency caching, bounded concurrency
and embedded-asset checks. Require GitHub Actions `check`, an up-to-date branch, linear
history and administrator enforcement. Merge A before beginning B.

**PR B — command and installation cutover.** Rename the package and primary binary to
`bulletfarm`, keep Rust library name `bf`, and install `bf` as an alias to the same executable.
Replace the former launcher pointing to `bullet`; privately preserve that binary and give
the obsolete entry point a retirement message. Both supported names keep `~/.bf` and
`BF_DATA_DIR`, with no database creation or copy for this rename. Update help, installation,
package metadata, UI branding, badges and active links. PR discovery maps only the five
retiring product identities above to `neverhuman/bulletfarm` and deduplicates requests;
unrelated repositories and historical note text/URLs retain their identities. A CI identity
check rejects retired URLs in active instructions/configuration, with explicit exceptions
for historical records, the preserved specification and deliberate compatibility tests.

Both PRs require local `scripts/check`, successful hosted CI and independent different-vendor
review of their final heads. Shared-account review comments are process review, not native
GitHub approvals. Material edits require affected checks and review again.

## Cutover and retirement gates

After both PRs merge, install the verified build, update parent workspace instructions and
active agent entry points, release old-path claims normally and acquire new-path claims.
Preserve the generated coordination digest, its symlink and history; never rewrite historical
claims or terminate other agents. Disable pushes from retired local checkouts and label them
historical. Publish the source/PR migration index, recheck every source ref and queue, and
capture any new work before proceeding. Deprecate and temporarily archive exactly the five
retiring repositories while final preservation checks run.

Deletion requires all of the following evidence:

- A fresh GitHub mirror contains every inventoried source ref at its original object ID and
  passes Git integrity checks.
- Reviews, logs and artifact bytes match manifests/checksums; an independent-backup
  restoration drill succeeds. Every currently available record has been captured.
- A fresh checkout passes `scripts/check`, hosted CI and embedded-asset verification.
- Installed `bulletfarm` and `bf` report the same version and expose the same data, board,
  claims and notes; the old launcher no longer starts the legacy product.
- Historical board entries cause neither duplicate PR requests nor requests to retired
  repositories. Active instructions/configuration identify one repository and checkout.
- No new source changes or work appeared after the final preservation inventory.

Only then obtain `delete_repo` through operator-assisted authentication and delete exactly
`bf`, `bullet-farm`, `bullet-kernel`, `bullet-git`, and `bullet-portal`. If authentication is
unavailable, leave them archived and report retirement incomplete. Never delete, replace,
or force-push away the existing `bulletfarm` identity/history. Finally verify the GitHub
inventory contains only `neverhuman/bulletfarm` for this product. Retain private backups.

Any failed gate stops retirement. Before deletion, restore the previous launcher if needed
and fix source through reviewed commits. Consolidation is incomplete until all gates pass;
archive tags alone do not establish completion. Resume the browser/controller roadmap only
after consolidation passes.
