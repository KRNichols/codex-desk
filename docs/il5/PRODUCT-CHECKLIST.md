# Codex Desk IL5 product checklist

Hill-climb target for **Desk Improver** and **IL5 Architecture Grader**.
Parse the `il5-rows` fence. Do not invent official control counts.

**IL5** = FedRAMP High + DoD overlays + architecture constraints. High alone fails.

**READY** (this file / grader report) = product-owned rows are `PASS` and this
local operator shell is ready for a **human GRC / 3PAO prep review**.
READY is **never** an ATO, FedRAMP authorization, or DISA PA. The AO authorizes.

Rubric: `docs/il5/FEDRAMP-HIGH-IL5-STANDARD.md` +
[KRNichols/IL5-Agent-Protocol](https://github.com/KRNichols/IL5-Agent-Protocol).
Official workbooks beat this snapshot (`SOURCE.txt`).

`HOLD` if any `product` row is not `PASS`, if encryption/audit/secrets/TLS/egress
are weakened, if High-only is claimed as IL5, or if the worker claims authorization.

AO / tenant / Azure / FIPS-CMVP rows may stay `MISSING`. That does not block
product READY.

```il5-rows
product|encrypted-store|PASS|src-tauri/src/vault.rs + crypto.rs AES-256-GCM CDEX1; src-preview/secure-store.ts
product|key-custody|PASS|src-tauri/src/keystore.rs DPAPI/keyring/machine-bound HKDF; preview crypto.ts
product|audit-chain|PASS|src-tauri/src/audit.rs SHA-256 prev||fields; preview secure-store.ts; GET /api/audit/export
product|secret-redaction|PASS|PAT never in SQLite/git; env/env_key/OS slot; audit redact; refuse config.toml keys
product|cleartext-refusal|PASS|boundary.rs + preview policy.ts refuse http/ws and credentialed URLs
product|egress-allowlist|PASS|local codex/codex.exe/codex.cmd only; remote/UNC/URL spawn fail closed
product|identity-gate|PASS|identity.rs HOLDs workspace-write hill-climbs until operator attestation
product|no-phone-home|PASS|No analytics, crash phone-home, or Desk-owned Azure SDK
product|lockfiles|PASS|package-lock.json + src-tauri/Cargo.lock committed
product|sbom-note|PASS|docs/il5/SBOM.md lockfile-derived component list (not a signed provenance attestation)
product|hillclimb-policy|PASS|policy.rs / src-preview/policy.ts HOLD on ATO claims and weakened controls
product|exploit-policy|PASS|AGENTS.md + briefs forbid exploits/PoCs/attack playbooks
product|telemetry|PASS|None from Desk
product|package-prep|PASS|docs/il5/BOUNDARY.md + SECURITY.md + ARCHITECTURE.md (prep package, not an SSP)
ao|categorization|MISSING|AO CIA/IL/NSS memo
ao|four-layer-cso|MISSING|Desk is not a CSO; Azure + workstation packaged separately
ao|fips-cmvp|MISSING|Desk AES is not a CMVP module; inherit OS/Codex/Azure certs
ao|cac-piv|MISSING|Credential Strength D / Windows Hello hardware — enterprise/AO
ao|siem-retention|MISSING|CSSP feed, 12+18 or contract retention, NTP — AO
ao|bcap-scca|MISSING|DoD-connected workstation overlays — AO
ao|scan-program|MISSING|Authenticated OS/web/SAST program — AO (Desk is not a scan platform)
ao|poam-conmon|MISSING|Mission POA&M / ConMon — AO
ao|ssp-crm|MISSING|Full SSP / CRM / DISA package — AO
ao|azure-pa|MISSING|Tenant IL5 / DISA PA for the Azure model path — AO
ao|pat-issuance|MISSING|PAT issuance, rotation, CAC-backed authenticators — AO
```
