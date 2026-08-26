# Schema removal — `UNSUPPORTED_SCHEMA` sites and what an operator can do

Status: **diagnostics only; no export or removal command exists**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25  
Component receipt baselines (minimum; replay current-head lanes before use): bullet-farm `347da232`, bullet-kernel `c4731aa` (SQLite ledger), the checked-in schema-2
`family.lock`

Current rule (retained from the historical
[`v1-closure-plan.md`](../assurance/v1-closure-plan.md) and governed now by
Waves 1–2 of the active [`closure roadmap`](../assurance/closure-roadmap.md)):
**pre-1.0 schemas are disposable.** An unknown or legacy database, lock,
manifest, or log fails closed with typed
`UNSUPPORTED_SCHEMA` plus explicit guidance. Nothing migrates silently and nothing is deleted for you.

## 1. Where the code refuses

| Subject | Site | Accepted | Refusal text / behaviour | Exit / status |
| --- | --- | --- | --- | --- |
| `family.lock` (install authority) | `src/family_lock/schema.rs` `load`/`validate` | schema `3` | "family.lock schema N is not installable; remove it or regenerate schema 3 from authenticated signed tags" | exit 4 (`src/coord/mod.rs`) |
| `family.lock` (diagnostic read) | `src/doctor/discovery.rs` | schema `3` read as current; `2` read as legacy diagnostic | any other version: "family.lock schema N is not supported" | `doctor` exit 3 with `source_metadata` `BLOCKED` for schema 2 |
| Hub `repos.manifest.toml` | `src/checkout.rs` | `schema_version = "1.2.0"`, family/umbrella `bullet-farm` | "hub manifest is not the supported Bullet Farm 1.2.0 schema" | exit 4 |
| Release manifest | `src/release/schema.rs` | `release_manifest_schema_version = "2"` binding `family_lock_schema_version = "3"`; five byte-sorted targets, each with separately signed archive/checksum/CycloneDX/SPDX/provenance subjects | own version: `UNSUPPORTED_RELEASE_MANIFEST_SCHEMA`; lock version: "release manifest must bind family.lock schema 3" | exit 2 / 4 |
| Coordination ledger `.bullet-family/coord/events.jsonl` | `src/coord/store.rs` | record `schema_version` == current | "line N uses an unsupported schema" | exit 4 |
| Policy snapshot | `crates/bullet-wire/src/policy.rs` | `v1alpha1`, `v1alpha2` | `UNSUPPORTED_POLICY_SCHEMA` (ADR 0012) | validator error |
| Kernel SQLite ledger | `crates/adapters/src/sqlite/mod.rs` `SqliteLedger::open`, `migrations.rs` | the exact disposable schema this binary owns | `LedgerError::UnsupportedSchema { detail }` before any mutation; farmd maps it to HTTP `412 UNSUPPORTED_SCHEMA` (`apps/bullet-farmd/src/errors.rs`) | daemon refuses to open |
| Restored Kernel ledger | `SqliteLedger::open` | admitted databases only | `RESTORE_ADMISSION_REQUIRED` while `pending_admission=true` | see [`backup-restore.md`](backup-restore.md) |

## 2. Observed on this host (2026-08-25, hub `347da232`)

```text
bullet-family doctor --json      -> status BLOCKED, exit 3
   source_metadata: "family.lock schema 2 is diagnostic-only and lacks the complete install authority"
   repair: "restore authenticated Jeryu sources, publish signed member tags, and generate schema 3 with
            exact trees, lockfiles, and artifact checksums"
bullet-family checkout verify    -> UNSUPPORTED_SCHEMA: family.lock schema 2 is not installable; remove it or
                                    regenerate schema 3 from authenticated signed tags   (exit 4)
scripts/setup.sh --offline       -> refuses first because no external pre-admitted binary/tool subjects were
                                    supplied; it never falls back to ambient Cargo or bullet-family
wrapper with all explicit        -> UNSUPPORTED_SCHEMA, exit 4, before any member or staging directory is created
absolute tool subjects
```

The Hub setup-refusal tests assert both boundaries: the wrapper refuses before
launch without explicit external subjects, and an explicitly selected component
reaches the schema-2 `UNSUPPORTED_SCHEMA` refusal before publication. A lock that
unexpectedly authorizes setup fails the lane.

ReleaseManifest v2 is independently disposable. Schema-1 release manifests now
fail with `UNSUPPORTED_RELEASE_MANIFEST_SCHEMA`; no compatibility parser silently
promotes them. V2's signed byte bindings remain structural-only until the
package, checksum, SBOM, and provenance semantic validators are admitted.

## 3. What an operator does today

### Family lock (schema 2)

- **Do not delete it.** `hub check` (`src/hub_check.rs` `REQUIRED_FILES`) and the required lane
  (`require_file "family.lock"`) fail without the file, and `doctor` needs it to prove the member set. Runtime
  guidance requires retaining it for diagnosis, regenerating schema 3 from authenticated signed tags, and
  replacing it atomically.
- **Regeneration is blocked.** `bullet-family lock generate --tag VERSION --subjects <absolute-path>` needs authenticated Jeryu URLs and
  slugs, signed member tags, exact trees, lockfiles, and artifact checksums. Those are operator inputs
  ([ADR 0013](../decisions/0013-operator-decision-register.md) OD-D). OD-B is later live-forge mutation
  custody and is not a predecessor. Until OD-D exists, use the
  refusal as the expected negative and cite `doctor --json` as the diagnostic.
- Never hand-edit `schema_version` to `3`; strict schema-3 decoding then fails on the missing signed
  subjects, and the required lane's negative assertion flips.

### Kernel SQLite ledger

- No export command exists. `farm backup` opens the database through the same admission and will refuse a
  schema it does not own, so an unsupported ledger cannot be snapshotted either.
- Retain the file read-only with its host/binary provenance. Do not open it with a newer or older `bullet`
  to "see what happens": open is fail-closed, but the honest record is the exact binary subject that refused.
- Disposable means a fresh data directory is the supported path forward (`bullet farm init` with `BULLET_DATA_DIR`
  pointing at a new absolute directory — the default is `./target/demo`); the old file is evidence, not state to migrate.
- A restored database is a separate case: it is quarantined by design, and the missing production-admission
  workflow is described in [`backup-restore.md`](backup-restore.md). Do not clear `pending_admission`.

### Coordination ledger

- `events.jsonl` is ignored by Git and owned by the family, not by one agent. An unsupported record refuses
  every `coord` command with exit 4. Do not edit or truncate it; report to the orchestrator, who decides
  whether the log is retired (all claims lost; every lane re-claims) or replayed by a matching binary.

## 4. What waits

| Missing | Notes |
| --- | --- |
| A schema-3 lock from authenticated signed non-Hub tags, committed before the Hub tag is signed | OD-D; gate `release.installable-lock` |
| A typed export/removal command for legacy lock, ledger, or log | required by the active roadmap's explicit export → verify → import and operational lifecycle work; not implemented — this document is the diagnostic half written today |
| Production restore admission | [`backup-restore.md`](backup-restore.md) |

Nothing here migrates, exports, or removes anything; every command above is read-only on this host.
