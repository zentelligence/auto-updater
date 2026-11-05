use std::path::Path;

use crate::error::CoreError;
use crate::version::ProductVersion;

/// Read-only access to which release version is currently active.
///
/// Implemented by `updater-engine` for both platforms' pointer formats. `updater-launcher` depends
/// only on this trait, never on the write-capable [`ActivationPort`], since a launcher must never
/// activate a release.
pub trait ActiveVersionReader {
    /// Returns the active version, or `Ok(None)` if no release has been activated yet.
    fn active_version(&self) -> Result<Option<ProductVersion>, CoreError>;
}

/// Read-write access to the active and previous release pointers.
///
/// Implemented by `updater-engine`. Every write performed through this trait must follow the
/// atomic-write path: a uniquely named temporary file or pointer on the same filesystem, flushed,
/// then an atomic rename or platform-safe replace over the target.
pub trait ActivationPort: ActiveVersionReader {
    /// Returns the previous version, or `Ok(None)` if there is no prior release to roll back to.
    fn previous_version(&self) -> Result<Option<ProductVersion>, CoreError>;

    /// Atomically activates `version`, moving the formerly active version to `previous`.
    fn activate(&self, version: &ProductVersion) -> Result<(), CoreError>;
}

/// Atomic, flush-then-rename (or platform-safe replace) writes to a single target file.
///
/// This is the sole path by which the active pointer, `previous`, or `state.json` may be written.
/// No code writes those files directly outside an implementation of this trait.
pub trait AtomicWriter {
    fn write_atomic(&self, target: &Path, contents: &[u8]) -> Result<(), CoreError>;
}
