import { Badge, GradeBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { AUTONOMY_RUNGS, type Consequence } from "@/lib/autonomy";
import { JOB_CARDS, PROMOTE_KINDS, RECOVERY_STEPS, emptyHarness } from "@/lib/harness";
import type { HarnessMap, HarnessRecord } from "@/lib/types";
import { cn } from "@/lib/utils";

function asTier(value?: string | null): Consequence {
  if (value === "write" || value === "send_merge_deploy" || value === "delete_pay_publish") return value;
  return "read";
}

function stepIndex(phase: string): number {
  const i = RECOVERY_STEPS.findIndex((s) => s.id === phase);
  return i < 0 ? 0 : i;
}

export function SixJobGrid({
  harness,
  envKeyPresent,
}: {
  harness: HarnessRecord;
  envKeyPresent?: boolean;
}) {
  const jobs = harness.jobs.length ? harness.jobs : emptyHarness().jobs;
  return (
    <section className="rounded-sm border border-border bg-card p-4">
      <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
        The minimum viable harness
      </p>
      <h3 className="mt-1 text-lg font-semibold">Six jobs that turn model capability into controlled execution</h3>
      <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {JOB_CARDS.map((card, idx) => {
          const job = jobs.find((j) => j.name === card.name) ?? jobs[idx];
          return (
            <article key={card.name} className="flex flex-col rounded-sm border border-border bg-background p-3">
              <div className="flex items-start justify-between gap-2">
                <span
                  className={cn(
                    "inline-flex size-8 shrink-0 items-center justify-center rounded-full font-mono text-[11px] font-semibold",
                    idx % 2 === 0 ? "bg-primary text-primary-foreground" : "bg-grade-warn text-background",
                  )}
                >
                  {card.num}
                </span>
                <GradeBadge grade={job?.status} />
              </div>
              <h4 className="mt-3 font-mono text-sm font-semibold uppercase tracking-[0.12em]">{card.title}</h4>
              <div className="console-rule my-2 w-10" />
              <p className="text-sm text-foreground/85">{card.purpose}</p>
              {card.name === "tools" ? (
                <p className="mt-2 font-mono text-[11px] text-muted-foreground">
                  Setup / Env {envKeyPresent ? "lists env_key names" : "reads config.toml env_key names"}. Do not
                  invent that a secret is set.
                </p>
              ) : null}
              <p className="mt-2 text-xs text-foreground/75">{job?.summary || "Not scored yet."}</p>
            </article>
          );
        })}
      </div>
      <p className="mt-4 text-xs text-muted-foreground">
        A prompt steers one inference. A harness governs the whole run. Grader scores all six on the run record.
      </p>
      <p className="mt-1 font-mono text-[11px] text-muted-foreground">
        Sandbox {harness.sandbox} · {harness.allowlist} · recovery {harness.recovery_phase}
      </p>
    </section>
  );
}

export function AutonomyLadder({
  current,
  approvalStatus,
}: {
  current: Consequence | string;
  approvalStatus?: string;
}) {
  const tier = asTier(typeof current === "string" ? current : current);
  return (
    <section className="rounded-sm border border-border bg-card p-4">
      <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
        Autonomy is earned by evidence
      </p>
      <h3 className="mt-1 text-lg font-semibold">Increase control only when the consequence increases</h3>
      <ol className="mt-4 space-y-2">
        {AUTONOMY_RUNGS.map((rung, idx) => {
          const active = rung.id === tier;
          const gated = idx > 1;
          return (
            <li
              key={rung.id}
              className={cn(
                "flex flex-wrap items-center justify-between gap-3 rounded-sm border px-3 py-2",
                active
                  ? gated
                    ? "border-hold/50 bg-hold/10"
                    : "border-pass/50 bg-secondary"
                  : "border-border bg-background",
              )}
            >
              <div className="min-w-0">
                <p className="font-mono text-xs font-semibold uppercase tracking-[0.1em]">{rung.title}</p>
                <p className="text-sm text-foreground/80">{rung.action}</p>
              </div>
              <Badge variant={gated ? (active ? "hold" : "outline") : active ? "pass" : "idle"}>
                {rung.control}
              </Badge>
            </li>
          );
        })}
      </ol>
      {approvalStatus && approvalStatus !== "none" ? (
        <p className="mt-3 font-mono text-[11px] text-foreground/80">Gate: {approvalStatus}</p>
      ) : null}
      <p className="mt-3 text-xs text-muted-foreground">
        Freedom inside boundaries, not freedom from boundaries. YOLO workspace writes stay always-on — no write-permission
        chrome. Send / merge / deploy still needs evidence + approval. Delete / pay / publish still needs explicit confirm.
      </p>
    </section>
  );
}

export function FailureUpgradeLoop({
  harness,
  map,
  runId,
  onPromote,
}: {
  harness: HarnessRecord;
  map: HarnessMap | null;
  runId?: string | null;
  onPromote?: (runId: string, promoId: string) => void;
}) {
  const current = stepIndex(harness.recovery_phase);
  const offered = harness.promotions.filter((p) => p.status === "offered");
  return (
    <section className="rounded-sm border border-border bg-card p-4">
      <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
        Failure should upgrade the harness
      </p>
      <h3 className="mt-1 text-lg font-semibold">Fix the current run, then fix the class of failure</h3>
      <ol className="mt-4 grid gap-2 sm:grid-cols-3 lg:grid-cols-6">
        {RECOVERY_STEPS.map((step, idx) => {
          const reached = idx <= current;
          const active = idx === current;
          return (
            <li
              key={step.id}
              className={cn(
                "rounded-sm border px-2 py-2 text-center",
                active ? "border-primary bg-hold/10" : reached ? "border-pass/40 bg-secondary" : "border-border bg-background",
              )}
            >
              <p className="font-mono text-[10px] font-semibold uppercase tracking-[0.14em]">{step.label}</p>
              <p className="mt-1 text-[11px] text-muted-foreground">{step.blurb}</p>
            </li>
          );
        })}
      </ol>
      <p className="mt-3 font-mono text-[11px] text-hold">
        Fail: return the exact gap to Classify. Do not retry blindly.
      </p>
      {harness.classified_gap ? (
        <p className="mt-2 font-mono text-xs">
          Classify [{harness.gap_category ?? "map"}]: {harness.classified_gap}
        </p>
      ) : (
        <p className="mt-2 text-xs text-muted-foreground">No classified gap on this run yet.</p>
      )}
      <div className="mt-4 rounded-sm border border-border bg-secondary p-3">
        <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.14em]">Promote the fix into the harness</p>
        <ul className="mt-2 grid gap-2 sm:grid-cols-2">
          {PROMOTE_KINDS.map((kind) => (
            <li
              key={kind.id}
              className={cn(
                "rounded-sm border px-2 py-1.5 font-mono text-[11px] uppercase tracking-[0.08em]",
                harness.gap_category === kind.id ? "border-primary text-foreground" : "border-border text-muted-foreground",
              )}
            >
              {kind.label}
            </li>
          ))}
        </ul>
        <p className="mt-2 text-xs text-muted-foreground">
          The patch fixes one run. The harness change improves every run after it.
        </p>
      </div>
      {harness.promotions.length ? (
        <ul className="mt-3 space-y-2">
          {harness.promotions.map((promo) => (
            <li key={promo.id} className="rounded-sm border border-border bg-background px-3 py-2 text-xs">
              <div className="flex items-center justify-between gap-2">
                <span className="font-mono">
                  {promo.category} · {promo.status}
                </span>
                {promo.status === "offered" && runId && onPromote ? (
                  <Button size="sm" variant="outline" onClick={() => onPromote(runId, promo.id)}>
                    Promote into harness
                  </Button>
                ) : (
                  <GradeBadge grade="PASS" />
                )}
              </div>
              <p className="mt-1">{promo.gap}</p>
              <p className="mt-1 text-muted-foreground">{promo.patch}</p>
            </li>
          ))}
        </ul>
      ) : offered.length === 0 ? (
        <p className="mt-3 text-xs text-muted-foreground">
          On HOLD/WARN Desk classifies the gap and offers map / tool / policy / test. Desk Improver auto-promotes after
          verify.
        </p>
      ) : null}
      {map?.notes?.length ? (
        <div className="mt-4">
          <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.12em]">Harness map (promoted)</p>
          <p className="mt-1 text-xs text-muted-foreground">Injected into every later worker/grader prompt.</p>
          <ul className="mt-2 list-disc space-y-1 pl-5 text-xs">
            {map.notes.slice(0, 8).map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
