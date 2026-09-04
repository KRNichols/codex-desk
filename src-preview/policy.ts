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
