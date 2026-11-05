use std::cmp::Ordering;
use std::fmt;

use crate::error::CoreError;

/// A validated semantic version identifying one release set.
///
/// Wraps [`semver::Version`] so every other crate in the workspace compares, orders, and
/// serialises release versions through one project type rather than depending on `semver`
/// directly at call sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProductVersion(semver::Version);

impl ProductVersion {
    /// Returns the wrapped semantic version for comparison or formatting.
    pub fn as_semver(&self) -> &semver::Version {
        &self.0
    }
}

impl TryFrom<&str> for ProductVersion {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        semver::Version::parse(value)
            .map(ProductVersion)
            .map_err(|source| CoreError::InvalidVersion {
                value: value.to_string(),
                source,
            })
    }
}

impl TryFrom<String> for ProductVersion {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ProductVersion::try_from(value.as_str())
    }
}

impl From<ProductVersion> for String {
    fn from(value: ProductVersion) -> Self {
        value.0.to_string()
    }
}

impl fmt::Display for ProductVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for ProductVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProductVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(test)]
#[path = "version_test.rs"]
mod tests;
