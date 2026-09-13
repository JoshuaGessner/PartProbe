# Huntsville Aerospace Test Rate Baseline

## Metadata

- **Status:** In Review
- **Last updated:** 2026-09-11
- **Related requirements:** REQ-F-014–REQ-F-018, DATA-003–DATA-010
- **Related ADRs:** ADR-0006, ADR-0007
- **Open questions:** Shop-approved rates, material offers, stock practice, runtime studies, overhead treatment, and pricing policy before deployment
- **Dependencies:** Public sources listed below and estimator review
- **Supersedes:** The earlier unreferenced synthetic UI demo values

## Purpose and authority

This document defines `huntsville-2026q3-test-*` version 1: a durable, source-controlled baseline for internal usability tests around Huntsville, Alabama. It is not a survey of local job-shop prices, an approved shop rate card, a customer quote basis, or deployment data. The application loads it only after an explicit user action, leaves all confirmations clear, and does not persist it until the user reviews and saves a local draft.

Public data anchors the labor and physical reference values. The machine rate, stock allowance, removal rate, fixed times, and markup are transparent testing assumptions where no defensible public Huntsville shop-rate dataset exists. Before deployment, preserve this test profile separately and replace every value with reviewed shop records and calibration evidence.

## Version 1 values

| Input | Test value | Basis | Confidence / limitation |
|---|---:|---|---|
| Currency | USD | Product decision | High |
| Setup labor | USD 47.00/hr | Huntsville machinist wage, burdened and rounded | Medium |
| Programming | USD 50.90/hr | Huntsville CNC programmer wage, burdened and rounded | Medium |
| Run labor | USD 47.50/hr | Huntsville CNC operator wage, burdened and rounded | Medium |
| Quality inspection | USD 37.00/hr | Huntsville inspector wage, burdened and rounded | Medium |
| Machine | USD 45.00/spindle-hr | Test-only asset, power, maintenance, and facility allowance | Low; not a quoted local market rate |
| Pricing markup | 25% of cost | Test-only commercial policy | Low; equals 20% gross margin before excluded costs |
| Material | 6061-T651 aluminum, 2,700 kg/m³ | Aluminum Association density | High for generic density; exact certified lot may vary |
| Material price | USD 10.00/kg | Rounded public 6061 plate benchmark | Medium-low; excludes freight, cuts, minimums, and certificates |
| Rectangular stock allowance | 6.4 mm total on X, Y, and Z | Test assumption equivalent to 3.2 mm per side | Low; PartProbe stores total axis addition, not per-side allowance |
| Reference machine envelope | 762 × 406 × 508 mm | Haas VF-2 published travels | High as a public reference; not an installed-shop capability |
| Coarse removal rate | 250,000 mm³/min | Aluminum roughing test assumption | Low; not feeds/speeds, CAM, or a cycle-time guarantee |
| Setup | 90 min/lot | Test assumption | Low |
| Programming | 120 min/lot | Test assumption | Low |
| Load/unload | 5 min/item | Test assumption | Low |
| Inspection | 15 min/lot | Test assumption | Low; excludes FAI/CMM/programming/certification scope |

The wage-to-burden conversion uses the BLS manufacturing compensation relationship in which wages represent 66.8% of employer compensation, or approximately `wage / 0.668`. The resulting values are rounded for stable test inputs rather than presented as precise costs. The recurring three-axis test basis is USD 92.50/hr before markup (`run labor + machine`) and USD 115.63/hr after the 25% markup; this is a calculation sanity check, not a local quote benchmark.

## Market context

Huntsville has unusually high engineering and production employment concentration and a substantial aerospace/defense ecosystem around NASA Marshall and Redstone Arsenal. That supports using aerospace-style review expectations in tests, but it does not establish local contract machining prices. The baseline intentionally excludes aerospace-specific cost drivers that the current application cannot yet derive: drawing tolerances/GD&T, AS9102 first-article scope, CMM programming, material certifications, source inspection, special processes, tooling/fixtures, outside processing, freight, overhead, risk, and detailed CAM cycle analysis.

## Public sources

- [BLS Huntsville occupational employment and wages, May 2025](https://www.bls.gov/regions/southeast/news-release/occupationalemploymentandwages_huntsville.htm)
- [Alabama Department of Labor Huntsville wage publication](https://www2.labor.alabama.gov/oes/wage/Huntsville.pdf)
- [BLS employer costs for employee compensation, June 2025](https://www.bls.gov/news.release/archives/ecec_09122025.htm)
- [NASA Marshall supplier and business information](https://www.nasa.gov/marshall/do-business-with-marshall/)
- [Aluminum Association Aluminum Standards and Data](https://www.aluminum.org/sites/default/files/2026-01/Teal%20Sheets%20December%202025.pdf)
- [Public 6061 price reference](https://www.materialpricebook.com/prices/aluminum/6061/plate/1)
- [Haas VF-2 published specifications](https://www.haascnc.com/machines/vertical-mills/vf-series/models/small/vf-2.html)

## Replacement gate

Deployment values require a shop owner to review burden treatment, machine ownership/utilization, utilities, consumables, overhead allocation, supplier quotes, common purchased stock sizes, scrap/remnant policy, operation-level runtime studies, quality scope, minimum charges, margin targets, and effective-date/scope rules. Each adopted record needs immutable version, source, reviewer, time, reason, and replay evidence; the test profile must never be relabeled as that authority.
