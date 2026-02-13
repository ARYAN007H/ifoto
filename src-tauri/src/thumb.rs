use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use tauri::Manager;

const THUMB_SIZE: u32 = 256;

pub fn thumbnail_cache_dir(app_handle: &tauri::AppHandle) -> io::Result<PathBuf> {
    let cache = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let thumb_dir = cache.join("thumbnails");
    fs::create_dir_all(&thumb_dir)?;
    Ok(thumb_dir)
}

fn hash_path(path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn get_or_create_thumbnail(
    app_handle: &tauri::AppHandle,
    source_path: &str,
) -> Result<PathBuf, String> {
    let cache_dir = thumbnail_cache_dir(app_handle).map_err(|e| e.to_string())?;
    let key = hash_path(source_path);
    let ext = Path::new(source_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let thumb_path = cache_dir.join(format!("{}.{}", key, ext));

    if thumb_path.exists() {
        return Ok(thumb_path);
    }

    let path = Path::new(source_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    let media_type = crate::scan::media_type_from_path(path);
    if media_type == "video" {
        return Err("Video thumbnails not implemented yet".to_string());
    }

    let img = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let thumb = img.thumbnail(THUMB_SIZE, THUMB_SIZE);
    thumb
        .save(&thumb_path)
        .map_err(|e| e.to_string())?;

    Ok(thumb_path)
}
