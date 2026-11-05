use std::path::Path;

use super::*;

#[test]
fn computes_the_known_digest_of_the_empty_input() {
    let digest = compute_sha256(std::io::empty(), Path::new("<empty>")).unwrap();

    assert_eq!(
        digest.as_str(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn computes_the_known_digest_of_a_short_string() {
    let digest = compute_sha256("abc".as_bytes(), Path::new("<memory>")).unwrap();

    assert_eq!(
        digest.as_str(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn verify_accepts_matching_digests() {
    let digest = compute_sha256("abc".as_bytes(), Path::new("<memory>")).unwrap();

    verify_sha256(&digest, &digest).expect("digests match");
}

#[test]
fn verify_rejects_mismatched_digests() {
    let expected = compute_sha256("abc".as_bytes(), Path::new("<memory>")).unwrap();
    let actual = compute_sha256("xyz".as_bytes(), Path::new("<memory>")).unwrap();

    let error = verify_sha256(&expected, &actual).unwrap_err();

    assert!(matches!(error, EngineError::ChecksumMismatch { .. }));
}
