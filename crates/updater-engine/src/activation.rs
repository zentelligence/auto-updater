use std::path::{Path, PathBuf};

use updater_core::{ActivationPort, ActiveVersionReader, CoreError, ProductVersion};

/// The filesystem-backed [`ActivationPort`] for this platform: a `current`/`previous` symlink pair
/// on Unix-like systems, or a `current.txt`/`previous.txt` pointer-file pair on Windows.
pub struct FilesystemActivationPort {
    root: PathBuf,
}

impl FilesystemActivationPort {
    pub fn new(root: PathBuf) -> Self {
        FilesystemActivationPort { root }
    }
}

fn io_error(path: &Path, source: std::io::Error) -> CoreError {
    CoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(unix)]
mod platform {
    use std::os::unix::fs::symlink;

    use updater_core::version_from_symlink_target;

    use super::*;

    fn read_pointer(root: &Path, name: &str) -> Result<Option<ProductVersion>, CoreError> {
        let pointer = root.join(name);
        match std::fs::read_link(&pointer) {
            Ok(target) => Ok(Some(version_from_symlink_target(&target)?)),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(io_error(&pointer, source)),
        }
    }

    fn write_pointer(root: &Path, name: &str, version: &ProductVersion) -> Result<(), CoreError> {
        let pointer = root.join(name);
        let temp = root.join(format!(".{name}.{}.tmp", std::process::id()));
        let target = format!("versions/{version}");

        symlink(&target, &temp).map_err(|source| io_error(&temp, source))?;
        std::fs::rename(&temp, &pointer).map_err(|source| {
            let _ = std::fs::remove_file(&temp);
            io_error(&pointer, source)
        })
    }

    impl ActiveVersionReader for super::FilesystemActivationPort {
        fn active_version(&self) -> Result<Option<ProductVersion>, CoreError> {
            read_pointer(&self.root, "current")
        }
    }

    impl ActivationPort for super::FilesystemActivationPort {
        fn previous_version(&self) -> Result<Option<ProductVersion>, CoreError> {
            read_pointer(&self.root, "previous")
        }

        fn activate(&self, version: &ProductVersion) -> Result<(), CoreError> {
            if let Some(formerly_active) = self.active_version()? {
                write_pointer(&self.root, "previous", &formerly_active)?;
            }
            write_pointer(&self.root, "current", version)
        }
    }
}

#[cfg(windows)]
mod platform {
    use updater_core::{AtomicWriter, version_from_pointer_text};

    use super::*;
    use crate::atomic_write::FilesystemAtomicWriter;

    fn read_pointer(root: &Path, file_name: &str) -> Result<Option<ProductVersion>, CoreError> {
        let pointer = root.join(file_name);
        match std::fs::read_to_string(&pointer) {
            Ok(text) => Ok(Some(version_from_pointer_text(&text)?)),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(io_error(&pointer, source)),
        }
    }

    fn write_pointer(root: &Path, file_name: &str, version: &ProductVersion) -> Result<(), CoreError> {
        let pointer = root.join(file_name);
        FilesystemAtomicWriter.write_atomic(&pointer, version.to_string().as_bytes())
    }

    impl ActiveVersionReader for super::FilesystemActivationPort {
        fn active_version(&self) -> Result<Option<ProductVersion>, CoreError> {
            read_pointer(&self.root, "current.txt")
        }
    }

    impl ActivationPort for super::FilesystemActivationPort {
        fn previous_version(&self) -> Result<Option<ProductVersion>, CoreError> {
            read_pointer(&self.root, "previous.txt")
        }

        fn activate(&self, version: &ProductVersion) -> Result<(), CoreError> {
            if let Some(formerly_active) = self.active_version()? {
                write_pointer(&self.root, "previous.txt", &formerly_active)?;
            }
            write_pointer(&self.root, "current.txt", version)
        }
    }
}

#[cfg(test)]
#[path = "activation_test.rs"]
mod tests;
