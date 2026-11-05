use std::collections::HashSet;

use crate::error::CoreError;
use crate::hash::Sha256Digest;
use crate::target::TargetTriple;
use crate::version::ProductVersion;

/// The `schema_version` this crate knows how to read for the signed update manifest.
pub const UPDATE_MANIFEST_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateManifestDto {
    schema_version: u64,
    product: String,
    channel: String,
    version: String,
    released_at: String,
    minimum_supported_version: String,
    minimum_launcher_version: String,
    release_notes_url: String,
    assets: Vec<AssetDto>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetDto {
    target: String,
    url: String,
    sha256: String,
    size: u64,
}

/// One target-specific downloadable archive declared by the update manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDescriptor {
    pub target: TargetTriple,
    pub url: String,
    pub sha256: Sha256Digest,
    pub size: u64,
}

/// The validated contents of the signed update manifest (`latest.json`).
///
/// Parsing here only performs structural validation: schema version, well-formed fields, unique
/// asset targets, and that every asset URL is HTTPS. Signature verification over the exact
/// manifest bytes happens in `updater-engine`, before any of these fields are trusted; this type
/// must never be constructed from unverified bytes in production code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateManifest {
    pub product: String,
    pub channel: String,
    pub version: ProductVersion,
    pub released_at: String,
    pub minimum_supported_version: ProductVersion,
    pub minimum_launcher_version: ProductVersion,
    pub release_notes_url: String,
    pub assets: Vec<AssetDescriptor>,
}

impl TryFrom<&[u8]> for UpdateManifest {
    type Error = CoreError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let dto: UpdateManifestDto =
            serde_json::from_slice(bytes).map_err(|source| CoreError::ManifestDecode { source })?;

        if dto.schema_version != UPDATE_MANIFEST_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion {
                what: "update manifest",
                found: dto.schema_version,
                expected: UPDATE_MANIFEST_SCHEMA_VERSION,
            });
        }

        let version = ProductVersion::try_from(dto.version)?;
        let minimum_supported_version = ProductVersion::try_from(dto.minimum_supported_version)?;
        let minimum_launcher_version = ProductVersion::try_from(dto.minimum_launcher_version)?;

        if dto.assets.is_empty() {
            return Err(CoreError::NoAssetsDeclared);
        }

        let mut seen_targets = HashSet::with_capacity(dto.assets.len());
        let mut assets = Vec::with_capacity(dto.assets.len());
        for asset in dto.assets {
            let target = TargetTriple::try_from(asset.target.as_str())?;
            if !seen_targets.insert(target.clone()) {
                return Err(CoreError::DuplicateManifestTarget {
                    target: target.to_string(),
                });
            }
            if !asset.url.starts_with("https://") {
                return Err(CoreError::InsecureAssetUrl { url: asset.url });
            }
            let sha256 = Sha256Digest::try_from(asset.sha256)?;
            assets.push(AssetDescriptor {
                target,
                url: asset.url,
                sha256,
                size: asset.size,
            });
        }

        Ok(UpdateManifest {
            product: dto.product,
            channel: dto.channel,
            version,
            released_at: dto.released_at,
            minimum_supported_version,
            minimum_launcher_version,
            release_notes_url: dto.release_notes_url,
            assets,
        })
    }
}

impl UpdateManifest {
    /// Finds the asset matching the given detected target triple, if any.
    pub fn asset_for(&self, target: &TargetTriple) -> Option<&AssetDescriptor> {
        self.assets.iter().find(|asset| &asset.target == target)
    }
}

#[cfg(test)]
#[path = "manifest_test.rs"]
mod tests;
