use super::*;

#[test]
fn installation_root_accepts_an_absolute_path() {
    let root = InstallationRoot::try_from(PathBuf::from("/home/user/.local/share/tool")).unwrap();

    assert_eq!(root.as_path(), Path::new("/home/user/.local/share/tool"));
}

#[test]
fn installation_root_rejects_a_relative_path() {
    let error = InstallationRoot::try_from(PathBuf::from("relative/path")).unwrap_err();

    assert!(matches!(error, CoreError::InstallationRootNotAbsolute { .. }));
}

#[test]
fn confined_path_accepts_a_simple_relative_path() {
    let path = ConfinedRelativePath::try_from("bin/tool").unwrap();

    assert_eq!(path.as_path(), Path::new("bin/tool"));
}

#[test]
fn confined_path_rejects_an_absolute_path() {
    let error = ConfinedRelativePath::try_from("/etc/passwd").unwrap_err();

    assert!(matches!(error, CoreError::PathNotRelative { .. }));
}

#[test]
fn confined_path_rejects_parent_directory_traversal() {
    let error = ConfinedRelativePath::try_from("../etc/passwd").unwrap_err();

    assert!(matches!(error, CoreError::PathTraversal { .. }));
}

#[test]
fn confined_path_rejects_embedded_parent_directory_traversal() {
    let error = ConfinedRelativePath::try_from("bin/../../etc/passwd").unwrap_err();

    assert!(matches!(error, CoreError::PathTraversal { .. }));
}

#[test]
fn confined_path_rejects_an_empty_path() {
    let error = ConfinedRelativePath::try_from("").unwrap_err();

    assert!(matches!(error, CoreError::PathEmpty { .. }));
}

#[test]
fn confined_path_normalises_current_directory_components() {
    let path = ConfinedRelativePath::try_from("./bin/./tool").unwrap();

    assert_eq!(path.as_path(), Path::new("bin/tool"));
}

#[test]
fn resolve_within_joins_onto_the_installation_root() {
    let root = InstallationRoot::try_from(PathBuf::from("/opt/tool")).unwrap();
    let path = ConfinedRelativePath::try_from("versions/1.10.0/bin/tool").unwrap();

    let resolved = path.resolve_within(root.as_path()).unwrap();

    assert_eq!(resolved, Path::new("/opt/tool/versions/1.10.0/bin/tool"));
}

#[test]
fn resolve_within_rejects_a_relative_root() {
    let path = ConfinedRelativePath::try_from("bin/tool").unwrap();

    let error = path.resolve_within(Path::new("relative/root")).unwrap_err();

    assert!(matches!(error, CoreError::InstallationRootNotAbsolute { .. }));
}
