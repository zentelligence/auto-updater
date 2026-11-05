use super::*;

#[test]
fn invalid_version_message_names_the_offending_value() {
    let source = semver::Version::parse("not-a-version").unwrap_err();
    let error = CoreError::InvalidVersion {
        value: "not-a-version".to_string(),
        source,
    };

    assert_eq!(
        error.to_string(),
        "version \"not-a-version\" is not a valid semantic version"
    );
}

#[test]
fn path_traversal_message_names_the_offending_path() {
    let error = CoreError::PathTraversal {
        value: PathBuf::from("../etc/passwd"),
    };

    assert!(error.to_string().contains("../etc/passwd"));
}

#[test]
fn unsupported_schema_version_message_names_expected_and_found() {
    let error = CoreError::UnsupportedSchemaVersion {
        what: "release.json",
        found: 2,
        expected: 1,
    };

    assert_eq!(
        error.to_string(),
        "release.json declares unsupported schema_version 2 (expected 1)"
    );
}
