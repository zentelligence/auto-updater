use super::*;

#[test]
fn parses_a_valid_envelope() {
    let json = r#"{ "key_id": "primary", "signature": "deadbeef" }"#;

    let envelope = SignatureEnvelope::parse(json.as_bytes()).unwrap();

    assert_eq!(envelope.key_id, "primary");
    assert_eq!(envelope.signature_bytes().unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn rejects_unknown_fields() {
    let json = r#"{ "key_id": "primary", "signature": "deadbeef", "extra": true }"#;

    let error = SignatureEnvelope::parse(json.as_bytes()).unwrap_err();

    assert!(matches!(error, EngineError::InvalidSignatureEnvelope { .. }));
}

#[test]
fn rejects_a_non_hex_signature() {
    let json = r#"{ "key_id": "primary", "signature": "not-hex!!" }"#;

    let envelope = SignatureEnvelope::parse(json.as_bytes()).unwrap();
    let error = envelope.signature_bytes().unwrap_err();

    assert!(matches!(error, EngineError::InvalidSignatureEnvelope { .. }));
}

#[test]
fn rejects_an_odd_length_signature() {
    let json = r#"{ "key_id": "primary", "signature": "abc" }"#;

    let envelope = SignatureEnvelope::parse(json.as_bytes()).unwrap();
    let error = envelope.signature_bytes().unwrap_err();

    assert!(matches!(error, EngineError::InvalidSignatureEnvelope { .. }));
}
