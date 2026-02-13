use crate::db::Database;
use crate::scan;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;

pub struct AppState {
    db: Mutex<Option<Database>>,
    library_root: Mutex<Option<String>>,
    library_roots: Mutex<Vec<(i64, String)>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    pub phase: String,
    pub current: u64,
    pub total: Option<u64>,
}

fn db_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("app data dir")
        .join("photo_sorter.db")
}

#[tauri::command]
pub async fn select_and_index(app: AppHandle, path: String) -> Result<serde_json::Value, String> {
    let path = std::path::PathBuf::from(&path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid or missing directory".to_string());
    }
    let root_str = path.to_string_lossy().to_string();

    let db_path = db_path(&app);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let db = Database::new(&db_path).map_err(|e| e.to_string())?;
    let library_id = db.get_or_create_library(&root_str).map_err(|e| e.to_string())?;
    
    // For now, we clear existing photos to avoid duplicates during re-scan. 
    // Ideally we would do a diff, but clearing is safer for MVP.
    // db.clear_photos_for_library(library_id).map_err(|e| e.to_string())?;

    app.emit("index-progress", IndexProgress {
        phase: "scanning".to_string(),
        current: 0,
        total: None,
    })
    .ok();

    let (tx, mut rx) = mpsc::unbounded_channel::<(Vec<crate::db::PhotoRecord>, u64, u64)>();
    let path_clone = path.clone();
    let app_handle = app.clone();
    
    // Spawn listener to handle DB insertion and event emission on the main thread (or async context)
    let recv_handle = tauri::async_runtime::spawn(async move {
        // We need a separate connection for the async listener to insert data? 
        // Or we can just emit the data and let the frontend handle it?
        // Actually, we should insert here or in the blocking thread. 
        // Let's insert in the blocking thread where we have the data, and just emit here.
        // Wait, DB insertion should happen in the blocking thread to avoid locking async runtime.
        // So the channel will just carry the "Saved" photos to emit to frontend.
        
        while let Some((photos, current, total)) = rx.recv().await {
             app_handle.emit("photos-added", &photos).ok();
             app_handle
                .emit("index-progress", IndexProgress {
                    phase: "indexing".to_string(),
                    current,
                    total: Some(total),
                })
                .ok();
        }
    });

    let scanned_count = tauri::async_runtime::spawn_blocking(move || {
        let root = path_clone.canonicalize().unwrap_or_else(|_| path_clone.clone());
        let paths = scan::collect_media_paths(&path_clone);
        let total = paths.len() as u64;
        
        // We need a DB connection here in the thread
        let db = match Database::new(&db_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open DB in thread: {}", e);
                return 0;
            }
        };

        const CHUNK: usize = 20; // Smaller chunk for faster feedback
        let mut processed = 0;

        for chunk in paths.chunks(CHUNK) {
            let batch = scan::process_paths_batch(chunk, &root);
            let mut saved_photos = Vec::new();

            for s in &batch {
                 match db.insert_photo(
                    library_id,
                    &s.path,
                    &s.filename,
                    &s.folder_rel,
                    s.taken_at.as_deref(),
                    &s.modified_at,
                    &s.media_type,
                    s.size_bytes,
                    s.width,
                    s.height,
                    s.camera_make.as_deref(),
                    s.camera_model.as_deref(),
                    s.lens.as_deref(),
                    s.iso,
                    s.shutter_speed.as_deref(),
                    s.aperture.as_deref(),
                    s.focal_length.as_deref(),
                    s.gps_lat,
                    s.gps_lon,
                ) {
                    Ok(record) => saved_photos.push(record),
                    Err(e) => eprintln!("Failed to insert photo {}: {}", s.path, e),
                }
            }
            
            processed += batch.len();
            let _ = tx.send((saved_photos, processed as u64, total));
        }
        processed
    })
    .await
    .map_err(|e| e.to_string())?;

    let _ = recv_handle.await;

    app.emit("index-progress", IndexProgress {
        phase: "done".to_string(),
        current: scanned_count as u64,
        total: Some(scanned_count as u64),
    })
    .ok();

    if let Some(state) = app.try_state::<AppState>() {
        *state.db.lock().unwrap() = Some(db);
        *state.library_root.lock().unwrap() = Some(root_str.clone());
        eprintln!("✓ App state set: library_root = {}, library_id = {}, total photos = {}", root_str, library_id, scanned_count);
    }

    Ok(serde_json::json!({
        "libraryPath": root_str,
        "totalPhotos": scanned_count,
        "libraryId": library_id
    }))
}



