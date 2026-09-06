import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { DATA_DIR } from "./secure-store";
import { envVaultExport, envVaultHas, envVaultKeys, getPatSlot, patSlotPresent } from "./crypto";
import type { ConfigFieldRow, EnvVarRow, SetupEnvStatus } from "../src/lib/types";

const ENV_LOCAL = path.resolve(process.cwd(), ".env.local");

function parseEnvFile(filePath: string): Record<string, string> {
  if (!existsSync(filePath)) return {};
  const out: Record<string, string> = {};
  for (const raw of readFileSync(filePath, "utf8").split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#") || !line.includes("=")) continue;
    const eq = line.indexOf("=");
    const key = line.slice(0, eq).trim();
    let value = line.slice(eq + 1).trim();
    if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
      value = value.slice(1, -1);
    }
    if (key) out[key] = value;
  }
  return out;
}

function describe(key: string): string {
  if (key === "AZURE_OPENAI_API_KEY") {
    return "API key / PAT the Codex Azure provider reads. Point config.toml env_key here, or let Desk export AZURE_LLM_PAT as this name to the child Codex process only.";
  }
  if (key === "AZURE_LLM_PAT") {
    return "Desk-preferred name for the Azure PAT. Never put this in config.toml. Desk exports it to child Codex as AZURE_OPENAI_API_KEY when that var is unset.";
  }
  if (key === "AZURE_LLM_ENDPOINT") {
    return "Optional HTTPS Azure base URL when config.toml has no base_url. Desk does not open Azure sockets.";
  }
  if (key === "AZURE_OPENAI_ENDPOINT") {
    return "Standard Azure OpenAI endpoint name. Related documentation alias — Desk is not an Azure SDK client.";
  }
  if (key === "AZURE_OPENAI_DEPLOYMENT") {
    return "Azure deployment / model name some tools expect. Prefer config.toml model= for Codex.";
  }
  if (key === "OPENAI_API_KEY") {
    return "Generic Codex env_key some configs use. If config.toml points here, set this in the Desk vault or the process environment.";
  }
  return "Environment variable referenced by Codex config or the Azure template. Desk exports vault values only to the child Codex process.";
}

function collectEnvKeys(text: string): string[] {
  const keys = new Set<string>();
  for (const match of text.matchAll(/env_key\s*=\s*"([^"]+)"/g)) {
    if (match[1]) keys.add(match[1]);
  }
  return [...keys];
}

function lookup(key: string): { found: boolean; source: string } {
  const file = parseEnvFile(ENV_LOCAL);
  if (file[key]) return { found: true, source: "env-file" };
  if (process.env[key]) return { found: true, source: "process" };
  if (envVaultHas(DATA_DIR, key)) return { found: true, source: "desk-vault" };
  if (key === "AZURE_LLM_PAT" && patSlotPresent(DATA_DIR)) return { found: true, source: "os-slot" };
  return { found: false, source: "missing" };
}

export function setupEnvStatus(): SetupEnvStatus {
  const home =
    process.env.CODEX_HOME || path.join(process.env.USERPROFILE || process.env.HOME || "", ".codex");
  const configPath = path.join(home, "config.toml");
  const exists = existsSync(configPath);
  const raw = exists ? readFileSync(configPath, "utf8") : "";
  const model = raw.match(/^\s*model\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  const provider = raw.match(/^\s*model_provider\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  const baseUrl = raw.match(/^\s*base_url\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  const configKeys = collectEnvKeys(raw);
  const wanted = new Set([
    ...configKeys,
    "AZURE_OPENAI_API_KEY",
    "AZURE_LLM_PAT",
    "AZURE_LLM_ENDPOINT",
    "AZURE_OPENAI_ENDPOINT",
    "AZURE_OPENAI_DEPLOYMENT",
    ...envVaultKeys(DATA_DIR),
  ]);
  const vars: EnvVarRow[] = [...wanted].sort().map((key) => {
    const { found, source } = lookup(key);
    return {
      key,
      kind: "env",
      description: describe(key),
      status: found ? "FOUND" : "MISSING",
      source,
      required: configKeys.includes(key) || key === "AZURE_OPENAI_API_KEY" || key === "AZURE_LLM_PAT",
      from_config: configKeys.includes(key),
      related_to: key === "AZURE_LLM_ENDPOINT" || key === "AZURE_OPENAI_ENDPOINT" ? "base_url" : null,
      display_value: null,
      settable: true,
    };
  });
  const field = (key: string, description: string, value: string | null): ConfigFieldRow => ({
    key,
    description,
    status: value ? "FOUND" : "MISSING",
    display_value: value,
  });
  const homeSource = process.env.CODEX_HOME
    ? "CODEX_HOME"
    : process.env.USERPROFILE
      ? "USERPROFILE"
      : process.env.HOME
        ? "HOME"
        : "fallback";
  return {
    codex_home: home,
    config_path: configPath,
    config_toml_exists: exists,
    home_source: homeSource,
    model,
    model_provider: provider,
    base_url: baseUrl,
    env_keys_in_config: configKeys,
    vars,
    config_fields: [
      field("model", "Azure deployment / model name in config.toml. Codex Desk will not invent one.", model),
      field("base_url", "HTTPS Azure resource endpoint in config.toml. No PAT in the URL.", baseUrl),
      field("model_provider", "Selected Codex provider (azure). Desk does not invent a second client.", provider),
      field(
        "env_key",
        "Name of the environment variable that holds the PAT. The value stays out of config.toml.",
        configKeys.length ? configKeys.join(", ") : null,
      ),
    ],
    note: "Desk reads Codex config.toml only. Vault values export to the child codex process — Desk is not an Azure SDK.",
  };
}

export function childEnv(): NodeJS.ProcessEnv {
  const file = parseEnvFile(ENV_LOCAL);
  const env = { ...process.env, ...file };
  for (const [k, v] of Object.entries(envVaultExport(DATA_DIR))) {
    if (!env[k]) env[k] = v;
  }
  const slot = getPatSlot(DATA_DIR);
  if (!env.AZURE_LLM_PAT && slot) env.AZURE_LLM_PAT = slot;
  if (!env.AZURE_OPENAI_API_KEY && env.AZURE_LLM_PAT) env.AZURE_OPENAI_API_KEY = env.AZURE_LLM_PAT;
  return env;
}
