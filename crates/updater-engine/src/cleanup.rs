use std::collections::HashSet;
use std::fs;
use std::path::Path;

use updater_core::ProductVersion;

/// A version whose directory could not be removed, and why. Deletion of a superseded release set
/// is best-effort: a locked file (typically a running `.exe` on Windows) defers cleanup rather than
/// failing the whole operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredCleanup {
    pub version: ProductVersion,
    pub reason: String,
}

/// Deletes every installed version directory under `versions_dir` except those in `keep`.
///
/// Never deletes the active or previous release set; callers must include both in `keep`. A
/// directory this function cannot parse as a version is left untouched rather than guessed at.
pub fn cleanup_obsolete_versions(versions_dir: &Path, keep: &HashSet<ProductVersion>) -> Vec<DeferredCleanup> {
    let mut deferred = Vec::new();

    let entries = match fs::read_dir(versions_dir) {
        Ok(entries) => entries,
        Err(_) => return deferred,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Ok(version) = ProductVersion::try_from(name) else {
            continue;
        };
        if keep.contains(&version) {
            continue;
        }
        if let Err(source) = fs::remove_dir_all(&path) {
            deferred.push(DeferredCleanup {
                version,
                reason: source.to_string(),
            });
        }
    }

    deferred
}

#[cfg(test)]
#[path = "cleanup_test.rs"]
mod tests;
