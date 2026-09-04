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
| **Codex Desk** | Local UI, encrypted local store, Codex process spawn, hill-climb loop, hash-chained audit, secret non-storage, identity gate |

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
| Categorization (§3) | Docs assume potential CUI; no AO memo in-app | **MISSING** (AO) | Mission AO writes CIA / IL / NSS memo |
| Four-layer stack (§4) | Desk is not a CSO; docs refuse High-only-as-IL5 | **N/A / MISSING** (AO/Azure) | Package Azure + workstation separately |
| Secrets / authenticators (§9.4, IA-05) | PAT never in git or SQLite. Read from process env / Codex `env_key` / optional OS secret slot (`keystore.rs`, `src-preview/crypto.ts`). Setup refuses PAT-in-store, PAT-in-`config.toml`, and endpoint query tokens. Logs redact token-like lines | **PARTIAL** | Mission must still prove PAT issuance/rotation and CAC-backed authenticators. Windows Credential Manager / DPAPI is the IL5-relevant slot; Linux uses machine-bound wrap when Secret Service is absent |
| Data at rest SC-28 (§9.2) | Working AES-256-GCM envelope (`CDEX1`) over the SQLite/JSON store. DEK is random 256-bit; custody is Windows DPAPI + Credential Manager, else OS keyring, else machine-bound HKDF wrap (`vault.rs`, `keystore.rs`, `src-preview/secure-store.ts`). Legacy plaintext files are migrated and deleted. Key never in repo | **PARTIAL** | Desk ships working encryption, not a CMVP module. Mission still collects FIPS 140-3 evidence for the OS crypto library used at runtime |
| Data in transit SC-08 | Desk does not open Azure sockets. Setup and `run_turn` **refuse** `http://` / `ws://` endpoints and credentialed URLs (`boundary.rs`, `codex.rs`) | **PARTIAL** (inherited + Desk refusal) | Confirm Codex + OS FIPS mode; list CMVP certs in a real Appendix Q |
| FIPS 140-3 (§9) | **What Desk verifies:** envelope algorithm (AES-256-GCM), TLS URL shape, no custom TLS stack, no cleartext spawn. **What Desk does not have:** a CMVP certificate of its own | **MISSING** (evidence) | Inherit validated modules from Windows CNG / OpenSSL FIPS / Codex / Azure. Record cert numbers. “AES-256” alone is not enough |
| Identity / CAC/PIV (§10) | First-class IL5 identity gate: machine-bound unlock + operator attestation that **HOLDs workspace-write hill-climbs** until configured (`identity.rs`, Identity gate UI). Bind is OS user session (Windows `USERNAME`/`USERPROFILE`; POSIX user). Not silent local-anyone for writes | **PARTIAL** | CAC/PIV at Credential Strength D and Windows Hello hardware prompt remain AO/enterprise work. Do not add a password fallback that drops below Strength D |
| Audit AU family / §19 | Append-only **hash-chained** events (SHA-256 of prev\|\|canonical fields) inside the encrypted store (`audit.rs`). Actions include agent create, hill-climb start/grade/cancel, secret access failure, encryption key unlock failure (unlock failures also go to a no-secret `unlock-failures.jsonl` because the store is sealed). No secret values | **PARTIAL** | Central SIEM/CSSP feed, 12+18 or contract retention, NTP-backed timestamps remain AO |
| Retention | Operator keeps app-data; no automated purge | **MISSING** (AO) | Configurable retention + signed export |
| Network boundary (§8, §11) | No phone-home. No analytics. Runner allowlist is **local Codex binary only**; remote/`http`/`\\` paths fail closed (`boundary.rs`) | **PARTIAL** | Mission BCAP/SCCA if the workstation is DoD-connected |
| Tenancy / US location (§8) | Local process. No Desk cloud | **N/A** | AO owns machine and Azure tenant location |
| Scan program (§14) | Not a scan platform. Deterministic IL5 grader HOLDs ATO claims and weakened product controls (`policy.rs`) | **MISSING** (AO) | Authenticated OS/web/SAST/secrets program |
| POA&M / ConMon (§16–17) | Residual risks listed here | **MISSING** (AO) | Mission POA&M |
| Package artifacts (§23) | SECURITY.md + ARCHITECTURE.md | **MISSING** (AO) | SSP / CRM / boundary diagrams |
| Supply chain | Lockfiles committed when present | **PARTIAL** | SBOM, provenance attestations |
| Telemetry | None from Desk | **PASS** | Keep it that way |
| Hill-climb / self-improve | Worker + grader briefs + **local policy HOLD**. Seeded goal: “Close IL5 MISSING items in SECURITY.md for product-owned rows.” | **PARTIAL** | Human review of every write; no unattended push |
| Exploits / pentest (§15) | Agents forbidden to write exploits/PoCs | **PASS** (policy) | Mission pentest is a separate 3PAO activity |
| CMMC / 800-171 as IL5 (§27) | Docs refuse the mix-up | **PASS** (policy) | Keep COCO vs CSP paths separate |

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
2. **No CAC/PIV.** Write hill-climbs require operator attestation; that is not Strength D.
3. **Linux key custody** falls back to machine-bound wrap when Secret Service / Credential Manager is absent (PARTIAL vs Windows DPAPI).
4. **No ConMon / scan program** for Desk as a CSO (it is not one).
5. **Hill-climb with workspace-write** can edit the path the operator selected after attestation. Desk does not auto-commit or push.

## Audit events

Written to `audit_events` inside the encrypted store. Fields: `at`,
`action`, `actor` (`local-user:<session>`), `entity_type`,
`entity_id`, `detail` (no secrets), `prev_hash`, `event_hash`.
Actions: `agent.create`, `agent.update`, `hillclimb.start`,
`hillclimb.iteration`, `hillclimb.grade`, `hillclimb.stop`,
`hillclimb.cancel`, `secret.access_failure`,
`encryption.key_unlock_failure`, `identity.attest`,
`secret.slot_write`, `secret.slot_clear`.

Plaintext `audit.jsonl` is no longer the system of record.

## What “aligned” means here

Engineering choices try not to *create* IL5-disqualifying habits
(secrets in git, fake ATO, High-only language, invented counts,
phone-home, silent local-anyone writes, plaintext CUI store).
That is not an assessment. A 3PAO / DISA reviewer would bounce
this as a CSO package — correctly — because Desk is a local tool,
not an IL5 cloud service offering.
