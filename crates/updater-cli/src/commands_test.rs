use std::collections::HashMap;
use std::io::Write;

use tempfile::tempdir;
use updater_core::TargetTriple;
use updater_engine::{Ed25519ManifestVerifier, FilesystemActivationPort, SystemClock, TrustedKey};

use super::*;

const SIGNING_KEY: [u8; 32] = [5_u8; 32];

struct FakeTransport(HashMap<String, Vec<u8>>);

impl FakeTransport {
    fn new() -> Self {
        FakeTransport(HashMap::new())
    }

    fn with(mut self, url: &str, bytes: Vec<u8>) -> Self {
        self.0.insert(url.to_string(), bytes);
        self
    }
}

impl HttpTransport for FakeTransport {
    fn fetch(&self, url: &str, _max_bytes: u64) -> Result<Vec<u8>, updater_engine::EngineError> {
        self.0
            .get(url)
            .cloned()
            .ok_or_else(|| updater_engine::EngineError::Transport {
                url: url.to_string(),
                reason: "no fixture registered".to_string(),
            })
    }

    fn stream(&self, url: &str, writer: &mut dyn Write) -> Result<u64, updater_engine::EngineError> {
        let bytes = self.fetch(url, u64::MAX)?;
        writer.write_all(&bytes).unwrap();
        Ok(bytes.len() as u64)
    }
}

struct FixedProbe(updater_core::ProductVersion);

impl ExecutableVersionProbe for FixedProbe {
    fn probe_version(
        &self,
        _name: &str,
        _executable: &std::path::Path,
    ) -> Result<updater_core::ProductVersion, updater_engine::EngineError> {
        Ok(self.0.clone())
    }
}

fn trusted_verifier() -> Ed25519ManifestVerifier {
    let public_key = ed25519_dalek::SigningKey::from_bytes(&SIGNING_KEY)
        .verifying_key()
        .to_bytes();
    Ed25519ManifestVerifier::new(vec![TrustedKey::new("primary", &public_key).unwrap()])
}

fn sign_envelope(message: &[u8]) -> Vec<u8> {
    use ed25519_dalek::Signer;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&SIGNING_KEY);
    let signature = signing_key.sign(message).to_bytes();
    let hex: String = signature.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(r#"{{ "key_id": "primary", "signature": "{hex}" }}"#).into_bytes()
}

fn manifest_json(version: &str, target: &str) -> Vec<u8> {
    format!(
        r#"{{
          "schema_version": 1,
          "product": "tool",
          "channel": "stable",
          "version": "{version}",
          "released_at": "2026-09-21T04:00:00Z",
          "minimum_supported_version": "1.0.0",
          "minimum_launcher_version": "1.0.0",
          "release_notes_url": "https://example.com/notes",
          "assets": [
            {{
              "target": "{target}",
              "url": "https://example.com/tool-{version}.tar.gz",
              "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
              "size": 0
            }}
          ]
        }}"#
    )
    .into_bytes()
}

#[test]
fn check_reports_no_installed_release_when_none_is_active() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin");
    let signature_bytes = sign_envelope(&manifest_bytes);
    let transport = FakeTransport::new()
        .with("https://example.com/latest.json", manifest_bytes)
        .with("https://example.com/latest.json.sig", signature_bytes);
    let verifier = trusted_verifier();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &[],
        retain_versions: 2,
    };

    let message = check(&transport, &verifier, &activation, &context).unwrap();

    assert!(message.contains("No release set is installed yet"));
}

#[test]
fn diagnostics_reports_no_state_before_any_install() {
    let temp = tempdir().unwrap();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());

    let report = diagnostics("tool", temp.path(), "aarch64-apple-darwin", &activation).unwrap();

    assert!(report.contains("product: tool"));
    assert!(report.contains("active_version: none"));
    assert!(report.contains("state: no state.json recorded yet"));
}

#[test]
fn cleanup_reports_no_obsolete_release_sets_on_a_fresh_install() {
    let temp = tempdir().unwrap();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    std::fs::create_dir_all(temp.path().join("versions")).unwrap();

    let report = cleanup(&activation, &temp.path().join("versions")).unwrap();

    assert_eq!(report, "Cleanup complete; no obsolete release sets remain.");
}

#[test]
fn rollback_reports_the_error_when_there_is_no_previous_release() {
    let temp = tempdir().unwrap();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());

    let error = rollback(&activation, &temp.path().join("versions")).unwrap_err();

    assert!(matches!(
        error,
        CliError::Engine(updater_engine::EngineError::NoPreviousRelease)
    ));
}

#[test]
fn install_pinned_rejects_a_version_the_channel_does_not_currently_offer() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin");
    let signature_bytes = sign_envelope(&manifest_bytes);
    let transport = FakeTransport::new()
        .with("https://example.com/latest.json", manifest_bytes)
        .with("https://example.com/latest.json.sig", signature_bytes);
    let verifier = trusted_verifier();
    let probe = FixedProbe(updater_core::ProductVersion::try_from("1.9.0").unwrap());
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    let clock = SystemClock;
    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &[],
        retain_versions: 2,
    };

    let error = install_pinned(&transport, &verifier, &probe, &activation, &clock, &context, "1.9.0").unwrap_err();

    assert!(matches!(error, CliError::PinnedVersionUnavailable { .. }));
}
