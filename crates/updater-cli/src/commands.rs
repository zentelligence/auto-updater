use std::collections::HashSet;

use updater_core::{ActivationPort, ActiveVersionReader, ProductVersion};
use updater_engine::{
    Clock, ExecutableVersionProbe, HttpTransport, ManifestVerifier, UpdateContext, check_for_update,
    cleanup_obsolete_versions, install_update, read_state, rollback as engine_rollback,
};

use crate::error::CliError;

/// Implements `update check`: reports whether a newer, verified release set is available without
/// changing the active release.
pub fn check(
    transport: &dyn HttpTransport,
    verifier: &dyn ManifestVerifier,
    activation: &dyn ActiveVersionReader,
    context: &UpdateContext,
) -> Result<String, CliError> {
    let outcome = check_for_update(transport, verifier, activation, context)?;

    Ok(match (&outcome.active_version, outcome.update_available) {
        (None, _) => format!(
            "No release set is installed yet. Run 'install --version {}' to install it.",
            outcome.manifest.version
        ),
        (Some(active), true) => format!(
            "Update available: {}, current release set: {active}.\nRun 'update' to install release set {}.",
            outcome.manifest.version, outcome.manifest.version
        ),
        (Some(active), false) => format!("Release set {active} is already up to date."),
    })
}

/// Implements `update` / `update --yes`: performs the full install transaction.
pub fn update(
    transport: &dyn HttpTransport,
    verifier: &dyn ManifestVerifier,
    probe: &dyn ExecutableVersionProbe,
    activation: &updater_engine::FilesystemActivationPort,
    clock: &dyn Clock,
    context: &UpdateContext,
    allow_downgrade: bool,
) -> Result<String, CliError> {
    let collaborators = updater_engine::InstallCollaborators {
        transport,
        verifier,
        probe,
        activation,
        clock,
    };
    let installed = install_update(&collaborators, context, allow_downgrade)?;
    Ok(format!(
        "Installed release set {installed}.\nAll new command invocations will now use {installed}."
    ))
}

/// Implements `rollback`: switches the whole release set back to the previous one.
pub fn rollback(
    activation: &updater_engine::FilesystemActivationPort,
    versions_dir: &std::path::Path,
) -> Result<String, CliError> {
    let from = activation.active_version()?;
    let to = engine_rollback(activation, versions_dir)?;
    Ok(match from {
        Some(from) => {
            format!("Rolled back release set from {from} to {to}.\nAll new command invocations will now use {to}.")
        }
        None => format!("Rolled back to release set {to}.\nAll new command invocations will now use {to}."),
    })
}

/// Implements `update cleanup`: best-effort deletion of obsolete release sets.
pub fn cleanup(
    activation: &updater_engine::FilesystemActivationPort,
    versions_dir: &std::path::Path,
) -> Result<String, CliError> {
    let mut keep = HashSet::new();
    if let Some(active) = activation.active_version()? {
        keep.insert(active);
    }
    if let Some(previous) = activation.previous_version()? {
        keep.insert(previous);
    }

    let deferred = cleanup_obsolete_versions(versions_dir, &keep);
    if deferred.is_empty() {
        return Ok("Cleanup complete; no obsolete release sets remain.".to_string());
    }

    let mut lines =
        vec!["Cleanup completed; some release sets are still in use and will be retried later:".to_string()];
    for item in deferred {
        lines.push(format!("  {} ({})", item.version, item.reason));
    }
    Ok(lines.join("\n"))
}

/// Implements `update diagnostics`: a structured, human-readable summary of the installation.
pub fn diagnostics(
    product: &str,
    installation_root: &std::path::Path,
    target: &str,
    activation: &updater_engine::FilesystemActivationPort,
) -> Result<String, CliError> {
    let active = activation.active_version()?;
    let previous = activation.previous_version()?;
    let state = read_state(installation_root)?;

    let mut lines = vec![
        format!("product: {product}"),
        format!("installation_root: {}", installation_root.display()),
        format!("detected_target: {target}"),
        format!("active_version: {}", display_optional(active.as_ref())),
        format!("previous_version: {}", display_optional(previous.as_ref())),
    ];

    match state {
        Some(state) => {
            lines.push(format!("channel: {}", state.channel));
            let installed = state
                .installed_versions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("installed_versions: {installed}"));
            lines.push(format!(
                "last_check_at: {}",
                state.last_check_at.as_deref().unwrap_or("never")
            ));
            lines.push(format!(
                "last_successful_update_at: {}",
                state.last_successful_update_at.as_deref().unwrap_or("never")
            ));
            lines.push(format!(
                "last_update_error: {}",
                state.last_update_error.as_deref().unwrap_or("none")
            ));
        }
        None => lines.push("state: no state.json recorded yet".to_string()),
    }

    Ok(lines.join("\n"))
}

fn display_optional(version: Option<&ProductVersion>) -> String {
    match version {
        Some(version) => version.to_string(),
        None => "none".to_string(),
    }
}

/// Implements `install --version <version>`: installs a specific, pinned release for CI and
/// automation. This crate's manifest model only exposes the current channel head, so a pinned
/// install is only satisfiable when it matches the manifest's current version; installing an
/// arbitrary historical version needs per-version manifest infrastructure the specification leaves
/// unspecified for this workspace.
pub fn install_pinned(
    transport: &dyn HttpTransport,
    verifier: &dyn ManifestVerifier,
    probe: &dyn ExecutableVersionProbe,
    activation: &updater_engine::FilesystemActivationPort,
    clock: &dyn Clock,
    context: &UpdateContext,
    requested_version: &str,
) -> Result<String, CliError> {
    let requested = ProductVersion::try_from(requested_version)?;

    let outcome = check_for_update(transport, verifier, activation, context)?;
    if outcome.manifest.version != requested {
        return Err(CliError::PinnedVersionUnavailable {
            requested: requested.to_string(),
            available: outcome.manifest.version.to_string(),
        });
    }

    let collaborators = updater_engine::InstallCollaborators {
        transport,
        verifier,
        probe,
        activation,
        clock,
    };
    let installed = install_update(&collaborators, context, true)?;
    Ok(format!("Installed pinned release set {installed}."))
}

#[cfg(test)]
#[path = "commands_test.rs"]
mod tests;
