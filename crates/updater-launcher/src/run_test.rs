use super::*;

#[cfg(unix)]
#[test]
fn preserves_the_child_exit_code() {
    let resolved = ResolvedCommand {
        version: updater_core::ProductVersion::try_from("1.10.0").unwrap(),
        executable_path: std::path::PathBuf::from("/bin/sh"),
    };
    let args = vec![OsString::from("-c"), OsString::from("exit 7")];

    let status = run_resolved_command(&resolved, "tool", &args).unwrap();

    assert_eq!(status.code(), Some(7));
}

#[cfg(unix)]
#[test]
fn reports_a_spawn_failure_for_a_nonexistent_executable() {
    let resolved = ResolvedCommand {
        version: updater_core::ProductVersion::try_from("1.10.0").unwrap(),
        executable_path: std::path::PathBuf::from("/no/such/executable"),
    };

    let error = run_resolved_command(&resolved, "tool", &[]).unwrap_err();

    assert!(matches!(error, LauncherError::Spawn { .. }));
}
