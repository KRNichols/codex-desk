use crate::error::AppError;
use crate::store::AppStore;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub details: serde_json::Value,
    pub classification: String,
}

fn audit_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(dir.join("audit.jsonl"))
}

fn append_event(app: &AppHandle, event: &AuditEvent) -> Result<(), AppError> {
    let path = audit_path(app)?;
    let line = serde_json::to_string(event).map_err(|e| AppError::Io(e.to_string()))?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| AppError::Io(e.to_string()))?;
    writeln!(file, "{line}").map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}

fn read_events(app: &AppHandle) -> Result<Vec<AuditEvent>, AppError> {
    let path = audit_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| AppError::Io(e.to_string()))?;
    let mut events = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<AuditEvent>(line) {
            events.push(event);
        }
    }
    Ok(events)
}

fn new_event(
    actor: &str,
    action: &str,
    resource: &str,
    outcome: &str,
    details: serde_json::Value,
    classification: &str,
) -> AuditEvent {
    AuditEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        actor: actor.to_string(),
        action: action.to_string(),
        resource: resource.to_string(),
        outcome: outcome.to_string(),
        details,
        classification: classification.to_string(),
    }
}

#[tauri::command]
pub fn record_audit_event(
    app: AppHandle,
    store: tauri::State<AppStore>,
    action: String,
    resource: String,
    outcome: String,
    details: serde_json::Value,
    classification: Option<String>,
) -> Result<AuditEvent, AppError> {
    let actor = store.current_identity()?.id;
    let event = new_event(
        &actor,
        &action,
        &resource,
        &outcome,
        details,
        classification.as_deref().unwrap_or("CUI"),
    );
    append_event(&app, &event)?;
    Ok(event)
}

#[tauri::command]
pub fn list_audit_events(
    app: AppHandle,
    _store: tauri::State<AppStore>,
    limit: Option<usize>,
) -> Result<Vec<AuditEvent>, AppError> {
    let mut events = read_events(&app)?;
    events.reverse();
    if let Some(limit) = limit {
        events.truncate(limit);
    }
    Ok(events)
}

#[tauri::command]
pub fn export_audit_bundle(
    app: AppHandle,
    store: tauri::State<AppStore>,
) -> Result<String, AppError> {
    let events = read_events(&app)?;
    let identity = store.current_identity()?;
    let payload = serde_json::json!({
        "exportedAt": Utc::now().to_rfc3339(),
        "exportedBy": identity.id,
        "integrityNote": "Hash-chained audit export is a planned IL5 control. Current export is chronological JSONL reconstruction.",
        "events": events,
    });
    serde_json::to_string_pretty(&payload).map_err(|e| AppError::Io(e.to_string()))
}

pub fn record_system_event(
    app: &AppHandle,
    actor: &str,
    action: &str,
    resource: &str,
    outcome: &str,
    details: serde_json::Value,
) -> Result<(), AppError> {
    let event = new_event(actor, action, resource, outcome, details, "CUI");
    append_event(app, &event)
}
