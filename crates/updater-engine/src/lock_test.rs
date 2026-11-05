use tempfile::tempdir;

use super::*;

#[test]
fn acquires_and_releases_the_lock() {
    let temp = tempdir().unwrap();

    let lock = UpdateLock::acquire(temp.path()).unwrap();
    drop(lock);

    let lock = UpdateLock::acquire(temp.path()).unwrap();
    drop(lock);
}

#[test]
fn try_acquire_fails_while_another_handle_holds_the_lock() {
    let temp = tempdir().unwrap();
    let held = UpdateLock::acquire(temp.path()).unwrap();

    let attempt = UpdateLock::try_acquire(temp.path()).unwrap();

    assert!(attempt.is_none());
    drop(held);
}

#[test]
fn try_acquire_succeeds_once_the_holder_releases_it() {
    let temp = tempdir().unwrap();
    let held = UpdateLock::acquire(temp.path()).unwrap();
    drop(held);

    let attempt = UpdateLock::try_acquire(temp.path()).unwrap();

    assert!(attempt.is_some());
}
