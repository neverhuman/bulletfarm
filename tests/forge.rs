//! Localhost-first forge verbs. Pin refuses unsigned tags. GitLab refuses.

use std::ffi::OsString;

use bullet_family::forge::{execute, setup_forge_banner, setup_forge_only, should_intercept};

fn args(words: &[&str]) -> Vec<OsString> {
    std::iter::once("bullet-family")
        .chain(words.iter().copied())
        .map(OsString::from)
        .collect()
}

#[test]
fn forge_command_is_intercepted() {
    assert!(should_intercept(&args(&["forge", "status"])));
    assert!(!should_intercept(&args(&["doctor", "--json"])));
}

#[test]
fn setup_forge_local_is_banner_only() {
    let argv = args(&["setup", "--forge", "local"]);
    let banner = setup_forge_banner(&argv).expect("banner");
    assert!(banner.contains("127.0.0.1:8787"));
    assert!(setup_forge_only(&argv));
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
    assert!(out.output().contains("\"merge_group\": \"unsupported\""));
}

#[test]
fn live_probe_stays_unprobed() {
    let err = execute(
        args(&["forge", "probe", "--url", "http://127.0.0.1:8787"]),
        Ok(std::env::temp_dir()),
    )
    .expect_err("unprobed");
    assert_eq!(err.code(), "CAPABILITY_UNPROBED");
}
