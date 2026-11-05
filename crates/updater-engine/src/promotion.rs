use std::fs;
use std::path::{Path, PathBuf};

use updater_core::ProductVersion;

use crate::error::EngineError;

/// Promotes a fully validated staging directory to its immutable versioned directory with a
/// same-filesystem rename, per the write pipeline: this is the only step that turns a validated,
/// staged release set into one the activation step may point at.
pub fn promote_release(
    staging_dir: &Path,
    versions_dir: &Path,
    version: &ProductVersion,
) -> Result<PathBuf, EngineError> {
    fs::create_dir_all(versions_dir).map_err(|source| EngineError::Promotion {
        target: versions_dir.to_path_buf(),
        source,
    })?;

    let target = versions_dir.join(version.to_string());
    if target.exists() {
        return Err(EngineError::VersionAlreadyInstalled {
            version: version.to_string(),
            path: target,
        });
    }

    fs::rename(staging_dir, &target).map_err(|source| EngineError::Promotion {
        target: target.clone(),
        source,
    })?;
    Ok(target)
}

#[cfg(test)]
#[path = "promotion_test.rs"]
mod tests;
