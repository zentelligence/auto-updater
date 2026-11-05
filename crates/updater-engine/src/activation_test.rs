use tempfile::tempdir;

use super::*;

#[cfg(unix)]
mod unix {
    use super::*;

    #[test]
    fn reports_no_active_release_before_any_activation() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());

        assert!(port.active_version().unwrap().is_none());
        assert!(port.previous_version().unwrap().is_none());
    }

    #[test]
    fn activate_sets_the_current_symlink() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        let version = ProductVersion::try_from("1.10.0").unwrap();

        port.activate(&version).unwrap();

        assert_eq!(port.active_version().unwrap(), Some(version));
        assert!(port.previous_version().unwrap().is_none());
    }

    #[test]
    fn activating_a_second_release_moves_the_first_to_previous() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        let first = ProductVersion::try_from("1.9.0").unwrap();
        let second = ProductVersion::try_from("1.10.0").unwrap();

        port.activate(&first).unwrap();
        port.activate(&second).unwrap();

        assert_eq!(port.active_version().unwrap(), Some(second));
        assert_eq!(port.previous_version().unwrap(), Some(first));
    }

    #[test]
    fn current_symlink_target_is_relative_to_the_installation_root() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        let version = ProductVersion::try_from("1.10.0").unwrap();

        port.activate(&version).unwrap();

        let target = std::fs::read_link(temp.path().join("current")).unwrap();
        assert_eq!(target, std::path::Path::new("versions/1.10.0"));
    }
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn reports_no_active_release_before_any_activation() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());

        assert!(port.active_version().unwrap().is_none());
        assert!(port.previous_version().unwrap().is_none());
    }

    #[test]
    fn activate_writes_current_txt() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        let version = ProductVersion::try_from("1.10.0").unwrap();

        port.activate(&version).unwrap();

        assert_eq!(port.active_version().unwrap(), Some(version));
        assert_eq!(
            std::fs::read_to_string(temp.path().join("current.txt")).unwrap(),
            "1.10.0"
        );
    }

    #[test]
    fn activating_a_second_release_moves_the_first_to_previous_txt() {
        let temp = tempdir().unwrap();
        let port = FilesystemActivationPort::new(temp.path().to_path_buf());
        let first = ProductVersion::try_from("1.9.0").unwrap();
        let second = ProductVersion::try_from("1.10.0").unwrap();

        port.activate(&first).unwrap();
        port.activate(&second).unwrap();

        assert_eq!(port.active_version().unwrap(), Some(second));
        assert_eq!(port.previous_version().unwrap(), Some(first));
    }
}
