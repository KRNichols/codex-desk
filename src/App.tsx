import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle,
  Bot,
  LoaderCircle,
  Menu,
  MessageSquarePlus,
  Trash2,
  X,
} from "lucide-react";
import { AgentPanel } from "@/components/AgentPanel";
import { IdentityPanel } from "@/components/IdentityPanel";
import { SetupEnvPanel } from "@/components/SetupEnvPanel";
import { Input } from "@/components/ui/input";
import { Badge, GradeBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import {
  createAgent,
  createChat,
  deleteChat,
  getRuntimeStatus,
  listAgents,
  listChats,
  listMessages,
  sendMessage,
} from "@/lib/runtime";
import { OPERATOR_CONTRACT } from "@/lib/prompts";
import type { Agent, Chat, Message, RuntimeStatus, StreamEvent } from "@/lib/types";
import { cn } from "@/lib/utils";

export default function App() {
  const [status, setStatus] = useState<RuntimeStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [chats, setChats] = useState<Chat[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [statusLine, setStatusLine] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [view, setView] = useState<"desk" | "agent" | "identity" | "setup">("desk");
  const [agents, setAgents] = useState<Agent[]>([]);
  const [activeAgentId, setActiveAgentId] = useState<string | null>(null);
  const [newAgentOpen, setNewAgentOpen] = useState(false);
  const [newAgentName, setNewAgentName] = useState("");
  const [newAgentBrief, setNewAgentBrief] = useState(OPERATOR_CONTRACT);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  const activeChat = useMemo(
    () => chats.find((c) => c.id === activeId) ?? null,
    [chats, activeId],
  );
  const activeAgent = useMemo(
    () => agents.find((a) => a.id === activeAgentId) ?? null,
    [agents, activeAgentId],
  );

  const refreshStatus = useCallback(async () => {
    try {
      setStatus(await getRuntimeStatus());
      setStatusError(null);
    } catch (err) {
      setStatusError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const refreshChats = useCallback(async (preferId?: string) => {
    const next = await listChats();
    setChats(next);
    setActiveId((current) => {
      if (preferId && next.some((c) => c.id === preferId)) return preferId;
      if (current && next.some((c) => c.id === current)) return current;
      return next[0]?.id ?? null;
    });
    return next;
  }, []);

  const refreshMessages = useCallback(async (chatId: string) => {
    setMessages(await listMessages(chatId));
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        await refreshStatus();
        const loaded = await listAgents();
        setAgents(loaded);
        const next = await refreshChats();
        if (next.length === 0) {
          const created = await createChat();
          await refreshChats(created.id);
          setMessages([]);
        }
      } catch (err) {
        setLoadError(err instanceof Error ? err.message : String(err));
      }
    })();
  }, [refreshChats, refreshStatus]);

  useEffect(() => {
    if (!activeId) return;
    void refreshMessages(activeId).catch((err) => {
      setLoadError(err instanceof Error ? err.message : String(err));
    });
  }, [activeId, refreshMessages]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [messages, statusLine]);

  async function handleNewChat() {
    const created = await createChat();
    await refreshChats(created.id);
    setMessages([]);
    setSidebarOpen(false);
  }

  async function handleDelete(chatId: string) {
    if (!window.confirm("Delete this chat? This is a delete-tier action and needs your confirmation.")) {
      return;
    }
    await deleteChat(chatId);
    const next = await refreshChats();
    if (next.length === 0) {
      const created = await createChat();
      await refreshChats(created.id);
      setMessages([]);
    }
  }

  function applyStream(event: StreamEvent) {
    if (event.kind === "status") {
      setStatusLine(event.text);
      return;
    }
    setMessages((prev) => {
      const existing = prev.find((msg) => msg.id === event.message_id);
      const nextStatus =
        event.kind === "done" ? "complete" : event.kind === "error" ? "error" : "running";
      if (!existing) {
        if (event.kind === "assistant" || event.kind === "done" || event.kind === "error") {
          return [
            ...prev,
            {
              id: event.message_id,
              chat_id: event.chat_id,
              role: "assistant",
              content: event.text,
              created_at: new Date().toISOString(),
              status: nextStatus,
            },
          ];
        }
        return prev;
      }
      return prev.map((msg) => {
        if (msg.id !== event.message_id) return msg;
        if (event.kind === "assistant" || event.kind === "done" || event.kind === "error") {
          return { ...msg, content: event.text || msg.content, status: nextStatus };
        }
        return msg;
      });
    });
    if (event.kind === "done" || event.kind === "error") {
      setBusy(false);
      setStatusLine(null);
      void refreshChats(event.chat_id);
    }
  }

  async function handleSend() {
    if (!activeId || busy) return;
    const text = draft.trim();
    if (!text) return;
    setDraft("");
    setBusy(true);
    setStatusLine("Starting Codex…");
    const user: Message = {
      id: `local-${Date.now()}`,
      chat_id: activeId,
      role: "user",
      content: text,
      created_at: new Date().toISOString(),
      status: "complete",
    };
    setMessages((prev) => [...prev, user]);
    try {
      const assistant = await sendMessage(activeId, text, applyStream);
      setMessages((prev) => {
        const withoutLocal = prev.filter((m) => m.id !== user.id);
        if (withoutLocal.some((m) => m.id === assistant.id)) {
          return withoutLocal;
        }
        return [...withoutLocal, user, assistant];
      });
      await refreshMessages(activeId);
      await refreshChats(activeId);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setMessages((prev) => [
        ...prev,
        {
          id: `err-${Date.now()}`,
          chat_id: activeId,
          role: "assistant",
          content: message,
          created_at: new Date().toISOString(),
          status: "error",
        },
      ]);
      setBusy(false);
      setStatusLine(null);
      try {
        await refreshMessages(activeId);
        await refreshChats(activeId);
      } catch {
        // keep the local error bubble if the store has not caught up
      }
    }
  }

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      <div className="console-rule" aria-hidden />
      <div className="flex min-h-0 flex-1">
      <aside
        className={cn(
          "fixed inset-y-0 left-0 z-30 flex w-72 flex-col border-r border-border bg-card transition-transform md:static md:translate-x-0",
          sidebarOpen ? "translate-x-0" : "-translate-x-full",
        )}
      >
        <div className="flex items-center justify-between px-4 py-3">
          <div>
            <p className="font-mono text-[10px] uppercase tracking-[0.28em] text-primary">Codex Desk</p>
            <h1 className="text-sm font-semibold tracking-wide">Local operator console</h1>
          </div>
          <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setSidebarOpen(false)}>
            <X />
          </Button>
        </div>
        <div className="px-4 pb-3 space-y-2">
          <Button className="w-full" onClick={() => void handleNewChat()}>
            <MessageSquarePlus />
            New chat
          </Button>
          <Button
            variant="outline"
            className="w-full"
            onClick={() => {
              setNewAgentOpen((v) => !v);
            }}
          >
            <Bot />
            New agent
          </Button>
        </div>
        {newAgentOpen ? (
          <div className="space-y-2 px-4 pb-3">
            <Input
              placeholder="Agent name"
              value={newAgentName}
              onChange={(e) => setNewAgentName(e.target.value)}
            />
            <Textarea
              placeholder="briefs/OPERATOR.md is the default Desk contract"
              value={newAgentBrief}
              onChange={(e) => setNewAgentBrief(e.target.value)}
              className="min-h-[64px]"
            />
            <Button
              size="sm"
              className="w-full"
              disabled={!newAgentName.trim()}
              onClick={() =>
                void (async () => {
                  const created = await createAgent({
                    name: newAgentName.trim(),
                    brief: newAgentBrief.trim() || OPERATOR_CONTRACT,
                    workspace_path: status?.suggested_workspace ?? undefined,
                  });
                  setAgents(await listAgents());
                  setActiveAgentId(created.id);
                  setView("agent");
                  setNewAgentOpen(false);
                  setNewAgentName("");
                  setNewAgentBrief(OPERATOR_CONTRACT);
                  setSidebarOpen(false);
                })()
              }
            >
              Create agent
            </Button>
          </div>
        ) : null}
        <Separator />
        <ScrollArea className="flex-1">
          <div className="space-y-1 p-3">
            <p className="px-2 pb-1 font-mono text-[11px] font-semibold uppercase tracking-[0.16em] text-foreground">
              Chats
            </p>
            {chats.length === 0 ? (
              <p className="px-2 py-4 text-sm text-muted-foreground">No chats yet.</p>
            ) : (
              chats.map((chat) => (
                <div
                  key={chat.id}
                  className={cn(
                    "group flex items-center gap-1 rounded-sm px-2 py-2 text-left text-sm",
                    activeId === chat.id ? "bg-accent" : "hover:bg-accent/60",
                  )}
                >
                  <button
                    className="min-w-0 flex-1 truncate text-left"
                    onClick={() => {
                      setActiveId(chat.id);
                      setView("desk");
                      setSidebarOpen(false);
                    }}
                  >
                    {chat.title}
                  </button>
                  <button
                    className="rounded p-1 text-muted-foreground opacity-0 hover:text-destructive group-hover:opacity-100"
                    onClick={() => void handleDelete(chat.id)}
                    aria-label={`Delete ${chat.title}`}
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </div>
              ))
            )}
            <p className="px-2 pb-1 pt-3 font-mono text-[11px] font-semibold uppercase tracking-[0.16em] text-foreground">
              Agents
            </p>
            <button
              className={cn(
                "flex w-full items-center justify-between rounded-sm px-2 py-2 text-left text-sm",
                view === "setup" ? "bg-accent" : "hover:bg-accent/60",
              )}
              onClick={() => {
                setView("setup");
                setSidebarOpen(false);
              }}
            >
              <span className="truncate">Setup / Env</span>
              <Badge variant={status?.env_key_present ? "pass" : "hold"}>
                {status?.env_key_present ? "FOUND" : "MISSING"}
              </Badge>
            </button>
            <button
              className={cn(
                "flex w-full items-center justify-between rounded-sm px-2 py-2 text-left text-sm",
                view === "identity" ? "bg-accent" : "hover:bg-accent/60",
              )}
              onClick={() => {
                setView("identity");
                setSidebarOpen(false);
              }}
            >
              <span className="truncate">Identity / audit</span>
              <Badge variant="pass">YOLO</Badge>
            </button>
            {agents.map((agent) => (
              <button
                key={agent.id}
                className={cn(
                  "flex w-full items-center justify-between rounded-sm px-2 py-2 text-left text-sm",
                  view === "agent" && activeAgentId === agent.id ? "bg-accent" : "hover:bg-accent/60",
                )}
                onClick={() => {
                  setActiveAgentId(agent.id);
                  setView("agent");
                  setSidebarOpen(false);
                }}
              >
                <span className="truncate">{agent.name}</span>
                <GradeBadge
                  grade={
                    agent.status === "blocked" || agent.status === "error"
                      ? "HOLD"
                      : agent.status === "done"
                        ? "PASS"
                        : agent.status === "running"
                          ? "WARN"
                          : null
                  }
                />
              </button>
            ))}
          </div>
        </ScrollArea>
        <Separator />
        <RuntimeCard status={status} error={statusError} onRefresh={() => void refreshStatus()} />
      </aside>

      {sidebarOpen ? (
        <button
          className="fixed inset-0 z-20 bg-black/50 md:hidden"
          onClick={() => setSidebarOpen(false)}
          aria-label="Close sidebar"
        />
      ) : null}

      <main className="flex min-w-0 flex-1 flex-col">
        <header className="flex items-center gap-3 border-b border-border px-4 py-2.5">
          <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setSidebarOpen(true)}>
            <Menu />
          </Button>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">
              {view === "setup"
                ? "Setup / Env"
                : view === "identity"
                ? "Identity / audit"
                : view === "agent"
                  ? (activeAgent?.name ?? "Agent")
                  : (activeChat?.title ?? "Codex Desk")}
            </p>
            <p className="truncate font-mono text-[11px] text-muted-foreground">
              User → Desk → local Codex CLI → Azure (shared Codex home)
            </p>
          </div>
          {busy ? (
            <Badge variant="hold" className="gap-1">
              <LoaderCircle className="size-3 animate-spin" />
              Codex working
            </Badge>
          ) : status?.codex_found ? (
            <Badge variant="pass">Codex ready</Badge>
          ) : (
            <Badge variant="hold">Setup needed</Badge>
          )}
        </header>

        {view === "setup" ? (
          <ScrollArea className="flex-1">
            <SetupEnvPanel onChange={() => void refreshStatus()} />
          </ScrollArea>
        ) : view === "identity" ? (
          <div className="mx-auto w-full max-w-3xl px-4 py-6">
            <IdentityPanel status={status} onChange={() => void refreshStatus()} />
          </div>
        ) : view === "agent" && activeAgent ? (
          <AgentPanel
            agent={activeAgent}
            status={status}
            onChange={(next) => setAgents((prev) => prev.map((a) => (a.id === next.id ? next : a)))}
          />
        ) : (
        <ScrollArea className="flex-1">
          <div className="mx-auto flex w-full max-w-3xl flex-col gap-4 px-4 py-6">
            {loadError ? <ErrorNote text={loadError} /> : null}
            {status && (!status.codex_found || status.issues.length > 0) ? (
              <SetupPanel status={status} />
            ) : null}
            {messages.length === 0 && (status?.codex_found ?? true) ? (
              <EmptyState
                ready={Boolean(status?.codex_found)}
                onOpenImprover={() => {
                  const improver = agents.find((a) => a.template === "desk-improver");
                  if (improver) {
                    setActiveAgentId(improver.id);
                    setView("agent");
                  }
                }}
              />
            ) : null}
            {messages.map((message) => (
              <TranscriptBubble key={message.id} message={message} />
            ))}
            {statusLine ? (
              <p className="flex items-center gap-2 font-mono text-xs text-muted-foreground">
                <LoaderCircle className="size-3.5 animate-spin" />
                {statusLine}
              </p>
            ) : null}
            <div ref={bottomRef} />
          </div>
        </ScrollArea>
        )}

        {view === "desk" ? (
        <div className="border-t border-border bg-card px-4 py-3">
          <form
            className="mx-auto flex w-full max-w-3xl flex-col gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              void handleSend();
            }}
          >
            <Textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  void handleSend();
                }
              }}
              placeholder={
                status?.codex_found
                  ? "Message Codex… Enter to send, Shift+Enter for a new line"
                  : "Send hello to test Codex. You’ll get a setup error if the CLI isn’t installed."
              }
              className="max-h-40 min-h-[72px] resize-none bg-background"
              disabled={busy}
            />
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs text-muted-foreground">
                Codex Desk never calls Azure itself and never stores a PAT in the repo.
              </p>
              <Button type="submit" disabled={busy || !draft.trim()}>
                {busy ? "Running…" : "Send"}
              </Button>
            </div>
          </form>
        </div>
        ) : null}
      </main>
      </div>
    </div>
  );
}

