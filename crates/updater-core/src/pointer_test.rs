use std::path::Path;

use super::*;

#[test]
fn parses_a_version_from_a_relative_symlink_target() {
    let version = version_from_symlink_target(Path::new("versions/1.10.0")).unwrap();

    assert_eq!(version.to_string(), "1.10.0");
}

#[test]
fn parses_a_version_from_a_symlink_target_with_a_trailing_slash() {
    let version = version_from_symlink_target(Path::new("versions/1.10.0/")).unwrap();

    assert_eq!(version.to_string(), "1.10.0");
}

#[test]
fn rejects_a_symlink_target_with_a_malformed_version_component() {
    let error = version_from_symlink_target(Path::new("versions/not-a-version")).unwrap_err();

    assert!(matches!(error, CoreError::InvalidVersion { .. }));
}

#[test]
fn parses_a_version_from_pointer_text() {
    let version = version_from_pointer_text("1.10.0\n").unwrap();

    assert_eq!(version.to_string(), "1.10.0");
}

#[test]
fn rejects_malformed_pointer_text() {
    let error = version_from_pointer_text("not-a-version").unwrap_err();

    assert!(matches!(error, CoreError::InvalidVersion { .. }));
}
