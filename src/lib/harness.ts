import { classifyGoal, consequenceLabel, type Consequence } from "./autonomy";
import type { HarnessJob, HarnessPromotion, HarnessRecord } from "./types";

export const JOB_NAMES = ["contract", "context", "tools", "state", "evidence", "recovery"] as const;

export const JOB_CARDS = [
  { name: "contract", num: "01", title: "Contract", purpose: "Define the goal, constraints and done" },
  { name: "context", num: "02", title: "Context", purpose: "Select rules, facts and current state" },
  { name: "tools", num: "03", title: "Tools", purpose: "Act through schemas, permissions and sandboxes" },
  { name: "state", num: "04", title: "State", purpose: "Persist decisions, artifacts and open risks" },
  { name: "evidence", num: "05", title: "Evidence", purpose: "Verify with tests, sources and screenshots" },
  { name: "recovery", num: "06", title: "Recovery", purpose: "Retry locally, escalate and improve the system" },
] as const;

export const RECOVERY_STEPS = [
  { id: "run", label: "Run", blurb: "produce work" },
  { id: "observe", label: "Observe", blurb: "capture evidence" },
  { id: "classify", label: "Classify", blurb: "name the gap" },
  { id: "patch", label: "Patch", blurb: "repair locally" },
  { id: "verify", label: "Verify", blurb: "test exact case" },
  { id: "accept", label: "Accept", blurb: "ship with trace" },
] as const;

export const PROMOTE_KINDS = [
  { id: "map", label: "Update a map" },
  { id: "tool", label: "Improve a tool" },
  { id: "policy", label: "Tighten a policy" },
  { id: "test", label: "Add a test" },
] as const;

export function jobLabel(name: string): string {
  switch (name) {
    case "contract":
      return "1 Contract — goal, constraints, done";
    case "context":
      return "2 Context — rules, facts, current state";
    case "tools":
      return "3 Tools — schemas, permissions, sandboxes";
    case "state":
      return "4 State — persist decisions, artifacts, open risks";
    case "evidence":
      return "5 Evidence — tests, sources, screenshots";
    case "recovery":
      return "6 Recovery — retry locally, escalate, improve the system";
    default:
      return name;
  }
}

export type ScoreInput = {
  goal: string;
  criteria: string;
  workspace: string;
  briefPresent: boolean;
  sandbox: string;
  iteration: number;
  worker: string;
  grader: string;
  grade?: string | null;
  classifiedGap?: string | null;
  allowWrites: boolean;
};

