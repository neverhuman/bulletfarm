//! Corpus coverage: the committed policy validates, the committed page has no
//! drift, every anchor resolves where its checkout is present, and hostile
//! policies are refused with the typed schema code.

use bullet_family::check::corpus::{
    Disposition, PAGE_PATH, check_anchors, check_page, load, parse, render, validate,
};
use std::path::PathBuf;

fn hub() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn family_root() -> PathBuf {
    hub().parent().expect("hub has a parent").to_path_buf()
}

#[test]
fn committed_policy_validates_and_page_has_no_drift() {
    let spec = check_page(&hub()).expect("policy validates and page matches");
    assert!(!spec.units.is_empty());
    for disposition in Disposition::ALL {
        assert!(
            spec.units.iter().any(|u| u.disposition == disposition),
            "policy must exercise disposition {}",
            disposition.label()
        );
    }
}

#[test]
fn every_anchor_resolves_where_its_checkout_is_present() {
    let spec = load(&hub()).expect("policy loads");
    let resolution = check_anchors(&family_root(), &spec).expect("no unresolved anchors");
    eprintln!(
        "corpus-coverage anchors: resolved={} unverifiable(absent sibling)={}",
        resolution.resolved,
        resolution.unverifiable.len()
    );
    assert!(resolution.resolved > 0);
}

#[test]
fn page_states_addressed_and_implemented_separately() {
    let spec = load(&hub()).expect("policy loads");
    let page = render(&spec);
    assert!(page.contains("| Addressed | Implemented % |"));
    assert!(page.contains("this page holds no release, runtime, or scoring authority"));
    assert!(!page.contains("100/100"));
}

fn hostile(name: &str, json: &str) {
    let outcome = parse(json.as_bytes()).and_then(|spec| validate(&spec));
    let error = outcome
        .err()
        .unwrap_or_else(|| panic!("{name}: hostile policy was accepted"));
    let text = error.to_string();
    assert!(
        text.contains("CORPUS_COVERAGE_SCHEMA"),
        "{name}: expected CORPUS_COVERAGE_SCHEMA, got {text}"
    );
}

const CORPUS: &str = r#""corpus":[
 {"key":"spec","path":"docs/S.md","title":"S"},{"key":"git_role","path":"docs/g.md","title":"g"},
 {"key":"gastown","path":"docs/a.md","title":"a"},{"key":"nightshift","path":"docs/n.md","title":"n"},
 {"key":"potential","path":"docs/p.md","title":"p"},{"key":"paper","path":"docs/q.md","title":"q"},
 {"key":"evo","path":"docs/e.md","title":"e"}]"#;

fn policy(units: &str) -> String {
    format!(r#"{{"schema":"bullet-farm.corpus-coverage.v1",{CORPUS},"units":[{units}]}}"#)
}

const GOOD_IMPL: &str = r#"{"id":"spec.s1.a","doc":"spec","ref":"§1","unit":"u","disposition":"IMPLEMENTED","anchor":{"kind":"test","repo":"bullet-kernel","path":"crates/x.rs","symbol":"t"}}"#;

#[test]
fn hostile_policies_are_refused() {
    hostile(
        "wrong schema",
        &policy(GOOD_IMPL).replace("corpus-coverage.v1", "corpus-coverage.v2"),
    );
    hostile(
        "unknown top-level field",
        &policy(GOOD_IMPL).replace("\"units\"", "\"extra\":1,\"units\""),
    );
    hostile(
        "unknown row field",
        &policy(&GOOD_IMPL.replace("\"unit\":\"u\"", "\"unit\":\"u\",\"score\":100")),
    );
    hostile(
        "unknown disposition",
        &policy(&GOOD_IMPL.replace("IMPLEMENTED", "DONE")),
    );
    hostile(
        "missing disposition",
        &policy(&GOOD_IMPL.replace(",\"disposition\":\"IMPLEMENTED\"", "")),
    );
    hostile("duplicate id", &policy(&format!("{GOOD_IMPL},{GOOD_IMPL}")));
    hostile(
        "id without doc prefix",
        &policy(&GOOD_IMPL.replace("spec.s1.a", "s1.a")),
    );
    hostile(
        "implemented with wave anchor",
        &policy(&GOOD_IMPL.replace(
            r#"{"kind":"test","repo":"bullet-kernel","path":"crates/x.rs","symbol":"t"}"#,
            r#"{"kind":"wave","value":"W3"}"#,
        )),
    );
    hostile(
        "planned with adr anchor",
        &policy(&GOOD_IMPL.replace("IMPLEMENTED", "PLANNED").replace(
            r#"{"kind":"test","repo":"bullet-kernel","path":"crates/x.rs","symbol":"t"}"#,
            r#"{"kind":"adr","value":"0001-provider-execution-mode.md"}"#,
        )),
    );
    hostile(
        "refused with test anchor",
        &policy(&GOOD_IMPL.replace("IMPLEMENTED", "REFUSED")),
    );
    hostile(
        "wave out of range",
        &policy(&GOOD_IMPL.replace("IMPLEMENTED", "PLANNED").replace(
            r#"{"kind":"test","repo":"bullet-kernel","path":"crates/x.rs","symbol":"t"}"#,
            r#"{"kind":"wave","value":"W12"}"#,
        )),
    );
    hostile(
        "unknown repo",
        &policy(&GOOD_IMPL.replace("bullet-kernel", "bullet-forge")),
    );
    hostile(
        "path traversal",
        &policy(&GOOD_IMPL.replace("crates/x.rs", "../x.rs")),
    );
    hostile(
        "absolute path",
        &policy(&GOOD_IMPL.replace("crates/x.rs", "/etc/passwd")),
    );
    hostile("partial on implemented", &policy(&GOOD_IMPL.replace(
        "\"unit\":\"u\"",
        r#""unit":"u","partial":{"kind":"symbol","repo":"bullet-git","path":"a.rs","symbol":"s"}"#,
    )));
    hostile("empty units", &policy(""));
    hostile(
        "corpus out of order",
        &policy(GOOD_IMPL).replace("\"key\":\"spec\"", "\"key\":\"specx\""),
    );
    hostile(
        "adr with directory",
        &policy(&GOOD_IMPL.replace("IMPLEMENTED", "SUPERSEDED").replace(
            r#"{"kind":"test","repo":"bullet-kernel","path":"crates/x.rs","symbol":"t"}"#,
            r#"{"kind":"adr","value":"decisions/0001.md"}"#,
        )),
    );
}

/// Regeneration route used by `scripts/corpus-coverage.sh write`: with
/// `BULLET_CORPUS_COVERAGE_WRITE=1` it writes the page rendered from the
/// committed policy; otherwise it is the drift check, so ordinary test runs
/// never mutate the checkout and never skip.
#[test]
fn regenerate_page() {
    if std::env::var_os("BULLET_CORPUS_COVERAGE_WRITE").is_some() {
        let spec = load(&hub()).expect("policy loads");
        std::fs::write(hub().join(PAGE_PATH), render(&spec)).expect("page written");
    }
    check_page(&hub()).expect("page matches the policy");
}
