use std::collections::HashSet;
use std::fs;
use std::path::Path;

use updater_core::{
    ActivationPort, ActiveVersionReader, AssetDescriptor, LocalState, ProductVersion, ReleaseMetadata, TargetTriple,
    UpdateManifest,
};

use crate::checksum::{compute_sha256, verify_sha256};
use crate::clock::Clock;
use crate::download::download_to_file;
use crate::error::EngineError;
use crate::executable_validation::{ExecutableVersionProbe, validate_executable_versions};
use crate::promotion::promote_release;
use crate::signature::ManifestVerifier;
use crate::signature_envelope::SignatureEnvelope;
use crate::state_management::{read_state, write_state};
use crate::transport::{HttpTransport, MAX_DOCUMENT_BYTES};
use crate::{archive, bundle_validation, cleanup};

/// Static, product-supplied configuration for one update transaction. Never invents a product
/// name, command list, or installation root; every field is supplied by the embedding product.
pub struct UpdateContext<'a> {
    pub product: &'a str,
    pub installation_root: &'a Path,
    pub manifest_url: &'a str,
    pub target: &'a TargetTriple,
    pub declared_public_commands: &'a [String],
    pub retain_versions: usize,
}

/// The result of `update check`: whether a newer, verified release set is available, without
/// changing the active release.
#[derive(Debug)]
pub struct CheckOutcome {
    pub active_version: Option<ProductVersion>,
    pub manifest: UpdateManifest,
    pub asset: AssetDescriptor,
    pub update_available: bool,
}

fn fetch_verified_manifest(
    transport: &dyn HttpTransport,
    verifier: &dyn ManifestVerifier,
    manifest_url: &str,
) -> Result<UpdateManifest, EngineError> {
    let manifest_bytes = transport.fetch(manifest_url, MAX_DOCUMENT_BYTES)?;
    let signature_url = format!("{manifest_url}.sig");
    let envelope_bytes = transport.fetch(&signature_url, MAX_DOCUMENT_BYTES)?;
    let envelope = SignatureEnvelope::parse(&envelope_bytes)?;

    verifier.verify(&manifest_bytes, &envelope.signature_bytes()?, &envelope.key_id)?;

    Ok(UpdateManifest::try_from(manifest_bytes.as_slice())?)
}

/// Implements `<tool> update check`: retrieves and verifies the manifest, compares versions, and
/// reports the result without touching the active release.
pub fn check_for_update(
    transport: &dyn HttpTransport,
    verifier: &dyn ManifestVerifier,
    activation: &dyn ActiveVersionReader,
    context: &UpdateContext,
) -> Result<CheckOutcome, EngineError> {
    let manifest = fetch_verified_manifest(transport, verifier, context.manifest_url)?;
    let asset = manifest
        .asset_for(context.target)
        .cloned()
        .ok_or_else(|| EngineError::NoMatchingAsset {
            target: context.target.to_string(),
        })?;
    let active_version = activation.active_version()?;
    let update_available = match &active_version {
        Some(active) => manifest.version > *active,
        None => true,
    };

    Ok(CheckOutcome {
        active_version,
        manifest,
        asset,
        update_available,
    })
}

/// The injectable collaborators an install transaction needs: transport, signature verification,
/// executable version probing, activation, and the clock.
pub struct InstallCollaborators<'a> {
    pub transport: &'a dyn HttpTransport,
    pub verifier: &'a dyn ManifestVerifier,
    pub probe: &'a dyn ExecutableVersionProbe,
    pub activation: &'a dyn ActivationPort,
    pub clock: &'a dyn Clock,
}

