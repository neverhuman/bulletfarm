use super::*;

fn renewed_command(
    fixture: &FacadeFixture,
    authorized_at: u64,
    expires_at: u64,
) -> (RecoveryCommand, recovery_manifest::TestAuthority) {
    let authority = recovery_manifest::test_authority_with_decision(
        &fixture.inspection,
        10,
        authorized_at,
        expires_at,
    )
    .unwrap();
    let authorization = fixture
        .source
        .family
        .path()
        .join(format!("renewed-authorization-{authorized_at}.json"));
    let signature = fixture
        .source
        .family
        .path()
        .join(format!("renewed-signature-{authorized_at}.json"));
    let provenance = fixture
        .source
        .family
        .path()
        .join(format!("renewed-provenance-{authorized_at}.json"));
    super::super::super::sealed::write(&authorization, &authority.authorization).unwrap();
    super::super::super::sealed::write(&signature, &authority.signature).unwrap();
    super::super::super::sealed::write(&provenance, &authority.provenance).unwrap();
    (
        RecoveryCommand {
            authorization,
            authorization_signature: signature,
            bootstrap_provenance: provenance,
            ..fixture.command.clone()
        },
        authority,
    )
}

fn assert_unmutated_legacy(fixture: &FacadeFixture) {
    let coord = fixture.source.family.path().join(".bullet-family/coord");
    assert!(!coord.join("CURRENT").exists());
    assert!(!coord.join("LOCK").exists());
    assert_eq!(
        fs::metadata(coord).unwrap().permissions().mode() & 0o7777,
        0o775
    );
}

#[test]
fn expiry_at_lower_authority_boundary_refuses_before_mutation() {
    for (unix_ms, boottime_ms) in [(100, 20), (20, 100)] {
        let fixture = fixture_command();
        schedule_authority_clock(unix_ms, boottime_ms, (1, 1));
        assert_eq!(
            error_code(execute_linux(
                fixture.source.family.path(),
                &fixture.command
            )),
            "RECOVERY_AUTHORIZATION_EXPIRED"
        );
        assert_unmutated_legacy(&fixture);
    }
}

#[test]
fn time_namespace_change_at_lower_authority_boundary_refuses_before_mutation() {
    let fixture = fixture_command();
    schedule_authority_clock(20, 20, (2, 2));
    assert_eq!(
        error_code(execute_linux(
            fixture.source.family.path(),
            &fixture.command
        )),
        "RECOVERY_TIME_NAMESPACE_CHANGED"
    );
    assert_unmutated_legacy(&fixture);
}

#[test]
fn signed_renewal_resumes_writer_wait_without_changing_generation() {
    let fixture = fixture_command();
    let waiting =
        execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| Ok(true))
            .unwrap_err();
    assert_eq!(waiting.code(), "COORD_RECOVERY_WRITER_WAIT");
    let expired = recovery_manifest::install_test_clock(100);
    assert_eq!(
        error_code(execute_with_writer_probe(
            fixture.source.family.path(),
            &fixture.command,
            |_| panic!("expired window must not probe writers"),
        )),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
    drop(expired);

    let (renewed, _authority) = renewed_command(&fixture, 101, 191);
    let _renewed_clock = recovery_manifest::install_test_clock(110);
    let resumed =
        execute_with_writer_probe(fixture.source.family.path(), &renewed, |_| Ok(false)).unwrap();
    assert_eq!(resumed.state, RecoveryExecutionState::ResumedAndPublished);
    assert_eq!(
        resumed.generation_id,
        fixture.expected.generation_id().as_str()
    );
}

#[test]
fn signed_renewal_resumes_exchange_interruption_without_changing_generation() {
    let fixture = fixture_command();
    crate::coord::generation::recovery::test_crash_at_exchange();
    assert_eq!(
        error_code(execute_with_writer_probe(
            fixture.source.family.path(),
            &fixture.command,
            |_| Ok(false),
        )),
        "COORD_RECOVERY_TEST_INTERRUPTION"
    );
    let expired = recovery_manifest::install_test_clock(100);
    assert_eq!(
        error_code(execute_with_writer_probe(
            fixture.source.family.path(),
            &fixture.command,
            |_| panic!("expired window must not probe writers"),
        )),
        "RECOVERY_AUTHORIZATION_EXPIRED"
    );
    drop(expired);

    let (renewed, _authority) = renewed_command(&fixture, 101, 191);
    let _renewed_clock = recovery_manifest::install_test_clock(110);
    let resumed =
        execute_with_writer_probe(fixture.source.family.path(), &renewed, |_| Ok(false)).unwrap();
    assert_eq!(resumed.state, RecoveryExecutionState::ResumedAndPublished);
    assert_eq!(
        resumed.generation_id,
        fixture.expected.generation_id().as_str()
    );
}

#[test]
fn wall_clock_rollback_and_wrong_legacy_modes_fail_closed() {
    let fixture = fixture_command();
    let rollback = recovery_manifest::install_test_clock_pair(9, 20);
    assert_eq!(
        error_code(execute_with_writer_probe(
            fixture.source.family.path(),
            &fixture.command,
            |_| panic!("rollback must not probe writers"),
        )),
        "RECOVERY_AUTHORIZATION_NOT_YET_VALID"
    );
    drop(rollback);

    for mode in [0o750, 0o770, 0o777] {
        fs::set_permissions(
            fixture.source.family.path().join(".bullet-family/coord"),
            fs::Permissions::from_mode(mode),
        )
        .unwrap();
        assert_eq!(
            error_code(execute_with_writer_probe(
                fixture.source.family.path(),
                &fixture.command,
                |_| panic!("wrong legacy root mode must not probe writers"),
            )),
            "INVALID_COORD_RECOVERY"
        );
    }
}

#[test]
fn unrelated_fresh_inspection_error_is_not_treated_as_a_resume() {
    let fixture = fixture_command();
    fail_next_fresh_inspection();
    assert_eq!(
        error_code(execute_with_writer_probe(
            fixture.source.family.path(),
            &fixture.command,
            |_| panic!("fresh inspection failure must not probe writers"),
        )),
        "COORD_RECOVERY_SOURCE_CHANGED"
    );
    assert_unmutated_legacy(&fixture);
}
