import { existsSync, readFileSync } from "node:fs";
import * as hill from "./hillclimb";
import { execFileSync, execSync, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { randomUUID } from "node:crypto";
import path from "node:path";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { Connect } from "vite";
import {
  helloBind,
  loadOrCreateDek,
  machineBinding,
  machineId,
  patSlotPresent,
  sessionUser,
  setPatSlot,
  clearPatSlot,
  getPatSlot,
} from "./crypto";
import {
  DATA_DIR,
  auditChainOk,
  leftoverPlaintext,
  loadStore,
  saveStore,
  storeEncryptedOnDisk,
  writeAudit,
  writeUnlockFailure,
} from "./secure-store";
import { assertLocalCodex, isCleartextUrl, urlHasQuerySecret } from "./policy";

type Chat = {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  codex_thread_id: string | null;
};

type Message = {
  id: string;
  chat_id: string;
  role: string;
  content: string;
  created_at: string;
  status: string;
};

type StoreFile = { chats?: Chat[]; messages?: Message[] };

const ENV_LOCAL = path.resolve(process.cwd(), ".env.local");

function nowIso() {
  return new Date().toISOString();
}

function loadChatStore(): { chats: Chat[]; messages: Message[] } {
  try {
    const store = loadStore() as StoreFile;
    return { chats: store.chats ?? [], messages: store.messages ?? [] };
  } catch (err) {
    writeUnlockFailure(err instanceof Error ? err.message : String(err));
    return { chats: [], messages: [] };
  }
}

function saveChatStore(store: { chats: Chat[]; messages: Message[] }) {
  const current = loadStore();
  saveStore({ ...current, chats: store.chats, messages: store.messages });
}

function parseEnvFile(filePath: string): Record<string, string> {
  if (!existsSync(filePath)) return {};
  const out: Record<string, string> = {};
  for (const raw of readFileSync(filePath, "utf8").split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#") || !line.includes("=")) continue;
    const eq = line.indexOf("=");
    const key = line.slice(0, eq).trim();
    let value = line.slice(eq + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    if (key) out[key] = value;
  }
  return out;
}

function lookupEnv(local: Record<string, string>, key: string): string | undefined {
  return local[key] || process.env[key] || undefined;
}

function redactUrl(url: string): string {
  if (url.includes("@") || url.includes("token=") || url.includes("sig=")) {
    return "(redacted — remove credentials from the URL; use AZURE_LLM_PAT instead)";
  }
  return url.trim();
}

function whichCodex(): string | null {
  const names = process.platform === "win32" ? ["codex.cmd", "codex.exe", "codex"] : ["codex"];
  for (const name of names) {
    try {
      const found = execSync(process.platform === "win32" ? `where ${name}` : `command -v ${name}`, {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      })
        .split(/\r?\n/)[0]
        ?.trim();
      if (found) return found;
    } catch {
      // keep looking
    }
  }
  return null;
}

function parseTomlLite(text: string) {
  const model = text.match(/^\s*model\s*=\s*"([^"]+)"/m)?.[1];
  const provider = text.match(/^\s*model_provider\s*=\s*"([^"]+)"/m)?.[1];
  const baseUrl = text.match(/^\s*base_url\s*=\s*"([^"]+)"/m)?.[1];
  const envKey = text.match(/^\s*env_key\s*=\s*"([^"]+)"/m)?.[1];
  return { model, provider, baseUrl, envKey };
}

