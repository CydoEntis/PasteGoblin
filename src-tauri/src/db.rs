use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

use crate::models::{Category, Meme};

/// Thread-safe SQLite database wrapper for meme storage.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens the database at the given path and runs migrations.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS memes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command TEXT UNIQUE NOT NULL,
                category_id TEXT REFERENCES categories(id) ON DELETE SET NULL,
                original_filename TEXT NOT NULL,
                ext TEXT NOT NULL,
                mime TEXT NOT NULL,
                sha256 TEXT UNIQUE NOT NULL,
                stored_path TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                duration_ms INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_used_at INTEGER,
                use_count INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS meme_tags (
                meme_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (meme_id, tag_id),
                FOREIGN KEY (meme_id) REFERENCES memes(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            ",
        )?;
        Ok(())
    }

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, sort_order FROM categories ORDER BY sort_order ASC",
        )?;
        let cats = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(cats)
    }

    pub fn create_category(&self, name: &str) -> Result<Category> {
        let conn = self.conn.lock().unwrap();
        let max_order: i32 = conn
            .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM categories", [], |row| row.get(0))?;
        let id = Uuid::new_v4().to_string();
        let sort_order = max_order + 1;
        conn.execute(
            "INSERT INTO categories (id, name, sort_order) VALUES (?1, ?2, ?3)",
            params![id, name, sort_order],
        )?;
        Ok(Category {
            id,
            name: name.to_string(),
            sort_order,
        })
    }

    pub fn delete_category(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_or_create_tag(&self, conn: &Connection, name: &str) -> Result<String> {
        if let Ok(id) = conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![name],
            |row| row.get::<_, String>(0),
        ) {
            return Ok(id);
        }
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tags (id, name) VALUES (?1, ?2)",
            params![id, name],
        )?;
        Ok(id)
    }

    pub fn set_meme_tags(&self, meme_id: &str, tags: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM meme_tags WHERE meme_id = ?1", params![meme_id])?;
        for tag_name in tags {
            let trimmed = tag_name.trim();
            if trimmed.is_empty() {
                continue;
            }
            let tag_id = self.get_or_create_tag(&conn, trimmed)?;
            conn.execute(
                "INSERT OR IGNORE INTO meme_tags (meme_id, tag_id) VALUES (?1, ?2)",
                params![meme_id, tag_id],
            )?;
        }
        Ok(())
    }

    fn get_tags_for_meme_inner(&self, conn: &Connection, meme_id: &str) -> Result<Vec<String>> {
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t JOIN meme_tags mt ON t.id = mt.tag_id WHERE mt.meme_id = ?1 ORDER BY t.name",
        )?;
        let tags = stmt
            .query_map(params![meme_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>>>()?;
        Ok(tags)
    }

    /// Maps a database row to a Meme struct, including its tags.
    fn row_to_meme(&self, conn: &Connection, row: &rusqlite::Row) -> rusqlite::Result<Meme> {
        let id: String = row.get(0)?;
        let tags = self.get_tags_for_meme_inner(conn, &id).unwrap_or_default();
        Ok(Meme {
            id,
            name: row.get(1)?,
            command: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            original_filename: row.get(5)?,
            ext: row.get(6)?,
            mime: row.get(7)?,
            sha256: row.get(8)?,
            stored_path: row.get(9)?,
            width: row.get(10)?,
            height: row.get(11)?,
            duration_ms: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
            last_used_at: row.get(15)?,
            use_count: row.get(16)?,
            is_favorite: row.get::<_, i32>(17)? != 0,
            tags,
        })
    }

    pub fn insert_meme(&self, meme: &Meme) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memes (id, name, command, category_id, original_filename, ext, mime, sha256, stored_path, width, height, duration_ms, created_at, updated_at, last_used_at, use_count, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                meme.id,
                meme.name,
                meme.command,
                meme.category_id,
                meme.original_filename,
                meme.ext,
                meme.mime,
                meme.sha256,
                meme.stored_path,
                meme.width,
                meme.height,
                meme.duration_ms,
                meme.created_at,
                meme.updated_at,
                meme.last_used_at,
                meme.use_count,
                meme.is_favorite as i32,
            ],
        )?;
        Ok(())
    }

    /// Returns all memes ordered by creation date (newest first).
    pub fn get_all_memes(&self) -> Result<Vec<Meme>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.name, m.command, m.category_id, c.name, m.original_filename, m.ext, m.mime, m.sha256, m.stored_path, m.width, m.height, m.duration_ms, m.created_at, m.updated_at, m.last_used_at, m.use_count, m.is_favorite
             FROM memes m LEFT JOIN categories c ON m.category_id = c.id
             ORDER BY m.created_at DESC",
        )?;
        let mut memes = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            memes.push(self.row_to_meme(&conn, row)?);
        }
        Ok(memes)
    }

    pub fn update_meme(
        &self,
        id: &str,
        name: &str,
        command: &str,
        category_id: Option<&str>,
        tags: &[String],
    ) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            conn.execute(
                "UPDATE memes SET name = ?1, command = ?2, category_id = ?3, updated_at = ?4 WHERE id = ?5",
                params![name, command, category_id, now, id],
            )?;
        }
        self.set_meme_tags(id, tags)?;
        Ok(())
    }

    /// Replaces the stored file for a meme, updating all file-related metadata.
    pub fn replace_meme_file(
        &self,
        id: &str,
        original_filename: &str,
        ext: &str,
        mime: &str,
        sha256: &str,
        stored_path: &str,
        updated_at: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memes SET original_filename = ?1, ext = ?2, mime = ?3, sha256 = ?4, stored_path = ?5, updated_at = ?6 WHERE id = ?7",
            params![original_filename, ext, mime, sha256, stored_path, updated_at, id],
        )?;
        Ok(())
    }

    /// Deletes a meme and returns its stored file path for cleanup.
    pub fn delete_meme(&self, id: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let path: Option<String> = conn
            .query_row(
                "SELECT stored_path FROM memes WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok();
        conn.execute("DELETE FROM memes WHERE id = ?1", params![id])?;
        Ok(path)
    }

    /// Increments use_count and updates last_used_at for clipboard tracking.
    pub fn bump_usage(&self, meme_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        conn.execute(
            "UPDATE memes SET use_count = use_count + 1, last_used_at = ?1 WHERE id = ?2",
            params![now, meme_id],
        )?;
        Ok(())
    }

    /// Returns the most recently used memes, up to `limit`.
    pub fn get_recently_used(&self, limit: usize) -> Result<Vec<Meme>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.name, m.command, m.category_id, c.name, m.original_filename, m.ext, m.mime, m.sha256, m.stored_path, m.width, m.height, m.duration_ms, m.created_at, m.updated_at, m.last_used_at, m.use_count, m.is_favorite
             FROM memes m LEFT JOIN categories c ON m.category_id = c.id
             WHERE m.last_used_at IS NOT NULL
             ORDER BY m.last_used_at DESC
             LIMIT ?1",
        )?;
        let mut memes = Vec::new();
        let mut rows = stmt.query(params![limit as i64])?;
        while let Some(row) = rows.next()? {
            memes.push(self.row_to_meme(&conn, row)?);
        }
        Ok(memes)
    }

    /// Checks if a file with the given SHA-256 hash already exists.
    pub fn has_sha256(&self, hash: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM memes WHERE sha256 = ?1",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
