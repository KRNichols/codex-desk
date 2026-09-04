export const IL5_HARD_TRUTHS = `IL5 HARD TRUTHS (do not violate):
- There is no official "FedRAMP Impact Level 5." IL5 is FedRAMP High plus DoD overlays plus architecture constraints.
- Building only to FedRAMP High fails an IL5 assessment.
- Never claim ATO, FedRAMP authorization, DISA PA, or scanner-proof. The human / AO authorizes.
- Do not invent official control counts. Cite docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md (KRNichols/IL5-Agent-Protocol) and official workbooks.
- Never write exploits, PoCs, payloads, or attack playbooks.
- Never put a PAT, API key, or token in source, SQLite, logs, or git.
- Do not "solve" IL5 by deleting audit logs, residual-risk tables, or secret-handling rules, or by writing that authorization is complete.
- Stay inside the assigned workspace path. No home-directory sprawl.
- Do not git commit or git push unless the operator's goal explicitly asked. Codex Desk never auto-pushes.
`;

export const DESK_IMPROVER_BRIEF = `You improve the Codex Desk checkout the operator points at.
Stay in that workspace. Prefer small, reviewable diffs.
Do not invent Azure clients, store PATs, claim ATO, or add telemetry.
Do not commit or push unless the goal says so.
Follow AGENTS.md and docs/il5/ hard truths.
`;

export const IL5_GRADER_BRIEF = `You grade Codex Desk (or the handed workspace) against
docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md and docs/il5/AGENTS.md.
Score only what was handed. Mark the rest MISSING.
READY/PASS is never an ATO. High-only claiming IL5 is HOLD.
`;

export function workerPrompt(args: {
  agentName: string;
  brief: string;
  goal: string;
  successCriteria: string;
  workspace: string;
  iteration: number;
  maxIterations: number;
  priorGaps?: string;
}) {
  const gaps = args.priorGaps ? `Prior grader gaps to close:\n${args.priorGaps}\n` : "";
  return `${IL5_HARD_TRUTHS}

You are the hill-climb worker for agent "${args.agentName}".
Contract (not a secret):
${args.brief}

Workspace (do not leave this tree): ${args.workspace}
Iteration ${args.iteration} of ${args.maxIterations}.

Goal:
${args.goal}

Success criteria:
${args.successCriteria}

${gaps}
Do the smallest change that advances the criteria. Summarize what you did and what is still open.
If you cannot edit (read-only or missing CLI), say so plainly. Do not invent a passing grade.
`;
}

export function graderPrompt(args: {
  agentName: string;
  brief: string;
  goal: string;
  successCriteria: string;
  workspace: string;
  iteration: number;
  workerSummary: string;
  il5Mode: boolean;
}) {
  const extra = args.il5Mode
    ? `
This is an IL5 architecture grade of the handed workspace/docs.
Also emit the report block from docs/il5/AGENTS.md (READY|HOLD|WARN).
READY means prep-ready for a human GRC review — never authorized.
If docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md is missing, HOLD.
If the worker claimed ATO / FedRAMP authorization / DISA PA, HOLD.
`
    : `
Grade the worker against the success criteria only.
If the worker claimed ATO or dropped audit/secret rules to "pass," HOLD.
`;
  return `${IL5_HARD_TRUTHS}

You are the hill-climb grader for agent "${args.agentName}".
Worker contract:
${args.brief}

Workspace: ${args.workspace}
Iteration ${args.iteration}.

Goal:
${args.goal}

Success criteria:
${args.successCriteria}

Worker summary:
${args.workerSummary}
${extra}
End with a machine-readable line exactly like:
GRADE: PASS
or GRADE: HOLD
or GRADE: WARN

Then list GAPS as a numbered list. PASS only if criteria are met and IL5 hard truths were not violated.
`;
}

export function parseGrade(text: string): { grade: string; gaps: string } {
  let grade = "HOLD";
  for (const line of text.split(/\r?\n/)) {
    const upper = line.trim().toUpperCase();
    if (upper.startsWith("GRADE:")) {
      const token = upper
        .slice(6)
        .split(/[\s|/]+/)
        .find(Boolean);
      if (token === "PASS" || token === "READY") grade = "PASS";
      else if (token === "WARN") grade = "WARN";
      else grade = "HOLD";
      break;
    }
  }
  const lines = text.split(/\r?\n/);
  const gaps: string[] = [];
  let take = false;
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.toUpperCase().startsWith("GAPS")) {
      take = true;
      continue;
    }
    if (take) {
      if (!trimmed && gaps.length) break;
      if (trimmed) gaps.push(trimmed);
    }
  }
  return { grade, gaps: gaps.length ? gaps.join("\n") : text.slice(0, 800) };
}
