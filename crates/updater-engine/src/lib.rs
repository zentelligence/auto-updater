//! The update transaction: manifest retrieval and signature verification, download, checksum
//! verification, archive extraction, bundle and executable validation, promotion, pointer
//! activation, rollback, cleanup, state management, stable launchers, and the update lock.

mod activation;
mod archive;
mod atomic_write;
mod bundle_validation;
mod checksum;
mod cleanup;
mod clock;
mod download;
mod error;
mod executable_validation;
mod lock;
mod promotion;
mod rollback;
mod signature;
mod signature_envelope;
mod stable_launchers;
mod state_management;
mod target_detection;
mod transaction;
mod transport;

#[cfg(test)]
mod test_support;

pub use activation::FilesystemActivationPort;
pub use archive::extract_archive;
pub use atomic_write::FilesystemAtomicWriter;
pub use bundle_validation::validate_bundle;
pub use checksum::{compute_sha256, verify_sha256};
pub use cleanup::{DeferredCleanup, cleanup_obsolete_versions};
pub use clock::{Clock, SystemClock};
pub use download::download_to_file;
pub use error::EngineError;
pub use executable_validation::{ExecutableVersionProbe, SpawnVersionProbe, validate_executable_versions};
pub use lock::UpdateLock;
pub use promotion::promote_release;
pub use rollback::rollback;
pub use signature::{Ed25519ManifestVerifier, ManifestVerifier, TrustedKey};
pub use signature_envelope::SignatureEnvelope;
pub use stable_launchers::{LauncherRemoval, PathChange, StableLaunchers};
pub use state_management::{read_state, write_state};
pub use target_detection::detect_target;
pub use transaction::{CheckOutcome, InstallCollaborators, UpdateContext, check_for_update, install_update};
pub use transport::{HttpTransport, MAX_DOCUMENT_BYTES, UreqTransport};
