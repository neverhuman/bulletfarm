//! One falsifiable claim per registered release gate, plus the product gaps
//! that block a release without owning a gate id. Nightshift style: each
//! sentence names what remains unproved, never work remaining.
//!
//! Evidence class comes from the gate catalog, never from this table. Product
//! gap ids come from the G-table in `docs/assurance/product-gaps.md`. A claim
//! sentence must never read as closed while its gate is unreceipted; the unit
//! tests below enforce vocabulary, one-to-one catalog coverage, and that every
//! G-id is visible through a gate row, the ungated section, or the inventory
//! itself. G12 deliberately binds both the inventory and its admission gate.

mod gates;
mod ungated;

pub(super) use gates::{CONDITION_ROWS, NATIVE_ROWS, ROWS};
pub(super) use ungated::UNGATED;

use super::facts::RegisterTable;
use crate::check::model::GateClass;

/// The product gap register's G-ids and titles (`docs/assurance/product-gaps.md`).
pub(super) const PRODUCT_GAPS: &[(&str, &str)] = &[
    ("G1", "Hub-only signed install"),
    ("G2", "Connected five-plane transaction"),
    ("G3", "Production Kernel write path"),
    ("G4", "Production BulletGit write path"),
    ("G5", "Live provider conformance"),
    ("G6", "Jeryu live effect"),
    ("G7", "GitHub live effect"),
    ("G8", "Security release floor"),
    ("G9", "Signed profile-selected release"),
    ("G10", "Platform containment"),
    ("G11", "Evolutionary runtime"),
    ("G12", "Family `check release`"),
    ("G13", "Portal product surfaces"),
    ("G14", "farmd production API"),
    ("G15", "Cognitive persistence"),
    ("G16", "GitLab adapter effects"),
    ("G17", "Distributed team mode"),
    ("G18", "Cross-repository sagas"),
];

/// The gap that is this exact profile inventory and its semantic admission
/// machinery. `release.receipt-contracts` owns the machinery; the renderer
/// separately shows the inventory without counting either as evidence.
pub(super) const INVENTORY_GAP: &str = "G12";

/// Words that would make an unreceipted claim read as closed.
pub(super) const CLOSED_WORDS: &[&str] = &["verified", "proven", "done", "complete"];

/// Who can close a claim. The label set is closed; the per-row text says what
/// the offline part and the external predecessor are.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Owner {
    /// Closable offline from the checked-out family with admitted local tools.
    Local(&'static str),
    /// Closable offline only up to a predecessor that needs an operator
    /// credential, signer, service, platform, or the tagged release bytes.
    LocalThenExternal {
        offline: &'static str,
        external: &'static str,
    },
    /// Needs an operator credential, signer, service, or platform throughout.
    External(&'static str),
}

impl Owner {
    pub(super) const LABELS: [&'static str; 3] = ["LOCAL", "LOCAL-then-EXTERNAL", "EXTERNAL"];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Local(_) => Self::LABELS[0],
            Self::LocalThenExternal { .. } => Self::LABELS[1],
            Self::External(_) => Self::LABELS[2],
        }
    }

    pub(super) fn render(self) -> String {
        let label = self.label();
        match self {
            Self::Local(what) => format!("{label} (closable offline) — {what}"),
            Self::LocalThenExternal { offline, external } => format!(
                "{label} (closable offline only up to an external predecessor) — offline part: {offline}; external predecessor: {external}"
            ),
            Self::External(what) => format!(
                "{label} (needs operator credential, signer, service, or platform) — {what}"
            ),
        }
    }

    pub(super) fn texts(self) -> Vec<&'static str> {
        match self {
            Self::Local(what) | Self::External(what) => vec![what],
            Self::LocalThenExternal { offline, external } => vec![offline, external],
        }
    }
}

/// The exact typed command that moves a claim, or the honest absence of one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Next {
    pub command: Option<&'static str>,
    /// What the command does and does not produce today.
    pub note: &'static str,
}

impl Next {
    pub(super) const NONE_LABEL: &'static str = "NONE — no typed command exists yet";

    pub(super) fn render(self) -> String {
        match self.command {
            Some(command) => format!("`{command}` — {}", self.note),
            None => format!("{}; {}", Self::NONE_LABEL, self.note),
        }
    }
}

const fn command(command: &'static str, note: &'static str) -> Next {
    Next {
        command: Some(command),
        note,
    }
}

const fn none(note: &'static str) -> Next {
    Next {
        command: None,
        note,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ClaimRow {
    pub id: &'static str,
    /// G-ids from the product gap register, ascending.
    pub gap_ids: &'static [&'static str],
    pub claim: &'static str,
    pub why: &'static str,
    pub acceptance: &'static str,
    /// Component evidence that exists and must not be promoted.
    pub exists: &'static str,
    pub owner: Owner,
    pub next: Next,
}

/// A product gap that blocks the release without owning a `release.*` id.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UngatedRow {
    pub gap_id: &'static str,
    pub claim: &'static str,
    pub why: &'static str,
    pub acceptance: &'static str,
    /// Class of the first receipt that would count for this gap.
    pub class: GateClass,
    /// Current evidence; always component or design level here.
    pub evidence: &'static str,
    pub owner: Owner,
    pub next: Next,
    /// How this gap blocks the release decision, or that it does not for V1.
    pub blocking: &'static str,
}