function scoreOne(name: string, input: ScoreInput): HarnessJob {
  let status = "WARN";
  let summary = "Not scored yet.";
  if (name === "contract") {
    if (!input.goal.trim() || !input.criteria.trim()) {
      status = "HOLD";
      summary = "Goal or done criteria missing.";
    } else {
      status = "PASS";
      summary = "Goal, constraints, and done criteria are on the run record.";
    }
  } else if (name === "context") {
    if (!input.briefPresent) {
      status = "HOLD";
      summary = "Worker brief / operator contract missing.";
    } else {
      status = "PASS";
      summary = `Operator contract + rules in prompt. Workspace: ${input.workspace || "(app-data)"}`;
    }
  } else if (name === "tools") {
    if (input.sandbox === "read-only" || input.sandbox === "workspace-write") {
      status = "PASS";
      summary = `Sandbox ${input.sandbox} · allowlist local-codex-only. YOLO writes when workspace is set.`;
    } else {
      status = "HOLD";
      summary = "Sandbox / allowlist not recorded.";
    }
  } else if (name === "state") {
    if (input.iteration <= 0) {
      status = "WARN";
      summary = "Run queued — no iteration persisted yet.";
    } else {
      status = "PASS";
      summary = `Iteration ${input.iteration} persisted with decisions / gaps on the run record.`;
    }
  } else if (name === "evidence") {
    const blob = `${input.worker}\n${input.grader}`.toLowerCase();
    const has = ["test", "cargo test", "npm test", "screenshot", "evidence", "src/", "docs/"].some((k) =>
      blob.includes(k),
    );
    if (input.grade === "HOLD" && !has) {
      status = "HOLD";
      summary = "Grade is HOLD and the worker/grader cited no tests, sources, or files.";
    } else if (has) {
      status = "PASS";
      summary = "Worker/grader cited tests, sources, or files.";
    } else if (input.iteration <= 0) {
      status = "WARN";
      summary = "No evidence yet — run has not produced a worker summary.";
    } else {
      status = "WARN";
      summary = "Work happened; cite a test, source, or screenshot next.";
    }
  } else if (name === "recovery") {
    if (input.grade === "HOLD" || input.grade === "WARN") {
      if (input.classifiedGap?.trim()) {
        status = "PASS";
        summary = `Gap classified (not a blind retry): ${input.classifiedGap}`;
      } else {
        status = "HOLD";
        summary = "Failure without Classify. Return the exact gap; do not retry blindly.";
      }
    } else if (input.grade === "PASS" && input.classifiedGap) {
      status = "PASS";
      summary = "Verified after Classify → Patch. Offer / promote into the harness.";
    } else if (input.iteration <= 0) {
      status = "WARN";
      summary = "Recovery idle until a fail is observed.";
    } else {
      status = "PASS";
      summary = "No open recovery — last grade is not a fail.";
    }
  }
  return { name, label: jobLabel(name), status, summary };
}

function rankStatus(status: string): number {
  if (status === "HOLD") return 2;
  if (status === "WARN") return 1;
  return 0;
}

function normalizeStatus(token: string): string {
  const t = token.trim().toUpperCase();
  if (t === "PASS" || t === "READY") return "PASS";
  if (t === "WARN") return "WARN";
  if (t === "HOLD") return "HOLD";
  return "WARN";
}

/** Parse the grader/worker `HARNESS-JOBS:` block. */
export function parseJobBlock(text: string): HarnessJob[] | null {
  const upper = text.toUpperCase();
  const start = upper.indexOf("HARNESS-JOBS:");
  if (start < 0) return null;
  const rest = text.slice(start + "HARNESS-JOBS:".length);
  const jobs: HarnessJob[] = [];
  for (const line of rest.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed && jobs.length) break;
    const split = trimmed.indexOf(":");
    if (split < 0) continue;
    const name = trimmed.slice(0, split).trim().toLowerCase();
    if (!JOB_NAMES.includes(name as (typeof JOB_NAMES)[number])) continue;
    const body = trimmed.slice(split + 1).trim();
    let status = "WARN";
    let summary = "";
    const em = body.indexOf("—");
    const dash = body.indexOf(" - ");
    if (em >= 0) {
      status = normalizeStatus(body.slice(0, em));
      summary = body.slice(em + 1).trim();
    } else if (dash >= 0) {
      status = normalizeStatus(body.slice(0, dash));
      summary = body.slice(dash + 3).trim();
    } else {
      const [tok, ...more] = body.split(/\s+/);
      status = normalizeStatus(tok ?? "WARN");
      summary = more.join(" ").trim();
    }
    jobs.push({ name, label: jobLabel(name), status, summary });
  }
  return jobs.length ? jobs : null;
}

export function scoreJobs(input: ScoreInput): HarnessJob[] {
  const parsed = parseJobBlock(input.worker) ?? parseJobBlock(input.grader);
  return JOB_NAMES.map((name) => {
    const deterministic = scoreOne(name, input);
    const fromModel = parsed?.find((j) => j.name === name);
    if (!fromModel) return deterministic;
    if (rankStatus(deterministic.status) > rankStatus(fromModel.status)) return deterministic;
    return { ...deterministic, status: fromModel.status, summary: fromModel.summary || deterministic.summary };
  });
}

