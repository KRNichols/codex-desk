//! Practical IL5 identity bind for a desk: machine-bound unlock plus an
//! optional operator record. Desk does **not** own in-app write permissions.
//! Workspace-write is YOLO whenever a workspace path is set.
//!
//! CAC/PIV is not shipped. Windows Hello hardware prompt is not invoked here;
//! the bind is the OS user session (USERNAME / USERPROFILE). See SECURITY.md.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorAttestation {
    pub configured: bool,
    pub operator_name: Option<String>,
    pub organization: Option<String>,
    pub statement: Option<String>,
    pub at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityStatus {
    pub session_user: String,
    pub machine_id_present: bool,
    pub machine_bound: bool,
    pub machine_binding_ok: bool,
    pub key_backend: String,
    pub store_encrypted: bool,
    pub audit_chain_ok: bool,
    pub operator_attestation: OperatorAttestation,
    pub pat_slot: String,
    pub hello_bind: String,
}

pub fn session_user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".into())
}

pub fn machine_id() -> String {
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".into())
}

pub fn machine_binding() -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"codex-desk-bind-v1");
    hasher.update(machine_id().as_bytes());
    hasher.update(&[0u8]);
    hasher.update(session_user().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hello_bind_label() -> &'static str {
    if cfg!(windows) {
        "windows-user-session"
    } else {
        "posix-user-session"
    }
}

pub fn migrate_identity(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS identity_state (
            id TEXT PRIMARY KEY,
            machine_binding TEXT NOT NULL,
            session_user TEXT NOT NULL,
            attestation_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| format!("migrate identity: {e}"))?;
    ensure_row(conn)
}

fn ensure_row(conn: &Connection) -> Result<(), String> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM identity_state WHERE id = 'local'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if n > 0 {
        return Ok(());
    }
    let now = Utc::now().to_rfc3339();
    let empty = serde_json::to_string(&OperatorAttestation {
        configured: false,
        operator_name: None,
        organization: None,
        statement: None,
        at: None,
    })
    .unwrap_or_else(|_| "{}".into());
    conn.execute(
        "INSERT INTO identity_state (id, machine_binding, session_user, attestation_json, created_at, updated_at)
         VALUES ('local', ?1, ?2, ?3, ?4, ?5)",
        params![machine_binding(), session_user(), empty, now, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_attestation(conn: &Connection) -> Result<OperatorAttestation, String> {
    ensure_row(conn)?;
    let json: String = conn
        .query_row(
            "SELECT attestation_json FROM identity_state WHERE id = 'local'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

pub fn stored_binding(conn: &Connection) -> Result<String, String> {
    ensure_row(conn)?;
    conn.query_row(
        "SELECT machine_binding FROM identity_state WHERE id = 'local'",
        [],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

pub fn binding_matches(conn: &Connection) -> Result<bool, String> {
    Ok(stored_binding(conn)? == machine_binding())
}

pub fn set_attestation(
    conn: &Connection,
    operator_name: &str,
    organization: &str,
    statement: &str,
) -> Result<OperatorAttestation, String> {
    let name = operator_name.trim();
    let org = organization.trim();
    let stmt = statement.trim();
    if name.is_empty() || org.is_empty() {
        return Err("Operator name and organization are required.".into());
    }
    if stmt.len() < 12 {
        return Err("Attestation statement is too short.".into());
    }
    let record = OperatorAttestation {
        configured: true,
        operator_name: Some(name.to_string()),
        organization: Some(org.to_string()),
        statement: Some(stmt.to_string()),
        at: Some(Utc::now().to_rfc3339()),
    };
    let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE identity_state SET attestation_json = ?1, session_user = ?2, updated_at = ?3 WHERE id = 'local'",
        params![json, session_user(), Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;
    Ok(record)
}

/// Always-on YOLO: attestation is not a write gate. Kept so leftover callers
/// cannot HOLD hill-climb or workspace-write.
pub fn require_write_attestation(_conn: &Connection) -> Result<(), String> {
    Ok(())
}

/// Workspace-write is on whenever the operator set a non-empty workspace path.
/// Home directory refusal stays in the Codex runner. Desk never auto-pushes.
pub fn yolo_writes_enabled(workspace_path: Option<&str>) -> bool {
    workspace_path
        .map(|p| !p.trim().is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_is_stable_in_process() {
        assert_eq!(machine_binding(), machine_binding());
        assert_eq!(machine_binding().len(), 64);
    }

    #[test]
    fn write_attestation_never_holds() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        migrate_identity(&conn).unwrap();
        assert!(require_write_attestation(&conn).is_ok());
        let att = load_attestation(&conn).unwrap();
        assert!(!att.configured);
    }

    #[test]
    fn yolo_writes_follow_workspace() {
        assert!(!yolo_writes_enabled(None));
        assert!(!yolo_writes_enabled(Some("")));
        assert!(!yolo_writes_enabled(Some("   ")));
        assert!(yolo_writes_enabled(Some("/workspace")));
    }
}
