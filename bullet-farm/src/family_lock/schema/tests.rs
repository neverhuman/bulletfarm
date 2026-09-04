use super::validate_jeryu_source;

#[test]
fn jeryu_source_accepts_the_canonical_smart_http_path() {
    for url in [
        "http://127.0.0.1:8787/git/root/bullet-kernel.git",
        "http://localhost:8787/git/root/bullet-kernel.git",
        "http://[::1]:8787/git/root/bullet-kernel.git",
        "https://jeryu.example/git/root/bullet-kernel.git",
        "ssh://jeryu.example/git/root/bullet-kernel.git",
    ] {
        validate_jeryu_source(url, "root/bullet-kernel")
            .unwrap_or_else(|error| panic!("canonical URL {url} was refused: {error}"));
    }
}

#[test]
fn jeryu_source_refuses_derived_or_ambiguous_paths() {
    for url in [
        "https://jeryu.example/root/bullet-kernel.git",
        "https://jeryu.example/api/v3/git/root/bullet-kernel.git",
        "https://jeryu.example/git/root/bullet-kernel",
        "https://jeryu.example/git/root/bullet-kernel.git/",
        "https://jeryu.example/git/root/other.git",
        "https://user@jeryu.example/git/root/bullet-kernel.git",
        "http://jeryu.example/git/root/bullet-kernel.git",
    ] {
        assert!(
            validate_jeryu_source(url, "root/bullet-kernel").is_err(),
            "ambiguous or unsafe URL was admitted: {url}"
        );
    }
}
