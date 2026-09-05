mod agents;
mod audit;
mod boundary;
mod codex;
mod crypto;
mod hillclimb;
mod identity;
mod keystore;
mod local_env;
mod policy;
mod prompts;
mod store;
mod vault;

use crate::agents::{Agent, HillclimbIteration, HillclimbRun};
use crate::codex::{find_codex, probe_status, run_turn, CodexEvent, RuntimeStatus};
use crate::identity::IdentityStatus;
use crate::prompts::{CLOSE_IL5_MISSING_CRITERIA, CLOSE_IL5_MISSING_GOAL};
use crate::store::{Chat, Message};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    db: Mutex<vault::Vault>,
    app_data: PathBuf,
    project_cwd: PathBuf,
    runs: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[derive(Clone, Serialize)]
struct StreamPayload {
    chat_id: String,
    message_id: String,
    kind: String,
    text: String,
    thread_id: Option<String>,
}

fn emit_stream(app: &AppHandle, payload: StreamPayload) {
    let _ = app.emit("codex-stream", payload);
}

fn persist_locked(db: &mut vault::Vault) {
    let _ = db.persist();
}

#[tauri::command]
fn runtime_status(state: State<AppState>) -> RuntimeStatus {
    let mut status = probe_status(Some(&state.app_data), &state.project_cwd, "tauri");
    if let Ok(db) = state.db.lock() {
        status.store_encrypted = db.encrypted_on_disk;
        status.key_backend = Some(db.backend.as_str().to_string());
        status.audit_chain_ok = audit::chain_ok(&db);
        status.session_user = identity::session_user();
        status.machine_bound = true;
        status.machine_binding_ok = identity::binding_matches(&db).unwrap_or(false);
        status.operator_attested = identity::load_attestation(&db)
            .map(|a| a.configured)
            .unwrap_or(false);
        status.pat_slot = if keystore::pat_slot_present(&state.app_data) {
            "os-secret-store".into()
        } else {
            "unset".into()
        };
        status.hello_bind = identity::hello_bind_label().into();
        if !status.machine_binding_ok {
            status.issues.push(crate::codex::SetupIssue {
                code: "identity_binding".into(),
                message: "Machine-bound identity does not match this OS user/host. Store unlock should fail closed.".into(),
            });
        }
        if let Ok(found) = crate::codex::scan_store_for_secrets(&db) {
            if found {
                status.issues.push(crate::codex::SetupIssue {
                    code: "pat_in_store".into(),
                    message: "Refusing insecure secret storage: a PAT/token-like value was found in the local store. Remove it. PAT belongs in env or the OS secret slot only.".into(),
                });
            }
        }
    }
    status
}

#[tauri::command]
fn identity_status(state: State<AppState>) -> Result<IdentityStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(IdentityStatus {
        session_user: identity::session_user(),
        machine_id_present: !identity::machine_id().is_empty(),
        machine_bound: true,
        machine_binding_ok: identity::binding_matches(&db)?,
        key_backend: db.backend.as_str().to_string(),
        store_encrypted: db.encrypted_on_disk,
        audit_chain_ok: audit::chain_ok(&db),
        operator_attestation: identity::load_attestation(&db)?,
        pat_slot: if keystore::pat_slot_present(&state.app_data) {
            "os-secret-store".into()
        } else {
            "unset".into()
        },
        hello_bind: identity::hello_bind_label().into(),
    })
}

#[tauri::command]
fn set_operator_attestation(
    state: State<AppState>,
    operator_name: String,
    organization: String,
    statement: String,
) -> Result<identity::OperatorAttestation, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let record = identity::set_attestation(&db, &operator_name, &organization, &statement)?;
    audit::write(
        &db,
        &state.app_data,
        "identity.attest",
        "identity",
        "local",
        "operator attestation recorded (no secret values)",
    );
    persist_locked(&mut db);
    Ok(record)
}

#[tauri::command]
fn set_pat_slot(state: State<AppState>, pat: String) -> Result<String, String> {
    let backend = keystore::set_pat_slot(&state.app_data, &pat)?;
    if let Ok(mut db) = state.db.lock() {
        audit::write(
            &db,
            &state.app_data,
            "secret.slot_write",
            "secret",
            "pat",
            "PAT written to OS secret store (value not logged)",
        );
        persist_locked(&mut db);
    }
    Ok(backend)
}

