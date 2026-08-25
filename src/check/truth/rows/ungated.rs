//! Product gaps that block the release without owning a `release.*` id
//! (`docs/assurance/product-gaps.md`: G4, G11, G13, G14, G15). Each row has the
//! same falsifiable shape as a gate row and says exactly which gate it blocks
//! through; none can be receipted here.

use super::{Owner, UngatedRow, none};
use crate::check::model::GateClass;

const THROUGH_G2: &str =
    "yes — through G2 (`release.transaction-demo`); no receipt here can count on its own";

pub(crate) const UNGATED: &[UngatedRow] = &[
    UngatedRow {
        gap_id: "G4",
        claim: "Public BulletGit `clone` still answers AUTHORITY_CONTRACT_UNAVAILABLE, and no immutable `bullet-wire` tag has been published for consumers to pin.",
        why: "Without positive online authority and a published wire tag no Candidate can be written by the production path, so the five-plane transaction (G2) cannot start.",
        acceptance: "Publish the immutable `bullet-wire` tag, land Kernel online reservation/settlement and BulletGit positive online authority, and read one exact Candidate back through the public path inside the G2 transaction.",
        class: GateClass::Transaction,
        evidence: "COMPONENT only — dissociate clone, hostile-git, generations, preservation, and honest cleanup UNKNOWN (BulletGit `236f4ef`); no positive online authority and no published wire tag",
        owner: Owner::LocalThenExternal {
            offline: "Kernel online reservation/settlement and BulletGit positive authority (V1-S3)",
            external: "operator publishes the immutable `bullet-wire` tag",
        },
        next: none("`just contract` proves generated wire identity, not a published tag"),
        blocking: THROUGH_G2,
    },
    UngatedRow {
        gap_id: "G11",
        claim: "Self-tuning optimization and evolutionary campaigns are explicitly post-V1; `evolutionary_authority=false` is the committed V1 policy.",
        why: "The frozen V1 scope requires the minimum typed cognitive plane, not an authority-bearing evolutionary runtime. Treating a preview diagnostic as a GA gate would fork that contract.",
        acceptance: "Keep evolutionary authority disabled through V1. A later release may implement recipes, archive, study, canary, and promotion behind the evolutionary-control contract; the policy bit changes last, by operator ratification.",
        class: GateClass::Component,
        evidence: "design only — `docs/architecture/evolutionary-control.md` and policy `route_policy.evolutionary_authority=false`; no recipe, archive, study, canary, or promotion code path",
        owner: Owner::Local(
            "post-V1 evolutionary-control engineering; never by flipping the policy bit first",
        ),
        next: none(
            "do not start a V1 campaign; the typed durable study and canary surfaces do not exist",
        ),
        blocking: "no for V1 — self-tuning optimization is post-V1; `linux-preview` surfaces an extra evolution diagnostic that cannot alter the canonical 26-gate GA contract",
    },
    UngatedRow {
        gap_id: "G13",
        claim: "Six of fifteen Portal spec surfaces (Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, Workspace Hygiene) have no durable ledger subject and render explicit UNKNOWN, and the Portal is not packaged or embedded.",
        why: "A projection without a ledger subject cannot be truthful, and a Portal served from a Vite preview is not the released product surface.",
        acceptance: "After G2/G3, add the missing ledger subjects, project all fifteen surfaces with watermark-bound truth, and embed the built Portal in the Rust distribution.",
        class: GateClass::Transaction,
        evidence: "COMPONENT only — the projected surfaces (Portal `95108e3`: unit, mocked-browser, and real-farmd lanes), CSRF/202, PENDING→UNKNOWN, SSE STALE; seven surfaces stay UNKNOWN and no packaged runtime exists",
        owner: Owner::Local("Portal + farmd owners after G2/G3 (V1-S5)"),
        next: none(
            "Portal `npm test` lanes are component evidence; no packaged Portal command exists",
        ),
        blocking: "yes — through G2 (truthful projection plane of `release.transaction-demo`) and G9 (embedded Portal in `release.package-matrix`)",
    },
    UngatedRow {
        gap_id: "G14",
        claim: "farmd's public surface is the authenticated command/snapshot/SSE subset plus six read-only projections (including revision-one Context Lineage), not the designed control plane; no signed dispatch settles a public command beyond PENDING, UNKNOWN, or FAILED.",
        why: "Until signed dispatch and the missing ledger subjects exist, every public command can only settle PENDING to UNKNOWN or FAILED, so a transaction cannot finish through the API.",
        acceptance: "After signed internal lease transport and the missing ledger subjects, serve the control-plane routes with authenticated 202 commands that reconcile past PENDING to their applied and independently checked states and read back exactly.",
        class: GateClass::Transaction,
        evidence: "COMPONENT only — loopback origin, no-wildcard CORS, command 202, ready/outbox/missions plus Fleet/Session/Merge/Quality/Audit snapshots (Kernel `529bad1`); no signed dispatch",
        owner: Owner::Local("Kernel API after signed dispatch (V1-S5)"),
        next: none("`bullet-farmd` serves the committed subset only"),
        blocking: THROUGH_G2,
    },
    UngatedRow {
        gap_id: "G15",
        claim: "CognitiveTask, SelectionGroup, Role, and Fusion are wire IDs or design, not durable scheduler objects; no routing or fusion decision can be replayed from persisted inputs.",
        why: "Provider selection that cannot be replayed cannot be audited, and a transaction whose routing provenance is not durable cannot be reconstructed after a crash.",
        acceptance: "After G2, persist the cognitive objects in the Kernel ledger (V1-S6) and replay one routing/fusion decision from persisted inputs with an exact receipt.",
        class: GateClass::Component,
        evidence: "design and wire shapes only — routing provenance records and the offline provider parsers; no durable scheduler object",
        owner: Owner::Local("Kernel V1-S6 after G2"),
        next: none("cognitive objects have no CLI surface"),
        blocking: THROUGH_G2,
    },
];
