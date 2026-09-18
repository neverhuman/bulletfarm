//! SPDX expression admission against the committed `deny.toml` allow-lists.

use std::{collections::BTreeSet, path::Path};

use serde::Deserialize;

use super::invalid;
use crate::coord::CoordError;

const MAX_DENY_BYTES: u64 = 1024 * 1024;
const MAX_TOKENS: usize = 64;

#[derive(Deserialize)]
struct DenyProbe {
    licenses: DenyLicenses,
}

#[derive(Deserialize)]
struct DenyLicenses {
    allow: Vec<String>,
}

/// One repository's reviewed license allow-list, read from its committed
/// `deny.toml` so this component can never drift from the scanned policy.
#[derive(Clone, Debug)]
pub(super) struct AllowList {
    pub(super) member: String,
    pub(super) allowed: BTreeSet<String>,
}

impl AllowList {
    pub(super) fn read(member: &str, repository: &Path) -> Result<Self, CoordError> {
        let path = repository.join("deny.toml");
        let metadata = std::fs::symlink_metadata(&path).map_err(CoordError::io)?;
        if !metadata.file_type().is_file() || metadata.len() == 0 || metadata.len() > MAX_DENY_BYTES
        {
            return Err(invalid(format!(
                "{member} has no bounded committed deny.toml license policy"
            )));
        }
        let bytes = std::fs::read(&path).map_err(CoordError::io)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| invalid(format!("{member} deny.toml must contain valid UTF-8")))?;
        let probe: DenyProbe = toml::from_str(text)
            .map_err(|error| invalid(format!("invalid {member} deny.toml: {error}")))?;
        if probe.licenses.allow.is_empty() {
            return Err(invalid(format!(
                "{member} deny.toml declares an empty license allow-list"
            )));
        }
        Ok(Self {
            member: member.to_owned(),
            allowed: probe.licenses.allow.into_iter().collect(),
        })
    }

    /// The reviewed permissive set of the whole family: a license admitted by
    /// any member's committed policy. Per-workspace enforcement is deliberately
    /// not used here because `cargo metadata` reports the unfiltered feature
    /// graph while `cargo deny` scans the resolved one, so a component such as
    /// `subtle 2.6.1` appears in the Hub's metadata without being reachable in
    /// the Hub's resolved features.
    pub(super) fn union(lists: &[Self]) -> Self {
        Self {
            member: lists
                .iter()
                .map(|list| list.member.as_str())
                .collect::<Vec<_>>()
                .join(" + "),
            allowed: lists
                .iter()
                .flat_map(|list| list.allowed.iter().cloned())
                .collect(),
        }
    }

    fn permits(&self, identifier: &str) -> bool {
        self.allowed.contains(identifier)
    }
}

