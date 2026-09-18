//! Structural validation of a corpus-coverage policy: shape, uniqueness,
//! bounds, and disposition/anchor compatibility. No filesystem access.

use super::model::{
    Anchor, CORPUS_KEYS, CorpusCoverageSpec, CorpusUnit, Disposition, MAX_ID_LEN, MAX_NOTE_LEN,
    MAX_UNIT_LEN, REPOS, SCHEMA, parse_wave,
};
use crate::coord::CoordError;
use std::collections::BTreeSet;

pub const CODE_SCHEMA: &str = "CORPUS_COVERAGE_SCHEMA";

fn schema_error(message: impl Into<String>) -> CoordError {
    CoordError::new(CODE_SCHEMA, message.into())
}

/// Validate the whole policy. Errors name the first offending row.
pub fn validate(spec: &CorpusCoverageSpec) -> Result<(), CoordError> {
    if spec.schema != SCHEMA {
        return Err(schema_error(format!(
            "schema must be {SCHEMA}, found {:?}",
            spec.schema
        )));
    }
    let declared: Vec<&str> = spec.corpus.iter().map(|d| d.key.as_str()).collect();
    if declared != CORPUS_KEYS {
        return Err(schema_error(format!(
            "corpus keys must be exactly {CORPUS_KEYS:?} in order, found {declared:?}"
        )));
    }
    for doc in &spec.corpus {
        validate_relative_path(&doc.path, &format!("corpus {}", doc.key))?;
        if doc.title.trim().is_empty() {
            return Err(schema_error(format!(
                "corpus {} has an empty title",
                doc.key
            )));
        }
    }
    if spec.units.is_empty() {
        return Err(schema_error("policy declares zero units"));
    }
    let mut ids = BTreeSet::new();
    for unit in &spec.units {
        validate_unit(unit)?;
        if !ids.insert(unit.id.as_str()) {
            return Err(schema_error(format!("duplicate unit id {}", unit.id)));
        }
    }
    Ok(())
}

fn validate_unit(unit: &CorpusUnit) -> Result<(), CoordError> {
    let id = unit.id.as_str();
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err(schema_error(format!(
            "unit id {id:?} must be 1..={MAX_ID_LEN} bytes"
        )));
    }
    if !id.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-' || b == b'_'
    }) {
        return Err(schema_error(format!(
            "unit id {id:?} may contain only [a-z0-9._-]"
        )));
    }
    let doc_prefix = format!("{}.", unit.doc);
    if !id.starts_with(&doc_prefix) {
        return Err(schema_error(format!(
            "unit id {id:?} must start with its doc key {doc_prefix:?}"
        )));
    }
    if !CORPUS_KEYS.contains(&unit.doc.as_str()) {
        return Err(schema_error(format!(
            "unit {id} names unknown doc {:?}",
            unit.doc
        )));
    }
    if unit.reference.trim().is_empty() {
        return Err(schema_error(format!("unit {id} has an empty ref")));
    }
    if unit.unit.trim().is_empty() || unit.unit.chars().count() > MAX_UNIT_LEN {
        return Err(schema_error(format!(
            "unit {id} text must be 1..={MAX_UNIT_LEN} chars"
        )));
    }
    if let Some(note) = &unit.note
        && (note.trim().is_empty() || note.chars().count() > MAX_NOTE_LEN)
    {
        return Err(schema_error(format!(
            "unit {id} note must be 1..={MAX_NOTE_LEN} chars"
        )));
    }
    validate_anchor(id, &unit.anchor)?;
    if let Some(partial) = &unit.partial {
        validate_anchor(id, partial)?;
        if !partial.is_code() {
            return Err(schema_error(format!(
                "unit {id} partial anchor must be a test or symbol"
            )));
        }
        if unit.disposition != Disposition::Planned {
            return Err(schema_error(format!(
                "unit {id} may carry a partial anchor only when PLANNED"
            )));
        }
    }
    let compatible = match unit.disposition {
        Disposition::Implemented => unit.anchor.is_code(),
        Disposition::Planned => matches!(unit.anchor, Anchor::Wave { .. }),
        Disposition::Superseded | Disposition::Refused => {
            matches!(unit.anchor, Anchor::Adr { .. })
        }
    };
    if !compatible {
        return Err(schema_error(format!(
            "unit {id} is {} but anchors a {} (implemented needs test|symbol, planned needs wave, superseded/refused need adr)",
            unit.disposition.label(),
            unit.anchor.kind()
        )));
    }
    Ok(())
}

fn validate_anchor(id: &str, anchor: &Anchor) -> Result<(), CoordError> {
    match anchor {
        Anchor::Test { repo, path, symbol } | Anchor::Symbol { repo, path, symbol } => {
            if !REPOS.contains(&repo.as_str()) {
                return Err(schema_error(format!(
                    "unit {id} anchors unknown repo {repo:?}"
                )));
            }
            validate_relative_path(path, &format!("unit {id} anchor"))?;
            if symbol.trim().is_empty() || symbol.len() > MAX_UNIT_LEN {
                return Err(schema_error(format!(
                    "unit {id} anchor symbol is empty or too long"
                )));
            }
            Ok(())
        }
        Anchor::Wave { value } => parse_wave(value)
            .map(|_| ())
            .ok_or_else(|| schema_error(format!("unit {id} wave {value:?} is not W0..=W11"))),
        Anchor::Adr { value } => {
            if !value.ends_with(".md")
                || value.contains('/')
                || !value.bytes().take(4).all(|b| b.is_ascii_digit())
            {
                return Err(schema_error(format!(
                    "unit {id} ADR {value:?} must be a four-digit-prefixed markdown file name"
                )));
            }
            Ok(())
        }
    }
}

fn validate_relative_path(path: &str, what: &str) -> Result<(), CoordError> {
    let hostile = path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..");
    if hostile {
        return Err(schema_error(format!(
            "{what} path {path:?} must be a clean relative path"
        )));
    }
    Ok(())
}
