use super::*;

fn gate(id: &str, status: GateStatus) -> GateResult {
    GateResult::new(
        id,
        status,
        if status == GateStatus::Neutral {
            GateClass::Live
        } else {
            GateClass::Component
        },
        "detail",
        matches!(
            status,
            GateStatus::Fail | GateStatus::Blocked | GateStatus::Unknown
        )
        .then(|| "repair".into()),
    )
    .unwrap()
}

#[test]
fn aggregation_is_fail_closed_and_tier_aware() {
    let cases = [
        (
            CheckTier::Fast,
            vec![gate("a", GateStatus::Pass)],
            GateStatus::Pass,
            0,
        ),
        (
            CheckTier::Fast,
            vec![gate("a", GateStatus::Pass), gate("b", GateStatus::Neutral)],
            GateStatus::Pass,
            0,
        ),
        (
            CheckTier::Release,
            vec![gate("a", GateStatus::Pass), gate("b", GateStatus::Neutral)],
            GateStatus::Unknown,
            1,
        ),
        (
            CheckTier::Fast,
            vec![gate("a", GateStatus::Neutral)],
            GateStatus::Neutral,
            1,
        ),
        (
            CheckTier::Required,
            vec![gate("a", GateStatus::Blocked)],
            GateStatus::Blocked,
            3,
        ),
        (
            CheckTier::Required,
            vec![
                gate("a", GateStatus::Blocked),
                gate("b", GateStatus::Unknown),
            ],
            GateStatus::Unknown,
            1,
        ),
        (
            CheckTier::Required,
            vec![gate("a", GateStatus::Blocked), gate("b", GateStatus::Fail)],
            GateStatus::Fail,
            1,
        ),
    ];
    for (tier, gates, status, exit) in cases {
        let report = CheckReport::new(tier, gates).unwrap();
        assert_eq!(report.status, status);
        assert_eq!(report.exit_code(), exit);
    }
}

#[test]
fn report_order_and_json_are_stable() {
    let report = CheckReport::new(
        CheckTier::Required,
        vec![
            gate("z-last", GateStatus::Pass),
            gate("a-first", GateStatus::Blocked),
        ],
    )
    .unwrap();
    assert_eq!(report.gates()[0].id(), "a-first");
    assert_eq!(report.gates()[1].id(), "z-last");
    assert_eq!(report.stable_json().unwrap(), report.stable_json().unwrap());
    assert_eq!(
        report.stable_json().unwrap(),
        serde_json::to_string(&report).unwrap()
    );
}

#[test]
fn empty_duplicate_and_missing_repair_are_rejected() {
    assert_eq!(
        CheckReport::new(CheckTier::Fast, vec![])
            .unwrap_err()
            .code(),
        "EMPTY_CHECK_REPORT"
    );
    assert_eq!(
        CheckReport::new(
            CheckTier::Fast,
            vec![
                gate("same", GateStatus::Pass),
                gate("same", GateStatus::Pass)
            ],
        )
        .unwrap_err()
        .code(),
        "DUPLICATE_GATE_ID"
    );
    assert_eq!(
        GateResult::new(
            "blocked",
            GateStatus::Blocked,
            GateClass::Release,
            "detail",
            None,
        )
        .unwrap_err()
        .code(),
        "MISSING_GATE_REPAIR"
    );
    for status in [GateStatus::Fail, GateStatus::Unknown] {
        assert_eq!(
            GateResult::new("failure", status, GateClass::Component, "detail", None,)
                .unwrap_err()
                .code(),
            "MISSING_GATE_REPAIR"
        );
    }
    assert_eq!(
        GateResult::new(
            "neutral-component",
            GateStatus::Neutral,
            GateClass::Component,
            "detail",
            None,
        )
        .unwrap_err()
        .code(),
        "INVALID_NEUTRAL_GATE"
    );
    assert!(GateResult::optional_live_neutral("optional-live", "not selected").is_ok());
}
