use crate::agents::{self, Agent, HillclimbIteration, HillclimbRun};
use crate::audit;
use crate::codex::{find_codex, run_turn, validate_workspace, workspace_dir, CodexEvent, ExecOpts};
use crate::prompts::{grader_prompt, parse_grade, worker_prompt};
use crate::store;
use crate::AppState;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Serialize)]
pub struct HillclimbEvent {
    pub run_id: String,
    pub agent_id: String,
    pub kind: String,
    pub iteration: i64,
    pub phase: String,
    pub text: String,
    pub grade: Option<String>,
}

pub fn emit(app: &AppHandle, event: HillclimbEvent) {
    let _ = app.emit("hillclimb-event", event);
}

fn with_db<T>(app: &AppHandle, f: impl FnOnce(&rusqlite::Connection) -> T) -> Option<T> {
    let state = app.try_state::<AppState>()?;
    let mut db = state.db.lock().ok()?;
    let out = f(&*db);
    let _ = db.persist();
    Some(out)
}

pub fn start_run(
    app: AppHandle,
    agent_id: String,
    goal: String,
    success_criteria: String,
    max_iterations: i64,
    allow_writes: bool,
) -> Result<HillclimbRun, String> {
    let app_data;
    let project_cwd;
    let run;
    let agent;
    {
        let state = app.state::<AppState>();
        app_data = state.app_data.clone();
        project_cwd = state.project_cwd.clone();
        let mut db = state.db.lock().map_err(|e| e.to_string())?;
        agent = agents::get_agent(&db, &agent_id)?.ok_or_else(|| "Agent not found.".to_string())?;
        if allow_writes {
            crate::identity::require_write_attestation(&db)?;
        }
        run = agents::create_run(&db, &agent_id, &goal, &success_criteria, max_iterations, allow_writes)?;
        let _ = agents::update_agent(&db, &agent_id, None, None, None, Some("running"), None, None);
        audit::write(
            &db,
            &app_data,
            "hillclimb.start",
            "run",
            &run.id,
            &format!("agent={} writes={} max={}", agent.name, allow_writes, run.max_iterations),
        );
        let _ = db.persist();
    }

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let state = app.state::<AppState>();
        let mut runs = state.runs.lock().map_err(|e| e.to_string())?;
        runs.insert(format!("run:{}", run.id), cancel.clone());
    }

    let run_id = run.id.clone();
    std::thread::spawn(move || {
        execute_loop(app, agent, run_id, cancel, app_data, project_cwd);
    });
    Ok(run)
}

pub fn cancel_run(app: &AppHandle, run_id: &str) -> Result<HillclimbRun, String> {
    let state = app.state::<AppState>();
    if let Ok(runs) = state.runs.lock() {
        if let Some(flag) = runs.get(&format!("run:{run_id}")) {
            flag.store(true, Ordering::Relaxed);
        }
    }
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let run = agents::get_run(&db, run_id)?.ok_or_else(|| "Run not found.".to_string())?;
    let updated = agents::update_run(&db, run_id, run.current_iteration, "cancelled", run.last_grade.as_deref(), run.last_gaps.as_deref())?;
    let _ = agents::update_agent(&db, &run.agent_id, None, None, None, Some("idle"), None, None);
    audit::write(&db, &state.app_data, "hillclimb.cancel", "run", run_id, "user cancel");
    let _ = db.persist();
    Ok(updated)
}

