use super::*;

#[test]
fn parses_a_valid_semantic_version() {
    let version = ProductVersion::try_from("1.10.0").expect("valid version");

    assert_eq!(version.to_string(), "1.10.0");
}

#[test]
fn rejects_a_malformed_version() {
    let error = ProductVersion::try_from("not-a-version").unwrap_err();

    assert!(matches!(error, CoreError::InvalidVersion { .. }));
}

#[test]
fn orders_versions_by_semantic_version_precedence() {
    let older = ProductVersion::try_from("1.9.0").unwrap();
    let newer = ProductVersion::try_from("1.10.0").unwrap();

    assert!(older < newer);
}

#[test]
fn round_trips_through_json() {
    let version = ProductVersion::try_from("1.10.0").unwrap();

    let json = serde_json::to_string(&version).unwrap();
    assert_eq!(json, "\"1.10.0\"");

    let parsed: ProductVersion = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, version);
}

#[test]
fn rejects_malformed_version_from_json() {
    let result: Result<ProductVersion, _> = serde_json::from_str("\"not-a-version\"");

    assert!(result.is_err());
}