#[tauri::command]
pub async fn get_categories(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let root_guard = state.library_root.lock().unwrap();
    let root = root_guard.as_ref().ok_or("No library path")?;

    let library_id = db.get_or_create_library(root).map_err(|e| e.to_string())?;

    let years = db.get_years(library_id).map_err(|e| e.to_string())?;
    let folders = db.get_folders_flat(library_id).map_err(|e| e.to_string())?;
    let types = db.get_media_type_counts(library_id).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "years": years,
        "folders": folders,
        "mediaTypes": types
    }))
}

#[tauri::command]
pub async fn get_months(
    state: State<'_, AppState>,
    year: i32,
) -> Result<Vec<crate::db::CategoryMonth>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let root_guard = state.library_root.lock().unwrap();
    let root = root_guard.as_ref().ok_or("No library path")?;
    let library_id = db.get_or_create_library(root).map_err(|e| e.to_string())?;
    db.get_months(library_id, year).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct GetPhotosParams {
    limit: Option<i64>,
    offset: Option<i64>,
    year: Option<i32>,
    month: Option<i32>,
    folder: Option<String>,
    #[serde(rename = "mediaType")]
    media_type: Option<String>,
}

#[tauri::command]
pub async fn get_photos(
    state: State<'_, AppState>,
    params: Option<GetPhotosParams>,
) -> Result<Vec<crate::db::PhotoRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let root_guard = state.library_root.lock().unwrap();
    let root = root_guard.as_ref().ok_or("No library path")?;

    let library_id = db.get_or_create_library(root).map_err(|e| e.to_string())?;

    let limit = params.as_ref().and_then(|p| p.limit).unwrap_or(100);
    // Security hard cap to prevent DOS
    let limit = limit.min(10000);
    let offset = params.as_ref().and_then(|p| p.offset).unwrap_or(0);
    let year = params.as_ref().and_then(|p| p.year);
    let month = params.as_ref().and_then(|p| p.month);
    let folder = params.as_ref().and_then(|p| p.folder.as_deref());
    let media_type = params.as_ref().and_then(|p| p.media_type.as_deref());

    db.get_photos(library_id, limit, offset, year, month, folder, media_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_photos(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<crate::db::PhotoRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let root_guard = state.library_root.lock().unwrap();
    let root = root_guard.as_ref().ok_or("No library path")?;

    let library_id = db.get_or_create_library(root).map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(100).min(1000);

    db.search_photos(library_id, &query, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_thumbnail_path(app: AppHandle, source_path: String) -> Result<String, String> {
    let path = crate::thumb::get_or_create_thumbnail(&app, &source_path)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_index_progress() -> Result<Option<IndexProgress>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn get_current_library_path(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let root = state.library_root.lock().unwrap();
    Ok(root.clone())
}

pub fn setup_state(app: &tauri::AppHandle) {
    app.manage(AppState {
        db: Mutex::new(None),
        library_root: Mutex::new(None),
        library_roots: Mutex::new(Vec::new()),
    });
}

/// Auto-scan default user directories for photos
#[tauri::command]
pub async fn scan_default_directories(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let home = std::env::var("HOME").map_err(|_| "Could not determine HOME directory".to_string())?;
    let dirs_to_scan: Vec<(&str, String)> = vec![
        ("Pictures", format!("{}/Pictures", home)),
        ("Downloads", format!("{}/Downloads", home)),
        ("Documents", format!("{}/Documents", home)),
    ];

    let db_path = db_path(&app);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let db = Database::new(&db_path).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let mut all_library_roots: Vec<(i64, String)> = Vec::new();

    for (name, dir_path) in &dirs_to_scan {
        let path = std::path::PathBuf::from(dir_path);
        if !path.exists() || !path.is_dir() {
            eprintln!("⏭ Skipping {}: directory not found", dir_path);
            continue;
        }

        let root_str = path.to_string_lossy().to_string();
        let library_id = db.get_or_create_library(&root_str).map_err(|e| e.to_string())?;

        app.emit("index-progress", IndexProgress {
            phase: format!("scanning-{}", name.to_lowercase()),
            current: 0,
            total: None,
        }).ok();

        // Check if already indexed (has photos) — skip if so for speed
        let existing_count = db.count_photos_for_library(library_id).unwrap_or(0);
        if existing_count > 0 {
            eprintln!("✓ {} already indexed ({} photos), skipping", name, existing_count);
            all_library_roots.push((library_id, root_str.clone()));
            results.push(serde_json::json!({
                "name": name,
                "path": root_str,
                "libraryId": library_id,
                "photoCount": existing_count,
                "skipped": true
            }));
            continue;
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<(u64, u64)>();
        let app_handle = app.clone();
        let scan_name = name.to_string();
        let recv_handle = tauri::async_runtime::spawn(async move {
            while let Some((current, total)) = rx.recv().await {
                app_handle
                    .emit("index-progress", IndexProgress {
                        phase: format!("indexing-{}", scan_name.to_lowercase()),
                        current,
                        total: Some(total),
                    })
                    .ok();
            }
        });

        let path_clone = path.clone();
        let scanned = tauri::async_runtime::spawn_blocking(move || {
            let root = path_clone.canonicalize().unwrap_or_else(|_| path_clone.clone());
            let paths = scan::collect_media_paths(&path_clone);
            let total = paths.len() as u64;
            let _ = tx.send((0, total));
            let mut all: Vec<scan::ScannedFile> = Vec::new();
            const CHUNK: usize = 50;
            for chunk in paths.chunks(CHUNK) {
                let batch = scan::process_paths_batch(chunk, &root);
                all.extend(batch);
                let _ = tx.send((all.len() as u64, total));
            }
            all
        })
        .await
        .map_err(|e| e.to_string())?;

        let _ = recv_handle.await;

        let photo_count = scanned.len();
        for s in &scanned {
            db.insert_photo(
                library_id,
                &s.path,
                &s.filename,
                &s.folder_rel,
                s.taken_at.as_deref(),
                &s.modified_at,
                &s.media_type,
                s.size_bytes,
                s.width,
                s.height,
                s.camera_make.as_deref(),
                s.camera_model.as_deref(),
                s.lens.as_deref(),
                s.iso,
                s.shutter_speed.as_deref(),
                s.aperture.as_deref(),
                s.focal_length.as_deref(),
                s.gps_lat,
                s.gps_lon,
            )
            .map_err(|e| e.to_string())?;
        }

        eprintln!("✓ Indexed {} ({} photos)", name, photo_count);
        all_library_roots.push((library_id, root_str.clone()));
        results.push(serde_json::json!({
            "name": name,
            "path": root_str,
            "libraryId": library_id,
            "photoCount": photo_count,
            "skipped": false
        }));
    }

    app.emit("index-progress", IndexProgress {
        phase: "done".to_string(),
        current: 0,
        total: Some(0),
    }).ok();

    // Store in state
    *state.db.lock().unwrap() = Some(db);
    *state.library_roots.lock().unwrap() = all_library_roots.clone();
    // Also set first library as primary for backward compatibility
    if let Some((_, ref path)) = all_library_roots.first() {
        *state.library_root.lock().unwrap() = Some(path.clone());
    }

    Ok(serde_json::json!({ "sources": results }))
}

/// Get photos from all indexed libraries
#[tauri::command]
pub async fn get_all_photos(
    state: State<'_, AppState>,
    params: Option<GetPhotosParams>,
) -> Result<Vec<crate::db::PhotoRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let roots = state.library_roots.lock().unwrap();
    let library_ids: Vec<i64> = roots.iter().map(|(id, _)| *id).collect();

    if library_ids.is_empty() {
        return Ok(Vec::new());
    }

    let limit = params.as_ref().and_then(|p| p.limit).unwrap_or(10000).min(20000);
    let offset = params.as_ref().and_then(|p| p.offset).unwrap_or(0);

    db.get_photos_all_libraries(&library_ids, limit, offset)
        .map_err(|e| e.to_string())
}

/// Add a new library path to be indexed and tracked
#[tauri::command]
pub async fn add_library_path(
    app: AppHandle,
    path: String,
) -> Result<serde_json::Value, String> {
    // This reuses select_and_index logic but is explicitly for adding a secondary path
    select_and_index(app, path).await
}

/// Remove a library path (and its photos) from the index
#[tauri::command]
pub async fn remove_library_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    
    // Check if library exists
    let libs = db.get_all_libraries().map_err(|e| e.to_string())?;
    let lib = libs.into_iter().find(|l| l.root_path == path).ok_or("Library not found")?;

    // Delete from DB
    // We need a method in DB to delete library and all its photos.
    // For now we just clear photos.
    db.clear_photos_for_library(lib.id).map_err(|e| e.to_string())?;
    
    // Also remove from state if it's there
    let mut roots = state.library_roots.lock().unwrap();
    roots.retain(|(_, p)| p != &path);
    
    Ok(())
}

/// Get list of all indexed libraries/sources
#[tauri::command]
pub async fn get_library_paths(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::LibraryInfo>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_all_libraries().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_libraries(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::LibraryInfo>, String> {
    get_library_paths(state).await
}

/// Toggle favorite status on a photo
#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppState>,
    photo_id: i64,
) -> Result<bool, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.toggle_favorite(photo_id).map_err(|e| e.to_string())
}

/// Soft-delete photos (move to trash)
#[tauri::command]
pub async fn soft_delete_photos(
    state: State<'_, AppState>,
    photo_ids: Vec<i64>,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.soft_delete(&photo_ids).map_err(|e| e.to_string())
}

/// Restore photos from trash
#[tauri::command]
pub async fn restore_photos(
    state: State<'_, AppState>,
    photo_ids: Vec<i64>,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.restore_from_trash(&photo_ids).map_err(|e| e.to_string())
}

/// Get detailed photo info by ID
#[tauri::command]
pub async fn get_photo_detail(
    state: State<'_, AppState>,
    photo_id: i64,
) -> Result<Option<crate::db::PhotoRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_photo_by_id(photo_id).map_err(|e| e.to_string())
}

// ── File operations ──

/// Permanently delete photos from DB and optionally from disk
#[tauri::command]
pub async fn hard_delete_photos(
    state: State<'_, AppState>,
    photo_ids: Vec<i64>,
    delete_from_disk: bool,
) -> Result<u64, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    let paths = db.hard_delete_photos(&photo_ids).map_err(|e| e.to_string())?;
    if delete_from_disk {
        for p in &paths {
            let _ = std::fs::remove_file(p);
        }
    }
    Ok(paths.len() as u64)
}

/// Rename a photo file on disk and in DB
#[tauri::command]
pub async fn rename_photo(
    state: State<'_, AppState>,
    photo_id: i64,
    new_filename: String,
) -> Result<String, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    // Get old path first
    let old_photo = db.get_photo_by_id(photo_id).map_err(|e| e.to_string())?;
    let old_photo = old_photo.ok_or("Photo not found")?;
    let old_path = std::path::PathBuf::from(&old_photo.path);
    let new_path_str = db.rename_photo(photo_id, &new_filename).map_err(|e| e.to_string())?;
    let new_path = std::path::PathBuf::from(&new_path_str);
    // Rename on disk
    if old_path.exists() {
        std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    }
    Ok(new_path_str)
}

