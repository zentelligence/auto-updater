use std::ffi::OsString;

use tempfile::tempdir;

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const LABEL: &str = "Demo";

#[test]
fn a_launcher_is_copied_where_nothing_is_there() -> TestResult {
    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"launcher")?;
    let target = folder.path().join("contextos");

    assert_eq!(place_launcher(&launcher, &target, None)?, Placement::Placed);

    assert_eq!(fs::read(target)?, b"launcher");
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_copied_launcher_can_still_be_run() -> TestResult {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"launcher")?;
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755))?;
    let target = folder.path().join("contextos");

    place_launcher(&launcher, &target, None)?;

    assert_eq!(fs::metadata(target)?.permissions().mode() & 0o111, 0o111);
    Ok(())
}

#[test]
fn installing_again_finds_the_launcher_already_in_place() -> TestResult {
    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"launcher")?;
    let target = folder.path().join("contextos");
    fs::write(&target, b"launcher")?;

    assert_eq!(place_launcher(&launcher, &target, None)?, Placement::AlreadyInPlace);
    Ok(())
}

// Someone may already have a `contextos` of their own there, such as a build from source.
#[test]
fn a_different_file_is_left_alone() -> TestResult {
    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"launcher")?;
    let target = folder.path().join("contextos");
    fs::write(&target, b"something of the operator's")?;

    assert_eq!(place_launcher(&launcher, &target, None)?, Placement::SomethingElseThere);

    assert_eq!(fs::read(target)?, b"something of the operator's");
    Ok(())
}

#[test]
fn a_failed_copy_leaves_nothing_behind() -> TestResult {
    let folder = tempdir()?;
    let target = folder.path().join("contextos");

    assert!(place_launcher(&folder.path().join("missing"), &target, None).is_err());

    assert_eq!(fs::read_dir(folder.path())?.count(), 0);
    Ok(())
}

#[test]
fn a_folder_on_the_path_is_found() -> TestResult {
    let folder = tempdir()?;
    let other = tempdir()?;
    let path = std::env::join_paths([other.path(), folder.path()])?;

    assert!(path_holds(Some(&path), folder.path()));
    assert!(!path_holds(Some(&OsString::from(other.path())), folder.path()));
    assert!(!path_holds(None, folder.path()));
    Ok(())
}

#[test]
fn zsh_reads_its_own_profile() {
    let home = Path::new("/home/someone");

    assert_eq!(profile_files(home, Some(OsStr::new("/bin/zsh"))), [home.join(".zshrc")]);
}

#[test]
fn bash_login_shells_read_the_profile_when_there_is_no_bash_profile() -> TestResult {
    let home = tempdir()?;

    assert_eq!(
        profile_files(home.path(), Some(OsStr::new("/usr/bin/bash"))),
        [home.path().join(".bashrc"), home.path().join(".profile")]
    );
    Ok(())
}

// Bash reads only the first login profile it finds, so `.profile` would never be reached.
#[test]
fn bash_login_shells_read_an_existing_bash_profile_first() -> TestResult {
    let home = tempdir()?;
    fs::write(home.path().join(".bash_profile"), "")?;

    assert_eq!(
        profile_files(home.path(), Some(OsStr::new("/bin/bash"))),
        [home.path().join(".bashrc"), home.path().join(".bash_profile")]
    );
    Ok(())
}

#[test]
fn an_unknown_shell_has_no_profile_to_edit() {
    let home = Path::new("/home/someone");

    assert_eq!(
        profile_files(home, Some(OsStr::new("/usr/bin/fish"))),
        Vec::<PathBuf>::new()
    );
    assert_eq!(profile_files(home, None), Vec::<PathBuf>::new());
}

