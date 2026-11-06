#[cfg(unix)]
use tempfile::tempdir;

#[cfg(unix)]
use super::*;

#[cfg(unix)]
mod unix {
    use std::fs;

    use updater_core::{ActivationPort, ActiveVersionReader};

    use super::*;
    use crate::activation::FilesystemActivationPort;

    fn install_release_dir(root: &Path, version: &str) {
        let dir = root.join("versions").join(version);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("release.json"), b"{}").unwrap();
    }

    #[test]
    fn rolls_back_to_the_previous_release_set() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        install_release_dir(temp.path(), "1.9.0");
        install_release_dir(temp.path(), "1.10.0");
        port.activate(&ProductVersion::try_from("1.9.0").unwrap()).unwrap();
        port.activate(&ProductVersion::try_from("1.10.0").unwrap()).unwrap();

        let rolled_back_to = rollback(&port, &temp.path().join("versions")).unwrap();

        assert_eq!(rolled_back_to.to_string(), "1.9.0");
        assert_eq!(port.active_version().unwrap().unwrap().to_string(), "1.9.0");
        assert_eq!(port.previous_version().unwrap().unwrap().to_string(), "1.10.0");
    }

    #[test]
    fn rejects_rollback_with_no_previous_release() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        install_release_dir(temp.path(), "1.10.0");
        port.activate(&ProductVersion::try_from("1.10.0").unwrap()).unwrap();

        let error = rollback(&port, &temp.path().join("versions")).unwrap_err();

        assert!(matches!(error, EngineError::NoPreviousRelease));
    }

    #[test]
    fn rejects_rollback_when_the_previous_release_directory_is_gone() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        install_release_dir(temp.path(), "1.9.0");
        install_release_dir(temp.path(), "1.10.0");
        port.activate(&ProductVersion::try_from("1.9.0").unwrap()).unwrap();
        port.activate(&ProductVersion::try_from("1.10.0").unwrap()).unwrap();
        fs::remove_dir_all(temp.path().join("versions").join("1.9.0")).unwrap();

        let error = rollback(&port, &temp.path().join("versions")).unwrap_err();

        assert!(matches!(error, EngineError::NoPreviousRelease));
    }
}
