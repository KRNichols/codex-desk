use crate::prompts::{
    DESK_IMPROVER_BRIEF, IL5_GRADER_BRIEF, LEGACY_DESK_IMPROVER_BRIEF, LEGACY_IL5_GRADER_BRIEF,
    PRIOR_DESK_IMPROVER_BRIEF, PRIOR_IL5_GRADER_BRIEF,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub brief: String,
    pub template: String,
    pub status: String,
    pub workspace_path: Option<String>,
    pub chat_id: Option<String>,
    pub worker_thread_id: Option<String>,
    pub grader_thread_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HillclimbRun {
    pub id: String,
    pub agent_id: String,
    pub goal: String,
    pub success_criteria: String,
    pub max_iterations: i64,
    pub current_iteration: i64,
    pub status: String,
    pub last_grade: Option<String>,
    pub last_gaps: Option<String>,
    pub allow_writes: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HillclimbIteration {
    pub id: String,
    pub run_id: String,
    pub iteration: i64,
    pub phase: String,
    pub worker_summary: Option<String>,
    pub grade: Option<String>,
    pub gaps: Option<String>,
    pub created_at: String,
}

pub fn migrate_agents(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            brief TEXT NOT NULL,
            template TEXT NOT NULL,
            status TEXT NOT NULL,
            workspace_path TEXT,
            chat_id TEXT,
            worker_thread_id TEXT,
            grader_thread_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS hillclimb_runs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            goal TEXT NOT NULL,
            success_criteria TEXT NOT NULL,
            max_iterations INTEGER NOT NULL,
            current_iteration INTEGER NOT NULL,
            status TEXT NOT NULL,
            last_grade TEXT,
            last_gaps TEXT,
            allow_writes INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS hillclimb_iterations (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            iteration INTEGER NOT NULL,
            phase TEXT NOT NULL,
            worker_summary TEXT,
            grade TEXT,
            gaps TEXT,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY,
            at TEXT NOT NULL,
            action TEXT NOT NULL,
            actor TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            detail TEXT NOT NULL
        );
        ",
    )
    .map_err(|e| format!("migrate agents: {e}"))?;
    seed_builtin(conn)?;
    refresh_template_brief(conn, "desk-improver", LEGACY_DESK_IMPROVER_BRIEF, DESK_IMPROVER_BRIEF)?;
    refresh_template_brief(conn, "desk-improver", PRIOR_DESK_IMPROVER_BRIEF, DESK_IMPROVER_BRIEF)?;
    refresh_template_brief(conn, "il5-grader", LEGACY_IL5_GRADER_BRIEF, IL5_GRADER_BRIEF)?;
    refresh_template_brief(conn, "il5-grader", PRIOR_IL5_GRADER_BRIEF, IL5_GRADER_BRIEF)
}

fn seed_builtin(conn: &Connection) -> Result<(), String> {
    if !template_exists(conn, "desk-improver")? {
        create_agent(
            conn,
            "Desk Improver",
            DESK_IMPROVER_BRIEF,
            "desk-improver",
            None,
        )?;
    }
    if !template_exists(conn, "il5-grader")? {
        create_agent(conn, "IL5 Architecture Grader", IL5_GRADER_BRIEF, "il5-grader", None)?;
    }
    Ok(())
}

fn refresh_template_brief(
    conn: &Connection,
    template: &str,
    legacy: &str,
    next: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE agents SET brief = ?1, updated_at = ?2 WHERE template = ?3 AND brief = ?4",
        params![next, Utc::now().to_rfc3339(), template, legacy],
    )
    .map_err(|e| format!("refresh {template} brief: {e}"))?;
    Ok(())
}

fn template_exists(conn: &Connection, template: &str) -> Result<bool, String> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agents WHERE template = ?1",
            params![template],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

