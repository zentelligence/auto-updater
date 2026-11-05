use std::path::PathBuf;

/// The single error enum for `updater-core`. Every variant is stable and matchable so callers in
/// other crates can react to a specific validation failure rather than a generic message.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("version {value:?} is not a valid semantic version")]
    InvalidVersion {
        value: String,
        #[source]
        source: semver::Error,
    },

    #[error("target triple must not be empty")]
    EmptyTargetTriple,

    #[error("installation root {path:?} must be an absolute path")]
    InstallationRootNotAbsolute { path: PathBuf },

    #[error("path {value:?} must be relative")]
    PathNotRelative { value: PathBuf },

    #[error("path {value:?} contains a parent-directory ('..') component")]
    PathTraversal { value: PathBuf },

    #[error("path {value:?} contains an unsupported root or prefix component")]
    PathInvalidComponent { value: PathBuf },

    #[error("path {value:?} has no components")]
    PathEmpty { value: PathBuf },

    #[error("resolved path {resolved:?} escapes installation root {root:?}")]
    PathEscapesRoot { resolved: PathBuf, root: PathBuf },

    #[error("release.json could not be parsed: {source}")]
    ReleaseMetadataDecode {
        #[source]
        source: serde_json::Error,
    },

    #[error("release.json declares duplicate command name {name:?}")]
    DuplicateCommandName { name: String },

    #[error("release.json declares no commands")]
    NoCommandsDeclared { product: String, version: String },

    #[error("update manifest could not be parsed: {source}")]
    ManifestDecode {
        #[source]
        source: serde_json::Error,
    },

    #[error("update manifest declares no assets")]
    NoAssetsDeclared,

    #[error("update manifest declares duplicate target {target:?}")]
    DuplicateManifestTarget { target: String },

    #[error("local state could not be parsed: {source}")]
    StateDecode {
        #[source]
        source: serde_json::Error,
    },

    #[error("{what} declares unsupported schema_version {found} (expected {expected})")]
    UnsupportedSchemaVersion {
        what: &'static str,
        found: u64,
        expected: u64,
    },

    #[error("{what} could not be serialised to JSON: {source}")]
    Encode {
        what: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("active pointer content {value:?} is not a valid version")]
    InvalidActivePointer { value: String },

    #[error("{value:?} is not a valid 64-character hexadecimal SHA-256 digest")]
    InvalidSha256Digest { value: String },

    #[error("asset url {url:?} is not an HTTPS url")]
    InsecureAssetUrl { url: String },

    #[error("I/O failure writing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
#[path = "error_test.rs"]
mod tests;
