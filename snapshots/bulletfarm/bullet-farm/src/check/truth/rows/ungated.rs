//! Product gaps that block the release without owning a `release.*` id
//! (`docs/assurance/product-gaps.md`: G4). Each row has the
//! same falsifiable shape as a gate row and says exactly which gate it blocks
//! through; none can be receipted here.

use super::{Owner, UngatedRow, none};
use crate::check::model::GateClass;

const THROUGH_G2: &str =
    "yes — through G2 (`release.transaction-demo`); no receipt here can count on its own";

pub(crate) const UNGATED: &[UngatedRow] = &[UngatedRow {
    gap_id: "G4",
    claim: "Public BulletGit `clone` still answers AUTHORITY_CONTRACT_UNAVAILABLE, and no immutable `bullet-wire` tag has been published for consumers to pin.",
    why: "Without positive online authority and a published wire tag no Candidate can be written by the production path, so the five-plane transaction (G2) cannot start.",
    acceptance: "Publish the immutable `bullet-wire` tag, land Kernel online reservation/settlement and BulletGit positive authority, and read one exact Candidate back through the public path inside the G2 transaction.",
    class: GateClass::Transaction,
    evidence: "COMPONENT only — dissociate clone, hostile-git, generations, preservation, and honest cleanup UNKNOWN (BulletGit `236f4ef`); no positive online authority and no published wire tag",
    owner: Owner::LocalThenExternal {
        offline: "Kernel online reservation/settlement and BulletGit positive authority (V1-S3)",
        external: "operator publishes the immutable `bullet-wire` tag",
    },
    next: none("`just contract` proves generated wire identity, not a published tag"),
    blocking: THROUGH_G2,
}];