#[tauri::command]
fn clear_pat_slot(state: State<AppState>) -> Result<(), String> {
    keystore::clear_pat_slot(&state.app_data)?;
    if let Ok(mut db) = state.db.lock() {
        audit::write(
            &db,
            &state.app_data,
            "secret.slot_clear",
            "secret",
            "pat",
            "PAT slot cleared (value not logged)",
        );
        persist_locked(&mut db);
    }
    Ok(())
}

#[tauri::command]
fn list_chats(state: State<AppState>) -> Result<Vec<Chat>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_chats(&db)
}

#[tauri::command]
fn create_chat(state: State<AppState>) -> Result<Chat, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let chat = store::create_chat(&db, "New chat")?;
    persist_locked(&mut db);
    Ok(chat)
}

#[tauri::command]
fn delete_chat(state: State<AppState>, chat_id: String) -> Result<(), String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_chat(&db, &chat_id)?;
    persist_locked(&mut db);
    Ok(())
}

#[tauri::command]
fn list_messages(state: State<AppState>, chat_id: String) -> Result<Vec<Message>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_messages(&db, &chat_id)
}

#[tauri::command]
fn send_message(app: AppHandle, state: State<AppState>, chat_id: String, content: String) -> Result<Message, String> {
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err("Message is empty.".into());
    }

    let (user_message, assistant, thread_id) = {
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        if store::get_chat(&db, &chat_id)?.is_none() {
            return Err("Chat not found.".into());
        }
        let user_message = store::add_message(&db, &chat_id, "user", &trimmed, "complete")?;
        let _ = store::touch_title_if_default(&db, &chat_id, &trimmed);
        let assistant = store::add_message(&db, &chat_id, "assistant", "", "running")?;
        let chat = store::get_chat(&db, &chat_id)?;
        persist_locked(&mut db);
        (user_message, assistant, chat.and_then(|c| c.codex_thread_id))
    };

    let Some(binary) = find_codex().and_then(|p| crate::boundary::assert_local_codex(&p).ok()) else {
        let error = "The `codex` CLI was not found on PATH. Install Codex, confirm `codex --version` works, then restart Codex Desk.";
        if let Ok(mut db) = state.db.lock() {
            let _ = store::update_message(&db, &assistant.id, error, "error");
            persist_locked(&mut db);
        }
        emit_stream(
            &app,
            StreamPayload {
                chat_id: chat_id.clone(),
                message_id: assistant.id.clone(),
                kind: "error".into(),
                text: error.to_string(),
                thread_id: None,
            },
        );
        let _ = user_message;
        return Ok(Message {
            id: assistant.id,
            chat_id: assistant.chat_id,
            role: assistant.role,
            content: error.to_string(),
            created_at: assistant.created_at,
            status: "error".into(),
        });
    };

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut runs = state.runs.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = runs.insert(chat_id.clone(), cancel.clone()) {
            existing.store(true, Ordering::Relaxed);
        }
    }

    let app_data = state.app_data.clone();
    let project_cwd = state.project_cwd.clone();
    let assistant_id = assistant.id.clone();
    let chat_id_clone = chat_id.clone();

    std::thread::spawn(move || {
        let app_for_events = app.clone();
        let prompt = crate::prompts::operator_chat_prompt(&trimmed);
        let result = run_turn(
            &binary,
            thread_id.as_deref(),
            &prompt,
            &app_data,
            &project_cwd,
            cancel,
            None,
            |event: CodexEvent| {
                emit_stream(
                    &app_for_events,
                    StreamPayload {
                        chat_id: chat_id_clone.clone(),
                        message_id: assistant_id.clone(),
                        kind: event.kind,
                        text: event.text,
                        thread_id: event.thread_id,
                    },
                );
            },
        );

        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut runs) = state.runs.lock() {
                runs.remove(&chat_id_clone);
            }
            if let Ok(mut db) = state.db.lock() {
                match result {
                    Ok((text, new_thread)) => {
                        let _ = store::update_message(&db, &assistant_id, &text, "complete");
                        if let Some(id) = new_thread {
                            let _ = store::set_thread_id(&db, &chat_id_clone, &id);
                        }
                        persist_locked(&mut db);
                        emit_stream(
                            &app,
                            StreamPayload {
                                chat_id: chat_id_clone,
                                message_id: assistant_id,
                                kind: "done".into(),
                                text,
                                thread_id: None,
                            },
                        );
                    }
                    Err(err) => {
                        let _ = store::update_message(&db, &assistant_id, &err, "error");
                        persist_locked(&mut db);
                        emit_stream(
                            &app,
                            StreamPayload {
                                chat_id: chat_id_clone,
                                message_id: assistant_id,
                                kind: "error".into(),
                                text: err,
                                thread_id: None,
                            },
                        );
                    }
                }
            }
        }
    });

    let _ = user_message;
    Ok(assistant)
}

