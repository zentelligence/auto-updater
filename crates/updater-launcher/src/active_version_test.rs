#[cfg(unix)]
mod unix {
    use std::os::unix::fs::symlink;

    use tempfile::tempdir;
    use updater_core::InstallationRoot;

    use super::super::*;

    #[test]
    fn reads_the_version_from_the_current_symlink() {
        let temp = tempdir().unwrap();
        symlink("versions/1.10.0", temp.path().join("current")).unwrap();
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let version = read_active_version(&root).unwrap();

        assert_eq!(version.to_string(), "1.10.0");
    }

    #[test]
    fn reports_no_active_release_when_the_pointer_is_missing() {
        let temp = tempdir().unwrap();
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let error = read_active_version(&root).unwrap_err();

        assert!(matches!(error, LauncherError::NoActiveRelease { .. }));
    }

    #[test]
    fn rejects_a_symlink_target_with_a_malformed_version() {
        let temp = tempdir().unwrap();
        symlink("versions/not-a-version", temp.path().join("current")).unwrap();
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let error = read_active_version(&root).unwrap_err();

        assert!(matches!(error, LauncherError::Core(_)));
    }
}
