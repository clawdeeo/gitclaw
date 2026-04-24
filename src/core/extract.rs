use std::fs;
use std::io;
use std::path::Path;

use thiserror::Error;

use crate::core::constants::{
    DEB_DATA_TAR, DEB_DATA_TAR_BZ2, DEB_DATA_TAR_GZ, DEB_DATA_TAR_XZ, DEB_DATA_TAR_ZST,
    DIR_EXTRACTED, EXT_BIN, EXT_DEB, EXT_TAR, EXT_TAR_BZ2, EXT_TAR_GZ, EXT_TAR_XZ, EXT_TAR_ZST,
    EXT_TBZ2, EXT_TGZ, EXT_TZST, EXT_TXZ, EXT_ZIP,
};

const AR_HEADER_SIZE: usize = 60;
const AR_FILENAME_END: usize = 16;
const AR_FILESIZE_START: usize = 48;
const AR_FILESIZE_END: usize = 58;
const AR_FILESIZE_END_SHORT: usize = 56;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    TarGz,
    Zip,
    TarBz2,
    TarXz,
    TarZst,
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

    if lower.ends_with(EXT_TAR_GZ) || lower.ends_with(EXT_TGZ) {
        Ok(ArchiveType::TarGz)
    } else if lower.ends_with(EXT_ZIP) {
        Ok(ArchiveType::Zip)
    } else if lower.ends_with(EXT_TAR_BZ2) || lower.ends_with(EXT_TBZ2) {
        Ok(ArchiveType::TarBz2)
    } else if lower.ends_with(EXT_TAR_XZ) || lower.ends_with(EXT_TXZ) {
        Ok(ArchiveType::TarXz)
    } else if lower.ends_with(EXT_TAR_ZST) || lower.ends_with(EXT_TZST) {
        Ok(ArchiveType::TarZst)
    } else if lower.ends_with(EXT_TAR) {
        Ok(ArchiveType::TarGz)
    } else if lower.ends_with(EXT_DEB) {
        Ok(ArchiveType::Deb)
    } else if lower.ends_with(EXT_BIN) || !lower.contains('.') {
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
    const AR_MAGIC: &[u8] = b"!<arch>\n";

    if deb_content.len() < AR_MAGIC.len() || &deb_content[0..AR_MAGIC.len()] != AR_MAGIC {
        return Err(ExtractionError::UnsupportedFormat(
            "Invalid .deb file: missing ar magic".to_string(),
        ));
    }

    let mut offset = AR_MAGIC.len();

    while offset + AR_HEADER_SIZE <= deb_content.len() {
        let header = &deb_content[offset..offset + AR_HEADER_SIZE];
        offset += AR_HEADER_SIZE;

        let name_field = &header[0..AR_FILENAME_END];

        let name_end = name_field
            .iter()
            .rposition(|&b| b != b' ' && b != b'/')
            .map(|i| i + 1)
            .unwrap_or(0);

        let name_bytes = &name_field[0..name_end];

        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| {
                ExtractionError::UnsupportedFormat(
                    "Invalid UTF-8 in .deb header name field".to_string(),
                )
            })?
            .to_string();

        let size_field = std::str::from_utf8(&header[AR_FILESIZE_START..AR_FILESIZE_END])
            .map_err(|_| {
                ExtractionError::UnsupportedFormat(
                    "Invalid UTF-8 in .deb header size field".to_string(),
                )
            })?
            .trim();

        let size: usize = size_field
            .parse()
            .or_else(|_| {
                std::str::from_utf8(&header[AR_FILESIZE_START..AR_FILESIZE_END_SHORT])
                    .map_err(|_| {
                        ExtractionError::UnsupportedFormat(
                            "Invalid UTF-8 in .deb size field fallback".to_string(),
                        )
                    })
                    .map(|s| s.trim().parse::<usize>().unwrap_or(0))
            })
            .unwrap_or(0);

        if size == 0 || offset + size > deb_content.len() {
            if offset >= deb_content.len() {
                break;
            }

            continue;
        }

        if name == DEB_DATA_TAR_GZ
            || name == DEB_DATA_TAR_XZ
            || name == DEB_DATA_TAR_BZ2
            || name == DEB_DATA_TAR_ZST
            || name == DEB_DATA_TAR
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

    if full_name.ends_with(EXT_TAR_GZ) || ext == "gz" {
        extract_tar_gz(tar_path, dest_dir)
    } else if full_name.ends_with(EXT_TAR_XZ) || ext == "xz" {
        extract_tar_xz(tar_path, dest_dir)
    } else if full_name.ends_with(EXT_TAR_BZ2) || ext == "bz2" {
        extract_tar_bz2(tar_path, dest_dir)
    } else if full_name.ends_with(EXT_TAR_ZST) || ext == "zst" {
        extract_tar_zst(tar_path, dest_dir)
    } else {
        let file = fs::File::open(tar_path)?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(dest_dir)?;
        Ok(())
    }
}

fn dispatch_extract(archive_type: ArchiveType, archive_path: &Path, dest: &Path) -> Result<()> {
    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(archive_path, dest),
        ArchiveType::Zip => extract_zip(archive_path, dest),
        ArchiveType::TarBz2 => extract_tar_bz2(archive_path, dest),
        ArchiveType::TarXz => extract_tar_xz(archive_path, dest),
        ArchiveType::TarZst => extract_tar_zst(archive_path, dest),
        ArchiveType::Deb => extract_deb(archive_path, dest),
        ArchiveType::PlainBinary => extract_plain_binary(archive_path, dest),
    }
}

pub fn extract_archive(archive_path: &Path, dest_dir: &Path, prefer_strip: bool) -> Result<()> {
    let archive_type = detect_archive_type(archive_path)?;

    let effective_dest = if prefer_strip {
        dest_dir.to_path_buf()
    } else {
        let name = archive_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or(DIR_EXTRACTED);

        let d = dest_dir.join(name);
        fs::create_dir_all(&d)?;
        d
    };

    dispatch_extract(archive_type, archive_path, &effective_dest)
}
