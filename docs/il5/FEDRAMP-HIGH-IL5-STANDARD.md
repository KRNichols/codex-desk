# FedRAMP High / DoD Impact Level 5
## Complete Build-Against, Scan, and Certification Guide

**Document status:** Working implementation guide compiled 4 September 2026  
**Audience:** Cloud service provider (CSP) engineering, GRC, 3PAO-facing package owners, and DoD mission sponsors  
**Purpose:** One file that names every standard, scan, overlay, artifact, clock, and authorization step required so a High / IL5 package is not missing a hidden requirement at assessment time.

> **Read this first.** There is no official “FedRAMP Impact Level 5.” FedRAMP uses Low / Moderate / High, and in 2026 Certification Classes A–D (Class D = High). “Impact Level 5 (IL5)” is a Department of Defense Cloud Computing Security Requirements Guide (CC SRG / CSP SRG) designation administered by DISA. IL5 is **FedRAMP High plus DoD overlays plus architecture constraints**. Building only to FedRAMP High fails an IL5 assessment. Building to IL5 from day one includes FedRAMP High.

**Two buyer paths. Do not mix them.**

| You are… | You need… | You do not need… |
|---|---|---|
| A **cloud service provider** selling a CSO that DoD missions will run on | FedRAMP High / Class D + DISA IL5 PA + mission ATO + BCAP | CMMC as a substitute for the PA |
| A **defense contractor** holding CUI on *your own* enclave (COCO) | DFARS 252.204-7012, NIST SP 800-171, CMMC Level 2 (or 3), SPRS, 72-hour DIBNet reporting | A DISA IL5 PA. IL5 is not available for contractor-owned systems |
| Handling **ITAR / USML** technical data | US-person ops + typically GovCloud / Azure Government / GCC-High style tenancy | Assuming IL5 automatically equals ITAR authorization (DDTC / 22 CFR 120–130 is separate) |

FedRAMP RFC-0020 “Certified Level 5” is High package-depth language. It is not DoD IL5.

**Do not treat blog control counts as the baseline.** Download the current official workbooks listed in §2 and implement those. Counts in this file are directional so you can staff and budget. Official counts live in:

