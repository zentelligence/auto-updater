use super::*;

const SIGNING_KEY_A: [u8; 32] = [7_u8; 32];
const SIGNING_KEY_B: [u8; 32] = [9_u8; 32];

fn public_key_for(signing_key_bytes: &[u8; 32]) -> [u8; 32] {
    ed25519_dalek::SigningKey::from_bytes(signing_key_bytes)
        .verifying_key()
        .to_bytes()
}

#[test]
fn accepts_a_valid_signature_from_a_trusted_key() {
    let public_key = public_key_for(&SIGNING_KEY_A);
    let trusted = TrustedKey::new("key-a", &public_key).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted]);
    let message = b"manifest bytes";
    let signature = sign_for_test(&SIGNING_KEY_A, message);

    verifier.verify(message, &signature, "key-a").expect("valid signature");
}

#[test]
fn rejects_a_signature_from_an_untrusted_key() {
    let public_key = public_key_for(&SIGNING_KEY_A);
    let trusted = TrustedKey::new("key-a", &public_key).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted]);
    let message = b"manifest bytes";
    let signature = sign_for_test(&SIGNING_KEY_B, message);

    let error = verifier.verify(message, &signature, "key-a").unwrap_err();

    assert!(matches!(error, EngineError::ManifestSignatureInvalid));
}

#[test]
fn rejects_a_signature_over_tampered_bytes() {
    let public_key = public_key_for(&SIGNING_KEY_A);
    let trusted = TrustedKey::new("key-a", &public_key).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted]);
    let signature = sign_for_test(&SIGNING_KEY_A, b"manifest bytes");

    let error = verifier.verify(b"tampered bytes!!", &signature, "key-a").unwrap_err();

    assert!(matches!(error, EngineError::ManifestSignatureInvalid));
}

#[test]
fn rejects_an_unknown_signing_key_id() {
    let public_key = public_key_for(&SIGNING_KEY_A);
    let trusted = TrustedKey::new("key-a", &public_key).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted]);
    let signature = sign_for_test(&SIGNING_KEY_A, b"manifest bytes");

    let error = verifier
        .verify(b"manifest bytes", &signature, "key-unknown")
        .unwrap_err();

    assert!(matches!(error, EngineError::UnknownSigningKey { .. }));
}

#[test]
fn supports_key_rotation_across_multiple_trusted_keys() {
    let trusted_a = TrustedKey::new("key-a", &public_key_for(&SIGNING_KEY_A)).unwrap();
    let trusted_b = TrustedKey::new("key-b", &public_key_for(&SIGNING_KEY_B)).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted_a, trusted_b]);
    let message = b"manifest bytes";

    let signature_b = sign_for_test(&SIGNING_KEY_B, message);
    verifier
        .verify(message, &signature_b, "key-b")
        .expect("key-b is trusted");
}

#[test]
fn rejects_a_malformed_signature() {
    let trusted = TrustedKey::new("key-a", &public_key_for(&SIGNING_KEY_A)).unwrap();
    let verifier = Ed25519ManifestVerifier::new(vec![trusted]);

    let error = verifier.verify(b"manifest bytes", b"too-short", "key-a").unwrap_err();

    assert!(matches!(error, EngineError::ManifestSignatureInvalid));
}
