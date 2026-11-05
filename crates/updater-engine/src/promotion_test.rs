use tempfile::tempdir;

use super::*;

#[test]
fn promotes_a_staging_directory_to_the_versioned_directory() {
    let temp = tempdir().unwrap();
    let staging = temp.path().join("staging").join("abc123");
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("release.json"), b"{}").unwrap();
    let versions_dir = temp.path().join("versions");
    let version = ProductVersion::try_from("1.10.0").unwrap();

    let target = promote_release(&staging, &versions_dir, &version).unwrap();

    assert_eq!(target, versions_dir.join("1.10.0"));
    assert!(target.join("release.json").is_file());
    assert!(!staging.exists());
}

#[test]
fn rejects_promoting_over_an_already_installed_version() {
    let temp = tempdir().unwrap();
    let versions_dir = temp.path().join("versions");
    let version = ProductVersion::try_from("1.10.0").unwrap();
    fs::create_dir_all(versions_dir.join("1.10.0")).unwrap();
    let staging = temp.path().join("staging").join("abc123");
    fs::create_dir_all(&staging).unwrap();

    let error = promote_release(&staging, &versions_dir, &version).unwrap_err();

    assert!(matches!(error, EngineError::VersionAlreadyInstalled { .. }));
}
