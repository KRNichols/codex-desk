//! Six-job harness record, grader scores, classify → promote.

use crate::autonomy::{self, Consequence};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const JOB_NAMES: [&str; 6] = [
    "contract",
    "context",
    "tools",
    "state",
    "evidence",
    "recovery",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessJob {
    pub name: String,
    pub label: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessPromotion {
    pub id: String,
    pub category: String,
    pub gap: String,
    pub patch: String,
    pub status: String,
    pub created_at: String,
    pub promoted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessRecord {
    pub jobs: Vec<HarnessJob>,
    pub autonomy_tier: String,
    pub autonomy_label: String,
    pub approval_status: String,
    pub approval_evidence: Option<String>,
    pub classified_gap: Option<String>,
    pub gap_category: Option<String>,
    pub recovery_phase: String,
    pub promotions: Vec<HarnessPromotion>,
    pub sandbox: String,
    pub allowlist: String,
}

impl Default for HarnessRecord {
    fn default() -> Self {
        Self {
            jobs: initial_jobs(),
            autonomy_tier: Consequence::Read.as_str().into(),
            autonomy_label: Consequence::Read.label().into(),
            approval_status: "none".into(),
            approval_evidence: None,
            classified_gap: None,
            gap_category: None,
            recovery_phase: "observe".into(),
            promotions: Vec::new(),
            sandbox: "read-only".into(),
            allowlist: "local-codex-only".into(),
        }
    }
}

fn job_label(name: &str) -> &'static str {
    match name {
        "contract" => "1 Contract — goal, constraints, done",
        "context" => "2 Context — rules, facts, current state",
        "tools" => "3 Tools — schemas, permissions, sandboxes",
        "state" => "4 State — persist decisions, artifacts, open risks",
        "evidence" => "5 Evidence — tests, sources, screenshots",
        "recovery" => "6 Recovery — retry locally, escalate, improve the system",
        _ => "job",
    }
}

fn initial_jobs() -> Vec<HarnessJob> {
    JOB_NAMES
        .iter()
        .map(|name| HarnessJob {
            name: (*name).into(),
            label: job_label(name).into(),
            status: "WARN".into(),
            summary: "Not scored yet.".into(),
        })
        .collect()
}

pub fn new_record(goal: &str, criteria: &str, workspace: Option<&str>, allow_writes: bool) -> HarnessRecord {
    let tier = autonomy::classify_goal(goal, criteria);
    let approval = match tier {
        Consequence::SendMergeDeploy => "required",
        Consequence::DeletePayPublish => "confirm_required",
        _ => "none",
    };
    let sandbox = if allow_writes {
        "workspace-write"
    } else {
        "read-only"
    };
    let mut rec = HarnessRecord {
        jobs: initial_jobs(),
        autonomy_tier: tier.as_str().into(),
        autonomy_label: tier.label().into(),
        approval_status: approval.into(),
        approval_evidence: None,
        classified_gap: None,
        gap_category: None,
        recovery_phase: "observe".into(),
        promotions: Vec::new(),
        sandbox: sandbox.into(),
        allowlist: "local-codex-only".into(),
    };
    rec.jobs = score_jobs(&ScoreInput {
        goal,
        criteria,
        workspace: workspace.unwrap_or(""),
        brief_present: true,
        sandbox,
        iteration: 0,
        worker: "",
        grader: "",
        grade: None,
        classified_gap: None,
        allow_writes,
    });
    rec
}

pub struct ScoreInput<'a> {
    pub goal: &'a str,
    pub criteria: &'a str,
    pub workspace: &'a str,
    pub brief_present: bool,
    pub sandbox: &'a str,
    pub iteration: i64,
    pub worker: &'a str,
    pub grader: &'a str,
    pub grade: Option<&'a str>,
    pub classified_gap: Option<&'a str>,
    pub allow_writes: bool,
}

pub fn score_jobs(input: &ScoreInput<'_>) -> Vec<HarnessJob> {
    let parsed = parse_job_block(input.worker).or_else(|| parse_job_block(input.grader));
    JOB_NAMES
        .iter()
        .map(|name| {
            let deterministic = score_one(name, input);
            let from_model = parsed.as_ref().and_then(|m| m.iter().find(|j| j.name == *name));
            let (status, summary) = merge_score(deterministic, from_model);
            HarnessJob {
                name: (*name).into(),
                label: job_label(name).into(),
                status,
                summary,
            }
        })
        .collect()
}

fn merge_score(det: (String, String), model: Option<&HarnessJob>) -> (String, String) {
    let Some(model) = model else {
        return det;
    };
    if rank_status(&det.0) > rank_status(&model.status) {
        det
    } else {
        (model.status.clone(), model.summary.clone())
    }
}

fn rank_status(s: &str) -> u8 {
    match s {
        "HOLD" => 2,
        "WARN" => 1,
        _ => 0,
    }
}

fn score_one(name: &str, input: &ScoreInput<'_>) -> (String, String) {
    match name {
        "contract" => {
            if input.goal.trim().is_empty() || input.criteria.trim().is_empty() {
                ("HOLD".into(), "Goal or done criteria missing.".into())
            } else {
                (
                    "PASS".into(),
                    "Goal, constraints, and done criteria are on the run record.".into(),
                )
            }
        }
        "context" => {
            if !input.brief_present {
                ("HOLD".into(), "Worker brief / operator contract missing.".into())
            } else if input.workspace.trim().is_empty() && input.allow_writes {
                (
                    "WARN".into(),
                    "Operator contract present; workspace path empty (app-data workspace).".into(),
                )
            } else {
                (
                    "PASS".into(),
                    format!(
                        "Operator contract + rules in prompt. Workspace: {}",
                        if input.workspace.trim().is_empty() {
                            "(app-data)"
                        } else {
                            input.workspace
                        }
                    ),
                )
            }
        }
        "tools" => {
            let sandbox_ok = input.sandbox == "read-only" || input.sandbox == "workspace-write";
            if sandbox_ok {
                (
                    "PASS".into(),
                    format!(
                        "Sandbox {} · allowlist local-codex-only. YOLO writes when workspace is set.",
                        input.sandbox
                    ),
                )
            } else {
                ("HOLD".into(), "Sandbox / allowlist not recorded.".into())
            }
        }
        "state" => {
            if input.iteration <= 0 {
                ("WARN".into(), "Run queued — no iteration persisted yet.".into())
            } else {
                (
                    "PASS".into(),
                    format!(
                        "Iteration {} persisted with decisions / gaps on the run record.",
                        input.iteration
                    ),
                )
            }
        }
        "evidence" => {
            let blob = format!("{}\n{}", input.worker, input.grader);
            let lower = blob.to_ascii_lowercase();
            let has = ["test", "cargo test", "npm test", "screenshot", "evidence", "src/", "docs/"]
                .iter()
                .any(|k| lower.contains(k));
            if input.grade == Some("HOLD") && !has {
                (
                    "HOLD".into(),
                    "Grade is HOLD and the worker/grader cited no tests, sources, or files.".into(),
                )
            } else if has {
                ("PASS".into(), "Worker/grader cited tests, sources, or files.".into())
            } else if input.iteration <= 0 {
                ("WARN".into(), "No evidence yet — run has not produced a worker summary.".into())
            } else {
                ("WARN".into(), "Work happened; cite a test, source, or screenshot next.".into())
            }
        }
        "recovery" => {
            let grade = input.grade.unwrap_or("");
            if grade == "HOLD" || grade == "WARN" {
                if input.classified_gap.map(|g| !g.trim().is_empty()).unwrap_or(false) {
                    (
                        "PASS".into(),
                        format!(
                            "Gap classified (not a blind retry): {}",
                            input.classified_gap.unwrap_or("")
                        ),
                    )
                } else {
                    (
                        "HOLD".into(),
                        "Failure without Classify. Return the exact gap; do not retry blindly.".into(),
                    )
                }
            } else if grade == "PASS" && input.classified_gap.is_some() {
                (
                    "PASS".into(),
                    "Verified after Classify → Patch. Offer / promote into the harness.".into(),
                )
            } else if input.iteration <= 0 {
                ("WARN".into(), "Recovery idle until a fail is observed.".into())
            } else {
                ("PASS".into(), "No open recovery — last grade is not a fail.".into())
            }
        }
        _ => ("WARN".into(), "Unknown job.".into()),
    }
}

pub fn parse_job_block(text: &str) -> Option<Vec<HarnessJob>> {
    let upper = text.to_ascii_uppercase();
    let start = upper.find("HARNESS-JOBS:")?;
    let rest = &text[start + "HARNESS-JOBS:".len()..];
    let mut jobs = Vec::new();
    for line in rest.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() && !jobs.is_empty() {
            break;
        }
        let Some((name, rest)) = trimmed.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if !JOB_NAMES.contains(&name.as_str()) {
            continue;
        }
        let rest = rest.trim();
        let (status, summary) = if let Some((tok, sum)) = rest.split_once('—') {
            (normalize_status(tok), sum.trim().to_string())
        } else if let Some((tok, sum)) = rest.split_once(" - ") {
            (normalize_status(tok), sum.trim().to_string())
        } else {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let tok = parts.next().unwrap_or("WARN");
            (normalize_status(tok), parts.next().unwrap_or("").trim().to_string())
        };
        jobs.push(HarnessJob {
            label: job_label(&name).into(),
            name,
            status,
            summary,
        });
    }
    if jobs.is_empty() {
        None
    } else {
        Some(jobs)
    }
}

