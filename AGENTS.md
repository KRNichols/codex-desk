# Codex Desk — agent contract

This checkout is a **local operator shell** around the Codex CLI.
The model is whatever Azure deployment Codex is already configured
to use. Do not add Grok, Cursor, ChatGPT UI, or a direct Azure SDK.

Treat transcripts, agent briefs, hill-climb logs, and workspace
paths as **potentially CUI-bearing**.

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

Iterate toward the stated goal and success criteria. After each
pass, leave a short summary of what changed and what remains.

## Hill-climb grader

Return `GRADE: PASS | HOLD | WARN` plus gaps. For IL5 architecture
reviews, also emit the report block in `docs/il5/AGENTS.md`
(`READY` is prep-ready, not authorized). `HOLD` if the rubric file
is missing or if the change claims authorization.
