use anyhow::Result;
use std::fs;
use std::io;
use std::path::Path;

pub fn extract_archive(bytes: &[u8], filename: &str, dest: &Path) -> Result<()> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let dec = flate2::read::GzDecoder::new(bytes);
        tar::Archive::new(dec).unpack(dest)?;
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        let mut dec_bytes = Vec::new();
        let mut dec = xz2::read::XzDecoder::new(bytes);
        io::Read::read_to_end(&mut dec, &mut dec_bytes)?;
        tar::Archive::new(&dec_bytes[..]).unpack(dest)?;
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        let dec = bzip2::read::BzDecoder::new(bytes);
        tar::Archive::new(dec).unpack(dest)?;
    } else if lower.ends_with(".tar") {
        tar::Archive::new(bytes).unpack(dest)?;
    } else if lower.ends_with(".zip") {
        let reader = io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out = dest.join(file.name());
            if file.name().ends_with('/') {
                fs::create_dir_all(&out)?;
            } else {
                if let Some(p) = out.parent() { fs::create_dir_all(p)?; }
                let mut f = fs::File::create(&out)?;
                io::copy(&mut file, &mut f)?;
                #[cfg(unix)] {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        fs::set_permissions(&out, fs::Permissions::from_mode(mode))?;
                    }
                }
            }
        }
    } else {
        let out = dest.join(filename);
        fs::write(&out, bytes)?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&out, fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

pub fn detect_archive_type(filename: &str) -> Option<&'static str> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") { Some("tar.gz") }
    else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") { Some("tar.xz") }
    else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") { Some("tar.bz2") }
    else if lower.ends_with(".tar") { Some("tar") }
    else if lower.ends_with(".zip") { Some("zip") }
    else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_archive_type() {
        assert_eq!(detect_archive_type("file.tar.gz"), Some("tar.gz"));
        assert_eq!(detect_archive_type("file.zip"), Some("zip"));
        assert_eq!(detect_archive_type("file.exe"), None);
    }
}