#[tauri::command]
fn list_agents(state: State<AppState>) -> Result<Vec<Agent>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    agents::list_agents(&db)
}

#[tauri::command]
fn create_agent(state: State<AppState>, name: String, brief: String, workspace_path: Option<String>) -> Result<Agent, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let brief = if brief.trim().is_empty() {
        crate::prompts::OPERATOR_CONTRACT.to_string()
    } else {
        brief
    };
    let agent = agents::create_agent(&db, &name, &brief, "custom", workspace_path.as_deref())?;
    audit::write(&db, &state.app_data, "agent.create", "agent", &agent.id, &agent.name);
    persist_locked(&mut db);
    Ok(agent)
}

#[tauri::command]
fn update_agent(
    state: State<AppState>,
    agent_id: String,
    name: Option<String>,
    brief: Option<String>,
    workspace_path: Option<String>,
) -> Result<Agent, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let agent = agents::update_agent(
        &db,
        &agent_id,
        name.as_deref(),
        brief.as_deref(),
        workspace_path.as_deref(),
        None,
        None,
        None,
    )?;
    audit::write(&db, &state.app_data, "agent.update", "agent", &agent.id, "updated");
    persist_locked(&mut db);
    Ok(agent)
}

#[tauri::command]
fn list_agent_runs(state: State<AppState>, agent_id: String) -> Result<Vec<HillclimbRun>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    agents::list_runs(&db, &agent_id)
}

#[tauri::command]
fn get_run(state: State<AppState>, run_id: String) -> Result<(HillclimbRun, Vec<HillclimbIteration>), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    hillclimb::run_detail(&db, &run_id)
}

#[tauri::command]
fn start_hillclimb(
    app: AppHandle,
    agent_id: String,
    goal: String,
    success_criteria: String,
    max_iterations: i64,
    allow_writes: bool,
) -> Result<HillclimbRun, String> {
    hillclimb::start_run(app, agent_id, goal, success_criteria, max_iterations, allow_writes)
}

#[tauri::command]
fn cancel_hillclimb(app: AppHandle, run_id: String) -> Result<HillclimbRun, String> {
    hillclimb::cancel_run(&app, &run_id)
}

#[tauri::command]
fn il5_goal_template() -> (String, String) {
    (CLOSE_IL5_MISSING_GOAL.to_string(), CLOSE_IL5_MISSING_CRITERIA.to_string())
}

#[tauri::command]
fn list_audit(state: State<AppState>) -> Result<Vec<audit::AuditEvent>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    audit::list_recent(&db, 50)
}

#[tauri::command]
fn export_audit(state: State<AppState>) -> Result<String, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    audit::write(
        &db,
        &state.app_data,
        "audit.export",
        "audit",
        "local",
        "operator exported hash-chained audit (no secret values)",
    );
    persist_locked(&mut db);
    audit::export_json(&db)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from(".data"));
            std::fs::create_dir_all(&app_data)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&app_data, std::fs::Permissions::from_mode(0o700));
            }
            let db = match vault::Vault::open(&app_data) {
                Ok(v) => v,
                Err(err) => {
                    vault::write_unlock_failure(&app_data, &err);
                    return Err(err.into());
                }
            };
            let project_cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            app.manage(AppState {
                db: Mutex::new(db),
                app_data,
                project_cwd,
                runs: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            identity_status,
            set_operator_attestation,
            set_pat_slot,
            clear_pat_slot,
            list_chats,
            create_chat,
            delete_chat,
            list_messages,
            send_message,
            list_agents,
            create_agent,
            update_agent,
            list_agent_runs,
            get_run,
            start_hillclimb,
            cancel_hillclimb,
            list_audit,
            export_audit,
            il5_goal_template
        ])
        .run(tauri::generate_context!())
        .expect("error while running Codex Desk");
}
