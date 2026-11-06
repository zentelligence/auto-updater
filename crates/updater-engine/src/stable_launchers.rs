//! Stable launchers: puts a product's public commands on the operator's `PATH` and takes them off it again. The
//! launcher is copied under each declared command's name and under its own, and its folder is added to `PATH`, because
//! the people installing a product should never need to know what an environment variable is.
//!
//! Everything placed outside the installation root is recorded in a ledger inside it. That is how a later update knows
//! an older launcher from a file of the operator's with the same name, and how a removal touches only what was added.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use updater_core::{AtomicWriter, ProductVersion, ReleaseMetadata};

use crate::atomic_write::FilesystemAtomicWriter;
use crate::checksum::compute_sha256;
use crate::error::EngineError;

/// The ledger's file name in the installation root.
const LEDGER_FILE: &str = "launchers.json";
const LEDGER_SCHEMA_VERSION: u32 = 1;
/// The shell profiles this module may edit, relative to the home folder.
const PROFILE_NAMES: [&str; 4] = [".zshrc", ".bashrc", ".bash_profile", ".profile"];

/// One launcher copy, with the digest it had when placed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PlacedLauncher {
    path: PathBuf,
    sha256: String,
}

/// What an install added outside the installation root.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    schema_version: u32,
    launchers: Vec<PlacedLauncher>,
    /// Shell profiles that gained the `PATH` line.
    profiles: Vec<PathBuf>,
    /// The folder added to the user's `Path` on Windows.
    path_folder: Option<PathBuf>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            schema_version: LEDGER_SCHEMA_VERSION,
            launchers: Vec::new(),
            profiles: Vec::new(),
            path_folder: None,
        }
    }
}

impl Ledger {
    /// The ledger in `root`, or an empty one when none has been written.
    fn load(root: &Path) -> Result<Self, EngineError> {
        let path = root.join(LEDGER_FILE);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(EngineError::Io { path, source }),
        };
        let ledger: Self = serde_json::from_slice(&bytes).map_err(|source| EngineError::InvalidLauncherLedger {
            path: path.clone(),
            reason: source.to_string(),
        })?;
        if ledger.schema_version != LEDGER_SCHEMA_VERSION {
            return Err(EngineError::InvalidLauncherLedger {
                path,
                reason: format!("unknown schema version {}", ledger.schema_version),
            });
        }
        Ok(ledger)
    }

    /// Writes the ledger to `root`, whole or not at all.
    fn save(&self, root: &Path) -> Result<(), EngineError> {
        let path = root.join(LEDGER_FILE);
        let bytes = serde_json::to_vec_pretty(self).map_err(|source| EngineError::InvalidLauncherLedger {
            path: path.clone(),
            reason: source.to_string(),
        })?;
        FilesystemAtomicWriter.write_atomic(&path, &bytes)?;
        Ok(())
    }

    fn recorded_digest(&self, target: &Path) -> Option<&str> {
        self.launchers
            .iter()
            .find(|placed| placed.path == target)
            .map(|placed| placed.sha256.as_str())
    }

    fn record(&mut self, path: PathBuf, sha256: String) {
        self.launchers.retain(|placed| placed.path != path);
        self.launchers.push(PlacedLauncher { path, sha256 });
    }
}

/// What became of one public command's launcher.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Placement {
    /// The launcher was copied there, over nothing or over a launcher placed earlier.
    Placed,
    /// The same launcher was already there.
    AlreadyInPlace,
    /// A file that was never placed here, which is the operator's and is left alone.
    SomethingElseThere,
}

/// How the launcher folder came to be on `PATH`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathChange {
    /// It already was.
    AlreadyOnPath,
    /// These shell profiles now add it, for terminals opened from now on.
    AddedToProfiles(Vec<PathBuf>),
    /// The Windows user `Path` now holds it.
    AddedToUserPath,
    /// The shell's profile is not one this module can edit, so the operator has to add it.
    NeedsAddingByHand,
}

/// What a removal could not do. Nothing here stops the rest of a removal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LauncherRemoval {
    /// Launcher copies that have changed since they were placed, which are left as they are.
    pub changed: Vec<PathBuf>,
    /// Files that could not be removed or changed.
    pub failed: Vec<PathBuf>,
}

