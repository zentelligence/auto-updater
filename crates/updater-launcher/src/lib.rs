//! Resolves the active release set for a requested command name and starts its executable.
//! Never activates a release or deletes one.

mod active_version;
mod error;
mod install_root;
mod resolve;
mod run;

use std::ffi::OsString;
use std::process::ExitStatus;

pub use error::LauncherError;
pub use install_root::{EnvironmentLookup, ProcessEnvironment, resolve_installation_root};
pub use resolve::{ResolvedCommand, resolve_command};
pub use run::run_resolved_command;
use updater_core::InstallationRoot;

/// Resolves and starts `command_name` from the active release set under `root`, with the given
/// arguments. This is the single entry point a per-command stable launcher binary should call.
pub fn launch(root: &InstallationRoot, command_name: &str, args: &[OsString]) -> Result<ExitStatus, LauncherError> {
    let resolved = resolve_command(root, command_name)?;
    run_resolved_command(&resolved, command_name, args)
}
