use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::OnceLock;
use tauri::Manager;
use tokio::sync::Semaphore;

const THUMB_SIZE: u32 = 320;
/// Max concurrent thumbnail generations to prevent memory spikes
const MAX_CONCURRENT_THUMBS: usize = 4;

static THUMB_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

fn semaphore() -> &'static Semaphore {
    THUMB_SEMAPHORE.get_or_init(|| Semaphore::new(MAX_CONCURRENT_THUMBS))
}

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

/// Compute the expected thumbnail path without creating it
pub fn thumbnail_path_for(app_handle: &tauri::AppHandle, source_path: &str) -> Result<PathBuf, String> {
    let cache_dir = thumbnail_cache_dir(app_handle).map_err(|e| e.to_string())?;
    let key = hash_path(source_path);
    Ok(cache_dir.join(format!("{}.jpg", key)))
}

pub async fn get_or_create_thumbnail(
    app_handle: &tauri::AppHandle,
    source_path: &str,
) -> Result<PathBuf, String> {
    let cache_dir = thumbnail_cache_dir(app_handle).map_err(|e| e.to_string())?;
    let key = hash_path(source_path);
    let thumb_path = cache_dir.join(format!("{}.jpg", key));

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

    // Limit concurrent thumbnail generation to prevent memory spikes
    let _permit = semaphore()
        .acquire()
        .await
        .map_err(|e| format!("Semaphore error: {}", e))?;

    // Double-check after acquiring permit (another task may have created it)
    if thumb_path.exists() {
        return Ok(thumb_path);
    }

    let source = source_path.to_string();
    let out_path = thumb_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let img = image::ImageReader::open(&source)
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;

        let thumb = img.thumbnail(THUMB_SIZE, THUMB_SIZE);
        // Explicitly drop the full image to free memory immediately
        drop(img);

        thumb
            .save(&out_path)
            .map_err(|e| e.to_string())?;
        Ok::<PathBuf, String>(out_path)
    })
    .await
    .map_err(|e| e.to_string())?
}