- FedRAMP High / Class D: SSP Appendix A High + FedRAMP Security Controls Baseline (Excel) + [fedramp.gov/2026/reference/fedramp-certification](https://fedramp.gov/2026/reference/fedramp-certification/)
- DoD overlay: *DoD Rev 5 SSP Addendum Controls v1.2* and *DoD SRG Control Crosswalk* on [public.cyber.mil/dccs/dccs-documents](https://public.cyber.mil/dccs/dccs-documents/)
- NSS overlay: CNSSI 1253 (29 July 2022 and later attachments) on [cnss.gov](https://www.cnss.gov/CNSS/issuances/Instructions.cfm)

---

## Table of contents

1. [Terminology and decision tree](#1-terminology-and-decision-tree)
2. [Official source library (download these)](#2-official-source-library-download-these)
3. [Categorization — FIPS 199, NSS, CUI](#3-categorization--fips-199-nss-cui)
4. [The four-layer control stack](#4-the-four-layer-control-stack)
5. [FedRAMP High / Class D control families](#5-fedramp-high--class-d-control-families)
6. [DoD FedRAMP+ / Table D-1 / SSP Addendum](#6-dod-fedramp--table-d-1--ssp-addendum)
7. [CNSSI 1253 NSS overlays](#7-cnssi-1253-nss-overlays)
8. [Non-control IL5 architecture requirements](#8-non-control-il5-architecture-requirements)
9. [Cryptography (FIPS 140-3 sunset)](#9-cryptography-fips-140-3-sunset)
10. [Identity, CAC/PIV, Credential Strength D](#10-identity-cacpiv-credential-strength-d)
11. [Network: BCAP, SCCA, NIPRNet, SNAP](#11-network-bcap-scca-niprnet-snap)
12. [Personnel, citizenship, screening](#12-personnel-citizenship-screening)
13. [STIG / SRG / SCAP / ACAS hardening](#13-stig--srg--scap--acas-hardening)
14. [Scan program — types, cadence, evidence](#14-scan-program--types-cadence-evidence)
15. [Penetration test and red team](#15-penetration-test-and-red-team)
16. [POA&M, deviations, remediation clocks](#16-poam-deviations-remediation-clocks)
17. [Continuous monitoring calendar](#17-continuous-monitoring-calendar)
18. [Incident reporting clocks](#18-incident-reporting-clocks)
19. [Logging (M-21-31 and successors)](#19-logging-m-21-31-and-successors)
20. [Significant change process](#20-significant-change-process)
21. [IaaS / PaaS / SaaS shared responsibility](#21-iaas--paas--saas-shared-responsibility)
22. [Inheritance, CRM, GovCloud / Azure Government](#22-inheritance-crm-govcloud--azure-government)
23. [Package artifacts and SSP appendices](#23-package-artifacts-and-ssp-appendices)
24. [3PAO assessment mechanics](#24-3pao-assessment-mechanics)
25. [Authorization sequence: FedRAMP then DISA PA then ATO](#25-authorization-sequence-fedramp-then-disa-pa-then-ato)
26. [FedRAMP 2026 / 20x transition dates](#26-fedramp-2026--20x-transition-dates)
27. [Related regimes that are not IL5 (CMMC, 800-171, ITAR, CJIS)](#27-related-regimes-that-are-not-il5-cmmc-800-171-itar-cjis)
28. [Common failure modes](#28-common-failure-modes)
29. [Build order and staffing checklist](#29-build-order-and-staffing-checklist)
30. [Contacts, portals, and mailboxes](#30-contacts-portals-and-mailboxes)
31. [Revision and verification notes](#31-revision-and-verification-notes)

---

## 1. Terminology and decision tree

### 1.1 What the words actually mean

| Term | Who owns it | What it covers |
|---|---|---|
| FedRAMP Low / Moderate / High | GSA FedRAMP PMO + agency AO or JAB | Civilian federal cloud authorization against NIST SP 800-53 Rev 5 tailored baselines |
| FedRAMP Class A / B / C / D | FedRAMP Consolidated Rules for 2026 | New certification-class language. Class B ≈ Low, Class C ≈ Moderate, Class D ≈ High |
| FedRAMP 20x | FedRAMP PMO | Modernized continuous-validation path (KSIs, machine-readable evidence). Class D 20x is still rolling out |
| RFC-0020 “Certified Level 5” | FedRAMP RFC language | Package-depth label mapping to High. **Not DoD IL5** |
| DoD Impact Level 2 | DISA CC SRG | Public / non-CUI DoD data. FedRAMP Moderate floor |
| DoD Impact Level 4 | DISA CC SRG | CUI / non-critical mission data. FedRAMP Moderate or High + FedRAMP+ |
| **DoD Impact Level 5** | DISA CC SRG | Higher-sensitivity CUI and unclassified NSS / NSI. **FedRAMP High floor** + FedRAMP+ + (if NSS) CNSSI 1253 |
| DoD Impact Level 6 | DISA CC SRG | SECRET. Dedicated classified infrastructure, SIPRNet, not commercial FedRAMP |
| DoD Provisional Authorization (PA) | DISA AO | Reusable authorization that a mission owner can leverage |
| Agency / Mission ATO | Agency or DoD Authorizing Official | System-specific Authority to Operate under RMF (NIST SP 800-37 / DoDI 8510.01) |
| 3PAO | FedRAMP-recognized assessor | Independent assessor for FedRAMP packages; often reused for DISA delta |
| FedRAMP+ | DISA CSP SRG Appendix D | DoD parameter values and extra C/CEs on top of FedRAMP |
| NSS | CNSS / NIST SP 800-59 | National Security System. Triggers CNSSI 1253 overlays |

### 1.2 Decision tree (do this before writing a single control)

```
What data will the CSO process, store, or transmit?
├─ Public / non-CUI DoD          → IL2 (FedRAMP Moderate)
├─ Ordinary CUI, not NSS         → IL4 (FedRAMP Moderate or High + FedRAMP+)
├─ Higher-sensitivity CUI
│     or unclassified NSS/NSI    → IL5 (FedRAMP High + FedRAMP+ + NSS if designated)
└─ SECRET or above               → IL6 (stop; this guide does not apply)

Is the system an NSS under NIST SP 800-59?
├─ No, but AO still wants IL5    → FedRAMP High + IL5 non-NSS FedRAMP+ + SRG architecture
└─ Yes                           → FedRAMP High + IL5 NSS FedRAMP+ + CNSSI 1253 Table D-1 “+” H-H-x
                                    (~600 controls, not ~410)

Who is the customer?
├─ Civilian agency only          → FedRAMP High / Class D may be sufficient
├─ DoD mission on commercial cloud → FedRAMP High THEN DISA IL5 PA THEN mission ATO
└─ Both                          → Build IL5; civilian agencies can reuse the High package
```

Get the categorization and NSS determination **in writing from the sponsoring AO**. That memo is the most expensive page in the program.

### 1.3 Approximate control counts (directional only)

Industry analysis of CSP SRG v1r3 (2 July 2025) and FedRAMP Rev 5 High:

| Stack | FedRAMP | DoD SRG extras | CNSSI 1253 “+” H-H-x | Approx total |
|---|---|---|---|---|
| FedRAMP Moderate | 323 | — | — | 323 |
| FedRAMP High / Class D | 410 | — | — | **410** |
| FedRAMP High + IL4 FedRAMP+ | 410 | ~18 | — | ~428 |
| FedRAMP High + IL5 (non-NSS, older Table 2 language) | 410 | ~10–21 | — | ~420–431 |
| **FedRAMP High + IL5 NSS** | 410 | ~20 | ~170 | **~600** |
| FedRAMP High + IL6 NSS | 410 | ~19 | ~170 + classified overlay ~29 | ~628 |

Microsoft Learn and older CC SRG v1r4 still say Table 2 required **10 additional C/CEs** beyond FedRAMP High for an IL5 PA. Current CSP SRG language for IL5 NSS is stricter and pulls in CNSSI 1253. **Use the SSP Addendum on cyber.mil as the source of truth.**

---

## 2. Official source library (download these)

### 2.1 FedRAMP

| Artifact | Where |
|---|---|
| Class D / High certification rules and control list | https://fedramp.gov/2026/reference/fedramp-certification/ |
| Full Rev 5 control reference with FedRAMP parameters | https://fedramp.gov/2026/reference/controls/ |
| Rev 5 documents and templates | https://www.fedramp.gov/rev5/documents-templates/ |
| Rev 5 playbooks (Getting Started, Authorization, ConMon, Agency) | https://fedramp.gov/docs/rev5/ |
| SSP Appendix A — High FedRAMP Security Controls | Rev5 templates library (Word) |
| FedRAMP High / Moderate / Low / LI-SaaS SSP template | Rev5 templates library |
| SAR Appendix A Risk Exposure Table (RET) | Rev5 templates library (Excel) |
| SAR Appendix B High Security Requirements Traceability Matrix | Rev5 templates library |
| High Readiness Assessment Report (RAR) template | Rev5 templates library |
| POA&M template | Rev5 templates library (Excel) |
| Vulnerability Deviation Request Form | Rev5 templates library (Excel) |
| Continuous Monitoring Playbook (v1.0, 17 Nov 2025) | https://www.fedramp.gov/resources/documents/Continuous_Monitoring_Playbook.pdf |
| Continuous Monitoring Deliverables Template | Rev5 templates library |
| Annual Assessment Controls Selection Worksheet | Rev5 templates library |
| Penetration Test Guidance | https://www.fedramp.gov/resources/documents/CSP_Penetration_Test_Guidance.pdf |
| Vulnerability scanning (legacy playbook page) | https://fedramp.gov/legacy/playbook/csp/continuous-monitoring/vulnerability-scanning/ |
| Vulnerability Detection and Response (CR26) | https://fedramp.gov/2026/reference/rev5/b/vulnerability-detection-and-response/ |
| Incident Evaluation and Communication (CR26) | https://fedramp.gov/2026/reference/incident-evaluation-and-communication/ |
| Significant Change Notifications | https://www.fedramp.gov/docs/significant-change-notifications/ |
| 2026 Consolidated Rules timeline | https://fedramp.gov/2026/timeline/ |
| 3PAO Obligations and Performance Standards | FedRAMP resources/documents |
| FedRAMP Marketplace | https://marketplace.fedramp.gov/ |

### 2.2 DoD / DISA

| Artifact | Where | Library date noted |
|---|---|---|
| DCCS home | https://public.cyber.mil/dccs/ | — |
| DCCS document library | https://public.cyber.mil/dccs/dccs-documents/ | — |
| Cloud Computing SRG (current zip) | DCCS library | 2026-06-26 |
| DoD Rev 5 SSP Addendum Controls v1.2 | DCCS library | 2025-12-03 |
| DoD SRG Control Crosswalk | DCCS library | 2026-04-23 |
| DoD DISA CSP Architecture Briefing Preparation Guide | DCCS library | 2026-04-21 |
| Cloud CSP Onboarding / Questionnaire | DCCS library | 2026-08-11 / 2026-04-27 |
| SOP for CSP Submitting eMASS Controls for Validation | DCCS library | 2026-06-03 |
| DoD Cloud Authorization Process (June 2024) | DCCS library | 2024-06-24 |
| DoD Cloud Authorization Process Diagram | DCCS library | 2025-05-09 |
| Cloud Change / Extension / EOL / IATT forms | DCCS library | 2026-06-04 |
| DISN Connection Process Guide | https://dl.dod.cyber.mil/wp-content/uploads/connect/CPG/ConnProcGuide.html | — |
| STIGs / SRGs | https://public.cyber.mil/stigs/ | quarterly |
| Cloud Computing Mission Owner Network SRG | STIG portal / cyber.trackr.live mirror | — |

### 2.3 NIST / CNSS / crypto / identity

| Artifact | Where |
|---|---|
| NIST SP 800-53 Rev 5.2.0 catalog | https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final |
| NIST SP 800-53B baselines | CSRC |
| NIST SP 800-53A Rev 5.2.0 assessment procedures | CSRC |
| OSCAL catalog (machine-readable) | https://github.com/usnistgov/oscal-content |
| NIST SP 800-37 Rev 2 RMF | CSRC |
| NIST SP 800-59 identifying NSS | CSRC |
| NIST SP 800-60 Vol I/II categorization | CSRC |
| NIST SP 800-115 technical testing | CSRC |
| NIST SP 800-70 National Checklist Program | CSRC |
| NIST SP 800-52 Rev 2 TLS | CSRC |
| NIST SP 800-63 digital identity | CSRC |
| FIPS 199, FIPS 200 | CSRC |
| FIPS 140-3 / CMVP validated modules | https://csrc.nist.gov/projects/cryptographic-module-validation-program |
| CNSSI 1253 + overlays | https://www.cnss.gov/CNSS/issuances/Instructions.cfm |
| CNSSP 32 (NSS ↔ FedRAMP High floor) | CNSS |
| DoDI 8510.01 RMF | https://www.esd.whs.mil/DD/ |
| DoDI 8500.01 Cybersecurity | esd.whs.mil |
| DoDI 8520.03 Identity | esd.whs.mil |
| DoDI 8530.01 CSSP / DTM-24-001 | esd.whs.mil |
| DoDI 5200.48 CUI | esd.whs.mil |
| DoDI 8551.01 PPSM | esd.whs.mil |

---

## 3. Categorization — FIPS 199, NSS, CUI

### 3.1 FIPS 199 (FedRAMP High water mark)

Score Confidentiality, Integrity, and Availability independently as Low / Moderate / High. The system impact level is the **high-water mark**. One High objective makes the whole system High.

FedRAMP High is for severe or catastrophic adverse effect on operations, assets, or individuals.

The **agency customer**, not the vendor, sets the impact level for FedRAMP. For DoD, the mission AO sets the Impact Level under the SRG.

### 3.2 CNSSI 1253 (NSS — CIA chosen independently)

NSS categorization does **not** use the high-water mark the same way. Confidentiality, Integrity, and Availability are selected independently. IL5 typically accommodates NSS/CUI up to moderate confidentiality and moderate integrity in older SRG language (M-M-x). Later CSP SRG NSS language for IL5 uses FedRAMP High + CNSSI 1253 Table D-1 “+” with H-H-x selections. **The AO’s written categorization wins.**

NIST SP 800-59 decides whether the system is an NSS. If it is, CNSSI 1253 and CNSSP 32 apply. CNSSP 32 makes FedRAMP High the floor for unclassified NSS in commercial cloud.

### 3.3 CUI vs NSS vs elevated CUI

- CUI is defined by 32 CFR 2002 and the NARA CUI Registry; DoD implements via DoDI 5200.48.
- Ordinary CUI often lands at IL4.
- CUI the AO judges needs more protection than IL4, plus unclassified NSS/NSI, lands at IL5.
- Export-controlled CUI (ITAR/EAR), certain personnel records, and mission-critical planning data are the usual IL5 drivers.
- Privacy / PII / PHI may add the CNSSI 1253 Privacy Overlay on top of whatever IL you are in.

### 3.4 What you produce in this step

- FIPS 199 worksheet (SSP Appendix K)
- Written AO categorization memo (CIA + IL + NSS yes/no + applicable overlays)
- Information types list from NIST SP 800-60 / CUI Registry
- Data-flow diagram showing every place that data at that categorization exists

---

## 4. The four-layer control stack

```
Layer 4  CNSSI 1253 overlays (NSS, Privacy, ICS, CDS, Classified-if-ever)
Layer 3  DoD CSP SRG FedRAMP+ parameters + extra C/CEs + SRG non-control requirements
Layer 2  FedRAMP High / Class D  (~410 NIST SP 800-53 Rev 5 C/CEs + FedRAMP parameters)
Layer 1  NIST SP 800-53 Rev 5 catalog + 800-53B High baseline + 800-53A test cases
```

You implement from the top of the catalog downward, but you **assess** from Layer 2 up. A FedRAMP High P-ATO does not automatically grant IL4 or IL5. DISA still assesses Layers 3 and 4 and every non-control SRG requirement (location, citizenship, BCAP, STIGs).

---

## 5. FedRAMP High / Class D control families

Source: FedRAMP Consolidated Rules Class D list + SSP Appendix A High. Implement from the official Appendix A. This section is the working checklist of High-weight IDs so engineering can see the shape of the baseline.

Assign **every** organization-defined parameter. Empty assignments fail.

### 5.1 Access Control (AC)

- AC-01 Policy and Procedures
- AC-02 Account Management  
  AC-02(01) Automated System Account Management  
  AC-02(02) Automated Temporary and Emergency Account Management  
  AC-02(03) Disable Accounts  
  AC-02(04) Automated Audit Actions  
  AC-02(05) Inactivity Logout  
  AC-02(07) Privileged User Accounts  
  AC-02(09) Restrictions on Use of Shared and Group Accounts  
  AC-02(11) Usage Conditions  
  AC-02(12) Account Monitoring for Atypical Usage  
  AC-02(13) Disable Accounts for High-risk Individuals
- AC-03 Access Enforcement
- AC-04 Information Flow Enforcement  
  AC-04(04) Flow Control of Encrypted Information  
  AC-04(21) Physical or Logical Separation of Information Flows
- AC-05 Separation of Duties
- AC-06 Least Privilege  
  AC-06(01) Authorize Access to Security Functions  
  AC-06(02) Non-privileged Access for Nonsecurity Functions  
  AC-06(03) Network Access to Privileged Commands  
  AC-06(05) Privileged Accounts  
  AC-06(07) Review of User Privileges  
  AC-06(08) Privilege Levels for Code Execution  
  AC-06(09) Log Use of Privileged Functions  
  AC-06(10) Prohibit Non-privileged Users from Executing Privileged Functions
- AC-07 Unsuccessful Logon Attempts
- AC-08 System Use Notification
- AC-10 Concurrent Session Control
- AC-11 Device Lock + AC-11(01) Pattern-hiding Displays
- AC-12 Session Termination
- AC-14 Permitted Actions Without Identification or Authentication
- AC-17 Remote Access  
  AC-17(01) Monitoring and Control  
  AC-17(02) Protection of Confidentiality and Integrity Using Encryption  
  AC-17(03) Managed Access Control Points  
  AC-17(04) Privileged Commands and Access
- AC-18 Wireless Access  
  AC-18(01) Authentication and Encryption  
  AC-18(03) Disable Wireless Networking  
  AC-18(04) Restrict Configurations by Users  
  AC-18(05) Antennas and Transmission Power Levels
- AC-19 Access Control for Mobile Devices + AC-19(05) Full Device or Container-based Encryption
- AC-20 Use of External Systems  
  AC-20(01) Limits on Authorized Use  
  AC-20(02) Portable Storage Devices — Restricted Use
- AC-21 Information Sharing
- AC-22 Publicly Accessible Content

**IL5 pressure points:** AC-04 flow isolation, AC-06 privileged path, AC-17(03) managed access points (this is where BCAP lands), AC-07 lockout (DoD overrides to 3 tries for privileged).

### 5.2 Awareness and Training (AT)

- AT-01 Policy and Procedures
- AT-02 Literacy Training and Awareness  
  AT-02(02) Insider Threat  
  AT-02(03) Social Engineering and Mining
- AT-03 Role-based Training
- AT-04 Training Records

### 5.3 Audit and Accountability (AU)

- AU-01 Policy and Procedures
- AU-02 Event Logging
- AU-03 Content of Audit Records + AU-03(01) Additional Audit Information
- AU-04 Audit Log Storage Capacity
- AU-05 Response to Audit Logging Process Failures  
  AU-05(01) Storage Capacity Warning  
  AU-05(02) Real-time Alerts
- AU-06 Audit Record Review, Analysis, and Reporting  
  AU-06(01) Automated Process Integration  
  AU-06(03) Correlate Audit Record Repositories  
  AU-06(04) Central Review and Analysis  
  AU-06(05) Integrated Analysis of Audit Records  
  AU-06(06) Correlation with Physical Monitoring  
  AU-06(07) Permitted Actions
- AU-07 Audit Record Reduction and Report Generation + AU-07(01) Automatic Processing
- AU-08 Time Stamps
- AU-09 Protection of Audit Information  
  AU-09(02) Store on Separate Physical Systems or Components  
  AU-09(03) Cryptographic Protection  
  AU-09(04) Access by Subset of Privileged Users
- AU-10 Non-repudiation
- AU-11 Audit Record Retention
- AU-12 Audit Record Generation + AU-12(01) + AU-12(03) Changes by Authorized Individuals

**IL5 pressure points:** centralized correlation, crypto-protected logs on separate components, time sync to authoritative source (see also SC-45), retention aligned to §19.

### 5.4 Assessment, Authorization, and Monitoring (CA)

- CA-01 Policy and Procedures
- CA-02 Control Assessments  
  CA-02(01) Independent Assessors  
  CA-02(02) Specialized Assessments *(Class D / High)*  
  CA-02(03) Leveraging Results from External Organizations
- CA-03 Information Exchange + CA-03(06) Transfer Authorizations
- CA-06 Authorization
- CA-07 Continuous Monitoring  
  CA-07(01) Independent Assessment  
  CA-07(04) Risk Monitoring
- CA-08 Penetration Testing  
  CA-08(01) Independent Penetration Testing Agent or Team  
  **CA-08(02) Red Team Exercises** *(High / Class D — separate from annual pentest)*
- CA-09 Internal System Connections

### 5.5 Configuration Management (CM)

- CM-01 Policy and Procedures
- CM-02 Baseline Configuration  
  CM-02(02) Automation Support for Accuracy and Currency  
  CM-02(03) Retention of Previous Configurations  
  CM-02(07) Configure Systems and Components for High-risk Areas
- CM-03 Configuration Change Control  
  CM-03(01) Automated Documentation, Notification, and Prohibition of Changes  
  CM-03(02) Testing, Validation, and Documentation of Changes  
  CM-03(04) Security and Privacy Representatives  
  CM-03(06) Cryptography Management
- CM-04 Impact Analyses  
  CM-04(01) Separate Test Environments  
  CM-04(02) Verification of Controls
- CM-05 Access Restrictions for Change  
  CM-05(01) Automated Access Enforcement and Audit Records  
  CM-05(05) Privilege Limitation for Production and Operation
- CM-06 Configuration Settings  
  CM-06(01) Automated Management, Application, and Verification  
  CM-06(02) Respond to Unauthorized Changes
- CM-07 Least Functionality  
  CM-07(01) Periodic Review  
  CM-07(02) Prevent Program Execution  
  CM-07(05) Authorized Software — Allow-by-exception
- CM-08 System Component Inventory  
  CM-08(01) Updates During Installation and Removal  
  CM-08(02) Automated Maintenance  
  CM-08(03) Automated Unauthorized Component Detection  
  CM-08(04) Accountability Information
- CM-09 Configuration Management Plan
- CM-10 Software Usage Restrictions
- CM-11 User-installed Software
- CM-12 Information Location + CM-12(01) Automated Tools to Support Information Location
- CM-14 Signed Components

**IL5 pressure points:** CM-06 = STIGs (see §13). CM-07(05) allow-by-exception (DoD DSPAV often required). CM-08 inventory must match every scan target.

### 5.6 Contingency Planning (CP)

- CP-01 Policy and Procedures
- CP-02 Contingency Plan  
  CP-02(02) Capacity Planning  
  CP-02(03) Resume Mission and Business Functions  
  CP-02(05) Continue Mission and Business Functions  
  CP-02(08) Identify Critical Assets
- CP-03 Contingency Training + CP-03(01) Simulated Events
- CP-04 Contingency Plan Testing + CP-04(02) Alternate Processing Site
- CP-06 Alternate Storage Site  
  CP-06(01) Separation from Primary Site  
  CP-06(02) Recovery Time and Recovery Point Objectives  
  CP-06(03) Accessibility
- CP-07 Alternate Processing Site  
  CP-07(01) Separation from Primary Site  
  CP-07(02) Accessibility  
  CP-07(03) Priority of Service  
  CP-07(04) Preparation for Use
- CP-08 Telecommunications Services  
  CP-08(01) Priority of Service Provisions  
  CP-08(02) Single Points of Failure  
  CP-08(03) Separation of Primary and Alternate Providers  
  CP-08(04) Provider Contingency Plan
- CP-09 System Backup  
  CP-09(01) Testing for Reliability and Integrity  
  CP-09(02) Test Restoration Using Sampling  
  CP-09(03) Separate Storage for Critical Information  
  CP-09(05) Transfer to Alternate Storage Site  
  CP-09(08) Cryptographic Protection
- CP-10 System Recovery and Reconstitution  
  CP-10(02) Transaction Recovery  
  CP-10(04) Restore Within Time Period

IL5 also expects crisis-survivable operations (SRG continuity language). Alternate sites must still meet US-location and tenancy rules.

### 5.7 Identification and Authentication (IA)

- IA-01 Policy and Procedures
- IA-02 Identification and Authentication (Organizational Users)  
  IA-02(01) MFA to Privileged Accounts  
  IA-02(02) MFA to Non-privileged Accounts  
  IA-02(05) Individual Authentication with Group Authentication  
  IA-02(06) Access to Accounts — Separate Device  
  IA-02(08) Access to Accounts — Replay Resistant  
  IA-02(12) Acceptance of PIV Credentials
- IA-03 Device Identification and Authentication
- IA-04 Identifier Management + IA-04(04) Identify User Status
- IA-05 Authenticator Management  
  IA-05(01) Password-based Authentication  
  IA-05(02) Public Key-based Authentication  
  IA-05(06) Protection of Authenticators  
  IA-05(07) No Embedded Unencrypted Static Authenticators  
  IA-05(08) Multiple System Accounts  
  IA-05(13) Expiration of Cached Authenticators
- IA-06 Authentication Feedback
- IA-07 Cryptographic Module Authentication
- IA-08 Identification and Authentication (Non-organizational Users)  
  IA-08(01) Acceptance of PIV Credentials from Other Agencies  
  IA-08(02) Acceptance of External Authenticators  
  IA-08(04) Use of Defined Profiles
- IA-11 Re-authentication
- IA-12 Identity Proofing  
  IA-12(02) Identity Evidence  
  IA-12(03) Identity Evidence Validation and Verification  
  IA-12(04) In-person Validation and Verification  
  IA-12(05) Address Confirmation

See §10 for DoDI 8520.03 Strength D (hardware token / PKI). IA-05(01) is a common DSPAV override.

### 5.8 Incident Response (IR)

- IR-01 Policy and Procedures
- IR-02 Incident Response Training  
  IR-02(01) Simulated Events  
  IR-02(02) Automated Training Environments
- IR-03 Incident Response Testing
- IR-04 Incident Handling  
  IR-04(01) Automated Incident Handling Processes  
  IR-04(02) Dynamic Reconfiguration  
  IR-04(04) Information Correlation  
  IR-04(06) Insider Threats  
  IR-04(11) Integrated Incident Response Team
- IR-05 Incident Monitoring + IR-05(01) Automated Tracking, Data Collection, and Analysis
- IR-06 Incident Reporting  
  IR-06(01) Automated Reporting  
  IR-06(03) Supply Chain Coordination
- IR-07 Incident Response Assistance + IR-07(01) Automation Support
- IR-08 Incident Response Plan
- IR-09 Information Spillage Response  
  IR-09(02) Training  
  IR-09(03) Post-spill Operations  
  IR-09(04) Exposure to Unauthorized Personnel

Clocks are in §18. DoD adds CSSP coordination and CJCSM 6510.01 / DFARS 204.73.

### 5.9 Maintenance (MA)

- MA-01 Policy and Procedures
- MA-02 Controlled Maintenance + MA-02(02) Automated Maintenance Activities
- MA-03 Maintenance Tools  
  MA-03(01) Inspect Tools  
  MA-03(02) Inspect Media  
  MA-03(03) Prevent Unauthorized Removal
- MA-04 Nonlocal Maintenance + MA-04(03) Comparable Security and Sanitization
- MA-05 Maintenance Personnel + MA-05(01) Individuals Without Appropriate Access
- MA-06 Timely Maintenance

Nonlocal maintenance of IL5 systems cannot originate from non-US / non-cleared staff over the open internet.

### 5.10 Media Protection (MP)

- MP-01 Policy and Procedures
- MP-02 Media Access
- MP-03 Media Marking
- MP-04 Media Storage
- MP-05 Media Transport
- MP-06 Media Sanitization  
  MP-06(01) Review, Approve, Track, Document, and Verify  
  MP-06(02) Equipment Testing  
  MP-06(03) Nondestructive Techniques
- MP-07 Media Use

CUI marking follows DoDI 5200.48. Sanitization evidence is a favorite 3PAO sample.

### 5.11 Physical and Environmental Protection (PE)

- PE-01 Policy and Procedures
- PE-02 Physical Access Authorizations
- PE-03 Physical Access Control + PE-03(01) System Access
- PE-04 Access Control for Transmission
- PE-05 Access Control for Output Devices
- PE-06 Monitoring Physical Access  
  PE-06(01) Intrusion Alarms and Surveillance Equipment  
  PE-06(04) Monitoring Physical Access to Systems
- PE-08 Visitor Access Records + PE-08(01) Automated Records Maintenance and Review
- PE-09 Power Equipment and Cabling
- PE-10 Emergency Shutoff
- PE-11 Emergency Power + PE-11(01) Alternate Power Supply — Minimal Operational Capability
- PE-12 Emergency Lighting
- PE-13 Fire Protection  
  PE-13(01) Detection Systems — Automatic Activation and Notification  
  PE-13(02) Suppression Systems — Automatic Activation and Notification
- PE-14 Environmental Controls + PE-14(02) Monitoring with Alarms and Notifications
- PE-15 Water Damage Protection + PE-15(01) Automation Support
- PE-16 Delivery and Removal
- PE-17 Alternate Work Site
- PE-18 Location of System Components

For commercial CSPs, many PE controls are inherited from the IaaS/colocation provider — but only if that provider is inside the authorization boundary or a leveraged FedRAMP/IL5 CSO. PE-18 is where US-location is documented.

### 5.12 Planning (PL)

- PL-01 Policy and Procedures
- PL-02 System Security and Privacy Plans
- PL-04 Rules of Behavior + PL-04(01) Social Media and External Site/Application Usage Restrictions
- PL-08 Security and Privacy Architectures
- PL-10 Baseline Selection
- PL-11 Baseline Tailoring

PL-08 architecture narrative must describe tenant isolation, key management, and BCAP paths in enough detail that a 3PAO can pentest them.

### 5.13 Personnel Security (PS)

- PS-01 Policy and Procedures
- PS-02 Position Risk Designation
- PS-03 Personnel Screening + PS-03(03) Information Requiring Special Protective Measures
- PS-04 Personnel Termination + PS-04(02) Automated Actions
- PS-05 Personnel Transfer
- PS-06 Access Agreements
- PS-07 External Personnel Security
- PS-08 Personnel Sanctions
- PS-09 Position Descriptions

See §12. PS-03(04) appears in DoD Table D-1 as a personnel-citizenship parameter.

### 5.14 Risk Assessment (RA)

- RA-01 Policy and Procedures
- RA-02 Security Categorization
- RA-03 Risk Assessment + **RA-03(01) Supply Chain Risk Assessment**
- RA-05 Vulnerability Monitoring and Scanning  
  RA-05(02) Update Vulnerabilities to Be Scanned  
  RA-05(03) Breadth and Depth of Coverage  
  RA-05(04) Discoverable Information  
  **RA-05(05) Privileged Access** (authenticated scans)  
  RA-05(08) Review Historic Audit Logs  
  **RA-05(11) Public Disclosure Program**
- RA-07 Risk Response
- RA-09 Criticality Analysis

RA-05 is the legal home of the entire scan program in §14.

### 5.15 System and Services Acquisition (SA)

- SA-01 Policy and Procedures
- SA-02 Allocation of Resources
- SA-03 System Development Life Cycle
- SA-04 Acquisition Process  
  SA-04(01) Functional Properties of Controls  
  SA-04(02) Design and Implementation Information for Controls  
  SA-04(05) System, Component, and Service Configurations  
  SA-04(09) Functions, Ports, Protocols, and Services in Use  
  SA-04(10) Use of Approved PIV Products
- SA-05 System Documentation
- SA-08 Security and Privacy Engineering Principles
- SA-09 External System Services  
  SA-09(01) Risk Assessments and Organizational Approvals  
  SA-09(02) Identification of Functions, Ports, Protocols, and Services  
  SA-09(05) Processing, Storage, and Service Location
- SA-10 Developer Configuration Management
- SA-11 Developer Testing and Evaluation  
  **SA-11(01) Static Code Analysis**  
  **SA-11(02) Threat Modeling and Vulnerability Analyses**
- SA-15 Development Process, Standards, and Tools + SA-15(03) Criticality Analysis
- SA-16 Developer-provided Training
- SA-17 Developer Security and Privacy Architecture and Design
- SA-21 Developer Screening
- SA-22 Unsupported System Components

SA-09(05) is how you document that processing stays in the US. SA-22 unsupported components become automatic POA&M / High findings.

### 5.16 System and Communications Protection (SC)

- SC-01 Policy and Procedures
- SC-02 Separation of System and User Functionality
- SC-03 Security Function Isolation
- SC-04 Information in Shared System Resources
- SC-05 Denial-of-service Protection
- SC-07 Boundary Protection  
  SC-07(03) Access Points  
  SC-07(04) External Telecommunications Services  
  SC-07(05) Deny by Default — Allow by Exception  
  SC-07(07) Split Tunneling for Remote Devices  
  SC-07(08) Route Traffic to Authenticated Proxy Servers  
  SC-07(10) Prevent Exfiltration  
  SC-07(12) Host-based Protection  
  SC-07(18) Fail Secure  
  SC-07(20) Dynamic Isolation and Segregation  
  SC-07(21) Isolation of System Components
- SC-08 Transmission Confidentiality and Integrity + SC-08(01) Cryptographic Protection
- SC-10 Network Disconnect
- SC-12 Cryptographic Key Establishment and Management + SC-12(01) Availability
- SC-13 Cryptographic Protection
- SC-15 Collaborative Computing Devices and Applications
- SC-17 Public Key Infrastructure Certificates
- SC-18 Mobile Code (+ DoD SC-18(2)(3) DSPAV / mobile-code restrictions)
- SC-20 / SC-21 / SC-22 Secure Name/Address Resolution (DNSSEC)
- SC-23 Session Authenticity
- SC-24 Fail in Known State
- SC-28 Protection of Information at Rest + SC-28(01) Cryptographic Protection
- SC-39 Process Isolation
- SC-45 System Time Synchronization + SC-45(01) Synchronization with Authoritative Time Source

This family plus isolation language in the SRG is most of the IL5 architecture.

### 5.17 System and Information Integrity (SI)

- SI-01 Policy and Procedures
- SI-02 Flaw Remediation  
  SI-02(02) Automated Flaw Remediation Status  
  SI-02(03) Time to Remediate Flaws and Benchmarks for Corrective Actions
- SI-03 Malicious Code Protection
- SI-04 System Monitoring  
  SI-04(01) System-wide Intrusion Detection System  
  SI-04(02) Automated Tools and Mechanisms for Real-time Analysis  
  SI-04(04) Inbound and Outbound Communications Traffic  
  SI-04(05) System-generated Alerts  
  SI-04(10) Visibility of Encrypted Communications  
  SI-04(11) Analyze Communications Traffic Anomalies  
  SI-04(12) Automated Organization-generated Alerts  
  SI-04(14) Wireless Intrusion Detection  
  SI-04(16) Correlate Monitoring Information  
  SI-04(18) Analyze Traffic and Covert Exfiltration  
  SI-04(19) Risk for Individuals  
  SI-04(20) Privileged Users  
  SI-04(22) Unauthorized Network Services  
  SI-04(23) Host-based Devices
- SI-05 Security Alerts, Advisories, and Directives + SI-05(01) Automated Alerts and Advisories
- SI-06 Security and Privacy Function Verification
- SI-07 Software, Firmware, and Information Integrity  
  SI-07(01) Integrity Checks  
  SI-07(02) Automated Notifications of Integrity Violations  
  SI-07(05) Automated Response to Integrity Violations  
  SI-07(07) Integration of Detection and Response  
  SI-07(15) Code Authentication
- SI-08 Spam Protection + SI-08(02) Automatic Updates
- SI-10 Information Input Validation
- SI-11 Error Handling
- SI-12 Information Management and Retention
- SI-16 Memory Protection

SI-02 clocks start at **vendor release**, not detection. SI-04 encrypted-traffic visibility (10) is a High-specific design constraint (TLS inspection architecture or equivalent).

### 5.18 Supply Chain Risk Management (SR)

- SR-01 Policy and Procedures
- SR-02 Supply Chain Risk Management Plan + SR-02(01) Establish SCRM Team
- SR-03 Supply Chain Controls and Processes
- SR-05 Acquisition Strategies, Tools, and Methods
- SR-06 Supplier Assessments and Reviews
- SR-08 Notification Agreements
- SR-09 Tamper Resistance and Detection + SR-09(01) Multiple Stages of System Development Life Cycle
- SR-10 Inspection of Systems or Components
- SR-11 Component Authenticity  
  SR-11(01) Anti-counterfeit Training  
  SR-11(02) Configuration Control for Component Service and Repair
- SR-12 Component Disposal

SCRM plan is SSP Appendix P. SBOMs, provenance, and update integrity are now expected evidence, not optional.

### 5.19 Program Management (PM) and Privacy (PT)

PM controls are organization-level. Many are inherited from the company security program; they still must be described. PT (Personally Identifiable Information Processing and Transparency) applies when PII is in scope; pair with the CNSSI Privacy Overlay if DoD PII/PHI is present.

Use the official Appendix A High template as the complete ID list. If a control is in Appendix A High and not listed above, Appendix A wins.

---

## 6. DoD FedRAMP+ / Table D-1 / SSP Addendum

### 6.1 What “FedRAMP+” actually is

Not a second catalog. It is:

1. Parameter value overrides (DSPAV — DoD Specific Assignment Value) on FedRAMP controls.
2. A short list of extra C/CEs FedRAMP High does not already require.
3. Non-control SRG requirements (location, tenancy, BCAP, citizenship, STIGs, CSSP).

Documented in:

- CSP SRG Appendix D / Table D-1
- *DoD Rev 5 SSP Addendum Controls v1.2* (Excel, 3 Dec 2025)
- *DoD SRG Control Crosswalk* (23 Apr 2026)

### 6.2 Table D-1 examples (verify in the current SRG zip)

These are illustrative of the *kind* of override DISA publishes. They are not a substitute for the official table.

| Control | DoD direction seen in CSP SRG Appendix D | ILs |
|---|---|---|
| AC-07 | Privileged lockout after **3** unsuccessful attempts | IL2/4/5 (5 attempts if SIPR token at higher IL) |
| AU-05(01) | May use FedRAMP value | IL4/5/6 |
| CM-07(05) | DSPAV must be used | IL4/5/6 |
| IA-05(01) | DSPAV must be used | IL4/5/6 |
| PE-15 | DSPAV must be used | IL4/5/6 |
| PS-03(04) | Users = US citizens, US nationals, or US persons; foreign personnel only with AO approval | IL4/5/6 |
| SA-04(05), SA-09(01), SA-09(06)(07)(08) | Listed for IL4/5/6 | IL4/5/6 |
| SC-12(06), SC-17 | PKI per DoDI 8520.02 / 8520.03 | IL4/5/6 |
| SC-18 / SC-18(02) | DSPAV; mobile code restrictions | IL4/5/6 |

Older CC SRG v1r4 Table 2: **10 additional C/CEs** beyond FedRAMP High for IL5. Rev 5 SSP Addendum replaced that list. InfusionPoints’ public breakdown of the Rev 5 addendum: IL5 added ~21 controls and removed ~47 relative to prior FedRAMP+; IL5 NSS added ~178. Use the Excel addendum.

### 6.3 How to implement FedRAMP+ without drowning

1. Export FedRAMP High Appendix A to a workbook.
2. Join the DoD SSP Addendum on Control ID.
3. Flag every row that is “DoD added,” “parameter changed,” or “DSPAV required.”
4. Write the DoD implementation statement in the SSP Addendum, not by silently editing the FedRAMP SSP.
5. Map each extra row to a test case the 3PAO / DISA SCA will run.

---

## 7. CNSSI 1253 NSS overlays

### 7.1 When this layer turns on

The system is an NSS under NIST SP 800-59, **or** the AO applies NSS overlays to elevated CUI at IL5. Current CSP SRG v1r3 language treats IL5 as the NSS tier.

### 7.2 What you add

- CNSSI 1253 Table D-1 “+” selections at the CIA level the AO chose (commonly H-H-x for NSS IL5 in recent SRG analysis — ~170 controls).
- Applicable Appendix E / F attachments:
  - Overlay template
  - Cross Domain Solution Overlay (if any CDS)
  - Intelligence Overlay (U//FOUO; restricted site)
  - Classified System Overlay (IL6, not IL5 — listed so you do not grab it by accident)
  - Privacy Overlay (PII/PHI)
  - Space Platform Overlay (if applicable)
  - ICS overlay if industrial control

Overlays are published on cnss.gov and updated independently of the base instruction. Check the site when you freeze the baseline.

### 7.3 Privacy overlay interaction

DoD PII/PHI in the cloud can add privacy C/CEs that are **not** in FedRAMP High or FedRAMP+. Older SRG tables listed extras such as AC-02(13), AC-03(09), AC-04(08)(15)(17)(18), AC-16, IR-10, MP-06(01)(08), PE-18, and a block of PM controls, with different PII-Low / PII-Mod / PII-High / PHI parameter values. If PII is in scope, pull the current Privacy Overlay and the SRG PII/PHI parameter tables.

---

## 8. Non-control IL5 architecture requirements

These fail packages that have perfect NIST narratives and the wrong cloud shape.

### 8.1 Location

- Processing, storage, and administration of IL5 data: United States, US outlying areas, or DoD on-premises.
- SA-09(05) and PE-18 must name regions, facilities, and any replication targets.
- Support staff who can reach IL5 data cannot do so from offshore operations.

### 8.2 Tenancy and isolation (SRG 5.2.2.3)

- **Physical separation** from non-DoD and non-federal tenants (public, state, local, commercial). Shared physical hosts, storage arrays, and network switches with those tenants are out.
- **Virtual / logical separation** among DoD and federal tenants is acceptable.
- Virtual / logical separation between tenant/mission systems is the minimum inside the federal community.
- Dedicated Host / isolated VM / dedicated tenancy constructs are how hyperscalers usually meet compute isolation (Azure Government Dedicated Host guidance is the public example).
- Storage isolation via customer-managed keys in FIPS-validated HSMs has been accepted by DISA as cryptographic separation for some services — **only where the current PA says so**. Do not assume it.

### 8.3 Federal-community cloud, not public commercial

IL5 CSOs are federal government community clouds. Commercial customers do not ride the same physical stack.

### 8.4 Management plane

DISA will ask, in the architecture briefing:

- Is the CSO management plane shared with the vendor’s commercial cloud?
- Where do break-glass and hypervisor admins sit?
- Can a commercial-region admin reach IL5 hosts?
- How does the vendor corporate network touch the CSO?

A shared commercial management plane is a classic IL5 finding.

### 8.5 Data at rest / in use / in transit placement

- No IL5 data on non-federal physical media.
- Encryption does not waive physical-separation rules for compute during processing unless the current SRG / PA explicitly allows cryptographic isolation for that service class.
- Backups, snapshots, logs, crash dumps, and support bundles are in-scope data.

### 8.6 SCCA functional components

Secure Cloud Computing Architecture is primarily a **mission-owner** stack the CSO must support, not a FedRAMP control family. Four parts:

| Component | Function |
|---|---|
| **CAP** | BCAP (off-prem commercial CSO) or ICAP (on-prem commercial CSO inside a DoD facility). The DISN extension. |
| **VDSS** | Virtual Datacenter Security Stack — firewall, IDS/IPS, WAF facing the CAP |
| **VDMS** | Virtual Datacenter Management Stack — privileged access path, host security, jump / management plane |
| **TCCM** | Trusted Cloud Credential Manager — cloud admin credential vault / issuance |

Mission owner implements SCCA. CSP must support CAP connectivity, flow logs, CSSP sensor insertion, and a management plane that VDMS can actually administer. PA is **not** permission to connect. CATC + CPTC + SCCA checklist come after the PA. Cloud CPG 3.0 and the DISN Connection Process Guide are the connection how-to.

---

## 9. Cryptography (FIPS 140-3 sunset)

### 9.1 The rule

Every cryptographic function in the CSO uses a **CMVP-validated** module on the active validation list, operated **in FIPS mode**. “FIPS-compliant algorithms,” “AES-256,” or “FIPS inside the library but not the module” fail SC-13.

### 9.2 Where crypto is required

| Use | Control | Bar |
|---|---|---|
| Data in transit | SC-08 / SC-08(01), AC-17(02) | TLS 1.2 minimum, 1.3 preferred. SSL / TLS 1.0 / 1.1 forbidden (NIST SP 800-52 Rev 2) |
| Data at rest | SC-28 / SC-28(01) | AES-256 typical; customer-managed keys in HSM |
| Authenticators | IA-05, IA-07 | Module-authenticated |
| Hashing, RNG, signing, keygen | SC-12, SC-13 | Validated module |
| Backups | CP-09(08) | Encrypted with validated module |
| Logs | AU-09(03) | Cryptographic protection |
| Internal service-to-service (NSS IL5) | SC-08 / SC-13 | Treat every internal cryptographic boundary as in-scope |

### 9.3 FIPS 140-2 historical date

**FIPS 140-2 modules go historical 21 September 2026.** New IL5 builds should specify FIPS 140-3 only. Existing 140-2 modules need a documented transition off the CMVP historical list. Appendix Q (Cryptographic Modules Table) must list every module, version, certificate number, and FIPS mode evidence.

### 9.4 Key management

- Customer-managed keys for IL5 mission data.
- HSM-backed (FIPS 140-3 Level 3 is the usual target for CMK).
- Key material stored separately from ciphertext.
- Document SC-12 availability (SC-12(01)) so key-service loss does not become a silent availability failure.

### 9.5 Certificates

DoD users authenticate with DoD PKI (CAC). External-facing and some NPE certs follow DoD CIO commercial-PKI memos and DoDI 8520.02. Let’s Encrypt and public CAs are not a default for IL5 DoD-facing endpoints.

---

## 10. Identity, CAC/PIV, Credential Strength D

### 10.1 FedRAMP High floor

- MFA for privileged and non-privileged organizational users (IA-02(01)(02)).
- Replay-resistant authentication (IA-02(08)).
- Accept PIV (IA-02(12), IA-08(01)).
- NIST SP 800-63 alignment via the Digital Identity Worksheet (SSP Appendix E).

### 10.2 IL5 raise — DoDI 8520.03 Credential Strength D

Hardware token implementing multifactor OTP **or** a PKI certificate. Software TOTP by itself is not Strength D.

Practical pattern:

- DoD human users: CAC / PIV smart card, PKI.
- CSP privileged admins: hardware token or PKI, US persons, no shared accounts.
- Service accounts / NPEs: PKI or equivalent hardware-backed identity; no embedded static secrets (IA-05(07)).
- SaaS customer portals at IL4/5: PIV/CAC per older SRG IA-2(12) language; alternatives only by AO waiver.

### 10.3 Federation

If you federate to DoD ICAM / mission IdP, document IA-08 profiles, assertion protection, and what happens when the IdP is unreachable. Do not invent a local password fallback that drops below Strength D.

---

## 11. Network: BCAP, SCCA, NIPRNet, SNAP

### 11.1 Traffic path

```
DoD user on NIPRNet
    → Boundary Cloud Access Point (BCAP)  [DISA]
        → dedicated circuit / encrypted path
            → IL5 CSO enclave (US federal-community cloud)

Internet user (if the mission allows)
    → NIPRNet IAP / DoD DMZ
        → BCAP
            → IL5 CSO
```

There is **no direct IL5 mission path across the open internet** except through NIPRNet IAPs and a DMZ owned by the mission owner, a DoD component, or DISA.

### 11.2 BCAP facts

- DISA BCAP meet-me points publicly listed in authorization-process decks: Equinix Ashburn VA, San Jose CA, Dallas TX, Chicago IL (confirm current list with SCCA PMO).
- IL4/5 CSOs fund and sustain the circuit from the hosting enclave to the BCAPs they will use.
- Meet-me points in commercial facilities need locked cage/cabinet physical protection per SRG.
- SCCA PMO activates the BCAP connection only after CAO issues Cloud Permission to Connect (CPTC) for a mission Cloud IT Project.

### 11.3 Addressing, DNS, PPSM

- Prefer DoD NIC addressing where the SRG requires it; some IL4/5 CSOs are approved for commercial addressing **with VPN**.
- DNS through DoD-approved DNS.
- Ports, protocols, services registered under DoDI 8551.01 PPSM. Unregistered PPS is a connection-approval finding.

### 11.4 Management access

Management from inside NIPRNet: defined source IPs, encrypted tunnel through CAP/ICAP. Jump hosts. No split-tunnel admin laptops into IL5 from coffee-shop networks (SC-07(07)).

### 11.5 Registration and connection artifacts

| Artifact | System | Who |
|---|---|---|
| Sponsor request | DCAS portal (CAC) | DoD sponsor |
| CSO assessment package | Cloud eMASS | CSP + 3PAO + sponsor |
| Cloud IT Project registration | SNAP (IL2/4/5) or SGS (IL6) | Mission owner |
| Cloud Approval to Connect (CATC) | SNAP / CAO | DISA CAO → CSO |
| Cloud Permission to Connect (CPTC) | SNAP / CAO | DISA CAO → mission C-ITP |
| IATT (if testing) | DCCS IATT form + SNAP | Mission owner |
| Whitelist / NIC / PKI tickets | NIC, PKE, SCCA | CSP + mission |

SNAP: https://snap.dod.mil/ (CAC)

### 11.6 Connection process in order (DISA diagram, condensed)

1. DoD sponsor submits via DCAS.
2. DISA RE2 initial contact + kickoff (sponsor, CSP, 3PAO, CAO, SCCA, NIC, PKI, whitelist).
3. Architecture briefing (use DISA’s preparation guide).
4. Package in cloud eMASS; 3PAO assesses FedRAMP High + FedRAMP+ + SRG.
5. DISA SCA-R / JVT review → AO decision → PA or IATT.
6. CAO issues CATC for the CSO.
7. Mission owner ATOs the C-ITP, registers in SNAP.
8. CAO issues CPTC; SCCA turns up BCAP.
9. CSSP monitoring live before operational traffic.

---

## 12. Personnel, citizenship, screening

### 12.1 Who can touch IL5

SRG 5.2.2.3 / 5.5.x family (confirm current text):

- Privileged access to IL5 CSO infrastructure: **US citizens**.
- Broader IL4/5 user language in Table D-1 PS-03(04): US citizens, US nationals, or US persons; foreign persons only with AO approval.
- IL6 is citizens with SECRET and SSBI — out of scope here.

Implement the stricter of “citizens-only privileged” and whatever the current SRG table says. Do not staff IL5 hypervisor / KMS / SOC privileged roles with non-US persons.

### 12.2 Screening

Typical IL5 privileged bar in public briefings: ADP-2 / IT-2 / Tier 3 (NACLC) plus NDA. Some roles rise to ADP-1 / Tier 5. Mission AO and vendor personnel security policy set the exact tier. Contractors supporting the CSO follow PS-07.

### 12.3 Operations implications

- 24x7 ops must be staffed with eligible persons on US soil.
- Break-glass procedures cannot route through an offshore NOC.
- Background reinvestigation and termination (PS-04 automated disable) must be fast enough that a leaver cannot retain cloud-admin tokens.

---

## 13. STIG / SRG / SCAP / ACAS hardening

### 13.1 CM-6 priority order

1. Current DISA STIG or Application SRG for that product.
2. If no STIG: CIS Level 2 or NIST NCP checklist (SP 800-70).
3. If neither: a custom benchmark the 3PAO validates and DISA accepts.

### 13.2 STIG severity

| CAT | Meaning | Authorization posture |
|---|---|---|
| CAT I | High / severe | **None open at ATO / PA** |
| CAT II | Medium | Remediate or POA&M |
| CAT III | Low | Remediate or accept with documentation |

STIGs refresh quarterly and when vendors ship new versions. Treat STIG maintenance as a standing sprint, not a one-time harden.

### 13.3 What to STIG in a typical IL5 CSO

Build a coverage matrix. Every row needs a STIG/SRG ID, scan tool, last result date, CAT I count.

- Hypervisor / dedicated host
- Guest OS (RHEL, Windows Server, Ubuntu if a STIG exists)
- Containers / Kubernetes (DISA Container Hardening / Kubernetes STIG)
- Database (PostgreSQL, Oracle, SQL Server, Mongo if STIG exists)
- Web server / reverse proxy
- Application server
- Identity provider
- Jump host / bastion
- Network devices / load balancers / firewalls
- Logging stack
- Backup infrastructure
- Laptop / admin workstation images used to manage the CSO

### 13.4 How to scan STIGs

- SCAP content from DISA + OpenSCAP / Evaluate-STIG / STIG Viewer.
- Nessus compliance plugins (often inside ACAS for DoD-connected IaaS/PaaS).
- Results mapped to Vuln IDs (V-###### / SV-######).
- Store 90+ days of evidence.

### 13.5 ACAS

For IaaS/PaaS connected to DISN, Cloud Computing Mission Owner Network SRG expects ACAS (Tenable Security Center + Nessus) or a CSP scanner DISA accepts as meeting DoD scanning and reporting requirements, feeding the CSSP. SaaS offerings often N/A this specific rule but still owe FedRAMP authenticated scans.

DoDI 8531.01 sets DoD vulnerability management policy. USCYBERCOM TASKORD 13-670 is the operational scan order ACAS implements. Next-gen ACAS discussions have floated sub-72-hour cadence; do not design a program that can only scan monthly if your connection agreement demands faster.

---

## 14. Scan program — types, cadence, evidence

This is the “scans” half of the original request. Assessors score the **system**, not a one-off Qualys export.

### 14.1 Scan types (all of them)

| Type | Control hook | What “done” looks like |
|---|---|---|
| Discovery / inventory | CM-08, RA-05(04) | Every addressable asset in Appendix M; 3PAO will discovery-scan the boundary |
| OS / infrastructure vuln | RA-05 | Authenticated, full-plugin, nondestructive, 100% or approved sample |
| Web application / API DAST | RA-05 | Every URL / service in the boundary, authenticated + unauthenticated crawl |
| Database | RA-05 | Every DB including infrastructure DBs, authenticated |
| Container image | RA-05, CM-8 | Every production image class scanned before deploy and on ≤30-day cycle; running instances mapped to scanned image |
| Container runtime / K8s config | CM-06, RA-05 | CIS/STIG against cluster and node |
| STIG / SCAP compliance | CM-06 | Current quarterly STIG, CAT I = 0 at authorization |
| SAST / secret scan | SA-11(01) | Pipeline gate on every release |
| SCA / dependency / SBOM | RA-03(01), SR, SA-11(02) | Dependencies inventoried; KEV matched |
| IaC / policy as code | CM-03, CM-06 | Terraform/K8s manifests fail the build on High/CAT I |
| Malware / EDR | SI-03, SI-04 | Real-time on hosts; periodic scans |
| Wireless IDS | SI-04(14) | If any wireless exists in the boundary (usually “none, disabled”) |
| External attack-surface | RA-05(04) | What the 3PAO will see from the internet before you do |

### 14.2 Cadence

| When | Who | What |
|---|---|---|
| Every build / deploy | CSP CI | SAST, container image, IaC, secret scan |
| Daily (High VDR recommendation) | CSP | Sample of assets likely to drift — CR26 FRR-VDR-TF-HI-02 |
| Every 7 days (High VDR recommendation) | CSP | Drift-prone assets — FRR-VDR-TF-HI-03 |
| **Monthly minimum** | CSP | Authenticated OS + web + DB + container of 100% inventory (or AO-approved unique-class sample). **No sampling of internet-reachable assets.** |
| Monthly | CSP | Inventory export matching scans |
| Monthly | CSP | POA&M + deviation file |
| Quarterly | CSP | STIG content refresh + full compliance rescan |
| 60–90 days before SAR | CSP → 3PAO | Prior scan corpus in machine-readable form |
| Scans of record | CSP, validated by 3PAO | Immediately before SAR; show remediation vs earlier scans |
| Annual | 3PAO | Independent vuln + config + pentest + (High) red team |
| After significant change | CSP + 3PAO as typed | Delta scans of affected inventory |

### 14.3 Authentication and authorization rules (the ones that trigger DFR)

- Moderate and High: **authenticated** scans wherever possible (RA-05(05)).
- Scans run with **full system authorization** — scanners must actually be allowed to see what they claim to see.
- Unauthenticated results ≥ **10%** of a submission → Detailed Finding Review on first offense; Corrective Action Plan on repeat within six months.
- Scanner hardened (unnecessary ports/services closed).
- Plugin / signature updates at least monthly (RA-05(02)).
- Scanner configuration checksum matches the last 3PAO-validated config. Quietly changing plugins mid-ConMon is a finding.
- Machine-readable output (XML / CSV / JSON) with CVE + CVSSv3 (or tool-native severity mapped).
- Every finding maps to a unique asset ID from Appendix M.

### 14.4 Sampling

FedRAMP has a separate *Guide for Determining Eligibility and Requirements for the Use of Sampling for Vulnerability Scans*. Unique asset classes may be sampled; externally accessible assets may not. Document the methodology in the SAP. If you cannot defend sampling, scan 100%.

### 14.5 Containers specifically

- Only hardened images in production.
- Unique asset ID per image class; instances tracked automatically (CM-08).
- Scan the image that will run, not a cousin tag.
- 30-day freshness window is the working FedRAMP expectation.
- Unsigned or unpinned base images fail CM-14 / SI-07(15).

### 14.6 What “good evidence” looks like

For each monthly cycle:

1. Raw scan files (OS, web, DB, container, SCAP).
2. Summary report (exec + detailed + inventory).
3. Inventory workbook (Appendix M) with same hostnames / IPs / image IDs.
4. POA&M with one row per unique scanner finding ID.
5. Deviation file for FP / RA / OR.
6. Scanner config export + signature date.
7. Change log if scanner or plugin set moved.

Keep at least 90 days online; annual-assessment lookback is longer.

### 14.7 CR26 Vulnerability Detection and Response (parallel path)

Consolidated Rules treat “vulnerability” as any weakness, including a stale control statement in the Security Decision Record. High-specific VDR targets include:

- 7-day machine-readable history of detection/response (FRR-VDR-TF-HI-01)
- Daily sampling of similar machine resources (HI-02)
- 7-day drift detection (HI-03)
- Monthly detection of non-drifting resources (HI-04)
- Evaluate all vulns within 2 days of detection (HI-05)

VDR path can replace traditional POA&M with an Accepted Weaknesses list for providers who implement it. Until you have formally switched, run the classic POA&M program. BOD 26-04 / KEV dates override local clocks when shorter. FedRAMP has pointed to mandatory VDR/VER alignment on a 2026 timetable — track the current RFC / notice.

---

## 15. Penetration test and red team

### 15.1 Rules

- Required at Moderate and High.
- Performed by the FedRAMP-recognized 3PAO (JAB path) or the agency-designated assessor (agency ATO path). For IL5, use a 3PAO that has done both High and DISA packages.
- Methodology: NIST SP 800-115 + FedRAMP Penetration Test Guidance.
- Automated scanning is **not** a pentest.
- Initial test **≤ 6 months before the SAR**. Then at least every 12 months.
- Rules of Engagement and test plan live in the SAP and are AO-approved before testing.
- High / Class D additionally requires **CA-08(02) red team exercises**, scoped and scheduled separately from the annual pentest.

### 15.2 Six mandatory attack vectors

The 3PAO covers each vector or writes a formal omission justification in the ROE:

1. **External → corporate** — phishing / social engineering of CSP admins and corporate systems that can reach the CSO.
2. **External → CSP target system** — unauthenticated internet attack plus movement to the internal boundary.
3. **Tenant → CSP management plane** — web app, APIs, control plane, ticketing, “support” paths.
4. **Tenant → tenant** — isolation breakout. This is the IL5 vector. Physical/logical separation claims die here if they are paper-only.
5. **Mobile application → target system** — if a mobile client exists; otherwise justify N/A.
6. **Client-side application / agent → target system** — browser agents, desktop clients, sidecars.

### 15.3 Threat models FedRAMP names

Enterprise (recon, privilege escalation, infiltration/exfil, detection evasion, persistence), web application, and mobile. Infrastructure vs SaaS changes origin of testing but not the need to cover the vectors.

### 15.4 What the report must give the AO

Attack path narratives, evidence, detection/response observations (whether SI-04 actually fired), residual risk, and a map back to failed controls (often AC-04, SC-07, SC-04, SC-39, CM-06).

Rehearse all six vectors internally before the 3PAO arrives. Tenant isolation surprises are the expensive kind.

---

## 16. POA&M, deviations, remediation clocks

### 16.1 POA&M rules

- Official FedRAMP POA&M Excel template only.
- One unique scanner vulnerability ID = one POA&M item. You may split one ID into several items (different asset classes, different fixes). You may **not** bundle different IDs into one row.
- Every open SAR finding and every late ConMon finding lives here.
- Weak control implementations discovered by scan (unsupported software = SA-22, bad crypto = SC-13) are logged as **both** the vuln and the control finding.

### 16.2 Deviation types (Vulnerability Deviation Request Form)

| Type | Meaning | Bar |
|---|---|---|
| False Positive (FP) | Tool is wrong | Reproduce, show why, 3PAO/AO accept; closed tab |
| Risk Adjustment (RA) | Severity is lower in this context | Compensating control + residual risk; 3PAO/AO accept |
| Operational Requirement (OR) | Cannot fix without breaking the service | **High-impact ORs are not approved.** Approved ORs stay Open and are reassessed |
| RA + OR | Combined request | Same High-OR bar |
| Vendor Dependency (VD) | Fix is waiting on a vendor | **Not a deviation request.** Track on the POA&M. Monthly vendor check-in evidence. High VDs must be **mitigated to Moderate within 30 days** |

Unapproved deviations are just late POA&Ms. FedRAMP will not authorize with open High residual risks that lack an accepted RA. Official form: Vulnerability Deviation Request Form.

### 16.3 Remediation clocks

| Item | Clock start | Deadline |
|---|---|---|
| Critical / High vuln | Detection | **30 days** |
| Moderate vuln | Detection | **90 days** |
| Low vuln | Detection | **180 days** |
| Vendor security patch (SI-02) | **Vendor release date** | **30 days** |
| CISA KEV | KEV due date | KEV date if shorter than local clock |
| STIG CAT I | Detection / new STIG drop | Treat as High; 0 open at authorization |
| STIG CAT II | Detection | 90-day class unless AO says otherwise |
| BOD 26-04 high-risk KEV triage | KEV add | Agency/CSP forensic triage in days, not weeks |

### 16.4 Escalation triggers (ConMon performance management)

- Unique vuln count +20% from P-ATO baseline (or +10 unique, whichever greater) → visibility trigger.
- Unauthenticated ≥10% → DFR, then CAP on repeat.
- 5+ unique High/Critical POA&Ms aged >30 days → DFR.
- Same aged >60 days → Corrective Action Plan.
- Late monthly package → DFR / CAP path.

Track these as operational SLOs, not GRC folklore.

---

## 17. Continuous monitoring calendar

### 17.1 Monthly package

- Raw OS, web, DB, container scans + summaries
- Updated Integrated Inventory Workbook (Appendix M)
- Updated POA&M
- Deviation request file
- ConMon monthly executive summary (template)
- Scanner signature / config attestation
- Incident list for the period (even if “none”)

Upload on a fixed calendar day. Use the FedRAMP Continuous Monitoring Deliverables Template to publish the schedule and repository path (Connect.gov / agency repository). Folder naming must include POAM or POA&M if using the managed repository.

### 17.2 Collaborative ConMon

RFC-0026 / CA-07: if you have multiple agency customers, you owe monitoring information to **all** of them and to FedRAMP — either via the VDR balance-improvement path or via monthly scans + POA&M + annual independent scans. Do not build a one-sponsor-only reporting pipe if you intend to scale.

### 17.3 Annual

- SSP update (every control implementation that drifted)
- 3PAO annual assessment: core controls + ~1/3 of the remainder so every control is tested at least once every three years
- Annual independent scans
- Annual pentest (and High red team per CA-08(02) schedule)
- CP-04 / IR-03 tests
- Inventory and boundary re-validation (discovery scan)
- Significant-change backlog closed or scheduled
- POA&M aging review with AO

### 17.4 DoD extras after PA

- eMASS continuous control status
- CSSP ticket / sensor health
- SNAP registration kept current
- STIG quarterly deltas
- Architecture-change notices to DISA RE2 / SCCA when the CSO shape moves

---

## 18. Incident reporting clocks

### 18.1 FedRAMP Consolidated Rules (Class D / High)

Incident Evaluation and Communication + RFC-0031. Ratings use Potential Agency Impact N-rating (PAIN / N5–N1).

Published RFC-0031 Class D table (confirm live CR26 page before you print this into an IRP):

| PAIN | Initial Incident Report | Ongoing | Final |
|---|---|---|---|
| N5 | **15 minutes** after evaluation | every **3 hours** | **3 hours** after recovery |
| N4 | 30 minutes | every 6 hours | 6 hours after recovery |
| N3 | 1 hour | every 6 hours | 6 hours after recovery |
| N2 | 1 hour | every 6 hours | 6 hours after recovery |
| N1 | 1 hour | every 24 hours | 24 hours after recovery |

A later notice described an even tighter “15 min IIR / 3 hr OIR / 3 hr FIR for N5–N3 on Class D.” **IRP must quote the live CR26 page**, not this file, on the day you freeze the plan.

IIR contents: IR coordinator contact, provider tracking ID, description, timeline (start, detect, evaluation complete), estimated customer impact, affected agencies, recovery sketch.

### 18.2 CISA

CR26 moves primary CISA notification for agency-customer impact onto the **agency**, except where contracts still require the CSP to notify CISA. Maintain a CISA path anyway for:

- Confirmed compromise of federal customer data
- KEV / BOD 26-04 escalation
- CIRCIA if you are a covered entity (72-hour covered-cyber-incident / 24-hour ransom payment — separate statute)

### 18.3 DoD

- CSSP notified per DoDI 8530.01 / DTM-24-001 and the PA.
- CJCSM 6510.01 incident reporting timelines for DoD networks.
- DFARS 252.204-7012 / Subpart 204.73 for CUI incidents on contractor systems (72-hour to DoD) — still relevant if the CSO or a connected contractor enclave holds CUI.
- Spillage (IR-09) is its own playbook: isolate, report, sanitize, after-action.

### 18.4 IR capability DISA will ask about

Volatile memory capture, forensic imaging, malware detonation, time-sync’d logs, ability to preserve tenant data without mixing tenants. Name the CSSP and the CSP IR retainer in the IRP.

---

## 19. Logging (M-21-31 and successors)

OMB M-21-31 created EL1 / EL2 / EL3 maturity and long hot/cold retention (often cited as 12 months hot + 18 months cold for EL1). A 2026 OMB memo revised federal logging toward a risk-based six-month searchable baseline for agencies. **Your CSO still owes whatever FedRAMP AU family + customer contracts + DoD CSSP feed specify.**

Build for the stricter of:

- FedRAMP AU-02/03/04/06/11 parameter values in Appendix A High
- Customer agency EL-tier
- CSSP sensor requirements
- M-21-31 Appendix event types that still appear in contracts (CloudTrail / Azure Activity / GCP Admin, IdP sign-in, DNS, DHCP, firewall, break-glass, VPC flow, WAF, EDR)

Minimum engineering bar that survives assessment:

- UTC timestamps from an authoritative source (SC-45(01), NIST time)
- User identity, source/destination, action, result on every privileged and auth event
- Central store, crypto-protected, access-limited (AU-09)
- Queryable by CSP IR and by CSSP / agency SOC without a ticket that takes days
- Break-glass account use is always logged and alerted
- Encrypted-traffic visibility strategy documented under SI-04(10)

---

## 20. Significant change process

### 20.1 Rule

Every change is typed. Impact-level changes (High ↔ Moderate) are **reauthorization**, not a significant-change notice.

Types used in FedRAMP SCN / RFC-0007 language: ordinary significant, transformative, adaptive, etc. Transformative changes are the ones that move boundary, tenancy, crypto, identity, or data flows.

### 20.2 Transformative change pattern (RFC-0007)

1. Notify FedRAMP and agency customers ≥ 14 calendar days before the first ConMon meeting that will discuss the change.
2. Discuss in **two sequential** monthly ConMon meetings before executing.
3. 3PAO reviews scope **before** execution and concurs; SAP for the delta exists first.
4. Execute.
5. Notify FedRAMP and customers within **1 calendar day** and at the next ConMon meeting.
6. 3PAO starts assessment within **1 calendar day**; target complete within **7 calendar days**.

### 20.3 DoD parallel

Cloud Change Request / whitelist forms on the DCCS library. Architecture deltas that touch BCAP, tenancy, region, or management plane go through DISA RE2 / SCCA, not just FedRAMP ConMon.

### 20.4 Examples that are significant / transformative

- New region or AD
- New service that stores CUI
- IdP change
- Customer-managed key service change
- Container platform swap
- Opening a public endpoint
- Absorbing a corporate shared service into the boundary
- Losing physical isolation (this is an IL5-breaking change)

---

## 21. IaaS / PaaS / SaaS shared responsibility

| Concern | IaaS | PaaS | SaaS |
|---|---|---|---|
| Physical / PE | Inherit from IL5 IaaS | Inherit | Inherit |
| Hypervisor isolation | CSP | CSP | CSP |
| Guest OS STIG + scan | Mission / CSP depending on who patches | Often CSP | CSP |
| App code SAST / pentest | Mission | Split | CSP |
| Customer tenant isolation | CSP + mission config | CSP + mission config | CSP |
| CAC/PIV on the app | Mission | Split | CSP must offer it at IL5 |
| BCAP | CSP enclave + mission C-ITP | Same | Same; SaaS still needs CATC/CPTC |
| ACAS | Required for IaaS/PaaS DISN-connected | Required | Often N/A; FedRAMP scans still required |
| Inventory | VMs, VPCs, disks | Plus platform services | Plus every SaaS component and job |

Write a Customer Responsibility Matrix (CRM / CIS workbook, SSP Appendix J). Every inherited control names the offering and authorization ID you inherit from. Every customer-responsible control is written so a mission AO can put it in **their** SSP.

---

## 22. Inheritance, CRM, GovCloud / Azure Government

### 22.1 Leveraged authorizations

You may inherit PE, some SC, some AU, some CP from:

- AWS GovCloud (US) / AWS Secret-Region is out of IL5 scope; GovCloud has FedRAMP High and DoD IL2/4/5 PAs for in-scope services
- Azure Government (US Gov AZ / TX / VA) — IL5 isolation guidance requires Dedicated Host in those regions for compute isolation
- Google Cloud Assured / Google public IL5 PA scope list
- Other FedRAMP High + IL5 PAs

Inheritance is **service-by-service**. A platform PA does not cover a service that is not on the PA boundary list. Check the FedRAMP Marketplace package and the DISA PA service list.

### 22.2 What you cannot inherit

- Your application’s AC / IA / SI-10 / SA-11
- Your customer-managed keys if you hold them
- Your tenant-isolation design
- Your ConMon scans of **your** inventory
- Your IRP for **your** CSO
- Citizenship of **your** privileged staff

### 22.3 CRM hygiene

- One row per control: Implemented / Inherited / Shared / Customer / Not applicable
- Inherited-from package ID + date
- Shared: exact split sentence
- Customer: exact implementation they must perform
- 3PAO tests inherited controls by examining the leveraged package and residual risk, not by ignoring them

---

## 23. Package artifacts and SSP appendices

### 23.1 FedRAMP SSP appendices (Rev 5)

Starred items must use the FedRAMP template.

| App | Content |
|---|---|
| A* | High security controls (the big one) |
| B | Acronyms |
| C | Policies and procedures |
| D | User guide |
| E | Digital Identity Worksheet |
| F* | Rules of Behavior |
| G* | Information System Contingency Plan |
| H | Configuration Management Plan |
| I | Incident Response Plan |
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
