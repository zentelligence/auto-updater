use crate::error::CoreError;
use crate::hash::Sha256Digest;
use crate::version::ProductVersion;

/// The `schema_version` this crate knows how to read for `state.json`.
pub const LOCAL_STATE_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalStateDto {
    schema_version: u64,
    product: String,
    channel: String,
    active_version: String,
    previous_version: Option<String>,
    installed_versions: Vec<String>,
    last_check_at: Option<String>,
    last_successful_update_at: Option<String>,
    last_manifest_version: Option<String>,
    last_manifest_sha256: Option<String>,
    last_update_error: Option<String>,
}

/// The validated, in-memory contents of `state.json`: the local record of which release sets are
/// installed and which are active, kept in sync with the on-disk pointers by the same atomic-write
/// path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalState {
    pub product: String,
    pub channel: String,
    pub active_version: ProductVersion,
    pub previous_version: Option<ProductVersion>,
    pub installed_versions: Vec<ProductVersion>,
    pub last_check_at: Option<String>,
    pub last_successful_update_at: Option<String>,
    pub last_manifest_version: Option<ProductVersion>,
    pub last_manifest_sha256: Option<Sha256Digest>,
    pub last_update_error: Option<String>,
}

impl TryFrom<&[u8]> for LocalState {
    type Error = CoreError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let dto: LocalStateDto = serde_json::from_slice(bytes).map_err(|source| CoreError::StateDecode { source })?;

        if dto.schema_version != LOCAL_STATE_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion {
                what: "state.json",
                found: dto.schema_version,
                expected: LOCAL_STATE_SCHEMA_VERSION,
            });
        }

        let active_version = ProductVersion::try_from(dto.active_version)?;
        let previous_version = dto.previous_version.map(ProductVersion::try_from).transpose()?;
        let installed_versions = dto
            .installed_versions
            .into_iter()
            .map(ProductVersion::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let last_manifest_version = dto.last_manifest_version.map(ProductVersion::try_from).transpose()?;
        let last_manifest_sha256 = dto.last_manifest_sha256.map(Sha256Digest::try_from).transpose()?;

        Ok(LocalState {
            product: dto.product,
            channel: dto.channel,
            active_version,
            previous_version,
            installed_versions,
            last_check_at: dto.last_check_at,
            last_successful_update_at: dto.last_successful_update_at,
            last_manifest_version,
            last_manifest_sha256,
            last_update_error: dto.last_update_error,
        })
    }
}

impl LocalState {
    /// Serialises this state as the exact bytes to write atomically to `state.json`.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let dto = LocalStateDto {
            schema_version: LOCAL_STATE_SCHEMA_VERSION,
            product: self.product.clone(),
            channel: self.channel.clone(),
            active_version: self.active_version.to_string(),
            previous_version: self.previous_version.as_ref().map(ToString::to_string),
            installed_versions: self.installed_versions.iter().map(ToString::to_string).collect(),
            last_check_at: self.last_check_at.clone(),
            last_successful_update_at: self.last_successful_update_at.clone(),
            last_manifest_version: self.last_manifest_version.as_ref().map(ToString::to_string),
            last_manifest_sha256: self.last_manifest_sha256.as_ref().map(ToString::to_string),
            last_update_error: self.last_update_error.clone(),
        };
        serde_json::to_vec_pretty(&dto).map_err(|source| CoreError::Encode {
            what: "state.json",
            source,
        })
    }
}

#[cfg(test)]
#[path = "state_test.rs"]
mod tests;
