#[cfg(unix)]
mod unix {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};

    use tempfile::tempdir;
    use updater_core::InstallationRoot;

    use super::super::*;

    fn release_json(commands: &str) -> String {
        format!(
            r#"{{
              "schema_version": 1,
              "product": "tool",
              "version": "1.10.0",
              "target": "aarch64-apple-darwin",
              "published_at": "2026-09-21T04:00:00Z",
              "minimum_launcher_version": "1.0.0",
              "commands": [{commands}]
            }}"#
        )
    }

    fn install_release_set(root: &std::path::Path) {
        let version_dir = root.join("versions").join("1.10.0");
        fs::create_dir_all(version_dir.join("bin")).unwrap();
        fs::write(
            version_dir.join("release.json"),
            release_json(r#"{ "name": "tool", "path": "bin/tool" }"#),
        )
        .unwrap();
        let executable = version_dir.join("bin").join("tool");
        fs::write(&executable, "#!/bin/sh\necho tool\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        symlink("versions/1.10.0", root.join("current")).unwrap();
    }

    #[test]
    fn resolves_a_declared_command_to_its_executable() {
        let temp = tempdir().unwrap();
        install_release_set(temp.path());
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let resolved = resolve_command(&root, "tool").unwrap();

        assert_eq!(resolved.version.to_string(), "1.10.0");
        assert_eq!(resolved.executable_path, temp.path().join("versions/1.10.0/bin/tool"));
    }

    #[test]
    fn rejects_an_undeclared_command_name() {
        let temp = tempdir().unwrap();
        install_release_set(temp.path());
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let error = resolve_command(&root, "tool-worker").unwrap_err();

        assert!(matches!(error, LauncherError::UndeclaredCommand { .. }));
    }

    #[test]
    fn rejects_a_declared_command_whose_executable_is_missing() {
        let temp = tempdir().unwrap();
        let version_dir = temp.path().join("versions").join("1.10.0");
        fs::create_dir_all(&version_dir).unwrap();
        fs::write(
            version_dir.join("release.json"),
            release_json(r#"{ "name": "tool", "path": "bin/tool" }"#),
        )
        .unwrap();
        symlink("versions/1.10.0", temp.path().join("current")).unwrap();
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let error = resolve_command(&root, "tool").unwrap_err();

        assert!(matches!(error, LauncherError::ExecutableMissing { .. }));
    }

    #[test]
    fn reports_no_active_release_when_there_is_no_current_pointer() {
        let temp = tempdir().unwrap();
        let root = InstallationRoot::try_from(temp.path().to_path_buf()).unwrap();

        let error = resolve_command(&root, "tool").unwrap_err();

        assert!(matches!(error, LauncherError::NoActiveRelease { .. }));
    }
}
