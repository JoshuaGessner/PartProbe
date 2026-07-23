# Product Roadmap

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001–REQ-F-065; REQ-NF-001–REQ-NF-022  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Shop validation and technical spikes  
> **Supersedes:** None

| Phase | Outcome | Exit evidence |
|---|---|---|
| 0 Discovery & validation | Approved problem, boundaries, decisions, fixture/data access | Interviews; reviewed docs; UI/geometry/persistence/calculation spikes; advanced-system ADR review; risk owners |
| 1 Foundation | Cross-platform workspace and trustworthy future-safe primitives | Domain/money/units/provenance; route-set, requirement, availability, correction and revision snapshot contracts; repositories; geometry protocol; CI |
| 2 CAD-assisted vertical slice | One approved model-to-transparent-quote workflow without advanced-scope overload | Original 28 criteria; basic requirement checklist; manual availability fields and make/buy comparison; preserved revisions; three-platform demo |
| 3 Feature and decision support | Reviewed features plus manual/multiple route comparison and revision insight | Route alternatives; basic capacity lead-time check; geometry revision comparison; feature-cost overlays; correction capture; CAM report import |
| 4 Shop libraries/readiness | Governed material/tool/machine/rate/vendor/template and availability data | Effective-dated approval/versioning, freshness/reservation workflows, theoretical versus readiness-adjusted estimates |
| 5 Aerospace/quality/MBD depth | Requirement coverage and quality/documentation effort are explicit | Readiness matrix, FAI/CMM/cert/traceability; experimental AP242 PMI review under ADR-0012 |
| 6 Actuals/CAM/calibration | Controlled estimate↔CAM↔machine↔actual reconciliation and learning | Neutral adapters, categorized variance, correction cohorts, reviewed rule proposals, immutable originals |
| 7 Advanced optimization | Probabilistic estimates, automated route optimization, capacity/opportunity scoring and advanced revision explanations | Reviewed distributions/sensitivity, optimizer/capacity validation, structured PMI evidence, controlled recommendations; still not CAM/scheduling authority |
| 8 Team deployment/integration | Authorized multi-user LAN operation and governed live integrations | Service boundary, central storage, RBAC/audit/backup/conflict tests; approved CAM/ERP/QMS/schedule adapters |

Dates are intentionally absent until shop access, spike results, staffing, and supported-platform policy are known.

## Capability placement

| Capability | Early/initial scope | Intermediate | Advanced/research |
|---|---|---|---|
| Routing alternatives | Manual alternatives and one approved route | Generated/compared alternatives | Automated bounded optimizer |
| Capacity/opportunity | Typed data and manual bottleneck | Capacity-aware feasibility/basic metrics | Opportunity scenarios and capacity-adjusted scoring |
| Uncertainty | Confidence + user ranges/sensitivity | Three-point/scenario analysis | Validated PERT/triangular or seeded Monte Carlo where justified |
| Revision analysis | Preserve all revisions and basic facts | Geometry/feature/cost comparison | Advanced causal explanations across process alternatives |
| Requirement coverage/PMI | Basic manual checklist | Full readiness matrix and AP242 research | Structured semantic PMI where translator evidence supports it |
| Learning/CAM | Capture corrections; manual reconciliation | CAM report import and cohort analytics | Governed recommendations and vendor adapters |
| Availability/make-buy | Manual status and comparisons | Freshness/reservation-aware estimates | Integrated readiness planning, not ERP replacement |
| Feature visualization/bid scoring | Existing provenance/approval foundations | Feature cost/risk modes and configurable bid review | Capacity-adjusted prioritization after policy validation |
