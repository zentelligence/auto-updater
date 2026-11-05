use std::path::Path;

use crate::error::CoreError;
use crate::version::ProductVersion;

/// Parses the version encoded by a Unix-like `current`/`previous` symlink target, such as
/// `versions/1.10.0`.
///
/// This crate never reads the symlink itself; callers (`updater-launcher`, `updater-engine`)
/// perform the filesystem read and pass the resulting target path here to interpret it.
pub fn version_from_symlink_target(target: &Path) -> Result<ProductVersion, CoreError> {
    let raw = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| CoreError::InvalidActivePointer {
            value: target.to_string_lossy().into_owned(),
        })?;
    ProductVersion::try_from(raw.as_str())
}

/// Parses the version encoded by a Windows `current.txt`/`previous.txt` file's contents.
pub fn version_from_pointer_text(text: &str) -> Result<ProductVersion, CoreError> {
    ProductVersion::try_from(text.trim())
}

#[cfg(test)]
#[path = "pointer_test.rs"]
mod tests;
