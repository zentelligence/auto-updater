use std::collections::HashSet;

use tempfile::tempdir;

use super::*;

fn make_version_dir(root: &Path, version: &str) {
    fs::create_dir_all(root.join(version)).unwrap();
}

#[test]
fn removes_versions_not_in_the_keep_set() {
    let temp = tempdir().unwrap();
    make_version_dir(temp.path(), "1.8.0");
    make_version_dir(temp.path(), "1.9.0");
    make_version_dir(temp.path(), "1.10.0");
    let keep = HashSet::from([
        ProductVersion::try_from("1.9.0").unwrap(),
        ProductVersion::try_from("1.10.0").unwrap(),
    ]);

    let deferred = cleanup_obsolete_versions(temp.path(), &keep);

    assert!(deferred.is_empty());
    assert!(!temp.path().join("1.8.0").exists());
    assert!(temp.path().join("1.9.0").exists());
    assert!(temp.path().join("1.10.0").exists());
}

#[test]
fn never_removes_a_version_in_the_keep_set() {
    let temp = tempdir().unwrap();
    make_version_dir(temp.path(), "1.10.0");
    let keep = HashSet::from([ProductVersion::try_from("1.10.0").unwrap()]);

    cleanup_obsolete_versions(temp.path(), &keep);

    assert!(temp.path().join("1.10.0").exists());
}

#[test]
fn ignores_a_missing_versions_directory() {
    let temp = tempdir().unwrap();

    let deferred = cleanup_obsolete_versions(&temp.path().join("versions"), &HashSet::new());

    assert!(deferred.is_empty());
}

#[test]
fn leaves_unparseable_directory_names_untouched() {
    let temp = tempdir().unwrap();
    make_version_dir(temp.path(), "not-a-version");

    let deferred = cleanup_obsolete_versions(temp.path(), &HashSet::new());

    assert!(deferred.is_empty());
    assert!(temp.path().join("not-a-version").exists());
}
