//! Scenario-specific termination observations; intentional failures stay explicit.
use super::*;
#[derive(Default)]
pub(super) struct Cleanup(BTreeMap<(String, String), usize>);
impl Cleanup {
    pub(super) fn observe(&mut self, case: &str, data: &Value) -> Result<()> {
        let kind = text(data, "kind")?;
        *self.0.entry((case.into(), kind.into())).or_default() += 1;
        match kind {
            "process_settlement" => {
                ensure!(
                    data["reaped"] == true && data["terminal_read_error"].is_null(),
                    "EVIDENCE_CLEANUP_UNREAPED"
                );
                let (code, signal, restored, requested) = match case {
                    "custody_nonce_rejection" => (json!(1), Value::Null, false, true),
                    "custody_exec_failure" => (json!(1), Value::Null, true, false),
                    "custody_nonzero_exit" => (json!(19), Value::Null, true, false),
                    "custody_detach_timeout" => (Value::Null, json!(9), false, true),
                    "custody_partial_terminal_restore" => (json!(0), Value::Null, false, false),
                    _ if crate::cases::IDENTITIES.contains(&case)
                        || case == "custody_normal_detach" =>
                    {
                        (json!(0), Value::Null, true, false)
                    }
                    _ => anyhow::bail!("EVIDENCE_CLEANUP_CASE_UNKNOWN"),
                };
                let expected_exit = (data["code"] == code && data["signal"] == signal)
                    || (case == "custody_nonce_rejection"
                        && data["code"].is_null()
                        && data["signal"] == 9);
                ensure!(
                    expected_exit
                        && data["terminal_restored"] == restored
                        && data["termination_requested"] == requested,
                    "EVIDENCE_CLEANUP_EXIT"
                );
            }
            "http_fixture_settled" => ensure!(
                crate::cases::IDENTITIES.contains(&case)
                    && data["threads_joined"] == true
                    && data["errors"] == json!([])
                    && data["reads"].as_u64().is_some_and(|n| n > 0),
                "EVIDENCE_HTTP_CLEANUP"
            ),
            "preconstruction_cleanup" => ensure!(
                case == "custody_after_page_failure"
                    && data["reaped"] == true
                    && ((data["code"].is_null() && data["signal"] == 9)
                        || (data["code"] == 1 && data["signal"].is_null())),
                "EVIDENCE_PRECONSTRUCTION_CLEANUP"
            ),
            "parent_fixture_exit" => ensure!(
                case == "custody_parent_death"
                    && data["diagnostic_error"].is_null()
                    && data["status"] == "Ok(ExitStatus(unix_wait_status(9)))"
                    && data["stderr"] == "",
                "EVIDENCE_PARENT_CLEANUP"
            ),
            "failure_screen" => ensure!(
                case == "custody_nonce_rejection"
                    && data["text"].is_string()
                    && data["cols"].as_u64().is_some_and(|n| n > 0)
                    && data["rows"].as_u64().is_some_and(|n| n > 0),
                "EVIDENCE_FAILURE_SCREEN"
            ),
            _ => anyhow::bail!("EVIDENCE_CLEANUP_UNKNOWN"),
        }
        Ok(())
    }
    pub(super) fn finish(self, selected: &[String]) -> Result<()> {
        let mut expected = BTreeMap::new();
        for case in selected {
            let rows: Vec<(&str, usize)> = match case.as_str() {
                "custody_nonce_rejection" => vec![("failure_screen", 1), ("process_settlement", 1)],
                "custody_after_page_failure" => vec![("preconstruction_cleanup", 1)],
                "custody_parent_death" => vec![("parent_fixture_exit", 1)],
                "custody_exec_failure"
                | "custody_nonzero_exit"
                | "custody_detach_timeout"
                | "custody_partial_terminal_restore"
                | "custody_normal_detach" => vec![("process_settlement", 1)],
                "six_withheld_http" | "six_locked_credentials" | "six_missing_credentials" => {
                    vec![("process_settlement", 6), ("http_fixture_settled", 1)]
                }
                "revoked_owner_recovery" => {
                    vec![("process_settlement", 2), ("http_fixture_settled", 2)]
                }
                "bare_alias_first_launch" => vec![("process_settlement", 1)],
                _ => anyhow::bail!("EVIDENCE_CLEANUP_CASE_UNDECLARED"),
            };
            for (kind, count) in rows {
                expected.insert((case.clone(), kind.into()), count);
            }
        }
        ensure!(self.0 == expected, "EVIDENCE_CLEANUP_INCOMPLETE");
        Ok(())
    }
}
