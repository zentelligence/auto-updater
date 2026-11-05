use tempfile::tempdir;
use updater_core::ProductVersion;

use super::*;

fn sample_state() -> LocalState {
    LocalState {
        product: "tool".to_string(),
        channel: "stable".to_string(),
        active_version: ProductVersion::try_from("1.10.0").unwrap(),
        previous_version: Some(ProductVersion::try_from("1.9.0").unwrap()),
        installed_versions: vec![ProductVersion::try_from("1.10.0").unwrap()],
        last_check_at: None,
        last_successful_update_at: None,
        last_manifest_version: None,
        last_manifest_sha256: None,
        last_update_error: None,
    }
}

#[test]
fn returns_none_when_state_json_does_not_exist() {
    let temp = tempdir().unwrap();

    let state = read_state(temp.path()).unwrap();

    assert!(state.is_none());
}

#[test]
fn writes_and_reads_back_the_same_state() {
    let temp = tempdir().unwrap();
    let state = sample_state();

    write_state(temp.path(), &state).unwrap();
    let read_back = read_state(temp.path()).unwrap().unwrap();

    assert_eq!(read_back, state);
}

#[test]
fn write_state_overwrites_a_previous_state_file() {
    let temp = tempdir().unwrap();
    write_state(temp.path(), &sample_state()).unwrap();

    let mut updated = sample_state();
    updated.active_version = ProductVersion::try_from("1.11.0").unwrap();
    write_state(temp.path(), &updated).unwrap();

    let read_back = read_state(temp.path()).unwrap().unwrap();
    assert_eq!(read_back.active_version.to_string(), "1.11.0");
}
