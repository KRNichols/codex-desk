use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub codex_thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub status: String,
}

pub fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            codex_thread_id TEXT
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            status TEXT NOT NULL,
            FOREIGN KEY(chat_id) REFERENCES chats(id) ON DELETE CASCADE
        );
        ",
    )
    .map_err(|e| format!("migrate sqlite: {e}"))?;
    crate::agents::migrate_agents(conn)?;
    crate::audit::migrate_audit(conn)?;
    crate::identity::migrate_identity(conn)?;
    crate::harness::migrate_harness(conn)?;
    Ok(())
}

#[allow(dead_code)]
pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create data dir: {e}"))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("open sqlite: {e}"))?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn list_chats(conn: &Connection) -> Result<Vec<Chat>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, created_at, updated_at, codex_thread_id
             FROM chats ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Chat {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                codex_thread_id: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_chat(conn: &Connection, id: &str) -> Result<Option<Chat>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, created_at, updated_at, codex_thread_id
             FROM chats WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(Chat {
            id: row.get(0).map_err(|e| e.to_string())?,
            title: row.get(1).map_err(|e| e.to_string())?,
            created_at: row.get(2).map_err(|e| e.to_string())?,
            updated_at: row.get(3).map_err(|e| e.to_string())?,
            codex_thread_id: row.get(4).map_err(|e| e.to_string())?,
        }))
    } else {
        Ok(None)
    }
}

pub fn create_chat(conn: &Connection, title: &str) -> Result<Chat, String> {
    let now = Utc::now().to_rfc3339();
    let chat = Chat {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        created_at: now.clone(),
        updated_at: now,
        codex_thread_id: None,
    };
    conn.execute(
        "INSERT INTO chats (id, title, created_at, updated_at, codex_thread_id)
         VALUES (?1, ?2, ?3, ?4, NULL)",
        params![chat.id, chat.title, chat.created_at, chat.updated_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(chat)
}

pub fn delete_chat(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM messages WHERE chat_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM chats WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_thread_id(conn: &Connection, chat_id: &str, thread_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE chats SET codex_thread_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![thread_id, Utc::now().to_rfc3339(), chat_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn touch_title_if_default(conn: &Connection, chat_id: &str, first_line: &str) -> Result<(), String> {
    let chat = get_chat(conn, chat_id)?;
    let Some(chat) = chat else {
        return Ok(());
    };
    if chat.title != "New chat" {
        return Ok(());
    }
    let trimmed = first_line.trim().replace('\n', " ");
    let title: String = trimmed.chars().take(48).collect();
    if title.is_empty() {
        return Ok(());
    }
    conn.execute(
        "UPDATE chats SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, Utc::now().to_rfc3339(), chat_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_messages(conn: &Connection, chat_id: &str) -> Result<Vec<Message>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, chat_id, role, content, created_at, status
             FROM messages WHERE chat_id = ?1 ORDER BY created_at ASC, rowid ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![chat_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn add_message(
    conn: &Connection,
    chat_id: &str,
    role: &str,
    content: &str,
    status: &str,
) -> Result<Message, String> {
    let message = Message {
        id: Uuid::new_v4().to_string(),
        chat_id: chat_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at: Utc::now().to_rfc3339(),
        status: status.to_string(),
    };
    conn.execute(
        "INSERT INTO messages (id, chat_id, role, content, created_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            message.id,
            message.chat_id,
            message.role,
            message.content,
            message.created_at,
            message.status
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        params![message.created_at, chat_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(message)
}

pub fn update_message(
    conn: &Connection,
    id: &str,
    content: &str,
    status: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE messages SET content = ?1, status = ?2 WHERE id = ?3",
        params![content, status, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
