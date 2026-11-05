use std::collections::HashSet;

use crate::error::CoreError;
use crate::paths::ConfinedRelativePath;
use crate::target::TargetTriple;
use crate::version::ProductVersion;

/// The `schema_version` this crate knows how to read. A newer value is rejected rather than
/// silently ignored, per the release contract.
pub const RELEASE_METADATA_SCHEMA_VERSION: u64 = 1;

/// The wire representation of `release.json`. Unknown fields are rejected here, at the
/// deserialisation boundary, rather than by a hand-rolled parser.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseMetadataDto {
    schema_version: u64,
    product: String,
    version: String,
    target: String,
    published_at: String,
    minimum_launcher_version: String,
    commands: Vec<CommandDeclarationDto>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandDeclarationDto {
    name: String,
    path: String,
}

/// A single public command declared by a release set, as validated domain data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDeclaration {
    pub name: String,
    pub path: ConfinedRelativePath,
}

/// The validated contents of a release set's `release.json`.
///
/// Parsing only performs structural validation (schema version, well-formed fields, no duplicate
/// command names, at least one command, and path confinement). Contextual bundle validation, such
/// as matching the installed product or detected target, and executable validation belong to
/// `updater-engine`, which has the filesystem and process context to perform them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseMetadata {
    pub product: String,
    pub version: ProductVersion,
    pub target: TargetTriple,
    pub published_at: String,
    pub minimum_launcher_version: ProductVersion,
    pub commands: Vec<CommandDeclaration>,
}

impl TryFrom<&[u8]> for ReleaseMetadata {
    type Error = CoreError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let dto: ReleaseMetadataDto =
            serde_json::from_slice(bytes).map_err(|source| CoreError::ReleaseMetadataDecode { source })?;

        if dto.schema_version != RELEASE_METADATA_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion {
                what: "release.json",
                found: dto.schema_version,
                expected: RELEASE_METADATA_SCHEMA_VERSION,
            });
        }

        let version = ProductVersion::try_from(dto.version)?;
        let target = TargetTriple::try_from(dto.target.as_str())?;
        let minimum_launcher_version = ProductVersion::try_from(dto.minimum_launcher_version)?;

        if dto.commands.is_empty() {
            return Err(CoreError::NoCommandsDeclared {
                product: dto.product,
                version: version.to_string(),
            });
        }

        let mut seen_names = HashSet::with_capacity(dto.commands.len());
        let mut commands = Vec::with_capacity(dto.commands.len());
        for command in dto.commands {
            if !seen_names.insert(command.name.clone()) {
                return Err(CoreError::DuplicateCommandName { name: command.name });
            }
            let path = ConfinedRelativePath::try_from(command.path.as_str())?;
            commands.push(CommandDeclaration {
                name: command.name,
                path,
            });
        }

        Ok(ReleaseMetadata {
            product: dto.product,
            version,
            target,
            published_at: dto.published_at,
            minimum_launcher_version,
            commands,
        })
    }
}

impl ReleaseMetadata {
    /// Finds the declared command with the given public name, if any.
    pub fn command(&self, name: &str) -> Option<&CommandDeclaration> {
        self.commands.iter().find(|command| command.name == name)
    }
}

#[cfg(test)]
#[path = "release_metadata_test.rs"]
mod tests;
