use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use updater_core::{AtomicWriter, CoreError};

/// A filesystem-backed [`AtomicWriter`]: write a uniquely named temporary file in the target's own
/// directory, flush it to disk, then atomically rename it over the target.
///
/// The temporary file is created in the same directory as `target` so the final rename is a
/// same-filesystem operation, which is what makes it atomic on both Unix-like systems and Windows.
pub struct FilesystemAtomicWriter;

impl AtomicWriter for FilesystemAtomicWriter {
    fn write_atomic(&self, target: &Path, contents: &[u8]) -> Result<(), CoreError> {
        let parent = target.parent().unwrap_or_else(|| Path::new("."));
        let temp_path = unique_temp_path(parent, target);

        let mut file = File::create(&temp_path).map_err(|source| io_error(&temp_path, source))?;
        file.write_all(contents)
            .map_err(|source| io_error(&temp_path, source))?;
        file.sync_all().map_err(|source| io_error(&temp_path, source))?;
        drop(file);

        fs::rename(&temp_path, target).map_err(|source| {
            let _ = fs::remove_file(&temp_path);
            io_error(target, source)
        })
    }
}

fn unique_temp_path(parent: &Path, target: &Path) -> PathBuf {
    let file_name = target.file_name().and_then(|name| name.to_str()).unwrap_or("state");
    parent.join(format!(".{file_name}.{}.tmp", std::process::id()))
}

fn io_error(path: &Path, source: std::io::Error) -> CoreError {
    CoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
#[path = "atomic_write_test.rs"]
mod tests;
