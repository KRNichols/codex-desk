use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub id: String,
    pub at: String,
    pub action: String,
    pub actor: String,
    pub entity_type: String,
    pub entity_id: String,
    pub detail: String,
    pub prev_hash: String,
    pub event_hash: String,
}

pub fn migrate_audit(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY,
            at TEXT NOT NULL,
            action TEXT NOT NULL,
            actor TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            detail TEXT NOT NULL,
            prev_hash TEXT NOT NULL DEFAULT '',
            event_hash TEXT NOT NULL DEFAULT ''
        );
        ",
    )
    .map_err(|e| format!("migrate audit: {e}"))?;
    let _ = conn.execute(
        "ALTER TABLE audit_events ADD COLUMN prev_hash TEXT NOT NULL DEFAULT ''",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audit_events ADD COLUMN event_hash TEXT NOT NULL DEFAULT ''",
        [],
    );
    backfill_hashes(conn)
}

fn backfill_hashes(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, at, action, actor, entity_type, entity_id, detail, prev_hash, event_hash
             FROM audit_events ORDER BY at ASC, rowid ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut prev = GENESIS.to_string();
    for row in rows {
        let (id, at, action, actor, entity_type, entity_id, detail, _stored_prev, stored_hash) =
            row.map_err(|e| e.to_string())?;
        if !stored_hash.is_empty() && stored_hash.len() == 64 {
            prev = stored_hash;
            continue;
        }
        let hash = event_hash(&prev, &id, &at, &action, &actor, &entity_type, &entity_id, &detail);
        conn.execute(
            "UPDATE audit_events SET prev_hash = ?1, event_hash = ?2 WHERE id = ?3",
            params![prev, hash, id],
        )
        .map_err(|e| e.to_string())?;
        prev = hash;
    }
    Ok(())
}

pub fn event_hash(
    prev: &str,
    id: &str,
    at: &str,
    action: &str,
    actor: &str,
    entity_type: &str,
    entity_id: &str,
    detail: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(id.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(at.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(action.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(actor.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(entity_type.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(entity_id.as_bytes());
    hasher.update(&[0u8]);
    hasher.update(detail.as_bytes());
    hex::encode(hasher.finalize())
}

fn last_hash(conn: &Connection) -> String {
    conn.query_row(
        "SELECT event_hash FROM audit_events WHERE event_hash != '' ORDER BY at DESC, rowid DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| GENESIS.to_string())
}

pub fn write(
    conn: &Connection,
    _app_data: &std::path::Path,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    detail: &str,
) {
    let event_id = Uuid::new_v4().to_string();
    let at = Utc::now().to_rfc3339();
    let actor = format!("local-user:{}", crate::identity::session_user());
    let detail = redact_detail(detail);
    let prev = last_hash(conn);
    let hash = event_hash(
        &prev,
        &event_id,
        &at,
        action,
        &actor,
        entity_type,
        entity_id,
        &detail,
    );
    let _ = conn.execute(
        "INSERT INTO audit_events (id, at, action, actor, entity_type, entity_id, detail, prev_hash, event_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            event_id, at, action, actor, entity_type, entity_id, detail, prev, hash
        ],
    );
}

pub fn export_json(conn: &Connection) -> Result<String, String> {
    let events = list_recent(conn, 10_000)?;
    serde_json::to_string_pretty(&events).map_err(|e| format!("audit export: {e}"))
}

pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<AuditEvent>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, at, action, actor, entity_type, entity_id, detail, prev_hash, event_hash
             FROM audit_events ORDER BY at DESC, rowid DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(AuditEvent {
                id: row.get(0)?,
                at: row.get(1)?,
                action: row.get(2)?,
                actor: row.get(3)?,
                entity_type: row.get(4)?,
                entity_id: row.get(5)?,
                detail: row.get(6)?,
                prev_hash: row.get(7)?,
                event_hash: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn chain_ok(conn: &Connection) -> bool {
    let mut stmt = match conn.prepare(
        "SELECT id, at, action, actor, entity_type, entity_id, detail, prev_hash, event_hash
         FROM audit_events ORDER BY at ASC, rowid ASC",
    ) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let rows = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
        ))
    }) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let mut prev = GENESIS.to_string();
    for row in rows {
        let Ok((id, at, action, actor, entity_type, entity_id, detail, stored_prev, stored_hash)) =
            row
        else {
            return false;
        };
        if stored_hash.is_empty() {
            return false;
        }
        if stored_prev != prev {
            return false;
        }
        let expected = event_hash(
            &stored_prev,
            &id,
            &at,
            &action,
            &actor,
            &entity_type,
            &entity_id,
            &detail,
        );
        if expected != stored_hash {
            return false;
        }
        prev = stored_hash;
    }
    true
}

pub fn redact_detail(detail: &str) -> String {
    detail
        .lines()
        .map(|line| {
            let upper = line.to_ascii_uppercase();
            if (upper.contains("PAT")
                || upper.contains("API_KEY")
                || upper.contains("TOKEN")
                || upper.contains("SECRET")
                || upper.contains("BEARER "))
                && (line.contains('=') || line.contains(':'))
            {
                "[redacted]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_links() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_audit(&conn).unwrap();
        write(&conn, std::path::Path::new("."), "agent.create", "agent", "a1", "desk");
        write(&conn, std::path::Path::new("."), "hillclimb.start", "run", "r1", "writes=false");
        assert!(chain_ok(&conn));
        let events = list_recent(&conn, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_ne!(events[0].event_hash, events[1].event_hash);
    }

    #[test]
    fn redacts_pat_lines() {
        assert_eq!(redact_detail("AZURE_LLM_PAT=supersecret"), "[redacted]");
    }
}
