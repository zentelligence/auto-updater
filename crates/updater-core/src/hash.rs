use std::fmt;

use crate::error::CoreError;

/// A validated, lower-cased 64-character hexadecimal SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256Digest(String);

impl Sha256Digest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Sha256Digest {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(CoreError::InvalidSha256Digest {
                value: value.to_string(),
            });
        }
        Ok(Sha256Digest(value.to_ascii_lowercase()))
    }
}

impl TryFrom<String> for Sha256Digest {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Sha256Digest::try_from(value.as_str())
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Sha256Digest> for String {
    fn from(value: Sha256Digest) -> Self {
        value.0
    }
}

#[cfg(test)]
#[path = "hash_test.rs"]
mod tests;
