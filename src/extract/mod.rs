use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    TarGz,
    Zip,
    TarBz2,
    TarXz,
    PlainBinary,
}

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("IO error during extraction: {0}")]
    Io(#[from] io::Error),
    #[error("Zip extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Unknown archive type for file: {0}")]
    UnknownArchiveType(String),
    #[error("Unsupported archive format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, ExtractionError>;

pub fn detect_archive_type(file_path: &Path) -> Result<ArchiveType> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            ExtractionError::UnknownArchiveType(file_path.display().to_string())
        })?;

    let lower = filename.to_lowercase();

    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Ok(ArchiveType::TarGz)
    } else if lower.ends_with(".zip") {
        Ok(ArchiveType::Zip)
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Ok(ArchiveType::TarBz2)
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Ok(ArchiveType::TarXz)
    } else if lower.ends_with(".tar") {
        // Plain tar - treat as TarGz for extraction purposes
        Ok(ArchiveType::TarGz)
    } else if lower.ends_with(".exe")
        || lower.ends_with(".bin")
        || !lower.contains('.')
    {
        Ok(ArchiveType::PlainBinary)
    } else {
        Err(ExtractionError::UnknownArchiveType(filename.to_string()))
    }
}

pub fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let file = fs::File::open(archive_path)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest_dir)?;

    Ok(())
}

pub fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let file = fs::File::open(archive_path)?;
    let reader = io::BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = dest_dir.join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut out_file)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    Ok(())
}

pub fn extract_tar_bz2(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let file = fs::File::open(archive_path)?;
    let dec = bzip2::read::BzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest_dir)?;

    Ok(())
}

pub fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let file = fs::File::open(archive_path)?;
    let dec = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest_dir)?;

    Ok(())
}

pub fn extract_plain_binary(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let filename = archive_path
        .file_name()
        .ok_or_else(|| {
            ExtractionError::UnknownArchiveType(archive_path.display().to_string())
        })?;

    let dest_file = dest_dir.join(filename);
    fs::copy(archive_path, dest_file)?;

    Ok(())
}

pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let archive_type = detect_archive_type(archive_path)?;

    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(archive_path, dest_dir),
        ArchiveType::Zip => extract_zip(archive_path, dest_dir),
        ArchiveType::TarBz2 => extract_tar_bz2(archive_path, dest_dir),
        ArchiveType::TarXz => extract_tar_xz(archive_path, dest_dir),
        ArchiveType::PlainBinary => extract_plain_binary(archive_path, dest_dir),
    }
}

pub fn extract(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    extract_archive(archive_path, dest_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

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
    fn test_detect_archive_type_tar_gz() {
        let path = std::path::Path::new("/tmp/test.tar.gz");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarGz);

        let path = std::path::Path::new("/tmp/test.tgz");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarGz);
    }

    #[test]
    fn test_detect_archive_type_zip() {
        let path = std::path::Path::new("/tmp/test.zip");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::Zip);
    }

    #[test]
    fn test_detect_archive_type_tar_bz2() {
        let path = std::path::Path::new("/tmp/test.tar.bz2");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarBz2);

        let path = std::path::Path::new("/tmp/test.tbz2");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarBz2);
    }

    #[test]
    fn test_detect_archive_type_tar_xz() {
        let path = std::path::Path::new("/tmp/test.tar.xz");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarXz);

        let path = std::path::Path::new("/tmp/test.txz");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::TarXz);
    }

    #[test]
    fn test_detect_archive_type_plain_binary() {
        let path = std::path::Path::new("/tmp/test.bin");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::PlainBinary);

        let path = std::path::Path::new("/tmp/test.exe");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::PlainBinary);

        let path = std::path::Path::new("/tmp/test");
        assert_eq!(detect_archive_type(path).unwrap(), ArchiveType::PlainBinary);
    }

    #[test]
    fn test_detect_archive_type_unknown() {
        let path = std::path::Path::new("/tmp/test.txt");
        assert!(detect_archive_type(path).is_err());

        let path = std::path::Path::new("/tmp/test.pdf");
        assert!(detect_archive_type(path).is_err());
    }

    #[test]
    fn test_extract_tar_gz() {
        let temp = TempDir::new().unwrap();
        let files = vec![("file1.txt", b"Hello, World!"), ("file2.txt", b"Test data")];
        let archive_path = create_test_tar_gz(&temp, &files);

        let dest_dir = temp.path().join("extracted");
        extract_tar_gz(&archive_path, &dest_dir).unwrap();

        let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "Hello, World!");

        let content2 = fs::read_to_string(dest_dir.join("file2.txt")).unwrap();
        assert_eq!(content2, "Test data");
    }

    #[test]
    fn test_extract_zip() {
        let temp = TempDir::new().unwrap();
        let files = vec![("file1.txt", b"Hello, World!"), ("file2.txt", b"Test data")];
        let archive_path = create_test_zip(&temp, &files);

        let dest_dir = temp.path().join("extracted");
        extract_zip(&archive_path, &dest_dir).unwrap();

        let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "Hello, World!");

        let content2 = fs::read_to_string(dest_dir.join("file2.txt")).unwrap();
        assert_eq!(content2, "Test data");
    }

    #[test]
    fn test_extract_dispatch_tar_gz() {
        let temp = TempDir::new().unwrap();
        let files = vec![("test.txt", b"Dispatched!")];
        let archive_path = create_test_tar_gz(&temp, &files);

        let dest_dir = temp.path().join("extracted");
        extract(&archive_path, &dest_dir).unwrap();

        let content = fs::read_to_string(dest_dir.join("test.txt")).unwrap();
        assert_eq!(content, "Dispatched!");
    }

    #[test]
    fn test_extract_dispatch_zip() {
        let temp = TempDir::new().unwrap();
        let files = vec![("test.txt", b"Dispatched!")];
        let archive_path = create_test_zip(&temp, &files);

        let dest_dir = temp.path().join("extracted");
        extract(&archive_path, &dest_dir).unwrap();

        let content = fs::read_to_string(dest_dir.join("test.txt")).unwrap();
        assert_eq!(content, "Dispatched!");
    }
}