# Fleet runbook

Status: Active
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: all bullet repos

Coordination is rooted at the outermost ancestor containing `repos.manifest.toml`; no public file
contains a host-specific absolute path. Human decisions remain append-only in
`<family-root>/AGENT_CHAT.md`. Machine claims are append-only in the ignored
`<family-root>/.bullet-family/coord/events.jsonl` ledger and are changed only through:

```bash
bullet-family coord claim --agent codex-a --lane L4 --repo bullet-kernel --path crates/runner
bullet-family coord heartbeat --claim clm_... --agent codex-a --note proof-started
bullet-family coord handoff --claim clm_... --agent codex-a \
  --proof 'cargo test --locked' --exit-code 0 --changed-path crates/runner/src/lib.rs
bullet-family coord receipt --claim clm_... --orchestrator codex-root \
  --commit 0123456789abcdef0123456789abcdef01234567 \
  --committed-path crates/runner/src/lib.rs
bullet-family coord receipt-group --claim clm_a... --claim clm_b... \
  --orchestrator codex-root --commit 0123456789abcdef0123456789abcdef01234567
bullet-family coord correct-receipt --claim clm_... --orchestrator codex-root \
  --previous-commit 0123456789abcdef0123456789abcdef01234567 \
  --commit 89abcdef0123456789abcdef0123456789abcdef \
  --committed-path crates/runner/src/lib.rs --reason 'amended commit after path-exact fixup'
bullet-family coord correct-receipt-group --claim clm_a... --claim clm_b... \
  --orchestrator codex-root \
  --previous-commit 0123456789abcdef0123456789abcdef01234567 \
  --commit 89abcdef0123456789abcdef0123456789abcdef \
  --reason 'split a contaminated shared-index commit into an exact replacement commit'
bullet-family coord status --json --all
```

The CLI takes an exclusive file lock across replay, overlap detection, and one append. Active claims
overlap when either repository-relative path contains the other on a segment boundary. Expired
claims stop blocking and cannot be revived; claim again. Handoff requires green proof and rejects
every changed path outside the claim. After an exact-path commit, the orchestrator records either a
single-claim `receipt` with every committed path or a `receipt-group` whose handed-off path union
exactly matches the commit. `correct-receipt` rebinds one already-receipted claim to a different
commit: it refuses unless `--previous-commit` equals the commit OID currently recorded on that claim
(`RECEIPT_CORRECTION_MISMATCH`), requires the `--committed-path` set to match the handed-off changed
paths (`COMMITTED_PATH_MISMATCH`) and to equal the new commit's actual path set, and appends a
`CommitReceiptCorrection` record carrying the mandatory `--reason`; nothing is rewritten or deleted.
`correct-receipt-group` applies the same append-only repair to at least two distinct claims from one
repository. Every claim must currently name the same `--previous-commit`; the replacement commit
must exactly equal their handed-off path union, and replay deterministically reconstructs each
claim's covered leaf set. Any mismatch rejects the whole locked append.
Run a heartbeat at least every five minutes and on proof, blocker, commit, or handoff.

Handed-off claims are committed by whichever orchestrator reaches them first (codex-root or
claude-orch): that orchestrator commits the claim path-exactly and records its receipt; the other
verifies the recorded commit OID and moves on instead of committing again
(`AGENT_CHAT.md`, 2026-08-25T05:46:14Z). A claim whose `commit_oid` is already set is never staged a
second time.

Only the orchestrator commits. Proof commands are per-repo
`bash scripts/ci-local.sh required` plus the lane-specific Cargo/npm commands. Zero Git worktrees,
zero pushes, zero new remotes.
Provider CLI execution is quarantined. `BULLET_LIVE_PROVIDERS`, provider OAuth state, and forge
tokens do not authorize a run. A signed launch-grant validator already exists (ADR 0011). Live
dispatch stays policy-disabled: committed policy is v1alpha1 / generation 1 /
`live_admission_enabled=false`. ADR 0012 ratification plus a Kernel loader mirror are operator
acts, not environment variables.