fn normalize_status(tok: &str) -> String {
    match tok.trim().to_ascii_uppercase().as_str() {
        "PASS" | "READY" => "PASS".into(),
        "HOLD" => "HOLD".into(),
        _ => "WARN".into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedGap {
    pub category: String,
    pub gap: String,
    pub promote: String,
}

pub fn classify_gap(gaps: &str) -> ClassifiedGap {
    let lower = gaps.to_ascii_lowercase();
    let (category, promote) = if lower.contains("test") || lower.contains("unvalidated") {
        ("test", "Add or run a test that locks the gap.")
    } else if lower.contains("policy") || lower.contains("hold:") || lower.contains("ato") {
        ("policy", "Tighten policy.rs / preview policy so the next run HOLDs this fail.")
    } else if lower.contains("brief") || lower.contains("operator.md") || lower.contains("contract") {
        ("brief", "Fix the worker brief or OPERATOR.md hook.")
    } else if lower.contains("sandbox") || lower.contains("allowlist") || lower.contains("tool") {
        ("tool", "Improve the tool schema, sandbox, or allowlist note.")
    } else if lower.contains("loop") || lower.contains("iteration") || lower.contains("retry") {
        ("loop", "Fix the hill-climb loop so Classify happens before the next patch.")
    } else if lower.contains("setup") || lower.contains("env_key") || lower.contains("config.toml") {
        ("setup", "Point Setup / Env at the missing env_key; do not invent an Azure client.")
    } else {
        ("map", "Record the fact/rule on the harness map so the next run starts with it.")
    };
    let gap = gaps
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.eq_ignore_ascii_case("gaps"))
        .unwrap_or(gaps)
        .chars()
        .take(280)
        .collect();
    ClassifiedGap {
        category: category.into(),
        gap,
        promote: promote.into(),
    }
}

pub fn offer_promotion(rec: &mut HarnessRecord, classified: &ClassifiedGap) {
    if rec
        .promotions
        .iter()
        .any(|p| p.gap == classified.gap && p.category == classified.category)
    {
        return;
    }
    rec.promotions.push(HarnessPromotion {
        id: Uuid::new_v4().to_string(),
        category: classified.category.clone(),
        gap: classified.gap.clone(),
        patch: classified.promote.clone(),
        status: "offered".into(),
        created_at: Utc::now().to_rfc3339(),
        promoted_at: None,
    });
}

pub fn migrate_harness(conn: &Connection) -> Result<(), String> {
    let _ = conn.execute(
        "ALTER TABLE hillclimb_runs ADD COLUMN harness_json TEXT NOT NULL DEFAULT '{}'",
        [],
    );
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS harness_map (
            id TEXT PRIMARY KEY,
            json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_promotions (
            id TEXT PRIMARY KEY,
            run_id TEXT,
            category TEXT NOT NULL,
            gap TEXT NOT NULL,
            patch TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            promoted_at TEXT
        );
        ",
    )
    .map_err(|e| format!("migrate harness: {e}"))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessMap {
    pub promotions: Vec<HarnessPromotion>,
    pub notes: Vec<String>,
    pub updated_at: String,
}

impl Default for HarnessMap {
    fn default() -> Self {
        Self {
            promotions: Vec::new(),
            notes: Vec::new(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

pub fn load_map(conn: &Connection) -> Result<HarnessMap, String> {
    let json: Result<String, _> = conn.query_row(
        "SELECT json FROM harness_map WHERE id = 'local'",
        [],
        |row| row.get(0),
    );
    match json {
        Ok(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
        Err(_) => Ok(HarnessMap::default()),
    }
}

pub fn save_map(conn: &Connection, map: &HarnessMap) -> Result<(), String> {
    let json = serde_json::to_string(map).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO harness_map (id, json, updated_at) VALUES ('local', ?1, ?2)
         ON CONFLICT(id) DO UPDATE SET json = ?1, updated_at = ?2",
        params![json, map.updated_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn promote(
    conn: &Connection,
    rec: &mut HarnessRecord,
    promotion_id: &str,
    auto: bool,
) -> Result<HarnessPromotion, String> {
    let promo = rec
        .promotions
        .iter_mut()
        .find(|p| p.id == promotion_id)
        .ok_or_else(|| "Promotion not found.".to_string())?;
    promo.status = if auto { "auto-promoted" } else { "promoted" }.into();
    promo.promoted_at = Some(Utc::now().to_rfc3339());
    let snapshot = promo.clone();
    let mut map = load_map(conn)?;
    map.promotions.insert(0, snapshot.clone());
    map.notes.insert(
        0,
        format!(
            "[{}] {} — {}",
            snapshot.category, snapshot.gap, snapshot.patch
        ),
    );
    map.notes.truncate(40);
    map.promotions.truncate(40);
    map.updated_at = Utc::now().to_rfc3339();
    save_map(conn, &map)?;
    conn.execute(
        "INSERT INTO harness_promotions (id, run_id, category, gap, patch, status, created_at, promoted_at)
         VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            snapshot.id,
            snapshot.category,
            snapshot.gap,
            snapshot.patch,
            snapshot.status,
            snapshot.created_at,
            snapshot.promoted_at
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(snapshot)
}

pub fn list_promotions(conn: &Connection) -> Result<Vec<HarnessPromotion>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category, gap, patch, status, created_at, promoted_at
             FROM harness_promotions ORDER BY created_at DESC LIMIT 40",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(HarnessPromotion {
                id: row.get(0)?,
                category: row.get(1)?,
                gap: row.get(2)?,
                patch: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                promoted_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn parse_record(json: &str) -> HarnessRecord {
    if json.trim().is_empty() || json.trim() == "{}" {
        return HarnessRecord::default();
    }
    serde_json::from_str(json).unwrap_or_default()
}

pub fn to_json(rec: &HarnessRecord) -> String {
    serde_json::to_string(rec).unwrap_or_else(|_| "{}".into())
}

pub fn apply_scores(rec: &mut HarnessRecord, input: ScoreInput<'_>) {
    rec.jobs = score_jobs(&input);
    rec.sandbox = input.sandbox.into();
}

pub fn recovery_phase_for(grade: &str, classified: bool, passed_after_fail: bool) -> &'static str {
    if passed_after_fail {
        "accept"
    } else if grade == "HOLD" || grade == "WARN" {
        if classified {
            "classify"
        } else {
            "observe"
        }
    } else if classified {
        "verify"
    } else {
        "observe"
    }
}

pub fn live_recovery_phase(saw_fail: bool, at: &str) -> &'static str {
    match at {
        "worker" => {
            if saw_fail {
                "patch"
            } else {
                "run"
            }
        }
        _ => {
            if saw_fail {
                "verify"
            } else {
                "observe"
            }
        }
    }
}

pub fn format_map_notes(notes: &[String]) -> Option<String> {
    if notes.is_empty() {
        None
    } else {
        Some(
            notes
                .iter()
                .filter(|n| !n.trim().is_empty())
                .map(|n| format!("- {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_jobs_always_scored() {
        let rec = new_record(
            "Fix the README smoke path without claiming ATO.",
            "A newcomer can run npm run dev and send hello.",
            Some("/workspace"),
            true,
        );
        assert_eq!(rec.jobs.len(), 6);
        assert_eq!(rec.jobs[0].name, "contract");
        assert_eq!(rec.jobs[0].status, "PASS");
        assert_eq!(rec.jobs[2].status, "PASS");
        assert_eq!(rec.autonomy_tier, "write");
        assert_eq!(rec.approval_status, "none");
    }

    #[test]
    fn empty_contract_holds() {
        let jobs = score_jobs(&ScoreInput {
            goal: "",
            criteria: "",
            workspace: "/ws",
            brief_present: true,
            sandbox: "read-only",
            iteration: 0,
            worker: "",
            grader: "",
            grade: None,
            classified_gap: None,
            allow_writes: false,
        });
        assert_eq!(jobs[0].status, "HOLD");
    }

    #[test]
    fn recovery_holds_blind_retry() {
        let jobs = score_jobs(&ScoreInput {
            goal: "g",
            criteria: "c",
            workspace: "/ws",
            brief_present: true,
            sandbox: "workspace-write",
            iteration: 1,
            worker: "tried again",
            grader: "GRADE: HOLD\nGAPS:\n1. still broken",
            grade: Some("HOLD"),
            classified_gap: None,
            allow_writes: true,
        });
        let recovery = jobs.iter().find(|j| j.name == "recovery").unwrap();
        assert_eq!(recovery.status, "HOLD");
    }

    #[test]
    fn classify_then_pass_offers_promote() {
        let classified = classify_gap("HOLD: missing unit test for autonomy gate");
        assert_eq!(classified.category, "test");
        let mut rec = HarnessRecord::default();
        offer_promotion(&mut rec, &classified);
        assert_eq!(rec.promotions.len(), 1);
        assert_eq!(rec.promotions[0].status, "offered");
    }

    #[test]
    fn parse_jobs_block() {
        let text = r#"
HARNESS-JOBS:
contract: PASS — goal set
context: PASS — OPERATOR.md
tools: WARN sandbox unknown
"#;
        let jobs = parse_job_block(text).unwrap();
        assert_eq!(jobs[0].status, "PASS");
        assert_eq!(jobs[2].name, "tools");
    }

    #[test]
    fn send_goal_requires_approval() {
        let rec = new_record("git push and deploy to staging", "merged", None, true);
        assert_eq!(rec.autonomy_tier, "send_merge_deploy");
        assert_eq!(rec.approval_status, "required");
    }

    #[test]
    fn live_phases_and_map_notes() {
        assert_eq!(live_recovery_phase(false, "worker"), "run");
        assert_eq!(live_recovery_phase(true, "worker"), "patch");
        assert_eq!(live_recovery_phase(false, "grader"), "observe");
        assert_eq!(live_recovery_phase(true, "grader"), "verify");
        assert_eq!(recovery_phase_for("PASS", true, true), "accept");
        assert_eq!(recovery_phase_for("HOLD", true, false), "classify");
        let notes = format_map_notes(&[String::from("[test] lock the gap")]).unwrap();
        assert!(notes.contains("- [test] lock the gap"));
        assert!(format_map_notes(&[]).is_none());
    }
}
