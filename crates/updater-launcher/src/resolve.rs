use updater_core::{InstallationRoot, ProductVersion, ReleaseMetadata};

use crate::active_version::read_active_version;
use crate::error::LauncherError;

/// A validated, ready-to-execute command from the active release set.
#[derive(Debug)]
pub struct ResolvedCommand {
    pub version: ProductVersion,
    pub executable_path: std::path::PathBuf,
}

/// Resolves `command_name` against the active release set under `root`.
///
/// Performs every launcher-side check from the specification's launcher behaviour: locating the
/// active version, resolving the declared executable path, confirming the command is declared in
/// `release.json`, and confirming the executable exists.
pub fn resolve_command(root: &InstallationRoot, command_name: &str) -> Result<ResolvedCommand, LauncherError> {
    let version = read_active_version(root)?;
    let version_dir = root.as_path().join("versions").join(version.to_string());
    let metadata_path = version_dir.join("release.json");

    let bytes = std::fs::read(&metadata_path).map_err(|source| LauncherError::ReadReleaseMetadata {
        path: metadata_path.clone(),
        source,
    })?;
    let metadata =
        ReleaseMetadata::try_from(bytes.as_slice()).map_err(|source| LauncherError::InvalidReleaseMetadata {
            path: metadata_path.clone(),
            source,
        })?;

    let command = metadata
        .command(command_name)
        .ok_or_else(|| LauncherError::UndeclaredCommand {
            name: command_name.to_string(),
            version: version.to_string(),
        })?;

    let executable_path = command.path.resolve_within(&version_dir)?;
    if !executable_path.exists() {
        return Err(LauncherError::ExecutableMissing {
            name: command_name.to_string(),
            path: executable_path,
        });
    }

    Ok(ResolvedCommand {
        version,
        executable_path,
    })
}

#[cfg(test)]
#[path = "resolve_test.rs"]
mod tests;
