use std::path::Path;

#[cfg(windows)]
use updater_core::version_from_pointer_text;
#[cfg(unix)]
use updater_core::version_from_symlink_target;
use updater_core::{InstallationRoot, ProductVersion};

use crate::error::LauncherError;

/// Reads the active version from the installation root's `current` symlink (Unix-like) or
/// `current.txt` file (Windows).
pub fn read_active_version(root: &InstallationRoot) -> Result<ProductVersion, LauncherError> {
    #[cfg(unix)]
    {
        let pointer = root.as_path().join("current");
        let target = std::fs::read_link(&pointer).map_err(|source| map_missing_pointer(root.as_path(), source))?;
        Ok(version_from_symlink_target(&target)?)
    }

    #[cfg(windows)]
    {
        let pointer = root.as_path().join("current.txt");
        let text = std::fs::read_to_string(&pointer).map_err(|source| map_missing_pointer(root.as_path(), source))?;
        Ok(version_from_pointer_text(&text)?)
    }
}

fn map_missing_pointer(root: &Path, source: std::io::Error) -> LauncherError {
    if source.kind() == std::io::ErrorKind::NotFound {
        LauncherError::NoActiveRelease {
            root: root.to_path_buf(),
        }
    } else {
        LauncherError::ReadActivePointer {
            path: root.to_path_buf(),
            source,
        }
    }
}

#[cfg(test)]
#[path = "active_version_test.rs"]
mod tests;