function TranscriptBubble({ message }: { message: Message }) {
  const isUser = message.role === "user";
  return (
    <article className={cn("flex", isUser ? "justify-end" : "justify-start")}>
      <div
        className={cn(
          "max-w-[90%] rounded-sm border px-4 py-3 text-sm leading-6",
          isUser
            ? "border-primary/40 bg-secondary text-foreground"
            : "border-border bg-card text-secondary-foreground",
          message.status === "error" && "border-hold/60 bg-hold/10 text-foreground",
        )}
      >
        <p className="mb-1 font-mono text-[10px] uppercase tracking-[0.16em] text-muted-foreground">
          {isUser ? "Operator" : "Codex"}
          {message.status === "running" ? " · writing" : ""}
          {message.status === "error" ? " · setup or runtime error" : ""}
        </p>
        <pre className="whitespace-pre-wrap font-sans">{message.content || (message.status === "running" ? "…" : "")}</pre>
      </div>
    </article>
  );
}

function EmptyState({ ready, onOpenImprover }: { ready: boolean; onOpenImprover: () => void }) {
  return (
    <div className="rounded-sm border border-dashed border-border bg-card px-5 py-8">
      <h2 className="text-lg font-semibold tracking-wide">Talk to your local Codex</h2>
      <p className="mt-2 max-w-xl text-sm text-muted-foreground">
        This desk is a chat shell. Each turn runs <code className="text-foreground">codex exec</code> on
        this machine. The model is whatever Azure deployment Codex already uses.
      </p>
      <p className="mt-3 text-sm">
        {ready
          ? "Smoke path: send hello and wait for the Codex reply in this transcript."
          : "Finish the first-run checklist in the sidebar, then send hello."}
      </p>
      <p className="mt-3 text-sm text-muted-foreground">
        Operators can also create independent hill-climb agents. Desk Improver can iterate on this
        checkout if you point its workspace at the repo. It will not claim ATO. Use Setup / Env to
        set Codex env_key values into the Desk vault.
      </p>
      <Button size="sm" variant="outline" className="mt-3" onClick={onOpenImprover}>
        Open Desk Improver
      </Button>
    </div>
  );
}

