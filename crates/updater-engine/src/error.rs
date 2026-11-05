use std::path::PathBuf;

use updater_core::CoreError;

/// The single error enum for `updater-engine`, covering every stage of the update transaction:
/// manifest retrieval, signature verification, download, checksum verification, extraction,
/// bundle and executable validation, promotion, activation, rollback, cleanup, state management,
/// and locking.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("manifest url {url:?} is not an HTTPS url")]
    InsecureManifestUrl { url: String },

    #[error("failed to retrieve {url:?}: {reason}")]
    Transport { url: String, reason: String },

    #[error("response from {url:?} exceeded the maximum allowed size of {limit} bytes")]
    ResponseTooLarge { url: String, limit: u64 },

    #[error("manifest signature verification failed")]
    ManifestSignatureInvalid,

    #[error("signature envelope is invalid: {reason}")]
    InvalidSignatureEnvelope { reason: String },

    #[error("no trusted public key matches signing key id {key_id:?}")]
    UnknownSigningKey { key_id: String },

    #[error("trusted key {key_id:?} is not a valid Ed25519 public key: {reason}")]
    InvalidTrustedKey { key_id: String, reason: String },

    #[error("no release archive is declared for detected target {target}")]
    NoMatchingAsset { target: String },

    #[error("downloaded archive sha256 {actual} does not match manifest sha256 {expected}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("downloaded archive size {actual} does not match the manifest-declared size {expected}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("archive entry {entry:?} could not be read: {reason}")]
    ArchiveEntryUnreadable { entry: String, reason: String },

    #[error("archive has an unsupported extension: {path:?}")]
    UnsupportedArchiveFormat { path: PathBuf },

    #[error("release product {found:?} does not match the installed product {expected:?}")]
    ProductMismatch { expected: String, found: String },

    #[error("release version {found} does not match the signed manifest version {expected}")]
    VersionMismatch { expected: String, found: String },

    #[error("release target {found} does not match the detected platform target {expected}")]
    TargetMismatch { expected: String, found: String },

    #[error("declared command {name:?} is missing its executable at {path:?}")]
    MissingCommandExecutable { name: String, path: PathBuf },

    #[error("declared command {name:?} at {path:?} lacks the required executable permission")]
    NotExecutable { name: String, path: PathBuf },

    #[error("command {name:?} reported version {reported}, expected {expected}")]
    ExecutableVersionMismatch {
        name: String,
        reported: String,
        expected: String,
    },

    #[error("failed to run command {name:?} for version validation: {reason}")]
    ExecutableProbeFailed { name: String, reason: String },

    #[error("stable launcher {name:?} has no corresponding declared command")]
    LauncherWithoutCommand { name: String },

    #[error("failed to promote staged release to {target:?}: {source}")]
    Promotion { target: PathBuf, source: std::io::Error },

    #[error("version {version} is already installed at {path:?}")]
    VersionAlreadyInstalled { version: String, path: PathBuf },

    #[error("failed to activate release {version}: {source}")]
    Activation { version: String, source: std::io::Error },

    #[error("no active release is installed under {root:?}")]
    NoActiveRelease { root: PathBuf },

    #[error("no previous release is available to roll back to")]
    NoPreviousRelease,

    #[error("failed to acquire the update lock at {path:?}: {reason}")]
    LockUnavailable { path: PathBuf, reason: String },

    #[error("failed to read local state at {path:?}: {source}")]
    ReadState { path: PathBuf, source: std::io::Error },

    #[error("failed to write to {path:?}: {source}")]
    AtomicWrite { path: PathBuf, source: std::io::Error },

    #[error("the available release {available} is not newer than the active release {active}")]
    NoNewerRelease { active: String, available: String },

    #[error("unsupported host platform: os {os:?}, arch {arch:?}")]
    UnsupportedHostPlatform { os: String, arch: String },

    #[error("failed to read {path:?} while computing its checksum: {source}")]
    ChecksumIo { path: PathBuf, source: std::io::Error },

    #[error("I/O failure at {path:?}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
}

#[cfg(test)]
#[path = "error_test.rs"]
mod tests;
