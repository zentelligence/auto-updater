use std::path::{Component, Path, PathBuf};

use crate::error::CoreError;

/// A validated, absolute installation-root directory.
///
/// Construction only checks shape (the path must be absolute); resolving symlinks and confirming
/// existence is the caller's responsibility, typically by canonicalising with the filesystem
/// before constructing this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationRoot(PathBuf);

impl InstallationRoot {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<PathBuf> for InstallationRoot {
    type Error = CoreError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if !path.is_absolute() {
            return Err(CoreError::InstallationRootNotAbsolute { path });
        }
        Ok(InstallationRoot(path))
    }
}

/// A relative filesystem path validated to contain no traversal, absolute, or prefix components.
///
/// This is the sole boundary for turning an archive entry name or a `release.json` command path
/// into something that may touch the filesystem. It never asserts confinement by itself; callers
/// must still resolve it against an absolute confinement root with
/// [`ConfinedRelativePath::resolve_within`], which revalidates confinement at the point of use. The
/// confinement root is typically an [`InstallationRoot`], but may be any absolute directory the
/// caller has already validated, such as one release set's own directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfinedRelativePath(PathBuf);

impl ConfinedRelativePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Joins this path onto `root` and revalidates that the result remains within `root`.
    ///
    /// Revalidation is a defence-in-depth check against a regression in construction; it does not
    /// perform any filesystem I/O and cannot detect confinement violations introduced by symlinks
    /// created after this call returns.
    pub fn resolve_within(&self, root: &Path) -> Result<PathBuf, CoreError> {
        if !root.is_absolute() {
            return Err(CoreError::InstallationRootNotAbsolute {
                path: root.to_path_buf(),
            });
        }
        let resolved = root.join(&self.0);
        if resolved.strip_prefix(root).is_err() {
            return Err(CoreError::PathEscapesRoot {
                resolved,
                root: root.to_path_buf(),
            });
        }
        Ok(resolved)
    }
}

impl TryFrom<&str> for ConfinedRelativePath {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ConfinedRelativePath::try_from(PathBuf::from(value))
    }
}

impl TryFrom<PathBuf> for ConfinedRelativePath {
    type Error = CoreError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        if value.as_os_str().is_empty() {
            return Err(CoreError::PathEmpty { value });
        }
        if value.is_absolute() {
            return Err(CoreError::PathNotRelative { value });
        }

        let mut normalised = PathBuf::new();
        for component in value.components() {
            match component {
                Component::Normal(part) => normalised.push(part),
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(CoreError::PathTraversal { value });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(CoreError::PathInvalidComponent { value });
                }
            }
        }

        if normalised.as_os_str().is_empty() {
            return Err(CoreError::PathEmpty { value });
        }

        Ok(ConfinedRelativePath(normalised))
    }
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;
