use super::*;

#[test]
fn checksum_mismatch_message_names_both_digests() {
    let error = EngineError::ChecksumMismatch {
        expected: "aaaa".to_string(),
        actual: "bbbb".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "downloaded archive sha256 bbbb does not match manifest sha256 aaaa"
    );
}

#[test]
fn core_error_is_wrapped_transparently() {
    let error = EngineError::from(CoreError::EmptyTargetTriple);

    assert_eq!(error.to_string(), "target triple must not be empty");
}
