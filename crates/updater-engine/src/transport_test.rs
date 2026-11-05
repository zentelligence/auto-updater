use super::*;

#[test]
fn fetch_rejects_a_non_https_url() {
    let transport = UreqTransport::default();

    let error = transport
        .fetch("http://example.com/latest.json", MAX_DOCUMENT_BYTES)
        .unwrap_err();

    assert!(matches!(error, EngineError::InsecureManifestUrl { .. }));
}

#[test]
fn stream_rejects_a_non_https_url() {
    let transport = UreqTransport::default();
    let mut sink = Vec::new();

    let error = transport
        .stream("http://example.com/archive.tar.gz", &mut sink)
        .unwrap_err();

    assert!(matches!(error, EngineError::InsecureManifestUrl { .. }));
}
