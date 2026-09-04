# Codex Desk

A personal desktop chat desk that shells the **local Codex CLI**. Codex Desk does not talk to Grok, Cursor models, ChatGPT’s UI, or Azure OpenAI SDKs. The model is whatever **Azure-hosted deployment Codex is already configured to use**.

```
You  →  Codex Desk  →  local `codex` CLI  →  Codex config (endpoint + PAT)  →  Azure-hosted LLM
```

Working name: **Codex Desk**. Rename later if you want.

## IL5 posture (aligned, not authorized)

Codex Desk is built against the public rubric in
[KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol)
(local snapshot: `docs/il5/`). **This is not an ATO, FedRAMP authorization, or
DISA PA.** The human / AO authorizes.

IL5 is **FedRAMP High + DoD overlays + architecture constraints**. High alone
fails. Official workbooks beat blog control counts. See `SECURITY.md` for a
theme → implementation → PASS/PARTIAL/MISSING table. Residual risks (plaintext
SQLite, no CAC/PIV, no Desk FIPS module) are marked **MISSING** on purpose.

**Shared responsibility:** you / the AO own categorization, Azure tenant IL5
posture, endpoint+PAT handling, and the mission ATO. Codex Desk is a local
operator shell. It does not phone home. The only egress is Codex → your
configured Azure endpoint.

## What this build does

- Native desktop shell (Tauri 2 + React + TypeScript). Windows is the primary target.
- Operator chat plus independent **agents** and **hill-climb** jobs (iterate → grade → fix until PASS/HOLD).
- Each Codex turn is still local `codex exec --json` (resume per agent worker/grader thread).
- Local store (SQLite / preview JSON) treated as potentially CUI-bearing. PAT never stored there.
- Structured local audit events (no secret values).
- Clear setup errors if `codex` is missing or Azure auth is incomplete.

## What this build does not do

- No Grok / Cursor / ChatGPT integration and no Azure SDK in the app.
- No PAT or real endpoint in the repo.
- No fake SSO/CAC. No automatic git push. No routines/MCP marketplace/cloud VMs.

## Auth model (endpoint + PAT, local only)

You were given an Azure HTTP endpoint and a personal access token (PAT). Those stay on your machine.

**Never** put the PAT, or an endpoint URL that embeds credentials, in source files that get committed.

Preferred setup:

1. Put the **endpoint** in Codex’s own config (`base_url`).
2. Put the **PAT** in an environment variable (or a gitignored `.env.local`).
3. Point Codex `env_key` at that variable.
4. Launch Codex Desk. The app only starts `codex` and inherits that environment.

### Codex `config.toml` (no secrets)

Windows (native): `%USERPROFILE%\.codex\config.toml`

WSL: `~/.codex/config.toml`

```toml
model = "YOUR_AZURE_DEPLOYMENT_NAME"
model_provider = "azure"

[model_providers.azure]
name = "Azure OpenAI"
base_url = "https://YOUR_RESOURCE.openai.azure.com/openai/v1"
env_key = "AZURE_LLM_PAT"
wire_api = "responses"
```

Use the endpoint you were given. Do not append the PAT to the URL. If your contact did not give a deployment / model name, set `model` to whatever they use for that Azure resource — Codex Desk will not invent one.

### Environment variables

Copy `.env.example` to `.env.local` (repo, for `npm run dev`) **or** create the same keys as User environment variables on Windows.

```
AZURE_LLM_ENDPOINT=https://YOUR_RESOURCE.openai.azure.com/openai/v1
AZURE_LLM_PAT=
```

Windows User env (PowerShell, current session):

```powershell
$env:AZURE_LLM_ENDPOINT = "https://YOUR_RESOURCE.openai.azure.com/openai/v1"
$env:AZURE_LLM_PAT = "<paste PAT in the terminal, not in git>"
```

Persistent User env (PowerShell):

