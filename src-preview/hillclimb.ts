import { existsSync, mkdirSync } from "node:fs";
import { execSync, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { randomUUID } from "node:crypto";
import path from "node:path";
import {
  DESK_IMPROVER_BRIEF,
  IL5_GRADER_BRIEF,
  LEGACY_DESK_IMPROVER_BRIEF,
  LEGACY_IL5_GRADER_BRIEF,
  PRIOR_DESK_IMPROVER_BRIEF,
  PRIOR_IL5_GRADER_BRIEF,
  OPERATOR_CONTRACT,
  deskAgentExecConfigArgs,
  graderPrompt,
  parseGrade,
  workerPrompt,
} from "../src/lib/prompts";
import type {
  Agent,
  HarnessMap,
  HarnessPromotion,
  HillclimbIteration,
  HillclimbRun,
  OperatorAttestation,
} from "../src/lib/types";
import { classifyGoal, gate } from "../src/lib/autonomy";
import {
  classifyGap,
  formatHarnessMapNotes,
  liveRecoveryPhase,
  newHarnessRecord,
  offerPromotion,
  recoveryPhaseFor,
  scoreJobs,
} from "../src/lib/harness";
import { DATA_DIR, loadStore, patchStore, writeAudit } from "./secure-store";
import { childEnv } from "./setup";
import { assertLocalCodex, enforceGrade, enforceProductChecklist } from "./policy";
import { sessionUser } from "./crypto";

type StoreFile = {
  agents?: Agent[];
  runs?: HillclimbRun[];
  iterations?: HillclimbIteration[];
  identity?: { machine_binding: string; attestation: OperatorAttestation };
  harnessMap?: HarnessMap;
};

const cancels = new Set<string>();

function nowIso() {
  return new Date().toISOString();
}

function load(): StoreFile {
  return loadStore() as StoreFile;
}

function savePartial(patch: StoreFile) {
  patchStore(patch);
}

function audit(action: string, entityType: string, entityId: string, detail: string) {
  writeAudit(action, entityType, entityId, detail);
}

export function getAttestation(): OperatorAttestation {
  return (
    load().identity?.attestation ?? {
      configured: false,
      operator_name: null,
      organization: null,
      statement: null,
      at: null,
    }
  );
}

export function setAttestation(operatorName: string, organization: string, statement: string): OperatorAttestation {
  const name = operatorName.trim();
  const org = organization.trim();
  const stmt = statement.trim();
  if (!name || !org) throw new Error("Operator name and organization are required.");
  if (stmt.length < 12) throw new Error("Attestation statement is too short.");
  const record: OperatorAttestation = {
    configured: true,
    operator_name: name,
    organization: org,
    statement: stmt,
    at: nowIso(),
  };
  const store = load();
  patchStore({
    identity: {
      machine_binding: store.identity?.machine_binding ?? `${sessionUser()}`,
      attestation: record,
    },
  });
  writeAudit("identity.attest", "identity", "local", "operator attestation recorded (no secret values)");
  return record;
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
  for (const agent of agents) {
    if (
      agent.template === "desk-improver" &&
      (agent.brief.trim() === LEGACY_DESK_IMPROVER_BRIEF.trim() ||
        agent.brief.trim() === PRIOR_DESK_IMPROVER_BRIEF.trim())
    ) {
      agent.brief = DESK_IMPROVER_BRIEF;
    }
    if (
      agent.template === "il5-grader" &&
      (agent.brief.trim() === LEGACY_IL5_GRADER_BRIEF.trim() ||
        agent.brief.trim() === PRIOR_IL5_GRADER_BRIEF.trim())
    ) {
      agent.brief = IL5_GRADER_BRIEF;
    }
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
  const agent = makeAgent(name, brief.trim() || OPERATOR_CONTRACT, "custom", workspace);
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

export function exportAudit() {
  writeAudit("audit.export", "audit", "local", "operator exported hash-chained audit (no secret values)");
  return (load().audit ?? []).slice();
}

function whichCodex(): string | null {
  try {
    const found = execSync(process.platform === "win32" ? "where codex" : "command -v codex", {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    })
      .split(/\r?\n/)[0]
      ?.trim();
    if (found) return found;
  } catch {
    // fall through to extra dirs
  }
  const home = process.env.HOME || process.env.USERPROFILE || "";
  const names = process.platform === "win32" ? ["codex.cmd", "codex.exe", "codex"] : ["codex"];
  for (const dir of [path.join(home, ".npm-global", "bin"), path.join(home, ".local", "bin")]) {
    for (const name of names) {
      const candidate = path.join(dir, name);
      if (existsSync(candidate)) return candidate;
    }
  }
  return null;
}

function explainCodexFailure(detail: string): string {
  const lower = detail.toLowerCase();
  if (lower.includes("401") || lower.includes("unauthorized") || lower.includes("missing bearer")) {
    return (
      "Codex ran but could not authenticate. This desk expects Azure via ~/.codex/config.toml " +
      "(HTTPS base_url + env_key) and AZURE_LLM_PAT. Without that, Codex hits a default host and 401s. " +
      "Desk does not call Azure or send the PAT itself.\n\n" +
      detail
    );
  }
  return detail;
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
    args.push("--json", "--skip-git-repo-check", "--sandbox", sandbox, ...deskAgentExecConfigArgs(), "-");
    const child = spawn(binary, args, {
      cwd: workdir,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
      env: childEnv(),
    });
    child.stdin.write(prompt);
    child.stdin.end();
    let assistant = "";
    let seen = threadId;
    const stderr: string[] = [];
    const rl = createInterface({ input: child.stdout });
    rl.on("line", (line) => {
      try {
        const value = JSON.parse(line) as {
          type?: string;
          thread_id?: string;
          message?: string;
          error?: string | { message?: string };
          item?: { type?: string; text?: string; message?: string };
        };
        if (value.type === "thread.started" && value.thread_id) seen = value.thread_id;
        if (value.item?.type === "agent_message" && value.item.text) assistant = value.item.text;
        if (value.type === "turn.failed" || value.type === "error") {
          const nested = typeof value.error === "object" ? value.error?.message : value.error;
          assistant = nested || value.message || value.item?.message || assistant;
        }
      } catch {
        if (line.trim()) assistant = assistant ? `${assistant}\n${line}` : line;
      }
    });
    createInterface({ input: child.stderr }).on("line", (l) => l.trim() && stderr.push(l.trim()));
    const timer = setTimeout(() => child.kill("SIGTERM"), 45_000);
    child.on("error", (err) => {
      clearTimeout(timer);
      reject(err);
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      const detail = assistant || stderr.join("\n") || `exit ${code}`;
      if (code !== 0) reject(new Error(explainCodexFailure(detail)));
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

export function yoloWritesEnabled(workspacePath?: string | null): boolean {
  return Boolean(workspacePath?.trim());
}

export function startRun(
  agentId: string,
  goal: string,
  successCriteria: string,
  maxIterations: number,
  _allowWrites?: boolean,
  approvalEvidence?: string,
  confirmDestructive?: boolean,
) {
  const agent = ensureAgents().find((a) => a.id === agentId);
  if (!agent) throw new Error("Agent not found.");
  const allowWrites = yoloWritesEnabled(agent.workspace_path);
  const harness = newHarnessRecord(goal, successCriteria, agent.workspace_path, allowWrites);
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
    harness,
  };
  const blocked = gate(classifyGoal(goal, successCriteria), Boolean(approvalEvidence), Boolean(confirmDestructive), approvalEvidence);
  if (blocked) {
    run.status = harness.approval_status === "required" ? "awaiting_approval" : "awaiting_confirm";
    run.last_gaps = blocked;
    upsertRun(run);
    audit("hillclimb.gate", "run", run.id, `tier=${harness.autonomy_tier} held (no identity attestation)`);
    return run;
  }
  if (approvalEvidence) {
    harness.approval_status = "approved";
    harness.approval_evidence = approvalEvidence;
  }
  if (confirmDestructive) harness.approval_status = "confirmed";
  upsertRun(run);
  updateAgentStatus(agentId, "running");
  audit("hillclimb.start", "run", run.id, `agent=${agent.name} writes=${allowWrites} tier=${harness.autonomy_tier}`);
  setImmediate(() => void loop(run.id));
  return run;
}

export function approveRun(runId: string, evidence: string) {
  const err = gate("send_merge_deploy", true, false, evidence);
  if (err) throw new Error(err);
  const { run } = getRun(runId);
  if (!run.harness) run.harness = newHarnessRecord(run.goal, run.success_criteria, null, run.allow_writes);
  run.harness.approval_status = "approved";
  run.harness.approval_evidence = evidence.trim();
  run.status = "queued";
  run.updated_at = nowIso();
  upsertRun(run);
  updateAgentStatus(run.agent_id, "running");
  audit("hillclimb.approve", "run", runId, "send/merge/deploy approved (evidence recorded, no secret values)");
  setImmediate(() => void loop(run.id));
  return run;
}

export function confirmRun(runId: string) {
  const { run } = getRun(runId);
  if (!run.harness) run.harness = newHarnessRecord(run.goal, run.success_criteria, null, run.allow_writes);
  run.harness.approval_status = "confirmed";
  run.status = "queued";
  run.updated_at = nowIso();
  upsertRun(run);
  updateAgentStatus(run.agent_id, "running");
  audit("hillclimb.confirm", "run", runId, "delete/pay/publish confirmed by operator");
  setImmediate(() => void loop(run.id));
  return run;
}

export function promoteRun(runId: string, promotionId: string) {
  const { run } = getRun(runId);
  if (!run.harness) throw new Error("No harness record.");
  const promo = run.harness.promotions.find((p) => p.id === promotionId);
  if (!promo) throw new Error("Promotion not found.");
  promo.status = "promoted";
  promo.promoted_at = nowIso();
  const map = loadHarnessMap();
  map.promotions.unshift({ ...promo });
  map.notes.unshift(`[${promo.category}] ${promo.gap} — ${promo.patch}`);
  map.notes = map.notes.slice(0, 40);
  map.promotions = map.promotions.slice(0, 40);
  map.updated_at = nowIso();
  savePartial({ harnessMap: map });
  upsertRun(run);
  audit("harness.promote", "run", runId, "operator promoted a classified gap into the harness map");
  return run;
}

export function loadHarnessMap(): HarnessMap {
  return (
    load().harnessMap ?? {
      promotions: [],
      notes: [],
      updated_at: nowIso(),
    }
  );
}

export function listPromotions(): HarnessPromotion[] {
  return loadHarnessMap().promotions;
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
  let binary = whichCodex();
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
  try {
    binary = assertLocalCodex(binary);
  } catch (err) {
    run.status = "error";
    run.last_grade = "HOLD";
    run.last_gaps = err instanceof Error ? err.message : String(err);
    run.updated_at = nowIso();
    upsertRun(run);
    updateAgentStatus(agent.id, "blocked");
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
  const writes = yoloWritesEnabled(agent.workspace_path);
  let gaps: string | undefined;
  let sawFail = false;
  for (let i = 1; i <= run.max_iterations; i += 1) {
    if (cancels.has(runId)) break;
    run = getRun(runId).run;
    run.current_iteration = i;
    run.status = "running";
    run.updated_at = nowIso();
    upsertRun(run);
    audit("hillclimb.iteration", "run", runId, `iteration=${i} phase=worker`);
    const mapNotes = formatHarnessMapNotes(loadHarnessMap().notes);
    if (!run.harness) {
      run.harness = newHarnessRecord(run.goal, run.success_criteria, agent.workspace_path, writes);
    }
    run.harness.recovery_phase = liveRecoveryPhase(sawFail, "worker");
    upsertRun(run);
    const wprompt = workerPrompt({
      agentName: agent.name,
      brief: agent.brief,
      goal: run.goal,
      successCriteria: run.success_criteria,
      workspace: workdir,
      iteration: i,
      maxIterations: run.max_iterations,
      priorGaps: gaps,
      harnessMapNotes: mapNotes || undefined,
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
      run.harness.recovery_phase = liveRecoveryPhase(sawFail, "grader");
      upsertRun(run);
      const gprompt = graderPrompt({
        agentName: agent.name,
        brief: agent.brief,
        goal: run.goal,
        successCriteria: run.success_criteria,
        workspace: workdir,
        iteration: i,
        workerSummary: worker.text,
        il5Mode: agent.template === "il5-grader",
        harnessMapNotes: mapNotes || undefined,
      });
      const grader = await runCodex(binary, gprompt, workdir, "read-only", agent.grader_thread_id);
      if (grader.threadId) updateAgentStatus(agent.id, "running", { grader: grader.threadId });
      const approved =
        run.harness?.approval_status === "approved" || run.harness?.approval_status === "confirmed";
      const confirmed = run.harness?.approval_status === "confirmed";
      const parsed = enforceProductChecklist(
        workdir,
        enforceGrade(
          worker.text,
          grader.text,
          parseGrade(grader.text).grade,
          parseGrade(grader.text).gaps,
          approved,
          confirmed,
        ),
      );
      if (parsed.grade === "HOLD" || parsed.grade === "WARN") sawFail = true;
      const classified =
        parsed.grade === "HOLD" || parsed.grade === "WARN"
          ? classifyGap(parsed.gaps)
          : run.harness.classified_gap
            ? {
                category: run.harness.gap_category ?? "map",
                gap: run.harness.classified_gap,
                promote: "Keep the verified fix on the harness map.",
              }
            : undefined;
      if (classified) {
        gaps = `${parsed.gaps}\n\nCLASSIFIED GAP: [${classified.category}] ${classified.gap}\nPROMOTE CANDIDATE: ${classified.promote}\nRECOVERY: classify → patch this iteration (do not retry blindly)`;
      } else {
        gaps = parsed.gaps;
      }
      if (!run.harness) {
        run.harness = newHarnessRecord(run.goal, run.success_criteria, agent.workspace_path, writes);
      }
      run.harness.classified_gap = classified?.gap ?? run.harness.classified_gap;
      run.harness.gap_category = classified?.category ?? run.harness.gap_category;
      run.harness.recovery_phase = recoveryPhaseFor(
        parsed.grade,
        Boolean(classified),
        parsed.grade === "PASS" && sawFail,
      );
      run.harness.jobs = scoreJobs({
        goal: run.goal,
        criteria: run.success_criteria,
        workspace: workdir,
        briefPresent: Boolean(agent.brief.trim()),
        sandbox: writes ? "workspace-write" : "read-only",
        iteration: i,
        worker: worker.text,
        grader: grader.text,
        grade: parsed.grade,
        classifiedGap: run.harness.classified_gap,
        allowWrites: writes,
      });
      if (classified) offerPromotion(run.harness, classified);
      if (parsed.grade === "PASS" && classified && agent.template === "desk-improver") {
        const offered = [...run.harness.promotions].reverse().find((p) => p.status === "offered");
        if (offered) {
          offered.status = "auto-promoted";
          offered.promoted_at = nowIso();
          const map = loadHarnessMap();
          map.promotions.unshift({ ...offered });
          map.notes.unshift(`[${offered.category}] ${offered.gap} — ${offered.patch}`);
          map.updated_at = nowIso();
          savePartial({ harnessMap: map });
          audit("harness.promote", "run", runId, "auto-promoted after verify (Desk Improver)");
        }
      }
      addIteration({
        id: randomUUID(),
        run_id: runId,
        iteration: i,
        phase: "grader",
        worker_summary: grader.text,
        grade: parsed.grade,
        gaps,
        created_at: nowIso(),
      });
      run.last_grade = parsed.grade;
      run.last_gaps = gaps;
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