fn execute_loop(
    app: AppHandle,
    mut agent: Agent,
    run_id: String,
    cancel: Arc<AtomicBool>,
    app_data: PathBuf,
    project_cwd: PathBuf,
) {
    let Some(binary) = find_codex() else {
        fail(
            &app,
            &agent,
            &run_id,
            "The `codex` CLI was not found on PATH. Hill-climb cannot start.",
            true,
        );
        return;
    };

    let workdir = match agent.workspace_path.as_deref() {
        Some(path) => match validate_workspace(path) {
            Ok(dir) => dir,
            Err(err) => {
                fail(&app, &agent, &run_id, &err, false);
                return;
            }
        },
        None => match workspace_dir(&app_data) {
            Ok(dir) => dir,
            Err(err) => {
                fail(&app, &agent, &run_id, &err, false);
                return;
            }
        },
    };

    let allow_writes = with_db(&app, |db| {
        agents::get_run(db, &run_id)
            .ok()
            .flatten()
            .map(|r| r.allow_writes && agent.workspace_path.is_some())
            .unwrap_or(false)
    })
    .unwrap_or(false);

    let sandbox = if allow_writes {
        "workspace-write"
    } else {
        "read-only"
    };
    let opts = ExecOpts {
        workdir: workdir.clone(),
        sandbox: sandbox.into(),
        desk_agent_job: true,
    };
    let il5_mode = agent.template == "il5-grader";

    let max = with_db(&app, |db| {
        agents::get_run(db, &run_id)
            .ok()
            .flatten()
            .map(|r| r.max_iterations)
            .unwrap_or(3)
    })
    .unwrap_or(3);

    let mut prior_gaps: Option<String> = None;
    for i in 1..=max {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let snapshot = with_db(&app, |db| agents::get_run(db, &run_id).ok().flatten()).flatten();
        let Some(run) = snapshot else {
            break;
        };
        if run.status == "cancelled" {
            break;
        }

        emit(
            &app,
            HillclimbEvent {
                run_id: run_id.clone(),
                agent_id: agent.id.clone(),
                kind: "status".into(),
                iteration: i,
                phase: "worker".into(),
                text: format!("Iteration {i}/{max}: worker (Codex)"),
                grade: None,
            },
        );
        with_db(&app, |db| {
            let _ = agents::update_run(db, &run_id, i, "running", run.last_grade.as_deref(), run.last_gaps.as_deref());
            audit::write(db, &app_data, "hillclimb.iteration", "run", &run_id, &format!("iteration={i} phase=worker"));
        });

        let prompt = worker_prompt(
            &agent.name,
            &agent.brief,
            &run.goal,
            &run.success_criteria,
            &workdir.display().to_string(),
            i as u32,
            max as u32,
            prior_gaps.as_deref(),
        );
        let worker = run_turn(
            &binary,
            agent.worker_thread_id.as_deref(),
            &prompt,
            &app_data,
            &project_cwd,
            cancel.clone(),
            Some(&opts),
            |event: CodexEvent| {
                emit(
                    &app,
                    HillclimbEvent {
                        run_id: run_id.clone(),
                        agent_id: agent.id.clone(),
                        kind: event.kind,
                        iteration: i,
                        phase: "worker".into(),
                        text: event.text,
                        grade: None,
                    },
                );
            },
        );

        let worker_text = match worker {
            Ok((text, thread)) => {
                if let Some(id) = thread {
                    agent.worker_thread_id = Some(id.clone());
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Ok(db) = state.db.lock() {
                            let _ = agents::update_agent(&db, &agent.id, None, None, None, None, Some(&id), None);
                        }
                    }
                }
                text
            }
            Err(err) => {
                fail(&app, &agent, &run_id, &err, err.contains("not found on PATH") || err.contains("authenticate"));
                return;
            }
        };

        with_db(&app, |db| {
            let _ = agents::add_iteration(db, &run_id, i, "worker", Some(&worker_text), None, None);
        });

        if cancel.load(Ordering::Relaxed) {
            break;
        }

        emit(
            &app,
            HillclimbEvent {
                run_id: run_id.clone(),
                agent_id: agent.id.clone(),
                kind: "status".into(),
                iteration: i,
                phase: "grader".into(),
                text: format!("Iteration {i}/{max}: grader (Codex)"),
                grade: None,
            },
        );

        let gprompt = grader_prompt(
            &agent.name,
            &agent.brief,
            &run.goal,
            &run.success_criteria,
            &workdir.display().to_string(),
            i as u32,
            &worker_text,
            il5_mode,
        );
        let grader = run_turn(
            &binary,
            agent.grader_thread_id.as_deref(),
            &gprompt,
            &app_data,
            &project_cwd,
            cancel.clone(),
            Some(&ExecOpts {
                workdir: workdir.clone(),
                sandbox: "read-only".into(),
                desk_agent_job: true,
            }),
            |event: CodexEvent| {
                emit(
                    &app,
                    HillclimbEvent {
                        run_id: run_id.clone(),
                        agent_id: agent.id.clone(),
                        kind: event.kind,
                        iteration: i,
                        phase: "grader".into(),
                        text: event.text,
                        grade: None,
                    },
                );
            },
        );

        let grader_text = match grader {
            Ok((text, thread)) => {
                if let Some(id) = thread {
                    agent.grader_thread_id = Some(id.clone());
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Ok(db) = state.db.lock() {
                            let _ = agents::update_agent(&db, &agent.id, None, None, None, None, None, Some(&id));
                        }
                    }
                }
                text
            }
            Err(err) => {
                fail(&app, &agent, &run_id, &err, false);
                return;
            }
        };

        let (grade, gaps) = parse_grade(&grader_text);
        let (grade, gaps) = crate::policy::enforce_grade(&worker_text, &grader_text, &grade, &gaps);
        prior_gaps = Some(gaps.clone());
        let terminal = if grade == "PASS" {
            "passed"
        } else if i >= max {
            "hold"
        } else {
            "running"
        };
        with_db(&app, |db| {
            let _ = agents::add_iteration(db, &run_id, i, "grader", Some(&grader_text), Some(&grade), Some(&gaps));
            let _ = agents::update_run(db, &run_id, i, terminal, Some(&grade), Some(&gaps));
            audit::write(
                db,
                &app_data,
                "hillclimb.grade",
                "run",
                &run_id,
                &format!("iteration={i} grade={grade}"),
            );
        });
        emit(
            &app,
            HillclimbEvent {
                run_id: run_id.clone(),
                agent_id: agent.id.clone(),
                kind: "grade".into(),
                iteration: i,
                phase: "grader".into(),
                text: gaps.clone(),
                grade: Some(grade.clone()),
            },
        );

        if grade == "PASS" || i >= max {
            let agent_status = if grade == "PASS" { "done" } else { "blocked" };
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(db) = state.db.lock() {
                    let _ = agents::update_agent(&db, &agent.id, None, None, None, Some(agent_status), None, None);
                    audit::write(&db, &app_data, "hillclimb.stop", "run", &run_id, &format!("status={terminal}"));
                }
                if let Ok(mut runs) = state.runs.lock() {
                    runs.remove(&format!("run:{run_id}"));
                }
            }
            emit(
                &app,
                HillclimbEvent {
                    run_id: run_id.clone(),
                    agent_id: agent.id.clone(),
                    kind: "done".into(),
                    iteration: i,
                    phase: "grader".into(),
                    text: terminal.to_string(),
                    grade: Some(grade),
                },
            );
            return;
        }
    }
}