export function recoveryPhaseFor(grade: string, classified: boolean, passedAfterFail: boolean): string {
  if (passedAfterFail) return "accept";
  if (grade === "HOLD" || grade === "WARN") return classified ? "classify" : "observe";
  if (classified) return "verify";
  return "observe";
}

export function liveRecoveryPhase(sawFail: boolean, at: "worker" | "grader"): string {
  if (at === "worker") return sawFail ? "patch" : "run";
  return sawFail ? "verify" : "observe";
}

export function formatHarnessMapNotes(notes: string[]): string {
  return notes
    .filter((n) => n.trim())
    .map((n) => `- ${n}`)
    .join("\n");
}

export function newHarnessRecord(
  goal: string,
  criteria: string,
  workspace: string | null | undefined,
  allowWrites: boolean,
): HarnessRecord {
  const tier = classifyGoal(goal, criteria);
  const approval =
    tier === "send_merge_deploy" ? "required" : tier === "delete_pay_publish" ? "confirm_required" : "none";
  const sandbox = allowWrites ? "workspace-write" : "read-only";
  return {
    jobs: scoreJobs({
      goal,
      criteria,
      workspace: workspace ?? "",
      briefPresent: true,
      sandbox,
      iteration: 0,
      worker: "",
      grader: "",
      allowWrites,
    }),
    autonomy_tier: tier,
    autonomy_label: consequenceLabel(tier),
    approval_status: approval,
    approval_evidence: null,
    classified_gap: null,
    gap_category: null,
    recovery_phase: "observe",
    promotions: [],
    sandbox,
    allowlist: "local-codex-only",
  };
}

export function classifyGap(gaps: string): { category: string; gap: string; promote: string } {
  const lower = gaps.toLowerCase();
  let category = "map";
  let promote = "Record the fact/rule on the harness map so the next run starts with it.";
  if (lower.includes("test") || lower.includes("unvalidated")) {
    category = "test";
    promote = "Add or run a test that locks the gap.";
  } else if (lower.includes("policy") || lower.includes("hold:") || lower.includes("ato")) {
    category = "policy";
    promote = "Tighten policy.rs / preview policy so the next run HOLDs this fail.";
  } else if (lower.includes("brief") || lower.includes("operator.md") || lower.includes("contract")) {
    category = "brief";
    promote = "Fix the worker brief or OPERATOR.md hook.";
  } else if (lower.includes("sandbox") || lower.includes("allowlist") || lower.includes("tool")) {
    category = "tool";
    promote = "Improve the tool schema, sandbox, or allowlist note.";
  } else if (lower.includes("loop") || lower.includes("iteration") || lower.includes("retry")) {
    category = "loop";
    promote = "Fix the hill-climb loop so Classify happens before the next patch.";
  } else if (lower.includes("setup") || lower.includes("env_key") || lower.includes("config.toml")) {
    category = "setup";
    promote = "Point Setup / Env at the missing env_key; do not invent an Azure client.";
  }
  const gap = (gaps.split(/\r?\n/).map((l) => l.trim()).find((l) => l && l.toLowerCase() !== "gaps") ?? gaps).slice(
    0,
    280,
  );
  return { category, gap, promote };
}

export function emptyHarness(): HarnessRecord {
  return newHarnessRecord("", "", null, false);
}

export function asConsequence(value?: string | null): Consequence {
  if (value === "write" || value === "send_merge_deploy" || value === "delete_pay_publish") return value;
  return "read";
}

export function offerPromotion(rec: HarnessRecord, classified: { category: string; gap: string; promote: string }) {
  if (rec.promotions.some((p) => p.gap === classified.gap && p.category === classified.category)) return;
  const promo: HarnessPromotion = {
    id: crypto.randomUUID(),
    category: classified.category,
    gap: classified.gap,
    patch: classified.promote,
    status: "offered",
    created_at: new Date().toISOString(),
    promoted_at: null,
  };
  rec.promotions.push(promo);
}
