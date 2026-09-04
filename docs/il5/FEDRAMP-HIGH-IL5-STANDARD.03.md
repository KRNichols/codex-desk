| J* | CIS and CRM workbook |
| K | FIPS 199 worksheet |
| L | CSO-specific laws and regulations |
| M* | Integrated Inventory Workbook |
| N | Continuous Monitoring Plan |
| O* | POA&M |
| P | Supply Chain Risk Management Plan |
| Q* | Cryptographic Modules Table |

Plus: authorization boundary diagram, data-flow diagrams, network diagrams, interconnection table.

### 23.2 Assessment package

- Security Assessment Plan (SAP) + Rules of Engagement
- Test Case Workbook
- Penetration test report
- Vulnerability scan corpus (scans of record + 60–90 day history)
- Security Assessment Report (SAR)
- SAR Appendix A Risk Exposure Table
- SAR Appendix B High SRTM
- POA&M
- Deviation file
- 3PAO attestation / independence statement

### 23.3 DoD delta package

- DoD Rev 5 SSP Addendum (filled)
- Architecture briefing slides per DISA guide
- Onboarding questionnaire
- eMASS control import (DISA SOP)
- SCCA / BCAP design
- CSSP concept of operations
- Personnel citizenship / screening evidence
- SNAP artifacts once connection starts
- PA request through DCAS sponsor

### 23.4 High Readiness Assessment Report

Optional but useful. 3PAO RAR before full assessment catches missing FIPS modules, missing MFA, missing boundary hygiene, missing scan authentication. High RAR template is specific — do not use the Moderate RAR.

---

## 24. 3PAO assessment mechanics

### 24.1 Who

FedRAMP-recognized 3PAO (A2LA / FedRAMP list). Independence rules include a cooling-off period after consulting (commonly described as two years — confirm current 3PAO Obligations document). Do not use the firm that designed the system as the assessor.

### 24.2 Initial vs annual

- Initial: **all ~410** High controls + all in-scope FedRAMP+ / NSS overlay controls.
- Annual: core set published by FedRAMP (AC-2 family, AU family, CA-8, CM-5/6/7/8, IA-2/4/5, IR-3/4, RA-5, SC-7/8/12/13/28, SI-3/4/7, etc. — use the current Independent Verification page) plus about one-third of the rest.

### 24.3 Methods

Examine, interview, test. High CA-02(02) specialized assessments: the SAP must say which of in-depth monitoring, instrumentation, automated test cases, vuln scanning, malicious-user testing, insider-threat assessment, performance/load, data-leakage apply.

### 24.4 Sampling of people-process controls

Account request / termination / transfer samples, change tickets, training records, visitor logs, media sanitization certificates. 3PAO picks the sample; you produce the population.

### 24.5 What “ready for 3PAO” means

- Boundary diagrams match reality (they will nmap)
- Inventory = scan targets = running systems
- Authenticated scan coverage ≥ 90%
- No unexplained CAT I / Critical
- Appendix Q complete with certificate numbers
- CRM internally consistent with Appendix A
- Pentest ROE drafted
- Policies dated and trained
- FIPS mode actually enabled, not just licensed

---

## 25. Authorization sequence: FedRAMP then DISA PA then ATO

```
[0] AO categorization memo (IL + NSS + overlays)
[1] Architect IL5 constraints (region, dedicated hardware, citizens, BCAP, FIPS 140-3, STIGs)
[2] Implement FedRAMP High / Class D + write SSP Appendix A
[3] Internal rehearsal scans + rehearsal pentest (all 6 vectors)
[4] High RAR (optional but recommended)
[5] 3PAO SAP → test → SAR
[6] Agency ATO and/or JAB P-ATO → Marketplace
[7] Fill DoD SSP Addendum + Table D-1 DSPAVs
[8] If NSS: apply CNSSI 1253 “+” and overlays; delta test
[9] DoD sponsor via DCAS → DISA kickoff + architecture briefing
[10] Cloud eMASS + DISA SCA review
[11] DISA AO issues IL5 PA (or IATT for test)
[12] Mission owner RMF ATO under DoDI 8510.01
[13] SNAP C-ITP + CATC + CPTC + SCCA BCAP up + CSSP live
[14] ConMon forever
```

Two paths to a DoD PA (public.cyber.mil/dccs):

- Leverage an existing FedRAMP authorization and assess the delta; or
- DoD component sponsors the CSO for a PA without waiting on Marketplace listing — you still must **demonstrate FedRAMP High control compliance** during the IL5 audit.

A FedRAMP High P-ATO does not skip steps 7–14.

---

## 26. FedRAMP 2026 / 20x transition dates

