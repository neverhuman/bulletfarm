use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireError {
    code: &'static str,
    reason: String,
}

impl WireError {
    pub fn new(code: &'static str, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for WireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.reason)
    }
}

impl std::error::Error for WireError {}
