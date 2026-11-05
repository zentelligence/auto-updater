use flate2::Compression;
use flate2::write::GzEncoder;
use tempfile::tempdir;
use updater_core::ActivationPort;

use super::*;
use crate::activation::FilesystemActivationPort;
use crate::executable_validation::ExecutableVersionProbe;
use crate::signature::{Ed25519ManifestVerifier, TrustedKey, sign_for_test};
use crate::test_support::{FakeTransport, FixedClock};

const SIGNING_KEY: [u8; 32] = [3_u8; 32];

fn trusted_verifier() -> Ed25519ManifestVerifier {
    let public_key = ed25519_dalek::SigningKey::from_bytes(&SIGNING_KEY)
        .verifying_key()
        .to_bytes();
    Ed25519ManifestVerifier::new(vec![TrustedKey::new("primary", &public_key).unwrap()])
}

fn sign_envelope(message: &[u8]) -> Vec<u8> {
    let signature = sign_for_test(&SIGNING_KEY, message);
    let hex: String = signature.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(r#"{{ "key_id": "primary", "signature": "{hex}" }}"#).into_bytes()
}

fn build_release_archive(version: &str, target: &str) -> Vec<u8> {
    let release_json = format!(
        r#"{{
          "schema_version": 1,
          "product": "tool",
          "version": "{version}",
          "target": "{target}",
          "published_at": "2026-09-21T04:00:00Z",
          "minimum_launcher_version": "1.0.0",
          "commands": [ {{ "name": "tool", "path": "bin/tool" }} ]
        }}"#
    );

    let mut buffer = Vec::new();
    {
        let encoder = GzEncoder::new(&mut buffer, Compression::default());
        let mut builder = tar::Builder::new(encoder);

        let mut header = tar::Header::new_gnu();
        header.set_size(release_json.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "release.json", release_json.as_bytes())
            .unwrap();

        let mut header = tar::Header::new_gnu();
        header.set_size(4);
        header.set_mode(0o755);
        header.set_cksum();
        builder.append_data(&mut header, "bin/tool", &b"stub"[..]).unwrap();

        builder.into_inner().unwrap().finish().unwrap();
    }
    buffer
}

fn manifest_json(version: &str, target: &str, sha256: &str, size: usize) -> String {
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
              "url": "https://example.com/tool-{version}-{target}.tar.gz",
              "sha256": "{sha256}",
              "size": {size}
            }}
          ]
        }}"#
    )
}

struct FixedVersionProbe(ProductVersion);

impl ExecutableVersionProbe for FixedVersionProbe {
    fn probe_version(&self, _command_name: &str, _executable: &std::path::Path) -> Result<ProductVersion, EngineError> {
        Ok(self.0.clone())
    }
}

fn digest_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn installs_a_fresh_release_set_end_to_end() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let archive_bytes = build_release_archive("1.10.0", "aarch64-apple-darwin");
    let sha256 = digest_hex(&archive_bytes);
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin", &sha256, archive_bytes.len()).into_bytes();
    let signature_bytes = sign_envelope(&manifest_bytes);

    let transport = FakeTransport::default()
        .with_response("https://example.com/latest.json", manifest_bytes)
        .with_response("https://example.com/latest.json.sig", signature_bytes)
        .with_response(
            "https://example.com/tool-1.10.0-aarch64-apple-darwin.tar.gz",
            archive_bytes,
        );
    let verifier = trusted_verifier();
    let probe = FixedVersionProbe(ProductVersion::try_from("1.10.0").unwrap());
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    let clock = FixedClock("2026-09-21T04:16:43Z");

    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &["tool".to_string()],
        retain_versions: 2,
    };
    let collaborators = InstallCollaborators {
        transport: &transport,
        verifier: &verifier,
        probe: &probe,
        activation: &activation,
        clock: &clock,
    };

    let installed = install_update(&collaborators, &context, false).unwrap();

    assert_eq!(installed.to_string(), "1.10.0");
    assert_eq!(activation.active_version().unwrap().unwrap().to_string(), "1.10.0");
    assert!(temp.path().join("versions/1.10.0/bin/tool").is_file());
    let state = read_state(temp.path()).unwrap().unwrap();
    assert_eq!(state.active_version.to_string(), "1.10.0");
    assert!(state.previous_version.is_none());
}

#[test]
fn check_for_update_reports_availability_without_activating() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let archive_bytes = build_release_archive("1.10.0", "aarch64-apple-darwin");
    let sha256 = digest_hex(&archive_bytes);
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin", &sha256, archive_bytes.len()).into_bytes();
    let signature_bytes = sign_envelope(&manifest_bytes);

    let transport = FakeTransport::default()
        .with_response("https://example.com/latest.json", manifest_bytes)
        .with_response("https://example.com/latest.json.sig", signature_bytes);
    let verifier = trusted_verifier();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());

    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &["tool".to_string()],
        retain_versions: 2,
    };

    let outcome = check_for_update(&transport, &verifier, &activation, &context).unwrap();

    assert!(outcome.update_available);
    assert!(outcome.active_version.is_none());
    assert!(activation.active_version().unwrap().is_none());
}