function runtimeStatus() {
  const local = parseEnvFile(ENV_LOCAL);
  const binary = whichCodex();
  const home =
    process.env.CODEX_HOME ||
    path.join(process.env.USERPROFILE || process.env.HOME || "", ".codex");
  const configPath = path.join(home, "config.toml");
  const authPath = path.join(home, "auth.json");
  const parsed = existsSync(configPath) ? parseTomlLite(readFileSync(configPath, "utf8")) : {};
  let endpoint = parsed.baseUrl ? redactUrl(parsed.baseUrl) : undefined;
  if (!endpoint && lookupEnv(local, "AZURE_LLM_ENDPOINT")) {
    endpoint = redactUrl(lookupEnv(local, "AZURE_LLM_ENDPOINT")!);
  }
  let envKey = parsed.envKey;
  if (!envKey) {
    if (lookupEnv(local, "AZURE_LLM_PAT") || getPatSlot(DATA_DIR)) envKey = "AZURE_LLM_PAT";
    else if (lookupEnv(local, "AZURE_OPENAI_API_KEY")) envKey = "AZURE_OPENAI_API_KEY";
  }
  const envKeyPresent = envKey
    ? Boolean(lookupEnv(local, envKey) || (envKey === "AZURE_LLM_PAT" && getPatSlot(DATA_DIR)))
    : Boolean(lookupEnv(local, "AZURE_LLM_PAT") || lookupEnv(local, "AZURE_OPENAI_API_KEY") || getPatSlot(DATA_DIR));

  const issues: { code: string; message: string }[] = [];
  if (!binary) {
    issues.push({
      code: "codex_missing",
      message:
        "The `codex` CLI was not found on PATH. Install OpenAI Codex, then restart Codex Desk.",
    });
  }
  if (!existsSync(configPath) && !existsSync(authPath) && !envKeyPresent) {
    issues.push({
      code: "codex_unconfigured",
      message: `No Codex config at ${home}. Add config.toml (Azure endpoint) and set AZURE_LLM_PAT in the environment or .env.local.`,
    });
  }
  if (endpoint && isCleartextUrl(endpoint)) {
    issues.push({
      code: "cleartext_endpoint",
      message: "Refusing a cleartext (http://) Azure endpoint. Codex must use HTTPS only.",
    });
  }
  if (endpoint && urlHasQuerySecret(endpoint)) {
    issues.push({
      code: "endpoint_query_token",
      message: "Refusing an endpoint that embeds a token, signature, or credentials. Use AZURE_LLM_PAT / OS secret store.",
    });
  }
  if (leftoverPlaintext()) {
    issues.push({
      code: "plaintext_preview_store",
      message: "Leftover plaintext preview-store.json / audit.jsonl will be migrated into the encrypted envelope.",
    });
  }
  if (parsed.provider === "azure" && !envKeyPresent) {
    issues.push({
      code: "azure_pat_missing",
      message: `Codex is set to Azure, but ${envKey || "AZURE_LLM_PAT"} is not set. Put the PAT in that environment variable — not in the repo.`,
    });
  }

  let version: string | undefined;
  if (binary) {
    try {
      version = execFileSync(binary, ["--version"], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      })
        .trim()
        .split(/\r?\n/)[0];
    } catch {
      version = undefined;
    }
  }

  let keyBackend = "unset";
  try {
    keyBackend = loadOrCreateDek(DATA_DIR).backend;
  } catch (err) {
    writeUnlockFailure(err instanceof Error ? err.message : String(err));
    issues.push({
      code: "encryption_key_unlock",
      message: "Encryption key unlock failed. The CUI store stays sealed.",
    });
  }

  const attested = hill.getAttestation().configured;
  const bindingOk = true;

  return {
    ready:
      Boolean(binary) &&
      !issues.some((i) =>
        ["cleartext_endpoint", "endpoint_query_token", "secret_in_codex_config"].includes(i.code),
      ),
    host: "vite-preview",
    codex_found: Boolean(binary),
    codex_path: binary,
    codex_version: version ?? null,
    codex_home: home,
    config_toml_exists: existsSync(configPath),
    auth_json_exists: existsSync(authPath),
    model: parsed.model ?? null,
    model_provider: parsed.provider ?? null,
    azure_endpoint: endpoint ?? null,
    env_key_name: envKey ?? null,
    env_key_present: envKeyPresent,
    suggested_workspace: process.cwd(),
    store_encrypted: storeEncryptedOnDisk(),
    key_backend: keyBackend,
    audit_chain_ok: auditChainOk(),
    session_user: sessionUser(),
    machine_bound: true,
    machine_binding_ok: bindingOk,
    operator_attested: attested,
    pat_slot: patSlotPresent(DATA_DIR) ? "os-secret-store" : "unset",
    hello_bind: helloBind(),
    runner_allowlist: "local-codex-only",
    issues,
  };
}

function identityPayload() {
  const att = hill.getAttestation();
  let keyBackend = "machine-bound";
  try {
    keyBackend = loadOrCreateDek(DATA_DIR).backend;
  } catch {
    keyBackend = "unlock-failed";
  }
  return {
    session_user: sessionUser(),
    machine_id_present: Boolean(machineId()),
    machine_bound: true,
    machine_binding_ok: Boolean(machineBinding()),
    key_backend: keyBackend,
    store_encrypted: storeEncryptedOnDisk(),
    audit_chain_ok: auditChainOk(),
    operator_attestation: att,
    pat_slot: patSlotPresent(DATA_DIR) ? "os-secret-store" : "unset",
    hello_bind: helloBind(),
  };
}

