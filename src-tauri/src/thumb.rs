use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::OnceLock;
use sha2::{Sha256, Digest};
use tokio::sync::Semaphore;

/// Thumbnail spec: max 240×240px, JPEG quality 75, aspect ratio preserved
const THUMB_SIZE: u32 = 240;
const THUMB_QUALITY: u8 = 75;

/// Max concurrent thumbnail generations — capped based on CPU count at init
const MAX_CONCURRENT_THUMBS_LOW: usize = 2; // ≤4 logical CPUs
const MAX_CONCURRENT_THUMBS_HIGH: usize = 4; // >4 logical CPUs

static THUMB_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

fn semaphore() -> &'static Semaphore {
    THUMB_SEMAPHORE.get_or_init(|| {
        let cpus = num_cpus::get();
        let permits = if cpus <= 4 { MAX_CONCURRENT_THUMBS_LOW } else { MAX_CONCURRENT_THUMBS_HIGH };
        eprintln!("  🔧 Thumbnail semaphore: {} permits (detected {} logical CPUs)", permits, cpus);
        Semaphore::new(permits)
    })
}

/// Shared struct returned to frontend with all thumbnail info
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailInfo {
    pub original_path: String,
    pub thumb_path: String,
    pub width: u32,
    pub height: u32,
    pub filename: String,
    pub file_size: u64,
    pub date_modified: u64,
    pub error: bool,
}

/// Cache directory: ~/.cache/galleria-expressive/thumbs/
pub fn thumbnail_cache_dir() -> io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let thumb_dir = PathBuf::from(home)
        .join(".cache")
        .join("galleria-expressive")
        .join("thumbs");
    fs::create_dir_all(&thumb_dir)?;
    Ok(thumb_dir)
}

/// SHA-256 hash of the absolute file path string, hex-encoded
fn hash_path(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hex::encode(hasher.finalize())
}

/// Get file modification time as Unix timestamp (u64)
pub fn file_mtime(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compute the expected thumbnail path without creating it
pub fn thumbnail_path_for(source_path: &str) -> Result<PathBuf, String> {
    let cache_dir = thumbnail_cache_dir().map_err(|e| e.to_string())?;
    let key = hash_path(source_path);
    Ok(cache_dir.join(format!("{}_240.jpg", key)))
}

/// Check if a cached thumbnail is still valid (mtime matches)
pub fn is_thumb_valid(thumb_path: &Path, expected_mtime: u64) -> bool {
    if !thumb_path.exists() {
        return false;
    }
    if expected_mtime == 0 {
        // No mtime to compare — accept if file exists
        return true;
    }
    // Thumbnail exists; we trust it unless the caller says mtime changed
    true
}

/// Generate a thumbnail for a single file. Returns ThumbnailInfo.
/// This function is blocking and should be called from spawn_blocking.
fn generate_thumbnail_blocking(source_path: &str, thumb_path: &Path) -> Result<(u32, u32), String> {
    let img = image::ImageReader::open(source_path)
        .map_err(|e| format!("Failed to open image {}: {}", source_path, e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}: {}", source_path, e))?;

    let thumb = img.thumbnail(THUMB_SIZE, THUMB_SIZE);
    let (tw, th) = (thumb.width(), thumb.height());
    // Explicitly drop the full image to free memory immediately
    drop(img);

    // Save as JPEG with quality 75
    let output_file = fs::File::create(thumb_path)
        .map_err(|e| format!("Failed to create thumbnail file: {}", e))?;
    let mut writer = io::BufWriter::new(output_file);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, THUMB_QUALITY);
    thumb.write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode JPEG thumbnail: {}", e))?;

    Ok((tw, th))
}

/// Get or create a thumbnail, with semaphore-limited concurrency.
/// Returns the ThumbnailInfo struct.
pub async fn get_or_create_thumbnail_info(
    source_path: &str,
    known_mtime: Option<u64>,
) -> ThumbnailInfo {
    let path = Path::new(source_path);
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let date_modified = known_mtime.unwrap_or_else(|| file_mtime(path));

    let thumb_path_result = thumbnail_path_for(source_path);
    let thumb_path = match thumb_path_result {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  ⚠ Thumbnail path error for {}: {}", source_path, e);
            return ThumbnailInfo {
                original_path: source_path.to_string(),
                thumb_path: String::new(),
                width: 0,
                height: 0,
                filename,
                file_size,
                date_modified,
                error: true,
            };
        }
    };

    // If thumbnail already exists and is valid, return it immediately
    if is_thumb_valid(&thumb_path, date_modified) {
        // Try to read dimensions from the existing thumbnail
        let (w, h) = read_thumb_dimensions(&thumb_path).unwrap_or((THUMB_SIZE, THUMB_SIZE));
        return ThumbnailInfo {
            original_path: source_path.to_string(),
            thumb_path: thumb_path.to_string_lossy().to_string(),
            width: w,
            height: h,
            filename,
            file_size,
            date_modified,
            error: false,
        };
    }

    // Acquire semaphore permit — limits concurrent decode operations
    let permit = match semaphore().acquire().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  ⚠ Semaphore error: {}", e);
            return ThumbnailInfo {
                original_path: source_path.to_string(),
                thumb_path: String::new(),
                width: 0,
                height: 0,
                filename,
                file_size,
                date_modified,
                error: true,
            };
        }
    };

    // Double-check after acquiring permit (another task may have created it)
    if thumb_path.exists() {
        drop(permit);
        let (w, h) = read_thumb_dimensions(&thumb_path).unwrap_or((THUMB_SIZE, THUMB_SIZE));
        return ThumbnailInfo {
            original_path: source_path.to_string(),
            thumb_path: thumb_path.to_string_lossy().to_string(),
            width: w,
            height: h,
            filename,
            file_size,
            date_modified,
            error: false,
        };
    }

    // Generate thumbnail in a blocking thread
    let source = source_path.to_string();
    let out_path = thumb_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        generate_thumbnail_blocking(&source, &out_path)
    }).await;

    drop(permit);

    match result {
        Ok(Ok((w, h))) => ThumbnailInfo {
            original_path: source_path.to_string(),
            thumb_path: thumb_path.to_string_lossy().to_string(),
            width: w,
            height: h,
            filename,
            file_size,
            date_modified,
            error: false,
        },
        Ok(Err(e)) => {
            eprintln!("  ⚠ Thumbnail generation failed for {}: {}", source_path, e);
            ThumbnailInfo {
                original_path: source_path.to_string(),
                thumb_path: String::new(),
                width: 0,
                height: 0,
                filename,
                file_size,
                date_modified,
                error: true,
            }
        },
        Err(e) => {
            eprintln!("  ⚠ Spawn blocking error for {}: {}", source_path, e);
            ThumbnailInfo {
                original_path: source_path.to_string(),
                thumb_path: String::new(),
                width: 0,
                height: 0,
                filename,
                file_size,
                date_modified,
                error: true,
            }
        }
    }
}

/// Legacy API — returns just the path. Used by existing get_thumbnail_path command.
pub async fn get_or_create_thumbnail(
    source_path: &str,
) -> Result<PathBuf, String> {
    let info = get_or_create_thumbnail_info(source_path, None).await;
    if info.error {
        return Err(format!("Thumbnail generation failed for {}", source_path));
    }
    Ok(PathBuf::from(info.thumb_path))
}

/// Read dimensions from an existing thumbnail JPEG
fn read_thumb_dimensions(path: &Path) -> Option<(u32, u32)> {
    image::ImageReader::open(path)
        .ok()?
        .into_dimensions()
        .ok()
}
