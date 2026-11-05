//! The `update`/`rollback`/`install`/`update diagnostics` subcommands, for a product to embed in
//! its own CLI. Depends on `updater-core`, `updater-launcher`, and `updater-engine`.

mod cli_args;
mod commands;
mod config;
mod error;

use std::io::Write;

use clap::Parser;
use cli_args::{Cli, Command, UpdateAction};
use config::CliConfig;
use updater_engine::{
    Ed25519ManifestVerifier, FilesystemActivationPort, SpawnVersionProbe, SystemClock, UpdateContext, UreqTransport,
};
use updater_launcher::ProcessEnvironment;

pub use error::CliError;

/// Parses `args` (including the program name, per `clap`'s convention), executes the
/// corresponding subcommand, prints its result, and returns the process exit code.
pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            eprint!("{error}");
            return 2;
        }
    };

    match execute(cli) {
        Ok(message) => {
            println!("{message}");
            0
        }
        Err(error) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
    }
}

fn build_context(config: &CliConfig) -> UpdateContext<'_> {
    UpdateContext {
        product: &config.product,
        installation_root: config.install_root.as_path(),
        manifest_url: &config.manifest_url,
        target: &config.target,
        declared_public_commands: &config.declared_public_commands,
        retain_versions: config.retain_versions,
    }
}

fn execute(cli: Cli) -> Result<String, CliError> {
    let env = ProcessEnvironment;
    let config = config::load(&env)?;
    let activation = FilesystemActivationPort::new(config.install_root.as_path().to_path_buf());
    let versions_dir = config.install_root.as_path().join("versions");

    match cli.command {
        Command::Update { action: None, yes } => {
            if !yes && !confirm("Install this update?") {
                return Ok("Update cancelled.".to_string());
            }
            let transport = UreqTransport::default();
            let verifier = Ed25519ManifestVerifier::new(config.trusted_keys.clone());
            let probe = SpawnVersionProbe;
            let clock = SystemClock;
            let context = build_context(&config);
            commands::update(&transport, &verifier, &probe, &activation, &clock, &context, false)
        }
        Command::Update {
            action: Some(UpdateAction::Check),
            ..
        } => {
            let transport = UreqTransport::default();
            let verifier = Ed25519ManifestVerifier::new(config.trusted_keys.clone());
            let context = build_context(&config);
            commands::check(&transport, &verifier, &activation, &context)
        }
        Command::Update {
            action: Some(UpdateAction::Cleanup),
            ..
        } => commands::cleanup(&activation, &versions_dir),
        Command::Update {
            action: Some(UpdateAction::Diagnostics),
            ..
        } => commands::diagnostics(
            &config.product,
            config.install_root.as_path(),
            config.target.as_str(),
            &activation,
        ),
        Command::Rollback => commands::rollback(&activation, &versions_dir),
        Command::Install { version } => {
            let transport = UreqTransport::default();
            let verifier = Ed25519ManifestVerifier::new(config.trusted_keys.clone());
            let probe = SpawnVersionProbe;
            let clock = SystemClock;
            let context = build_context(&config);
            commands::install_pinned(&transport, &verifier, &probe, &activation, &clock, &context, &version)
        }
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{prompt} [y/N]: ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
