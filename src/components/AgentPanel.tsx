import { useEffect, useState } from "react";
import { LoaderCircle } from "lucide-react";
import { Badge, GradeBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import {
  approveHillclimb,
  cancelHillclimb,
  confirmHillclimb,
  getHarnessMap,
  getRun,
  listAgentRuns,
  promoteHarness,
  startHillclimb,
  updateAgent,
} from "@/lib/runtime";
import { classifyGoal } from "@/lib/autonomy";
import { emptyHarness } from "@/lib/harness";
import type { HarnessMap } from "@/lib/types";
import { CLOSE_IL5_MISSING_CRITERIA, CLOSE_IL5_MISSING_GOAL } from "@/lib/prompts";
import type { Agent, HillclimbIteration, HillclimbRun, RuntimeStatus } from "@/lib/types";
import { IdentityPanel } from "@/components/IdentityPanel";
import { AutonomyLadder, FailureUpgradeLoop, SixJobGrid } from "@/components/HarnessBoard";

export function AgentPanel({
  agent,
  status,
  onChange,
}: {
  agent: Agent;
  status: RuntimeStatus | null;
  onChange: (agent: Agent) => void;
}) {
  const [workspace, setWorkspace] = useState(agent.workspace_path ?? "");
  const [brief, setBrief] = useState(agent.brief);
  const [goal, setGoal] = useState("");
  const [criteria, setCriteria] = useState("");
  const [maxIter, setMaxIter] = useState(3);
  const [runs, setRuns] = useState<HillclimbRun[]>([]);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const [iterations, setIterations] = useState<HillclimbIteration[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [evidence, setEvidence] = useState("");
  const [confirmDestructive, setConfirmDestructive] = useState(false);
  const [harnessMap, setHarnessMap] = useState<HarnessMap | null>(null);

  useEffect(() => {
    setWorkspace(agent.workspace_path ?? "");
    setBrief(agent.brief);
    void listAgentRuns(agent.id).then((next) => {
      setRuns(next);
      setActiveRunId(next[0]?.id ?? null);
    });
    void getHarnessMap()
      .then(setHarnessMap)
      .catch(() => undefined);
  }, [agent.id, agent.brief, agent.workspace_path]);

  useEffect(() => {
    if (!activeRunId) {
      setIterations([]);
      return;
    }
    let stop = false;
    const tick = async () => {
      try {
        const detail = await getRun(activeRunId);
        if (stop) return;
        setIterations(detail.iterations);
        setRuns((prev) => prev.map((r) => (r.id === detail.run.id ? detail.run : r)));
        if (detail.run.status === "running" || detail.run.status === "queued") {
          window.setTimeout(() => void tick(), 1500);
        }
      } catch {
        // keep last snapshot
      }
    };
    void tick();
    return () => {
      stop = true;
    };
  }, [activeRunId]);

  const activeRun = runs.find((r) => r.id === activeRunId) ?? null;
  const running = activeRun?.status === "running" || activeRun?.status === "queued";
  const harness = activeRun?.harness ?? emptyHarness();
  const previewTier = classifyGoal(goal, criteria);

  async function saveWorkspace() {
    const next = await updateAgent(agent.id, { brief, workspace_path: workspace });
    onChange(next);
  }

  async function start() {
    if (!goal.trim() || !criteria.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await saveWorkspace();
      const run = await startHillclimb({
        agentId: agent.id,
        goal: goal.trim(),
        successCriteria: criteria.trim(),
        maxIterations: maxIter,
        allowWrites: Boolean(workspace.trim()),
        approvalEvidence: evidence.trim() || undefined,
        confirmDestructive,
      });
      setRuns((prev) => [run, ...prev]);
      setActiveRunId(run.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ScrollArea className="flex-1">
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-5 px-4 py-6">
        {activeRun ? (
          <section className="rounded-sm border border-hold/40 bg-hold/10 p-4">
            <div className="flex items-center justify-between gap-2">
              <h3 className="font-medium">Live run</h3>
              {running ? (
                <Button size="sm" variant="outline" onClick={() => void cancelHillclimb(activeRun.id)}>
                  Cancel
                </Button>
              ) : null}
            </div>
            <p className="mt-1 flex flex-wrap items-center gap-2 font-mono text-sm">
              Iteration {activeRun.current_iteration}/{activeRun.max_iterations}
              <Badge
                variant={
                  activeRun.status === "error" || activeRun.status === "blocked" ? "hold" : "outline"
                }
              >
                {activeRun.status}
              </Badge>
              {activeRun.last_grade ? <GradeBadge grade={activeRun.last_grade} /> : <GradeBadge grade="HOLD" />}
            </p>
            {running ? (
              <p className="mt-2 flex items-center gap-2 font-mono text-xs text-muted-foreground">
                <LoaderCircle className="size-3.5 animate-spin" />
                Codex worker/grader running in the background. Operator chat stays usable.
              </p>
            ) : null}
            {activeRun.last_gaps ? (
              <pre className="grade-log mt-3 whitespace-pre-wrap text-foreground">{activeRun.last_gaps}</pre>
            ) : null}
            {activeRun.status === "awaiting_approval" ? (
              <div className="mt-3 space-y-2">
                <p className="text-xs text-foreground/85">
                  Send / merge / deploy is gated. YOLO workspace writes are not. Paste evidence, then approve.
                </p>
                <Textarea
                  value={evidence}
                  onChange={(e) => setEvidence(e.target.value)}
                  placeholder="Evidence: tests run, files changed, why this send is ready."
                  className="min-h-[64px] bg-background"
                />
                <Button
                  size="sm"
                  onClick={() =>
                    void approveHillclimb(activeRun.id, evidence.trim()).then((next) => {
                      setRuns((prev) => prev.map((r) => (r.id === next.id ? next : r)));
                    })
                  }
                  disabled={evidence.trim().length < 8}
                >
                  Approve send / merge / deploy
                </Button>
              </div>
            ) : null}
            {activeRun.status === "awaiting_confirm" ? (
              <div className="mt-3 space-y-2">
                <p className="text-xs text-foreground/85">
                  Delete / pay / publish needs explicit human confirmation. This is not an identity-attestation write HOLD.
                </p>
                <Button
                  size="sm"
                  onClick={() =>
                    void confirmHillclimb(activeRun.id).then((next) => {
                      setRuns((prev) => prev.map((r) => (r.id === next.id ? next : r)));
                    })
                  }
                >
                  I confirm this destructive action
                </Button>
              </div>
            ) : null}
          </section>
        ) : null}

        <SixJobGrid harness={harness} envKeyPresent={status?.env_key_present} />
        <AutonomyLadder
          current={activeRun?.harness?.autonomy_tier ?? previewTier}
          approvalStatus={harness.approval_status}
        />
        <FailureUpgradeLoop
          harness={harness}
          map={harnessMap}
          runId={activeRun?.id}
          onPromote={(runId, promoId) =>
            void promoteHarness(runId, promoId).then((next) => {
              setRuns((prev) => prev.map((r) => (r.id === next.id ? next : r)));
              void getHarnessMap().then(setHarnessMap);
            })
          }
        />

        <IdentityPanel status={status} compact />

        <section className="rounded-sm border border-border bg-card p-4">
          <div className="mb-2 flex items-center justify-between gap-2">
            <h2 className="text-lg font-semibold">{agent.name}</h2>
            <GradeBadge
              grade={
                agent.status === "blocked" || agent.status === "error"
                  ? "HOLD"
                  : agent.status === "done"
                    ? "PASS"
                    : agent.status === "running"
                      ? "WARN"
                      : null
              }
            />
          </div>
          <p className="text-xs text-muted-foreground">
            Template: {agent.template}. Desk injects <code>briefs/OPERATOR.md</code> plus this worker brief via{" "}
            <code>codex exec</code> — not VS Code system prompts. Treat briefs as potential CUI.
          </p>
          <label className="mt-3 block text-xs text-muted-foreground">Brief / contract</label>
          <Textarea value={brief} onChange={(e) => setBrief(e.target.value)} className="mt-1 min-h-[88px] bg-background" />
          <label className="mt-3 block text-xs text-muted-foreground">Workspace path (YOLO writes when set)</label>
          <Input
            value={workspace}
            onChange={(e) => setWorkspace(e.target.value)}
            placeholder={status?.suggested_workspace ?? "C:\\src\\codex-desk"}
            className="mt-1"
          />
          <p className="mt-1 text-xs text-muted-foreground">
            YOLO is always-on: a workspace path enables workspace-write. No attestation gate. Home directory is
            refused. Desk never auto-commits or pushes.
          </p>
          <Button size="sm" variant="outline" className="mt-2" onClick={() => void saveWorkspace()}>
            Save agent
          </Button>
        </section>

        <section className="rounded-sm border border-border bg-card p-4">
          <h3 className="font-medium">Start hill-climb</h3>
          <p className="mt-2 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            Grades:
            <GradeBadge grade="PASS" />
            <GradeBadge grade="HOLD" />
            <GradeBadge grade="WARN" />
            <span>HOLD on unvalidated claims. Never an ATO.</span>
          </p>
          <Button
            size="sm"
            variant="outline"
            className="mt-2"
            onClick={() => {
              setGoal(CLOSE_IL5_MISSING_GOAL);
              setCriteria(CLOSE_IL5_MISSING_CRITERIA);
            }}
          >
            Use IL5 product-owned gap template
          </Button>
          <label className="mt-3 block text-xs text-muted-foreground">Goal</label>
          <Textarea
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
            placeholder="Example: Clarify the README smoke path without claiming ATO."
            className="mt-1 min-h-[72px] bg-background"
          />
          <label className="mt-3 block text-xs text-muted-foreground">Success criteria</label>
          <Textarea
            value={criteria}
            onChange={(e) => setCriteria(e.target.value)}
            placeholder="Example: A newcomer can run npm run dev and send hello; SECURITY.md residual risks stay marked MISSING."
            className="mt-1 min-h-[72px] bg-background"
          />
          <div className="mt-3 flex flex-wrap items-center gap-3 text-sm">
            <label className="flex items-center gap-2">
              Max iterations
              <Input
                type="number"
                min={1}
                max={12}
                value={maxIter}
                onChange={(e) => setMaxIter(Number(e.target.value) || 1)}
                className="w-20"
              />
            </label>
            <p className="text-xs text-muted-foreground">
              YOLO writes when a workspace path is set. No Allow-workspace-writes checkbox.
              Preview tier: {previewTier}.
            </p>
          </div>
          {previewTier === "send_merge_deploy" ? (
            <div className="mt-3 space-y-2">
              <label className="block text-xs text-muted-foreground">Evidence for send / merge / deploy</label>
              <Textarea
                value={evidence}
                onChange={(e) => setEvidence(e.target.value)}
                className="min-h-[56px] bg-background"
                placeholder="What was verified, and where."
              />
            </div>
          ) : null}
          {previewTier === "delete_pay_publish" ? (
            <label className="mt-3 flex items-center gap-2 text-xs">
              <input
                type="checkbox"
                checked={confirmDestructive}
                onChange={(e) => setConfirmDestructive(e.target.checked)}
              />
              I confirm this delete / pay / publish action (not an identity write HOLD).
            </label>
          ) : null}
          {error ? <p className="mt-2 text-sm text-destructive">{error}</p> : null}
          <Button className="mt-3" disabled={busy || !goal.trim() || !criteria.trim()} onClick={() => void start()}>
            {busy ? "Starting…" : "Start hill-climb"}
          </Button>
        </section>

        {iterations.length > 0 ? (
          <ol className="space-y-2 text-xs">
            {iterations.map((item) => (
              <li key={item.id} className="rounded-sm border border-border bg-card p-2">
                <p className="flex items-center gap-2 font-medium">
                  #{item.iteration} {item.phase}
                  {item.grade ? <GradeBadge grade={item.grade} /> : null}
                </p>
                <pre className="grade-log mt-1 max-h-40 overflow-auto whitespace-pre-wrap text-foreground/80">
                  {(item.gaps || item.worker_summary || "").slice(0, 1200)}
                </pre>
              </li>
            ))}
          </ol>
        ) : null}

        <section>
          <h3 className="mb-2 font-medium">Recent runs</h3>
          {runs.length === 0 ? (
            <p className="text-sm text-muted-foreground">No hill-climb jobs yet.</p>
          ) : (
            <ul className="space-y-1 text-sm">
              {runs.map((run) => (
                <li key={run.id}>
                  <button
                    className="flex w-full items-center justify-between gap-2 text-left hover:text-primary"
                    onClick={() => setActiveRunId(run.id)}
                  >
                    <span className="min-w-0 truncate">{run.goal.slice(0, 72)}</span>
                    <span className="flex items-center gap-1">
                      <Badge
                        variant={
                          run.status === "error" || run.status === "blocked" ? "hold" : "outline"
                        }
                      >
                        {run.status}
                      </Badge>
                      {run.last_grade ? <GradeBadge grade={run.last_grade} /> : null}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </ScrollArea>
  );
}