Live dates from FedRAMP timeline pages as of compilation. Recheck [fedramp.gov/2026/timeline](https://fedramp.gov/2026/timeline/).

| Date | Event |
|---|---|
| 4 Jul 2026 | CR26 optional / early adoption; 20x new apps follow CR26 |
| 6 Jul 2026 | Marketplace listings for Initial Implementation |
| 28 Jul 2026 | **FedRAMP Ready goes Legacy** — no new Ready submissions |
| 17 Nov 2026 | Ready holders convert to a Certification by the later of this date or their next annual-assessment expiry |
| 7 Dec 2026 | Track BOD 26-04 / VDR-VER alignment date on the live FedRAMP VDR page |
| 3 Aug 2026 | 20x Class A pipeline opens |
| 10 Aug 2026 | Temporary Rev5 Class B/C Ready Conversion / Lost Sponsor pipelines |
| 31 Aug 2026 | 20x Class B and Class C pipelines open |
| **1 Jan 2027** | CR26 mandatory for Rev5 stakeholders; Rev5 certs adopt at next independent assessment after this date |
| Late 2026 / early 2027 | 20x Class D pilot → formal option (High equivalent on 20x) |
| **11 Jun 2027** | **No new Rev5 certification applications** |
| 1 Feb 2028 | Remaining CR26 grace periods expire; non-compliant listings lose certification |
| 31 Dec 2027 | Legacy Ready status fully retired |
| 31 Dec 2028 | Existing Rev5 certifications targeted to remain at least until this date unless directed otherwise; CR26 practices expire no later than this date |

**Directional cost and time (industry, not official):** FedRAMP High commonly 12–24 months and mid-six to seven figures for 3PAO + documentation + engineering, depending on starting posture. IL5 adds dedicated federal-community infrastructure, US-person staffing, DISA cycle time, and BCAP circuit cost. Budget the IL5 architecture in year zero; do not treat it as a paperwork delta after High.

Engineering implication: write OSCAL-capable, machine-readable evidence **now**. A High SSP that can only exist as a 400-page Word file will be expensive to migrate.

RFC-0020 “Certified Level 5” language, if you see it, is FedRAMP package-depth / High — not DoD IL5.

---

## 27. Related regimes that are not IL5 (CMMC, 800-171, ITAR, CJIS)

Do not substitute these for IL5. They collide with it.

| Regime | What it is | Relationship |
|---|---|---|
| NIST SP 800-171 / CMMC Level 2 | CUI on **nonfederal** contractor systems | Contractor enclave path. Not a cloud PA. Significant-change rules differ. You can be CMMC L2 and still be unable to host DoD missions at IL5 |
| CMMC Level 3 / 800-172 | Higher CUI against APTs | Still not DISA IL5 |
| DFARS 252.204-7012 / 7019 / 7020 / 7021 | CUI clauses + CMMC | Appear in the contract that **uses** your CSO; they do not authorize the CSO |
| ITAR / EAR | Export control | Often the reason an AO picks IL5; adds US-person and US-soil constraints you already have |
| CJIS | Criminal justice data | Separate policy; some Gov clouds carry both |
| IRS 1075 | FTI | Separate overlay |
| HIPAA | PHI | Privacy overlay + BAAs; not IL5 |
| FedRAMP Moderate | Civilian CUI-ish | Not an IL5 floor |
| StateRAMP / GovRAMP | State/local | Irrelevant to DISA PA |
| SOC 2 | Commercial | Useful evidence, not a control baseline |

If you only need to **handle CUI as a defense contractor** on your own corporate systems, CMMC L2 + 800-171 is the path. If you need to **sell a cloud offering that DoD missions run on**, this IL5 file is the path.

---

## 28. Common failure modes

1. Building in commercial multi-tenant regions and planning to “encrypt harder” later.
2. Shared commercial management plane.
3. Offshore privileged support.
4. Software TOTP sold as IL5 MFA.
5. “FIPS-compliant” OpenSSL with no CMVP certificate.
6. FIPS 140-2 modules still in the design after 21 Sep 2026.
7. Unauthenticated monthly scans.
8. Inventory that does not match DNS / IPs / images the 3PAO discovers.
9. Bundled POA&M rows.
10. No tenant-to-tenant pentest.
11. STIG once, never again.
12. Assuming AWS/Azure/GCP IL5 PA covers *your* SaaS automatically.
13. Blank organization-defined parameters.
14. CRM that says “inherited” for controls the platform PA does not include.
15. Direct-to-internet admin paths.
16. Missing Appendix Q certificate numbers.
17. IRP clocks copied from a Moderate package.
18. Treating FedRAMP High Marketplace listing as a DISA PA.
19. Skipping the written NSS / IL decision and discovering at kickoff you owe 170 more controls.
20. Trusting a blog that said “10 extra controls” or “47 extra controls” instead of the current SSP Addendum.

---

## 29. Build order and staffing checklist

### 29.1 Order

1. Written AO categorization (IL, NSS, overlays, information types).
2. Boundary + data-flow + tenancy architecture that already satisfies §8–§12.
3. US federal-community region + dedicated hosts + HSM CMK + no commercial neighbors.
4. Identity: CAC/PIV or hardware token, US-person privileged roles, no shared admins.
5. FIPS 140-3 modules in FIPS mode everywhere crypto exists; fill Appendix Q as you go.
6. STIG baselines in the image pipeline; CAT I fails the build.
7. Authenticated scan pipeline: OS, web, DB, container, SCAP → inventory → POA&M.
8. SAST / SBOM / signing (SA-11, CM-14, SI-07(15)).
9. Logging + time sync + SIEM that CSSP can use.
10. Policies, SSP Appendix A, CRM, SCRM, CP, IRP using official templates.
11. Internal six-vector pentest.
12. 3PAO High RAR then full SAR.
13. FedRAMP authorization.
14. SSP Addendum + DISA architecture briefing + eMASS.
15. PA → mission ATO → SNAP → BCAP → ConMon.

### 29.2 Roles you actually need

- System owner / ISSO
- Boundary architect who has shipped an IL5 or Gov-cloud dedicated-host design
- Crypto / KMS owner
- Identity owner (PKI)
- Platform hardening (STIG) owner
- Scan + inventory automation owner
- Application security (SAST/DAST) owner
- GRC package owner who lives in Appendix A
- IR lead who can hit 15-minute N5 clocks
- Personnel security for citizenship / screening
- Sponsor liaison (DoD component) for DCAS
- 3PAO contract that forbids them from also consulting

### 29.3 Evidence you should be producing every week from month one

- Inventory diff
- Scan raw files
- CAT I / Critical aging
- FIPS module list
- Privileged-user roster with citizenship attestation
- Change tickets
- Pipeline pass/fail on STIG + SAST

If that weekly pack does not exist, you are not “almost ready for 3PAO.”

---

## 30. Contacts, portals, and mailboxes

Verify before use; mailboxes move.

| Function | Where |
|---|---|
| FedRAMP PMO | info@fedramp.gov — https://www.fedramp.gov/ |
| FedRAMP incident / security | fedramp_security@fedramp.gov |
| FedRAMP Marketplace | https://marketplace.fedramp.gov/ |
| DCCS public | https://public.cyber.mil/dccs/ |
| DCCS library | https://public.cyber.mil/dccs/dccs-documents/ |
| DISA Cloud Assessments | disa.meade.re.mbx.cloud-team@mail.mil |
| Cloud eMASS | disa.meade.re.mbx.disa-cloud-emass-team@mail.mil — https://cloud.emass.apps.mil/ (CAC/ECA) |
| DCAS sponsor portal | https://dod365.sharepoint-mil.us/sites/DISA-RE-Apps/cas (CAC) |
| SNAP | https://snap.dod.mil/ (CAC) |
| Connection Approval Office | disa.meade.re.mbx.ucao@mail.mil |
| SCCA PMO (BCAP) | hac-scca-pmo@mail.mil / disa.meade.se.mbx.disa-scca-pmo@mail.mil |
| DoD NIC | disa.columbus.ns.list.hostmaster-dod-nic-dl@mail.mil |
| PPSM | dod.ppsm@mail.mil |
| PKI/PKE | dodpke@mail.mil — https://public.cyber.mil/pki-pke/interoperability/ |
| CSSP listing | IntelShare CAC site (see DCCS Help) |
| CNSS issuances | https://www.cnss.gov/CNSS/issuances/Instructions.cfm |
| NIST CSRC | https://csrc.nist.gov/ |
| CMVP | https://csrc.nist.gov/projects/cryptographic-module-validation-program |
| CISA incident notifications | CISA Federal Incident Notification Guidelines |
| DoD CUI contractor incidents | DIBNet (DFARS 252.204-7012, 72 hours) |
| STIG downloads | https://public.cyber.mil/stigs/ |

---

## 31. Revision and verification notes

This file is a navigation layer over official publications. It is not a DISA PA, not a FedRAMP baseline, and not legal advice.

**Before you freeze a baseline for assessment:**

1. Re-download SSP Appendix A High and the DoD Rev 5 SSP Addendum. Diff against this file.
2. Re-read CSP SRG Appendix D Table D-1 for current DSPAVs.
3. Confirm NSS / overlay applicability with the AO in writing.
4. Confirm FIPS module certificates are still **active** on CMVP (not historical, not revoked).
5. Confirm FedRAMP incident clocks on the live CR26 Incident Evaluation page — they have moved during 2026 RFCs.
6. Confirm BCAP sites and SNAP/DCAS URLs with SCCA / RE2.
7. Confirm 20x Class D status if you are starting after early 2027.

### Suggested next artifacts to generate from this file

- Control workbook: Appendix A High ⋈ SSP Addendum ⋈ owner ⋈ evidence path ⋈ scan hook
- Inventory + scan coverage matrix
- Isolation design narrative (compute, storage, network, management plane, keys)
- Six-vector pentest ROE draft
- ConMon calendar with named producers and repository paths
- Architecture briefing slide outline matching DISA’s preparation guide

---

*End of guide.*
