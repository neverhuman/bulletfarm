//! Deterministic Markdown rendering of the corpus-coverage page.

use super::model::{CorpusCoverageSpec, CorpusUnit, Disposition};
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub const TITLE: &str = "# Corpus coverage (generated)";
pub const PROJECTION: &str = "PROJECTION — rendered from `policy/corpus-coverage-v1.json`; the historical \
corpus is provenance, not authority; this page holds no release, runtime, or scoring authority";

/// Per-document disposition counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocSummary {
    pub units: usize,
    pub implemented: usize,
    pub planned: usize,
    pub superseded: usize,
    pub refused: usize,
}

impl DocSummary {
    fn add(&mut self, disposition: Disposition) {
        self.units += 1;
        match disposition {
            Disposition::Implemented => self.implemented += 1,
            Disposition::Planned => self.planned += 1,
            Disposition::Superseded => self.superseded += 1,
            Disposition::Refused => self.refused += 1,
        }
    }

    fn merge(&mut self, other: &Self) {
        self.units += other.units;
        self.implemented += other.implemented;
        self.planned += other.planned;
        self.superseded += other.superseded;
        self.refused += other.refused;
    }

    /// Units carrying any disposition. Equal to `units` for a valid policy —
    /// rendered explicitly so the page states the claim rather than implying it.
    pub const fn addressed(&self) -> usize {
        self.implemented + self.planned + self.superseded + self.refused
    }
}

/// Summaries keyed by doc key, in corpus order, plus the family total.
pub fn summarize(spec: &CorpusCoverageSpec) -> (Vec<(String, DocSummary)>, DocSummary) {
    let mut by_doc: BTreeMap<&str, DocSummary> = BTreeMap::new();
    for unit in &spec.units {
        by_doc
            .entry(unit.doc.as_str())
            .or_default()
            .add(unit.disposition);
    }
    let mut total = DocSummary::default();
    let ordered = spec
        .corpus
        .iter()
        .map(|doc| {
            let summary = by_doc.remove(doc.key.as_str()).unwrap_or_default();
            total.merge(&summary);
            (doc.key.clone(), summary)
        })
        .collect();
    (ordered, total)
}

pub fn render(spec: &CorpusCoverageSpec) -> String {
    let (per_doc, total) = summarize(spec);
    let mut out = String::new();
    let _ = writeln!(out, "{TITLE}\n");
    let _ = writeln!(out, "Status: **{PROJECTION}.**\n");
    let _ = writeln!(
        out,
        "Every unit carries exactly one disposition: `IMPLEMENTED` (a named test in a proof lane), \
`PLANNED` (a closure-roadmap wave), `SUPERSEDED` or `REFUSED` (a reviewed ADR, registered in \
`docs/decisions/0014-corpus-dispositions.md`). \"Addressed\" counts dispositions; \"implemented\" counts \
only `IMPLEMENTED`. The two are never conflated.\n"
    );
    let _ = writeln!(out, "## Totals\n");
    let _ = writeln!(
        out,
        "| Document | Units | Implemented | Planned | Superseded | Refused | Addressed | Implemented % |"
    );
    let _ = writeln!(
        out,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for (key, summary) in &per_doc {
        let title = spec
            .corpus
            .iter()
            .find(|d| &d.key == key)
            .map_or(key.as_str(), |d| d.title.as_str());
        total_row(&mut out, title, summary);
    }
    total_row(&mut out, "**Family total**", &total);
    out.push('\n');
    for doc in &spec.corpus {
        let mut rows: Vec<&CorpusUnit> = spec.units.iter().filter(|u| u.doc == doc.key).collect();
        rows.sort_by(|a, b| a.id.cmp(&b.id));
        let _ = writeln!(out, "## {} (`{}`)\n", doc.title, doc.path);
        let _ = writeln!(out, "| Ref | Unit | Disposition | Anchor | Note |");
        let _ = writeln!(out, "| --- | --- | --- | --- | --- |");
        for unit in rows {
            unit_row(&mut out, unit);
        }
        out.push('\n');
    }
    while out.ends_with('\n') {
        out.pop();
    }
    out.push('\n');
    out
}

fn total_row(out: &mut String, title: &str, s: &DocSummary) {
    let pct = if s.units == 0 {
        0.0
    } else {
        (s.implemented as f64) * 100.0 / (s.units as f64)
    };
    let _ = writeln!(
        out,
        "| {title} | {} | {} | {} | {} | {} | {}/{} | {pct:.1} |",
        s.units,
        s.implemented,
        s.planned,
        s.superseded,
        s.refused,
        s.addressed(),
        s.units
    );
}

fn unit_row(out: &mut String, unit: &CorpusUnit) {
    let mut anchor = unit.anchor.render();
    if let Some(partial) = &unit.partial {
        let _ = write!(anchor, "; partial {}", partial.render());
    }
    let _ = writeln!(
        out,
        "| {} | {} | `{}` | {} | {} |",
        cell(&unit.reference),
        cell(&unit.unit),
        unit.disposition.label(),
        anchor,
        unit.note.as_deref().map_or(String::new(), cell)
    );
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}
