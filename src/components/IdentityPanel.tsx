import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { getIdentity, setOperatorAttestation } from "@/lib/runtime";
import type { IdentityStatus, RuntimeStatus } from "@/lib/types";

export function IdentityPanel({
  status,
  onChange,
}: {
  status: RuntimeStatus | null;
  onChange?: () => void;
}) {
  const [identity, setIdentity] = useState<IdentityStatus | null>(null);
  const [name, setName] = useState("");
  const [org, setOrg] = useState("");
  const [statement, setStatement] = useState(
    "I am the assigned operator for this workstation and accept CUI handling for this desk.",
  );
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void getIdentity()
      .then(setIdentity)
      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
  }, [status?.operator_attested, status?.store_encrypted]);

  async function save() {
    setBusy(true);
    setError(null);
    try {
      await setOperatorAttestation({
        operator_name: name,
        organization: org,
        statement,
      });
      setIdentity(await getIdentity());
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="rounded-sm border border-border bg-card p-4 text-sm">
      <div className="mb-2 flex items-center justify-between gap-2">
        <h3 className="font-medium">IL5 identity gate</h3>
        <Badge variant={identity?.operator_attestation.configured ? "pass" : "hold"}>
          {identity?.operator_attestation.configured ? "attested" : "HOLD writes"}
        </Badge>
      </div>
      <p className="text-xs text-muted-foreground">
        Machine-bound unlock plus operator attestation. CAC/PIV is not shipped. This is a{" "}
        {status?.hello_bind ?? identity?.hello_bind ?? "user-session"} bind, not an ATO.
      </p>
      <ul className="mt-3 space-y-1 text-xs text-muted-foreground">
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
      {identity?.operator_attestation.configured ? (
        <p className="mt-3 text-xs">
          Attested as {identity.operator_attestation.operator_name} /{" "}
          {identity.operator_attestation.organization}. Workspace-write hill-climbs are allowed;
          Desk still will not auto-push.
        </p>
      ) : (
        <div className="mt-3 space-y-2">
          <p className="text-xs text-hold">
            Workspace-write hill-climbs stay HOLD until you record an operator attestation for this
            machine-bound session.
          </p>
          <Input placeholder="Operator name" value={name} onChange={(e) => setName(e.target.value)} />
          <Input placeholder="Organization" value={org} onChange={(e) => setOrg(e.target.value)} />
          <Textarea
            className="min-h-[64px] bg-background"
            value={statement}
            onChange={(e) => setStatement(e.target.value)}
          />
          {error ? <p className="text-xs text-destructive">{error}</p> : null}
          <Button size="sm" disabled={busy || !name.trim() || !org.trim()} onClick={() => void save()}>
            Record attestation
          </Button>
        </div>
      )}
    </section>
  );
}