/// One product's stable launchers. Every field is supplied by the embedding product.
#[derive(Clone, Copy, Debug)]
pub struct StableLaunchers<'a> {
    pub installation_root: &'a Path,
    pub home: &'a Path,
    /// The launcher's own command name, which it also goes on `PATH` under.
    pub launcher_name: &'a str,
    /// The product's name as people read it, used to mark what is added to a shell profile.
    pub product_label: &'a str,
}

impl StableLaunchers<'_> {
    /// Where the launchers go: a folder the operator owns, so no administrator rights are needed.
    #[must_use]
    pub fn folder(&self) -> PathBuf {
        if cfg!(windows) {
            self.installation_root.join("bin")
        } else {
            self.home.join(".local").join("bin")
        }
    }

    /// Brings a launcher for every command the release set `version` declares, and for the launcher's own name, up to
    /// that release's launcher, returning the paths left alone because they are not launchers placed here.
    ///
    /// # Errors
    ///
    /// [`EngineError`] when the release's metadata or launcher cannot be read, a launcher cannot be placed, or the
    /// ledger cannot be read or written.
    pub fn refresh(&self, version: &ProductVersion) -> Result<Vec<PathBuf>, EngineError> {
        let folder = self.folder();
        fs::create_dir_all(&folder).map_err(|source| io_error(&folder, source))?;
        let release = self.installation_root.join("versions").join(version.to_string());
        let metadata_path = release.join("release.json");
        let metadata_bytes = fs::read(&metadata_path).map_err(|source| io_error(&metadata_path, source))?;
        let metadata = ReleaseMetadata::try_from(metadata_bytes.as_slice())?;
        let launcher = release.join("bin").join(executable_name(self.launcher_name));
        let launcher_digest = digest(&launcher)?;

        let mut names: Vec<&str> = metadata.commands.iter().map(|command| command.name.as_str()).collect();
        names.push(self.launcher_name);

        let mut ledger = Ledger::load(self.installation_root)?;
        let mut left_alone = Vec::new();
        for name in names {
            let target = folder.join(executable_name(name));
            match place_launcher(&launcher, &target, ledger.recorded_digest(&target))? {
                Placement::SomethingElseThere => left_alone.push(target),
                Placement::Placed | Placement::AlreadyInPlace => ledger.record(target, launcher_digest.clone()),
            }
        }
        ledger.save(self.installation_root)?;
        remove_set_aside(&folder);
        Ok(left_alone)
    }

    /// Puts the launcher folder on `PATH`, given the value of `PATH` and of `SHELL`.
    ///
    /// # Errors
    ///
    /// [`EngineError`] when a profile or the Windows user `Path` cannot be changed, or the ledger cannot be read or
    /// written.
    pub fn add_to_path(&self, path: Option<&OsStr>, shell: Option<&OsStr>) -> Result<PathChange, EngineError> {
        let folder = self.folder();
        let mut ledger = Ledger::load(self.installation_root)?;
        let change = self.add_folder(&folder, path, shell, &mut ledger)?;
        ledger.save(self.installation_root)?;
        Ok(change)
    }

    #[cfg(not(windows))]
    fn add_folder(
        &self,
        folder: &Path,
        path: Option<&OsStr>,
        shell: Option<&OsStr>,
        ledger: &mut Ledger,
    ) -> Result<PathChange, EngineError> {
        if path_holds(path, folder) {
            return Ok(PathChange::AlreadyOnPath);
        }
        let profiles = profile_files(self.home, shell);
        if profiles.is_empty() {
            return Ok(PathChange::NeedsAddingByHand);
        }
        for profile in &profiles {
            if add_to_profile(profile, self.product_label)? && !ledger.profiles.contains(profile) {
                ledger.profiles.push(profile.clone());
            }
        }
        Ok(PathChange::AddedToProfiles(profiles))
    }

    #[cfg(windows)]
    fn add_folder(
        &self,
        folder: &Path,
        _path: Option<&OsStr>,
        _shell: Option<&OsStr>,
        ledger: &mut Ledger,
    ) -> Result<PathChange, EngineError> {
        windows_path::add(folder)?;
        ledger.path_folder = Some(folder.to_path_buf());
        Ok(PathChange::AddedToUserPath)
    }

    /// Removes the launcher copies and the `PATH` changes the ledger records. The ledger lives in a folder anything
    /// running as the operator can write to, so an entry is only acted on if it is where this module would have put it.
    ///
    /// # Errors
    ///
    /// [`EngineError`] when the ledger cannot be read. What cannot be removed after that is reported in the result.
    pub fn remove(&self) -> Result<LauncherRemoval, EngineError> {
        let ledger = Ledger::load(self.installation_root)?;
        let folder = self.folder();
        let mut removal = LauncherRemoval::default();
        for placed in &ledger.launchers {
            if placed.path.parent() == Some(folder.as_path()) {
                self.remove_launcher(placed, &mut removal);
            } else {
                removal.failed.push(placed.path.clone());
            }
        }
        for profile in &ledger.profiles {
            let known = PROFILE_NAMES.iter().any(|name| *profile == self.home.join(name));
            if !known || remove_from_profile(profile, self.product_label).is_err() {
                removal.failed.push(profile.clone());
            }
        }
        #[cfg(windows)]
        if let Some(recorded) = &ledger.path_folder
            && (*recorded != folder || windows_path::remove(recorded).is_err())
        {
            removal.failed.push(recorded.clone());
        }
        Ok(removal)
    }

    fn remove_launcher(&self, placed: &PlacedLauncher, removal: &mut LauncherRemoval) {
        if !placed.path.exists() {
            return;
        }
        if digest(&placed.path).ok().as_deref() != Some(placed.sha256.as_str()) {
            removal.changed.push(placed.path.clone());
            return;
        }
        if fs::remove_file(&placed.path).is_ok() {
            return;
        }
        let mut folders = vec![std::env::temp_dir()];
        folders.extend(self.installation_root.parent().map(Path::to_path_buf));
        if !move_out_of_the_way(&placed.path, &folders) {
            removal.failed.push(placed.path.clone());
        }
    }
}

