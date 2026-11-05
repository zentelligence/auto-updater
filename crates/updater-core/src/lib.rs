//! Domain types and ports shared across the workspace: manifest and release metadata, local
//! state, validated/confined paths, error types, and the atomic-write and activation-port traits.
//! Depends on nothing else in this workspace.

mod error;
mod hash;
mod manifest;
mod paths;
mod pointer;
mod ports;
mod release_metadata;
mod state;
mod target;
mod version;

pub use error::CoreError;
pub use hash::Sha256Digest;
pub use manifest::{AssetDescriptor, UPDATE_MANIFEST_SCHEMA_VERSION, UpdateManifest};
pub use paths::{ConfinedRelativePath, InstallationRoot};
pub use pointer::{version_from_pointer_text, version_from_symlink_target};
pub use ports::{ActivationPort, ActiveVersionReader, AtomicWriter};
pub use release_metadata::{CommandDeclaration, RELEASE_METADATA_SCHEMA_VERSION, ReleaseMetadata};
pub use state::{LOCAL_STATE_SCHEMA_VERSION, LocalState};
pub use target::TargetTriple;
pub use version::ProductVersion;
