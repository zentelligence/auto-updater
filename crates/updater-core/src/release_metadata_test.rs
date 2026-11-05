use std::path::Path;

use super::*;

fn valid_release_metadata_json() -> &'static str {
    r#"{
      "schema_version": 1,
      "product": "tool",
      "version": "1.10.0",
      "target": "aarch64-apple-darwin",
      "published_at": "2026-09-21T04:00:00Z",
      "minimum_launcher_version": "1.0.0",
      "commands": [
        { "name": "tool", "path": "bin/tool" },
        { "name": "tool-agent", "path": "bin/tool-agent" }
      ]
    }"#
}

#[test]
fn parses_the_smallest_valid_document() {
    let json = r#"{
      "schema_version": 1,
      "product": "tool",
      "version": "1.10.0",
      "target": "aarch64-apple-darwin",
      "published_at": "2026-09-21T04:00:00Z",
      "minimum_launcher_version": "1.0.0",
      "commands": [ { "name": "tool", "path": "bin/tool" } ]
    }"#;

    let metadata = ReleaseMetadata::try_from(json.as_bytes()).expect("valid metadata");

    assert_eq!(metadata.product, "tool");
    assert_eq!(metadata.commands.len(), 1);
}

#[test]
fn parses_a_representative_complete_document() {
    let metadata = ReleaseMetadata::try_from(valid_release_metadata_json().as_bytes()).expect("valid metadata");

    assert_eq!(metadata.version.to_string(), "1.10.0");
    assert_eq!(metadata.target.as_str(), "aarch64-apple-darwin");
    assert_eq!(metadata.commands.len(), 2);
    assert_eq!(
        metadata.command("tool-agent").unwrap().path.as_path(),
        Path::new("bin/tool-agent")
    );
}

#[test]
fn rejects_unknown_fields() {
    let json = valid_release_metadata_json().replace("\"product\"", "\"unexpected_field\": true, \"product\"");

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::ReleaseMetadataDecode { .. }));
}

#[test]
fn rejects_a_missing_required_field() {
    let json = r#"{
      "schema_version": 1,
      "product": "tool",
      "version": "1.10.0",
      "target": "aarch64-apple-darwin",
      "published_at": "2026-09-21T04:00:00Z",
      "commands": []
    }"#;

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::ReleaseMetadataDecode { .. }));
}

#[test]
fn rejects_an_unsupported_schema_version() {
    let json = valid_release_metadata_json().replace("\"schema_version\": 1", "\"schema_version\": 2");

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

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
fn rejects_an_empty_command_list() {
    let json = valid_release_metadata_json().replacen(
        r#"[
        { "name": "tool", "path": "bin/tool" },
        { "name": "tool-agent", "path": "bin/tool-agent" }
      ]"#,
        "[]",
        1,
    );

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::NoCommandsDeclared { .. }));
}

#[test]
fn rejects_duplicate_command_names() {
    let json = r#"{
      "schema_version": 1,
      "product": "tool",
      "version": "1.10.0",
      "target": "aarch64-apple-darwin",
      "published_at": "2026-09-21T04:00:00Z",
      "minimum_launcher_version": "1.0.0",
      "commands": [
        { "name": "tool", "path": "bin/tool" },
        { "name": "tool", "path": "bin/tool-copy" }
      ]
    }"#;

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::DuplicateCommandName { name } if name == "tool"));
}

#[test]
fn rejects_a_command_path_that_escapes_the_release_set() {
    let json = valid_release_metadata_json().replace("bin/tool-agent", "../../etc/passwd");

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::PathTraversal { .. }));
}

#[test]
fn rejects_a_malformed_version() {
    let json = valid_release_metadata_json().replace("\"1.10.0\"", "\"not-a-version\"");

    let error = ReleaseMetadata::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::InvalidVersion { .. }));
}

#[test]
fn command_lookup_returns_none_for_an_undeclared_name() {
    let metadata = ReleaseMetadata::try_from(valid_release_metadata_json().as_bytes()).unwrap();

    assert!(metadata.command("tool-worker").is_none());
}
