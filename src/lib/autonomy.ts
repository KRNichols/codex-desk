export type Consequence = "read" | "write" | "send_merge_deploy" | "delete_pay_publish";

export const AUTONOMY_RUNGS = [
  {
    id: "read" as const,
    title: "Read / research",
    action: "Inspect and summarize.",
    control: "Automatic",
  },
  {
    id: "write" as const,
    title: "Write in workspace",
    action: "Edit isolated files; run tests.",
    control: "Automatic + checks",
  },
  {
    id: "send_merge_deploy" as const,
    title: "Send / merge / deploy",
    action: "Affect a person or shared system.",
    control: "Evidence + approval",
  },
  {
    id: "delete_pay_publish" as const,
    title: "Delete / pay / publish",
    action: "Irreversible external action.",
    control: "Explicit human confirmation",
  },
];

export function consequenceLabel(tier: Consequence): string {
  switch (tier) {
    case "write":
      return "Write in workspace — automatic + checks";
    case "send_merge_deploy":
      return "Send / merge / deploy — evidence + approval";
    case "delete_pay_publish":
      return "Delete / pay / publish — explicit human confirmation";
    default:
      return "Read / research — automatic";
  }
}

function negated(lower: string, needle: string): boolean {
  const idx = lower.indexOf(needle);
  if (idx < 0) return false;
  const window = lower.slice(Math.max(0, idx - 24), idx);
  return ["do not", "don't", "never", "without", "forbid", "not a"].some((p) => window.includes(p));
}

function hasIntent(lower: string, needles: string[]): boolean {
  return needles.some((n) => lower.includes(n) && !negated(lower, n));
}

export function classifyText(text: string): Consequence {
  const lower = text.toLowerCase();
  if (
    hasIntent(lower, [
      "npm publish",
      "cargo publish",
      "pypi publish",
      "wire payment",
      "send payment",
      "delete production",
      "drop table",
      "rm -rf",
      "git rm ",
    ])
  ) {
    return "delete_pay_publish";
  }
  if (
    hasIntent(lower, [
      "git push",
      "git merge",
      "merge pull request",
      "merge the pr",
      "deploy to",
      "kubectl apply",
      "helm upgrade",
      "ship to prod",
      "send to production",
    ])
  ) {
    return "send_merge_deploy";
  }
  if (
    hasIntent(lower, [
      "edit ",
      "write ",
      "patch ",
      "implement ",
      "fix ",
      "update ",
      "create file",
      "workspace-write",
      "hill-climb",
      "hill climb",
    ])
  ) {
    return "write";
  }
  return "read";
}

export function classifyGoal(goal: string, criteria: string): Consequence {
  const rank = (c: Consequence) =>
    c === "delete_pay_publish" ? 3 : c === "send_merge_deploy" ? 2 : c === "write" ? 1 : 0;
  const a = classifyText(goal);
  const b = classifyText(criteria);
  return rank(a) >= rank(b) ? a : b;
}

export function gate(
  tier: Consequence,
  approved: boolean,
  confirmed: boolean,
  evidence?: string,
): string | null {
  if (tier === "send_merge_deploy") {
    if (!approved) {
      return "Send / merge / deploy needs evidence plus explicit approval before Desk performs it.";
    }
    if ((evidence ?? "").trim().length < 8) {
      return "Send / merge / deploy approval requires evidence (what was verified, where).";
    }
  }
  if (tier === "delete_pay_publish" && !confirmed) {
    return "Delete / pay / publish needs explicit human confirmation before Desk performs it.";
  }
  return null;
}

export function workerViolatedGate(worker: string, approved: boolean, confirmed: boolean): string | null {
  const lower = worker.toLowerCase();
  if (
    hasIntent(lower, ["git push", "git merge", "deployed to", "kubectl apply", "merged pull request"]) &&
    !approved
  ) {
    return "HOLD: send/merge/deploy in the worker without evidence + approval. YOLO writes are not a send gate.";
  }
  if (hasIntent(lower, ["npm publish", "cargo publish", "wired payment", "deleted production"]) && !confirmed) {
    return "HOLD: delete/pay/publish in the worker without explicit human confirmation.";
  }
  return null;
}
