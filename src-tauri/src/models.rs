use serde::{Deserialize, Serialize};

/// A meme entry with metadata and file location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meme {
    pub id: String,
    pub name: String,
    pub command: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub original_filename: String,
    pub ext: String,
    pub mime: String,
    pub sha256: String,
    pub stored_path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i32>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub use_count: i32,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

/// A user-defined category for organizing memes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
}