fn io_error(path: &Path, source: std::io::Error) -> EngineError {
    EngineError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Implements `<tool> update` / `<tool> update --yes`: the full download, verify, extract,
/// validate, promote, activate, and best-effort cleanup sequence from the specification's "Install
/// an Update" process. A failure at any stage leaves the previously active release unchanged.
pub fn install_update(
    collaborators: &InstallCollaborators,
    context: &UpdateContext,
    allow_downgrade: bool,
) -> Result<ProductVersion, EngineError> {
    let manifest = fetch_verified_manifest(collaborators.transport, collaborators.verifier, context.manifest_url)?;
    let asset = manifest
        .asset_for(context.target)
        .ok_or_else(|| EngineError::NoMatchingAsset {
            target: context.target.to_string(),
        })?
        .clone();

    let active_version = collaborators.activation.active_version()?;
    if let Some(active) = &active_version
        && !allow_downgrade
        && manifest.version <= *active
    {
        return Err(EngineError::NoNewerRelease {
            active: active.to_string(),
            available: manifest.version.to_string(),
        });
    }

    let downloads_dir = context.installation_root.join("downloads");
    let downloaded_path = download_to_file(collaborators.transport, &asset.url, &downloads_dir)?;

    let downloaded_file = fs::File::open(&downloaded_path).map_err(|source| io_error(&downloaded_path, source))?;
    let actual_digest = compute_sha256(downloaded_file, &downloaded_path)?;
    if let Err(error) = verify_sha256(&asset.sha256, &actual_digest) {
        let _ = fs::remove_file(&downloaded_path);
        return Err(error);
    }

    let staging_root = context.installation_root.join("staging");
    fs::create_dir_all(&staging_root).map_err(|source| io_error(&staging_root, source))?;
    let staging_dir = staging_root.join(format!("{}-{}", manifest.version, std::process::id()));

    let asset_file_name = asset.url.rsplit('/').next().unwrap_or(&asset.url);
    if let Err(error) = archive::extract_archive(&downloaded_path, asset_file_name, &staging_dir) {
        let _ = fs::remove_dir_all(&staging_dir);
        let _ = fs::remove_file(&downloaded_path);
        return Err(error);
    }
    let _ = fs::remove_file(&downloaded_path);

    let release_metadata = match read_release_metadata(&staging_dir) {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }
    };

    if let Err(error) = bundle_validation::validate_bundle(
        &release_metadata,
        &staging_dir,
        context.product,
        &manifest.version,
        context.target,
        context.declared_public_commands,
    ) {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(error);
    }

    if let Err(error) = validate_executable_versions(collaborators.probe, &release_metadata, &staging_dir) {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err(error);
    }

    let versions_dir = context.installation_root.join("versions");
    promote_release(&staging_dir, &versions_dir, &manifest.version)?;

    collaborators.activation.activate(&manifest.version)?;

    persist_post_activation_state(context, &manifest, &asset, active_version.clone(), collaborators.clock)?;

    let retained = read_state(context.installation_root)?
        .map(|state| state.installed_versions)
        .unwrap_or_default();
    let keep = retained_versions(
        &retained,
        &manifest.version,
        active_version.as_ref(),
        context.retain_versions,
    );
    let _ = cleanup::cleanup_obsolete_versions(&versions_dir, &keep);

    Ok(manifest.version)
}

/// Chooses which installed versions to keep during cleanup: at minimum the newly active and
/// formerly active (now `previous`) release sets, and up to `retain_versions` of the most recent
/// installed versions overall.
fn retained_versions(
    installed_versions: &[ProductVersion],
    new_active: &ProductVersion,
    previous: Option<&ProductVersion>,
    retain_versions: usize,
) -> HashSet<ProductVersion> {
    let mut sorted = installed_versions.to_vec();
    if !sorted.contains(new_active) {
        sorted.push(new_active.clone());
    }
    sorted.sort();
    sorted.reverse();

    let take_count = retain_versions.max(2);
    let mut keep: HashSet<ProductVersion> = sorted.into_iter().take(take_count).collect();
    keep.insert(new_active.clone());
    if let Some(previous) = previous {
        keep.insert(previous.clone());
    }
    keep
}

fn read_release_metadata(staging_dir: &Path) -> Result<ReleaseMetadata, EngineError> {
    let path = staging_dir.join("release.json");
    let bytes = fs::read(&path).map_err(|source| io_error(&path, source))?;
    Ok(ReleaseMetadata::try_from(bytes.as_slice())?)
}

fn persist_post_activation_state(
    context: &UpdateContext,
    manifest: &UpdateManifest,
    asset: &AssetDescriptor,
    previous_version: Option<ProductVersion>,
    clock: &dyn Clock,
) -> Result<(), EngineError> {
    let mut installed_versions = read_state(context.installation_root)?
        .map(|state| state.installed_versions)
        .unwrap_or_default();
    if !installed_versions.contains(&manifest.version) {
        installed_versions.push(manifest.version.clone());
    }

    let now = clock.now_rfc3339();
    let state = LocalState {
        product: context.product.to_string(),
        channel: manifest.channel.clone(),
        active_version: manifest.version.clone(),
        previous_version,
        installed_versions,
        last_check_at: Some(now.clone()),
        last_successful_update_at: Some(now),
        last_manifest_version: Some(manifest.version.clone()),
        last_manifest_sha256: Some(asset.sha256.clone()),
        last_update_error: None,
    };
    write_state(context.installation_root, &state)
}

#[cfg(test)]
#[path = "transaction_test.rs"]
mod tests;
