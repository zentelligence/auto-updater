//! Offline signing tool for the update manifest: never linked into the client (`updater-engine`
//! only verifies). Used by repository maintainers and release CI to generate a signing keypair
//! and produce the detached-signature envelope alongside a release's `latest.json`.

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};

#[derive(Parser)]
#[command(
    name = "updater-keytool",
    about = "Generate and use Ed25519 keys for signing update manifests."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generates a new Ed25519 keypair and prints both keys as hex.
    Generate,
    /// Signs a manifest file, producing the `{"key_id", "signature"}` envelope this workspace's
    /// `SignatureEnvelope` reads.
    Sign {
        /// Path to a file containing the 64-character hex-encoded private key.
        #[arg(long)]
        private_key_file: PathBuf,
        /// The key id to embed in the envelope, matching a trusted key configured on the client.
        #[arg(long)]
        key_id: String,
        /// The manifest file to sign.
        manifest: PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(output) => {
            println!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<String, String> {
    match cli.command {
        Command::Generate => generate(),
        Command::Sign {
            private_key_file,
            key_id,
            manifest,
        } => sign(&private_key_file, &key_id, &manifest),
    }
}

fn generate() -> Result<String, String> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|error| format!("failed to read system randomness: {error}"))?;

    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    Ok(format!(
        "private_key (keep secret, store only in a CI secret): {}\npublic_key (embed as a trusted key): {}",
        encode_hex(&seed),
        encode_hex(verifying_key.as_bytes())
    ))
}

fn sign(private_key_file: &std::path::Path, key_id: &str, manifest: &std::path::Path) -> Result<String, String> {
    let private_key_hex = fs::read_to_string(private_key_file)
        .map_err(|error| format!("failed to read {private_key_file:?}: {error}"))?;
    let seed = decode_hex_32(private_key_hex.trim())
        .ok_or_else(|| format!("{private_key_file:?} does not contain a valid 64-character hex private key"))?;
    let signing_key = SigningKey::from_bytes(&seed);

    let message = fs::read(manifest).map_err(|error| format!("failed to read {manifest:?}: {error}"))?;
    let signature = signing_key.sign(&message);

    Ok(format!(
        "{{\"key_id\": {key_id:?}, \"signature\": \"{}\"}}",
        encode_hex(&signature.to_bytes())
    ))
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}
