//! Thin binary wiring for direct use and manual/integration testing of `updater-cli`.
//! Behaviour is added test-first, per `AGENTS.md`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let code = updater_cli::run(args);
    match u8::try_from(code) {
        Ok(code) => ExitCode::from(code),
        Err(_) => ExitCode::FAILURE,
    }
}
