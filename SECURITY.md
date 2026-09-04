# Codex Desk security and IL5 posture

**This product is not authorized.** Codex Desk is an IL5-**aligned
engineering build** of a local operator shell. It does not have a
FedRAMP authorization, DISA Provisional Authorization, or mission
ATO. The human / Authorizing Official (AO) decides those. Do not
treat this file as an SSP.

**IL5 is not “FedRAMP High.”** There is no official “FedRAMP Impact
Level 5.” IL5 is **FedRAMP High plus DoD overlays plus architecture
constraints** (CC SRG / CSP SRG). High alone fails an IL5 assessment.

**Rubric (do not invent control counts):**
[KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol)
— local snapshot in `docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md` (compiled
4 Sep 2026) and `docs/il5/AGENTS.md`. Official sources of truth remain
FedRAMP Appendix A High, the DoD SSP Addendum on
[cyber.mil](https://public.cyber.mil/dccs/dccs-documents/), and
CNSSI 1253. If those disagree with the snapshot, the official
workbook wins (`docs/il5/SOURCE.txt`).

Codex Desk is **not a CSO**. It is a local-first desktop operator
shell. Most IL5 CSP / BCAP / SCCA / tenancy requirements belong to
the user’s Azure-hosted model path and mission enclave, not this
binary. Mark those **MISSING / N/A (inherited or out of Desk
boundary)** rather than “passed because we are local.”

## Shared responsibility

| Owner | Owns |
|---|---|
| **User / mission AO** | Categorization (FIPS 199 / NSS / CUI), Azure tenant IL5 posture, endpoint + PAT handling, workstation hardening, CAC/PIV policy, mission ATO |
| **Codex CLI + Azure deployment** | TLS to the model, provider auth, whatever FIPS modules that stack actually runs |
| **Codex Desk** | Local UI, local store, Codex process spawn, hill-climb loop, local audit events, secret **non-storage** |

## Data assumption

Chat transcripts, agent briefs, hill-climb logs, and workspace paths
are treated as **potentially CUI-bearing**. Backups of the app data
directory are in-scope for the operator (standard §8.5).

## Mapping (theme → Desk → grade)

Grades are `PASS` / `PARTIAL` / `MISSING` for **this slice**, not
control-by-control 800-53 scores. Official IDs are cited as themes
only.

| Theme (standard) | How Desk addresses it | Grade | Next step if not PASS |
|---|---|---|---|
| Categorization (§3) | Docs assume potential CUI; no AO memo in-app | **MISSING** | Mission AO writes CIA / IL / NSS memo; Desk will not invent it |
| Four-layer stack (§4) | Desk is not a CSO; docs refuse High-only-as-IL5 | **N/A / MISSING** | Package Azure + workstation separately; do not claim Desk covers FedRAMP+ / CNSSI 1253 |
| Secrets / authenticators (§9.4, IA-05) | PAT never in git or SQLite. Env / Codex `config.toml` `env_key` / `.env.local` (gitignored). Logs and UI redact token-like lines | **PARTIAL** | Windows DPAPI / OS keychain wrap for PAT (not shipped) |
| Data at rest SC-28 (§9.2) | SQLite + preview JSON + `audit.jsonl` are **plaintext files** in the OS app-data directory. Unix dir mode `0700` when created. No SQLCipher | **MISSING** | SQLCipher (or equivalent) with a DPAPI-/keychain-wrapped key; FIPS 140-3 module — do not invent one |
| Data in transit SC-08 | Desk does not open Azure sockets. Codex → Azure uses the platform TLS stack | **PARTIAL** (inherited) | Confirm Codex + OS FIPS mode; list CMVP certs in a real Appendix Q |
| FIPS 140-3 (§9) | No custom crypto. No CMVP module shipped by Desk | **MISSING** | Inherit validated modules from OS / Codex / Azure; document cert numbers. “AES-256” alone is not enough |
| Identity / CAC/PIV (§10) | v1 is **local-user, machine-bound**. No SSO, no CAC | **MISSING** | Enterprise IdP / CAC-PIV at Credential Strength D; do not add a password fallback that drops below Strength D |
| Audit AU family / §19 | Structured local events: agent create/update, hill-climb start/stop/cancel, grade, secret-access **failures** (no secret values). JSONL + SQLite | **PARTIAL** | Central SIEM/CSSP feed, crypto-protected store (AU-09), 12+18 or contract retention, NTP-backed timestamps |
| Retention | Operator keeps app-data; no automated purge. Suggested: treat as CUI and retain per mission policy (M-21-31 / contract — **not implemented**) | **MISSING** | Configurable retention + signed export |
| Network boundary (§8, §11) | Desk does not phone home. No analytics. Egress is Codex → the user’s configured Azure endpoint only | **PARTIAL** | Mission BCAP/SCCA if this workstation is DoD-connected; Desk cannot provide BCAP |
| Tenancy / US location (§8) | Local process on the operator’s machine. No Desk cloud | **N/A** | AO still owns where that machine and Azure tenant live |
| Scan program (§14) | Not a scan platform. Hill-climb grader scores handed code/docs only | **MISSING** | Authenticated OS/web/SAST/secrets program for the mission package |
| POA&M / ConMon (§16–17) | Residual risks listed here as MISSING | **MISSING** | Mission POA&M; do not bundle rows |
| Package artifacts (§23) | SECURITY.md + ARCHITECTURE.md only | **MISSING** | SSP / CRM / boundary diagrams are AO work |
| Supply chain | `package-lock.json` and `src-tauri/Cargo.lock` committed | **PARTIAL** | SBOM (CycloneDX), provenance attestations, dependency review cadence |
| Telemetry | None from Desk | **PASS** for this binary | Keep it that way |
| Hill-climb / self-improve | Worker + grader briefs embed IL5 hard truths; cannot “pass” by claiming ATO or dropping audit/secret rules | **PARTIAL** | Human review of every write; no unattended push |
| Exploits / pentest (§15) | Agents forbidden to write exploits/PoCs | **PASS** (policy) | Mission pentest is a separate 3PAO activity |
| CMMC / 800-171 as IL5 (§27) | Docs refuse the mix-up | **PASS** (policy) | Keep COCO vs CSP paths separate |

## Residual risks (do not paper over)

1. **Plaintext local database** — anyone with filesystem access to the app-data directory can read transcripts and briefs.
2. **No CAC/PIV** — a logged-on OS user is the only identity.
3. **No FIPS module of our own** — transit/rest crypto is inherited or absent.
4. **No ConMon / scan program** for Desk as a CSO (it is not one).
5. **Hill-climb with workspace-write** can edit the path the operator selected. Desk does not auto-commit or push; the operator still reviews diffs.

## Audit events (v1)

Written to `audit_events` (SQLite) and `audit.jsonl` under the app data
dir (Windows: `%APPDATA%\com.codexdesk.app\`). Fields: `at` (RFC3339),
`action`, `actor` (`local-user`), `entity_type`, `entity_id`, `detail`
(JSON, no secrets). Actions: `agent.create`, `agent.update`,
`hillclimb.start`, `hillclimb.iteration`, `hillclimb.grade`,
`hillclimb.stop`, `hillclimb.cancel`, `secret.access_failure`.

Retention is **not** enforced in software. Treat the files as CUI and
apply the mission clock.

## What “aligned” means here

Engineering choices in this repo try not to *create* IL5-disqualifying
habits (secrets in git, fake ATO, High-only language, invented counts,
custom crypto, phone-home). That is not an assessment. A 3PAO / DISA
reviewer would bounce this as a CSO package — correctly — because
Desk is a local tool, not an IL5 cloud service offering.
