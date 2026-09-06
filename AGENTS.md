# Codex Desk — agent contract

This checkout is a **local operator shell** around the Codex CLI.
The model is whatever Azure deployment Codex is already configured
to use. Do not add Grok, Cursor, ChatGPT UI, or a direct Azure SDK.

Treat transcripts, agent briefs, hill-climb logs, and workspace
paths as **potentially CUI-bearing**.

Operator + worker default brief: `briefs/OPERATOR.md`. Desk injects
it on every `codex exec`. Do not water it with global `config.toml`
system prompts.

Desk is the **harness** around the operator’s existing local Codex CLI
and `config.toml` (hosted LLM). Not a second Azure or Grok client.
Connection is Codex `config.toml` only (endpoint + `env_key` for the PAT).
No second PAT store is required beyond what Codex already uses.

A prompt steers one inference. A harness governs the whole run.
Six jobs: **Contract** (goal/constraints/done), **Context** (rules/facts/state),
**Tools** (schemas/permissions/sandboxes), **State** (decisions/artifacts/open risks),
**Evidence** (tests/sources/screenshots), **Recovery** (retry locally, escalate,
improve the system).

Autonomy is earned by evidence — freedom inside boundaries, not freedom
from boundaries. Read/research → automatic. Write in workspace → automatic
+ checks (YOLO always-on; no in-app write-permission chrome). Send/merge/deploy
→ evidence + approval. Delete/pay/publish → explicit human confirmation.
YOLO does **not** waive send/merge/deploy or delete/pay/publish.

Failure should upgrade the harness: Run → Observe → Classify → Patch →
Verify → Accept. On fail, return the exact gap to Classify. Promote the
fix (map / tool / policy / test). The patch fixes one run. The harness
change improves every run after it.

## IL5 hard truths

Rubric: `docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md` (snapshot of
[KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol)).
Scanner brief: `docs/il5/AGENTS.md`. Official workbooks beat this
snapshot. Do not invent control counts.

- There is no official “FedRAMP Impact Level 5.” IL5 is FedRAMP High
  plus DoD overlays plus architecture constraints. High alone fails.
- Never claim ATO, FedRAMP authorization, DISA PA, or scanner-proof.
  The human / AO authorizes.
- Never write exploits, PoCs, payloads, or attack playbooks.
- Never put a PAT, API key, or token in source, SQLite, logs, or git.
- Do not “solve” IL5 by deleting audit logs, residual-risk tables,
  or secret-handling rules, or by writing “ATO complete.”
- Stay inside the workspace path the operator set. No home-directory
  sprawl. Do not `git push` unless the operator explicitly asked;
  Codex Desk itself never auto-pushes.

## Hill-climb worker

Spawn / validate / grade / judge. Iterate toward the stated goal
and success criteria. After each pass, leave a short summary of
what changed and what remains. HOLD on unvalidated claims. Desk
injects `briefs/OPERATOR.md` plus this contract via `codex exec`;
do not rely on global `config.toml` system prompts.

Run the six harness jobs. Workspace writes are YOLO (automatic +
checks). Do not send / merge / deploy without evidence + approval.
Do not delete / pay / publish without explicit human confirmation.
On fail, return the exact gap to Classify and promote a harness
change (map / tool / policy / test) when you can.

## Hill-climb grader

Return `GRADE: PASS | HOLD | WARN` plus gaps. For IL5 architecture
reviews, also emit the report block in `docs/il5/AGENTS.md`
(`READY` is prep-ready, not authorized). `HOLD` if the rubric file
is missing, if a claim is unvalidated, or if the change claims
authorization. YOLO workspace writes are not a HOLD. `HOLD` if
send / merge / deploy happened without evidence + approval, or
delete / pay / publish without human confirm. `HOLD` if a failure
was patched for this run only and the exact gap was not returned
for Classify / harness upgrade when a map, tool, policy, or test
could have been promoted.
