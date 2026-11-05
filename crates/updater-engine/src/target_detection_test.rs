use super::*;

#[test]
fn detects_a_supported_target_for_the_current_test_host() {
    // Every CI runner for this workspace is one of the six triples this function maps; a host
    // this function cannot map is a real gap this test is designed to surface.
    let target = detect_target().expect("supported host platform");

    assert!(!target.as_str().is_empty());
}
