use super::*;

fn valid_manifest_json() -> &'static str {
    r#"{
      "schema_version": 1,
      "product": "tool",
      "channel": "stable",
      "version": "1.10.0",
      "released_at": "2026-09-21T04:00:00Z",
      "minimum_supported_version": "1.6.0",
      "minimum_launcher_version": "1.0.0",
      "release_notes_url": "https://github.com/owner/repo/releases/tag/v1.10.0",
      "assets": [
        {
          "target": "aarch64-apple-darwin",
          "url": "https://github.com/owner/repo/releases/download/v1.10.0/tool-1.10.0-aarch64-apple-darwin.tar.gz",
          "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "size": 12345678
        }
      ]
    }"#
}

#[test]
fn parses_a_representative_complete_document() {
    let manifest = UpdateManifest::try_from(valid_manifest_json().as_bytes()).expect("valid manifest");

    assert_eq!(manifest.version.to_string(), "1.10.0");
    assert_eq!(manifest.assets.len(), 1);
}

#[test]
fn finds_the_asset_for_the_detected_target() {
    let manifest = UpdateManifest::try_from(valid_manifest_json().as_bytes()).unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    assert!(manifest.asset_for(&target).is_some());
}

#[test]
fn returns_none_for_an_undeclared_target() {
    let manifest = UpdateManifest::try_from(valid_manifest_json().as_bytes()).unwrap();
    let target = TargetTriple::try_from("x86_64-pc-windows-msvc").unwrap();

    assert!(manifest.asset_for(&target).is_none());
}

#[test]
fn rejects_unknown_fields() {
    let json = valid_manifest_json().replace("\"channel\"", "\"unexpected_field\": true, \"channel\"");

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::ManifestDecode { .. }));
}

#[test]
fn rejects_an_unsupported_schema_version() {
    let json = valid_manifest_json().replace("\"schema_version\": 1", "\"schema_version\": 2");

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(
        error,
        CoreError::UnsupportedSchemaVersion {
            found: 2,
            expected: 1,
            ..
        }
    ));
}

#[test]
fn rejects_an_empty_asset_list() {
    let json = valid_manifest_json().replacen(
        r#"[
        {
          "target": "aarch64-apple-darwin",
          "url": "https://github.com/owner/repo/releases/download/v1.10.0/tool-1.10.0-aarch64-apple-darwin.tar.gz",
          "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "size": 12345678
        }
      ]"#,
        "[]",
        1,
    );

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::NoAssetsDeclared));
}

#[test]
fn rejects_a_duplicate_target() {
    let json = valid_manifest_json().replace(
        r#"{
          "target": "aarch64-apple-darwin",
          "url": "https://github.com/owner/repo/releases/download/v1.10.0/tool-1.10.0-aarch64-apple-darwin.tar.gz",
          "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "size": 12345678
        }"#,
        r#"{
          "target": "aarch64-apple-darwin",
          "url": "https://github.com/owner/repo/releases/download/v1.10.0/tool-1.10.0-aarch64-apple-darwin.tar.gz",
          "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "size": 12345678
        },
        {
          "target": "aarch64-apple-darwin",
          "url": "https://github.com/owner/repo/releases/download/v1.10.0/tool-1.10.0-aarch64-apple-darwin-2.tar.gz",
          "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          "size": 12345678
        }"#,
    );

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::DuplicateManifestTarget { .. }));
}

#[test]
fn rejects_a_non_https_asset_url() {
    let json = valid_manifest_json().replace("https://github.com/owner/repo/releases/download", "http://example.com");

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::InsecureAssetUrl { .. }));
}

#[test]
fn rejects_an_invalid_checksum() {
    let json = valid_manifest_json().replace(
        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"",
        "\"not-a-digest\"",
    );

    let error = UpdateManifest::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::InvalidSha256Digest { .. }));
}
