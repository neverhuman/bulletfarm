//! Anchor resolution against the family checkout.
//!
//! Resolution is honest about what it cannot see: a sibling repository that
//! is absent from a hub-only checkout makes its anchors *unverifiable*, never
//! resolved. Unresolvable anchors in a present checkout are hard failures for
//! the caller to report.

use super::model::{Anchor, CorpusCoverageSpec, Disposition, parse_wave};
use std::path::{Path, PathBuf};

pub const DISPOSITION_ADR: &str = "0014-corpus-dispositions.md";
pub const ROADMAP: &str = "docs/assurance/closure-roadmap.md";
pub const DECISIONS_DIR: &str = "docs/decisions";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolution {
    pub resolved: usize,
    /// `(unit id, reason)` for anchors that should resolve but do not.
    pub unresolved: Vec<(String, String)>,
    /// `(unit id, repo)` for anchors into an absent sibling checkout.
    pub unverifiable: Vec<(String, String)>,
}

/// Resolve every anchor (and partial anchor). `family_root` is the directory
/// that contains the four member checkouts; the hub is `family_root/bullet-farm`.
pub fn resolve(family_root: &Path, spec: &CorpusCoverageSpec) -> Resolution {
    let hub = family_root.join("bullet-farm");
    let roadmap = std::fs::read_to_string(hub.join(ROADMAP)).ok();
    let register = std::fs::read_to_string(hub.join(DECISIONS_DIR).join(DISPOSITION_ADR)).ok();
    let mut out = Resolution::default();
    for unit in &spec.units {
        let anchors = std::iter::once(&unit.anchor).chain(unit.partial.iter());
        for anchor in anchors {
            match resolve_one(family_root, &hub, roadmap.as_deref(), anchor) {
                Ok(()) => out.resolved += 1,
                Err(Outcome::Unverifiable(repo)) => out.unverifiable.push((unit.id.clone(), repo)),
                Err(Outcome::Unresolved(reason)) => out.unresolved.push((unit.id.clone(), reason)),
            }
        }
        if matches!(
            unit.disposition,
            Disposition::Superseded | Disposition::Refused
        ) {
            match &register {
                Some(text) if text.contains(&format!("`{}`", unit.id)) => out.resolved += 1,
                Some(_) => out.unresolved.push((
                    unit.id.clone(),
                    format!("{DISPOSITION_ADR} does not list this unit"),
                )),
                None => out
                    .unresolved
                    .push((unit.id.clone(), format!("{DISPOSITION_ADR} is missing"))),
            }
        }
    }
    out
}

enum Outcome {
    Unverifiable(String),
    Unresolved(String),
}

fn resolve_one(
    family_root: &Path,
    hub: &Path,
    roadmap: Option<&str>,
    anchor: &Anchor,
) -> Result<(), Outcome> {
    match anchor {
        Anchor::Test { repo, path, symbol } | Anchor::Symbol { repo, path, symbol } => {
            let checkout = checkout_dir(family_root, hub, repo);
            if !checkout.is_dir() {
                return Err(Outcome::Unverifiable(repo.clone()));
            }
            let file = checkout.join(path);
            let text = std::fs::read_to_string(&file)
                .map_err(|_| Outcome::Unresolved(format!("{repo}/{path} is not readable")))?;
            if contains_symbol(&text, symbol) {
                Ok(())
            } else {
                Err(Outcome::Unresolved(format!(
                    "{repo}/{path} does not contain {symbol:?}"
                )))
            }
        }
        Anchor::Wave { value } => {
            let wave = parse_wave(value)
                .ok_or_else(|| Outcome::Unresolved(format!("wave {value:?} is malformed")))?;
            let heading = format!("### Wave {wave} ");
            match roadmap {
                Some(text) if text.contains(&heading) => Ok(()),
                Some(_) => Err(Outcome::Unresolved(format!(
                    "{ROADMAP} has no heading {heading:?}"
                ))),
                None => Err(Outcome::Unresolved(format!("{ROADMAP} is missing"))),
            }
        }
        Anchor::Adr { value } => {
            if hub.join(DECISIONS_DIR).join(value).is_file() {
                Ok(())
            } else {
                Err(Outcome::Unresolved(format!(
                    "{DECISIONS_DIR}/{value} is missing"
                )))
            }
        }
    }
}

fn checkout_dir(family_root: &Path, hub: &Path, repo: &str) -> PathBuf {
    if repo == "bullet-farm" {
        hub.to_path_buf()
    } else {
        family_root.join(repo)
    }
}

/// A symbol matches when it appears as a whole identifier (`fn name`, `name(`,
/// `name<`, `name {`) or as a quoted string title (`"title"`, `'title'`).
pub fn contains_symbol(text: &str, symbol: &str) -> bool {
    if symbol.is_empty() {
        return false;
    }
    let quoted = [
        format!("\"{symbol}\""),
        format!("'{symbol}'"),
        format!("`{symbol}`"),
    ];
    if quoted.iter().any(|q| text.contains(q)) {
        return true;
    }
    let bytes = text.as_bytes();
    let needle = symbol.as_bytes();
    let mut from = 0;
    while let Some(pos) = find(bytes, needle, from) {
        let before_ok = pos == 0 || !is_ident(bytes[pos - 1]);
        let end = pos + needle.len();
        let after_ok = end >= bytes.len() || !is_ident(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = pos + 1;
    }
    false
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}
