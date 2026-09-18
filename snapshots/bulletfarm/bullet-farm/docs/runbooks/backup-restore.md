# SQLite backup and quarantined restore

Status: **offline maintenance available; production restore admission blocked**  
Owner: Bullet Farm operators  
Last reviewed: 2026-08-25

This runbook covers the committed local SQLite maintenance commands. A backup
receipt proves byte integrity and database subject checks; it is not signed
authenticity, authorization, or permission to resume production.

## Before backup

- Resolve the exact source ledger and new destination paths. Do not use a
  directory, symlink, glob, `$HOME`, or a path that already exists.
- Keep the output SQLite snapshot and JSON receipt together in operator-managed
  storage. The receipt is required for restore.
- Prefer an application quiescence window. The SQLite backup API includes
  committed WAL content, but this command does not freeze external effect or
  provider activity for you.
- Ensure the destination filesystem has enough capacity and an appropriate
  retention/encryption policy.

From the Kernel checkout, using the exact locally built `bullet` binary whose
subject you record:

```bash
bullet farm backup \
  --database /absolute/path/to/ledger.sqlite \
  --output /absolute/path/to/snapshot.sqlite \
  --receipt /absolute/path/to/snapshot.receipt.json
```

Both output paths must be absent. The library creates a standalone SQLite
snapshot, checks schema/foreign-key/integrity state, computes its exact BLAKE3
digest and size, and publishes without replacing an existing target. The thin
CLI then creates the separate receipt. Those two publications are not atomic:
a receipt-write failure can leave an orphan snapshot that is not restorable.
Keep the command output and completed pair as maintenance evidence.

## Validate custody

Before moving or restoring the pair:

1. retain the original snapshot and receipt read-only;
2. record the source host, exact Kernel binary subject, time, and operator;
3. copy through a storage mechanism that preserves exact bytes; and
4. do not edit the receipt or SQLite file to make validation pass.

The current receipt has no protected signer. An attacker able to replace both
snapshot and receipt may create a mutually consistent pair. Treat custody and
access control as an operator boundary until signed backup receipts exist.

## Restore into quarantine

Restore only to a new absent path:

```bash
bullet farm restore \
  --backup /absolute/path/to/snapshot.sqlite \
  --receipt /absolute/path/to/snapshot.receipt.json \
  --destination /absolute/path/to/restored.sqlite
```

The command verifies the declared size/digest and database checks, advances the
restore epoch, and publishes a new database with `pending_admission=true`.
Ordinary `SqliteLedger::open` refuses it with `RESTORE_ADMISSION_REQUIRED`.
Re-running against the same destination refuses and leaves existing bytes
unchanged.

There is deliberately no production-admission command yet. Do not clear the
quarantine flag manually, substitute the restored file for a live ledger, or
restart farmd against it. Preserve the returned restore receipt and wait for a
future signed admission workflow that freezes authority, reconciles external
effects, rotates the authority epoch, and independently verifies the subject.

## Failure handling

- A missing/short/grown snapshot, mismatched receipt, corrupt schema/FK state,
  existing destination, or I/O failure is a failed restore.
- If backup published a snapshot but failed to create its receipt, preserve the
  orphan for diagnosis. Do not invent a receipt, overwrite it, or restore it.
- If publication succeeded but final directory sync reported an error, outcome
  may be `UNKNOWN`. Inspect the exact destination and receipt without retrying
  over it; never infer failure from the error alone.
- Retain failed inputs for diagnosis. Do not mutate either file and retry under
  the same identity.
- Backup/restore does not settle pending outbox/effect ambiguity. That requires
  the future effect read-back/reconciliation workflow.

Executable component evidence lives in Kernel backup, migration, and
maintenance-CLI tests. Fault-complete recovery, signed authenticity, retention,
GC, and production restore admission remain release blockers.
