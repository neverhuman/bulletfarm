fn crosswalk(out: &mut String, facts: &Facts, report: &CheckReport) {
    let inventory_explanation = if report.profile() == Some("legacy-v1-26") {
        "For the legacy diagnostic G12 is inventory-only: `release.receipt-contracts` remains a static catalog row and no semantic registry is evaluated."
    } else {
        "For semantic profiles G12 deliberately shows both the inventory projection and its semantic-admission gate without counting either as evidence."
    };
    let _ = writeln!(
        out,
        "\n{CROSSWALK_HEADING}\n\nEvery G-id of `{}` is visible here: by selected gate rows, by the ungated section below, or by this exact profile inventory. {inventory_explanation} Titles are compiled; the crosswalk rows are read from the register and compared here.\n\n| Gap | Title | Answered by |\n| --- | --- | --- |",
        super::facts::PRODUCT_GAP_REGISTER,
    );
    let profile = report.profile().unwrap_or("unprofiled");
    for &(gap, title) in rows::PRODUCT_GAPS {
        let global_gates = rows::gates_for(gap);
        let gates = global_gates
            .iter()
            .copied()
            .filter(|id| report.gates().iter().any(|gate| gate.id() == *id))
            .collect::<Vec<_>>();
        let answer = if gap == rows::INVENTORY_GAP {
            let admission = if profile == "legacy-v1-26" {
                "diagnostic `release.receipt-contracts` row selected statically; `--receipts` contents ignored"
                    .to_owned()
            } else if gates.is_empty() {
                "semantic admission gate not selected".to_owned()
            } else {
                format!(
                    "semantic admission: {}; the requested profile-condition receipt must also pass, and neither half can substitute for the other",
                    gates
                        .iter()
                        .map(|id| format!("`{id}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!(
                "this `{profile}` inventory — {} selected gates, {} receipted; {admission} (historical diagnostic: `bullet-family check release --profile legacy-v1-26 --receipts ABSOLUTE_REGISTRY --report --portable`)",
                report.gates().len(),
                receipted(report)
            )
        } else if !gates.is_empty() {
            gates
                .iter()
                .map(|id| format!("`{id}`"))
                .collect::<Vec<_>>()
                .join(", ")
        } else if !global_gates.is_empty() {
            format!(
                "NOT SELECTED BY `{profile}` — independently owned by {}",
                global_gates
                    .iter()
                    .map(|id| format!("`{id}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else if rows::ungated_for(gap).is_some() {
            if gap == "G4"
                && !report
                    .gates()
                    .iter()
                    .any(|gate| gate.id() == "release.transaction-demo")
            {
                "ungated — not release-blocking for this profile because `release.transaction-demo` is not selected; see below".to_owned()
            } else {
                "ungated — release-blocking through the selected `release.transaction-demo`; see below".to_owned()
            }
        } else {
            "UNANSWERED".to_owned()
        };
        let _ = writeln!(out, "| {gap} | {title} | {answer} |");
    }
    let _ = writeln!(
        out,
        "\nAgreement with `{}`: {}",
        super::facts::PRODUCT_GAP_REGISTER,
        agreement(&facts.register)
    );
}

fn agreement(register: &Register) -> String {
    let table = match register {
        Register::Absent => {
            return "UNKNOWN — the register is absent from this checkout".to_owned();
        }
        Register::Unparsed(reason) => return format!("UNKNOWN — register unparsed: {reason}"),
        Register::Read(table) => table,
    };
    let diffs = rows::crosswalk_diffs(table);
    if diffs.is_empty() {
        format!(
            "YES — all {} crosswalk rows and the G-id list agree",
            table.gates.len()
        )
    } else {
        format!("NO — {}", diffs.join("; "))
    }
}

fn ungated(out: &mut String, report: &CheckReport) -> Result<(), CoordError> {
    let _ = writeln!(
        out,
        "\n{UNGATED_HEADING}\n\nProduct gaps with no `release.*` id. Each row says which gate it blocks through; none can be receipted here, and each is listed so that no G-id is invisible to machine output.\n"
    );
    for (index, row) in rows::UNGATED.iter().enumerate() {
        ungated_row(out, index + 1, row, report)?;
    }
    Ok(())
}

fn ungated_row(
    out: &mut String,
    number: usize,
    row: &UngatedRow,
    report: &CheckReport,
) -> Result<(), CoordError> {
    reads_closed(row.gap_id, &row.texts())?;
    let title = rows::gap_title(row.gap_id).unwrap_or("UNKNOWN GAP");
    let _ = writeln!(out, "{number}. **{}** — {} {title}", row.claim, row.gap_id);
    let _ = writeln!(out, "   - Why it matters: {}", row.why);
    let _ = writeln!(out, "   - Acceptance: {}", row.acceptance);
    let _ = writeln!(out, "   - Evidence class: {}_PROOF", row.class.as_str());
    let _ = writeln!(out, "   - Current evidence: {}", row.evidence);
    let _ = writeln!(out, "   - Owner: {}", row.owner.render());
    let _ = writeln!(out, "   - Next command: {}", row.next.render());
    let blocking = if row.gap_id == "G4"
        && !report
            .gates()
            .iter()
            .any(|gate| gate.id() == "release.transaction-demo")
    {
        "no for this profile — `release.transaction-demo` is not selected"
    } else {
        row.blocking
    };
    let _ = writeln!(out, "   - Release-blocking: {blocking}");
    Ok(())
}

fn excluded(out: &mut String) {
    let _ = writeln!(
        out,
        "\n## Excluded from this decision\n\n\
- Self-signed component receipts, including the `required.demo-component` simulator lane: COMPONENT_PROOF never counts toward a transaction, live, or release gate.\n\
- Simulator runs, fixture-only policies, and the fixture-only live-enabled v1alpha2 policy: none is an operator ratification.\n\
- Prose: `docs/release.md`, `docs/assurance/product-gaps.md`, `docs/assurance/v1-closure-plan.md`, ADRs, changelog, and coordination chat describe evidence; they are not evidence.\n\
- Component receipts named in `docs/release.md` and in the “exists but does not count” text above: COMPONENT_PROOF for one crate or surface, never transaction, live, or release evidence.\n\
- The archived 2026-08-24 live demonstration receipt at the family root `.l7-bundle/`: incident evidence only (`transaction_gate_eligible: false`); it is not counted.\n\
- Any mechanical gate result recorded against subjects other than the current HEADs (STALE above).\n\
- Reviewer checkmarks, browser-local marks, exit codes, model statements, pushed branches, and pull requests."
    );
}

fn freshness(out: &mut String, facts: &Facts, report: &CheckReport) {
    let _ = writeln!(
        out,
        "\n## Freshness\n\n| Input | Path | Identity | mtime (unix seconds) |\n| --- | --- | --- | --- |"
    );
    for input in &facts.inputs {
        let mtime = match (&input.mtime, facts.variant) {
            (Some(mtime), _) => mtime.clone(),
            (None, Variant::Portable) => "excluded (portable variant)".to_owned(),
            (None, Variant::Live) => "—".to_owned(),
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {mtime} |",
            input.label, input.path, input.identity
        );
    }
    let _ = writeln!(
        out,
        "| gate catalog | `src/check/prerequisites.rs` (compiled into `bullet-family {}`) | check report schema {} | — |",
        env!("CARGO_PKG_VERSION"),
        report.schema_version()
    );
    let _ = writeln!(
        out,
        "| claim rows | `src/check/truth/rows.rs` + `rows/gates.rs` + `rows/ungated.rs` (compiled into `bullet-family {}`) | {} gate rows + {} ungated rows over {} G-ids | — |",
        env!("CARGO_PKG_VERSION"),
        rows::row_count(),
        rows::UNGATED.len(),
        rows::PRODUCT_GAPS.len()
    );
}

fn code_or_unknown(value: Option<&str>) -> String {
    value.map_or_else(|| "UNKNOWN".to_owned(), |value| format!("`{value}`"))
}
