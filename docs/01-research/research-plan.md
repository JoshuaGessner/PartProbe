# Research Plan

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-002–REQ-F-010, REQ-F-040–REQ-F-065, REQ-NF-001–REQ-NF-022  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Access to shop staff, anonymized estimates, representative models  
> **Supersedes:** None

## Objectives

Reduce the largest irreversible risks before production code: geometry fidelity and packaging, desktop framework fit, estimation validity, security boundaries, and availability of validation evidence.

## Evidence order

1. Law, regulation, contract, customer requirement, or formal standard.
2. Official technical specifications and project documentation.
3. Peer-reviewed or recognized institutional references.
4. Independent industry practice.
5. Vendor claim, explicitly labeled.
6. Shop-specific assumption, awaiting validation.
7. Unverified idea, never treated as a requirement.

## Workstreams and exit evidence

| Workstream | Immediate question | Exit evidence |
|---|---|---|
| Shop discovery | How are estimates actually created and approved? | Interviews, redacted examples, workflow maps |
| CAD/geometry | Can STEP/STL/3MF be parsed and packaged safely cross-platform? | Golden fixtures, kernel spike, license review |
| Runtime | Which estimation levels are useful at which evidence quality? | Reviewed examples and error bands by method |
| UI | Can the framework deliver dense accessible custom desktop interaction and 3D integration? | Time-boxed shell/viewport/table/keyboard spike |
| Persistence | Can immutable snapshots and revisions survive migration/backup? | Schema prototype, restore test, threat review |
| Security | What data classifications and deployment boundaries apply? | Written data-flow review and external determination where needed |
| Routes/capacity/economics | Which alternatives, bottlenecks, calendars, and cost bases match shop decisions? | expert-scored route corpus, source/freshness map, approved glossary |
| Uncertainty | Which range method improves decisions without false confidence? | scenario/three-point blind study, calibration and sensitivity evidence |
| Revision/requirements/PMI | Which model and requirement changes can be mapped reliably? | revision corpus, source-priority policy, AP242 translator scorecard |
| Learning/CAM/actuals | Which observations may support governed improvements? | event taxonomy, rights/privacy review, adapter fixtures, bias/backtest plan |
| Availability/sourcing/bid | Which sources and decision policies are authoritative enough to advise? | ownership/freshness map, make/buy examples, reviewed blocker/score policy |

## Method

Record hypotheses before spikes; use the same fixture corpus across kernel candidates; preserve negative results; separate cold-start, steady-state, accuracy, packaging, licensing, and developer-effort measurements. Interview at least two estimators plus one programmer, machinist, quality representative, and purchasing representative before finalizing shop-dependent rules.

## Remaining primary research

- Obtain current translator/kernel commercial terms directly from vendors.
- Obtain licensed copies of ISO/ASME/3MF specifications where evaluation needs normative detail.
- Have qualified counsel/security leadership determine export-control and CUI obligations for intended customers and deployment.
- Benchmark real representative models; public synthetic fixtures are not a substitute.
