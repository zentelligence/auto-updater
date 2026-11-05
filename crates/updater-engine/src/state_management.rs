use std::path::Path;

use updater_core::{AtomicWriter, LocalState};

use crate::atomic_write::FilesystemAtomicWriter;
use crate::error::EngineError;

/// Reads `state.json` from the installation root, or `Ok(None)` if it does not exist yet (a
/// fresh installation).
pub fn read_state(root: &Path) -> Result<Option<LocalState>, EngineError> {
    let path = root.join("state.json");
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(LocalState::try_from(bytes.as_slice())?)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(EngineError::ReadState { path, source }),
    }
}

/// Writes `state.json` atomically, per the write pipeline: validate, then temp write, flush,
/// atomic rename.
pub fn write_state(root: &Path, state: &LocalState) -> Result<(), EngineError> {
    let path = root.join("state.json");
    let bytes = state.to_json_bytes()?;
    FilesystemAtomicWriter.write_atomic(&path, &bytes)?;
    Ok(())
}

#[cfg(test)]
#[path = "state_management_test.rs"]
mod tests;
