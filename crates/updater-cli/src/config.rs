use updater_core::{InstallationRoot, TargetTriple};
use updater_engine::TrustedKey;
use updater_launcher::{EnvironmentLookup, resolve_installation_root};

use crate::error::CliError;

/// Runtime configuration for this CLI, loaded from environment variables named after the product
/// (`UPDATER_PRODUCT`, then `<PREFIX>_UPDATE_URL`, `<PREFIX>_TRUSTED_KEYS`, and so on), per the
/// specification's "Configuration" section. Never hardcodes a product name: the embedding product
/// supplies every one of these values.
#[derive(Debug)]
pub struct CliConfig {
    pub product: String,
    pub install_root: InstallationRoot,
    pub manifest_url: String,
    pub target: TargetTriple,
    pub declared_public_commands: Vec<String>,
    pub retain_versions: usize,
    pub trusted_keys: Vec<TrustedKey>,
}

const DEFAULT_RETAIN_VERSIONS: usize = 2;

pub fn load(env: &impl EnvironmentLookup) -> Result<CliConfig, CliError> {
    let product = require(env, "UPDATER_PRODUCT")?;
    let prefix = product.to_uppercase();

    let install_root = resolve_installation_root(env, &prefix, &product)?;
    let manifest_url = require(env, &format!("{prefix}_UPDATE_URL"))?;
    let target = updater_engine::detect_target()?;

    let declared_public_commands = env
        .get(&format!("{prefix}_COMMANDS"))
        .map(|value| value.split(',').map(str::trim).map(str::to_string).collect())
        .unwrap_or_default();

    let retain_versions = env
        .get(&format!("{prefix}_RETAIN_VERSIONS"))
        .map(|value| {
            value.trim().parse::<usize>().map_err(|_| CliError::InvalidConfig {
                variable: format!("{prefix}_RETAIN_VERSIONS"),
                value: value.clone(),
            })
        })
        .transpose()?
        .unwrap_or(DEFAULT_RETAIN_VERSIONS);

    let trusted_keys_raw = require(env, &format!("{prefix}_TRUSTED_KEYS"))?;
    let trusted_keys = parse_trusted_keys(&prefix, &trusted_keys_raw)?;

    Ok(CliConfig {
        product,
        install_root,
        manifest_url,
        target,
        declared_public_commands,
        retain_versions,
        trusted_keys,
    })
}

fn require(env: &impl EnvironmentLookup, variable: &str) -> Result<String, CliError> {
    env.get(variable)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| CliError::MissingConfig {
            variable: variable.to_string(),
        })
}

/// Parses `<key_id>:<64-hex-character-public-key>` pairs, separated by commas, into trusted keys.
fn parse_trusted_keys(prefix: &str, raw: &str) -> Result<Vec<TrustedKey>, CliError> {
    let variable = format!("{prefix}_TRUSTED_KEYS");
    raw.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (key_id, hex) = entry.split_once(':').ok_or_else(|| CliError::InvalidConfig {
                variable: variable.clone(),
                value: entry.to_string(),
            })?;
            let bytes = decode_hex_32(hex).ok_or_else(|| CliError::InvalidConfig {
                variable: variable.clone(),
                value: entry.to_string(),
            })?;
            Ok(TrustedKey::new(key_id, &bytes)?)
        })
        .collect()
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

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
