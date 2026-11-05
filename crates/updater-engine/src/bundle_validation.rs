use std::path::Path;

use updater_core::{ProductVersion, ReleaseMetadata, TargetTriple};

use crate::error::EngineError;

/// Performs every contextual bundle-validation check from the specification's "Bundle Validation"
/// list that this crate can check without spawning a process: product, version, and target match;
/// every declared executable exists; every declared executable has the required Unix executable
/// permission; and every expected public command name has a corresponding declared command.
///
/// Executable version reporting is validated separately by [`crate::executable_validation`], since
/// it requires spawning each declared executable.
pub fn validate_bundle(
    metadata: &ReleaseMetadata,
    release_dir: &Path,
    expected_product: &str,
    expected_manifest_version: &ProductVersion,
    expected_target: &TargetTriple,
    expected_public_commands: &[String],
) -> Result<(), EngineError> {
    if metadata.product != expected_product {
        return Err(EngineError::ProductMismatch {
            expected: expected_product.to_string(),
            found: metadata.product.clone(),
        });
    }
    if &metadata.version != expected_manifest_version {
        return Err(EngineError::VersionMismatch {
            expected: expected_manifest_version.to_string(),
            found: metadata.version.to_string(),
        });
    }
    if &metadata.target != expected_target {
        return Err(EngineError::TargetMismatch {
            expected: expected_target.to_string(),
            found: metadata.target.to_string(),
        });
    }

    for command in &metadata.commands {
        let executable_path = command.path.resolve_within(release_dir)?;
        if !executable_path.is_file() {
            return Err(EngineError::MissingCommandExecutable {
                name: command.name.clone(),
                path: executable_path,
            });
        }
        require_executable_permission(&command.name, &executable_path)?;
    }

    for expected_name in expected_public_commands {
        if metadata.command(expected_name).is_none() {
            return Err(EngineError::LauncherWithoutCommand {
                name: expected_name.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(unix)]
fn require_executable_permission(name: &str, path: &Path) -> Result<(), EngineError> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::metadata(path)
        .map_err(|_source| EngineError::MissingCommandExecutable {
            name: name.to_string(),
            path: path.to_path_buf(),
        })
        .map(|metadata| metadata.permissions())?;

    if permissions.mode() & 0o111 == 0 {
        return Err(EngineError::NotExecutable {
            name: name.to_string(),
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(windows)]
fn require_executable_permission(_name: &str, _path: &Path) -> Result<(), EngineError> {
    Ok(())
}

#[cfg(test)]
#[path = "bundle_validation_test.rs"]
mod tests;