// ── Tags ──

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<crate::db::TagRecord, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.create_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    tag_id: i64,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.delete_tag(tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tags(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::TagRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tag_photos(
    state: State<'_, AppState>,
    photo_ids: Vec<i64>,
    tag_id: i64,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.tag_photos(&photo_ids, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn untag_photos(
    state: State<'_, AppState>,
    photo_ids: Vec<i64>,
    tag_id: i64,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.untag_photos(&photo_ids, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_photo_tags(
    state: State<'_, AppState>,
    photo_id: i64,
) -> Result<Vec<crate::db::TagRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_tags_for_photo(photo_id).map_err(|e| e.to_string())
}

// ── Albums ──

#[tauri::command]
pub async fn create_album(
    state: State<'_, AppState>,
    name: String,
) -> Result<crate::db::AlbumRecord, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.create_album(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_album(
    state: State<'_, AppState>,
    album_id: i64,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.delete_album(album_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_album(
    state: State<'_, AppState>,
    album_id: i64,
    new_name: String,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.rename_album(album_id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_albums(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::AlbumRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_albums().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_album(
    state: State<'_, AppState>,
    album_id: i64,
    photo_ids: Vec<i64>,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.add_photos_to_album(album_id, &photo_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_album(
    state: State<'_, AppState>,
    album_id: i64,
    photo_ids: Vec<i64>,
) -> Result<(), String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.remove_photos_from_album(album_id, &photo_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_album_photos(
    state: State<'_, AppState>,
    album_id: i64,
) -> Result<Vec<crate::db::PhotoRecord>, String> {
    let db_guard = state.db.lock().unwrap();
    let db = db_guard.as_ref().ok_or("No library loaded")?;
    db.get_album_photos(album_id).map_err(|e| e.to_string())
}