impl ClaimRow {
    pub(super) fn texts(&self) -> Vec<&'static str> {
        let mut texts = vec![
            self.claim,
            self.why,
            self.acceptance,
            self.exists,
            self.next.note,
        ];
        texts.extend(self.owner.texts());
        texts
    }
}

impl UngatedRow {
    pub(super) fn texts(&self) -> Vec<&'static str> {
        let mut texts = vec![
            self.claim,
            self.why,
            self.acceptance,
            self.evidence,
            self.next.note,
            self.blocking,
        ];
        texts.extend(self.owner.texts());
        texts
    }
}

pub(super) fn all_rows() -> impl Iterator<Item = &'static ClaimRow> {
    ROWS.iter().chain(NATIVE_ROWS).chain(CONDITION_ROWS)
}

pub(super) const fn row_count() -> usize {
    ROWS.len() + NATIVE_ROWS.len() + CONDITION_ROWS.len()
}

pub(super) fn is_condition(id: &str) -> bool {
    CONDITION_ROWS.iter().any(|row| row.id == id)
}

pub(super) fn find(id: &str) -> Option<&'static ClaimRow> {
    all_rows().find(|row| row.id == id)
}

pub(super) fn gap_title(gap_id: &str) -> Option<&'static str> {
    PRODUCT_GAPS
        .iter()
        .find(|(id, _)| *id == gap_id)
        .map(|(_, title)| *title)
}

/// Gate ids whose row names `gap_id`, in catalog order.
pub(super) fn gates_for(gap_id: &str) -> Vec<&'static str> {
    all_rows()
        .filter(|row| row.gap_ids.contains(&gap_id))
        .map(|row| row.id)
        .collect()
}

pub(super) fn ungated_for(gap_id: &str) -> Option<&'static UngatedRow> {
    UNGATED.iter().find(|row| row.gap_id == gap_id)
}

/// Every disagreement between the register's crosswalk and the compiled rows.
pub(super) fn crosswalk_diffs(table: &RegisterTable) -> Vec<String> {
    let mut diffs = Vec::new();
    for row in all_rows() {
        match table.gates.iter().find(|(gate, _)| gate == row.id) {
            None => diffs.push(format!("`{}` is missing from the register", row.id)),
            Some((_, ids)) if ids != row.gap_ids => diffs.push(format!(
                "`{}`: page {} vs register {}",
                row.id,
                row.gap_ids.join(", "),
                ids.join(", ")
            )),
            Some(_) => {}
        }
    }
    for (gate, _) in &table.gates {
        if find(gate).is_none() {
            diffs.push(format!("register names unknown gate `{gate}`"));
        }
    }
    let compiled = PRODUCT_GAPS.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    if table.gap_ids != compiled {
        diffs.push(format!(
            "register G-ids {} vs page {}",
            table.gap_ids.join(", "),
            compiled.join(", ")
        ));
    }
    diffs
}

