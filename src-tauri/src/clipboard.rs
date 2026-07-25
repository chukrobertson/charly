use arboard::Clipboard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::db::Database;

#[derive(Clone, serde::Serialize)]
pub struct ClipboardEvent {
    pub id: String,
    pub content_type: String,
    pub content: Option<String>,
    pub file_name: Option<String>,
    pub created_at: String,
    pub x: f64,
    pub y: f64,
    pub width: i32,
    pub height: i32,
    pub color: String,
    pub z_index: i32,
    pub content_ref: Option<String>,
}

pub fn start_clipboard_watcher(
    app: AppHandle,
    db: Arc<Database>,
    running: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to open clipboard: {}", e);
                return;
            }
        };

        let mut last_text = String::new();
        let mut last_image_hash: u64 = 0;

        while running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(500));

            let blobs_dir = db.get_blobs_dir(&app);

            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() && text != last_text {
                    last_text = text.clone();

                    let is_file_uri = text.starts_with("file://");
                    let mut files: Vec<String> = Vec::new();

                    if is_file_uri {
                        for line in text.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("file://") {
                                let path = trimmed
                                    .strip_prefix("file://")
                                    .unwrap_or(trimmed);
                                let path = urlencoding_decode(path);
                                if std::path::Path::new(&path).exists() {
                                    files.push(path);
                                }
                            }
                        }
                    }

                    if !files.is_empty() {
                        for file_path in files {
                            let fname = std::path::Path::new(&file_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "file".to_string());
                            let id = Uuid::new_v4().to_string();
                            let now = timestamp_now();
                            let note = crate::db::Note {
                                id: id.clone(),
                                x: 200.0 + rand_offset(),
                                y: 200.0 + rand_offset(),
                                width: 220,
                                height: 120,
                                color: random_color(),
                                content_type: "file".to_string(),
                                content_ref: Some(file_path.clone()),
                                file_name: Some(fname.clone()),
                                created_at: now.clone(),
                                z_index: 0,
                            };

                            let _ = db.add_note(note.clone());
                            let _ = app.emit("clipboard-capture", ClipboardEvent {
                                id,
                                content_type: "file".to_string(),
                                content: Some(file_path),
                                file_name: Some(fname),
                                created_at: now,
                                x: note.x,
                                y: note.y,
                                width: note.width,
                                height: note.height,
                                color: note.color,
                                z_index: note.z_index,
                                content_ref: note.content_ref,
                            });
                        }
                    } else {
                        let id = Uuid::new_v4().to_string();
                        let now = timestamp_now();
                        let note = crate::db::Note {
                            id: id.clone(),
                            x: 200.0 + rand_offset(),
                            y: 200.0 + rand_offset(),
                            width: 220,
                            height: 180,
                            color: random_color(),
                            content_type: "text".to_string(),
                            content_ref: None,
                            file_name: None,
                            created_at: now.clone(),
                            z_index: 0,
                        };

                        let _ = db.add_note(note.clone());
                        let _ = app.emit("clipboard-capture", ClipboardEvent {
                            id,
                            content_type: "text".to_string(),
                            content: Some(text),
                            file_name: None,
                            created_at: now,
                            x: note.x,
                            y: note.y,
                            width: note.width,
                            height: note.height,
                            color: note.color,
                            z_index: note.z_index,
                            content_ref: None,
                        });
                    }
                }
            }

            if let Ok(image) = clipboard.get_image() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                image.bytes.hash(&mut hasher);
                let hash = hasher.finish();

                if hash != last_image_hash && last_image_hash != 0 {
                    last_image_hash = hash;

                    let img = match image::load_from_memory(&image.bytes) {
                        Ok(img) => img,
                        Err(_) => continue,
                    };

                    let file_name = format!("{}.png", Uuid::new_v4());
                    let file_path = blobs_dir.join(&file_name);
                    if img.save(&file_path).is_ok() {
                        let now = timestamp_now();
                        let id = Uuid::new_v4().to_string();
                        let note = crate::db::Note {
                            id: id.clone(),
                            x: 200.0 + rand_offset(),
                            y: 200.0 + rand_offset(),
                            width: img.width() as i32 + 40,
                            height: img.height() as i32 + 60,
                            color: random_color(),
                            content_type: "image".to_string(),
                            content_ref: Some(file_path.to_string_lossy().to_string()),
                            file_name: Some(file_name),
                            created_at: now.clone(),
                            z_index: 0,
                        };

                        let _ = db.add_note(note.clone());
                        let _ = app.emit("clipboard-capture", ClipboardEvent {
                            id,
                            content_type: "image".to_string(),
                            content: None,
                            file_name: Some(note.file_name.clone().unwrap_or_default()),
                            created_at: now,
                            x: note.x,
                            y: note.y,
                            width: note.width,
                            height: note.height,
                            color: note.color,
                            z_index: note.z_index,
                            content_ref: note.content_ref,
                        });
                    }
                } else if last_image_hash == 0 {
                    last_image_hash = hash;
                }
            }
        }
    });
}

fn rand_offset() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 200) as f64
}

fn random_color() -> String {
    let colors = [
        "#FEF08A", "#FECACA", "#BFDBFE", "#BBF7D0",
        "#DDD6FE", "#FED7AA", "#FBCFE8", "#A5F3FC",
    ];
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    colors[(nanos as usize) % colors.len()].to_string()
}

fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let total_secs = dur.as_secs() as i64;
    let days = total_secs / 86400;
    let time_secs = total_secs % 86400;

    let era: i64 = 719468;
    let doe = days + era;
    let y = (400 * doe) / 146097;
    let mut yoe = doe - y * 365 - y / 4 + y / 100 - y / 400;
    if yoe < 0 {
        let leap = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 1 } else { 0 };
        yoe += 365 + leap;
    }
    let doy = yoe;
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    let h = time_secs / 3600;
    let min = (time_secs % 3600) / 60;
    let s = time_secs % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", year, m, d, h, min, s)
}

fn urlencoding_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
