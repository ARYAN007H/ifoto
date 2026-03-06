use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::Path;
use tauri::Emitter;

/// Start watching a directory for file system changes.
/// Emits Tauri events when files are added, removed, or renamed.
pub fn start_watcher(
    app_handle: tauri::AppHandle,
    dir_path: String,
) -> Result<RecommendedWatcher, String> {
    let app = app_handle.clone();
    let watched_dir = dir_path.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => handle_fs_event(&app, &event, &watched_dir),
            Err(e) => eprintln!("  ⚠ File watcher error: {}", e),
        }
    }).map_err(|e| format!("Failed to create file watcher: {}", e))?;

    watcher.watch(Path::new(&dir_path), RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch directory {}: {}", dir_path, e))?;

    eprintln!("  👁 Watching directory: {}", dir_path);
    Ok(watcher)
}

fn handle_fs_event(app: &tauri::AppHandle, event: &Event, _watched_dir: &str) {
    let media_extensions = [
        "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif",
        "heic", "heif", "raw", "arw", "cr2", "nef", "dng",
        "mp4", "mov", "avi", "mkv", "webm", "m4v", "wmv", "3gp",
    ];

    let is_media = |path: &std::path::Path| -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| media_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    };

    match event.kind {
        EventKind::Create(_) => {
            for path in &event.paths {
                if is_media(path) {
                    let path_str = path.to_string_lossy().to_string();
                    eprintln!("  📁 File added: {}", path_str);
                    let _ = app.emit("gallery-file-added", serde_json::json!({
                        "path": path_str
                    }));
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                if is_media(path) {
                    let path_str = path.to_string_lossy().to_string();
                    eprintln!("  🗑 File removed: {}", path_str);
                    let _ = app.emit("gallery-file-removed", serde_json::json!({
                        "path": path_str
                    }));
                }
            }
        }
        EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
            // Rename events come in pairs on Linux (from, to)
            if event.paths.len() >= 2 {
                let old_path = event.paths[0].to_string_lossy().to_string();
                let new_path = event.paths[1].to_string_lossy().to_string();
                eprintln!("  📝 File renamed: {} → {}", old_path, new_path);
                let _ = app.emit("gallery-file-renamed", serde_json::json!({
                    "oldPath": old_path,
                    "newPath": new_path
                }));
            } else {
                // Single-path rename event (half of a rename pair)
                for path in &event.paths {
                    if is_media(path) {
                        if path.exists() {
                            let path_str = path.to_string_lossy().to_string();
                            let _ = app.emit("gallery-file-added", serde_json::json!({
                                "path": path_str
                            }));
                        } else {
                            let path_str = path.to_string_lossy().to_string();
                            let _ = app.emit("gallery-file-removed", serde_json::json!({
                                "path": path_str
                            }));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
