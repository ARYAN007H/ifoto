use rayon::prelude::*;
use rexif::parse_file;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

const PHOTO_EXT: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "raw", "arw", "cr2",
    "nef", "dng",
];
const VIDEO_EXT: &[&str] = &["mp4", "mov", "avi", "mkv", "webm", "m4v", "wmv", "3gp"];

fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

pub fn media_type_from_path(path: &Path) -> &'static str {
    let ext = match get_extension(path) {
        Some(e) => e,
        None => return "other",
    };
    if PHOTO_EXT.contains(&ext.as_str()) {
        "photo"
    } else if VIDEO_EXT.contains(&ext.as_str()) {
        "video"
    } else {
        "other"
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExifData {
    pub taken_at: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<i32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

fn parse_exif_data(path: &Path) -> ExifData {
    let mut data = ExifData::default();
    let exif = match parse_file(path) {
        Ok(e) => e,
        Err(_) => return data,
    };

    let mut gps_lat_vals: Option<Vec<f64>> = None;
    let mut gps_lat_ref: Option<String> = None;
    let mut gps_lon_vals: Option<Vec<f64>> = None;
    let mut gps_lon_ref: Option<String> = None;

    for entry in &exif.entries {
        match entry.tag {
            rexif::ExifTag::DateTimeOriginal | rexif::ExifTag::DateTime => {
                if data.taken_at.is_none() {
                    let s = entry.value_more_readable.to_string().replace(':', "-");
                    if s.len() >= 10 {
                        data.taken_at = Some(format!("{}T00:00:00Z", &s[..10]));
                    }
                }
            }
            rexif::ExifTag::Make => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() { data.camera_make = Some(v); }
            }
            rexif::ExifTag::Model => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() { data.camera_model = Some(v); }
            }
            rexif::ExifTag::LensModel => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() { data.lens = Some(v); }
            }
            rexif::ExifTag::ISOSpeedRatings => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if let Ok(iso) = v.parse::<i32>() {
                    data.iso = Some(iso);
                }
            }
            rexif::ExifTag::ExposureTime => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() { data.shutter_speed = Some(v); }
            }
            rexif::ExifTag::FNumber => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() {
                    data.aperture = Some(format!("f/{}", v.trim_start_matches("f/")));
                }
            }
            rexif::ExifTag::FocalLength => {
                let v = entry.value_more_readable.to_string().trim().to_string();
                if !v.is_empty() { data.focal_length = Some(v); }
            }
            rexif::ExifTag::GPSLatitude => {
                if let rexif::TagValue::URational(ref vals) = entry.value {
                    let floats: Vec<f64> = vals.iter().map(|r| r.numerator as f64 / r.denominator as f64).collect();
                    if floats.len() == 3 { gps_lat_vals = Some(floats); }
                }
            }
            rexif::ExifTag::GPSLatitudeRef => {
                gps_lat_ref = Some(entry.value_more_readable.to_string().trim().to_string());
            }
            rexif::ExifTag::GPSLongitude => {
                if let rexif::TagValue::URational(ref vals) = entry.value {
                    let floats: Vec<f64> = vals.iter().map(|r| r.numerator as f64 / r.denominator as f64).collect();
                    if floats.len() == 3 { gps_lon_vals = Some(floats); }
                }
            }
            rexif::ExifTag::GPSLongitudeRef => {
                gps_lon_ref = Some(entry.value_more_readable.to_string().trim().to_string());
            }
            _ => {}
        }
    }

    // Convert GPS DMS to decimal
    if let Some(ref vals) = gps_lat_vals {
        let mut lat = vals[0] + vals[1] / 60.0 + vals[2] / 3600.0;
        if gps_lat_ref.as_deref() == Some("S") { lat = -lat; }
        data.gps_lat = Some(lat);
    }
    if let Some(ref vals) = gps_lon_vals {
        let mut lon = vals[0] + vals[1] / 60.0 + vals[2] / 3600.0;
        if gps_lon_ref.as_deref() == Some("W") { lon = -lon; }
        data.gps_lon = Some(lon);
    }

    data
}

fn modified_time_string(path: &Path) -> String {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return "1970-01-01T00:00:00Z".to_string(),
    };
    let modified = match meta.modified() {
        Ok(t) => t,
        Err(_) => return "1970-01-01T00:00:00Z".to_string(),
    };
    let d = match modified.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "1970-01-01T00:00:00Z".to_string(),
    };
    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Phase 1: collect media file paths only (fast).
pub fn collect_media_paths(root: &Path) -> Vec<PathBuf> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            let ext = get_extension(p);
            ext.map(|e| {
                PHOTO_EXT.contains(&e.as_str()) || VIDEO_EXT.contains(&e.as_str())
            })
            .unwrap_or(false)
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: String,
    pub filename: String,
    pub folder_rel: String,
    pub taken_at: Option<String>,
    pub modified_at: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    // EXIF
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<i32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<String>,
    pub focal_length: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

fn build_scanned_file(path: &Path, root: &Path) -> Option<ScannedFile> {
    let path_str = path.to_string_lossy().to_string();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let folder_rel = path
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let media_type = media_type_from_path(path).to_string();
    let size_bytes = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);
    let modified_at = modified_time_string(path);

    let exif = if media_type == "photo" {
        parse_exif_data(path)
    } else {
        ExifData::default()
    };

    let (width, height) = if media_type == "photo" {
        image::ImageReader::open(path)
            .ok()
            .and_then(|r| r.into_dimensions().ok())
            .map(|(w, h)| (Some(w as i32), Some(h as i32)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    Some(ScannedFile {
        path: path_str,
        filename,
        folder_rel,
        taken_at: exif.taken_at.or_else(|| Some(modified_at.clone())),
        modified_at,
        media_type,
        size_bytes,
        width,
        height,
        camera_make: exif.camera_make,
        camera_model: exif.camera_model,
        lens: exif.lens,
        iso: exif.iso,
        shutter_speed: exif.shutter_speed,
        aperture: exif.aperture,
        focal_length: exif.focal_length,
        gps_lat: exif.gps_lat,
        gps_lon: exif.gps_lon,
    })
}

/// Process a batch of paths into ScannedFile (for chunked progress).
pub fn process_paths_batch(paths: &[PathBuf], root: &Path) -> Vec<ScannedFile> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    paths
        .iter()
        .filter_map(|path| build_scanned_file(path, &root))
        .collect()
}

#[allow(dead_code)]
pub fn scan_directory(root: &Path) -> Vec<ScannedFile> {
    let paths = collect_media_paths(root);
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    paths
        .par_iter()
        .filter_map(|path| build_scanned_file(path, &root))
        .collect()
}
