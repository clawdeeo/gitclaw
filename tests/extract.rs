use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

// Helper to create tar.gz archive
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
        builder.append_data(&mut header, *name, &content[..]).unwrap();
    }

    builder.finish().unwrap();
    archive_path
}

// Helper to create zip archive
fn create_test_zip(dir: &TempDir, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let archive_path = dir.path().join("test.zip");
    let file = fs::File::create(&archive_path).unwrap();
    let mut writer = zip::write::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (name, content) in files {
        writer.start_file(name, options).unwrap();
        writer.write_all(content).unwrap();
    }

    writer.finish().unwrap();
    archive_path
}

#[test]
fn test_extract_tar_gz() {
    let temp = TempDir::new().unwrap();
    let files: Vec<(&str, &[u8])> = vec![("file1.txt", b"Hello, World!" as &[u8]), ("file2.txt", b"Test data" as &[u8])];
    let archive_path = create_test_tar_gz(&temp, &files);

    let dest_dir = temp.path().join("extracted");
    gitclaw::extract::extract_tar_gz(&archive_path, &dest_dir).unwrap();

    let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
    assert_eq!(content1, "Hello, World!");

    let content2 = fs::read_to_string(dest_dir.join("file2.txt")).unwrap();
    assert_eq!(content2, "Test data");
}

#[test]
fn test_extract_zip() {
    let temp = TempDir::new().unwrap();
    let files: Vec<(&str, &[u8])> = vec![("file1.txt", b"Hello, World!" as &[u8]), ("file2.txt", b"Test data" as &[u8])];
    let archive_path = create_test_zip(&temp, &files);

    let dest_dir = temp.path().join("extracted");
    gitclaw::extract::extract_zip(&archive_path, &dest_dir).unwrap();

    let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
    assert_eq!(content1, "Hello, World!");

    let content2 = fs::read_to_string(dest_dir.join("file2.txt")).unwrap();
    assert_eq!(content2, "Test data");
}

#[test]
fn test_extract_unknown_archive_type() {
    let temp = TempDir::new().unwrap();
    
    // Create a file with unknown extension
    let unknown_file = temp.path().join("test.unknown");
    fs::write(&unknown_file, b"some data").unwrap();
    
    let dest_dir = temp.path().join("extracted");
    
    // Should fail with unknown archive type
    let result = gitclaw::extract::extract(&unknown_file, &dest_dir);
    assert!(result.is_err());
    
    // Verify it's the right kind of error
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Unknown archive type"));
}

#[test]
fn test_extract_dispatch_tar_gz() {
    let temp = TempDir::new().unwrap();
    let files: Vec<(&str, &[u8])> = vec![("test.txt", b"Dispatched from tar.gz!" as &[u8])];
    let archive_path = create_test_tar_gz(&temp, &files);

    let dest_dir = temp.path().join("extracted");
    gitclaw::extract::extract(&archive_path, &dest_dir).unwrap();

    let content = fs::read_to_string(dest_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Dispatched from tar.gz!");
}

#[test]
fn test_extract_dispatch_zip() {
    let temp = TempDir::new().unwrap();
    let files: Vec<(&str, &[u8])> = vec![("test.txt", b"Dispatched from zip!" as &[u8])];
    let archive_path = create_test_zip(&temp, &files);

    let dest_dir = temp.path().join("extracted");
    gitclaw::extract::extract(&archive_path, &dest_dir).unwrap();

    let content = fs::read_to_string(dest_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Dispatched from zip!");
}

#[test]
fn test_detect_archive_type_variations() {
    use gitclaw::extract::{detect_archive_type, ArchiveType};
    
    // Test tar.gz variations
    assert_eq!(detect_archive_type(Path::new("file.tar.gz")).unwrap(), ArchiveType::TarGz);
    assert_eq!(detect_archive_type(Path::new("file.tgz")).unwrap(), ArchiveType::TarGz);
    
    // Test zip
    assert_eq!(detect_archive_type(Path::new("file.zip")).unwrap(), ArchiveType::Zip);
    
    // Test tar.bz2 variations
    assert_eq!(detect_archive_type(Path::new("file.tar.bz2")).unwrap(), ArchiveType::TarBz2);
    assert_eq!(detect_archive_type(Path::new("file.tbz2")).unwrap(), ArchiveType::TarBz2);
    
    // Test tar.xz variations
    assert_eq!(detect_archive_type(Path::new("file.tar.xz")).unwrap(), ArchiveType::TarXz);
    assert_eq!(detect_archive_type(Path::new("file.txz")).unwrap(), ArchiveType::TarXz);
    
    // Test plain binary variations
    assert_eq!(detect_archive_type(Path::new("file.bin")).unwrap(), ArchiveType::PlainBinary);
    assert_eq!(detect_archive_type(Path::new("file.exe")).unwrap(), ArchiveType::PlainBinary);
    assert_eq!(detect_archive_type(Path::new("mybinary")).unwrap(), ArchiveType::PlainBinary);
    
    // Test unknown types
    assert!(detect_archive_type(Path::new("file.txt")).is_err());
    assert!(detect_archive_type(Path::new("file.pdf")).is_err());
    assert!(detect_archive_type(Path::new("file.docx")).is_err());
}