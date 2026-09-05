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
| **Codex Desk** | Local UI, encrypted local store, Codex process spawn, hill-climb loop, hash-chained audit, secret non-storage, machine-bound identity |

## Data assumption

Chat transcripts, agent briefs, hill-climb logs, and workspace paths
are treated as **potentially CUI-bearing**. Backups of the app data
directory are in-scope for the operator (standard §8.5).

## What READY means (product bar)

**READY** for Codex Desk = every **product-owned** row in
`docs/il5/PRODUCT-CHECKLIST.md` is `PASS`, and this local operator
shell is prep-ready for a **human GRC / 3PAO look**. READY is
**never** an ATO, FedRAMP authorization, or DISA PA.

Hill-climb graders `HOLD` if any `product` row is not `PASS`, if
encryption / hash-chained audit / secret non-storage / TLS refusal /
local-Codex-only egress is weakened, or if text claims authorization.

AO / tenant / Azure / FIPS-CMVP rows stay **MISSING**. That does not
block product READY. Official workbooks beat this file.

Machine-readable rows: `docs/il5/PRODUCT-CHECKLIST.md` (`il5-rows`).
Prep package (not an SSP): `docs/il5/BOUNDARY.md`, `docs/il5/SBOM.md`.

## Mapping (theme → Desk → grade)

Grades are `PASS` / `PARTIAL` / `MISSING` for **this slice**, not
control-by-control 800-53 scores. Official IDs are cited as themes
only.

### Product-owned (Desk ships working code)

| Theme (standard) | How Desk addresses it | Grade | Next step if not PASS |
|---|---|---|---|
| Secrets / authenticators — Desk half (§9.4, IA-05) | PAT never in git or SQLite. Read from process env / Codex `env_key` / optional OS secret slot (`keystore.rs`, `src-preview/crypto.ts`). Setup refuses PAT-in-store, PAT-in-`config.toml`, and endpoint query tokens. Logs redact token-like lines | **PASS** (product) | AO still owns PAT issuance/rotation and CAC-backed authenticators (row below) |
| Data at rest SC-28 — Desk envelope (§9.2) | Working AES-256-GCM envelope (`CDEX1`) over the SQLite/JSON store. DEK is random 256-bit; custody is Windows DPAPI + Credential Manager, else OS keyring, else machine-bound HKDF wrap (`vault.rs`, `keystore.rs`, `src-preview/secure-store.ts`). Legacy plaintext files are migrated and deleted. Key never in repo | **PASS** (product) | CMVP evidence is AO/inherited (FIPS row below). Desk AES is not a FIPS module |
| Data in transit — Desk refusal | Desk does not open Azure sockets. Setup and `run_turn` **refuse** `http://` / `ws://` endpoints and credentialed URLs (`boundary.rs`, `codex.rs`) | **PASS** (product) | Codex + OS FIPS / CMVP certs stay AO |
| Identity bind (not CAC/PIV) | Machine-bound unlock + optional operator record (`identity.rs`). **YOLO is always-on**: no in-app permission chrome, no identity-gate write HOLD, no Allow-workspace-writes checkbox. Bind is OS user session | **PASS** (product) | CAC/PIV Strength D remains AO |
| Audit AU — Desk half | Append-only **hash-chained** events (SHA-256 of prev\|\|canonical fields) inside the encrypted store (`audit.rs`). Operator **export** (`export_audit` / `GET /api/audit/export`) — no auto-purge. No secret values | **PASS** (product) | SIEM/CSSP, contract retention, NTP remain AO |
| Network boundary — Desk half | No phone-home. No analytics. Runner allowlist is **local Codex binary only**; remote/`http`/`\\` paths fail closed (`boundary.rs`) | **PASS** (product) | BCAP/SCCA if the workstation is DoD-connected — AO |
| Supply chain — Desk half | Lockfiles committed. Lockfile-derived note in `docs/il5/SBOM.md` | **PASS** (product) | Signed provenance / vendor attestations — AO |
| Telemetry | None from Desk | **PASS** | Keep it that way |
| Hill-climb / self-improve | Worker + grader briefs + **local policy HOLD** + PRODUCT-CHECKLIST parse. Seeded goal closes product rows. Operator contract: `briefs/OPERATOR.md` | **PASS** (product) | Human review of every write; no unattended push |
| Exploits / pentest (§15) | Agents forbidden to write exploits/PoCs | **PASS** (policy) | Mission pentest is a separate 3PAO activity |
| CMMC / 800-171 as IL5 (§27) | Docs refuse the mix-up | **PASS** (policy) | Keep COCO vs CSP paths separate |
| Package prep | `SECURITY.md` + `ARCHITECTURE.md` + `docs/il5/BOUNDARY.md` | **PASS** (prep) | Full SSP / CRM stays AO |

