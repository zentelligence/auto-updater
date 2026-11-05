use std::path::PathBuf;

use updater_core::CoreError;

/// The single error enum for `updater-launcher`.
#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("could not determine the home directory to resolve the installation root")]
    HomeDirectoryUnavailable,

    #[error("could not determine %LOCALAPPDATA% to resolve the installation root")]
    LocalAppDataUnavailable,

    #[error("failed to read active pointer at {path:?}: {source}")]
    ReadActivePointer { path: PathBuf, source: std::io::Error },

    #[error("no release has been activated under installation root {root:?}")]
    NoActiveRelease { root: PathBuf },

    #[error("failed to read release metadata at {path:?}: {source}")]
    ReadReleaseMetadata { path: PathBuf, source: std::io::Error },

    #[error("release metadata at {path:?} is invalid: {source}")]
    InvalidReleaseMetadata {
        path: PathBuf,
        #[source]
        source: CoreError,
    },

    #[error("command {name:?} is not declared by the active release set (version {version})")]
    UndeclaredCommand { name: String, version: String },

    #[error("executable for command {name:?} does not exist at {path:?}")]
    ExecutableMissing { name: String, path: PathBuf },

    #[error("failed to start command {name:?}: {source}")]
    Spawn { name: String, source: std::io::Error },
}

#[cfg(test)]
#[path = "error_test.rs"]
mod tests;