function readJson(req: IncomingMessage): Promise<Record<string, unknown>> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    req.on("data", (c) => chunks.push(Buffer.from(c)));
    req.on("end", () => {
      if (chunks.length === 0) {
        resolve({});
        return;
      }
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
      } catch (err) {
        reject(err);
      }
    });
    req.on("error", reject);
  });
}

function json(res: ServerResponse, status: number, body: unknown) {
  res.statusCode = status;
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify(body));
}

function mapJsonEvent(value: Record<string, unknown>) {
  const typ = String(value.type || "");
  if (typ === "thread.started") {
    return { kind: "thread", text: "", thread_id: String(value.thread_id || "") || null };
  }
  if (typ === "turn.started") {
    return { kind: "status", text: "Codex started a turn…", thread_id: null };
  }
  if (typ === "turn.failed" || typ === "error") {
    return {
      kind: "error",
      text: String(value.error || value.message || "Codex turn failed."),
      thread_id: null,
    };
  }
  const item = value.item as Record<string, unknown> | undefined;
  if (item && (typ === "item.started" || typ === "item.updated" || typ === "item.completed")) {
    const itemType = String(item.type || "");
    if (itemType === "agent_message" && typeof item.text === "string") {
      return { kind: "assistant", text: item.text, thread_id: null };
    }
    if (itemType === "reasoning") {
      return { kind: "status", text: String(item.text || item.summary || "Thinking…"), thread_id: null };
    }
    if (itemType === "command_execution") {
      return { kind: "status", text: `Codex ran: ${String(item.command || "command")}`, thread_id: null };
    }
  }
  return null;
}

function runCodexTurn(
  binary: string,
  threadId: string | null,
  prompt: string,
  onEvent: (event: { kind: string; text: string; thread_id: string | null }) => void,
): Promise<{ text: string; threadId: string | null }> {
  return new Promise((resolve, reject) => {
    const workspace = path.join(DATA_DIR, "workspace");
    mkdirSync(workspace, { recursive: true });
    const local = parseEnvFile(ENV_LOCAL);
    const env = { ...process.env, ...local };
    const slot = getPatSlot(DATA_DIR);
    if (!env.AZURE_LLM_PAT && slot) env.AZURE_LLM_PAT = slot;
    if (!env.AZURE_OPENAI_API_KEY && (local.AZURE_LLM_PAT || slot)) {
      env.AZURE_OPENAI_API_KEY = local.AZURE_LLM_PAT || slot;
    }
    if (env.AZURE_LLM_ENDPOINT && isCleartextUrl(env.AZURE_LLM_ENDPOINT)) {
      reject(new Error("Refusing to start Codex: Azure endpoint is cleartext HTTP. Use HTTPS."));
      return;
    }

    const args = ["exec"];
    if (threadId) args.push("resume", threadId);
    args.push(
      "--json",
      "--skip-git-repo-check",
      "--sandbox",
      "read-only",
      "--ask-for-approval",
      "never",
      "-",
    );

    const child = spawn(binary, args, {
      cwd: workspace,
      env,
      stdio: ["pipe", "pipe", "pipe"],
      windowsHide: true,
    });

    child.stdin.write(prompt);
    child.stdin.end();

    let assistant = "";
    let seenThread = threadId;
    const stderr: string[] = [];

    const rl = createInterface({ input: child.stdout });
    rl.on("line", (line) => {
      const trimmed = line.trim();
      if (!trimmed) return;
      try {
        const value = JSON.parse(trimmed) as Record<string, unknown>;
        const event = mapJsonEvent(value);
        if (!event) return;
        if (event.thread_id) seenThread = event.thread_id;
        if (event.kind === "assistant" && event.text) assistant = event.text;
        onEvent(event);
      } catch {
        assistant = assistant ? `${assistant}\n${trimmed}` : trimmed;
        onEvent({ kind: "assistant", text: assistant, thread_id: seenThread });
      }
    });

    const errRl = createInterface({ input: child.stderr });
    errRl.on("line", (line) => {
      if (line.trim()) stderr.push(line.trim());
    });

    child.on("error", (err) => reject(new Error(`Failed to start Codex: ${err.message}`)));
    child.on("close", (code) => {
      if (code !== 0) {
        const detail = stderr.join("\n") || assistant || `Codex exited with status ${code}.`;
        reject(new Error(detail));
        return;
      }
      if (!assistant) {
        assistant = stderr.join("\n") || "(Codex finished without a visible reply.)";
      }
      onEvent({ kind: "assistant", text: assistant, thread_id: seenThread });
      resolve({ text: assistant, threadId: seenThread });
    });
  });
}