fn fail(app: &AppHandle, agent: &Agent, run_id: &str, message: &str, secret_fail: bool) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut db) = state.db.lock() {
            let _ = agents::update_run(&db, run_id, 0, "error", Some("HOLD"), Some(message));
            let _ = agents::update_agent(&db, &agent.id, None, None, None, Some("blocked"), None, None);
            audit::write(&db, &state.app_data, "hillclimb.stop", "run", run_id, "error");
            if secret_fail {
                audit::write(&db, &state.app_data, "secret.access_failure", "run", run_id, "Codex missing or Azure auth failed (value not logged)");
            }
            let _ = db.persist();
        }
        if let Ok(mut runs) = state.runs.lock() {
            runs.remove(&format!("run:{run_id}"));
        }
    }
    emit(
        app,
        HillclimbEvent {
            run_id: run_id.to_string(),
            agent_id: agent.id.clone(),
            kind: "error".into(),
            iteration: 0,
            phase: "system".into(),
            text: message.to_string(),
            grade: Some("HOLD".into()),
        },
    );
}

#[allow(dead_code)]
pub fn attach_operator_note(db: &rusqlite::Connection, chat_id: &str, text: &str) -> Result<crate::store::Message, String> {
    store::add_message(db, chat_id, "assistant", text, "complete")
}

pub fn run_detail(db: &rusqlite::Connection, run_id: &str) -> Result<(HillclimbRun, Vec<HillclimbIteration>), String> {
    let run = agents::get_run(db, run_id)?.ok_or_else(|| "Run not found.".to_string())?;
    let iterations = agents::list_iterations(db, run_id)?;
    Ok((run, iterations))
}
