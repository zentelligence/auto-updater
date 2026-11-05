use tempfile::tempdir;

use super::*;
use crate::test_support::FakeTransport;

#[test]
fn downloads_the_response_body_to_a_file_under_the_downloads_directory() {
    let temp = tempdir().unwrap();
    let downloads_dir = temp.path().join("downloads");
    let transport =
        FakeTransport::default().with_response("https://example.com/tool-1.10.0.tar.gz", b"archive bytes".to_vec());

    let path = download_to_file(&transport, "https://example.com/tool-1.10.0.tar.gz", &downloads_dir).unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), b"archive bytes");
    assert!(path.starts_with(&downloads_dir));
}

#[test]
fn removes_the_partial_file_when_the_transport_fails() {
    let temp = tempdir().unwrap();
    let downloads_dir = temp.path().join("downloads");
    let transport = FakeTransport::default();

    let error = download_to_file(&transport, "https://example.com/missing.tar.gz", &downloads_dir).unwrap_err();

    assert!(matches!(error, EngineError::Transport { .. }));
    assert_eq!(std::fs::read_dir(&downloads_dir).unwrap().count(), 0);
}