fn io_error(path: &Path, source: io::Error) -> EngineError {
    EngineError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn executable_name(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

/// The SHA-256 of the file at `path`, as lowercase hex.
fn digest(path: &Path) -> Result<String, EngineError> {
    let file = fs::File::open(path).map_err(|source| io_error(path, source))?;
    Ok(compute_sha256(file, path)?.to_string())
}

/// Copies `launcher` to `target`, over an earlier launcher whose digest is `recorded` but over nothing else.
fn place_launcher(launcher: &Path, target: &Path, recorded: Option<&str>) -> Result<Placement, EngineError> {
    if target.exists() {
        let existing = fs::read(target).map_err(|source| io_error(target, source))?;
        let wanted = fs::read(launcher).map_err(|source| io_error(launcher, source))?;
        if existing == wanted {
            return Ok(Placement::AlreadyInPlace);
        }
        if recorded != Some(digest(target)?.as_str()) {
            return Ok(Placement::SomethingElseThere);
        }
    }
    let staged = target.with_extension("installing");
    fs::copy(launcher, &staged)
        .and_then(|_| swap_in(&staged, target))
        .map_err(|source| {
            let _ = fs::remove_file(&staged);
            io_error(target, source)
        })?;
    Ok(Placement::Placed)
}

/// Replaces `target` with `staged`. Renaming over a file works while the file is running on Unix.
#[cfg(not(windows))]
fn swap_in(staged: &Path, target: &Path) -> io::Result<()> {
    fs::rename(staged, target)
}

/// Replaces `target` with `staged`. Windows refuses to overwrite a running program, which this launcher may well be,
/// but lets it be renamed, so it is moved aside and deleted by a later run.
#[cfg(windows)]
fn swap_in(staged: &Path, target: &Path) -> io::Result<()> {
    if target.exists() {
        fs::rename(target, target.with_extension(format!("old-{}", std::process::id())))?;
    }
    fs::rename(staged, target)
}

/// Deletes launchers moved aside by earlier runs, leaving any still running.
fn remove_set_aside(folder: &Path) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let set_aside = path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.starts_with("old-"));
        if set_aside {
            let _ = fs::remove_file(path);
        }
    }
}

/// Windows will not delete a running program, which this launcher may be, but lets it be renamed: moved out of the
/// folder it was in, it no longer holds up that folder's removal. A rename cannot cross drives, so each of `folders` is
/// tried in turn, returning whether one worked.
fn move_out_of_the_way(path: &Path, folders: &[PathBuf]) -> bool {
    let name = format!("launcher-{}.old", std::process::id());
    folders
        .iter()
        .any(|folder| fs::rename(path, folder.join(&name)).is_ok())
}