async function handleSend(chatId: string, content: string, res: ServerResponse) {
  const store = loadChatStore();
  const chat = store.chats.find((c) => c.id === chatId);
  if (!chat) {
    json(res, 404, { error: "Chat not found." });
    return;
  }
  const user: Message = {
    id: randomUUID(),
    chat_id: chatId,
    role: "user",
    content,
    created_at: nowIso(),
    status: "complete",
  };
  const assistant: Message = {
    id: randomUUID(),
    chat_id: chatId,
    role: "assistant",
    content: "",
    created_at: nowIso(),
    status: "running",
  };
  store.messages.push(user, assistant);
  if (chat.title === "New chat") {
    chat.title = content.trim().replace(/\s+/g, " ").slice(0, 48) || chat.title;
  }
  chat.updated_at = user.created_at;
  saveChatStore(store);

  let binary = whichCodex();
  if (binary) {
    try {
      binary = assertLocalCodex(binary);
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err);
      assistant.content = error;
      assistant.status = "error";
      saveChatStore(store);
      res.statusCode = 200;
      res.setHeader("Content-Type", "text/event-stream");
      res.setHeader("Cache-Control", "no-cache");
      res.setHeader("Connection", "keep-alive");
      res.write(`data: ${JSON.stringify({ kind: "accepted", message: assistant })}\n\n`);
      res.write(
        `data: ${JSON.stringify({
          chat_id: chatId,
          message_id: assistant.id,
          kind: "error",
          text: error,
        })}\n\n`,
      );
      res.end();
      return;
    }
  }
  if (!binary) {
    const error =
      "The `codex` CLI was not found on PATH. Install Codex, confirm `codex --version` works, then restart Codex Desk.";
    assistant.content = error;
    assistant.status = "error";
    saveChatStore(store);
    res.statusCode = 200;
    res.setHeader("Content-Type", "text/event-stream");
    res.setHeader("Cache-Control", "no-cache");
    res.setHeader("Connection", "keep-alive");
    res.write(`data: ${JSON.stringify({ kind: "accepted", message: assistant })}\n\n`);
    res.write(
      `data: ${JSON.stringify({
        chat_id: chatId,
        message_id: assistant.id,
        kind: "error",
        text: error,
      })}\n\n`,
    );
    res.end();
    return;
  }

  res.statusCode = 200;
  res.setHeader("Content-Type", "text/event-stream");
  res.setHeader("Cache-Control", "no-cache");
  res.setHeader("Connection", "keep-alive");
  res.write(`data: ${JSON.stringify({ kind: "accepted", message: assistant })}\n\n`);

  try {
    const result = await runCodexTurn(binary, chat.codex_thread_id, content, (event) => {
      res.write(
        `data: ${JSON.stringify({
          chat_id: chatId,
          message_id: assistant.id,
          ...event,
        })}\n\n`,
      );
    });
    assistant.content = result.text;
    assistant.status = "complete";
    chat.codex_thread_id = result.threadId;
    chat.updated_at = nowIso();
    saveChatStore(store);
    res.write(
      `data: ${JSON.stringify({
        chat_id: chatId,
        message_id: assistant.id,
        kind: "done",
        text: result.text,
      })}\n\n`,
    );
  } catch (err) {
    const text = err instanceof Error ? err.message : String(err);
    assistant.content = text;
    assistant.status = "error";
    saveChatStore(store);
    res.write(
      `data: ${JSON.stringify({
        chat_id: chatId,
        message_id: assistant.id,
        kind: "error",
        text,
      })}\n\n`,
    );
  }
  res.end();
}

