use super::*;

fn valid_state_json() -> &'static str {
    r#"{
      "schema_version": 1,
      "product": "tool",
      "channel": "stable",
      "active_version": "1.10.0",
      "previous_version": "1.9.0",
      "installed_versions": ["1.8.0", "1.9.0", "1.10.0"],
      "last_check_at": "2026-09-21T04:15:00Z",
      "last_successful_update_at": "2026-09-21T04:16:43Z",
      "last_manifest_version": "1.10.0",
      "last_manifest_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "last_update_error": null
    }"#
}

#[test]
fn parses_the_smallest_valid_document() {
    let json = r#"{
      "schema_version": 1,
      "product": "tool",
      "channel": "stable",
      "active_version": "1.10.0",
      "previous_version": null,
      "installed_versions": ["1.10.0"],
      "last_check_at": null,
      "last_successful_update_at": null,
      "last_manifest_version": null,
      "last_manifest_sha256": null,
      "last_update_error": null
    }"#;

    let state = LocalState::try_from(json.as_bytes()).expect("valid state");

    assert_eq!(state.active_version.to_string(), "1.10.0");
    assert!(state.previous_version.is_none());
}

#[test]
fn parses_a_representative_complete_document() {
    let state = LocalState::try_from(valid_state_json().as_bytes()).expect("valid state");

    assert_eq!(state.previous_version.unwrap().to_string(), "1.9.0");
    assert_eq!(state.installed_versions.len(), 3);
    assert_eq!(state.last_manifest_sha256.unwrap().as_str().len(), 64);
}

#[test]
fn rejects_unknown_fields() {
    let json = valid_state_json().replace("\"channel\"", "\"unexpected_field\": true, \"channel\"");

    let error = LocalState::try_from(json.as_bytes()).unwrap_err();

    assert!(matches!(error, CoreError::StateDecode { .. }));
}

#[test]
fn rejects_an_unsupported_schema_version() {
    let json = valid_state_json().replace("\"schema_version\": 1", "\"schema_version\": 2");

    let error = LocalState::try_from(json.as_bytes()).unwrap_err();

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
fn round_trips_through_encode_and_parse() {
    let state = LocalState::try_from(valid_state_json().as_bytes()).unwrap();

    let bytes = state.to_json_bytes().unwrap();
    let parsed = LocalState::try_from(bytes.as_slice()).unwrap();

    assert_eq!(parsed, state);
}
