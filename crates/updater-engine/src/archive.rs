use std::fs::{self, File};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use updater_core::ConfinedRelativePath;

use crate::error::EngineError;

/// Extracts a `.tar.gz` or `.zip` release archive into `staging_dir`.
///
/// Every entry name is converted to a [`ConfinedRelativePath`] and resolved within `staging_dir`
/// before anything is written, per the security requirement that archive entries never touch the
/// filesystem through anything but that validated type.
///
/// Format is detected from `original_file_name` (the manifest asset's own file name), not from
/// `archive_path`, since a downloaded file is stored under a uniquely mangled temporary name that
/// does not preserve the original extension.
pub fn extract_archive(archive_path: &Path, original_file_name: &str, staging_dir: &Path) -> Result<(), EngineError> {
    if original_file_name.ends_with(".tar.gz") || original_file_name.ends_with(".tgz") {
        extract_tar_gz(archive_path, staging_dir)
    } else if original_file_name.ends_with(".zip") {
        extract_zip(archive_path, staging_dir)
    } else {
        Err(EngineError::UnsupportedArchiveFormat {
            path: PathBuf::from(original_file_name),
        })
    }
}

fn open_archive_file(path: &Path) -> Result<File, EngineError> {
    File::open(path).map_err(|source| EngineError::ArchiveEntryUnreadable {
        entry: path.display().to_string(),
        reason: source.to_string(),
    })
}

fn confine(raw: &Path, staging_dir: &Path) -> Result<PathBuf, EngineError> {
    let confined = ConfinedRelativePath::try_from(raw.to_path_buf())?;
    Ok(confined.resolve_within(staging_dir)?)
}

fn extract_tar_gz(archive_path: &Path, staging_dir: &Path) -> Result<(), EngineError> {
    let file = open_archive_file(archive_path)?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    let entries = archive
        .entries()
        .map_err(|source| EngineError::ArchiveEntryUnreadable {
            entry: archive_path.display().to_string(),
            reason: source.to_string(),
        })?;

    for entry_result in entries {
        let mut entry = entry_result.map_err(|source| EngineError::ArchiveEntryUnreadable {
            entry: archive_path.display().to_string(),
            reason: source.to_string(),
        })?;
        let raw_path = entry.path().map_err(archive_entry_error(archive_path))?.into_owned();
        let target = confine(&raw_path, staging_dir)?;

        match entry.header().entry_type() {
            tar::EntryType::Directory => {
                fs::create_dir_all(&target).map_err(archive_entry_error(&target))?;
            }
            tar::EntryType::Regular => {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(archive_entry_error(&target))?;
                }
                let mut out = File::create(&target).map_err(archive_entry_error(&target))?;
                std::io::copy(&mut entry, &mut out).map_err(archive_entry_error(&target))?;
                set_unix_mode(&target, entry.header().mode().unwrap_or(0o644))?;
            }
            other => {
                return Err(EngineError::ArchiveEntryUnreadable {
                    entry: target.display().to_string(),
                    reason: format!("unsupported tar entry type {other:?}"),
                });
            }
        }
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, staging_dir: &Path) -> Result<(), EngineError> {
    let file = open_archive_file(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|source| EngineError::ArchiveEntryUnreadable {
        entry: archive_path.display().to_string(),
        reason: source.to_string(),
    })?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|source| EngineError::ArchiveEntryUnreadable {
                entry: archive_path.display().to_string(),
                reason: source.to_string(),
            })?;
        let name = entry.name().map_err(|source| EngineError::ArchiveEntryUnreadable {
            entry: archive_path.display().to_string(),
            reason: source.to_string(),
        })?;
        let raw_path = PathBuf::from(name.as_ref());
        let target = confine(&raw_path, staging_dir)?;

        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(archive_entry_error(&target))?;
            continue;
        }
        if !entry.is_file() {
            return Err(EngineError::ArchiveEntryUnreadable {
                entry: target.display().to_string(),
                reason: "unsupported zip entry type: only regular files and directories are allowed".to_owned(),
            });
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(archive_entry_error(&target))?;
        }
        let mut out = File::create(&target).map_err(archive_entry_error(&target))?;
        std::io::copy(&mut entry, &mut out).map_err(archive_entry_error(&target))?;
        if let Some(mode) = entry.unix_mode() {
            set_unix_mode(&target, mode)?;
        }
    }
    Ok(())
}

fn archive_entry_error(target: &Path) -> impl Fn(std::io::Error) -> EngineError + '_ {
    move |source| EngineError::ArchiveEntryUnreadable {
        entry: target.display().to_string(),
        reason: source.to_string(),
    }
}

/// Only the owner/group/other read, write, and execute bits are honoured. Setuid, setgid, and sticky bits from an
/// archive entry are discarded so a release archive can never grant elevated privileges to an extracted file.
#[cfg(unix)]
fn set_unix_mode(path: &Path, mode: u32) -> Result<(), EngineError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o777)).map_err(archive_entry_error(path))
}

#[cfg(windows)]
fn set_unix_mode(_path: &Path, _mode: u32) -> Result<(), EngineError> {
    Ok(())
}

#[cfg(test)]
#[path = "archive_test.rs"]
mod tests;
