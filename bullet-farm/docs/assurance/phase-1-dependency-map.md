# Gate 0 dependency map

Status: Enforced
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: V1-S1 wire/schema/IPC component map; not a second closure plan

| Assertion | Machine source | Enforcement | Proof |
| --- | --- | --- | --- |
| Hostile TEAM bytes are preserved and non-authoritative | `fixtures/hostile/team-original.bin` | generated digest manifest plus fixed forensic classification | `policy_registry::hostile_team_fixture_preserves_the_audited_bytes` |
| Policy and schema inputs are canonical | `contract-catalog.json`, registry, template | `decode_canonical_value` and strict typed decode | `canonical_hostile` suite |
| Hash preimages are unambiguous | `digest.rs` | domain validation and two length frames | `framing_and_domains_disambiguate_hostile_preimages` |
| Every canonical transaction/research record is published | `contract-catalog.json` | exact required-record set in `ContractCatalogV1::validate` | `policy_and_catalog_are_strict_complete_and_offline` |
| Unknown fields fail closed | catalog `unknown_fields=reject`; Rust `deny_unknown_fields` | strict schema generation and typed decoder | unknown-field mutation tests |
| Every C/EV control is visible exactly once | invariant registry | exact crosswalk-set equality | `registry_is_complete_tiered_and_phase_honest` |
| Future controls cannot claim enforcement | invariant lifecycle and first wave | registry semantic validator | policy registry suite |
| Live admission remains unavailable | committed policy is v1alpha1, generation 1, and `live_admission_enabled=false`; the Kernel loader also validates v1alpha2, but no v1alpha2 live policy is admitted | `PolicySnapshotV1::validate`; Kernel `policy_snapshot/live.rs`; ADR 0012 Proposed | offline policy tests plus family provider quarantine tests; the committed policy still refuses before spawn |
| Lease/fence/scope/freeze/restore interleavings are bounded | `LeaseFence.tla/.cfg` | TLC 1.7.4 pinned module/config/state lock; exact trace replay through Kernel SQLite leases and the domain mutation guard | `just model-check` and Kernel `lease_fence_trace_replays_against_sqlite_and_domain_guard` |
| Effect/check ambiguity is read back before adoption | `EffectCheck.tla/.cfg` | TLC 1.7.4 pinned module/config/state lock; exact traces replay through Kernel SQLite effect rows and the domain effect machine | `just model-check` and Kernel `effect_check_traces_replay_against_sqlite_and_effect_machine` |
| Generated policy/schema/client constants do not drift | contract catalog, registry, template | deterministic contract tool `check` mode and atomic family sync `check` mode | `committed_generated_contracts_have_zero_byte_drift` plus member required lanes |
| Consumers refuse altered policy/schema identities | Farm-generated Rust verifier and TypeScript identity binding | domain-separated exact-byte pin plus bundle-manifest generated-client hash | `generated_pin_accepts_only_exact_canonical_contract_bytes` and consumer contract tests |

No row proves signed authority, authentication, sandboxing, vector budgets, durable freeze, external
audit anchoring, or production effects. Those remain V1-S2..S8 and [`../release.md`](../release.md)
gates.
