use std::io::Write;
use tempfile::TempDir;

fn create_test_tar_gz(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.gz");
    let file = std::fs::File::create(&archive_path).unwrap();
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

fn create_test_zip(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.zip");
    let file = std::fs::File::create(&archive_path).unwrap();
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

fn create_test_tar_bz2(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.bz2");
    let file = std::fs::File::create(&archive_path).unwrap();
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

fn create_test_tar_xz(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.tar.xz");
    let file = std::fs::File::create(&archive_path).unwrap();
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

fn create_test_binary(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let binary_path = dir.path().join(name);
    std::fs::write(&binary_path, content).unwrap();
    binary_path
}

#[test]
fn test_detect_archive_type_tar_gz() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tar.gz")).unwrap(),
        ArchiveType::TarGz
    );
    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tgz")).unwrap(),
        ArchiveType::TarGz
    );
}

#[test]
fn test_detect_archive_type_tar() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tar")).unwrap(),
        ArchiveType::TarGz
    );
}

#[test]
fn test_detect_archive_type_zip() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.zip")).unwrap(),
        ArchiveType::Zip
    );
}

#[test]
fn test_detect_archive_type_tar_bz2() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tar.bz2")).unwrap(),
        ArchiveType::TarBz2
    );
    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tbz2")).unwrap(),
        ArchiveType::TarBz2
    );
}

#[test]
fn test_detect_archive_type_tar_xz() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.tar.xz")).unwrap(),
        ArchiveType::TarXz
    );
    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.txz")).unwrap(),
        ArchiveType::TarXz
    );
}

#[test]
fn test_detect_archive_type_plain_binary() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.bin")).unwrap(),
        ArchiveType::PlainBinary
    );
    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test")).unwrap(),
        ArchiveType::PlainBinary
    );
}

#[test]
fn test_detect_archive_type_deb() {
    use gitclaw::extract::ArchiveType;
    use std::path::Path;

    assert_eq!(
        gitclaw::extract::detect_archive_type(Path::new("test.deb")).unwrap(),
        ArchiveType::Deb
    );
}

#[test]
fn test_detect_archive_type_unknown() {
    use std::path::Path;

    assert!(gitclaw::extract::detect_archive_type(Path::new("test.txt")).is_err());
    assert!(gitclaw::extract::detect_archive_type(Path::new("test.pdf")).is_err());
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

    extract_archive(&archive, &dest, true).unwrap();

    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("file2.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("file1.txt")).unwrap(),
        "Hello, World!"
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("file2.txt")).unwrap(),
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

    extract_archive(&archive, &dest, true).unwrap();

    assert!(dest.join("file1.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("file1.txt")).unwrap(),
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

    extract_archive(&archive, &dest, true).unwrap();

    assert!(dest.join("test.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("test.txt")).unwrap(),
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

    extract_archive(&archive, &dest, true).unwrap();

    assert!(dest.join("test.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("test.txt")).unwrap(),
        "Xz compressed"
    );
}

#[test]
fn test_extract_archive_plain_binary() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let binary = create_test_binary(&temp, "myapp", b"binary content");
    let dest = temp.path().join("extracted");

    extract_archive(&binary, &dest, true).unwrap();

    assert!(dest.join("myapp").exists());
    assert_eq!(
        std::fs::read(dest.join("myapp")).unwrap(),
        b"binary content"
    );
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

    extract_archive(&archive, &dest, true).unwrap();

    assert!(dest.join("dir1/file1.txt").exists());
    assert!(dest.join("dir1/dir2/file2.txt").exists());
}

#[test]
fn test_extract_archive_nonexistent() {
    use gitclaw::extract::extract_archive;

    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("nonexistent.tar.gz");
    let dest = temp.path().join("extracted");

    assert!(extract_archive(&nonexistent, &dest, true).is_err());
}

#[test]
fn test_extract_nonexistent_file() {
    let result = gitclaw::extract::extract_archive(
        std::path::Path::new("/nonexistent/path/to/file.tar.gz"),
        std::path::Path::new("/tmp/output"),
        true,
    );
    assert!(result.is_err());
}

#[test]
fn test_extract_corrupted_tar_gz() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("corrupted.tar.gz");

    let mut file = std::fs::File::create(&archive_path).unwrap();
    file.write_all(b"not a valid gzip file").unwrap();
    drop(file);

    let output_dir = temp_dir.path().join("output");
    assert!(gitclaw::extract::extract_archive(&archive_path, &output_dir, true).is_err());
}

#[test]
fn test_extract_corrupted_zip() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("corrupted.zip");

    let mut file = std::fs::File::create(&archive_path).unwrap();
    file.write_all(b"PK\x03\x04invalid").unwrap();
    drop(file);

    let output_dir = temp_dir.path().join("output");
    assert!(gitclaw::extract::extract_archive(&archive_path, &output_dir, true).is_err());
}

#[test]
fn test_detect_unknown_archive_type() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.unknown");
    std::fs::File::create(&file_path).unwrap();

    assert!(gitclaw::extract::detect_archive_type(&file_path).is_err());
}