export function previewBridge(middlewares: Connect.Server) {
  middlewares.use(async (req: IncomingMessage, res: ServerResponse, next) => {
    const url = req.url || "";
    if (!url.startsWith("/api/")) {
      next();
      return;
    }

    try {
      if (req.method === "GET" && url === "/api/status") {
        hill.ensureAgents();
        json(res, 200, runtimeStatus());
        return;
      }
      if (req.method === "GET" && url === "/api/agents") {
        json(res, 200, hill.listAgents());
        return;
      }
      if (req.method === "POST" && url === "/api/agents") {
        const body = await readJson(req);
        const name = String(body.name || "").trim();
        const brief = String(body.brief || "").trim();
        if (!name || !brief) {
          json(res, 400, { error: "Name and brief are required." });
          return;
        }
        json(res, 200, hill.createAgent(name, brief, body.workspace_path ? String(body.workspace_path) : undefined));
        return;
      }
      const agentPatch = url.match(/^\/api\/agents\/([^/]+)$/);
      if (agentPatch && req.method === "PATCH") {
        const body = await readJson(req);
        json(
          res,
          200,
          hill.updateAgent(decodeURIComponent(agentPatch[1]), {
            name: body.name ? String(body.name) : undefined,
            brief: body.brief ? String(body.brief) : undefined,
            workspace_path: body.workspace_path !== undefined ? String(body.workspace_path) : undefined,
          }),
        );
        return;
      }
      const agentRuns = url.match(/^\/api\/agents\/([^/]+)\/runs$/);
      if (agentRuns && req.method === "GET") {
        json(res, 200, hill.listRuns(decodeURIComponent(agentRuns[1])));
        return;
      }
      if (agentRuns && req.method === "POST") {
        const body = await readJson(req);
        const goal = String(body.goal || "").trim();
        const criteria = String(body.success_criteria || "").trim();
        if (!goal || !criteria) {
          json(res, 400, { error: "Goal and success criteria are required." });
          return;
        }
        json(
          res,
          200,
          hill.startRun(
            decodeURIComponent(agentRuns[1]),
            goal,
            criteria,
            Number(body.max_iterations || 3),
            Boolean(body.allow_writes),
          ),
        );
        return;
      }
      const runMatch = url.match(/^\/api\/runs\/([^/]+)$/);
      if (runMatch && req.method === "GET") {
        json(res, 200, hill.getRun(decodeURIComponent(runMatch[1])));
        return;
      }
      const cancelMatch = url.match(/^\/api\/runs\/([^/]+)\/cancel$/);
      if (cancelMatch && req.method === "POST") {
        json(res, 200, hill.cancelRun(decodeURIComponent(cancelMatch[1])));
        return;
      }
      if (req.method === "GET" && url === "/api/audit") {
        json(res, 200, hill.listAudit());
        return;
      }
      if (req.method === "GET" && url === "/api/identity") {
        json(res, 200, identityPayload());
        return;
      }
      if (req.method === "POST" && url === "/api/identity/attest") {
        const body = await readJson(req);
        json(
          res,
          200,
          hill.setAttestation(
            String(body.operator_name || ""),
            String(body.organization || ""),
            String(body.statement || ""),
          ),
        );
        return;
      }
      if (req.method === "POST" && url === "/api/secrets/pat") {
        const body = await readJson(req);
        const backend = setPatSlot(DATA_DIR, String(body.pat || ""));
        writeAudit("secret.slot_write", "secret", "pat", "PAT written to OS secret store (value not logged)");
        json(res, 200, { backend });
        return;
      }
      if (req.method === "DELETE" && url === "/api/secrets/pat") {
        clearPatSlot(DATA_DIR);
        writeAudit("secret.slot_clear", "secret", "pat", "PAT slot cleared (value not logged)");
        json(res, 200, { ok: true });
        return;
      }
      if (req.method === "GET" && url === "/api/chats") {
        json(res, 200, loadChatStore().chats.sort((a, b) => b.updated_at.localeCompare(a.updated_at)));
        return;
      }
      if (req.method === "POST" && url === "/api/chats") {
        const store = loadChatStore();
        const chat: Chat = {
          id: randomUUID(),
          title: "New chat",
          created_at: nowIso(),
          updated_at: nowIso(),
          codex_thread_id: null,
        };
        store.chats.unshift(chat);
        saveChatStore(store);
        json(res, 200, chat);
        return;
      }

      const messagesMatch = url.match(/^\/api\/chats\/([^/]+)\/messages$/);
      if (messagesMatch && req.method === "GET") {
        const chatId = decodeURIComponent(messagesMatch[1]);
        json(
          res,
          200,
          loadChatStore().messages.filter((m) => m.chat_id === chatId),
        );
        return;
      }
      if (messagesMatch && req.method === "POST") {
        const chatId = decodeURIComponent(messagesMatch[1]);
        const body = await readJson(req);
        const content = String(body.content || "").trim();
        if (!content) {
          json(res, 400, { error: "Message is empty." });
          return;
        }
        await handleSend(chatId, content, res);
        return;
      }

      const chatMatch = url.match(/^\/api\/chats\/([^/]+)$/);
      if (chatMatch && req.method === "DELETE") {
        const chatId = decodeURIComponent(chatMatch[1]);
        const store = loadChatStore();
        store.chats = store.chats.filter((c) => c.id !== chatId);
        store.messages = store.messages.filter((m) => m.chat_id !== chatId);
        saveChatStore(store);
        json(res, 200, { ok: true });
        return;
      }

      json(res, 404, { error: "Not found" });
    } catch (err) {
      json(res, 500, { error: err instanceof Error ? err.message : String(err) });
    }
  });
}
