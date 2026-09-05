import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const ATO_CLAIMS = [
  "we are authorized",
  "has an ato",
  "received an ato",
  "ato granted",
  "ato complete",
  "fedramp authorized",
  "fedramp authorization complete",
  "disa pa",
  "provisional authorization granted",
  "this product is authorized",
  "authorized to operate",
];

const WEAKEN = [
  "remove encryption",
  "disable encryption",
  "plaintext sqlite",
  "store the pat in sqlite",
  "pat in sqlite",
  "drop audit",
  "delete audit",
  "remove audit",
  "skip hash chain",
  "allow http://",
  "cleartext endpoint",
  "phone home",
  "phone-home",
  "add telemetry",
];

export function claimsAuthorization(text: string): boolean {
  const lower = text.toLowerCase();
  return ATO_CLAIMS.some((p) => lower.includes(p));
}

export function weakensProductControls(text: string): boolean {
  const lower = text.toLowerCase();
  return WEAKEN.some((p) => lower.includes(p));
}

export type Il5Row = {
  owner: "product" | "ao";
  id: string;
  grade: string;
  evidence: string;
};

export function parseIl5Rows(text: string): Il5Row[] {
  const start = text.indexOf("```il5-rows");
  if (start < 0) return [];
  const bodyStart = text.indexOf("\n", start);
  const end = text.indexOf("```", bodyStart + 1);
  if (bodyStart < 0 || end < 0) return [];
  const body = text.slice(bodyStart + 1, end);
  const rows: Il5Row[] = [];
  for (const line of body.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const parts = trimmed.split("|");
    if (parts.length < 4) continue;
    const owner = parts[0].trim();
    if (owner !== "product" && owner !== "ao") continue;
    rows.push({
      owner,
      id: parts[1].trim(),
      grade: parts[2].trim().toUpperCase(),
      evidence: parts.slice(3).join("|").trim(),
    });
  }
  return rows;
}

export function productRowsNotPass(rows: Il5Row[]): Il5Row[] {
  return rows.filter((r) => r.owner === "product" && r.grade !== "PASS");
}

export function enforceProductChecklist(
  workspace: string,
  parsed: { grade: string; gaps: string },
): { grade: string; gaps: string } {
  const file = path.join(workspace, "docs", "il5", "PRODUCT-CHECKLIST.md");
  if (!existsSync(file)) {
    return parsed;
  }
  const rows = parseIl5Rows(readFileSync(file, "utf8"));
  const bad = productRowsNotPass(rows);
  if (!bad.length) return parsed;
  const extra = `HOLD: product checklist rows not PASS: ${bad.map((r) => `${r.id}=${r.grade}`).join(", ")}`;
  return { grade: "HOLD", gaps: [extra, parsed.gaps].filter(Boolean).join("\n") };
}

export function enforceGrade(
  worker: string,
  graderText: string,
  grade: string,
  gaps: string,
): { grade: string; gaps: string } {
  const holds: string[] = [];
  if (claimsAuthorization(worker) || claimsAuthorization(graderText)) {
    holds.push("HOLD: text claims ATO / FedRAMP authorization / DISA PA. Desk never authorizes.");
  }
  if (weakensProductControls(worker)) {
    holds.push(
      "HOLD: worker weakens encryption, audit, secret non-storage, TLS, or no-phone-home rules.",
    );
  }
  if (!holds.length) return { grade, gaps };
  return { grade: "HOLD", gaps: [holds.join("\n"), gaps].filter(Boolean).join("\n") };
}

export function isCleartextUrl(url: string): boolean {
  const t = url.trim().toLowerCase();
  return t.startsWith("http://") || t.startsWith("ws://");
}

export function urlHasQuerySecret(url: string): boolean {
  const lower = url.toLowerCase();
  return (
    lower.includes("token=") ||
    lower.includes("sig=") ||
    lower.includes("access_token=") ||
    lower.includes("api_key=") ||
    lower.includes("pat=") ||
    url.includes("@")
  );
}

export function assertLocalCodex(binary: string): string {
  if (/^[a-z]+:\/\//i.test(binary) || binary.startsWith("\\\\")) {
    throw new Error("Refusing a remote Codex path. Desk may spawn only a local `codex` binary.");
  }
  const base = binary.split(/[/\\]/).pop()?.toLowerCase() ?? "";
  if (!["codex", "codex.exe", "codex.cmd"].includes(base)) {
    throw new Error(`Refusing binary \`${base}\`. Allowlist is local Codex only.`);
  }
  return binary;
}
