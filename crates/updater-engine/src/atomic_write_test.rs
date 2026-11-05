use tempfile::tempdir;

use super::*;

#[test]
fn writes_the_target_file_with_the_given_contents() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("state.json");

    FilesystemAtomicWriter.write_atomic(&target, b"hello").unwrap();

    assert_eq!(std::fs::read(&target).unwrap(), b"hello");
}

#[test]
fn overwrites_an_existing_target_atomically() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("state.json");
    std::fs::write(&target, b"old").unwrap();

    FilesystemAtomicWriter.write_atomic(&target, b"new").unwrap();

    assert_eq!(std::fs::read(&target).unwrap(), b"new");
}

#[test]
fn leaves_no_temporary_file_behind_on_success() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("state.json");

    FilesystemAtomicWriter.write_atomic(&target, b"hello").unwrap();

    let remaining: Vec<_> = std::fs::read_dir(temp.path()).unwrap().collect();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn reports_an_error_for_a_missing_parent_directory() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("missing").join("state.json");

    let error = FilesystemAtomicWriter.write_atomic(&target, b"hello").unwrap_err();

    assert!(matches!(error, CoreError::Io { .. }));
}
