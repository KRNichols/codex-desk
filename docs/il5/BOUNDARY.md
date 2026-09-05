# Codex Desk boundary (prep, not an SSP)

This is a **local operator shell**, not a Cloud Service Offering.
It is not an ATO package. Use it for GRC / 3PAO *prep* of this binary.

```
Operator workstation
  Codex Desk (UI + encrypted store + identity gate)
       |
       | spawn only local `codex` / `codex.exe` / `codex.cmd`
       v
  Codex CLI (operator-installed)
       |
       | TLS to operator-configured HTTPS base_url (Desk refuses cleartext)
       v
  Azure-hosted model (AO / tenant / PA — out of Desk boundary)
```

**In Desk boundary:** chat/agent UI, AES-256-GCM store, DEK custody, hash-chained
audit, PAT slot (OS/env only), identity attestation, `codex exec` argv, hill-climb
loop, local-Codex allowlist.

**Out of Desk boundary:** Azure tenant, DISA PA, CAC/PIV issuance, CMVP modules,
SIEM, BCAP/SCCA, workstation hardening, mission ATO.

Desk does not open Azure sockets and does not phone home.
