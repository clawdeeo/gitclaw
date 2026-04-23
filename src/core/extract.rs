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
    Deb,
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
        .ok_or_else(|| ExtractionError::UnknownArchiveType(file_path.display().to_string()))?;

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
        Ok(ArchiveType::TarGz)
    } else if lower.ends_with(".deb") {
        Ok(ArchiveType::Deb)
    } else if lower.ends_with(".bin") || !lower.contains('.') {
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

pub fn extract_tar_zst(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let file = fs::File::open(archive_path)?;
    let reader = io::BufReader::new(file);
    let dec = zstd::stream::read::Decoder::new(reader)?;
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest_dir)?;

    Ok(())
}

pub fn extract_plain_binary(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let filename = archive_path
        .file_name()
        .ok_or_else(|| ExtractionError::UnknownArchiveType(archive_path.display().to_string()))?;

    let dest_file = dest_dir.join(filename);
    fs::copy(archive_path, dest_file)?;

    Ok(())
}

pub fn extract_deb(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;

    let deb_content = fs::read(archive_path)?;
    let (data_tar, tar_name) = extract_data_tar_from_deb(&deb_content)?;

    let temp_dir = tempfile::tempdir()?;
    let data_tar_path = temp_dir.path().join(tar_name);
    fs::write(&data_tar_path, data_tar)?;

    extract_tar_auto(&data_tar_path, dest_dir)?;

    Ok(())
}

fn extract_data_tar_from_deb(deb_content: &[u8]) -> Result<(Vec<u8>, String)> {
    #[allow(clippy::byte_char_slices)]
    const AR_MAGIC: &[u8] = &[b'!', b'<', b'a', b'r', b'c', b'h', b'>', b'\n'];
    if deb_content.len() < AR_MAGIC.len() || &deb_content[0..AR_MAGIC.len()] != AR_MAGIC {
        return Err(ExtractionError::UnsupportedFormat(
            "Invalid .deb file: missing ar magic".to_string(),
        ));
    }

    let mut offset = AR_MAGIC.len();
    while offset + 60 <= deb_content.len() {
        let header = &deb_content[offset..offset + 60];
        offset += 60;

        let name_field = &header[0..16];
        let name_end = name_field
            .iter()
            .rposition(|&b| b != b' ' && b != b'/')
            .map(|i| i + 1)
            .unwrap_or(0);
        let name_bytes = &name_field[0..name_end];
        let name = std::str::from_utf8(name_bytes).unwrap_or("").to_string();

        let size_10 = std::str::from_utf8(&header[48..58]).unwrap_or("0").trim();
        let size_8 = std::str::from_utf8(&header[48..56]).unwrap_or("0").trim();

        let size: usize = size_10
            .parse()
            .ok()
            .or_else(|| size_8.parse().ok())
            .unwrap_or(0);

        if size == 0 || offset + size > deb_content.len() {
            if offset >= deb_content.len() {
                break;
            }
            continue;
        }

        if name == "data.tar.gz"
            || name == "data.tar.xz"
            || name == "data.tar.bz2"
            || name == "data.tar.zst"
            || name == "data.tar"
        {
            let data = deb_content[offset..offset + size].to_vec();
            return Ok((data, name));
        }

        offset += size;
        if !offset.is_multiple_of(2) {
            offset += 1;
        }
    }

    Err(ExtractionError::UnsupportedFormat(
        "Could not find data.tar in .deb package".to_string(),
    ))
}

fn extract_tar_auto(tar_path: &Path, dest_dir: &Path) -> Result<()> {
    let ext = tar_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let full_name = tar_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if full_name.ends_with(".tar.gz") || ext == "gz" {
        extract_tar_gz(tar_path, dest_dir)
    } else if full_name.ends_with(".tar.xz") || ext == "xz" {
        extract_tar_xz(tar_path, dest_dir)
    } else if full_name.ends_with(".tar.bz2") || ext == "bz2" {
        extract_tar_bz2(tar_path, dest_dir)
    } else if full_name.ends_with(".tar.zst") || ext == "zst" {
        extract_tar_zst(tar_path, dest_dir)
    } else {
        let file = fs::File::open(tar_path)?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(dest_dir)?;
        Ok(())
    }
}

pub fn extract_archive(archive_path: &Path, dest_dir: &Path, prefer_strip: bool) -> Result<()> {
    let archive_type = detect_archive_type(archive_path)?;

    if prefer_strip {
        match archive_type {
            ArchiveType::TarGz => extract_tar_gz(archive_path, dest_dir),
            ArchiveType::Zip => extract_zip(archive_path, dest_dir),
            ArchiveType::TarBz2 => extract_tar_bz2(archive_path, dest_dir),
            ArchiveType::TarXz => extract_tar_xz(archive_path, dest_dir),
            ArchiveType::Deb => extract_deb(archive_path, dest_dir),
            ArchiveType::PlainBinary => extract_plain_binary(archive_path, dest_dir),
        }
    } else {
        let name = archive_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("extracted");
        let effective_dest = dest_dir.join(name);
        fs::create_dir_all(&effective_dest)?;

        match archive_type {
            ArchiveType::TarGz => extract_tar_gz(archive_path, &effective_dest),
            ArchiveType::Zip => extract_zip(archive_path, &effective_dest),
            ArchiveType::TarBz2 => extract_tar_bz2(archive_path, &effective_dest),
            ArchiveType::TarXz => extract_tar_xz(archive_path, &effective_dest),
            ArchiveType::Deb => extract_deb(archive_path, &effective_dest),
            ArchiveType::PlainBinary => extract_plain_binary(archive_path, &effective_dest),
        }
    }
}
