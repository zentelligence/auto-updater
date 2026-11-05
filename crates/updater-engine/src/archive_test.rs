use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use tempfile::tempdir;
use updater_core::CoreError;

use super::*;

fn build_tar_gz(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (name, contents) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder.append_data(&mut header, name, *contents).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap();
}

fn build_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, contents) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(contents).unwrap();
    }
    writer.finish().unwrap();
}

#[test]
fn extracts_regular_files_from_a_tar_gz_archive() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.tar.gz");
    build_tar_gz(
        &archive_path,
        &[("bin/tool", b"binary contents"), ("release.json", b"{}")],
    );
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    extract_archive(&archive_path, "release.tar.gz", &staging).expect("valid archive");

    assert_eq!(fs::read(staging.join("bin/tool")).unwrap(), b"binary contents");
    assert_eq!(fs::read(staging.join("release.json")).unwrap(), b"{}");
}

/// Writes a tar entry with a raw, unvalidated name, bypassing `Header::set_path`'s own traversal
/// rejection. This is what proves our own reader-side confinement check, not just the writer's
/// safety, rejects a malicious archive: a real attacker is not bound by this crate's writer API.
fn append_raw_named_entry(builder: &mut tar::Builder<GzEncoder<File>>, raw_name: &[u8], contents: &[u8]) {
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    {
        let old = header.as_old_mut();
        old.name.fill(0);
        old.name[..raw_name.len()].copy_from_slice(raw_name);
    }
    header.set_cksum();
    builder.append(&header, contents).unwrap();
}

#[test]
fn rejects_a_tar_gz_entry_that_escapes_the_staging_directory() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.tar.gz");
    let file = File::create(&archive_path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    append_raw_named_entry(&mut builder, b"../evil.txt", b"escape");
    builder.into_inner().unwrap().finish().unwrap();
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    let error = extract_archive(&archive_path, "release.tar.gz", &staging).unwrap_err();

    assert!(matches!(error, EngineError::Core(CoreError::PathTraversal { .. })));
}

#[test]
fn extracts_regular_files_from_a_zip_archive() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.zip");
    build_zip(
        &archive_path,
        &[("bin/tool", b"binary contents"), ("release.json", b"{}")],
    );
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    extract_archive(&archive_path, "release.zip", &staging).expect("valid archive");

    assert_eq!(fs::read(staging.join("bin/tool")).unwrap(), b"binary contents");
    assert_eq!(fs::read(staging.join("release.json")).unwrap(), b"{}");
}

#[test]
fn rejects_a_zip_entry_that_escapes_the_staging_directory() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.zip");
    build_zip(&archive_path, &[("../evil.txt", b"escape")]);
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    let error = extract_archive(&archive_path, "release.zip", &staging).unwrap_err();

    assert!(matches!(error, EngineError::Core(CoreError::PathTraversal { .. })));
}

#[test]
fn rejects_an_unsupported_archive_extension() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.rar");
    fs::write(&archive_path, b"not an archive").unwrap();
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    let error = extract_archive(&archive_path, "release.rar", &staging).unwrap_err();

    assert!(matches!(error, EngineError::UnsupportedArchiveFormat { .. }));
}

#[cfg(unix)]
#[test]
fn preserves_the_executable_permission_bit_from_a_tar_gz_archive() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.tar.gz");
    build_tar_gz(&archive_path, &[("bin/tool", b"binary contents")]);
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    extract_archive(&archive_path, "release.tar.gz", &staging).unwrap();

    let mode = fs::metadata(staging.join("bin/tool")).unwrap().permissions().mode();
    assert_eq!(mode & 0o111, 0o111);
}

#[cfg(unix)]
#[test]
fn strips_setuid_setgid_and_sticky_bits_from_a_tar_gz_entry() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.tar.gz");
    let file = File::create(&archive_path).unwrap();
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));
    let contents: &[u8] = b"binary contents";
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o7755);
    header.set_cksum();
    builder.append_data(&mut header, "bin/tool", contents).unwrap();
    builder.into_inner().unwrap().finish().unwrap();
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    extract_archive(&archive_path, "release.tar.gz", &staging).unwrap();

    let mode = fs::metadata(staging.join("bin/tool")).unwrap().permissions().mode();
    assert_eq!(mode & 0o7000, 0, "special permission bits must not survive extraction");
    assert_eq!(mode & 0o111, 0o111, "the executable bits must still be preserved");
}

#[cfg(unix)]
#[test]
fn strips_setuid_setgid_and_sticky_bits_from_a_zip_entry() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.zip");
    let mut writer = zip::ZipWriter::new(File::create(&archive_path).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o7755);
    writer.start_file("bin/tool", options).unwrap();
    writer.write_all(b"binary contents").unwrap();
    writer.finish().unwrap();
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    extract_archive(&archive_path, "release.zip", &staging).unwrap();

    let mode = fs::metadata(staging.join("bin/tool")).unwrap().permissions().mode();
    assert_eq!(mode & 0o7000, 0, "special permission bits must not survive extraction");
    assert_eq!(mode & 0o111, 0o111, "the executable bits must still be preserved");
}

#[test]
fn rejects_a_zip_symlink_entry_without_writing_it() {
    let temp = tempdir().unwrap();
    let archive_path = temp.path().join("release.zip");
    let mut writer = zip::ZipWriter::new(File::create(&archive_path).unwrap());
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    writer.add_symlink("bin/tool", "/etc/passwd", options).unwrap();
    writer.finish().unwrap();
    let staging = temp.path().join("staging");
    fs::create_dir_all(&staging).unwrap();

    let error = extract_archive(&archive_path, "release.zip", &staging).unwrap_err();

    assert!(matches!(error, EngineError::ArchiveEntryUnreadable { .. }));
    assert!(fs::symlink_metadata(staging.join("bin/tool")).is_err());
}
