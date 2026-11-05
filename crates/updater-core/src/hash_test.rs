use super::*;

const VALID: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[test]
fn accepts_a_valid_lowercase_digest() {
    let digest = Sha256Digest::try_from(VALID).unwrap();

    assert_eq!(digest.as_str(), VALID);
}

#[test]
fn lower_cases_an_uppercase_digest() {
    let digest = Sha256Digest::try_from(VALID.to_ascii_uppercase().as_str()).unwrap();

    assert_eq!(digest.as_str(), VALID);
}

#[test]
fn rejects_a_digest_of_the_wrong_length() {
    let error = Sha256Digest::try_from("deadbeef").unwrap_err();

    assert!(matches!(error, CoreError::InvalidSha256Digest { .. }));
}

#[test]
fn rejects_a_digest_containing_non_hex_characters() {
    let bad = "g3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85";

    let error = Sha256Digest::try_from(bad).unwrap_err();

    assert!(matches!(error, CoreError::InvalidSha256Digest { .. }));
}