```powershell
[System.Environment]::SetEnvironmentVariable("AZURE_LLM_ENDPOINT", "https://YOUR_RESOURCE.openai.azure.com/openai/v1", "User")
[System.Environment]::SetEnvironmentVariable("AZURE_LLM_PAT", "<PAT>", "User")
```

Then start a **new** terminal / restart Codex Desk so it sees the variables.

If Codex examples expect `AZURE_OPENAI_API_KEY`, you can use that name instead. When `AZURE_LLM_PAT` is set and `AZURE_OPENAI_API_KEY` is not, Codex Desk exports the PAT as `AZURE_OPENAI_API_KEY` **only** to the child `codex` process.

Optional app-data file (Windows): `%APPDATA%\com.codexdesk.app\.env.local`  
Same placeholder keys as `.env.example`. Never commit it.

## First-run checklist

1. Install Node.js 20+ and Rust (Tauri prerequisites). On Windows also install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if prompted.
2. Install the Codex CLI and confirm it is on PATH:
   ```powershell
   npm install -g @openai/codex
   codex --version
   ```
3. Write `%USERPROFILE%\.codex\config.toml` with your Azure **endpoint** (see above). Do not paste the PAT into that file.
4. Set `AZURE_LLM_PAT` (and optionally `AZURE_LLM_ENDPOINT`) in User env or `.env.local`.
5. From this repo:
   ```powershell
   npm install
   npm run dev
   ```
6. Smoke path: open the app → send `hello` → the transcript should show a real Codex reply. If Codex is missing or the PAT/endpoint is wrong, the UI shows a setup error pointing at Codex config — not a fake model.

`npm run dev` launches the desktop app (`tauri dev`).  
`npm run dev:ui` is the Vite UI on `http://127.0.0.1:47321` (same Codex runner, preview host only).  
`npm run build` produces the native installer via Tauri.

## Windows notes

- This app is meant to run **natively on Windows**, not only inside WSL.
- Codex config for the Windows app is `%USERPROFILE%\.codex\`, not the Linux home inside WSL.
- If you develop in WSL but use the Windows Codex install, put `codex` on the **Windows** PATH and set the PAT in **Windows** User env.
- Codex Desk searches `PATH` for `codex`, `codex.exe`, and `codex.cmd`, plus common npm global folders.

## Hill-climb Codex Desk itself

1. Confirm `codex --version` and Azure endpoint + PAT via Codex config (not this repo).
2. Open **Desk Improver** in the sidebar (seeded agent).
3. Set **workspace** to this checkout (Windows example: `C:\src\codex-desk`). Home directory is refused.
4. Goal example: `Clarify the README smoke path without claiming ATO.`
5. Success criteria example: `A newcomer can run npm run dev and send hello; SECURITY.md residual risks stay marked MISSING.`
6. Max iterations 3. Check **Allow workspace writes** only if you want Codex to edit that path. Review the diff yourself. Desk will not push.
7. Watch iteration N, last grade (PASS/HOLD/WARN), and gaps. Cancel anytime.
8. Optional: run **IL5 Architecture Grader** on the same checkout. `READY`/`PASS` means prep-ready for a human GRC look — never authorized.

Hill-climb worker and grader briefs include IL5 hard truths. They cannot “solve” IL5 by claiming ATO or deleting audit/secret rules.

## Known limits

- Operator chat uses `codex exec` read-only. Hill-climb writes only if you set a workspace and enable writes; still `--ask-for-approval never`.
- SQLite is plaintext on disk (SC-28 **MISSING**). See `SECURITY.md`.
- Follow-up turns resume the Codex thread when Codex emits a `thread_id`. If resume is unavailable, the next turn starts a new exec and Codex will not see prior CLI context (the Desk transcript still has it).
- The Vite preview host is for development in a browser. The product is the desktop app.
- Linux/macOS are nice-to-have; they may work if `codex` is on PATH, but they are not the v0 target.
- No secrets, API keys, or sample PATs ship in this repository.

## License

MIT. See `LICENSE`.
