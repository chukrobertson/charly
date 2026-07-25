use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnData {
    pub id: String,
    pub from_note_id: String,
    pub to_note_id: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: i32,
    pub height: i32,
    pub color: String,
    pub content_type: String,
    pub content_ref: Option<String>,
    pub file_name: Option<String>,
    pub created_at: String,
    pub z_index: i32,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        std::fs::create_dir_all(&app_dir)?;

        let blobs_dir = app_dir.join("blobs");
        std::fs::create_dir_all(&blobs_dir)?;

        let db_path = app_dir.join("cliffnote.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                x REAL NOT NULL DEFAULT 0,
                y REAL NOT NULL DEFAULT 0,
                width INTEGER NOT NULL DEFAULT 200,
                height INTEGER NOT NULL DEFAULT 200,
                color TEXT NOT NULL DEFAULT '#FEF08A',
                content_type TEXT NOT NULL CHECK(content_type IN ('text', 'image', 'file')),
                content_ref TEXT,
                file_name TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                z_index INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                from_note_id TEXT NOT NULL,
                to_note_id TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#FF6B6B',
                FOREIGN KEY (from_note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (to_note_id) REFERENCES notes(id) ON DELETE CASCADE
            );",
        )?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_notes(&self) -> Result<Vec<Note>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, x, y, width, height, color, content_type, content_ref, file_name, created_at, z_index FROM notes ORDER BY z_index ASC, created_at DESC")
            .map_err(|e| e.to_string())?;

        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    x: row.get(1)?,
                    y: row.get(2)?,
                    width: row.get(3)?,
                    height: row.get(4)?,
                    color: row.get(5)?,
                    content_type: row.get(6)?,
                    content_ref: row.get(7)?,
                    file_name: row.get(8)?,
                    created_at: row.get(9)?,
                    z_index: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<Note>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(notes)
    }

    pub fn add_note(&self, note: Note) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO notes (id, x, y, width, height, color, content_type, content_ref, file_name, created_at, z_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                note.id,
                note.x,
                note.y,
                note.width,
                note.height,
                note.color,
                note.content_type,
                note.content_ref,
                note.file_name,
                note.created_at,
                note.z_index,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_note(
        &self,
        id: &str,
        x: f64,
        y: f64,
        width: i32,
        height: i32,
        z_index: i32,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE notes SET x = ?1, y = ?2, width = ?3, height = ?4, z_index = ?5 WHERE id = ?6",
            params![x, y, width, height, z_index, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_color(&self, id: &str, color: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE notes SET color = ?1 WHERE id = ?2",
            params![color, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_note(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let note: Option<Note> = conn
            .query_row(
                "SELECT id, x, y, width, height, color, content_type, content_ref, file_name, created_at, z_index FROM notes WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Note {
                        id: row.get(0)?,
                        x: row.get(1)?,
                        y: row.get(2)?,
                        width: row.get(3)?,
                        height: row.get(4)?,
                        color: row.get(5)?,
                        content_type: row.get(6)?,
                        content_ref: row.get(7)?,
                        file_name: row.get(8)?,
                        created_at: row.get(9)?,
                        z_index: row.get(10)?,
                    })
                },
            )
            .ok();

        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if let Some(n) = note {
            if let Some(ref path) = n.content_ref {
                let _ = std::fs::remove_file(path);
            }
        }

        Ok(())
    }

    pub fn get_blobs_dir(&self, app_handle: &AppHandle) -> std::path::PathBuf {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        app_dir.join("blobs")
    }

    pub fn get_connections(&self) -> Result<Vec<ConnData>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, from_note_id, to_note_id, color FROM connections")
            .map_err(|e| e.to_string())?;

        let connections = stmt
            .query_map([], |row| {
                Ok(ConnData {
                    id: row.get(0)?,
                    from_note_id: row.get(1)?,
                    to_note_id: row.get(2)?,
                    color: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<ConnData>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(connections)
    }

    pub fn add_connection(&self, conn_data: ConnData) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO connections (id, from_note_id, to_note_id, color)
             VALUES (?1, ?2, ?3, ?4)",
            params![conn_data.id, conn_data.from_note_id, conn_data.to_note_id, conn_data.color],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_connection(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM connections WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
