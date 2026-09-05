import type {
  Agent,
  AuditEvent,
  Chat,
  HillclimbEvent,
  HillclimbIteration,
  HillclimbRun,
  IdentityStatus,
  Message,
  OperatorAttestation,
  RuntimeStatus,
  StreamEvent,
} from "./types";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export async function getRuntimeStatus(): Promise<RuntimeStatus> {
  if (isTauri()) {
    return invokeTauri<RuntimeStatus>("runtime_status");
  }
  const res = await fetch("/api/status");
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function listChats(): Promise<Chat[]> {
  if (isTauri()) {
    return invokeTauri<Chat[]>("list_chats");
  }
  const res = await fetch("/api/chats");
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function createChat(): Promise<Chat> {
  if (isTauri()) {
    return invokeTauri<Chat>("create_chat");
  }
  const res = await fetch("/api/chats", { method: "POST" });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function deleteChat(chatId: string): Promise<void> {
  if (isTauri()) {
    await invokeTauri("delete_chat", { chatId });
    return;
  }
  const res = await fetch(`/api/chats/${encodeURIComponent(chatId)}`, { method: "DELETE" });
  if (!res.ok) throw new Error(await res.text());
}

export async function listMessages(chatId: string): Promise<Message[]> {
  if (isTauri()) {
    return invokeTauri<Message[]>("list_messages", { chatId });
  }
  const res = await fetch(`/api/chats/${encodeURIComponent(chatId)}/messages`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function sendMessage(
  chatId: string,
  content: string,
  onEvent: (event: StreamEvent) => void,
): Promise<Message> {
  if (isTauri()) {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<StreamEvent>("codex-stream", (event) => {
      if (event.payload.chat_id !== chatId) return;
      onEvent(event.payload);
      if (event.payload.kind === "done" || event.payload.kind === "error") {
        unlisten();
      }
    });
    return invokeTauri<Message>("send_message", { chatId, content });
  }

  const res = await fetch(`/api/chats/${encodeURIComponent(chatId)}/messages`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ content }),
  });
  if (!res.ok || !res.body) {
    const text = await res.text();
    let message = text || "Failed to send message.";
    try {
      const parsed = JSON.parse(text) as { error?: string };
      if (parsed.error) message = parsed.error;
    } catch {
      // keep raw text
    }
    throw new Error(message);
  }

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  return new Promise<Message>((resolve, reject) => {
    let settled = false;
    const fail = window.setTimeout(() => {
      if (!settled) {
        settled = true;
        reject(new Error("Codex did not accept the turn."));
      }
    }, 15000);

    const pump = async () => {
      try {
        while (true) {
          const { value, done } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const parts = buffer.split("\n\n");
          buffer = parts.pop() ?? "";
          for (const part of parts) {
            const line = part.split("\n").find((l) => l.startsWith("data: "));
            if (!line) continue;
            const payload = JSON.parse(line.slice(6)) as StreamEvent & { message?: Message };
            if (payload.kind === "accepted" && payload.message) {
              if (!settled) {
                settled = true;
                window.clearTimeout(fail);
                resolve(payload.message);
              }
              continue;
            }
            onEvent(payload);
          }
        }
        if (!settled) {
          settled = true;
          window.clearTimeout(fail);
          reject(new Error("Codex did not accept the turn."));
        }
      } catch (err) {
        if (!settled) {
          settled = true;
          window.clearTimeout(fail);
          reject(err);
        }
      }
    };
    void pump();
  });
}

export async function listAgents(): Promise<Agent[]> {
  if (isTauri()) return invokeTauri<Agent[]>("list_agents");
  const res = await fetch("/api/agents");
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function createAgent(input: {
  name: string;
  brief: string;
  workspace_path?: string;
}): Promise<Agent> {
  if (isTauri()) {
    return invokeTauri<Agent>("create_agent", {
      name: input.name,
      brief: input.brief,
      workspacePath: input.workspace_path ?? null,
    });
  }
  const res = await fetch("/api/agents", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function updateAgent(
  agentId: string,
  patch: { name?: string; brief?: string; workspace_path?: string },
): Promise<Agent> {
  if (isTauri()) {
    return invokeTauri<Agent>("update_agent", {
      agentId,
      name: patch.name ?? null,
      brief: patch.brief ?? null,
      workspacePath: patch.workspace_path ?? null,
    });
  }
  const res = await fetch(`/api/agents/${encodeURIComponent(agentId)}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(patch),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function listAgentRuns(agentId: string): Promise<HillclimbRun[]> {
  if (isTauri()) return invokeTauri<HillclimbRun[]>("list_agent_runs", { agentId });
  const res = await fetch(`/api/agents/${encodeURIComponent(agentId)}/runs`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function getRun(runId: string): Promise<{ run: HillclimbRun; iterations: HillclimbIteration[] }> {
  if (isTauri()) {
    const pair = await invokeTauri<[HillclimbRun, HillclimbIteration[]]>("get_run", { runId });
    return { run: pair[0], iterations: pair[1] };
  }
  const res = await fetch(`/api/runs/${encodeURIComponent(runId)}`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function startHillclimb(input: {
  agentId: string;
  goal: string;
  successCriteria: string;
  maxIterations: number;
  allowWrites: boolean;
}): Promise<HillclimbRun> {
  if (isTauri()) {
    return invokeTauri<HillclimbRun>("start_hillclimb", {
      agentId: input.agentId,
      goal: input.goal,
      successCriteria: input.successCriteria,
      maxIterations: input.maxIterations,
      allowWrites: input.allowWrites,
    });
  }
  const res = await fetch(`/api/agents/${encodeURIComponent(input.agentId)}/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      goal: input.goal,
      success_criteria: input.successCriteria,
      max_iterations: input.maxIterations,
      allow_writes: input.allowWrites,
    }),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function cancelHillclimb(runId: string): Promise<HillclimbRun> {
  if (isTauri()) return invokeTauri<HillclimbRun>("cancel_hillclimb", { runId });
  const res = await fetch(`/api/runs/${encodeURIComponent(runId)}/cancel`, { method: "POST" });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function listAudit(): Promise<AuditEvent[]> {
  if (isTauri()) return invokeTauri<AuditEvent[]>("list_audit");
  const res = await fetch("/api/audit");
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function exportAudit(): Promise<AuditEvent[]> {
  if (isTauri()) {
    const raw = await invokeTauri<string>("export_audit");
    return JSON.parse(raw) as AuditEvent[];
  }
  const res = await fetch("/api/audit/export");
  if (!res.ok) throw new Error(await res.text());
  const body = (await res.json()) as { events?: AuditEvent[] };
  return body.events ?? [];
}

export async function listenHillclimb(onEvent: (event: HillclimbEvent) => void): Promise<() => void> {
  if (isTauri()) {
    const { listen } = await import("@tauri-apps/api/event");
    return listen<HillclimbEvent>("hillclimb-event", (event) => onEvent(event.payload));
  }
  return () => undefined;
}

export async function getIdentity(): Promise<IdentityStatus> {
  if (isTauri()) return invokeTauri<IdentityStatus>("identity_status");
  const res = await fetch("/api/identity");
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function setOperatorAttestation(input: {
  operator_name: string;
  organization: string;
  statement: string;
}): Promise<OperatorAttestation> {
  if (isTauri()) {
    return invokeTauri<OperatorAttestation>("set_operator_attestation", {
      operatorName: input.operator_name,
      organization: input.organization,
      statement: input.statement,
    });
  }
  const res = await fetch("/api/identity/attest", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export function isDesktopRuntime() {
  return isTauri();
}
