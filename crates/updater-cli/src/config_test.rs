use std::collections::HashMap;

use updater_launcher::EnvironmentLookup;

use super::*;

struct FakeEnv(HashMap<String, String>);

impl EnvironmentLookup for FakeEnv {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

fn valid_public_key_hex() -> String {
    "0".repeat(64)
}

fn fake_env(pairs: &[(&str, &str)]) -> FakeEnv {
    FakeEnv(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
}

fn minimal_valid_pairs(key_hex: &str) -> Vec<(&'static str, String)> {
    vec![
        ("UPDATER_PRODUCT", "tool".to_string()),
        ("TOOL_INSTALL_ROOT", "/opt/tool".to_string()),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json".to_string()),
        ("TOOL_TRUSTED_KEYS", format!("primary:{key_hex}")),
    ]
}

#[test]
fn loads_a_complete_valid_configuration() {
    let key_hex = valid_public_key_hex();
    let pairs = minimal_valid_pairs(&key_hex);
    let refs: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let env = fake_env(&refs);

    let config = load(&env).expect("valid config");

    assert_eq!(config.product, "tool");
    assert_eq!(config.manifest_url, "https://example.com/latest.json");
    assert_eq!(config.trusted_keys.len(), 1);
    assert_eq!(config.retain_versions, 2);
}

#[test]
fn rejects_a_missing_product_name() {
    let env = fake_env(&[]);

    let error = load(&env).unwrap_err();

    assert!(matches!(error, CliError::MissingConfig { variable } if variable == "UPDATER_PRODUCT"));
}

#[test]
fn rejects_a_missing_manifest_url() {
    let env = fake_env(&[("UPDATER_PRODUCT", "tool"), ("TOOL_INSTALL_ROOT", "/opt/tool")]);

    let error = load(&env).unwrap_err();

    assert!(matches!(error, CliError::MissingConfig { variable } if variable == "TOOL_UPDATE_URL"));
}

#[test]
fn rejects_a_missing_trusted_keys_configuration() {
    let env = fake_env(&[
        ("UPDATER_PRODUCT", "tool"),
        ("TOOL_INSTALL_ROOT", "/opt/tool"),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json"),
    ]);

    let error = load(&env).unwrap_err();

    assert!(matches!(error, CliError::MissingConfig { variable } if variable == "TOOL_TRUSTED_KEYS"));
}

#[test]
fn rejects_a_malformed_trusted_key_entry() {
    let env = fake_env(&[
        ("UPDATER_PRODUCT", "tool"),
        ("TOOL_INSTALL_ROOT", "/opt/tool"),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json"),
        ("TOOL_TRUSTED_KEYS", "not-a-valid-entry"),
    ]);

    let error = load(&env).unwrap_err();

    assert!(matches!(error, CliError::InvalidConfig { variable, .. } if variable == "TOOL_TRUSTED_KEYS"));
}

#[test]
fn parses_multiple_trusted_keys_for_key_rotation() {
    let key_hex = valid_public_key_hex();
    let env = fake_env(&[
        ("UPDATER_PRODUCT", "tool"),
        ("TOOL_INSTALL_ROOT", "/opt/tool"),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json"),
        ("TOOL_TRUSTED_KEYS", &format!("primary:{key_hex},secondary:{key_hex}")),
    ]);

    let config = load(&env).expect("valid config");

    assert_eq!(config.trusted_keys.len(), 2);
}

#[test]
fn parses_declared_public_commands() {
    let key_hex = valid_public_key_hex();
    let env = fake_env(&[
        ("UPDATER_PRODUCT", "tool"),
        ("TOOL_INSTALL_ROOT", "/opt/tool"),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json"),
        ("TOOL_TRUSTED_KEYS", &format!("primary:{key_hex}")),
        ("TOOL_COMMANDS", "tool, tool-agent, tool-worker"),
    ]);

    let config = load(&env).expect("valid config");

    assert_eq!(
        config.declared_public_commands,
        vec!["tool", "tool-agent", "tool-worker"]
    );
}

#[test]
fn rejects_a_non_numeric_retain_versions_value() {
    let key_hex = valid_public_key_hex();
    let env = fake_env(&[
        ("UPDATER_PRODUCT", "tool"),
        ("TOOL_INSTALL_ROOT", "/opt/tool"),
        ("TOOL_UPDATE_URL", "https://example.com/latest.json"),
        ("TOOL_TRUSTED_KEYS", &format!("primary:{key_hex}")),
        ("TOOL_RETAIN_VERSIONS", "not-a-number"),
    ]);

    let error = load(&env).unwrap_err();

    assert!(matches!(error, CliError::InvalidConfig { variable, .. } if variable == "TOOL_RETAIN_VERSIONS"));
}
