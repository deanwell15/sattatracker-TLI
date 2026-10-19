//! Narrow read-only access to bundled icons. Request paths are never filesystem paths.
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const MAX_ICON_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, PartialEq)]
pub enum IconError {
    BadPath,
    NotFound,
    InvalidFile,
}

pub fn read_icon(root: &Path, request_path: &str) -> Result<(Vec<u8>, &'static str), IconError> {
    let name = request_path.strip_prefix('/').ok_or(IconError::BadPath)?;
    let id = name.strip_suffix(".png").ok_or(IconError::BadPath)?;
    if id != "fe" && (id.is_empty() || id.len() > 12 || !id.bytes().all(|b| b.is_ascii_digit())) {
        return Err(IconError::BadPath);
    }
    let root_meta = fs::symlink_metadata(root).map_err(|_| IconError::NotFound)?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(IconError::InvalidFile);
    }
    let candidate = root.join(name);
    let meta = fs::symlink_metadata(&candidate).map_err(|_| IconError::NotFound)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > MAX_ICON_BYTES {
        return Err(IconError::InvalidFile);
    }
    let canonical_root = root.canonicalize().map_err(|_| IconError::NotFound)?;
    let canonical_file = candidate.canonicalize().map_err(|_| IconError::NotFound)?;
    if canonical_file.parent() != Some(canonical_root.as_path()) {
        return Err(IconError::BadPath);
    }
    let mut bytes = Vec::new();
    File::open(canonical_file)
        .map_err(|_| IconError::NotFound)?
        .take(MAX_ICON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| IconError::InvalidFile)?;
    if bytes.len() as u64 > MAX_ICON_BYTES {
        return Err(IconError::InvalidFile);
    }
    // Original archive icons have .png names, but most contain WebP bytes.
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        return Err(IconError::InvalidFile);
    };
    Ok((bytes, mime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> std::path::PathBuf {
        let name = format!("deanfinite-icons-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let root = std::env::temp_dir().join(name);
        fs::create_dir(&root).unwrap();
        root
    }

    #[test]
    fn rejects_arbitrary_paths_before_disk_access() {
        for path in ["/../secret.png", "/%2e%2e/secret.png", "/C:/secret.png", "/100300.png/../config.json", "/100300\\secret.png", "//100300.png", "/100300.png%00", "/100300.png?path=secret", "/100300.PNG", "/.png", "/1234567890123.png", "100300.png"] {
            assert_eq!(read_icon(Path::new("does-not-exist"), path), Err(IconError::BadPath), "{path}");
        }
    }

    #[test]
    fn serves_original_bytes_with_actual_mime() {
        let root = fixture();
        for (id, data, mime) in [("100300", b"RIFF1234WEBPpayload".as_slice(), "image/webp"), ("fe", b"\x89PNG\r\n\x1a\npayload".as_slice(), "image/png")] {
            fs::write(root.join(format!("{id}.png")), data).unwrap();
            assert_eq!(read_icon(&root, &format!("/{id}.png")), Ok((data.to_vec(), mime)));
        }
        assert_eq!(read_icon(&root, "/999.png"), Err(IconError::NotFound));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_bad_data_directories_and_oversize() {
        let root = fixture();
        fs::write(root.join("1.png"), b"not an image").unwrap();
        fs::create_dir(root.join("2.png")).unwrap();
        File::create(root.join("3.png")).unwrap().set_len(MAX_ICON_BYTES + 1).unwrap();
        for id in 1..=3 {
            assert_eq!(read_icon(&root, &format!("/{id}.png")), Err(IconError::InvalidFile));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_links_outside_resource_directory() {
        let root = fixture();
        let outside = fixture();
        fs::write(outside.join("1.png"), b"\x89PNG\r\n\x1a\npayload").unwrap();
        std::os::unix::fs::symlink(outside.join("1.png"), root.join("1.png")).unwrap();
        assert_eq!(read_icon(&root, "/1.png"), Err(IconError::InvalidFile));
        let link = root.join("linked-root");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert_eq!(read_icon(&link, "/1.png"), Err(IconError::InvalidFile));
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }
}
