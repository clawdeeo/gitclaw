//! Integration tests for the extract module
//! Tests the public API from gitclaw::extract

use std::fs;
use std::io::Write;
use tempfile::TempDir;

/// Helper to create a tar.gz archive for testing
fn create_test_tar_gz(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.gz");
    let file = fs::File::create(&archive_path).unwrap();
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(enc);

    for (name, content) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, *name, &content[..])
            .unwrap();
    }

    builder.finish().unwrap();
    archive_path
}

/// Helper to create a zip archive for testing
fn create_test_zip(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.zip");
    let file = fs::File::create(&archive_path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for (name, content) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(*content).unwrap();
    }

    writer.finish().unwrap();
    archive_path
}

/// Helper to create a tar.bz2 archive for testing
fn create_test_tar_bz2(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.bz2");
    let file = fs::File::create(&archive_path).unwrap();
    let enc = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
    let mut builder = tar::Builder::new(enc);

    for (name, content) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, *name, &content[..])
            .unwrap();
    }

    builder.finish().unwrap();
    archive_path
}

/// Helper to create a tar.xz archive for testing
fn create_test_tar_xz(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.xz");
    let file = fs::File::create(&archive_path).unwrap();
    let enc = xz2::write::XzEncoder::new(file, 6);
    let mut builder = tar::Builder::new(enc);

    for (name, content) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, *name, &content[..])
            .unwrap();
    }

    builder.finish().unwrap();
    archive_path
}

/// Helper to create a plain binary file for testing
fn create_test_binary(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let binary_path = dir.path().join(name);
    fs::write(&binary_path, content).unwrap();
    binary_path
}

#[test]
fn test_detect_archive_type_tar_gz() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.tar.gz");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarGz
    );

    let path = Path::new("test.tgz");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarGz
    );
}

#[test]
fn test_detect_archive_type_tar() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.tar");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarGz
    );
}

#[test]
fn test_detect_archive_type_zip() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.zip");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::Zip
    );
}

#[test]
fn test_detect_archive_type_tar_bz2() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.tar.bz2");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarBz2
    );

    let path = Path::new("test.tbz2");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarBz2
    );
}

#[test]
fn test_detect_archive_type_tar_xz() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.tar.xz");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarXz
    );

    let path = Path::new("test.txz");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::TarXz
    );
}

#[test]
fn test_detect_archive_type_plain_binary() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    let path = Path::new("test.bin");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::PlainBinary
    );

    let path = Path::new("test.exe");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::PlainBinary
    );

    let path = Path::new("test");
    assert_eq!(
        gitclaw::extract::detect_archive_type(path).unwrap(),
        ArchiveType::PlainBinary
    );
}

#[test]
fn test_detect_archive_type_unknown() {
    use std::path::Path;

    let path = Path::new("test.txt");
    assert!(gitclaw::extract::detect_archive_type(path).is_err());

    let path = Path::new("test.pdf");
    assert!(gitclaw::extract::detect_archive_type(path).is_err());
}

#[test]
fn test_extract_archive_tar_gz() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let files: [(&str, &[u8]); 2] = [
        ("file1.txt", b"Hello, World!"),
        ("file2.txt", b"Test content"),
    ];
    let archive = create_test_tar_gz(&temp, &files);
    let dest = temp.path().join("extracted");

    extract_archive(&archive, &dest).unwrap();

    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("file2.txt").exists());
    assert_eq!(
        fs::read_to_string(dest.join("file1.txt")).unwrap(),
        "Hello, World!"
    );
    assert_eq!(
        fs::read_to_string(dest.join("file2.txt")).unwrap(),
        "Test content"
    );
}

#[test]
fn test_extract_archive_zip() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let files: [(&str, &[u8]); 2] = [
        ("file1.txt", b"Hello, World!"),
        ("file2.txt", b"Test content"),
    ];
    let archive = create_test_zip(&temp, &files);
    let dest = temp.path().join("extracted");

    extract_archive(&archive, &dest).unwrap();

    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("file2.txt").exists());
    assert_eq!(
        fs::read_to_string(dest.join("file1.txt")).unwrap(),
        "Hello, World!"
    );
}

#[test]
fn test_extract_archive_tar_bz2() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let files: [(&str, &[u8]); 1] = [("test.txt", b"Bz2 compressed")];
    let archive = create_test_tar_bz2(&temp, &files);
    let dest = temp.path().join("extracted");

    extract_archive(&archive, &dest).unwrap();

    assert!(dest.join("test.txt").exists());
    assert_eq!(
        fs::read_to_string(dest.join("test.txt")).unwrap(),
        "Bz2 compressed"
    );
}

#[test]
fn test_extract_archive_tar_xz() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let files: [(&str, &[u8]); 1] = [("test.txt", b"Xz compressed")];
    let archive = create_test_tar_xz(&temp, &files);
    let dest = temp.path().join("extracted");

    extract_archive(&archive, &dest).unwrap();

    assert!(dest.join("test.txt").exists());
    assert_eq!(
        fs::read_to_string(dest.join("test.txt")).unwrap(),
        "Xz compressed"
    );
}

#[test]
fn test_extract_archive_plain_binary() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let binary = create_test_binary(&temp, "myapp", b"binary content");
    let dest = temp.path().join("extracted");

    extract_archive(&binary, &dest).unwrap();

    assert!(dest.join("myapp").exists());
    assert_eq!(fs::read(dest.join("myapp")).unwrap(), b"binary content");
}

#[test]
fn test_extract_archive_nested_dirs() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let files: [(&str, &[u8]); 2] = [
        ("dir1/file1.txt", b"Nested file 1"),
        ("dir1/dir2/file2.txt", b"Nested file 2"),
    ];
    let archive = create_test_tar_gz(&temp, &files);
    let dest = temp.path().join("extracted");

    extract_archive(&archive, &dest).unwrap();

    assert!(dest.join("dir1/file1.txt").exists());
    assert!(dest.join("dir1/dir2/file2.txt").exists());
    assert_eq!(
        fs::read_to_string(dest.join("dir1/file1.txt")).unwrap(),
        "Nested file 1"
    );
    assert_eq!(
        fs::read_to_string(dest.join("dir1/dir2/file2.txt")).unwrap(),
        "Nested file 2"
    );
}

#[test]
fn test_extract_archive_nonexistent() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar.gz");
    let dest = temp.path().join("extracted");

    let result = extract_archive(&nonexistent, &dest);
    assert!(result.is_err());
}
