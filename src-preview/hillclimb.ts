import { existsSync, mkdirSync, readFileSync, writeFileSync, appendFileSync } from "node:fs";
import { execSync, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { randomUUID } from "node:crypto";
import path from "node:path";
import { DESK_IMPROVER_BRIEF, IL5_GRADER_BRIEF, graderPrompt, parseGrade, workerPrompt } from "../src/lib/prompts";
import type { Agent, AuditEvent, HillclimbIteration, HillclimbRun } from "../src/lib/types";

const DATA_DIR = path.resolve(process.cwd(), ".data");
const STORE_PATH = path.join(DATA_DIR, "preview-store.json");
const AUDIT_PATH = path.join(DATA_DIR, "audit.jsonl");

type StoreFile = {
  agents?: Agent[];
  runs?: HillclimbRun[];
  iterations?: HillclimbIteration[];
  audit?: AuditEvent[];
};

const cancels = new Set<string>();

function nowIso() {
  return new Date().toISOString();
}

function load(): StoreFile {
  if (!existsSync(STORE_PATH)) return {};
  try {
    return JSON.parse(readFileSync(STORE_PATH, "utf8")) as StoreFile;
  } catch {
    return {};
  }
}

function savePartial(patch: StoreFile) {
  const current = load();
  const next = { ...current, ...patch };
  mkdirSync(DATA_DIR, { recursive: true });
  // merge into existing preview-store without wiping chats
  let full: Record<string, unknown> = {};
  try {
    full = JSON.parse(readFileSync(STORE_PATH, "utf8")) as Record<string, unknown>;
  } catch {
    full = {};
  }
  writeFileSync(STORE_PATH, JSON.stringify({ ...full, ...next }, null, 2));
}

function audit(action: string, entityType: string, entityId: string, detail: string) {
  const event: AuditEvent = {
    id: randomUUID(),
    at: nowIso(),
    action,
    actor: "local-user",
    entity_type: entityType,
    entity_id: entityId,
    detail,
  };
  const store = load();
  const list = store.audit ?? [];
  list.unshift(event);
  savePartial({ audit: list.slice(0, 200) });
  mkdirSync(DATA_DIR, { recursive: true });
  appendFileSync(AUDIT_PATH, `${JSON.stringify(event)}\n`);
}

export function ensureAgents(): Agent[] {
  const store = load();
  let agents = store.agents ?? [];
  if (!agents.some((a) => a.template === "desk-improver")) {
    agents.push(makeAgent("Desk Improver", DESK_IMPROVER_BRIEF, "desk-improver"));
  }
  if (!agents.some((a) => a.template === "il5-grader")) {
    agents.push(makeAgent("IL5 Architecture Grader", IL5_GRADER_BRIEF, "il5-grader"));
  }
  savePartial({ agents });
  return agents;
}

function makeAgent(name: string, brief: string, template: string, workspace?: string): Agent {
  const now = nowIso();
  return {
    id: randomUUID(),
    name,
    brief,
    template,
    status: "idle",
    workspace_path: workspace ?? null,
    chat_id: null,
    worker_thread_id: null,
    grader_thread_id: null,
    created_at: now,
    updated_at: now,
  };
}

export function listAgents() {
  return ensureAgents();
}

export function createAgent(name: string, brief: string, workspace?: string) {
  const agents = ensureAgents();
  const agent = makeAgent(name, brief, "custom", workspace);
  agents.push(agent);
  savePartial({ agents });
  audit("agent.create", "agent", agent.id, agent.name);
  return agent;
}

export function updateAgent(id: string, patch: { name?: string; brief?: string; workspace_path?: string }) {
  const agents = ensureAgents();
  const agent = agents.find((a) => a.id === id);
  if (!agent) throw new Error("Agent not found.");
  if (patch.name !== undefined) agent.name = patch.name;
  if (patch.brief !== undefined) agent.brief = patch.brief;
  if (patch.workspace_path !== undefined) {
    agent.workspace_path = patch.workspace_path.trim() ? patch.workspace_path.trim() : null;
  }
  agent.updated_at = nowIso();
  savePartial({ agents });
  audit("agent.update", "agent", agent.id, "updated");
  return agent;
}

export function listRuns(agentId: string) {
  return (load().runs ?? [])
    .filter((r) => r.agent_id === agentId)
    .sort((a, b) => b.created_at.localeCompare(a.created_at));
}

export function getRun(runId: string) {
  const run = (load().runs ?? []).find((r) => r.id === runId);
  if (!run) throw new Error("Run not found.");
  const iterations = (load().iterations ?? []).filter((i) => i.run_id === runId);
  return { run, iterations };
}

export function listAudit() {
  return (load().audit ?? []).slice(0, 50);
}

function whichCodex(): string | null {
  try {
    const found = execSync(process.platform === "win32" ? "where codex" : "command -v codex", {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    })
      .split(/\r?\n/)[0]
      ?.trim();
    return found || null;
  } catch {
    return null;
  }
}

function runCodex(
  binary: string,
  prompt: string,
  workdir: string,
  sandbox: string,
  threadId: string | null,
): Promise<{ text: string; threadId: string | null }> {
  return new Promise((resolve, reject) => {
    mkdirSync(workdir, { recursive: true });
    const args = ["exec"];
    if (threadId) args.push("resume", threadId);
    args.push("--json", "--skip-git-repo-check", "--sandbox", sandbox, "--ask-for-approval", "never", "-");
    const child = spawn(binary, args, {
      cwd: workdir,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
      env: process.env,
    });
    child.stdin.write(prompt);
    child.stdin.end();
    let assistant = "";
    let seen = threadId;
    const stderr: string[] = [];
    const rl = createInterface({ input: child.stdout });
    rl.on("line", (line) => {
      try {
        const value = JSON.parse(line) as { type?: string; thread_id?: string; item?: { type?: string; text?: string } };
        if (value.type === "thread.started" && value.thread_id) seen = value.thread_id;
        if (value.item?.type === "agent_message" && value.item.text) assistant = value.item.text;
      } catch {
        if (line.trim()) assistant = assistant ? `${assistant}\n${line}` : line;
      }
    });
    createInterface({ input: child.stderr }).on("line", (l) => l.trim() && stderr.push(l.trim()));
    child.on("error", (err) => reject(err));
    child.on("close", (code) => {
      if (code !== 0) reject(new Error(stderr.join("\n") || assistant || `exit ${code}`));
      else resolve({ text: assistant || stderr.join("\n") || "(no reply)", threadId: seen });
    });
  });
}

function upsertRun(run: HillclimbRun) {
  const runs = load().runs ?? [];
  const idx = runs.findIndex((r) => r.id === run.id);
  if (idx >= 0) runs[idx] = run;
  else runs.unshift(run);
  savePartial({ runs });
}

function addIteration(item: HillclimbIteration) {
  const iterations = load().iterations ?? [];
  iterations.push(item);
  savePartial({ iterations });
}

function updateAgentStatus(id: string, status: string, threads?: { worker?: string; grader?: string }) {
  const agents = ensureAgents();
  const agent = agents.find((a) => a.id === id);
  if (!agent) return;
  agent.status = status;
  if (threads?.worker) agent.worker_thread_id = threads.worker;
  if (threads?.grader) agent.grader_thread_id = threads.grader;
  agent.updated_at = nowIso();
  savePartial({ agents });
}

export function startRun(agentId: string, goal: string, successCriteria: string, maxIterations: number, allowWrites: boolean) {
  const agent = ensureAgents().find((a) => a.id === agentId);
  if (!agent) throw new Error("Agent not found.");
  const run: HillclimbRun = {
    id: randomUUID(),
    agent_id: agentId,
    goal,
    success_criteria: successCriteria,
    max_iterations: Math.min(12, Math.max(1, maxIterations)),
    current_iteration: 0,
    status: "queued",
    last_grade: null,
    last_gaps: null,
    allow_writes: allowWrites,
    created_at: nowIso(),
    updated_at: nowIso(),
  };
  upsertRun(run);
  updateAgentStatus(agentId, "running");
  audit("hillclimb.start", "run", run.id, `agent=${agent.name} writes=${allowWrites}`);
  setImmediate(() => void loop(run.id));
  return run;
}

export function cancelRun(runId: string) {
  cancels.add(runId);
  const { run } = getRun(runId);
  run.status = "cancelled";
  run.updated_at = nowIso();
  upsertRun(run);
  updateAgentStatus(run.agent_id, "idle");
  audit("hillclimb.cancel", "run", runId, "user cancel");
  return run;
}

async function loop(runId: string) {
  const binary = whichCodex();
  let { run } = getRun(runId);
  const agent = ensureAgents().find((a) => a.id === run.agent_id);
  if (!agent) return;
  if (!binary) {
    run.status = "error";
    run.last_grade = "HOLD";
    run.last_gaps = "The `codex` CLI was not found on PATH. Hill-climb cannot start.";
    run.updated_at = nowIso();
    upsertRun(run);
    updateAgentStatus(agent.id, "blocked");
    audit("secret.access_failure", "run", runId, "Codex missing (value not logged)");
    return;
  }

  const workdir = agent.workspace_path && existsSync(agent.workspace_path)
    ? agent.workspace_path
    : path.join(DATA_DIR, "workspace");
  if (agent.workspace_path && (agent.workspace_path === process.env.HOME || agent.workspace_path === process.env.USERPROFILE)) {
    run.status = "error";
    run.last_gaps = "Refusing the user home directory as a workspace.";
    run.last_grade = "HOLD";
    upsertRun(run);
    updateAgentStatus(agent.id, "blocked");
    return;
  }
  mkdirSync(workdir, { recursive: true });
  const writes = run.allow_writes && Boolean(agent.workspace_path);
  let gaps: string | undefined;
  for (let i = 1; i <= run.max_iterations; i += 1) {
    if (cancels.has(runId)) break;
    run = getRun(runId).run;
    run.current_iteration = i;
    run.status = "running";
    run.updated_at = nowIso();
    upsertRun(run);
    audit("hillclimb.iteration", "run", runId, `iteration=${i} phase=worker`);
    const wprompt = workerPrompt({
      agentName: agent.name,
      brief: agent.brief,
      goal: run.goal,
      successCriteria: run.success_criteria,
      workspace: workdir,
      iteration: i,
      maxIterations: run.max_iterations,
      priorGaps: gaps,
    });
    try {
      const worker = await runCodex(binary, wprompt, workdir, writes ? "workspace-write" : "read-only", agent.worker_thread_id);
      if (worker.threadId) updateAgentStatus(agent.id, "running", { worker: worker.threadId });
      addIteration({
        id: randomUUID(),
        run_id: runId,
        iteration: i,
        phase: "worker",
        worker_summary: worker.text,
        grade: null,
        gaps: null,
        created_at: nowIso(),
      });
      const gprompt = graderPrompt({
        agentName: agent.name,
        brief: agent.brief,
        goal: run.goal,
        successCriteria: run.success_criteria,
        workspace: workdir,
        iteration: i,
        workerSummary: worker.text,
        il5Mode: agent.template === "il5-grader",
      });
      const grader = await runCodex(binary, gprompt, workdir, "read-only", agent.grader_thread_id);
      if (grader.threadId) updateAgentStatus(agent.id, "running", { grader: grader.threadId });
      const parsed = parseGrade(grader.text);
      gaps = parsed.gaps;
      addIteration({
        id: randomUUID(),
        run_id: runId,
        iteration: i,
        phase: "grader",
        worker_summary: grader.text,
        grade: parsed.grade,
        gaps: parsed.gaps,
        created_at: nowIso(),
      });
      run.last_grade = parsed.grade;
      run.last_gaps = parsed.gaps;
      run.current_iteration = i;
      const done = parsed.grade === "PASS" || i >= run.max_iterations;
      run.status = parsed.grade === "PASS" ? "passed" : done ? "hold" : "running";
      run.updated_at = nowIso();
      upsertRun(run);
      audit("hillclimb.grade", "run", runId, `iteration=${i} grade=${parsed.grade}`);
      if (done) {
        updateAgentStatus(agent.id, parsed.grade === "PASS" ? "done" : "blocked");
        audit("hillclimb.stop", "run", runId, `status=${run.status}`);
        return;
      }
    } catch (err) {
      run.status = "error";
      run.last_grade = "HOLD";
      run.last_gaps = err instanceof Error ? err.message : String(err);
      run.updated_at = nowIso();
      upsertRun(run);
      updateAgentStatus(agent.id, "blocked");
      return;
    }
  }
}
