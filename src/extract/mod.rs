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