use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

use crate::error::EngineError;

/// An exclusive, held-for-the-duration-of-the-call lock over the whole check/download/verify/
/// activate/rollback/cleanup transaction, serialising concurrent invocations against the same
/// installation root.
pub struct UpdateLock {
    file: File,
    path: PathBuf,
}

impl UpdateLock {
    /// The path of the lock file this guard holds.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Acquires the lock at `<root>/update.lock`, blocking until it is available.
    pub fn acquire(root: &Path) -> Result<Self, EngineError> {
        let path = root.join("update.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|source| EngineError::LockUnavailable {
                path: path.clone(),
                reason: source.to_string(),
            })?;
        file.lock().map_err(|source| EngineError::LockUnavailable {
            path: path.clone(),
            reason: source.to_string(),
        })?;
        Ok(UpdateLock { file, path })
    }

    /// Attempts to acquire the lock without blocking, returning `Ok(None)` if another process
    /// already holds it.
    pub fn try_acquire(root: &Path) -> Result<Option<Self>, EngineError> {
        let path = root.join("update.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|source| EngineError::LockUnavailable {
                path: path.clone(),
                reason: source.to_string(),
            })?;
        match file.try_lock() {
            Ok(()) => Ok(Some(UpdateLock { file, path })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(source)) => Err(EngineError::LockUnavailable {
                path,
                reason: source.to_string(),
            }),
        }
    }
}

impl Drop for UpdateLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
#[path = "lock_test.rs"]
mod tests;
