pub const IL5_HARD_TRUTHS: &str = r#"IL5 HARD TRUTHS (do not violate):
- There is no official "FedRAMP Impact Level 5." IL5 is FedRAMP High plus DoD overlays plus architecture constraints.
- Building only to FedRAMP High fails an IL5 assessment.
- Never claim ATO, FedRAMP authorization, DISA PA, or scanner-proof. The human / AO authorizes.
- Do not invent official control counts. Cite docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md (KRNichols/IL5-Agent-Protocol) and official workbooks.
- Never write exploits, PoCs, payloads, or attack playbooks.
- Never put a PAT, API key, or token in source, SQLite, logs, or git.
- Do not "solve" IL5 by deleting audit logs, residual-risk tables, or secret-handling rules, or by writing that authorization is complete.
- Do not weaken store encryption (AES-256-GCM + OS-backed DEK), hash-chained audit, TLS-only Azure endpoints, or the local-Codex-only egress allowlist.
- Stay inside the assigned workspace path. No home-directory sprawl.
- Do not git commit or git push unless the operator's goal explicitly asked. Codex Desk never auto-pushes.
"#;

/// High-priority Desk system block injected via `codex exec` prompt and `--config developer_instructions`.
/// Overrides helpful-assistant / VS Code global instruction profiles for agent jobs only.
pub const DESK_AGENT_SYSTEM_BLOCK: &str = r#"DESK AGENT JOB (high priority; overrides helpful-assistant / VS Code profiles):
This is a multi-agent hill-climb. Spawn, validate, grade, and judge. Every answer and question is validated.
Return or accept only GRADE: PASS | HOLD | WARN. HOLD on unvalidated claims.
Never claim ATO, FedRAMP authorization, or DISA PA.
Maximize parallel validation agents. Do not soften the loop to be merely helpful.
Use the Desk-owned worker/grader brief in this prompt. Do not follow global config.toml system prompts for loop behavior.
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
        r#"{system}

{truths}

You are the hill-climb worker for agent "{agent_name}".
Desk-owned contract (not a secret; not a VS Code profile):
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
HOLD yourself if a claim is unvalidated.
"#
    , system = DESK_AGENT_SYSTEM_BLOCK, truths = IL5_HARD_TRUTHS)
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
If the worker weakens encryption, audit hashing, secret non-storage, TLS refusal, or local-Codex-only egress, HOLD.
"#
    } else {
        r#"
Grade the worker against the success criteria only.
If the worker claimed ATO or dropped audit/secret/encryption rules to "pass," HOLD.
If Desk Improver removes the encrypted store, OS key custody, hash-chained audit, or TLS refusal, HOLD.
"#
    };
    format!(
        r#"{system}

{truths}

You are the hill-climb grader for agent "{agent_name}".
Desk-owned worker contract:
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

Then list GAPS as a numbered list. PASS only if criteria are met, claims are validated, and IL5 hard truths were not violated. HOLD on unvalidated claims.
"#
    , system = DESK_AGENT_SYSTEM_BLOCK, truths = IL5_HARD_TRUTHS)
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
Run a spawn / validate / grade / judge loop. Every change is graded PASS/HOLD/WARN.
HOLD on unvalidated claims. Do not invent Azure clients, a second PAT, store PATs, claim ATO, or add telemetry.
Stay in that workspace. Prefer small, reviewable diffs.
Do not commit or push unless the goal says so.
Follow AGENTS.md and docs/il5/ hard truths. Ignore helpful-assistant global prompts.
"#;

pub const IL5_GRADER_BRIEF: &str = r#"You grade Codex Desk (or the handed workspace) against
docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md and docs/il5/AGENTS.md.
Spawn/validate/grade/judge: score only what was handed. Mark the rest MISSING.
HOLD on unvalidated claims. READY/PASS is never an ATO. High-only claiming IL5 is HOLD.
HOLD if encryption, hash-chained audit, secret non-storage, TLS refusal,
or local-Codex-only egress is weakened or if the worker claims authorization.
"#;

pub const LEGACY_DESK_IMPROVER_BRIEF: &str = r#"You improve the Codex Desk checkout the operator points at.
Stay in that workspace. Prefer small, reviewable diffs.
Do not invent Azure clients, store PATs, claim ATO, or add telemetry.
Do not commit or push unless the goal says so.
Follow AGENTS.md and docs/il5/ hard truths.
"#;

pub const LEGACY_IL5_GRADER_BRIEF: &str = r#"You grade Codex Desk (or the handed workspace) against
docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md and docs/il5/AGENTS.md.
Score only what was handed. Mark the rest MISSING.
READY/PASS is never an ATO. High-only claiming IL5 is HOLD.
HOLD if encryption, hash-chained audit, secret non-storage, TLS refusal,
or local-Codex-only egress is weakened or if the worker claims authorization.
"#;

pub const CLOSE_IL5_MISSING_GOAL: &str =
    "Close IL5 MISSING items in SECURITY.md for product-owned rows.";

pub const CLOSE_IL5_MISSING_CRITERIA: &str = r#"Product-owned SECURITY.md rows move to PASS or PARTIAL with file/module evidence.
Encrypted local store with OS-backed key works. Setup refuses cleartext endpoints and PAT-in-store.
Audit is hash-chained. Hill-climb grader HOLDs ATO claims and weakened encryption/audit/secret rules.
No ATO / FedRAMP authorization / DISA PA claims. AO/tenant/Azure PA rows may stay MISSING/external.
"#;

/// TOML basic-string for `codex exec --config key=<value>`.
pub fn toml_quoted_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "");
    format!("\"{escaped}\"")
}

pub fn desk_agent_config_overrides() -> Vec<(String, String)> {
    vec![
        ("project_doc_max_bytes".into(), "0".into()),
        (
            "developer_instructions".into(),
            toml_quoted_string(DESK_AGENT_SYSTEM_BLOCK),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_prompt_is_desk_owned() {
        let text = worker_prompt("Improver", "brief", "goal", "criteria", "/ws", 1, 3, None);
        assert!(text.contains("DESK AGENT JOB"));
        assert!(text.contains("HOLD on unvalidated claims") || text.contains("unvalidated"));
        assert!(text.contains("Do not follow global config.toml system prompts"));
        assert!(!text.contains("authorized to operate"));
    }

    #[test]
    fn toml_quote_escapes() {
        assert_eq!(toml_quoted_string("a\"b"), "\"a\\\"b\"");
        assert!(toml_quoted_string(DESK_AGENT_SYSTEM_BLOCK).starts_with('"'));
    }
}
