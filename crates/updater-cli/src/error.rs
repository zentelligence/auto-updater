/// The single error enum for `updater-cli`.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("missing required environment variable {variable}")]
    MissingConfig { variable: String },

    #[error("environment variable {variable}={value:?} is invalid")]
    InvalidConfig { variable: String, value: String },

    #[error("pinned version {requested} is not available; the channel currently offers {available}")]
    PinnedVersionUnavailable { requested: String, available: String },

    #[error(transparent)]
    Core(#[from] updater_core::CoreError),

    #[error(transparent)]
    Launcher(#[from] updater_launcher::LauncherError),

    #[error(transparent)]
    Engine(#[from] updater_engine::EngineError),
}

impl CliError {
    /// The process exit code this error should produce, per the specification's requirement that
    /// scripted callers get a stable, documented result rather than parsed prose.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::MissingConfig { .. } | CliError::InvalidConfig { .. } => 2,
            CliError::PinnedVersionUnavailable { .. } => 3,
            CliError::Core(_) | CliError::Launcher(_) | CliError::Engine(_) => 1,
        }
    }
}

#[cfg(test)]
#[path = "error_test.rs"]
mod tests;
