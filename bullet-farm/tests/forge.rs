//! Localhost-first forge verbs. Pin refuses unsigned tags. GitLab refuses.

use std::ffi::OsString;
use std::process::{Command, Output};

use bullet_family::forge::{
    SETUP_FORGE_ONLY_EXIT_CODE, execute, setup_forge_banner, setup_forge_only, should_intercept,
};
use serde_json::Value;

fn args(words: &[&str]) -> Vec<OsString> {
    std::iter::once("bullet-family")
        .chain(words.iter().copied())
        .map(OsString::from)
        .collect()
}

fn run(words: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bullet-family"))
        .args(words)
        .output()
        .expect("run bullet-family")
}

#[test]
fn forge_command_is_intercepted() {
    assert!(should_intercept(&args(&["forge", "status"])));
    assert!(!should_intercept(&args(&["doctor", "--json"])));

    let output = run(&["forge", "status"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 stdout");
    assert!(stdout.contains("classification: DIAGNOSTIC_ONLY"));
    assert!(stdout.contains("observation-status: UNPROBED"));
    assert!(stdout.contains("promotional: false"));
    assert!(stdout.contains("admission-eligible: false"));
    assert!(stdout.contains("receipt-status: ABSENT"));
    assert!(!stdout.contains("LIVE_PROOF"));
    assert!(!stdout.contains("capability_receipt"));
}

#[test]
fn setup_forge_local_is_banner_only() {
    let argv = args(&["setup", "--forge", "local"]);
    let banner = setup_forge_banner(&argv).expect("banner");
    assert!(banner.contains("127.0.0.1:8787"));
    assert!(banner.contains("RECOMMENDED AFTER INDEPENDENT ADMISSION"));
    assert!(banner.contains("classification: DIAGNOSTIC_ONLY"));
    assert!(banner.contains("observation-status: UNPROBED"));
    assert!(banner.contains("promotional: false"));
    assert!(banner.contains("admission-eligible: false"));
    assert!(banner.contains("receipt-status: ABSENT"));
    assert!(banner.contains("First-GA self-hosted-v1"));
    assert!(banner.contains("selected only by later universal-v1"));
    assert!(!banner.contains("requires BOTH"));
    assert!(!banner.contains("does not remove the GitHub requirement"));
    assert!(!banner.contains("LIVE_PROOF"));
    assert!(!banner.contains("capability_receipt"));
    assert!(setup_forge_only(&argv));

    let github = setup_forge_banner(&args(&["setup", "--forge", "github"])).expect("banner");
    assert!(github.contains("NOT self-hosted-v1"));
    assert!(!github.contains("First-GA self-hosted-v1 requires"));

    for profile in ["local", "github", "gitlab"] {
        let output = run(&["setup", "--forge", profile]);
        assert_eq!(
            output.status.code(),
            Some(i32::from(SETUP_FORGE_ONLY_EXIT_CODE)),
            "banner-only setup for {profile} must report BLOCKED"
        );
        assert!(output.stderr.is_empty());
        assert!(!output.stdout.is_empty());
    }
}

#[test]
fn setup_forge_gitlab_is_unsupported() {
    let banner = setup_forge_banner(&args(&["setup", "--forge", "gitlab"])).expect("banner");
    assert!(banner.contains("UNSUPPORTED_BY_ADAPTER"));
}

#[test]
fn pin_refuses_unsigned_tags() {
    let err = execute(
        args(&["forge", "pin", "--tag", "v1"]),
        Ok(std::env::temp_dir()),
    )
    .expect_err("unsigned");
    assert_eq!(err.code(), "UNSIGNED_FORGE_TAG");
}

#[test]
fn probe_without_url_prints_the_matrix() {
    let out = execute(args(&["forge", "probe"]), Ok(std::env::temp_dir())).expect("probe");
    assert_eq!(out.exit_code(), 0);
    let matrix: Value = serde_json::from_str(out.output()).expect("diagnostic JSON");
    assert_eq!(matrix["classification"], "DIAGNOSTIC_ONLY");
    assert_eq!(matrix["observation_status"], "UNPROBED");
    assert_eq!(matrix["promotional"], false);
    assert_eq!(matrix["admission_eligible"], false);
    assert_eq!(matrix["receipt_status"], "ABSENT");
    assert_eq!(
        matrix["profiles"]["local"]["observation_status"],
        "UNPROBED"
    );
    assert_eq!(
        matrix["profiles"]["github"]["observation_status"],
        "UNPROBED"
    );
    assert_eq!(
        matrix["profiles"]["local"]["declared_capabilities"]["merge_group"],
        "unsupported"
    );
    assert!(!out.output().contains("LIVE_PROOF"));
    assert!(!out.output().contains("capability_receipt"));

    let output = run(&["forge", "probe"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("UTF-8 stdout");
    let diagnostic: Value = serde_json::from_str(&stdout).expect("diagnostic JSON");
    assert_eq!(diagnostic["classification"], "DIAGNOSTIC_ONLY");
    assert_eq!(diagnostic["observation_status"], "UNPROBED");
    assert_eq!(diagnostic["admission_eligible"], false);
    assert_eq!(diagnostic["receipt_status"], "ABSENT");
    assert!(!stdout.contains("LIVE_PROOF"));
    assert!(!stdout.contains("capability_receipt"));
}

#[test]
fn live_probe_stays_unprobed() {
    let err = execute(
        args(&["forge", "probe", "--url", "http://127.0.0.1:8787"]),
        Ok(std::env::temp_dir()),
    )
    .expect_err("unprobed");
    assert_eq!(err.code(), "CAPABILITY_UNPROBED");

    let output = run(&["forge", "probe", "--url", "http://127.0.0.1:8787"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(stderr.contains("CAPABILITY_UNPROBED"));
    assert!(!stderr.contains("LIVE_PROOF"));
    assert!(!stderr.contains("capability_receipt"));
}