/// Whether `folder` is one of the folders in the `PATH` value `path`.
#[cfg(any(not(windows), test))]
fn path_holds(path: Option<&OsStr>, folder: &Path) -> bool {
    path.is_some_and(|path| std::env::split_paths(path).any(|entry| entry == folder))
}

/// The profiles that `shell` reads, under `home`. Empty for a shell whose profile this does not know how to edit.
#[cfg(any(not(windows), test))]
fn profile_files(home: &Path, shell: Option<&OsStr>) -> Vec<PathBuf> {
    match shell.map(Path::new).and_then(Path::file_name).and_then(OsStr::to_str) {
        Some("zsh") => vec![home.join(".zshrc")],
        Some("bash") => {
            let bash_profile = home.join(".bash_profile");
            let login_profile = if bash_profile.exists() {
                bash_profile
            } else {
                home.join(".profile")
            };
            vec![home.join(".bashrc"), login_profile]
        }
        _ => Vec::new(),
    }
}

fn profile_marker(product_label: &str) -> String {
    format!("# Added by {product_label}")
}

fn profile_block(product_label: &str) -> String {
    format!(
        "\n{}\nexport PATH=\"$HOME/.local/bin:$PATH\"\n",
        profile_marker(product_label)
    )
}

/// Adds `~/.local/bin` to `PATH` in `profile`, once, returning whether it changed the file.
#[cfg(any(not(windows), test))]
fn add_to_profile(profile: &Path, product_label: &str) -> Result<bool, EngineError> {
    use std::io::Write;

    let existing = match fs::read_to_string(profile) {
        Ok(existing) => existing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(io_error(profile, source)),
    };
    if existing.contains(&profile_marker(product_label)) {
        return Ok(false);
    }
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(profile)
        .and_then(|mut file| file.write_all(profile_block(product_label).as_bytes()))
        .map_err(|source| io_error(profile, source))?;
    Ok(true)
}

/// Takes back what [`add_to_profile`] added, deleting a profile that then holds nothing, as it would if adding created
/// it. Anything else in the profile stays exactly as it is.
fn remove_from_profile(profile: &Path, product_label: &str) -> io::Result<()> {
    let existing = match fs::read_to_string(profile) {
        Ok(existing) => existing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let remaining = existing.replacen(&profile_block(product_label), "", 1);
    if remaining == existing {
        return Ok(());
    }
    if remaining.is_empty() {
        fs::remove_file(profile)
    } else {
        fs::write(profile, remaining)
    }
}

/// The user `Path` in the registry, changed through the one call that also tells running programs.
#[cfg(windows)]
mod windows_path {
    use std::path::Path;

    use crate::error::EngineError;

    const ADD: &str = "$folder = $env:UPDATER_LAUNCHER_FOLDER; \
        $path = [Environment]::GetEnvironmentVariable('Path', 'User'); \
        if (\"$path\" -split ';' -notcontains $folder) { \
        $entries = @(\"$path\".TrimEnd(';'), $folder) | Where-Object { $_ }; \
        [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User') }";
    const REMOVE: &str = "$folder = $env:UPDATER_LAUNCHER_FOLDER; \
        $path = [Environment]::GetEnvironmentVariable('Path', 'User'); \
        $entries = \"$path\" -split ';' | Where-Object { $_ -and $_ -ne $folder }; \
        [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')";

    pub(super) fn add(folder: &Path) -> Result<(), EngineError> {
        run(ADD, folder)
    }

    pub(super) fn remove(folder: &Path) -> Result<(), EngineError> {
        run(REMOVE, folder)
    }

    /// Windows PowerShell, by its full path so no same-named program can stand in for it.
    fn run(script: &str, folder: &Path) -> Result<(), EngineError> {
        let failed = |source| super::io_error(folder, source);
        let system_root =
            std::env::var_os("SystemRoot").ok_or_else(|| failed(std::io::Error::other("SystemRoot is not set")))?;
        let powershell = Path::new(&system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        let status = std::process::Command::new(powershell)
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("UPDATER_LAUNCHER_FOLDER", folder)
            .status()
            .map_err(failed)?;
        if status.success() {
            Ok(())
        } else {
            Err(failed(std::io::Error::other("the user Path could not be changed")))
        }
    }
}

#[cfg(test)]
#[path = "stable_launchers_test.rs"]
mod tests;
