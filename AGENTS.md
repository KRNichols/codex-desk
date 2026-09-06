# Codex Desk — agent contract

This checkout is a **local operator shell** (the **harness**) around **your**
Codex CLI and `config.toml`. The model is whatever Azure deployment Codex is
already configured to use.

Desk is **not** a second Azure, Grok, Cursor, or ChatGPT client. It does not
add an SDK or a second PAT store. Connection is Codex `config.toml` only
(endpoint + `env_key` for the PAT).

```
You  →  Codex Desk  →  local `codex` CLI  →  config.toml (endpoint + env_key)  →  Azure-hosted LLM
```

Desk injects `briefs/OPERATOR.md` on every `codex exec` (prompt + `--config
developer_instructions`). It does **not** use `--ignore-user-config` (that
would drop the Azure provider). Do not water the contract with global
`config.toml` system prompts.

A prompt steers one inference. This harness governs the whole run.

## Minimum viable harness

1. **Contract** — goal, constraints, done
2. **Context** — rules, facts, state
3. **Tools** — schemas, permissions, sandboxes (Setup / Env for `config.toml` `env_key` names)
4. **State** — persist decisions, artifacts, open risks
5. **Evidence** — tests, sources, screenshots
6. **Recovery** — retry locally, escalate, improve the system

Run these six jobs on every turn and on every hill-climb record.

## Autonomy by consequence

Freedom inside boundaries — not freedom from them. Increase control only when
consequence increases.

| Consequence | Control |
|---|---|
| Read / research | Automatic |
| Write in workspace | Automatic + checks (**YOLO** always-on; no in-app write-permission chrome) |
| Send / merge / deploy | Evidence + approval |
| Delete / pay / publish | Explicit human confirmation |

YOLO always-on means: when a workspace path is set, hill-climb uses
`workspace-write` with no attestation prompt and no “Allow workspace writes”
checkbox. That does **not** waive send / merge / deploy or delete / pay /
publish. Desk never auto-pushes.

## Failure upgrades the harness

Run → Observe → Classify → Patch → Verify → Accept.

On fail, return the **exact gap** to Classify. Do not retry blindly. Promote
the fix into the harness: **map / tool / policy / test** (or the brief / loop).
The patch fixes one run. The harness change improves every run after it.

## Orchestration

Practical loop. Not a second runtime.

- **Operator chat** — local `codex exec --json --sandbox read-only` in the
  app-data workspace. Resume the Codex `thread_id` when Codex emits one.
- **Agents** — independent records (name, brief, optional workspace, worker
  and grader thread ids). Seeded: **Desk Improver**, **IL5 Architecture Grader**.
- **Hill-climb** — spawn / validate / grade / judge until PASS, HOLD, cancel,
  or max iterations (README example: 3).
  1. Worker `codex exec` in the agent workspace (`workspace-write` if a
     workspace path is set; otherwise read-only app-data workspace).
  2. Grader `codex exec` is **read-only**. Return `GRADE: PASS | HOLD | WARN`
     plus gaps.
  3. Desk scores the six jobs, classifies a gap on HOLD/WARN, offers Promote.
     Desk Improver may auto-promote after Verify. Operator can Promote anytime.
  4. Send/merge/deploy goals wait for evidence + approval. Delete/pay/publish
     wait for explicit confirm. Then the loop may continue.
- **Workspace** — set a real checkout (Windows example: `C:\src\codex-desk`).
  Home directory and filesystem root are refused. Empty path → no home-dir
  writes; YOLO writes require a workspace path.
- **Injected briefs** — every exec gets `briefs/OPERATOR.md` plus
  `--config developer_instructions=…` and `project_doc_max_bytes=0`. Agent
  jobs also get the worker or grader prompt. Follow this file + OPERATOR.md +
  `docs/il5/` hard truths. Ignore helpful-assistant global prompts.
- **No auto-push.** Do not `git commit` or `git push` unless the operator’s
  goal explicitly asked. Review the diff yourself.
- **IL5 grader** — score the handed workspace against
  `docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md`, `docs/il5/PRODUCT-CHECKLIST.md`,
  and `docs/il5/AGENTS.md`. `READY` / product `PASS` is prep-ready, not
  authorized. AO/tenant/Azure/FIPS rows may stay `MISSING`.

`HOLD` on unvalidated claims, authorization claims, send/merge/deploy without
evidence + approval, or delete/pay/publish without confirm. YOLO workspace
writes are not a HOLD. After each pass: what changed, what remains.

## Install / run (Windows)

Native Windows is the primary target. Use **PowerShell**.

### Prerequisites

