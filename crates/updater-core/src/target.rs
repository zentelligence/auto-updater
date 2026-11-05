use std::fmt;

use crate::error::CoreError;

/// A validated Rust target triple, such as `aarch64-apple-darwin`.
///
/// Never inferred from the running platform without going through the workspace's own target
/// detection; this type only guarantees the value is a non-empty, single-token identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TargetTriple(String);

impl TargetTriple {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for TargetTriple {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
            return Err(CoreError::EmptyTargetTriple);
        }
        Ok(TargetTriple(value.to_string()))
    }
}

impl TryFrom<String> for TargetTriple {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TargetTriple::try_from(value.as_str())
    }
}

impl From<TargetTriple> for String {
    fn from(value: TargetTriple) -> Self {
        value.0
    }
}

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[path = "target_test.rs"]
mod tests;
