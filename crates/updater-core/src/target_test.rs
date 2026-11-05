use super::*;

#[test]
fn accepts_a_well_formed_target_triple() {
    let target = TargetTriple::try_from("aarch64-apple-darwin").unwrap();

    assert_eq!(target.as_str(), "aarch64-apple-darwin");
}

#[test]
fn rejects_an_empty_target_triple() {
    let error = TargetTriple::try_from("").unwrap_err();

    assert!(matches!(error, CoreError::EmptyTargetTriple));
}

#[test]
fn rejects_a_target_triple_containing_whitespace() {
    let error = TargetTriple::try_from("aarch64 apple darwin").unwrap_err();

    assert!(matches!(error, CoreError::EmptyTargetTriple));
}
