//! Validated exact Git subject carried by executable check receipts.

use serde::Serialize;

use super::model::CheckModelError;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(super) struct RepositorySubject {
    repository: String,
    commit_oid: String,
    tree_oid: String,
}

impl RepositorySubject {
    pub(super) fn new(
        repository: impl Into<String>,
        commit_oid: impl Into<String>,
        tree_oid: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        let subject = Self {
            repository: repository.into(),
            commit_oid: commit_oid.into(),
            tree_oid: tree_oid.into(),
        };
        if subject.repository.is_empty()
            || !subject
                .repository
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || !valid_git_oid(&subject.commit_oid)
            || !valid_git_oid(&subject.tree_oid)
        {
            return Err(CheckModelError::new(
                "INVALID_REPOSITORY_SUBJECT",
                "repository subjects require a validated name and algorithm-tagged Git OIDs",
            ));
        }
        Ok(subject)
    }

    pub(super) fn repository(&self) -> &str {
        &self.repository
    }
}

fn valid_git_oid(value: &str) -> bool {
    let Some((algorithm, digest)) = value.split_once(':') else {
        return false;
    };
    let expected = match algorithm {
        "sha1" => 40,
        "sha256" => 64,
        _ => return false,
    };
    digest.len() == expected
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithm_and_repository_are_validated() {
        assert!(
            RepositorySubject::new(
                "bullet-farm",
                format!("sha1:{}", "a".repeat(40)),
                format!("sha256:{}", "b".repeat(64)),
            )
            .is_ok()
        );
        for invalid in [
            ("Bullet", format!("sha1:{}", "a".repeat(40))),
            ("bullet", format!("sha1:{}", "A".repeat(40))),
            ("bullet", format!("sha256:{}", "a".repeat(40))),
            ("bullet", format!("md5:{}", "a".repeat(32))),
        ] {
            assert!(
                RepositorySubject::new(invalid.0, invalid.1, format!("sha1:{}", "b".repeat(40)))
                    .is_err()
            );
        }
    }
}