pub fn create_agent(
    conn: &Connection,
    name: &str,
    brief: &str,
    template: &str,
    workspace_path: Option<&str>,
) -> Result<Agent, String> {
    let now = Utc::now().to_rfc3339();
    let agent = Agent {
        id: Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        brief: brief.trim().to_string(),
        template: template.to_string(),
        status: "idle".into(),
        workspace_path: workspace_path
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        chat_id: None,
        worker_thread_id: None,
        grader_thread_id: None,
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO agents (id, name, brief, template, status, workspace_path, chat_id, worker_thread_id, grader_thread_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, ?7, ?8)",
        params![
            agent.id,
            agent.name,
            agent.brief,
            agent.template,
            agent.status,
            agent.workspace_path,
            agent.created_at,
            agent.updated_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(agent)
}

pub fn list_agents(conn: &Connection) -> Result<Vec<Agent>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, brief, template, status, workspace_path, chat_id, worker_thread_id, grader_thread_id, created_at, updated_at
             FROM agents ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                brief: row.get(2)?,
                template: row.get(3)?,
                status: row.get(4)?,
                workspace_path: row.get(5)?,
                chat_id: row.get(6)?,
                worker_thread_id: row.get(7)?,
                grader_thread_id: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_agent(conn: &Connection, id: &str) -> Result<Option<Agent>, String> {
    let list = list_agents(conn)?;
    Ok(list.into_iter().find(|a| a.id == id))
}

pub fn update_agent(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    brief: Option<&str>,
    workspace_path: Option<&str>,
    status: Option<&str>,
    worker_thread_id: Option<&str>,
    grader_thread_id: Option<&str>,
) -> Result<Agent, String> {
    let mut agent = get_agent(conn, id)?.ok_or_else(|| "Agent not found.".to_string())?;
    if let Some(name) = name {
        agent.name = name.trim().to_string();
    }
    if let Some(brief) = brief {
        agent.brief = brief.trim().to_string();
    }
    if let Some(path) = workspace_path {
        agent.workspace_path = if path.trim().is_empty() {
            None
        } else {
            Some(path.trim().to_string())
        };
    }
    if let Some(status) = status {
        agent.status = status.to_string();
    }
    if let Some(id) = worker_thread_id {
        agent.worker_thread_id = Some(id.to_string());
    }
    if let Some(id) = grader_thread_id {
        agent.grader_thread_id = Some(id.to_string());
    }
    agent.updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE agents SET name=?1, brief=?2, workspace_path=?3, status=?4, worker_thread_id=?5, grader_thread_id=?6, updated_at=?7 WHERE id=?8",
        params![
            agent.name,
            agent.brief,
            agent.workspace_path,
            agent.status,
            agent.worker_thread_id,
            agent.grader_thread_id,
            agent.updated_at,
            agent.id
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(agent)
}

pub fn create_run(
    conn: &Connection,
    agent_id: &str,
    goal: &str,
    success_criteria: &str,
    max_iterations: i64,
    allow_writes: bool,
) -> Result<HillclimbRun, String> {
    let now = Utc::now().to_rfc3339();
    let max_iterations = max_iterations.clamp(1, 12);
    let run = HillclimbRun {
        id: Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        goal: goal.trim().to_string(),
        success_criteria: success_criteria.trim().to_string(),
        max_iterations,
        current_iteration: 0,
        status: "queued".into(),
        last_grade: None,
        last_gaps: None,
        allow_writes,
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO hillclimb_runs (id, agent_id, goal, success_criteria, max_iterations, current_iteration, status, last_grade, last_gaps, allow_writes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, NULL, NULL, ?7, ?8, ?9)",
        params![
            run.id,
            run.agent_id,
            run.goal,
            run.success_criteria,
            run.max_iterations,
            run.status,
            if allow_writes { 1 } else { 0 },
            run.created_at,
            run.updated_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(run)
}

pub fn get_run(conn: &Connection, id: &str) -> Result<Option<HillclimbRun>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, goal, success_criteria, max_iterations, current_iteration, status, last_grade, last_gaps, allow_writes, created_at, updated_at
             FROM hillclimb_runs WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(Some(map_run(row)?))
    } else {
        Ok(None)
    }
}

fn map_run(row: &rusqlite::Row<'_>) -> Result<HillclimbRun, String> {
    let allow: i64 = row.get(9).map_err(|e| e.to_string())?;
    Ok(HillclimbRun {
        id: row.get(0).map_err(|e| e.to_string())?,
        agent_id: row.get(1).map_err(|e| e.to_string())?,
        goal: row.get(2).map_err(|e| e.to_string())?,
        success_criteria: row.get(3).map_err(|e| e.to_string())?,
        max_iterations: row.get(4).map_err(|e| e.to_string())?,
        current_iteration: row.get(5).map_err(|e| e.to_string())?,
        status: row.get(6).map_err(|e| e.to_string())?,
        last_grade: row.get(7).map_err(|e| e.to_string())?,
        last_gaps: row.get(8).map_err(|e| e.to_string())?,
        allow_writes: allow != 0,
        created_at: row.get(10).map_err(|e| e.to_string())?,
        updated_at: row.get(11).map_err(|e| e.to_string())?,
    })
}

pub fn list_runs(conn: &Connection, agent_id: &str) -> Result<Vec<HillclimbRun>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, goal, success_criteria, max_iterations, current_iteration, status, last_grade, last_gaps, allow_writes, created_at, updated_at
             FROM hillclimb_runs WHERE agent_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![agent_id]).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        out.push(map_run(row)?);
    }
    Ok(out)
}

pub fn update_run(
    conn: &Connection,
    id: &str,
    current_iteration: i64,
    status: &str,
    last_grade: Option<&str>,
    last_gaps: Option<&str>,
) -> Result<HillclimbRun, String> {
    conn.execute(
        "UPDATE hillclimb_runs SET current_iteration=?1, status=?2, last_grade=?3, last_gaps=?4, updated_at=?5 WHERE id=?6",
        params![
            current_iteration,
            status,
            last_grade,
            last_gaps,
            Utc::now().to_rfc3339(),
            id
        ],
    )
    .map_err(|e| e.to_string())?;
    get_run(conn, id)?.ok_or_else(|| "Run not found after update.".into())
}

pub fn add_iteration(
    conn: &Connection,
    run_id: &str,
    iteration: i64,
    phase: &str,
    worker_summary: Option<&str>,
    grade: Option<&str>,
    gaps: Option<&str>,
) -> Result<HillclimbIteration, String> {
    let item = HillclimbIteration {
        id: Uuid::new_v4().to_string(),
        run_id: run_id.to_string(),
        iteration,
        phase: phase.to_string(),
        worker_summary: worker_summary.map(|s| s.to_string()),
        grade: grade.map(|s| s.to_string()),
        gaps: gaps.map(|s| s.to_string()),
        created_at: Utc::now().to_rfc3339(),
    };
    conn.execute(
        "INSERT INTO hillclimb_iterations (id, run_id, iteration, phase, worker_summary, grade, gaps, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            item.id,
            item.run_id,
            item.iteration,
            item.phase,
            item.worker_summary,
            item.grade,
            item.gaps,
            item.created_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(item)
}

pub fn list_iterations(conn: &Connection, run_id: &str) -> Result<Vec<HillclimbIteration>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, iteration, phase, worker_summary, grade, gaps, created_at
             FROM hillclimb_iterations WHERE run_id = ?1 ORDER BY iteration ASC, created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![run_id], |row| {
            Ok(HillclimbIteration {
                id: row.get(0)?,
                run_id: row.get(1)?,
                iteration: row.get(2)?,
                phase: row.get(3)?,
                worker_summary: row.get(4)?,
                grade: row.get(5)?,
                gaps: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
