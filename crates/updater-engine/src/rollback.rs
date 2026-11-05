use std::path::Path;

use updater_core::{ActivationPort, ProductVersion};

use crate::error::EngineError;

/// Rolls back to the previous complete release set.
///
/// Verifies the previous release set's directory still exists before activating it, then switches
/// the whole release set at once: rollback must never switch only one executable.
pub fn rollback(port: &dyn ActivationPort, versions_dir: &Path) -> Result<ProductVersion, EngineError> {
    let previous = port.previous_version()?.ok_or(EngineError::NoPreviousRelease)?;

    let previous_dir = versions_dir.join(previous.to_string());
    if !previous_dir.join("release.json").is_file() {
        return Err(EngineError::NoPreviousRelease);
    }

    port.activate(&previous)?;
    Ok(previous)
}

#[cfg(test)]
#[path = "rollback_test.rs"]
mod tests;
