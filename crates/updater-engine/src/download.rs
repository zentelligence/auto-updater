use std::fs::File;
use std::path::{Path, PathBuf};

use crate::error::EngineError;
use crate::transport::HttpTransport;

/// Downloads `url` into a uniquely named file under `downloads_dir`, streaming to disk rather than
/// buffering the whole archive in memory. The returned path is not yet checksum-verified; the
/// caller must verify it before extracting or promoting it.
pub fn download_to_file(
    transport: &dyn HttpTransport,
    url: &str,
    downloads_dir: &Path,
) -> Result<PathBuf, EngineError> {
    std::fs::create_dir_all(downloads_dir).map_err(|source| EngineError::AtomicWrite {
        path: downloads_dir.to_path_buf(),
        source,
    })?;

    let file_name = url.rsplit('/').next().unwrap_or("release-archive");
    let destination = downloads_dir.join(format!(".{file_name}.{}.download", std::process::id()));

    let mut file = File::create(&destination).map_err(|source| EngineError::AtomicWrite {
        path: destination.clone(),
        source,
    })?;

    if let Err(error) = transport.stream(url, &mut file) {
        let _ = std::fs::remove_file(&destination);
        return Err(error);
    }

    Ok(destination)
}

#[cfg(test)]
#[path = "download_test.rs"]
mod tests;
