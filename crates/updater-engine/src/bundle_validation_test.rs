use std::fs;

use tempfile::tempdir;
use updater_core::ReleaseMetadata;

use super::*;

fn release_json() -> &'static str {
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

#[cfg(unix)]
fn write_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, b"#!/bin/sh\n").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(windows)]
fn write_executable(path: &Path) {
    fs::write(path, b"stub").unwrap();
}

fn install_fixture() -> (tempfile::TempDir, ReleaseMetadata) {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("bin")).unwrap();
    write_executable(&temp.path().join("bin/tool"));
    write_executable(&temp.path().join("bin/tool-agent"));
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    (temp, metadata)
}

#[test]
fn accepts_a_fully_valid_bundle() {
    let (temp, metadata) = install_fixture();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    validate_bundle(
        &metadata,
        temp.path(),
        "tool",
        &version,
        &target,
        &["tool".to_string(), "tool-agent".to_string()],
    )
    .expect("valid bundle");
}

#[test]
fn rejects_a_product_mismatch() {
    let (temp, metadata) = install_fixture();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    let error = validate_bundle(&metadata, temp.path(), "other-product", &version, &target, &[]).unwrap_err();

    assert!(matches!(error, EngineError::ProductMismatch { .. }));
}

#[test]
fn rejects_a_version_mismatch_against_the_manifest() {
    let (temp, metadata) = install_fixture();
    let version = ProductVersion::try_from("1.11.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    let error = validate_bundle(&metadata, temp.path(), "tool", &version, &target, &[]).unwrap_err();

    assert!(matches!(error, EngineError::VersionMismatch { .. }));
}

#[test]
fn rejects_a_target_mismatch_against_the_detected_platform() {
    let (temp, metadata) = install_fixture();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("x86_64-pc-windows-msvc").unwrap();

    let error = validate_bundle(&metadata, temp.path(), "tool", &version, &target, &[]).unwrap_err();

    assert!(matches!(error, EngineError::TargetMismatch { .. }));
}

#[test]
fn rejects_a_missing_declared_executable() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("bin")).unwrap();
    write_executable(&temp.path().join("bin/tool"));
    // "tool-agent" is declared but never written to disk.
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    let error = validate_bundle(&metadata, temp.path(), "tool", &version, &target, &[]).unwrap_err();

    assert!(matches!(error, EngineError::MissingCommandExecutable { .. }));
}

#[test]
fn rejects_a_stable_launcher_without_a_corresponding_declared_command() {
    let (temp, metadata) = install_fixture();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    let error = validate_bundle(
        &metadata,
        temp.path(),
        "tool",
        &version,
        &target,
        &["tool-worker".to_string()],
    )
    .unwrap_err();

    assert!(matches!(error, EngineError::LauncherWithoutCommand { .. }));
}

#[cfg(unix)]
#[test]
fn rejects_a_declared_executable_without_the_executable_bit() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("bin")).unwrap();
    fs::write(temp.path().join("bin/tool"), b"#!/bin/sh\n").unwrap();
    write_executable(&temp.path().join("bin/tool-agent"));
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    let version = ProductVersion::try_from("1.10.0").unwrap();
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    let error = validate_bundle(&metadata, temp.path(), "tool", &version, &target, &[]).unwrap_err();

    assert!(matches!(error, EngineError::NotExecutable { .. }));
}