#[test]
fn a_profile_gains_the_folder_and_keeps_what_it_held() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");
    fs::write(&profile, "alias ll='ls -l'\n")?;

    assert!(add_to_profile(&profile, LABEL)?);

    let contents = fs::read_to_string(profile)?;
    assert!(contents.starts_with("alias ll='ls -l'\n"), "{contents}");
    assert!(
        contents.contains(r#"export PATH="$HOME/.local/bin:$PATH""#),
        "{contents}"
    );
    Ok(())
}

#[test]
fn a_missing_profile_is_created() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");

    assert!(add_to_profile(&profile, LABEL)?);

    assert!(profile.is_file());
    Ok(())
}

#[test]
fn a_profile_is_only_changed_once() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");

    add_to_profile(&profile, LABEL)?;
    let once = fs::read_to_string(&profile)?;

    assert!(!add_to_profile(&profile, LABEL)?);
    assert_eq!(fs::read_to_string(profile)?, once);
    Ok(())
}

// This is how a fix to the updater itself reaches an installation.
#[test]
fn a_launcher_placed_earlier_is_replaced() -> TestResult {
    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"new launcher")?;
    let target = folder.path().join("contextos");
    fs::write(&target, b"old launcher")?;
    let recorded = digest(&target)?;

    assert_eq!(place_launcher(&launcher, &target, Some(&recorded))?, Placement::Placed);

    assert_eq!(fs::read(target)?, b"new launcher");
    Ok(())
}

#[test]
fn a_file_changed_since_it_was_placed_is_left_alone() -> TestResult {
    let folder = tempdir()?;
    let launcher = folder.path().join("contextos-launcher");
    fs::write(&launcher, b"new launcher")?;
    let target = folder.path().join("contextos");
    fs::write(&target, b"old launcher")?;
    let recorded = digest(&target)?;
    fs::write(&target, b"something of the operator's")?;

    assert_eq!(
        place_launcher(&launcher, &target, Some(&recorded))?,
        Placement::SomethingElseThere
    );

    assert_eq!(fs::read(target)?, b"something of the operator's");
    Ok(())
}

#[test]
fn a_ledger_survives_being_saved_and_loaded() -> TestResult {
    let root = tempdir()?;
    let mut ledger = Ledger::default();
    ledger.record(PathBuf::from("/commands/contextos"), "abc".to_owned());
    ledger.record(PathBuf::from("/commands/contextos"), "def".to_owned());
    ledger.profiles.push(PathBuf::from("/home/someone/.zshrc"));

    ledger.save(root.path())?;

    let loaded = Ledger::load(root.path())?;
    assert_eq!(loaded, ledger);
    assert_eq!(loaded.launchers.len(), 1);
    assert_eq!(loaded.recorded_digest(Path::new("/commands/contextos")), Some("def"));
    Ok(())
}

#[test]
fn no_ledger_reads_as_nothing_placed() -> TestResult {
    let root = tempdir()?;

    assert_eq!(Ledger::load(root.path())?, Ledger::default());
    Ok(())
}

#[test]
fn a_ledger_from_an_unknown_schema_is_refused() -> TestResult {
    let root = tempdir()?;
    fs::write(
        root.path().join(LEDGER_FILE),
        r#"{"schema_version":2,"launchers":[],"profiles":[],"path_folder":null}"#,
    )?;

    assert!(Ledger::load(root.path()).is_err());
    Ok(())
}

#[test]
fn taking_the_path_line_back_leaves_the_profile_as_it_was() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");
    fs::write(&profile, "alias ll='ls -l'\n")?;
    add_to_profile(&profile, LABEL)?;

    remove_from_profile(&profile, LABEL)?;

    assert_eq!(fs::read_to_string(profile)?, "alias ll='ls -l'\n");
    Ok(())
}

#[test]
fn a_profile_that_adding_created_is_removed_again() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");
    add_to_profile(&profile, LABEL)?;

    remove_from_profile(&profile, LABEL)?;

    assert!(!profile.exists());
    Ok(())
}

