pub const IL5_HARD_TRUTHS: &str = r#"IL5 HARD TRUTHS (do not violate):
- There is no official "FedRAMP Impact Level 5." IL5 is FedRAMP High plus DoD overlays plus architecture constraints.
- Building only to FedRAMP High fails an IL5 assessment.
- Never claim ATO, FedRAMP authorization, DISA PA, or scanner-proof. The human / AO authorizes.
- Do not invent official control counts. Cite docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md (KRNichols/IL5-Agent-Protocol) and official workbooks.
- Never write exploits, PoCs, payloads, or attack playbooks.
- Never put a PAT, API key, or token in source, SQLite, logs, or git.
- Do not "solve" IL5 by deleting audit logs, residual-risk tables, or secret-handling rules, or by writing that authorization is complete.
- Stay inside the assigned workspace path. No home-directory sprawl.
- Do not git commit or git push unless the operator's goal explicitly asked. Codex Desk never auto-pushes.
"#;

pub fn worker_prompt(
    agent_name: &str,
    brief: &str,
    goal: &str,
    success_criteria: &str,
    workspace: &str,
    iteration: u32,
    max_iterations: u32,
    prior_gaps: Option<&str>,
) -> String {
    let gaps = prior_gaps
        .map(|g| format!("Prior grader gaps to close:\n{g}\n"))
        .unwrap_or_default();
    format!(
        r#"{truths}

You are the hill-climb worker for agent "{agent_name}".
Contract (not a secret):
{brief}

Workspace (do not leave this tree): {workspace}
Iteration {iteration} of {max_iterations}.

Goal:
{goal}

Success criteria:
{success_criteria}

{gaps}
Do the smallest change that advances the criteria. Summarize what you did and what is still open.
If you cannot edit (read-only or missing CLI), say so plainly. Do not invent a passing grade.
"#
    , truths = IL5_HARD_TRUTHS)
}

pub fn grader_prompt(
    agent_name: &str,
    brief: &str,
    goal: &str,
    success_criteria: &str,
    workspace: &str,
    iteration: u32,
    worker_summary: &str,
    il5_mode: bool,
) -> String {
    let extra = if il5_mode {
        r#"
This is an IL5 architecture grade of the handed workspace/docs.
Also emit the report block from docs/il5/AGENTS.md (READY|HOLD|WARN).
READY means prep-ready for a human GRC review — never authorized.
If docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md is missing, HOLD.
If the worker claimed ATO / FedRAMP authorization / DISA PA, HOLD.
"#
    } else {
        r#"
Grade the worker against the success criteria only.
If the worker claimed ATO or dropped audit/secret rules to "pass," HOLD.
"#
    };
    format!(
        r#"{truths}

You are the hill-climb grader for agent "{agent_name}".
Worker contract:
{brief}

Workspace: {workspace}
Iteration {iteration}.

Goal:
{goal}

Success criteria:
{success_criteria}

Worker summary:
{worker_summary}
{extra}
End with a machine-readable line exactly like:
GRADE: PASS
or GRADE: HOLD
or GRADE: WARN

Then list GAPS as a numbered list. PASS only if criteria are met and IL5 hard truths were not violated.
"#
    , truths = IL5_HARD_TRUTHS)
}

pub fn parse_grade(text: &str) -> (String, String) {
    let mut grade = "HOLD".to_string();
    for line in text.lines() {
        let upper = line.trim().to_ascii_uppercase();
        if let Some(rest) = upper.strip_prefix("GRADE:") {
            let token = rest
                .split(|c: char| c.is_whitespace() || c == '|' || c == '/')
                .find(|p| !p.is_empty())
                .unwrap_or("HOLD");
            grade = match token {
                "PASS" | "READY" => "PASS".into(),
                "WARN" => "WARN".into(),
                _ => "HOLD".into(),
            };
            break;
        }
    }
    let gaps = extract_gaps(text);
    (grade, gaps)
}

fn extract_gaps(text: &str) -> String {
    let mut out = Vec::new();
    let mut take = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_uppercase().starts_with("GAPS") {
            take = true;
            continue;
        }
        if take {
            if trimmed.is_empty() && !out.is_empty() {
                break;
            }
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
    }
    if out.is_empty() {
        text.chars().take(800).collect()
    } else {
        out.join("\n")
    }
}

pub const DESK_IMPROVER_BRIEF: &str = r#"You improve the Codex Desk checkout the operator points at.
Stay in that workspace. Prefer small, reviewable diffs.
Do not invent Azure clients, store PATs, claim ATO, or add telemetry.
Do not commit or push unless the goal says so.
Follow AGENTS.md and docs/il5/ hard truths.
"#;

pub const IL5_GRADER_BRIEF: &str = r#"You grade Codex Desk (or the handed workspace) against
docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md and docs/il5/AGENTS.md.
Score only what was handed. Mark the rest MISSING.
READY/PASS is never an ATO. High-only claiming IL5 is HOLD.
"#;
