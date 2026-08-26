# Runbooks

Status: **index; each runbook carries its own status**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25  
Component receipt baseline (minimum; replay current-head lanes before use): bullet-farm `d762f86` and the member checkpoints each runbook names

A runbook describes an operator procedure. It cannot bypass an API or policy gate, and its commands prove
only what the named evidence class says ([`../release.md`](../release.md) "Evidence classes"). Every runbook
states what it can and cannot do; if a step is not observable on the reviewing host, the runbook says so
instead of showing an invented output.

| Runbook | Scope | Can | Cannot |
| --- | --- | --- | --- |
| [`fleet.md`](fleet.md) | multi-agent coordination: claims, heartbeats, handoffs, receipts | enforce path-exclusive claims through `bullet-family coord`; record proof and commit receipts | commit (orchestrator only), authorize a live provider run, create worktrees |
| [`source-setup.md`](source-setup.md) | contributor bootstrap vs the blocked signed installer; setup transaction rules | run family proof from an existing canonical family; explain why `scripts/setup.sh` refuses on the schema-2 lock | install from a hub-only clone; authenticate the source wrapper; produce installer evidence |
| [`setup-recovery.md`](setup-recovery.md) | the drill after a setup crash or refusal: doctor → checkout verify → staging check → refused rerun | prove prior vs complete-next state; preserve partial state; show the exact outputs observed on this host | repair anything; reach the complete-next branch under the schema-2 lock; detect a same-UID Git race after the fact |
| [`schema-removal.md`](schema-removal.md) | every `UNSUPPORTED_SCHEMA` site and the disposable pre-1.0 rule | name the refusal per subject (lock, manifest, release manifest, coord log, policy, SQLite ledger); tell the operator what to keep | export, migrate, or remove anything — no such command exists; regenerate schema 3 without authenticated inputs |
| [`backup-restore.md`](backup-restore.md) | receipt-bound SQLite snapshot and quarantined restore | create an exact-digest snapshot with a receipt; restore into a new quarantined database | admit a restored database for production; sign the receipt; settle outbox/effect ambiguity |
| [`effect-reconciliation.md`](effect-reconciliation.md) | the offline half: the `PENDING…UNKNOWN` ladder and the authenticated worker route | settle a pending command to honest `UNKNOWN`/`FAILED` (observed on this host); explain the broker's adopt/retry-once/quarantine rule | reach `APPLIED`/`VERIFIED`; read back a live forge; adopt an effect identity-exactly (C9) |
| [`live-conformance.md`](live-conformance.md) | the local schema-3 policy/enrollment-anchor and receipt-admission predecessor, followed by the operator act and nightly real-mode lane | run the neutral refusal lane today; describe local anchor/onboarding/registration work and later keygen → policy ratification → one PONG turn | treat PONG-shaped diagnostic output as LIVE_PROOF before semantic admission; produce a live receipt without the operator act; certify a provider for release |
| [`signer-rotation.md`](signer-rotation.md) | launch-grant key lifecycle (exists) and release-signing custody (does not) | describe `authority keygen` semantics, validity/retention, revocation via policy, and the composed rotation | rotate with one command (none exists); rotate a release-signing key (none is provisioned); claim any rotation receipt |
| [`platform-refusal.md`](platform-refusal.md) | what each binary does off Linux GNU, with the exact reason code per refusal | name every fail-closed refusal in hub, Kernel, and BulletGit from source | show a non-Linux refusal observed on this host; provide the `release.platform-containment` receipt |

Runbooks the active closure roadmap still requires but that cannot be written truthfully yet, because the typed
commands do not exist: upgrade/rollback/uninstall and `SAFE_STOPPED` handling
([`../assurance/closure-roadmap.md`](../assurance/closure-roadmap.md) Waves 2 and 7). The old V1-S7 inventory in
[`v1-closure-plan.md`](../assurance/v1-closure-plan.md) is historical provenance. Open operator decisions that
gate several runbooks are registered once, in
[`../decisions/0013-operator-decision-register.md`](../decisions/0013-operator-decision-register.md). Terms
are defined once, in [`../glossary.md`](../glossary.md).
