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
└─────────────┘     └──────┬───────┘     └─────────────────┘     └────────────┼─────────────┘
                           │                                                  ▼
                           ▼                                         Azure-hosted LLM
                    Encrypted vault                                  (endpoint + PAT
                    (AES-256-GCM + OS key)                           already in Codex)
```

IL5 mapping and residual risks: `SECURITY.md`. Rubric snapshot: `docs/il5/`
from [KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol).
This architecture is **not** an ATO.

## Auth

- Endpoint + PAT stay in Codex config / env / gitignored `.env.local`.
- Desk never writes the PAT to SQLite, audit logs, or git.
- Token-like lines are redacted in logs and Codex stderr display.
- Optional PAT slot is OS secret store / machine-bound wrap only.
- Identity: machine-bound unlock + optional operator record (CAC/PIV still MISSING). YOLO writes when a workspace is set; no in-app permission gate.

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
the app-data workspace. Hill-climb is YOLO `workspace-write` whenever the
operator set a workspace path. No Allow-writes checkbox and no attestation
HOLD. Home directory and filesystem root are refused.

### Agents and hill-climb

- Agent record: name, brief, status, optional workspace, independent
  worker/grader Codex thread ids.
- Run: goal, success criteria, max iterations, grade, gaps.
- Loop: worker Codex → grader Codex → PASS stop / HOLD or WARN feed gaps back
  until max or cancel.
- Seeded templates: Desk Improver, IL5 Architecture Grader.
- Prompts embed IL5 hard truths plus a Desk system block (`src-tauri/src/prompts.rs`, `src/lib/prompts.ts`).
- Agent jobs pass `--config project_doc_max_bytes=0` and `--config developer_instructions=…`. Auth still comes from the shared Codex home. Operator chat does not apply those overrides.
- UI theme token: `orbital` (`html[data-theme=orbital]`, `src/index.css`).
- Desk does not auto-commit or push.

### Store and audit

Encrypted envelope `codex-desk.db.enc` / `.data/preview-store.json.enc`
(AES-256-GCM, `CDEX1`). DEK in Windows DPAPI / OS keyring / machine-bound
wrap (`src-tauri/src/vault.rs`, `src-preview/secure-store.ts`). Hash-chained
audit inside the vault (`audit.rs`). Unix app-data dir mode `0700`.
No telemetry. Runner allowlist: local Codex binary only.

## Extension points (not in this slice)

| Later feature | Plug-in point |
|---|---|
| FIPS 140-3 CMVP evidence | Inheritance from OS / Codex / Azure — do not invent a module |
| CAC/PIV | Strengthen identity.rs beyond session bind (not an in-app write gate) |
| Tools / MCP | Codex `config.toml` only |
| Routines / cron | Same hill-climb runner, human-gated |
| SBOM / provenance | Follow-up on lockfiles already committed |

Do not add a second LLM client. If Codex cannot reach Azure, fix Codex config.