/// True when `text` contains one of [`CLOSED_WORDS`] as a whole word, ignoring case.
pub(super) fn contains_closed_word(text: &str) -> bool {
    text.split(|byte: char| !byte.is_ascii_alphanumeric())
        .any(|token| CLOSED_WORDS.contains(&token.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::prerequisites;

    fn gap_number(id: &str) -> u32 {
        id.strip_prefix('G')
            .unwrap()
            .bytes()
            .try_fold(0_u32, |number, byte| {
                number
                    .checked_mul(10)?
                    .checked_add(u32::from(byte.checked_sub(b'0')?))
            })
            .unwrap()
    }

    #[test]
    fn rows_cover_every_product_profile_exactly_once() {
        let rows = all_rows().map(|row| row.id).collect::<Vec<_>>();
        let mut reachable = std::collections::BTreeSet::new();
        for name in [
            "self-hosted-v1",
            "evolution-v1",
            "universal-v1",
            "team-v1",
            "saga-v1",
        ] {
            let profile = crate::check::profiles::ReleaseProfile::parse(name).unwrap();
            let report = prerequisites::report_release_profile(
                profile,
                std::path::Path::new("/nonexistent/bullet-release-truth-registry"),
            )
            .unwrap();
            let catalog = report
                .gates()
                .iter()
                .map(|gate| gate.id())
                .collect::<Vec<_>>();
            let unique = catalog
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(catalog.len(), unique.len(), "{name} duplicated a gate");
            for id in catalog {
                assert!(rows.contains(&id), "{name} gate has no claim row: {id}");
                reachable.insert(id.to_owned());
            }
        }
        for id in &rows {
            assert!(
                reachable.contains(*id),
                "claim row is unreachable from every product profile: {id}"
            );
        }
        assert!(ROWS.windows(2).all(|pair| pair[0].id < pair[1].id));
        assert!(NATIVE_ROWS.windows(2).all(|pair| pair[0].id < pair[1].id));
        assert!(
            CONDITION_ROWS
                .windows(2)
                .all(|pair| pair[0].id < pair[1].id)
        );
    }

    #[test]
    fn every_gate_names_a_gap_and_every_gap_is_answered_once() {
        let known = PRODUCT_GAPS.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        assert_eq!(known.len(), 18);
        assert!(
            known
                .windows(2)
                .all(|pair| gap_number(pair[0]) < gap_number(pair[1]))
        );
        for row in all_rows() {
            assert!(!row.gap_ids.is_empty(), "{} names no product gap", row.id);
            for gap in row.gap_ids {
                assert!(known.contains(gap), "{} names unknown gap {gap}", row.id);
            }
            assert!(
                row.gap_ids
                    .windows(2)
                    .all(|pair| gap_number(pair[0]) < gap_number(pair[1])),
                "{} gap ids are not ascending",
                row.id
            );
        }
        let ungated = UNGATED.iter().map(|row| row.gap_id).collect::<Vec<_>>();
        assert!(
            ungated
                .windows(2)
                .all(|pair| gap_number(pair[0]) < gap_number(pair[1]))
        );
        for gap in &ungated {
            assert!(known.contains(gap), "ungated row names unknown gap {gap}");
            assert!(gates_for(gap).is_empty(), "{gap} is both gated and ungated");
            assert_ne!(*gap, INVENTORY_GAP);
        }
        for gap in &known {
            let answered =
                !gates_for(gap).is_empty() || ungated_for(gap).is_some() || *gap == INVENTORY_GAP;
            assert!(answered, "{gap} is answered nowhere");
        }
        assert_eq!(gates_for(INVENTORY_GAP), ["release.receipt-contracts"]);
        assert!(ungated_for(INVENTORY_GAP).is_none());
    }

    #[test]
    fn unreceipted_claims_never_read_as_closed() {
        let sentences = all_rows()
            .flat_map(|row| {
                [
                    (row.id, row.claim),
                    (row.id, row.why),
                    (row.id, row.acceptance),
                ]
            })
            .chain(UNGATED.iter().flat_map(|row| {
                [
                    (row.gap_id, row.claim),
                    (row.gap_id, row.why),
                    (row.gap_id, row.acceptance),
                ]
            }));
        for (id, text) in sentences {
            assert!(text.ends_with('.'), "{id} field is not a sentence: {text}");
        }
        let texts = all_rows()
            .flat_map(|row| row.texts().into_iter().map(move |text| (row.id, text)))
            .chain(
                UNGATED
                    .iter()
                    .flat_map(|row| row.texts().into_iter().map(move |text| (row.gap_id, text))),
            );
        for (id, text) in texts {
            assert!(!text.trim().is_empty(), "{id} has an empty field");
            assert!(!contains_closed_word(text), "{id} reads as closed: {text}");
        }
        assert!(contains_closed_word("this is Done."));
        assert!(contains_closed_word("prior-or-complete-next"));
        assert!(!contains_closed_word("completeness verification verifier"));
    }

    #[test]
    fn next_commands_name_surfaces_that_exist() {
        let hub = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let justfile = std::fs::read_to_string(hub.join("Justfile")).unwrap();
        let recipes = justfile
            .lines()
            .filter_map(|line| line.split_once(':'))
            .filter(|(name, _)| !name.starts_with(' ') && !name.starts_with('['))
            .map(|(name, _)| name.split_whitespace().next().unwrap_or(""))
            .collect::<Vec<_>>();
        let nexts = all_rows()
            .map(|row| (row.id, row.next))
            .chain(UNGATED.iter().map(|row| (row.gap_id, row.next)));
        for (id, next) in nexts {
            assert!(
                !next.note.ends_with('.'),
                "{id} next note is not a fragment"
            );
            let Some(command) = next.command else {
                continue;
            };
            let local = command
                .strip_prefix("cd ../bullet-kernel && ")
                .map_or(command, |_| "");
            if let Some(recipe) = local.strip_prefix("just ") {
                assert!(
                    recipes.contains(&recipe),
                    "{id}: no Justfile recipe {recipe}"
                );
            } else if let Some(rest) = local.strip_prefix("bash ") {
                let script = rest.split_whitespace().next().unwrap();
                assert!(hub.join(script).is_file(), "{id}: no script {script}");
            } else if let Some(rest) = local.strip_prefix("bullet-family ") {
                let sub = rest.split_whitespace().next().unwrap();
                assert!(
                    ["check", "doctor", "lock", "release"].contains(&sub),
                    "{id}: unknown bullet-family subcommand {sub}"
                );
            } else {
                assert!(
                    local.is_empty(),
                    "{id}: unrecognised command shape {command}"
                );
                assert!(
                    command.contains("bash ops/ci/")
                        || command.contains("cargo run --locked -p bullet --"),
                    "{id}: kernel command must be a lane script or the bullet CLI: {command}"
                );
            }
        }
    }
}