#[test]
fn rejects_a_manifest_with_an_invalid_signature() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let archive_bytes = build_release_archive("1.10.0", "aarch64-apple-darwin");
    let sha256 = digest_hex(&archive_bytes);
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin", &sha256, archive_bytes.len()).into_bytes();
    let tampered_signature = sign_envelope(b"different bytes entirely");

    let transport = FakeTransport::default()
        .with_response("https://example.com/latest.json", manifest_bytes)
        .with_response("https://example.com/latest.json.sig", tampered_signature);
    let verifier = trusted_verifier();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());

    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &["tool".to_string()],
        retain_versions: 2,
    };

    let error = check_for_update(&transport, &verifier, &activation, &context).unwrap_err();

    assert!(matches!(error, EngineError::ManifestSignatureInvalid));
    assert!(activation.active_version().unwrap().is_none());
}

#[test]
fn rejects_a_downloaded_archive_with_a_checksum_mismatch() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();
    let archive_bytes = build_release_archive("1.10.0", "aarch64-apple-darwin");
    let wrong_sha256 = digest_hex(b"not the archive");
    let manifest_bytes =
        manifest_json("1.10.0", "aarch64-apple-darwin", &wrong_sha256, archive_bytes.len()).into_bytes();
    let signature_bytes = sign_envelope(&manifest_bytes);

    let transport = FakeTransport::default()
        .with_response("https://example.com/latest.json", manifest_bytes)
        .with_response("https://example.com/latest.json.sig", signature_bytes)
        .with_response(
            "https://example.com/tool-1.10.0-aarch64-apple-darwin.tar.gz",
            archive_bytes,
        );
    let verifier = trusted_verifier();
    let probe = FixedVersionProbe(ProductVersion::try_from("1.10.0").unwrap());
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    let clock = FixedClock("2026-09-21T04:16:43Z");

    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &["tool".to_string()],
        retain_versions: 2,
    };
    let collaborators = InstallCollaborators {
        transport: &transport,
        verifier: &verifier,
        probe: &probe,
        activation: &activation,
        clock: &clock,
    };

    let error = install_update(&collaborators, &context, false).unwrap_err();

    assert!(matches!(error, EngineError::ChecksumMismatch { .. }));
    assert!(activation.active_version().unwrap().is_none());
    assert_eq!(std::fs::read_dir(temp.path().join("downloads")).unwrap().count(), 0);
}

#[test]
fn upgrading_moves_the_formerly_active_release_to_previous() {
    let temp = tempdir().unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    // Pre-install 1.9.0 directly, as a prior successful install would have left it.
    let old_dir = temp.path().join("versions/1.9.0");
    std::fs::create_dir_all(old_dir.join("bin")).unwrap();
    std::fs::write(old_dir.join("bin/tool"), b"stub").unwrap();
    std::fs::write(
        old_dir.join("release.json"),
        r#"{
          "schema_version": 1,
          "product": "tool",
          "version": "1.9.0",
          "target": "aarch64-apple-darwin",
          "published_at": "2026-09-21T04:00:00Z",
          "minimum_launcher_version": "1.0.0",
          "commands": [ { "name": "tool", "path": "bin/tool" } ]
        }"#,
    )
    .unwrap();
    let activation = FilesystemActivationPort::new(temp.path().to_path_buf());
    activation
        .activate(&ProductVersion::try_from("1.9.0").unwrap())
        .unwrap();

    let archive_bytes = build_release_archive("1.10.0", "aarch64-apple-darwin");
    let sha256 = digest_hex(&archive_bytes);
    let manifest_bytes = manifest_json("1.10.0", "aarch64-apple-darwin", &sha256, archive_bytes.len()).into_bytes();
    let signature_bytes = sign_envelope(&manifest_bytes);
    let transport = FakeTransport::default()
        .with_response("https://example.com/latest.json", manifest_bytes)
        .with_response("https://example.com/latest.json.sig", signature_bytes)
        .with_response(
            "https://example.com/tool-1.10.0-aarch64-apple-darwin.tar.gz",
            archive_bytes,
        );
    let verifier = trusted_verifier();
    let probe = FixedVersionProbe(ProductVersion::try_from("1.10.0").unwrap());
    let clock = FixedClock("2026-09-21T04:16:43Z");

    let context = UpdateContext {
        product: "tool",
        installation_root: temp.path(),
        manifest_url: "https://example.com/latest.json",
        target: &target,
        declared_public_commands: &["tool".to_string()],
        retain_versions: 2,
    };
    let collaborators = InstallCollaborators {
        transport: &transport,
        verifier: &verifier,
        probe: &probe,
        activation: &activation,
        clock: &clock,
    };

    install_update(&collaborators, &context, false).unwrap();

    assert_eq!(activation.active_version().unwrap().unwrap().to_string(), "1.10.0");
    assert_eq!(activation.previous_version().unwrap().unwrap().to_string(), "1.9.0");
    let state = read_state(temp.path()).unwrap().unwrap();
    assert_eq!(state.previous_version.unwrap().to_string(), "1.9.0");
}
