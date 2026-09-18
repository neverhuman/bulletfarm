#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
#[test]
fn non_linux_setup_refuses_before_mutation() {
    use std::{ffi::OsString, fs};

    let fixture =
        std::env::temp_dir().join(format!("bullet-non-linux-refusal-{}", std::process::id()));
    if fixture.exists() {
        fs::remove_dir_all(&fixture).expect("remove stale fixture");
    }
    fs::create_dir_all(&fixture).expect("create fixture");
    let before = fs::read_dir(&fixture).expect("read fixture").count();
    let error = bullet_family::cli::run(
        [
            OsString::from("bullet-family"),
            OsString::from("setup"),
            OsString::from("--root"),
            fixture.clone().into_os_string(),
            OsString::from("--source"),
            OsString::from("jeryu"),
        ],
        Ok(fixture.clone()),
    )
    .expect_err("non-Linux setup must refuse before filesystem admission");
    assert_eq!(error.code(), "UNSUPPORTED_PLATFORM_CONTAINMENT");
    assert_eq!(
        fs::read_dir(&fixture).expect("read fixture").count(),
        before,
        "typed refusal changed the requested root"
    );
    fs::remove_dir_all(fixture).expect("remove fixture");
}
