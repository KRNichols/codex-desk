import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { exportAudit, getIdentity } from "@/lib/runtime";
import type { IdentityStatus, RuntimeStatus } from "@/lib/types";

export function IdentityPanel({
  status,
  compact = false,
}: {
  status: RuntimeStatus | null;
  onChange?: () => void;
  compact?: boolean;
}) {
  const [identity, setIdentity] = useState<IdentityStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [exportNote, setExportNote] = useState<string | null>(null);

  useEffect(() => {
    void getIdentity()
      .then(setIdentity)
      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
  }, [status?.store_encrypted, status?.audit_chain_ok]);

  async function downloadAudit() {
    setBusy(true);
    setError(null);
    setExportNote(null);
    try {
      const events = await exportAudit();
      const blob = new Blob([JSON.stringify(events, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "codex-desk-audit.json";
      a.click();
      URL.revokeObjectURL(url);
      setExportNote(`Exported ${events.length} hash-chained events. No PAT values.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="rounded-sm border border-border bg-card p-4 text-sm">
      <div className="mb-2 flex items-center justify-between gap-2">
        <h3 className="font-medium">Identity / audit</h3>
        <Badge variant="pass">YOLO writes</Badge>
      </div>
      <p className="text-xs text-foreground/85">
        Machine-bound unlock. CAC/PIV is not shipped. This is a{" "}
        {status?.hello_bind ?? identity?.hello_bind ?? "user-session"} bind, not an ATO. Desk has no
        in-app write permissions; YOLO is always-on when a workspace path is set.
      </p>
      {compact ? (
        <p className="mt-3 text-xs text-foreground/85">
          Writes are allowed without attestation. Export audit stays on this card.
        </p>
      ) : (
        <ul className="mt-3 space-y-1 font-mono text-xs text-foreground/85">
          <li>Session: {identity?.session_user ?? status?.session_user ?? "unknown"}</li>
          <li>
            Store:{" "}
            {status?.store_encrypted || identity?.store_encrypted
              ? `encrypted (${status?.key_backend ?? identity?.key_backend ?? "os-backed"})`
              : "unlocking / not yet sealed"}
          </li>
          <li>Audit chain: {status?.audit_chain_ok || identity?.audit_chain_ok ? "intact" : "empty or verifying"}</li>
          <li>PAT slot: {status?.pat_slot ?? identity?.pat_slot ?? "unset"} (never SQLite)</li>
          <li>Egress allowlist: {status?.runner_allowlist ?? "local-codex-only"}</li>
        </ul>
      )}
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Button size="sm" variant="outline" disabled={busy} onClick={() => void downloadAudit()}>
          Export audit
        </Button>
        {exportNote ? <p className="text-xs text-foreground/80">{exportNote}</p> : null}
        {error ? <p className="text-xs text-destructive">{error}</p> : null}
      </div>
    </section>
  );
}
