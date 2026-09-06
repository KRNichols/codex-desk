import { useEffect, useState } from "react";
import { AlertTriangle } from "lucide-react";
import { Badge, GradeBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { clearEnvVault, getSetupEnv, setEnvVault } from "@/lib/runtime";
import type { EnvVarRow, SetupEnvStatus } from "@/lib/types";

export function SetupEnvPanel({ onChange }: { onChange?: () => void }) {
  const [status, setStatus] = useState<SetupEnvStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);

  async function refresh() {
    setError(null);
    try {
      setStatus(await getSetupEnv());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function save(row: EnvVarRow) {
    const value = (drafts[row.key] ?? "").trim();
    if (!value) return;
    setBusyKey(row.key);
    setError(null);
    setNote(null);
    try {
      await setEnvVault(row.key, value);
      setDrafts((prev) => ({ ...prev, [row.key]: "" }));
      setNote(`${row.key} saved to the Desk vault. Exported only to child Codex — not an Azure SDK.`);
      await refresh();
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyKey(null);
    }
  }

  async function clear(row: EnvVarRow) {
    setBusyKey(row.key);
    setError(null);
    try {
      await clearEnvVault(row.key);
      setNote(`${row.key} cleared from the Desk vault.`);
      await refresh();
      onChange?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusyKey(null);
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-5 px-4 py-6">
      <section className="rounded-sm border border-border bg-card p-4">
        <div className="mb-2 flex items-center justify-between gap-2">
          <h2 className="text-lg font-semibold">Setup / Env</h2>
          <Badge variant={status?.config_toml_exists ? "pass" : "hold"}>
            {status?.config_toml_exists ? "config found" : "config missing"}
          </Badge>
        </div>
        <p className="text-sm text-foreground/85">
          Desk reads the shared Codex home (<code>CODEX_HOME</code>, else{" "}
          <code>%USERPROFILE%\.codex</code> / <code>~/.codex</code>). Secrets stay out of{" "}
          <code>config.toml</code>. Vault values export only to the child <code>codex</code> process.
          Desk is not a second Azure client.
        </p>
        {status ? (
          <ul className="mt-3 space-y-1 font-mono text-xs text-foreground/85">
            <li>Home source: {status.home_source}</li>
            <li className="truncate" title={status.codex_home}>
              Codex home: {status.codex_home}
            </li>
            <li className="truncate" title={status.config_path}>
              config.toml: {status.config_path}
            </li>
            <li>
              env_key in file: {status.env_keys_in_config.length ? status.env_keys_in_config.join(", ") : "none"}
            </li>
          </ul>
        ) : (
          <p className="mt-3 text-xs text-muted-foreground">Reading Codex config…</p>
        )}
        <Button size="sm" variant="outline" className="mt-3" onClick={() => void refresh()}>
          Recheck
        </Button>
      </section>

      {status && !status.config_toml_exists ? (
        <section className="rounded-sm border border-hold/40 bg-hold/10 px-4 py-3 text-sm">
          <div className="mb-1 flex items-center gap-2 font-medium text-hold">
            <AlertTriangle className="size-4" />
            No config.toml at the Codex home
          </div>
          <p>
            Write an HTTPS <code>base_url</code> and <code>env_key</code> there. Do not paste a PAT into the
            file. Then set the named variable below.
          </p>
        </section>
      ) : null}

      <section className="rounded-sm border border-border bg-card p-4">
        <h3 className="font-medium">Codex config fields</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          Deployment / model and base_url come from config.toml. Desk will not invent a deployment name.
        </p>
        <ul className="mt-3 space-y-2">
          {(status?.config_fields ?? []).map((field) => (
            <li key={field.key} className="rounded-sm border border-border bg-background px-3 py-2">
              <div className="flex items-center justify-between gap-2">
                <span className="font-mono text-xs">{field.key}</span>
                <GradeBadge grade={field.status} />
              </div>
              <p className="mt-1 text-xs text-foreground/80">{field.description}</p>
              {field.display_value ? (
                <p className="mt-1 truncate font-mono text-[11px] text-muted-foreground" title={field.display_value}>
                  {field.display_value}
                </p>
              ) : null}
            </li>
          ))}
        </ul>
      </section>

      <section className="rounded-sm border border-border bg-card p-4">
        <h3 className="font-medium">Environment keys</h3>
        <p className="mt-1 text-xs text-muted-foreground">
          Every <code>env_key</code> from config.toml plus the Azure template (
          <code>AZURE_OPENAI_API_KEY</code>, deployment/model, base_url / endpoint). FOUND means process env,
          gitignored env file, OS slot, or Desk vault. Values are never shown after save.
        </p>
        <ul className="mt-3 space-y-3">
          {(status?.vars ?? []).map((row) => (
            <li key={row.key} className="rounded-sm border border-border bg-background px-3 py-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <span className="font-mono text-xs">
                  {row.key}
                  {row.from_config ? " · config env_key" : ""}
                  {row.related_to ? ` · related ${row.related_to}` : ""}
                </span>
                <span className="flex items-center gap-2">
                  <Badge variant="outline">{row.source}</Badge>
                  <GradeBadge grade={row.status} />
                </span>
              </div>
              <p className="mt-1 text-xs text-foreground/80">{row.description}</p>
              <div className="mt-2 flex flex-wrap items-center gap-2">
                <Input
                  type="password"
                  autoComplete="off"
                  placeholder={row.status === "FOUND" ? "Update value (not shown)" : "Set value into Desk vault"}
                  value={drafts[row.key] ?? ""}
                  onChange={(e) => setDrafts((prev) => ({ ...prev, [row.key]: e.target.value }))}
                  className="max-w-sm"
                />
                <Button
                  size="sm"
                  disabled={busyKey === row.key || !(drafts[row.key] ?? "").trim()}
                  onClick={() => void save(row)}
                >
                  Save to vault
                </Button>
                {row.source === "desk-vault" || row.source === "os-slot" ? (
                  <Button size="sm" variant="outline" disabled={busyKey === row.key} onClick={() => void clear(row)}>
                    Clear vault
                  </Button>
                ) : null}
              </div>
            </li>
          ))}
        </ul>
      </section>

      {note ? <p className="text-xs text-foreground/80">{note}</p> : null}
      {error ? <p className="text-sm text-destructive">{error}</p> : null}
      {status ? <p className="text-xs text-muted-foreground">{status.note}</p> : null}
    </div>
  );
}
