# Planning Assumptions

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Shop validation  
> **Supersedes:** None

| ID | Assumption | Consequence if false | Validation |
|---|---|---|---|
| A-001 | Initial users are expert estimators on desktop workstations. | IA/input model may change. | Observe users and devices. |
| A-002 | Standalone/offline is valuable for first deployment. | Service architecture moves earlier. | OQ-021/022 interviews. |
| A-003 | STEP is the most useful exact interchange format. | Import priorities/kernel choice may change. | RFQ corpus + TASK-003. |
| A-004 | STL/3MF are common enough to support but cannot yield B-rep-equivalent claims. | Mesh investment/UX may change. | Corpus and estimator review. |
| A-005 | A separate native geometry worker is operationally acceptable. | Need in-process or service alternative. | ADR-0005 spike. |
| A-006 | A custom WebView-based UI can host the required dense workflow and wgpu surface. | Revisit Slint/Iced/native composition. | TASK-005. |
| A-007 | Shop rates and feeds/speeds can be represented as approved effective-dated libraries. | Configuration/governance model changes. | OQ-011–014. |
| A-008 | Deterministic proposals plus human review provide more trust than opaque automation. | Product positioning changes. | Usability/interview evidence. |
| A-009 | SQLite plus local blobs meets standalone scale. | Persistence choice changes. | TASK-006 workload tests. |
| A-010 | Representative models and actuals can be used under an approved private-data process. | Accuracy validation may be blocked. | TASK-008/legal-security review. |
| A-011 | Initial quote previews can be generated without replacing the shop's ERP. | Integration scope moves earlier. | OQ-019/025. |
| A-012 | Coarse runtime ranges are useful when method/uncertainty are explicit. | Vertical slice runtime scope changes. | Blind estimator study. |
| A-013 | Estimators can name a small set of plausible routing alternatives and infeasibility reasons. | Optimizer and UX scope must change. | Route workshop + historical quote sample. |
| A-014 | Capacity calendars and readiness observations can be obtained with owner, timestamp, and freshness policy. | Capacity/delivery views remain unavailable or manual. | OQ-032–035 source inventory. |
| A-015 | Three-point inputs and sensitivity rankings are understandable enough to improve decisions. | Use simpler deterministic scenarios. | Blind interpretation/usability study. |
| A-016 | Revision packages expose enough stable evidence to produce useful, uncertainty-aware deltas. | Limit comparison to hash/metadata/manual review. | Revision corpus and TEST-056–060. |
| A-017 | Requirement coverage can be reviewed without claiming automated engineering authority. | Narrow to a manual checklist. | Estimator/quality review. |
| A-018 | Shop corrections, CAM artifacts, and actuals can be linked under an approved privacy/data-rights policy. | Learning remains manual and per-estimate. | Security/legal/shop review. |
| A-019 | External supplier quotes can be compared locally without transmitting controlled geometry. | Make/buy remains manual narrative. | Sourcing workflow review. |
| A-020 | Bid/priority policies can be expressed transparently without displacing accountable human judgment. | Retain only explicit blockers and manual prioritization. | Leadership workshop and retrospective test. |

Assumptions are not requirements. Resolve or renew them with owner/date before their dependent ADR is accepted.
