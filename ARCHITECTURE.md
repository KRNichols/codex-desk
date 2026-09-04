# Codex Desk architecture

Codex Desk is a **desktop shell around the local Codex CLI**. It is not a model
client and not a CSO.

The model is whatever **Azure-hosted deployment Codex is configured to use**.
This app never calls Azure, Grok, Cursor, or ChatGPT APIs.

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐     ┌──────────────────────────┐
│ React UI    │ ──► │ App core     │ ──► │ Codex runner    │ ──► │ User's Codex CLI         │
│ chats       │     │ Tauri cmds   │     │ spawn + JSONL   │     │ config.toml + env + PAT  │
│ agents      │     │ or Vite /api │     │ hill-climb loop │     │            │             │
└─────────────┘     └──────┬───────┘     └─────────────────┘     └─────────────┼─────────────┘
                           │                                                  ▼
                           ▼                                         Azure-hosted LLM
                    SQLite + audit.jsonl                             (endpoint + PAT
                    (treat as potential CUI)                         already in Codex)
```

IL5 mapping and residual risks: `SECURITY.md`. Rubric snapshot: `docs/il5/`
from [KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol).
This architecture is **not** an ATO.

## Auth

- Endpoint + PAT stay in Codex config / env / gitignored `.env.local`.
- Desk never writes the PAT to SQLite, audit logs, or git.
- Token-like lines are redacted in logs and Codex stderr display.
- CAC/PIV / enterprise SSO is **MISSING** (local OS user only).

## Layers

### UI

Operator desk (chats) plus an Agents sidebar. Agent detail starts hill-climb
jobs without blocking the operator composer.

### App core

Tauri commands for chats, agents, hill-climb start/cancel, and audit list.
Events: `codex-stream`, `hillclimb-event`. Vite preview mirrors the HTTP API.

### Codex runner

Still the only model runtime. `run_turn` accepts an optional workspace +
sandbox (`read-only` or `workspace-write`). Default chat stays read-only in
the app-data workspace. Hill-climb writes only when the operator set a
workspace path and checked allow-writes. Home directory and filesystem root
are refused.

### Agents and hill-climb

- Agent record: name, brief, status, optional workspace, independent
  worker/grader Codex thread ids.
- Run: goal, success criteria, max iterations, grade, gaps.
- Loop: worker Codex → grader Codex → PASS stop / HOLD or WARN feed gaps back
  until max or cancel.
- Seeded templates: Desk Improver, IL5 Architecture Grader.
- Prompts embed IL5 hard truths (`src-tauri/src/prompts.rs`, `src/lib/prompts.ts`).
- Desk does not auto-commit or push.

### Store and audit

SQLite (`codex-desk.db`) plus `audit.jsonl`. Preview: `.data/preview-store.json`.
**Plaintext on disk** (SC-28 MISSING). Unix app-data dir mode `0700`.
No telemetry.

## Extension points (not in this slice)

| Later feature | Plug-in point |
|---|---|
| SQLCipher + DPAPI key | Store open path; do not invent a FIPS module |
| CAC/PIV | Identity gate before app data unlock |
| Tools / MCP | Codex `config.toml` only |
| Routines / cron | Same hill-climb runner, human-gated |
| SBOM / provenance | Follow-up on lockfiles already committed |

Do not add a second LLM client. If Codex cannot reach Azure, fix Codex config.