### AO / inherited / Azure (stay MISSING)

| Theme (standard) | How Desk addresses it | Grade | Next step |
|---|---|---|---|
| Categorization (§3) | Docs assume potential CUI; no AO memo in-app | **MISSING** (AO) | Mission AO writes CIA / IL / NSS memo |
| Four-layer stack (§4) | Desk is not a CSO; docs refuse High-only-as-IL5 | **MISSING** (AO/Azure) | Package Azure + workstation separately |
| FIPS 140-3 (§9) | Desk verifies envelope algorithm and TLS URL shape. Desk does **not** have a CMVP certificate | **MISSING** (evidence) | Inherit Windows CNG / OpenSSL FIPS / Codex / Azure certs |
| CAC/PIV (§10) | Identity is a session bind, not Strength D. YOLO writes are not CAC-gated | **MISSING** (AO) | Enterprise CAC/PIV / Windows Hello hardware |
| Retention / SIEM | Export exists; no CSSP feed | **MISSING** (AO) | 12+18 or contract retention + SIEM |
| BCAP / SCCA | Local process only | **MISSING** (AO) | If DoD-connected |
| Scan program (§14) | Not a scan platform. Deterministic HOLD on ATO claims | **MISSING** (AO) | Authenticated OS/web/SAST/secrets program |
| POA&M / ConMon (§16–17) | Residual risks listed here | **MISSING** (AO) | Mission POA&M |
| SSP / CRM (§23) | Prep docs only | **MISSING** (AO) | Full SSP / CRM / DISA package |
| Azure / DISA PA | Desk never opens the Azure socket | **MISSING** (AO) | Tenant IL5 + DISA PA for that CSO |
| PAT issuance | Desk refuses to store the PAT | **MISSING** (AO) | Issuance, rotation, CAC-backed authenticators |

## FIPS 140-3 inheritance path

Desk does **not** ship a validated cryptographic module.

| Layer | Inheritance | Desk verifies | Mission still collects |
|---|---|---|---|
| OS | Windows CNG / DPAPI; Linux kernel + userland TLS/crypto | DPAPI wrap on Windows; file mode `0600`/`0700`; machine binding | CMVP certs for the OS crypto in use; FIPS mode enabled |
| Codex CLI | Platform TLS to the configured Azure `https` base_url | Refuses cleartext and credentialed URLs; spawns local `codex` only | Codex build + FIPS-capable TLS library evidence |
| Azure | Tenant TLS / PA path | Nothing inside Azure. Desk never opens the socket | FedRAMP High + DoD IL5 overlays + DISA PA for that CSO |
| Desk envelope | AES-256-GCM (`crypto.rs` / `src-preview/crypto.ts`) | Working seal/open, CDEX1 magic, DEK not in repo | This application crypto is **not** a FIPS module |

## Residual risks (do not paper over)

1. **Desk AES-256-GCM is not a CMVP module.** Encryption works; FIPS evidence is still MISSING.
2. **No CAC/PIV.** Identity is a posix/windows user-session bind, not Strength D. YOLO writes do not wait on attestation.
3. **Linux key custody** falls back to machine-bound wrap when Secret Service / Credential Manager is absent (PARTIAL vs Windows DPAPI).
4. **No ConMon / scan program** for Desk as a CSO (it is not one).
5. **Hill-climb YOLO workspace-write** can edit the path the operator set. Desk does not auto-commit or push. No in-app Allow-writes checkbox.

## Audit events

Written to `audit_events` inside the encrypted store. Fields: `at`,
`action`, `actor` (`local-user:<session>`), `entity_type`,
`entity_id`, `detail` (no secrets), `prev_hash`, `event_hash`.
Actions: `agent.create`, `agent.update`, `hillclimb.start`,
`hillclimb.iteration`, `hillclimb.grade`, `hillclimb.stop`,
`hillclimb.cancel`, `secret.access_failure`,
`encryption.key_unlock_failure`, `identity.attest`,
`secret.slot_write`, `secret.slot_clear`, `audit.export`.

Plaintext `audit.jsonl` is no longer the system of record.

## What “aligned” means here

Engineering choices try not to *create* IL5-disqualifying habits
(secrets in git, fake ATO, High-only language, invented counts,
phone-home, plaintext CUI store). YOLO writes are a product choice,
not an in-app permission system.
That is not an assessment. A 3PAO / DISA reviewer would bounce
this as a CSO package — correctly — because Desk is a local tool,
not an IL5 cloud service offering.
