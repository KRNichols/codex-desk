//! Encrypted-at-rest SQLite vault.
//!
//! Work happens in an in-memory SQLite. After mutations the serialized
//! database is AES-256-GCM sealed to `codex-desk.db.enc` with the OS-backed DEK.
//! Legacy plaintext `codex-desk.db` is migrated once and deleted.

use crate::crypto;
use crate::keystore::{self, KeyBackend};
use rusqlite::{backup::Backup, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Vault {
    conn: Connection,
    dek: crypto::Dek,
    enc_path: PathBuf,
    pub backend: KeyBackend,
    pub encrypted_on_disk: bool,
}

impl std::ops::Deref for Vault {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.conn
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        let _ = self.persist();
        self.dek = [0u8; 32];
    }
}

impl Vault {
    pub fn open(app_data: &Path) -> Result<Self, String> {
        fs::create_dir_all(app_data).map_err(|e| format!("create data dir: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(app_data, fs::Permissions::from_mode(0o700));
        }

        let unlocked = keystore::load_or_create_dek(app_data).map_err(|e| {
            write_unlock_failure(app_data, &e);
            format!("encryption key unlock failed: {e}")
        })?;

        let enc_path = app_data.join("codex-desk.db.enc");
        let legacy = app_data.join("codex-desk.db");
        let mut conn = Connection::open_in_memory().map_err(|e| format!("open memory sqlite: {e}"))?;

        if enc_path.is_file() {
            let blob = fs::read(&enc_path).map_err(|e| format!("read encrypted store: {e}"))?;
            let plain = crypto::open(&unlocked.dek, &blob).map_err(|e| {
                write_unlock_failure(app_data, &e);
                format!("encryption key unlock failed: {e}")
            })?;
            load_sqlite_bytes(&mut conn, &plain, app_data)?;
        } else if legacy.is_file() {
            let src = Connection::open(&legacy).map_err(|e| format!("open legacy sqlite: {e}"))?;
            copy_db(&src, &mut conn)?;
        }

        crate::store::migrate(&conn)?;

        let mut vault = Vault {
            conn,
            dek: unlocked.dek,
            enc_path,
            backend: unlocked.backend,
            encrypted_on_disk: false,
        };
        vault.persist()?;
        if legacy.is_file() {
            overwrite_and_remove(&legacy);
        }
        Ok(vault)
    }

    pub fn persist(&mut self) -> Result<(), String> {
        let tmp = self.enc_path.with_extension("sqlite-plain.tmp");
        {
            let mut dst = Connection::open(&tmp).map_err(|e| format!("open persist tmp: {e}"))?;
            {
                let backup = Backup::new(&self.conn, &mut dst)
                    .map_err(|e| format!("backup start: {e}"))?;
                backup
                    .run_to_completion(100, Duration::from_millis(0), None)
                    .map_err(|e| format!("backup: {e}"))?;
            }
        }
        let plain = fs::read(&tmp).map_err(|e| format!("read persist tmp: {e}"))?;
        let sealed = crypto::seal(&self.dek, &plain)?;
        fs::write(&self.enc_path, sealed).map_err(|e| format!("write encrypted store: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.enc_path, fs::Permissions::from_mode(0o600));
        }
        overwrite_and_remove(&tmp);
        self.encrypted_on_disk = true;
        Ok(())
    }
}

fn load_sqlite_bytes(dest: &mut Connection, bytes: &[u8], app_data: &Path) -> Result<(), String> {
    let tmp = app_data.join("codex-desk.load.tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("write load tmp: {e}"))?;
    let src = Connection::open(&tmp).map_err(|e| format!("open decrypted snapshot: {e}"))?;
    copy_db(&src, dest)?;
    overwrite_and_remove(&tmp);
    Ok(())
}

fn copy_db(src: &Connection, dest: &mut Connection) -> Result<(), String> {
    let backup = Backup::new(src, dest).map_err(|e| format!("backup load: {e}"))?;
    backup
        .run_to_completion(100, Duration::from_millis(0), None)
        .map_err(|e| format!("backup load: {e}"))?;
    Ok(())
}

fn overwrite_and_remove(path: &Path) {
    if let Ok(meta) = fs::metadata(path) {
        let zeros = vec![0u8; meta.len() as usize];
        let _ = fs::write(path, zeros);
    }
    let _ = fs::remove_file(path);
}

pub fn write_unlock_failure(app_data: &Path, reason: &str) {
    let path = app_data.join("unlock-failures.jsonl");
    let line = serde_json::json!({
        "at": chrono::Utc::now().to_rfc3339(),
        "action": "encryption.key_unlock_failure",
        "detail": "key unlock failed (no key material logged)",
        "code": sanitize_reason(reason),
    });
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        if let Ok(text) = serde_json::to_string(&line) {
            let _ = writeln!(file, "{text}");
        }
    }
}

fn sanitize_reason(reason: &str) -> String {
    let lower = reason.to_ascii_lowercase();
    if lower.contains("tamper") {
        "tamper_or_wrong_key".into()
    } else if lower.contains("truncated") || lower.contains("magic") {
        "malformed_envelope".into()
    } else {
        "unlock_failed".into()
    }
}
