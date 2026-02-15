use chrono::Utc;
use rusqlite::{Connection, Result as SqlResult};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoRecord {
    pub id: i64,
    pub path: String,
    pub filename: String,
    pub folder_rel: String,
    pub taken_at: Option<String>,
    pub modified_at: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    // EXIF fields
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub id: i64,
    pub root_path: String,
    pub name: String,
    pub photo_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryYear {
    pub year: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryMonth {
    pub year: i32,
    pub month: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FolderNode {
    pub path: String,
    pub name: String,
    pub count: i64,
    pub children: Vec<FolderNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRecord {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumRecord {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub photo_count: i64,
    pub cover_path: Option<String>,
}

impl Database {
    pub fn new(db_path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS library (
                id INTEGER PRIMARY KEY,
                root_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS photos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                library_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                filename TEXT NOT NULL,
                folder_rel TEXT NOT NULL,
                taken_at TEXT,
                modified_at TEXT NOT NULL,
                media_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                width INTEGER,
                height INTEGER,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                deleted_at TEXT,
                camera_make TEXT,
                camera_model TEXT,
                lens TEXT,
                iso INTEGER,
                shutter_speed TEXT,
                aperture TEXT,
                focal_length TEXT,
                gps_lat REAL,
                gps_lon REAL,
                UNIQUE(library_id, path),
                FOREIGN KEY (library_id) REFERENCES library(id)
            );
            CREATE INDEX IF NOT EXISTS idx_photos_library_taken ON photos(library_id, taken_at);
            CREATE INDEX IF NOT EXISTS idx_photos_library_folder ON photos(library_id, folder_rel);
            CREATE INDEX IF NOT EXISTS idx_photos_library_type ON photos(library_id, media_type);
            CREATE INDEX IF NOT EXISTS idx_photos_path_search ON photos(path, filename);

            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#0071e3'
            );
            CREATE TABLE IF NOT EXISTS photo_tags (
                photo_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (photo_id, tag_id),
                FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS albums (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS album_photos (
                album_id INTEGER NOT NULL,
                photo_id INTEGER NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (album_id, photo_id),
                FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
                FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
            );
            "#,
        )?;
        // Run migrations for existing databases
        self.run_migrations(&conn)?;
        Ok(())
    }

    fn run_migrations(&self, conn: &Connection) -> SqlResult<()> {
        // Check if is_favorite column exists, if not add it
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(photos)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        let migrations: Vec<(&str, &str)> = vec![
            ("is_favorite", "ALTER TABLE photos ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0"),
            ("is_deleted", "ALTER TABLE photos ADD COLUMN is_deleted INTEGER NOT NULL DEFAULT 0"),
            ("deleted_at", "ALTER TABLE photos ADD COLUMN deleted_at TEXT"),
            ("camera_make", "ALTER TABLE photos ADD COLUMN camera_make TEXT"),
            ("camera_model", "ALTER TABLE photos ADD COLUMN camera_model TEXT"),
            ("lens", "ALTER TABLE photos ADD COLUMN lens TEXT"),
            ("iso", "ALTER TABLE photos ADD COLUMN iso INTEGER"),
            ("shutter_speed", "ALTER TABLE photos ADD COLUMN shutter_speed TEXT"),
            ("aperture", "ALTER TABLE photos ADD COLUMN aperture TEXT"),
            ("focal_length", "ALTER TABLE photos ADD COLUMN focal_length TEXT"),
            ("gps_lat", "ALTER TABLE photos ADD COLUMN gps_lat REAL"),
            ("gps_lon", "ALTER TABLE photos ADD COLUMN gps_lon REAL"),
        ];

        for (col, sql) in migrations {
            if !columns.contains(&col.to_string()) {
                conn.execute(sql, [])?;
                eprintln!("  ➕ Migrated: added column {}", col);
            }
        }
        Ok(())
    }

    pub fn get_or_create_library(&self, root_path: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let id: i64 = conn.query_row(
            "INSERT INTO library (root_path, created_at) VALUES (?1, ?2) ON CONFLICT(root_path) DO UPDATE SET created_at = ?2 RETURNING id",
            [root_path, &now],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn clear_photos_for_library(&self, library_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM photos WHERE library_id = ?1", [library_id])?;
        Ok(())
    }

    pub fn remove_library(&self, library_id: i64) -> SqlResult<()> {
        self.clear_photos_for_library(library_id)?;
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM library WHERE id = ?1", [library_id])?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_photo(
        &self,
        library_id: i64,
        path: &str,
        filename: &str,
        folder_rel: &str,
        taken_at: Option<&str>,
        modified_at: &str,
        media_type: &str,
        size_bytes: i64,
        width: Option<i32>,
        height: Option<i32>,
        camera_make: Option<&str>,
        camera_model: Option<&str>,
        lens: Option<&str>,
        iso: Option<i32>,
        shutter_speed: Option<&str>,
        aperture: Option<&str>,
        focal_length: Option<&str>,
        gps_lat: Option<f64>,
        gps_lon: Option<f64>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO photos (library_id, path, filename, folder_rel, taken_at, modified_at, media_type, size_bytes, width, height,
                                           camera_make, camera_model, lens, iso, shutter_speed, aperture, focal_length, gps_lat, gps_lon)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            rusqlite::params![
                library_id,
                path,
                filename,
                folder_rel,
                taken_at,
                modified_at,
                media_type,
                size_bytes,
                width,
                height,
                camera_make,
                camera_model,
                lens,
                iso,
                shutter_speed,
                aperture,
                focal_length,
                gps_lat,
                gps_lon,
            ],
        )?;
        Ok(())
    }

    /// Helper: standard columns for photo queries
    fn photo_select_cols() -> &'static str {
        "id, path, filename, folder_rel, taken_at, modified_at, media_type, size_bytes, width, height, is_favorite, is_deleted, deleted_at, camera_make, camera_model, lens, iso, shutter_speed, aperture, focal_length, gps_lat, gps_lon"
    }

    /// Helper: construct PhotoRecord from a row with standard columns
    fn photo_from_row(row: &rusqlite::Row, source: String) -> rusqlite::Result<PhotoRecord> {
        Ok(PhotoRecord {
            id: row.get(0)?,
            path: row.get(1)?,
            filename: row.get(2)?,
            folder_rel: row.get(3)?,
            taken_at: row.get(4)?,
            modified_at: row.get(5)?,
            media_type: row.get(6)?,
            size_bytes: row.get(7)?,
            width: row.get(8)?,
            height: row.get(9)?,
            source,
            is_favorite: row.get::<_, i32>(10).unwrap_or(0) != 0,
            is_deleted: row.get::<_, i32>(11).unwrap_or(0) != 0,
            deleted_at: row.get(12)?,
            camera_make: row.get(13)?,
            camera_model: row.get(14)?,
            lens: row.get(15)?,
            iso: row.get(16)?,
            shutter_speed: row.get(17)?,
            aperture: row.get(18)?,
            focal_length: row.get(19)?,
            gps_lat: row.get(20)?,
            gps_lon: row.get(21)?,
        })
    }

    pub fn get_photos(
        &self,
        library_id: i64,
        limit: i64,
        offset: i64,
        year: Option<i32>,
        month: Option<i32>,
        folder_rel: Option<&str>,
        media_type: Option<&str>,
    ) -> SqlResult<Vec<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = format!(
            "SELECT {} FROM photos WHERE library_id = ?1 AND is_deleted = 0",
            Self::photo_select_cols()
        );
        let mut extra: Vec<String> = Vec::new();

        if let Some(y) = year {
            sql.push_str(" AND strftime('%Y', COALESCE(taken_at, modified_at)) = ?");
            extra.push(format!("{:04}", y));
        }
        if let Some(m) = month {
            sql.push_str(" AND strftime('%m', COALESCE(taken_at, modified_at)) = ?");
            extra.push(format!("{:02}", m));
        }
        if let Some(f) = folder_rel {
            sql.push_str(" AND (folder_rel = ? OR folder_rel LIKE ?)");
            extra.push(f.to_string());
            extra.push(format!("{}%", f));
        }
        if let Some(t) = media_type {
            sql.push_str(" AND media_type = ?");
            extra.push(t.to_string());
        }

        sql.push_str(" ORDER BY COALESCE(taken_at, modified_at) DESC, path LIMIT ? OFFSET ?");

        let mut stmt = conn.prepare(&sql)?;
        let mut param_refs: Vec<&dyn rusqlite::ToSql> = vec![&library_id];
        for e in &extra {
            param_refs.push(e);
        }
        param_refs.push(&limit);
        param_refs.push(&offset);

        let mut rows = stmt.query(rusqlite::params_from_iter(param_refs))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(Self::photo_from_row(row, String::new())?);
        }
        Ok(out)
    }

    pub fn search_photos(
        &self,
        library_id: i64,
        query: &str,
        limit: i64,
    ) -> SqlResult<Vec<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let sql = format!(
            "SELECT {} FROM photos \
             WHERE library_id = ?1 AND is_deleted = 0 AND \
             (path LIKE ?2 ESCAPE '\\' OR filename LIKE ?2 ESCAPE '\\' OR folder_rel LIKE ?2 ESCAPE '\\' \
              OR taken_at LIKE ?2 ESCAPE '\\' OR camera_make LIKE ?2 ESCAPE '\\' OR camera_model LIKE ?2 ESCAPE '\\' \
              OR id IN (SELECT pt.photo_id FROM photo_tags pt JOIN tags t ON t.id=pt.tag_id WHERE t.name LIKE ?2 ESCAPE '\\')) \
             ORDER BY taken_at DESC LIMIT ?3",
            Self::photo_select_cols()
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params![library_id, pattern, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(Self::photo_from_row(row, String::new())?);
        }
        Ok(out)
    }

    pub fn get_years(&self, library_id: i64) -> SqlResult<Vec<CategoryYear>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT CAST(strftime('%Y', taken_at) AS INTEGER) AS y, COUNT(*) FROM photos 
             WHERE library_id = ?1 AND taken_at IS NOT NULL GROUP BY y ORDER BY y DESC",
        )?;
        let mut rows = stmt.query([library_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(CategoryYear {
                year: row.get(0)?,
                count: row.get(1)?,
            });
        }
        Ok(out)
    }

    pub fn get_months(&self, library_id: i64, year: i32) -> SqlResult<Vec<CategoryMonth>> {
        let conn = self.conn.lock().unwrap();
        let year_str = format!("{:04}", year);
        let mut stmt = conn.prepare(
            "SELECT CAST(strftime('%Y', taken_at) AS INTEGER), CAST(strftime('%m', taken_at) AS INTEGER), COUNT(*) 
             FROM photos WHERE library_id = ?1 AND strftime('%Y', taken_at) = ?2 GROUP BY strftime('%Y-%m', taken_at) ORDER BY 2 DESC",
        )?;
        let mut rows = stmt.query(rusqlite::params![library_id, year_str])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(CategoryMonth {
                year: row.get(0)?,
                month: row.get(1)?,
                count: row.get(2)?,
            });
        }
        Ok(out)
    }

    pub fn get_folders_flat(&self, library_id: i64) -> SqlResult<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT folder_rel, COUNT(*) FROM photos WHERE library_id = ?1 GROUP BY folder_rel ORDER BY folder_rel",
        )?;
        let mut rows = stmt.query([library_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((row.get(0)?, row.get(1)?));
        }
        Ok(out)
    }

    pub fn get_media_type_counts(&self, library_id: i64) -> SqlResult<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT media_type, COUNT(*) FROM photos WHERE library_id = ?1 GROUP BY media_type",
        )?;
        let mut rows = stmt.query([library_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push((row.get(0)?, row.get(1)?));
        }
        Ok(out)
    }

    pub fn get_photo_by_id(&self, id: i64) -> SqlResult<Option<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {} FROM photos WHERE id = ?1", Self::photo_select_cols());
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(Self::photo_from_row(row, String::new())?));
        }
        Ok(None)
    }

    pub fn get_all_libraries(&self) -> SqlResult<Vec<LibraryInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT l.id, l.root_path, COUNT(p.id) FROM library l LEFT JOIN photos p ON p.library_id = l.id GROUP BY l.id ORDER BY l.root_path",
        )?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let root_path: String = row.get(1)?;
            let name = std::path::Path::new(&root_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Library")
                .to_string();
            out.push(LibraryInfo {
                id: row.get(0)?,
                root_path,
                name,
                photo_count: row.get(2)?,
            });
        }
        Ok(out)
    }

    pub fn get_photos_all_libraries(
        &self,
        library_ids: &[i64],
        limit: i64,
        offset: i64,
    ) -> SqlResult<Vec<PhotoRecord>> {
        if library_ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<String> = library_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let sql = format!(
            "SELECT p.id, p.path, p.filename, p.folder_rel, p.taken_at, p.modified_at, p.media_type, p.size_bytes, p.width, p.height, \
             p.is_favorite, p.is_deleted, p.deleted_at, p.camera_make, p.camera_model, p.lens, p.iso, p.shutter_speed, p.aperture, p.focal_length, p.gps_lat, p.gps_lon, \
             l.root_path \
             FROM photos p JOIN library l ON l.id = p.library_id \
             WHERE p.library_id IN ({}) AND p.is_deleted = 0 \
             ORDER BY COALESCE(p.taken_at, p.modified_at) DESC, p.path LIMIT ?{} OFFSET ?{}",
            placeholders.join(", "),
            library_ids.len() + 1,
            library_ids.len() + 2,
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = library_ids.iter().map(|id| Box::new(*id) as Box<dyn rusqlite::ToSql>).collect();
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut rows = stmt.query(param_refs.as_slice())?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let root_path: String = row.get(22)?;
            let source = std::path::Path::new(&root_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Library")
                .to_string();
            // Use manual construction because source comes from col 22 (root_path)
            out.push(PhotoRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                folder_rel: row.get(3)?,
                taken_at: row.get(4)?,
                modified_at: row.get(5)?,
                media_type: row.get(6)?,
                size_bytes: row.get(7)?,
                width: row.get(8)?,
                height: row.get(9)?,
                source,
                is_favorite: row.get::<_, i32>(10).unwrap_or(0) != 0,
                is_deleted: row.get::<_, i32>(11).unwrap_or(0) != 0,
                deleted_at: row.get(12)?,
                camera_make: row.get(13)?,
                camera_model: row.get(14)?,
                lens: row.get(15)?,
                iso: row.get(16)?,
                shutter_speed: row.get(17)?,
                aperture: row.get(18)?,
                focal_length: row.get(19)?,
                gps_lat: row.get(20)?,
                gps_lon: row.get(21)?,
            });
        }
        Ok(out)
    }

    pub fn count_photos_for_library(&self, library_id: i64) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM photos WHERE library_id = ?1",
            [library_id],
            |row| row.get(0),
        )
    }

    // ── Favorites ──

    pub fn toggle_favorite(&self, photo_id: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let current: i32 = conn.query_row(
            "SELECT is_favorite FROM photos WHERE id = ?1",
            [photo_id],
            |row| row.get(0),
        )?;
        let new_val = if current == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE photos SET is_favorite = ?1 WHERE id = ?2",
            rusqlite::params![new_val, photo_id],
        )?;
        Ok(new_val != 0)
    }

    #[allow(dead_code)]
    pub fn get_favorites_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM photos WHERE is_favorite = 1 AND is_deleted = 0",
            [],
            |row| row.get(0),
        )
    }

    // ── Trash ──

    pub fn soft_delete(&self, photo_ids: &[i64]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        for id in photo_ids {
            conn.execute(
                "UPDATE photos SET is_deleted = 1, deleted_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id],
            )?;
        }
        Ok(())
    }

    pub fn restore_from_trash(&self, photo_ids: &[i64]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for id in photo_ids {
            conn.execute(
                "UPDATE photos SET is_deleted = 0, deleted_at = NULL WHERE id = ?1",
                [id],
            )?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_trash_count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM photos WHERE is_deleted = 1",
            [],
            |row| row.get(0),
        )
    }

    // ── File operations ──

    pub fn hard_delete_photos(&self, photo_ids: &[i64]) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut paths = Vec::new();
        for id in photo_ids {
            if let Ok(path) = conn.query_row(
                "SELECT path FROM photos WHERE id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            ) {
                paths.push(path);
            }
            conn.execute("DELETE FROM photo_tags WHERE photo_id = ?1", [id])?;
            conn.execute("DELETE FROM album_photos WHERE photo_id = ?1", [id])?;
            conn.execute("DELETE FROM photos WHERE id = ?1", [id])?;
        }
        Ok(paths)
    }

    pub fn rename_photo(&self, photo_id: i64, new_filename: &str) -> SqlResult<String> {
        let conn = self.conn.lock().unwrap();
        let old_path: String = conn.query_row(
            "SELECT path FROM photos WHERE id = ?1",
            [photo_id],
            |row| row.get(0),
        )?;
        let old = std::path::Path::new(&old_path);
        let new_path = old.with_file_name(new_filename).to_string_lossy().to_string();
        conn.execute(
            "UPDATE photos SET filename = ?1, path = ?2 WHERE id = ?3",
            rusqlite::params![new_filename, new_path, photo_id],
        )?;
        Ok(new_path)
    }

    // ── Tags ──

    pub fn create_tag(&self, name: &str, color: &str) -> SqlResult<TagRecord> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?1, ?2)",
            rusqlite::params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        Ok(TagRecord { id, name: name.to_string(), color: color.to_string() })
    }

    pub fn delete_tag(&self, tag_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM photo_tags WHERE tag_id = ?1", [tag_id])?;
        conn.execute("DELETE FROM tags WHERE id = ?1", [tag_id])?;
        Ok(())
    }

    pub fn get_tags(&self) -> SqlResult<Vec<TagRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(TagRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn tag_photos(&self, photo_ids: &[i64], tag_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for pid in photo_ids {
            conn.execute(
                "INSERT OR IGNORE INTO photo_tags (photo_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![pid, tag_id],
            )?;
        }
        Ok(())
    }

    pub fn untag_photos(&self, photo_ids: &[i64], tag_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for pid in photo_ids {
            conn.execute(
                "DELETE FROM photo_tags WHERE photo_id = ?1 AND tag_id = ?2",
                rusqlite::params![pid, tag_id],
            )?;
        }
        Ok(())
    }

    pub fn get_tags_for_photo(&self, photo_id: i64) -> SqlResult<Vec<TagRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color FROM tags t JOIN photo_tags pt ON pt.tag_id = t.id WHERE pt.photo_id = ?1 ORDER BY t.name"
        )?;
        let rows = stmt.query_map([photo_id], |row| {
            Ok(TagRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    // ── Albums ──

    pub fn create_album(&self, name: &str) -> SqlResult<AlbumRecord> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        conn.execute(
            "INSERT INTO albums (name, created_at) VALUES (?1, ?2)",
            rusqlite::params![name, now],
        )?;
        let id = conn.last_insert_rowid();
        Ok(AlbumRecord { id, name: name.to_string(), created_at: now, photo_count: 0, cover_path: None })
    }

    pub fn delete_album(&self, album_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM album_photos WHERE album_id = ?1", [album_id])?;
        conn.execute("DELETE FROM albums WHERE id = ?1", [album_id])?;
        Ok(())
    }

    pub fn rename_album(&self, album_id: i64, new_name: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE albums SET name = ?1 WHERE id = ?2",
            rusqlite::params![new_name, album_id],
        )?;
        Ok(())
    }

    pub fn get_albums(&self) -> SqlResult<Vec<AlbumRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.name, a.created_at,
                    (SELECT COUNT(*) FROM album_photos ap WHERE ap.album_id = a.id) as cnt,
                    (SELECT p.path FROM album_photos ap2 JOIN photos p ON p.id=ap2.photo_id WHERE ap2.album_id=a.id ORDER BY ap2.position LIMIT 1)
             FROM albums a ORDER BY a.created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AlbumRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                photo_count: row.get(3)?,
                cover_path: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn add_photos_to_album(&self, album_id: i64, photo_ids: &[i64]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let max_pos: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position),0) FROM album_photos WHERE album_id = ?1",
            [album_id],
            |row| row.get(0),
        )?;
        for (i, pid) in photo_ids.iter().enumerate() {
            conn.execute(
                "INSERT OR IGNORE INTO album_photos (album_id, photo_id, position) VALUES (?1, ?2, ?3)",
                rusqlite::params![album_id, pid, max_pos + 1 + i as i64],
            )?;
        }
        Ok(())
    }

    pub fn remove_photos_from_album(&self, album_id: i64, photo_ids: &[i64]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for pid in photo_ids {
            conn.execute(
                "DELETE FROM album_photos WHERE album_id = ?1 AND photo_id = ?2",
                rusqlite::params![album_id, pid],
            )?;
        }
        Ok(())
    }

    pub fn get_album_photos(&self, album_id: i64) -> SqlResult<Vec<PhotoRecord>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT p.{} FROM photos p JOIN album_photos ap ON ap.photo_id = p.id WHERE ap.album_id = ?1 AND p.is_deleted = 0 ORDER BY ap.position",
            Self::photo_select_cols()
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([album_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(Self::photo_from_row(row, String::new())?);
        }
        Ok(out)
    }
}
