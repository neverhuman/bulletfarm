//! L-69: citations are `path::symbol`. A missing symbol fails the resolver.
//! Line-number citations are not anchors.

use bullet_family::check::corpus::anchors::{contains_symbol, resolve};
use bullet_family::check::corpus::model::CorpusDocument;
use bullet_family::check::corpus::{Anchor, CorpusCoverageSpec, CorpusUnit, Disposition};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn missing_symbol_is_unresolved() {
    let root = fixture("missing-symbol");
    write_hub_docs(&root);
    write_source(
        &root,
        "bullet-kernel/crates/domain/src/lib.rs",
        "pub fn present() {}\n",
    );
    let spec = spec(Anchor::Symbol {
        repo: "bullet-kernel".into(),
        path: "crates/domain/src/lib.rs".into(),
        symbol: "absent_symbol".into(),
    });
    let resolution = resolve(&root, &spec);
    assert_eq!(resolution.resolved, 0);
    assert_eq!(resolution.unresolved.len(), 1);
    assert!(
        resolution.unresolved[0]
            .1
            .contains("does not contain \"absent_symbol\"")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn present_path_symbol_resolves() {
    let root = fixture("present-symbol");
    write_hub_docs(&root);
    write_source(
        &root,
        "bullet-kernel/crates/domain/src/lib.rs",
        "pub fn named_anchor() {}\n",
    );
    let spec = spec(Anchor::Symbol {
        repo: "bullet-kernel".into(),
        path: "crates/domain/src/lib.rs".into(),
        symbol: "named_anchor".into(),
    });
    let resolution = resolve(&root, &spec);
    assert_eq!(resolution.unresolved, Vec::<(String, String)>::new());
    assert!(resolution.resolved >= 1);
    assert!(contains_symbol("pub fn named_anchor() {}", "named_anchor"));
    assert!(!contains_symbol("pub fn named_anchor() {}", "named"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn line_number_citations_are_not_symbol_anchors() {
    assert!(parse_path_symbol("crates/domain/src/lib.rs:42").is_err());
    assert!(parse_path_symbol("crates/domain/src/lib.rs#L42").is_err());
    assert_eq!(
        parse_path_symbol("crates/domain/src/lib.rs::named_anchor").unwrap(),
        ("crates/domain/src/lib.rs", "named_anchor")
    );
}

fn parse_path_symbol(citation: &str) -> Result<(&str, &str), &'static str> {
    let (path, symbol) = citation
        .rsplit_once("::")
        .ok_or("citation is not path::symbol")?;
    if path.is_empty()
        || symbol.is_empty()
        || path.contains(':')
        || symbol.contains(':')
        || path.ends_with(".rs") && symbol.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("line-number citations are not anchors");
    }
    if !symbol
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("symbol is not an identifier");
    }
    Ok((path, symbol))
}

fn spec(anchor: Anchor) -> CorpusCoverageSpec {
    CorpusCoverageSpec {
        schema: bullet_family::check::corpus::model::SCHEMA.into(),
        corpus: vec![CorpusDocument {
            key: "spec".into(),
            path: "docs/spec/README.md".into(),
            title: "fixture".into(),
        }],
        units: vec![CorpusUnit {
            id: "U1".into(),
            doc: "spec".into(),
            reference: "§1".into(),
            unit: "symbol citation".into(),
            disposition: Disposition::Planned,
            anchor,
            partial: None,
            note: None,
        }],
    }
}

fn write_hub_docs(root: &Path) {
    let hub = root.join("bullet-farm");
    fs::create_dir_all(hub.join("docs/assurance")).unwrap();
    fs::create_dir_all(hub.join("docs/decisions")).unwrap();
    fs::create_dir_all(hub.join("docs/spec")).unwrap();
    fs::write(hub.join("docs/spec/README.md"), "fixture\n").unwrap();
    fs::write(
        hub.join("docs/assurance/closure-roadmap.md"),
        "### Wave 1 operator\n",
    )
    .unwrap();
}

fn write_source(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bullet-doc-anchors-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("bullet-farm")).unwrap();
    root
}
