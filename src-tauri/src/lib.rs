mod db;
mod models;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use arboard::Clipboard;
use sha2::{Digest, Sha256};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use uuid::Uuid;

use crate::db::Database;
use crate::models::{Category, Meme};

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn get_db(app: &tauri::AppHandle) -> tauri::State<'_, Database> {
    app.state::<Database>()
}

/// Returns the memes storage directory, creating it if needed.
fn memes_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir")
        .join("memes");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Maps a file extension to its MIME type.
fn mime_from_ext(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
fn hide_window(window: tauri::Window) {
    let _ = window.hide();
}

/// Reads a file and returns it as a base64 data URL for previewing in the webview.
#[tauri::command]
fn cmd_read_file_base64(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let ext = PathBuf::from(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let mime = mime_from_ext(&ext);
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
fn cmd_get_categories(app: tauri::AppHandle) -> Result<Vec<Category>, String> {
    get_db(&app)
        .get_categories()
        .map_err(|e| format!("Failed to load categories: {}", e))
}

#[tauri::command]
fn cmd_create_category(app: tauri::AppHandle, name: String) -> Result<Category, String> {
    get_db(&app)
        .create_category(&name)
        .map_err(|e| format!("Failed to create category: {}", e))
}

#[tauri::command]
fn cmd_delete_category(app: tauri::AppHandle, id: String) -> Result<(), String> {
    get_db(&app)
        .delete_category(&id)
        .map_err(|e| format!("Failed to delete category: {}", e))
}

/// Imports a new meme: copies the file to app storage, hashes for dedup, and saves metadata.
#[tauri::command]
fn cmd_import_meme(
    app: tauri::AppHandle,
    path: String,
    name: String,
    command: String,
    category_id: Option<String>,
    tags: Vec<String>,
) -> Result<Meme, String> {
    let db = get_db(&app);
    let src = PathBuf::from(&path);

    let bytes = fs::read(&src).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&bytes));

    if db.has_sha256(&hash).unwrap_or(false) {
        return Err("This file has already been imported (duplicate detected)".into());
    }

    let original_filename = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let mime = mime_from_ext(&ext).to_string();

    let id = Uuid::new_v4().to_string();
    let dest_filename = format!("{}.{}", id, ext);
    let dest = memes_dir(&app).join(&dest_filename);
    fs::copy(&src, &dest).map_err(|e| format!("Failed to copy file: {}", e))?;

    let now = now_ms();
    let meme = Meme {
        id: id.clone(),
        name,
        command,
        category_id,
        category_name: None,
        original_filename,
        ext,
        mime,
        sha256: hash,
        stored_path: dest.to_string_lossy().to_string(),
        width: None,
        height: None,
        duration_ms: None,
        created_at: now,
        updated_at: now,
        last_used_at: None,
        use_count: 0,
        is_favorite: false,
        tags: tags.clone(),
    };

    db.insert_meme(&meme)
        .map_err(|e| format!("Failed to save meme: {}", e))?;
    db.set_meme_tags(&id, &tags)
        .map_err(|e| format!("Failed to save tags: {}", e))?;

    db.get_all_memes()
        .map_err(|e| format!("Failed to reload: {}", e))?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| "Meme not found after insert".into())
}

#[tauri::command]
fn cmd_get_all_memes(app: tauri::AppHandle) -> Result<Vec<Meme>, String> {
    get_db(&app)
        .get_all_memes()
        .map_err(|e| format!("Failed to load memes: {}", e))
}

#[tauri::command]
fn cmd_update_meme(
    app: tauri::AppHandle,
    id: String,
    name: String,
    command: String,
    category_id: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    get_db(&app)
        .update_meme(&id, &name, &command, category_id.as_deref(), &tags)
        .map_err(|e| format!("Failed to update meme: {}", e))
}

/// Replaces a meme's image file: deletes old file, copies new one, updates DB metadata.
#[tauri::command]
fn cmd_replace_meme_file(
    app: tauri::AppHandle,
    id: String,
    new_path: String,
) -> Result<(), String> {
    let db = get_db(&app);
    let memes = db
        .get_all_memes()
        .map_err(|e| format!("Failed to load memes: {}", e))?;
    let meme = memes
        .iter()
        .find(|m| m.id == id)
        .ok_or("Meme not found")?;

    let src = PathBuf::from(&new_path);
    let bytes = fs::read(&src).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&bytes));

    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let mime = mime_from_ext(&ext);
    let original_filename = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let _ = fs::remove_file(&meme.stored_path);

    let dest_filename = format!("{}.{}", id, ext);
    let dest = memes_dir(&app).join(&dest_filename);
    fs::copy(&src, &dest).map_err(|e| format!("Failed to copy file: {}", e))?;

    db.replace_meme_file(&id, &original_filename, &ext, mime, &hash, &dest.to_string_lossy(), now_ms())
        .map_err(|e| format!("Failed to update meme file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn cmd_get_recently_used(app: tauri::AppHandle, limit: Option<usize>) -> Result<Vec<Meme>, String> {
    get_db(&app)
        .get_recently_used(limit.unwrap_or(10))
        .map_err(|e| format!("Failed to load recently used: {}", e))
}

#[tauri::command]
fn cmd_delete_meme(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let db = get_db(&app);
    if let Ok(Some(path)) = db.delete_meme(&id) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

/// Copies a meme's image to the system clipboard and bumps usage stats.
/// Static images are copied as RGBA data; GIFs/videos copy the file path.
#[tauri::command]
fn cmd_copy_to_clipboard(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let db = get_db(&app);
    let memes = db
        .get_all_memes()
        .map_err(|e| format!("Failed to load meme: {}", e))?;
    let meme = memes
        .iter()
        .find(|m| m.id == id)
        .ok_or("Meme not found")?;

    let bytes = fs::read(&meme.stored_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let _ = db.bump_usage(&id);

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    if meme.mime.starts_with("image/") && meme.ext != "gif" {
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("Failed to decode image: {}", e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let img_data = arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        };
        clipboard
            .set_image(img_data)
            .map_err(|e| format!("Failed to copy image: {}", e))?;
    } else {
        clipboard
            .set_text(&meme.stored_path)
            .map_err(|e| format!("Failed to copy path: {}", e))?;
    }

    Ok(())
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            hide_window,
            cmd_read_file_base64,
            cmd_get_categories,
            cmd_create_category,
            cmd_delete_category,
            cmd_import_meme,
            cmd_get_all_memes,
            cmd_update_meme,
            cmd_delete_meme,
            cmd_replace_meme_file,
            cmd_get_recently_used,
            cmd_copy_to_clipboard,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("db.sqlite");
            let db = Database::new(&db_path)
                .map_err(|e| format!("Failed to init database: {}", e))?;
            app.manage(db);

            fs::create_dir_all(data_dir.join("memes")).ok();

            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::KeyM,
            );
            let _ = app.global_shortcut().unregister(shortcut);
            app.global_shortcut().register(shortcut)?;

            let toggle =
                MenuItemBuilder::with_id("toggle", "Toggle (Ctrl+Shift+M)").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &quit])
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("PasteGoblin")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => toggle_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
