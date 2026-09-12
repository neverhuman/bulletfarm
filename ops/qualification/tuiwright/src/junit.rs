//! JUnit describes observed completion, including incomplete and run-level failures.
use anyhow::{ensure, Result};
use serde_json::Value;

fn xml(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&quot;".into(),
            '\'' => "&apos;".into(),
            '\t' | '\n' | '\r' => c.to_string(),
            '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}' => {
                c.to_string()
            }
            _ => "\u{fffd}".into(),
        })
        .collect()
}

pub fn render(selected: &[String], completed: &[Value], failures: &[String]) -> Result<String> {
    ensure!(!selected.is_empty(), "JUNIT_EMPTY_SELECTION");
    let unique = selected.iter().collect::<std::collections::BTreeSet<_>>();
    ensure!(unique.len() == selected.len(), "JUNIT_DUPLICATE_SELECTION");
    let mut rows = std::collections::BTreeMap::new();
    for row in completed {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("JUNIT_ID_MISSING"))?;
        ensure!(
            selected.iter().any(|value| value == id),
            "JUNIT_UNSELECTED_COMPLETION"
        );
        ensure!(rows.insert(id, row).is_none(), "JUNIT_DUPLICATE_COMPLETION");
        ensure!(
            (row["outcome"] == "PASS" && row["error"].is_null())
                || (row["outcome"] == "FAIL" && row["error"].is_string()),
            "JUNIT_OUTCOME_INVALID"
        );
    }
    let mut cases = String::new();
    let mut failed = 0;
    let mut errors = 0;
    for id in selected {
        cases.push_str(&format!(
            "<testcase classname=\"operator-tui.scenario\" name=\"{}\">",
            xml(id)
        ));
        match rows.get(id.as_str()) {
            Some(row) if row["outcome"] == "PASS" => {}
            Some(row) => {
                failed += 1;
                cases.push_str(&format!(
                    "<failure type=\"SCENARIO_FAILED\">{}</failure>",
                    xml(row["error"].as_str().unwrap())
                ));
            }
            None => {
                errors += 1;
                cases.push_str(
                    "<error type=\"NOT_COMPLETED\">Selected scenario did not complete</error>",
                );
            }
        }
        cases.push_str("</testcase>\n");
    }
    cases.push_str("<testcase classname=\"operator-tui.integrity\" name=\"run_integrity\">");
    if !failures.is_empty() || completed.len() != selected.len() || failed != 0 {
        errors += 1;
        cases.push_str(&format!(
            "<error type=\"RUN_FAILED\">{}</error>",
            xml(&failures.join("\n"))
        ));
    }
    cases.push_str("</testcase>\n");
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites><testsuite name=\"operator-tui\" tests=\"{}\" failures=\"{failed}\" errors=\"{errors}\" skipped=\"0\">\n{cases}</testsuite></testsuites>\n", selected.len() + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn complete_run_preserves_identity_and_xml_text() {
        let id = "unicode λ<&\"\u{1}";
        let result = render(
            &[id.into()],
            &[json!({"id":id,"outcome":"PASS","error":null})],
            &[],
        )
        .unwrap();
        assert!(result.contains("tests=\"2\" failures=\"0\" errors=\"0\" skipped=\"0\""));
        assert!(result.contains("unicode λ&lt;&amp;&quot;�"));
        assert!(!result.contains('\u{1}'));
    }

    #[test]
    fn incomplete_execution_and_cleanup_failure_are_errors() {
        let result = render(
            &["a".into(), "b".into()],
            &[json!({"id":"a","outcome":"PASS","error":null})],
            &["cleanup failed".into()],
        )
        .unwrap();
        assert!(result.contains("errors=\"2\" skipped=\"0\""));
        assert!(result.contains("NOT_COMPLETED"));
        assert!(result.contains("cleanup failed"));
    }

    #[test]
    fn primary_failure_and_finalization_failure_both_survive() {
        let result = render(
            &["a".into()],
            &[json!({"id":"a","outcome":"FAIL","error":"primary <error>"})],
            &["subject changed".into()],
        )
        .unwrap();
        assert!(result.contains("failures=\"1\" errors=\"1\""));
        assert!(result.contains("primary &lt;error&gt;"));
        assert!(result.contains("subject changed"));
    }

    #[test]
    fn fabricated_duplicate_or_inconsistent_completions_refuse() {
        let pass = json!({"id":"a","outcome":"PASS","error":null});
        for (selected, rows) in [
            (vec![], vec![]),
            (vec!["a".into(), "a".into()], vec![]),
            (vec!["b".into()], vec![pass.clone()]),
            (vec!["a".into()], vec![pass.clone(), pass]),
            (
                vec!["a".into()],
                vec![json!({"id":"a","outcome":"PASS","error":"failure"})],
            ),
            (
                vec!["a".into()],
                vec![json!({"id":"a","outcome":"SKIP","error":null})],
            ),
        ] {
            assert!(render(&selected, &rows, &[]).is_err());
        }
    }
}