- **Git**
- **Node.js 20+** — [nodejs.org](https://nodejs.org)
- **Rust** (stable, for Tauri 2) — [rustup](https://www.rust-lang.org/tools/install). This repo pins `stable` in `rust-toolchain.toml`.
- **WebView2** — usually already on Windows 10/11; install the [Evergreen runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if prompted.
- **Visual Studio C++ Build Tools** — required by Tauri 2 on Windows ([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

### Codex CLI

```powershell
npm install -g @openai/codex
codex --version
```

`codex` must be on the **Windows** PATH (`codex`, `codex.exe`, or `codex.cmd`).

### Clone and dependencies

```powershell
git clone https://github.com/KRNichols/codex-desk.git
cd codex-desk
npm install
```

### Run

```powershell
npm run dev
```

That is `tauri dev` (desktop app). First compile can take a few minutes.

| Command | What it does |
|---|---|
| `npm run dev` | Desktop app (`tauri dev`) |
| `npm run build` | Native installer via Tauri |
| `npm run dev:ui` | Vite UI only, `http://127.0.0.1:47321` (preview host, not the product) |

Smoke path: open the app → send `hello` → expect a real Codex reply. If Codex
is missing or Azure auth is incomplete, the UI shows a setup error — not a fake
model. Confirm keys in **Setup / Env** (FOUND / MISSING). Do not invent FOUND.

Linux/macOS may work if `codex` is on PATH; they are not the v0 target. The
Windows app reads `%USERPROFILE%\.codex\`, not a WSL Linux home.

## Config / secrets

`config.toml` holds **endpoint + `env_key` only**. The PAT lives in the
environment or the Desk **Setup / Env** vault (exported **only** to the child
`codex` process). The vault is optional and is not a second Azure client.
Setup / Env lists `env_key` names plus Azure template vars; it never returns
secret values. Full template: README [Connection](README.md#connection-codex-configtoml-only).

Windows path: `%USERPROFILE%\.codex\config.toml`

```toml
model = "YOUR_AZURE_DEPLOYMENT_NAME"
model_provider = "azure"

[model_providers.azure]
name = "Azure OpenAI"
base_url = "https://YOUR_RESOURCE.openai.azure.com/openai/v1"
env_key = "AZURE_LLM_PAT"
wire_api = "responses"
```

Do **not** put the PAT (or a credentialed URL) in `config.toml`, source, SQLite,
logs, or git. Copy `.env.example` → `.env.local` (gitignored) **or** set User env:

```powershell
$env:AZURE_LLM_ENDPOINT = "https://YOUR_RESOURCE.openai.azure.com/openai/v1"
$env:AZURE_LLM_PAT = "<paste PAT in the terminal, not in git>"
```

Persistent User env:

```powershell
[System.Environment]::SetEnvironmentVariable("AZURE_LLM_ENDPOINT", "https://YOUR_RESOURCE.openai.azure.com/openai/v1", "User")
[System.Environment]::SetEnvironmentVariable("AZURE_LLM_PAT", "<PAT>", "User")
```

Then open a **new** terminal / restart Desk. Optional: save the PAT in **Setup /
Env** (vault → child `codex` only).

## Hard rules

- **No ATO / FedRAMP / DISA PA claims.** Never claim authorization or
  scanner-proof. The human / AO authorizes.
- **IL5 product PASS ≠ authorization.** IL5 = FedRAMP High + DoD overlays +
  architecture; High alone fails. **READY** = every product-owned row in
  `docs/il5/PRODUCT-CHECKLIST.md` is `PASS` — prep-ready for a human GRC look
  at this local operator shell, never an ATO. AO/tenant/Azure PA and FIPS CMVP
  stay MISSING/external.
- **No SpaceX or vendor aerospace branding.** UI token is **orbital** /
  aero-night only. Never vendor aerospace names, logos, or wordmarks.
- **SIM-Windows ≠ a real air-gapped Windows workstation.** A cloud, Linux, or
  simulated Windows host is not IL5 isolation, an air gap, or CAC/PIV. Do not
  treat this environment as authorized or air-gapped.
- **Do not invent FOUND secrets** in docs, UI copy, or reviews.
- **Never** write exploits, PoCs, payloads, or attack playbooks.
- **Never** put a PAT, API key, or token in source, SQLite, logs, or git.
- Do not “solve” IL5 by deleting audit logs, residual-risk tables, or
  secret-handling rules, or by writing “ATO complete.”
- Do not weaken AES-256-GCM + OS-backed DEK, hash-chained audit, TLS-only
  endpoints, or local-Codex-only egress. Desk does not phone home or open
  Azure sockets.
- Stay in the operator’s workspace. No home-directory sprawl.
- Official IL5 workbooks beat this snapshot (`docs/il5/`). Do not invent
  control counts.

Theme, voice, and the injected exec contract stay in `briefs/OPERATOR.md`.
Do not fatten that brief with install steps.
