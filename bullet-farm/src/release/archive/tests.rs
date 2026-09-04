use std::{
    fs::{self, File},
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use super::*;

#[test]
fn plan_rejects_hostile_names_collisions_and_missing_layout() {
    for path in [
        "/bullet-farm/bin/tool",
        "bullet-farm/./tool",
        "bullet-farm/../tool",
        "bullet-farm/bin\\tool",
        "bullet-farm/.GIT/config",
        "bullet-farm/bin/tool:stream",
        "bullet-farm/bin/tool.",
        "bullet-farm/bin/tool ",
        "bullet-farm/bin/caf\u{e9}",
        "bullet-farm/bin/cafe\u{301}",
        "bullet-farm/CON.txt",
        "another-root/bin/tool",
    ] {
        let error = ArchivePlan::admit(
            vec![directory("bullet-farm"), file(path, 1)],
            "x86_64-unknown-linux-gnu",
            1024,
        )
        .unwrap_err();
        assert_eq!(error.code(), "INVALID_RELEASE_ARCHIVE", "{path}");
    }

    let case_collision = vec![
        directory("bullet-farm"),
        directory("bullet-farm/BIN"),
        directory("bullet-farm/bin"),
    ];
    assert_eq!(
        ArchivePlan::admit(case_collision, "x86_64-unknown-linux-gnu", 1024)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let prefix_conflict = vec![
        directory("bullet-farm"),
        file("bullet-farm/bin", 1),
        file("bullet-farm/bin/bullet-family", 1),
    ];
    assert_eq!(
        ArchivePlan::admit(prefix_conflict, "x86_64-unknown-linux-gnu", 1024)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_ARCHIVE"
    );
}

#[test]
fn plan_enforces_target_executable_and_decompression_budget() {
    let unix = minimal_entries("");
    ArchivePlan::admit(unix.clone(), "x86_64-unknown-linux-gnu", 1024)
        .expect("Unix executable layout");
    assert_eq!(
        ArchivePlan::admit(unix, "x86_64-pc-windows-msvc", 1024)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    for omitted in PACKAGED_BINARY_NAMES {
        let mut missing = minimal_entries("");
        missing.retain(|entry| entry.name != format!("bullet-farm/bin/{omitted}").as_bytes());
        assert_eq!(
            ArchivePlan::admit(missing, "x86_64-unknown-linux-gnu", 1024)
                .unwrap_err()
                .code(),
            "INVALID_RELEASE_ARCHIVE",
            "omitting {omitted} must fail"
        );
    }

    let mut unexpected = minimal_entries("");
    unexpected.push(file("bullet-farm/bin/unreviewed-tool", 1));
    unexpected.sort_by(|left, right| left.name.cmp(&right.name));
    assert_eq!(
        ArchivePlan::admit(unexpected, "x86_64-unknown-linux-gnu", 1024)
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let oversized = vec![
        directory("bullet-farm"),
        directory("bullet-farm/bin"),
        file("bullet-farm/bin/bullet-family", MAX_ENTRY_BYTES + 1),
    ];
    assert_eq!(
        ArchivePlan::admit(oversized, "x86_64-unknown-linux-gnu", 1024)
            .unwrap_err()
            .code(),
        "RELEASE_ARCHIVE_LIMIT_EXCEEDED"
    );
}

#[test]
fn raw_tar_refuses_pax_and_links_before_they_can_hide_metadata() {
    let pax = tar_bytes(true, None);
    assert_eq!(
        tar_zst::scan(file_with(&pax)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let link = tar_bytes(false, Some(tar::EntryType::Symlink));
    assert_eq!(
        tar_zst::scan(file_with(&link)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );
}

#[test]
fn tar_bounds_decompression_including_zero_padding() {
    let zeros = vec![0_u8; (MIN_RATIO_ALLOWANCE + 1) as usize];
    let compressed = zstd::stream::encode_all(Cursor::new(zeros), 19).expect("compress zero bomb");
    assert_eq!(
        tar_zst::scan(file_with(&compressed)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );
}

#[test]
fn valid_tar_and_stored_zip_materialize_deterministically() {
    let temp = tempfile::tempdir().expect("temporary output");
    for (name, bytes, scan, materialize) in [
        (
            "tar",
            tar_bytes(false, None),
            tar_zst::scan as fn(File) -> Result<Vec<RawEntry>, CoordError>,
            tar_zst::materialize as fn(File, &ArchivePlan, &Path) -> Result<(), CoordError>,
        ),
        ("zip", zip_bytes(false), zip::scan, zip::materialize),
    ] {
        let plan = ArchivePlan::admit(
            scan(file_with(&bytes)).expect("scan valid archive"),
            "x86_64-unknown-linux-gnu",
            bytes.len() as u64,
        )
        .expect("admit valid archive");
        let output = temp.path().join(name);
        fs::create_dir(&output).expect("format output");
        materialize(file_with(&bytes), &plan, &output).expect("materialize archive");
        plan.sync_tree(&output).expect("sync extracted tree");
        assert_eq!(
            fs::read(output.join("bullet-farm/bin/bullet-family")).expect("binary"),
            b"fixture-binary\n"
        );
        for executable in PACKAGED_BINARY_NAMES {
            use std::os::unix::fs::PermissionsExt;

            let path = output.join(format!("bullet-farm/bin/{executable}"));
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o755,
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn zip_refuses_symlinks_suffixes_and_ambiguous_raw_headers() {
    assert_eq!(
        zip::scan(file_with(&zip_bytes(true))).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );
    let mut suffixed = zip_bytes(false);
    suffixed.push(0);
    assert_eq!(
        zip::scan(file_with(&suffixed)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let mut multi_disk = zip_bytes(false);
    let eocd = multi_disk.len() - 22;
    multi_disk[eocd + 4] = 1;
    assert_eq!(
        zip::scan(file_with(&multi_disk)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let mut zip64 = zip_bytes(false);
    let eocd = zip64.len() - 22;
    zip64[eocd + 8..eocd + 10].copy_from_slice(&u16::MAX.to_le_bytes());
    assert_eq!(
        zip::scan(file_with(&zip64)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let mut descriptor = zip_bytes(false);
    descriptor[6] |= 0x08;
    let central = find_bytes(&descriptor, b"PK\x01\x02");
    descriptor[central + 8] |= 0x08;
    assert_eq!(
        zip::scan(file_with(&descriptor)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );

    let mut corrupt = zip_bytes(false);
    let payload = find_bytes(&corrupt, b"fixture-binary\n");
    corrupt[payload] ^= 1;
    assert_eq!(
        zip::scan(file_with(&corrupt)).unwrap_err().code(),
        "INVALID_RELEASE_ARCHIVE"
    );
}

#[test]
fn publication_is_no_replace_and_reports_post_rename_unknown() {
    let temp = tempfile::tempdir().expect("publication root");
    let destination = temp.path().join("installed");
    let admitted = publish::admit_destination(&destination).expect("admit absent destination");
    let source = stage_tree(temp.path(), "stage-one");
    publish::publish_no_replace(&source, &admitted).expect("publish tree");
    assert_eq!(fs::read(destination.join("marker")).unwrap(), b"complete");
    assert_eq!(
        publish::admit_destination(&destination).unwrap_err().code(),
        "RELEASE_DESTINATION_EXISTS"
    );

    let unknown_destination = temp.path().join("unknown");
    let admitted = publish::admit_destination(&unknown_destination).expect("admit second target");
    let source = stage_tree(temp.path(), "stage-two");
    assert_eq!(
        publish::publish_with_failed_sync_for_test(&source, &admitted)
            .unwrap_err()
            .code(),
        "RELEASE_PUBLICATION_UNKNOWN"
    );
    assert_eq!(
        fs::read(unknown_destination.join("marker")).unwrap(),
        b"complete"
    );
}

#[test]
fn destination_refuses_symlinked_components() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("destination root");
    let real = temp.path().join("real");
    let alias = temp.path().join("alias");
    fs::create_dir(&real).expect("real parent");
    symlink(&real, &alias).expect("symlinked parent");
    assert_eq!(
        publish::admit_destination(&alias.join("installed"))
            .unwrap_err()
            .code(),
        "INVALID_RELEASE_DESTINATION"
    );
}

fn minimal_entries(suffix: &str) -> Vec<RawEntry> {
    let mut entries = vec![directory("bullet-farm"), directory("bullet-farm/bin")];
    entries.extend(
        PACKAGED_BINARY_NAMES.map(|name| file(&format!("bullet-farm/bin/{name}{suffix}"), 1)),
    );
    entries
}

fn directory(path: &str) -> RawEntry {
    RawEntry {
        name: path.as_bytes().to_vec(),
        kind: EntryKind::Directory,
        size: 0,
    }
}

fn file(path: &str, size: u64) -> RawEntry {
    RawEntry {
        name: path.as_bytes().to_vec(),
        kind: EntryKind::File,
        size,
    }
}

fn tar_bytes(pax: bool, special: Option<tar::EntryType>) -> Vec<u8> {
    let encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).expect("zstd encoder");
    let mut builder = tar::Builder::new(encoder);
    if pax {
        builder
            .append_pax_extensions([("comment", b"hidden".as_slice())])
            .expect("PAX extension");
    }
    append_tar_directory(&mut builder, "bullet-farm");
    append_tar_directory(&mut builder, "bullet-farm/bin");
    if let Some(entry_type) = special {
        let mut header = tar::Header::new_ustar();
        header.set_entry_type(entry_type);
        header.set_path("bullet-farm/bin/link").unwrap();
        header.set_size(0);
        header.set_mode(0o777);
        header.set_cksum();
        builder.append(&header, std::io::empty()).unwrap();
    } else {
        for executable in PACKAGED_BINARY_NAMES {
            append_tar_file(
                &mut builder,
                &format!("bullet-farm/bin/{executable}"),
                if executable == "bullet-family" {
                    b"fixture-binary\n"
                } else {
                    b"fixture-tool\n"
                },
            );
        }
    }
    builder.finish().expect("finish TAR");
    builder.into_inner().unwrap().finish().unwrap()
}

fn append_tar_directory(builder: &mut tar::Builder<zstd::Encoder<'static, Vec<u8>>>, path: &str) {
    let mut header = tar::Header::new_ustar();
    header.set_entry_type(tar::EntryType::Directory);
    header.set_path(path).unwrap();
    header.set_size(0);
    header.set_mode(0o755);
    header.set_cksum();
    builder.append(&header, std::io::empty()).unwrap();
}

fn append_tar_file(
    builder: &mut tar::Builder<zstd::Encoder<'static, Vec<u8>>>,
    path: &str,
    bytes: &[u8],
) {
    let mut header = tar::Header::new_ustar();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_path(path).unwrap();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder.append(&header, bytes).unwrap();
}

fn zip_bytes(symlink: bool) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ::zip::ZipWriter::new(cursor);
    let directory = ::zip::write::SimpleFileOptions::default()
        .compression_method(::zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    writer.add_directory("bullet-farm/", directory).unwrap();
    writer.add_directory("bullet-farm/bin/", directory).unwrap();
    let file = ::zip::write::SimpleFileOptions::default()
        .compression_method(::zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    if symlink {
        writer
            .add_symlink("bullet-farm/bin/link", "target", file)
            .unwrap();
    } else {
        for executable in PACKAGED_BINARY_NAMES {
            writer
                .start_file(format!("bullet-farm/bin/{executable}"), file)
                .unwrap();
            writer
                .write_all(if executable == "bullet-family" {
                    b"fixture-binary\n"
                } else {
                    b"fixture-tool\n"
                })
                .unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn file_with(bytes: &[u8]) -> File {
    let mut temporary = tempfile::NamedTempFile::new().expect("archive temporary file");
    temporary.write_all(bytes).expect("archive bytes");
    temporary.flush().expect("flush archive");
    temporary.reopen().expect("reopen archive")
}

fn stage_tree(parent: &Path, name: &str) -> PathBuf {
    let root = parent.join(name).join("bullet-farm");
    fs::create_dir_all(&root).expect("staging tree");
    fs::write(root.join("marker"), b"complete").expect("staging marker");
    File::open(root.parent().unwrap())
        .and_then(|directory| directory.sync_all())
        .expect("sync staging parent");
    root
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("fixture bytes contain marker")
}
