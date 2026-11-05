/// This crate's CLI surface: `update check`, `update`, `update --yes`, `rollback`,
/// `update cleanup`, `update diagnostics`, and `install --version`, per the specification's
/// "Update Commands" section.
#[derive(clap::Parser)]
#[command(name = "updater", about = "Manage the product's versioned release sets.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Suite-level update operations: check, install, clean up, or diagnose.
    Update {
        #[command(subcommand)]
        action: Option<UpdateAction>,

        /// Install without an interactive confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Roll back to the previously active release set.
    Rollback,
    /// Install a specific, pinned release set, for CI and automation.
    Install {
        #[arg(long)]
        version: String,
    },
}

#[derive(clap::Subcommand)]
pub enum UpdateAction {
    /// Report whether a newer release set is available, without installing it.
    Check,
    /// Delete obsolete release sets, best-effort.
    Cleanup,
    /// Print a structured summary of the installation.
    Diagnostics,
}
