pub mod commands;
mod db;
mod scan;
mod thumb;



pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            commands::setup_state(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::select_and_index,
            commands::get_categories,
            commands::get_months,
            commands::get_photos,
            commands::search_photos,
            commands::get_thumbnail_path,
            commands::get_index_progress,
            commands::get_current_library_path,
            commands::scan_default_directories,
            commands::get_all_photos,
            commands::get_photo_count,
            commands::get_libraries,
            commands::restore_session,
            commands::add_library_path,
            commands::remove_library_path,
            commands::get_library_paths,
            commands::toggle_favorite,
            commands::soft_delete_photos,
            commands::restore_photos,
            commands::get_photo_detail,
            // File operations
            commands::hard_delete_photos,
            commands::rename_photo,
            // Tags
            commands::create_tag,
            commands::delete_tag,
            commands::get_tags,
            commands::tag_photos,
            commands::untag_photos,
            commands::get_photo_tags,
            // Albums
            commands::create_album,
            commands::delete_album,
            commands::rename_album,
            commands::get_albums,
            commands::add_to_album,
            commands::remove_from_album,
            commands::get_album_photos,
            // Photo editor
            commands::save_edited_photo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
