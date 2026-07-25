mod clipboard;
mod db;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::Manager;

use clipboard::start_clipboard_watcher;
use db::{ConnData, Database, Note};

#[tauri::command]
fn get_notes(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Note>, String> {
    db.get_notes()
}

#[tauri::command]
fn add_note(note: Note, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.add_note(note)
}

#[tauri::command]
fn update_note(
    id: String,
    x: f64,
    y: f64,
    width: i32,
    height: i32,
    z_index: i32,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.update_note(&id, x, y, width, height, z_index)
}

#[tauri::command]
fn update_note_color(
    id: String,
    color: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.update_color(&id, &color)
}

#[tauri::command]
fn delete_note(
    id: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.delete_note(&id)
}

#[tauri::command]
fn get_blobs_dir_path(
    app: tauri::AppHandle,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<String, String> {
    Ok(db.get_blobs_dir(&app).to_string_lossy().to_string())
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_in_file_manager(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let dir = if p.is_dir() { p } else { p.parent().unwrap_or(p) };
    open::that(dir).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_connections(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ConnData>, String> {
    db.get_connections()
}

#[tauri::command]
fn add_connection(
    conn: ConnData,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.add_connection(conn)
}

#[tauri::command]
fn delete_connection(
    id: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.delete_connection(&id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let clipboard_running = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let db = Arc::new(Database::new(app.handle()).expect("Failed to init database"));
            app.manage(db.clone());

            let cl_running = clipboard_running.clone();
            let cl_app = app.handle().clone();
            start_clipboard_watcher(cl_app, db, cl_running);
            clipboard_running.store(true, std::sync::atomic::Ordering::Relaxed);

            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&tauri::menu::MenuItemBuilder::with_id("show", "Show").build(app)?)
                .separator()
                .item(&tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?)
                .build()?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Charly — The connected notes app")
                .show_menu_on_left_click(false)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_notes,
            add_note,
            update_note,
            update_note_color,
            delete_note,
            get_blobs_dir_path,
            open_in_file_manager,
            copy_to_clipboard,
            get_connections,
            add_connection,
            delete_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
