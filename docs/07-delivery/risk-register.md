# Risk Register

> **Status:** In Review
> **Last updated:** 2026-08-01
> **Related requirements:** All
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Named owners after staffing
> **Supersedes:** None

| ID | Risk | Probability | Impact | Mitigation | Owner | Trigger | Review date | Affected milestone | State |
|---|---|---|---|---|---|---|---|---|---|
| RISK-001 | OCCT/FFI defect, malformed CAD, wrong-resource binding, or unintended descriptor/HANDLE inheritance escapes/crashes/leaks from the worker boundary | M | Critical | Isolated worker, identity-bound verified-copy transport, immutable byte-stream parsing, quotas, Unix descriptor allowlist/sentinel proof, fail-closed Windows direct policy, future Windows HANDLE allowlist, corpus/fuzzing, restart boundaries | Geometry lead | Host crash, escape, unrelated inherited resource, fallback represented as direct, leak, or corrupted output | 2026-08-01 | M0.2–M2 | Open |
| RISK-002 | STEP translation/healing silently changes geometry | M | Critical | Preserve source and pre/post metrics/actions; user review | Geometry lead | Material measurement/topology delta without warning | 2026-07-22 | M0.2–M2 | Open |
| RISK-003 | Mesh evidence is mistaken for exact topology | H | High | Separate representation types, confidence ceiling, labels | Product/geometry | Mesh-derived fact shown as exact/high confidence | 2026-07-22 | M1–M3 | Open |
| RISK-004 | UI/WebView/GPU stack fails dense accessible cross-platform needs | M | High | Time-boxed spike with common suite and fallback | UI lead | Platform workflow/accessibility/performance gate fails | 2026-07-22 | M0.2–M2 | Open |
| RISK-005 | Coarse runtime output creates false precision | H | High | Method/range labels, reviewed cohorts, no CAM claims | Estimation lead | User treats coarse result as simulated/guaranteed | 2026-07-22 | M1–M3 | Open |
| RISK-006 | Shop rate/scrap/overhead policy is unavailable/inconsistent or synthetic/demo values leak into production setup | H | High | Empty-on-install library, isolated synthetic fixtures, guided validation blockers, versioned assumptions, TASK-007 interviews/calibration | Product owner | Reconciliation cannot explain cost basis or a production estimate references synthetic data | 2026-07-29 | M0.2–M2 | Open |
| RISK-007 | Calculation, rate-selector, rate-card, pricing, or rounding change alters approved quotes | M | Critical | Typed DAG, pinned immutable versions/snapshots, golden/migration/replay tests | Domain lead | Replay differs without versioned migration | 2026-07-29 | M1–release | Open |
| RISK-008 | Sensitive/export-controlled data leaves boundary | M | Critical | Offline default, classification/RBAC/export controls, qualified determination | Security owner | Unapproved egress or external resolution | 2026-07-22 | All | Open |
| RISK-009 | Product language implies compliance or production CAM | M | High | Content gates, non-goals, training/release review | Product/legal | Unsupported certification/safety claim | 2026-07-22 | All | Open |
| RISK-010 | SQLite/blob backup is inconsistent/unrecoverable | M | Critical | Atomic manifests, WAL-aware backup/restore drills | Persistence lead | Hash mismatch, missing blob or failed restore | 2026-07-22 | M1–M2 | Open |
| RISK-011 | Native/proprietary CAD license, cost or packaging blocks format | H | High | No promise before terms/fixture/package proof; conversion path | Product/legal | Vendor terms or redistribution fail | 2026-07-22 | M0.2–M7 | Open |
| RISK-012 | No legally usable representative fixture corpus | H | High | Synthetic/public baseline plus governed private corpus | QA/security | Required test has no approved representative fixture | 2026-07-22 | M0.2–M7 | Open |
| RISK-013 | Historical calibration learns execution anomalies | M | High | Cohorts, causes, approval, immutable originals | Estimation lead | Proposed default is driven by isolated/out-of-scope jobs | 2026-07-22 | M6 | Open |
| RISK-014 | Feature/topology IDs are unstable across kernel/revisions | H | High | Snapshot-scoped IDs; explicit approximate cross-version mapping | Geometry lead | Mapping changes without explainable evidence | 2026-07-22 | M2–M7 | Open |
| RISK-015 | Team mode uses SQLite on a network share | L | Critical | Service-only team architecture and deployment checks | Architecture lead | Shared-file configuration requested/detected | 2026-07-22 | M8 | Open |
| RISK-016 | Dependency abandonment/license incompatibility | M | High | Inventory, license/maintenance/advisory gates, exit plan | Architecture lead | Unsupported release or incompatible obligation | 2026-07-22 | All | Open |
| RISK-017 | Route optimizer proposes implausible or unsafe manufacturing plans | H | Critical | Feasibility constraints, expert fixtures, rejection reasons, human adoption | Manufacturing lead | Expert rejects route for basic capability/access/workholding reason | 2026-07-22 | M3–M7 | Open |
| RISK-018 | Probabilistic outputs create false confidence | H | High | Staged methods, source/range disclosure, calibration and plain-language caveats | Estimation lead | Percentile presented without evidence/method suitability | 2026-07-22 | M7 | Open |
| RISK-019 | Capacity/backlog data is incomplete or poor | H | High | Source/freshness/confidence, missing-data behavior, reconciliation | Operations owner | Feasibility differs materially from actual schedule | 2026-07-22 | M3–M7 | Open |
| RISK-020 | Opportunity cost is confused with accounting cost | H | High | Separate typed totals, labels, formulas, training and tests | Finance/product | Opportunity value enters job cost/financial report silently | 2026-07-22 | M3–M7 | Open |
| RISK-021 | Revision comparison creates false-positive changes | H | Medium | Alignment/tolerance controls, identical/unit/mesh fixtures, review | Geometry lead | Unchanged manufacturing intent produces material delta | 2026-07-22 | M3 | Open |
| RISK-022 | Revision comparison misses critical change | M | Critical | Multi-method comparisons, requirement links, expert golden fixtures | Geometry/quality | Known geometry/PMI/requirement change absent | 2026-07-22 | M3–M7 | Open |
| RISK-023 | Requirement extraction/coverage omits obligation | M | Critical | Source inventory, reconciliation, human verification, readiness gate | Quality lead | Critical clause/requirement missing from matrix | 2026-07-22 | M2–M5 | Open |
| RISK-024 | PMI importer is inconsistent/incomplete across translators | H | High | AP/schema fixtures, semantic/graphical separation, translator/version evidence | Geometry/quality | Same PMI differs materially across supported path | 2026-07-22 | M5–M7 | Open |
| RISK-025 | Feature-level cost allocation implies unsupported precision | H | High | Reconcile to total, allocation method/coverage, approximation/unallocated labels | Product/estimation | Coarse cost displayed as exact per-feature value | 2026-07-22 | M3 | Open |
| RISK-026 | Correction learning reinforces estimator/customer bias | M | High | Segmentation, cohort/privacy/bias review, no auto-change, governance | Product/security | Suggestion correlates with person/customer rather than causal evidence | 2026-07-22 | M6–M7 | Open |
| RISK-027 | Too little correction/actual data produces unstable learning | H | High | Minimum cohorts, uncertainty, holdout/retest, no-recommendation state | Estimation lead | Recommendation changes materially with one observation | 2026-07-22 | M6–M7 | Open |
| RISK-028 | CAM integration becomes vendor-specific core coupling | M | High | Neutral DTO/ports, adapter contracts, two-shape fixtures | Integration lead | Vendor type leaks into domain/calculation API | 2026-07-22 | M3–M8 | Open |
| RISK-029 | Availability data becomes stale | H | High | As-of/freshness/owner, reservations, refresh/confirmation, baseline retained | Operations owner | Readiness output uses expired/unconfirmed state | 2026-07-22 | M2–M7 | Open |
| RISK-030 | Capacity work expands into ERP/scheduling replacement | H | High | Explicit decision-support boundary and deferred scheduler | Product owner | Backlog adds dispatching/MRP/master scheduling to core slice | 2026-07-22 | M3–M7 | Open |
| RISK-031 | Advanced analyses overwhelm normal estimating UX | H | High | Layered views, progressive disclosure, role/usability tests | UX lead | Core-task success/time materially degrades | 2026-07-22 | M2–M7 | Open |
| RISK-032 | Integration transmits controlled data | M | Critical | Per-adapter data-flow ADR, local/manual default, authorization and egress tests | Security owner | Technical data reaches unapproved endpoint | 2026-07-22 | M3–M8 | Open |
| RISK-033 | Historical quote is silently recalculated by new engines or current rate/policy versions | M | Critical | Immutable snapshots with pinned rate/selector/pricing/rounding versions and explicit new-revision adoption | Domain lead | Issued artifact/result changes after upgrade or rate update | 2026-07-29 | M1–M7 | Open |
| RISK-034 | Optimization/simulation/comparison computation is excessive | M | High | Search/sample/resource bounds, caching, progress, cancellation | Architecture lead | Interactive workflow exceeds approved budget or starves UI | 2026-07-22 | M3–M7 | Open |
| RISK-035 | Users treat scores/recommendations as authoritative | H | High | Explainability, alternatives, warnings, approvals and override audit | Product owner | Decision is accepted without reviewing blockers/evidence | 2026-07-22 | M3–M7 | Open |
| RISK-036 | External benchmark encourages uncontrolled marketplace upload | M | Critical | Manual benchmark entry; no model upload without approved data-flow | Security/product | UI or adapter sends model/derived geometry automatically | 2026-07-22 | M2–M7 | Open |

Probability is provisional until evidence review. Customer visibility and commercial impact are captured per estimate-level risk; this register tracks product delivery risk.
