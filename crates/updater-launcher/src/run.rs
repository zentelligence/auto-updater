use std::ffi::OsString;
use std::process::{Command, ExitStatus};

use crate::error::LauncherError;
use crate::resolve::ResolvedCommand;

/// Runs the resolved command with the given arguments.
///
/// Inherits the current process's working directory, environment, and standard streams, and
/// returns the child's exit status so the caller can preserve exit-code semantics.
pub fn run_resolved_command(
    resolved: &ResolvedCommand,
    command_name: &str,
    args: &[OsString],
) -> Result<ExitStatus, LauncherError> {
    Command::new(&resolved.executable_path)
        .args(args)
        .status()
        .map_err(|source| LauncherError::Spawn {
            name: command_name.to_string(),
            source,
        })
}

#[cfg(test)]
#[path = "run_test.rs"]
mod tests;
