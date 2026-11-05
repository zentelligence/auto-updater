use std::path::PathBuf;

use updater_core::InstallationRoot;

use crate::error::LauncherError;

/// A source of environment-variable values, injected so installation-root resolution is
/// deterministic and testable without mutating the real process environment.
pub trait EnvironmentLookup {
    fn get(&self, key: &str) -> Option<String>;
}

/// The real process environment.
pub struct ProcessEnvironment;

impl EnvironmentLookup for ProcessEnvironment {
    fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// Resolves the product's installation root.
///
/// Checks `<PREFIX>_INSTALL_ROOT` (uppercased) first; falls back to `~/.local/share/<dir_name>`
/// on Unix-like systems or `%LOCALAPPDATA%\<dir_name>` on Windows. `dir_name` is supplied by the
/// embedding product rather than derived here, since Windows conventionally capitalises the
/// directory name while the environment-variable prefix is always upper-cased.
pub fn resolve_installation_root(
    env: &impl EnvironmentLookup,
    env_prefix: &str,
    dir_name: &str,
) -> Result<InstallationRoot, LauncherError> {
    let override_var = format!("{}_INSTALL_ROOT", env_prefix.to_uppercase());
    if let Some(value) = env.get(&override_var) {
        return Ok(InstallationRoot::try_from(PathBuf::from(value))?);
    }

    #[cfg(unix)]
    {
        let home = env.get("HOME").ok_or(LauncherError::HomeDirectoryUnavailable)?;
        let root = PathBuf::from(home).join(".local").join("share").join(dir_name);
        Ok(InstallationRoot::try_from(root)?)
    }

    #[cfg(windows)]
    {
        let local_app_data = env.get("LOCALAPPDATA").ok_or(LauncherError::LocalAppDataUnavailable)?;
        let root = PathBuf::from(local_app_data).join(dir_name);
        Ok(InstallationRoot::try_from(root)?)
    }
}

#[cfg(test)]
#[path = "install_root_test.rs"]
mod tests;