/// Normalizes the legacy `A/B` slash form crates.io still carries into the
/// SPDX `A OR B` expression CycloneDX requires.
#[must_use]
pub(super) fn normalize(expression: &str) -> String {
    expression
        .split('/')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Refuses a component whose declared expression is empty, unparseable, or not
/// satisfiable from every allow-list that governs it. A missing license is a
/// failure, never a warning.
pub(super) fn admit(
    subject: &str,
    expression: &str,
    lists: &[&AllowList],
) -> Result<(), CoordError> {
    let expression = expression.trim();
    if expression.is_empty() {
        return Err(refused(format!(
            "{subject} declares no license; a component with no license cannot be released"
        )));
    }
    for list in lists {
        let tokens = tokenize(subject, expression)?;
        let mut cursor = Cursor {
            tokens: &tokens,
            index: 0,
        };
        let satisfied = cursor.expression(list)?;
        if cursor.index != tokens.len() {
            return Err(refused(format!(
                "{subject} license expression {expression:?} has trailing tokens"
            )));
        }
        if !satisfied {
            return Err(refused(format!(
                "{subject} license {expression:?} is outside the committed {} deny.toml allow-list",
                list.member
            )));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Token {
    Open,
    Close,
    And,
    Or,
    Identifier(String),
}

fn tokenize(subject: &str, expression: &str) -> Result<Vec<Token>, CoordError> {
    let spaced = expression.replace('(', " ( ").replace(')', " ) ");
    let mut tokens = Vec::new();
    for word in spaced.split_whitespace() {
        let token = match word {
            "(" => Token::Open,
            ")" => Token::Close,
            "AND" => Token::And,
            "OR" => Token::Or,
            "WITH" => match tokens.pop() {
                Some(Token::Identifier(left)) => Token::Identifier(format!("{left} WITH")),
                _ => {
                    return Err(refused(format!(
                        "{subject} license expression has a misplaced WITH"
                    )));
                }
            },
            other => {
                if !other
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
                {
                    return Err(refused(format!(
                        "{subject} license identifier {other:?} is not an SPDX identifier"
                    )));
                }
                match tokens.last() {
                    Some(Token::Identifier(left)) if left.ends_with(" WITH") => {
                        let joined = format!("{left} {other}");
                        tokens.pop();
                        Token::Identifier(joined)
                    }
                    _ => Token::Identifier(other.to_owned()),
                }
            }
        };
        tokens.push(token);
        if tokens.len() > MAX_TOKENS {
            return Err(refused(format!(
                "{subject} license expression exceeds its token bound"
            )));
        }
    }
    if tokens.is_empty() {
        return Err(refused(format!("{subject} license expression is empty")));
    }
    Ok(tokens)
}

struct Cursor<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl Cursor<'_> {
    fn expression(&mut self, list: &AllowList) -> Result<bool, CoordError> {
        let mut value = self.term(list)?;
        while self.tokens.get(self.index) == Some(&Token::Or) {
            self.index += 1;
            value = self.term(list)? || value;
        }
        Ok(value)
    }

    fn term(&mut self, list: &AllowList) -> Result<bool, CoordError> {
        let mut value = self.factor(list)?;
        while self.tokens.get(self.index) == Some(&Token::And) {
            self.index += 1;
            value = self.factor(list)? && value;
        }
        Ok(value)
    }

    fn factor(&mut self, list: &AllowList) -> Result<bool, CoordError> {
        match self.tokens.get(self.index) {
            Some(Token::Open) => {
                self.index += 1;
                let value = self.expression(list)?;
                if self.tokens.get(self.index) != Some(&Token::Close) {
                    return Err(refused("license expression has an unbalanced parenthesis"));
                }
                self.index += 1;
                Ok(value)
            }
            Some(Token::Identifier(identifier)) => {
                self.index += 1;
                Ok(list.permits(identifier))
            }
            _ => Err(refused("license expression is malformed")),
        }
    }
}

fn refused(reason: impl Into<String>) -> CoordError {
    CoordError::new("RELEASE_SBOM_LICENSE_REFUSED", reason)
}

#[cfg(test)]
mod tests {
    use super::{AllowList, admit, normalize};

    fn list() -> AllowList {
        AllowList {
            member: "test".to_owned(),
            allowed: [
                "MIT",
                "Apache-2.0",
                "Apache-2.0 WITH LLVM-exception",
                "Unicode-3.0",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        }
    }

    #[test]
    fn expressions_are_evaluated_exactly() {
        let list = list();
        let lists = [&list];
        for allowed in [
            "MIT",
            "MIT OR Apache-2.0",
            "CC0-1.0 OR MIT-0 OR Apache-2.0",
            "(MIT OR Apache-2.0) AND Unicode-3.0",
            "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
        ] {
            admit("subject", allowed, &lists).expect(allowed);
        }
        for refused in [
            "",
            "   ",
            "GPL-3.0-only",
            "MIT AND GPL-3.0-only",
            "(MIT OR Apache-2.0) AND CC-BY-4.0",
            "MIT OR",
            "WITH LLVM-exception",
        ] {
            assert_eq!(
                admit("subject", refused, &lists).unwrap_err().code(),
                "RELEASE_SBOM_LICENSE_REFUSED",
                "{refused}"
            );
        }
    }

    #[test]
    fn legacy_slash_form_becomes_an_spdx_expression() {
        assert_eq!(normalize("MIT/Apache-2.0"), "MIT OR Apache-2.0");
        assert_eq!(normalize("Apache-2.0 / MIT"), "Apache-2.0 OR MIT");
        assert_eq!(normalize("MIT"), "MIT");
    }
}