#[test]
fn a_profile_edited_by_hand_is_left_alone() -> TestResult {
    let home = tempdir()?;
    let profile = home.path().join(".zshrc");
    fs::write(&profile, "# Added by Demo\nexport PATH=\"/mine:$PATH\"\n")?;

    remove_from_profile(&profile, LABEL)?;

    assert_eq!(
        fs::read_to_string(profile)?,
        "# Added by Demo\nexport PATH=\"/mine:$PATH\"\n"
    );
    Ok(())
}

#[test]
fn a_launcher_that_cannot_reach_the_first_folder_moves_to_the_next() -> TestResult {
    let directory = tempdir()?;
    let launcher = directory.path().join("demo");
    fs::write(&launcher, b"a launcher")?;
    let unreachable = directory.path().join("missing");
    let reachable = directory.path().join("reachable");
    fs::create_dir(&reachable)?;

    assert!(move_out_of_the_way(&launcher, &[unreachable, reachable.clone()]));

    assert!(!launcher.exists());
    assert_eq!(fs::read_dir(reachable)?.count(), 1);
    Ok(())
}

#[test]
fn a_launcher_with_nowhere_to_go_stays_where_it_is() -> TestResult {
    let directory = tempdir()?;
    let launcher = directory.path().join("demo");
    fs::write(&launcher, b"a launcher")?;

    assert!(!move_out_of_the_way(&launcher, &[directory.path().join("missing")]));

    assert!(launcher.exists());
    Ok(())
}

/// An installation of release 1.0.0 declaring `demo` and `demo-web`, with `demo-launcher` holding `launcher`.
fn installed(home: &Path, root: &Path, launcher: &[u8]) -> TestResult {
    let bin = root.join("versions").join("1.0.0").join("bin");
    fs::create_dir_all(&bin)?;
    fs::write(bin.join(executable_name("demo-launcher")), launcher)?;
    fs::write(
        root.join("versions").join("1.0.0").join("release.json"),
        r#"{
          "schema_version": 1,
          "product": "demo",
          "version": "1.0.0",
          "target": "x86_64-unknown-linux-gnu",
          "published_at": "2026-01-01T00:00:00Z",
          "minimum_launcher_version": "0.1.0",
          "commands": [
            { "name": "demo", "path": "bin/demo" },
            { "name": "demo-web", "path": "bin/demo-web" }
          ]
        }"#,
    )?;
    fs::create_dir_all(home)?;
    Ok(())
}

fn launchers<'a>(home: &'a Path, root: &'a Path) -> StableLaunchers<'a> {
    StableLaunchers {
        installation_root: root,
        home,
        launcher_name: "demo-launcher",
        product_label: LABEL,
    }
}

#[test]
fn every_declared_command_and_the_launcher_itself_are_placed() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let launchers = launchers(&home, &root);

    let left_alone = launchers.refresh(&ProductVersion::try_from("1.0.0")?)?;

    assert_eq!(left_alone, Vec::<PathBuf>::new());
    for name in ["demo", "demo-web", "demo-launcher"] {
        assert_eq!(fs::read(launchers.folder().join(executable_name(name)))?, b"launcher");
    }
    Ok(())
}

#[test]
fn a_newer_release_replaces_the_launchers_placed_by_an_older_one() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"old launcher")?;
    let launchers = launchers(&home, &root);
    let version = ProductVersion::try_from("1.0.0")?;
    launchers.refresh(&version)?;
    installed(&home, &root, b"new launcher")?;

    let left_alone = launchers.refresh(&version)?;

    assert_eq!(left_alone, Vec::<PathBuf>::new());
    assert_eq!(
        fs::read(launchers.folder().join(executable_name("demo")))?,
        b"new launcher"
    );
    Ok(())
}

