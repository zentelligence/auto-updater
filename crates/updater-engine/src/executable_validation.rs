use std::path::Path;
use std::process::Command;

use updater_core::{ProductVersion, ReleaseMetadata};

use crate::error::EngineError;

/// Probes a single executable's self-reported version, injectable so tests never need to spawn a
/// real process.
pub trait ExecutableVersionProbe {
    fn probe_version(&self, command_name: &str, executable: &Path) -> Result<ProductVersion, EngineError>;
}

/// The real probe: runs `<executable> --version` and takes the last whitespace-separated token of
/// its standard output as the reported version.
pub struct SpawnVersionProbe;

impl ExecutableVersionProbe for SpawnVersionProbe {
    fn probe_version(&self, command_name: &str, executable: &Path) -> Result<ProductVersion, EngineError> {
        let output = Command::new(executable).arg("--version").output().map_err(|source| {
            EngineError::ExecutableProbeFailed {
                name: command_name.to_string(),
                reason: source.to_string(),
            }
        })?;

        if !output.status.success() {
            return Err(EngineError::ExecutableProbeFailed {
                name: command_name.to_string(),
                reason: format!("exited with {}", output.status),
            });
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let token = text
            .split_whitespace()
            .next_back()
            .ok_or_else(|| EngineError::ExecutableProbeFailed {
                name: command_name.to_string(),
                reason: "produced no output to parse a version from".to_string(),
            })?;

        Ok(ProductVersion::try_from(token)?)
    }
}

/// Validates that every declared command in `metadata` reports the release's own version, per the
/// specification's requirement that a single mismatched executable rejects the whole release set.
pub fn validate_executable_versions(
    probe: &dyn ExecutableVersionProbe,
    metadata: &ReleaseMetadata,
    release_dir: &Path,
) -> Result<(), EngineError> {
    for command in &metadata.commands {
        let executable_path = command.path.resolve_within(release_dir)?;
        let reported = probe.probe_version(&command.name, &executable_path)?;
        if reported != metadata.version {
            return Err(EngineError::ExecutableVersionMismatch {
                name: command.name.clone(),
                reported: reported.to_string(),
                expected: metadata.version.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "executable_validation_test.rs"]
mod tests;
