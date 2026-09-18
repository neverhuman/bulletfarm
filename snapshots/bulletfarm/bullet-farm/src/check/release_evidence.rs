//! Receipt admission for exactly one release gate: Rust 1.95 MSRV.

use std::path::Path;

use super::{executor::RepositorySet, subject::RepositorySubject};

mod admission;
mod schema;
mod verify;

pub(super) enum Evaluation {
    Absent,
    Rejected(String),
    Verified {
        detail: String,
        subjects: Vec<RepositorySubject>,
    },
}

pub(super) fn evaluate(hub: &Path) -> Evaluation {
    let admitted = match admission::load(hub) {
        Ok(None) => return Evaluation::Absent,
        Ok(Some(admitted)) => admitted,
        Err(error) => return Evaluation::Rejected(error.to_string()),
    };
    let repositories = match RepositorySet::discover(hub) {
        Ok(repositories) => repositories,
        Err(error) => return Evaluation::Rejected(error.to_string()),
    };
    let before = match repositories.capture_family() {
        Ok(subjects) => subjects,
        Err(error) => return Evaluation::Rejected(error.to_string()),
    };
    let verified = match verify::evaluate(&repositories, &admitted) {
        Ok(verified) => verified,
        Err(error) => return Evaluation::Rejected(error.to_string()),
    };
    match repositories.capture_family() {
        Ok(after) if after == before => Evaluation::Verified {
            detail: verified.detail,
            subjects: verified.subjects,
        },
        Ok(_) => Evaluation::Rejected(
            "MSRV evidence evaluation changed an exact family subject".to_owned(),
        ),
        Err(error) => Evaluation::Rejected(format!(
            "exact family subjects became unavailable after MSRV evidence evaluation: {error}"
        )),
    }
}

#[cfg(test)]
#[path = "release_evidence/tests.rs"]
mod tests;
