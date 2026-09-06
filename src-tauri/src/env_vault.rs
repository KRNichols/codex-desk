//! Encrypted Desk env vault.
//!
//! Values live in `env-vault.wrap` (AES-256-GCM via the OS-backed DEK).
//! They are exported **only** to the child `codex` process. Never SQLite,
//! never git, never process-wide `std::env`, never audit detail.

use crate::crypto;
use crate::keystore;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const VAULT_FILE: &str = "env-vault.wrap";

pub fn vault_path(app_data: &Path) -> std::path::PathBuf {
    app_data.join(VAULT_FILE)
}

fn load_map(app_data: &Path) -> Result<BTreeMap<String, String>, String> {
    let path = vault_path(app_data);
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let unlocked = keystore::load_or_create_dek(app_data)?;
    let blob = fs::read(&path).map_err(|e| format!("read env vault: {e}"))?;
    let raw = crypto::open(&unlocked.dek, &blob)?;
    let value: Value = serde_json::from_slice(&raw).map_err(|e| format!("env vault json: {e}"))?;
    let mut out = BTreeMap::new();
    if let Some(obj) = value.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    out.insert(k.clone(), s.to_string());
                }
            }
        }
    }
    Ok(out)
}

fn save_map(app_data: &Path, map: &BTreeMap<String, String>) -> Result<(), String> {
    fs::create_dir_all(app_data).map_err(|e| format!("create app data: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(app_data, fs::Permissions::from_mode(0o700));
    }
    let path = vault_path(app_data);
    if map.is_empty() {
        if path.is_file() {
            overwrite_and_remove(&path);
        }
        return Ok(());
    }
    let unlocked = keystore::load_or_create_dek(app_data)?;
    let json = serde_json::to_vec(map).map_err(|e| e.to_string())?;
    let blob = crypto::seal(&unlocked.dek, &json)?;
    fs::write(&path, blob).map_err(|e| format!("write env vault: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn overwrite_and_remove(path: &Path) {
    if let Ok(meta) = fs::metadata(path) {
        let zeros = vec![0u8; meta.len() as usize];
        let _ = fs::write(path, zeros);
    }
    let _ = fs::remove_file(path);
}

pub fn validate_key(key: &str) -> Result<String, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Environment key name is empty.".into());
    }
    if key.len() > 80 {
        return Err("Environment key name is too long.".into());
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return Err("Environment key must be A-Z, 0-9, or underscore.".into());
    }
    Ok(key.to_string())
}

pub fn present_keys(app_data: &Path) -> Vec<String> {
    load_map(app_data)
        .unwrap_or_default()
        .into_keys()
        .collect()
}

pub fn has_key(app_data: &Path, key: &str) -> bool {
    load_map(app_data)
        .ok()
        .and_then(|m| m.get(key).cloned())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

pub fn get_value(app_data: &Path, key: &str) -> Option<String> {
    load_map(app_data).ok()?.get(key).cloned().filter(|v| !v.is_empty())
}

/// All vault values for child `codex` only. Caller must not persist or log them.
pub fn export_for_codex(app_data: &Path) -> BTreeMap<String, String> {
    load_map(app_data).unwrap_or_default()
}

pub fn set_value(app_data: &Path, key: &str, value: &str) -> Result<String, String> {
    let key = validate_key(key)?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Value is empty.".into());
    }
    let mut map = load_map(app_data)?;
    map.insert(key.clone(), trimmed.to_string());
    save_map(app_data, &map)?;
    if key == "AZURE_LLM_PAT" {
        let _ = keystore::set_pat_slot(app_data, trimmed);
    }
    Ok(key)
}

pub fn clear_value(app_data: &Path, key: &str) -> Result<(), String> {
    let key = validate_key(key)?;
    let mut map = load_map(app_data)?;
    map.remove(&key);
    save_map(app_data, &map)?;
    if key == "AZURE_LLM_PAT" {
        let _ = keystore::clear_pat_slot(app_data);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trip_and_clear() {
        let dir = TempDir::new().unwrap();
        set_value(dir.path(), "AZURE_LLM_PAT", "not-a-real-pat-value").unwrap();
        assert!(has_key(dir.path(), "AZURE_LLM_PAT"));
        assert_eq!(
            get_value(dir.path(), "AZURE_LLM_PAT").as_deref(),
            Some("not-a-real-pat-value")
        );
        assert!(vault_path(dir.path()).is_file());
        clear_value(dir.path(), "AZURE_LLM_PAT").unwrap();
        assert!(!has_key(dir.path(), "AZURE_LLM_PAT"));
    }

    #[test]
    fn rejects_bad_keys() {
        assert!(validate_key("azure-key").is_err());
        assert!(validate_key("").is_err());
        assert_eq!(validate_key("AZURE_OPENAI_API_KEY").unwrap(), "AZURE_OPENAI_API_KEY");
    }

    #[test]
    fn export_is_namespaced_map() {
        let dir = TempDir::new().unwrap();
        set_value(dir.path(), "AZURE_LLM_ENDPOINT", "https://example.openai.azure.com/openai/v1").unwrap();
        let exported = export_for_codex(dir.path());
        assert_eq!(
            exported.get("AZURE_LLM_ENDPOINT").map(String::as_str),
            Some("https://example.openai.azure.com/openai/v1")
        );
    }
}