function SetupPanel({ status }: { status: RuntimeStatus }) {
  const missingCli = !status.codex_found;
  return (
    <section className="rounded-sm border border-hold/40 bg-hold/10 px-4 py-4 text-sm">
      <div className="mb-2 flex items-center gap-2 font-medium text-hold">
        <AlertTriangle className="size-4" />
        {missingCli ? "Codex is not on PATH" : "Setup checks"}
      </div>
      {missingCli ? (
        <ol className="list-decimal space-y-1 pl-5 text-foreground/90">
          <li>Install the Codex CLI so `codex --version` works in a terminal.</li>
          <li>
            Point Codex at Azure in <code>{status.codex_home}/config.toml</code> (HTTPS endpoint only; no PAT in the file).
          </li>
          <li>Open Setup / Env and set the named env_key in the Desk vault (or User env / gitignored .env.local).</li>
          <li>Restart Codex Desk and send hello.</li>
        </ol>
      ) : null}
      {!missingCli && status.issues.length > 0 ? (
        <ul className="mt-3 space-y-1 text-foreground/90">
          {status.issues.map((issue) => (
            <li key={issue.code}>• {issue.message}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}

function ErrorNote({ text }: { text: string }) {
  return (
    <div className="rounded-sm border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm whitespace-pre-wrap">
      {text}
    </div>
  );
}

function RuntimeCard({
  status,
  error,
  onRefresh,
}: {
  status: RuntimeStatus | null;
  error: string | null;
  onRefresh: () => void;
}) {
  if (error) {
    return (
      <div className="space-y-2 p-4 text-xs text-destructive">
        <p>Could not read runtime status.</p>
        <pre className="whitespace-pre-wrap">{error}</pre>
        <Button size="sm" variant="outline" onClick={onRefresh}>
          Retry
        </Button>
      </div>
    );
  }
  if (!status) {
    return <p className="p-4 text-xs text-muted-foreground">Checking local Codex…</p>;
  }
  const cells: [string, string][] = [
    ["home", status.codex_home],
    ["share", status.shared_provider_auth ? "provider+auth" : "not detected"],
    ["pat", `${status.env_key_name ?? "AZURE_LLM_PAT"} ${status.env_key_present ? "set" : "missing"}`],
    ["store", status.store_encrypted ? `enc ${status.key_backend ?? "os"}` : "unsealed"],
    ["ident", "session-bound YOLO"],
    ["jobs", "OPERATOR.md + --config"],
  ];
  return (
    <div className="space-y-2 p-4 text-xs text-muted-foreground">
      <div className="flex items-center justify-between">
        <span className="font-medium text-foreground">Codex runtime</span>
        <Badge variant={status.codex_found ? "pass" : "hold"}>
          {status.codex_found ? "found" : "missing"}
        </Badge>
      </div>
      <p className="text-[11px] leading-snug text-foreground/85">
        Shared Azure home with VS Code Codex. Desk injects briefs/OPERATOR.md on every exec.
      </p>
      <div className="grid grid-cols-2 gap-x-3 gap-y-2 font-mono text-[10px] leading-snug">
        {cells.map(([k, v]) => (
          <div key={k} className="min-w-0">
            <p className="uppercase tracking-[0.08em] text-foreground/70">{k}</p>
            <p className="truncate text-foreground" title={v}>
              {v}
            </p>
          </div>
        ))}
      </div>
      <Button size="sm" variant="outline" onClick={onRefresh}>
        Recheck
      </Button>
    </div>
  );
}