#[test]
fn a_file_of_the_operators_with_a_commands_name_is_reported() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let launchers = launchers(&home, &root);
    fs::create_dir_all(launchers.folder())?;
    let theirs = launchers.folder().join(executable_name("demo"));
    fs::write(&theirs, b"something of the operator's")?;

    let left_alone = launchers.refresh(&ProductVersion::try_from("1.0.0")?)?;

    assert_eq!(left_alone, std::slice::from_ref(&theirs));
    assert_eq!(fs::read(theirs)?, b"something of the operator's");
    Ok(())
}

#[cfg(not(windows))]
#[test]
fn the_folder_goes_on_the_path_through_the_shells_profile_and_comes_off_again() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let launchers = launchers(&home, &root);
    let zshrc = home.join(".zshrc");
    fs::write(&zshrc, "alias ll='ls -l'\n")?;

    let change = launchers.add_to_path(Some(OsStr::new("/usr/bin")), Some(OsStr::new("/bin/zsh")))?;

    assert_eq!(change, PathChange::AddedToProfiles(vec![zshrc.clone()]));
    assert!(fs::read_to_string(&zshrc)?.contains("# Added by Demo"));

    assert_eq!(launchers.remove()?, LauncherRemoval::default());
    assert_eq!(fs::read_to_string(zshrc)?, "alias ll='ls -l'\n");
    Ok(())
}

#[cfg(not(windows))]
#[test]
fn a_folder_already_on_the_path_changes_nothing() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let launchers = launchers(&home, &root);
    let path = std::env::join_paths([launchers.folder()])?;

    let change = launchers.add_to_path(Some(&path), Some(OsStr::new("/bin/zsh")))?;

    assert_eq!(change, PathChange::AlreadyOnPath);
    assert!(!home.join(".zshrc").exists());
    Ok(())
}

#[cfg(not(windows))]
#[test]
fn a_shell_it_cannot_edit_is_left_for_the_operator() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;

    let change = launchers(&home, &root).add_to_path(None, Some(OsStr::new("/usr/bin/fish")))?;

    assert_eq!(change, PathChange::NeedsAddingByHand);
    Ok(())
}

#[test]
fn removal_takes_away_only_what_is_still_as_it_was_placed() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let launchers = launchers(&home, &root);
    launchers.refresh(&ProductVersion::try_from("1.0.0")?)?;
    let changed = launchers.folder().join(executable_name("demo"));
    fs::write(&changed, b"something of the operator's")?;

    let removal = launchers.remove()?;

    assert_eq!(removal.changed, std::slice::from_ref(&changed));
    assert_eq!(fs::read(changed)?, b"something of the operator's");
    assert!(!launchers.folder().join(executable_name("demo-web")).exists());
    Ok(())
}

// The ledger sits in a folder anything running as the operator can write to, so it must not be a way to have a file
// elsewhere deleted, however good its digest.
#[test]
fn a_ledger_entry_outside_the_launcher_folder_is_refused() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let elsewhere = directory.path().join("important");
    fs::write(&elsewhere, b"important")?;
    let mut ledger = Ledger::default();
    ledger.record(elsewhere.clone(), digest(&elsewhere)?);
    ledger.save(&root)?;

    let removal = launchers(&home, &root).remove()?;

    assert_eq!(removal.failed, std::slice::from_ref(&elsewhere));
    assert!(elsewhere.exists());
    Ok(())
}

#[test]
fn a_ledger_profile_that_is_not_a_shell_profile_is_refused() -> TestResult {
    let directory = tempdir()?;
    let (home, root) = (directory.path().join("home"), directory.path().join("root"));
    installed(&home, &root, b"launcher")?;
    let elsewhere = home.join("notes.txt");
    fs::write(&elsewhere, profile_block(LABEL))?;
    let mut ledger = Ledger::default();
    ledger.profiles.push(elsewhere.clone());
    ledger.save(&root)?;

    let removal = launchers(&home, &root).remove()?;

    assert_eq!(removal.failed, std::slice::from_ref(&elsewhere));
    assert_eq!(fs::read_to_string(elsewhere)?, profile_block(LABEL));
    Ok(())
}
