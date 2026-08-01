# Functional Requirements

> **Status:** In Review
> **Last updated:** 2026-08-01
> **Related requirements:** REQ-F-001–REQ-F-065
> **Related ADRs:** ADR-0001–ADR-0014
> **Open questions:** OQ-001–OQ-050
> **Dependencies:** Shop review
> **Supersedes:** None

| ID | Requirement | Initial acceptance evidence | Status |
|---|---|---|---|
| REQ-F-001 | Users shall create and revision customers, RFQs, parts, input packages, estimates, and quotes with explicit lifecycle states. | TEST-007 persistence workflow | Draft |
| REQ-F-002 | The system shall import STEP, STL, and 3MF in the first vertical slice, preserve source bytes/hash, detect format, and record parser/version diagnostics. | TEST-003 format corpus | Draft |
| REQ-F-003 | The system shall extract or request model units, require confirmation when ambiguous, preserve scale decisions, and block authoritative downstream results when unresolved. | TEST-003 unit fixtures | Draft |
| REQ-F-004 | The system shall expose versioned geometry facts, validity/watertightness, warnings, confidence reasons, and review state. | TEST-003 geometry golden tests | Draft |
| REQ-F-005 | The system shall calculate material mass and propose editable stock alternatives with allowances, removed volume, evidence, cost, and confidence. | TEST-002 calculation + TEST-003 geometry; partial GUI-2 stock/mass/removed-volume composition | Draft |
| REQ-F-006 | The system shall propose editable process class, setup count/orientations, routing operations, machines, and alternatives without presenting them as production truth. | TEST-005 runtime/routing fixtures | Draft |
| REQ-F-007 | The system shall estimate cutting and non-cutting time using an identified method and versioned feeds/speeds/rate inputs. | TEST-005 runtime golden tests; partial GUI-2 explicit cycle-time/rate composition | Draft |
| REQ-F-008 | The system shall estimate itemized internal costs, explicit risk, and independently calculated quantity scenarios using deterministic traceable calculations; missing or ambiguous applicable rates shall yield `Unavailable` or `Blocked`, never zero. | TEST-001/002/014 calculation and rate-resolution tests; GUI-2 headless draft-service evidence | Draft |
| REQ-F-009 | The system shall apply a pinned versioned pricing and rounding policy while visibly distinguishing cost, risk, markup, margin, thresholds, unrounded value, rounded value, and selling price. | TEST-002 pricing/rounding examples; GUI-2 pinned policy/result trace | Draft |
| REQ-F-010 | The system shall permit correction of every generated assumption or selected rate and preserve original/new values, actor, time, reason, authorization, and source versions. | TEST-006/014 override/re-analysis tests | Draft |
| REQ-F-011 | The system shall generate internal estimate and customer quote previews from the same approved revision without exposing configured internal-only details. | TEST-009 report golden tests | Draft |
| REQ-F-012 | The system shall record actuals, compare category-level variance against immutable original estimates, and require approval before updating defaults. | TEST-010 calibration workflow | Draft |
| REQ-F-013 | The system shall capture drawing, specification, contract, quality, and customer requirements separately from geometry-derived facts. | TEST-006 requirements review | Draft |
| REQ-F-014 | The system shall support deliver, make, spare/destructive-test quantities and independently calculated quantity breaks. | TEST-002 quantity fixtures | Draft |
| REQ-F-015 | The system shall maintain versioned material definitions and time-bounded supplier offers without hard-coded shop catalogs or numeric commercial defaults. | TEST-007/014 library persistence | Draft |
| REQ-F-016 | The system shall maintain machine physical capability separately from user-governed, effective-dated financial rates. | TEST-006/007/014 workcenter tests | Draft |
| REQ-F-017 | The system shall maintain versioned tool assemblies and feeds/speeds with source, approval, and effective date. | TEST-005 library fixtures | Draft |
| REQ-F-018 | The system shall estimate inspection, documentation, FAI/CMM, outside-process, freight, fixture, tooling, and administrative effort as visible categories with explicit rate/offer source and basis where applicable. | TEST-002/014/017 worked estimates | Draft |
| REQ-F-019 | The system shall identify unresolved assumptions and blocking decisions before approval. | TEST-006 approval tests | Draft |
| REQ-F-020 | The system shall support bid/no-bid, technical review, commercial review, issue, revision, expiry, win/loss, and withdrawal states. | TEST-006 lifecycle tests | Draft |
| REQ-F-021 | Intake shall identify and hash model files, preserve source/revision, and record declared versus detected format. | TEST-021–TEST-023 | Draft |
| REQ-F-022 | Analysis shall produce representation-aware geometry facts, warnings, confidence reasons, and versioned snapshots. | TEST-024–TEST-030 | Draft |
| REQ-F-023 | Recognition shall produce editable feature candidates with evidence, stable snapshot-scoped references, and alternatives. | TEST-031–TEST-036 | Draft |
| REQ-F-024 | Tool access, process, and setup analysis shall remain reviewable proposals linked to geometry evidence. | TEST-037–TEST-039 | Draft |
| REQ-F-025 | The user shall inspect synchronized selection among model, feature, setup, operation, tool, cost, and warning views. | TEST-012 UI workflow | Draft |
| REQ-F-026 | The user shall accept, reject, edit, merge, split, and manually define feature candidates without deleting automatic evidence. | TEST-004/012 | Draft |
| REQ-F-027 | The user shall compare alternative stock/process/setup approaches and their cost/time/risk effects. | TEST-002/006/012 | Draft |
| REQ-F-028 | The system shall capture risk category, probability/impact, mitigation, owner, visibility, acceptance, and resolution. | TEST-002/006 | Draft |
| REQ-F-029 | The system shall support inspection frequency, destructive samples, certificates, traceability, and quality-document effort. | TEST-002/006 | Draft |
| REQ-F-030 | The system shall record outside-vendor offer, specification, charges, freight, lead time, expiry, approval, and alternatives. | TEST-002/007 | Draft |
| REQ-F-031 | Users shall search, batch edit, use templates, undo/redo, autosave drafts, and recover from validation/import failures. | TEST-012 | Draft |
| REQ-F-032 | Users shall create and manage effective-dated, versioned shop rate/pricing libraries under authorization and approval policy; installation shall provide structure but no numeric production defaults. | TEST-006/007/011/012/014 | Draft |
| REQ-F-033 | The system shall preserve attachment integrity, revision lineage, classification, and controlled access/export metadata. | TEST-007/011 | Draft |
| REQ-F-034 | The system shall support explicit approval thresholds and recorded exceptions for rate overrides, price, margin, and risk. | TEST-006/011/014 | Draft |
| REQ-F-035 | The system shall preview a model and sanitized derivatives without making rendering the geometry authority. | TEST-012/024 | Draft |
| REQ-F-036 | The system shall expose analysis progress and recoverable stage failures without discarding prior valid results. | TEST-024/012 | Draft |
| REQ-F-037 | The system shall compare source/model/estimate/quote revisions and show adopted versus retained decisions. | TEST-006/007 | Draft |
| REQ-F-038 | The system shall export approved, versioned structured data only through authorized and audited boundaries. | TEST-009/011 | Draft |
| REQ-F-039 | The system shall render internal and customer-facing quote artifacts with template/version provenance and leakage controls. | TEST-009 | Draft |
| REQ-F-040 | The system shall represent multiple versioned routing alternatives for one part, including internal, mixed, outsourced, and decline options. | TEST-040–TEST-042 | Draft |
| REQ-F-041 | Users shall compare, filter, and score routing alternatives by cost, cash cost, lead time, risk, confidence, capacity, bottleneck use, contribution, tooling/fixture reuse, and production context. | TEST-040–TEST-043 | Draft |
| REQ-F-042 | A routing optimizer shall explain generation, feasibility filters, rejection reasons, score components, assumptions, and alternatives; only a user-approved route becomes active. | TEST-041–TEST-044 | Draft |
| REQ-F-043 | The system shall model effective-dated capacity calendars and availability for machines, operators, programmers, inspection/CMM, outside vendors, material, tools, and fixtures. | TEST-045–TEST-048 | Draft |
| REQ-F-044 | The system shall calculate delivery feasibility and capacity consumption while distinguishing touch, occupancy, operator, setup, programming, inspection, queue, calendar, and bottleneck time. | TEST-045–TEST-049 | Draft |
| REQ-F-045 | The system shall expose contribution and opportunity-cost metrics without changing accounting, incremental, fully burdened, risk-adjusted cost, or selling price. | TEST-046–TEST-050 | Draft |
| REQ-F-046 | When schedule/capacity data is missing or stale, the system shall retain the baseline estimate, mark capacity outputs unavailable or provisional, and identify required refresh/confirmation. | TEST-047–TEST-049 | Draft |
| REQ-F-047 | The system shall represent uncertain inputs with source, range/distribution, correlation assumption, provenance, review state, and applicable estimate scope. | TEST-051–TEST-053 | Draft |
| REQ-F-048 | The system shall produce explainable minimum/most-likely/expected and configured percentile/scenario outputs, dominant sensitivities, and deterministic baseline side by side. | TEST-051–TEST-055 | Draft |
| REQ-F-049 | The system shall compare immutable model revisions by source, geometry, features, access, stock, setup, tool, runtime, inspection, risk, cost, price, and lead-time changes. | TEST-056–TEST-059 | Draft |
| REQ-F-050 | Revision review shall provide side-by-side, ghosted, and added/removed visual evidence where supported and explain accepted cost deltas in manufacturing terms. | TEST-056–TEST-060 | Draft |
| REQ-F-051 | The system shall maintain stable manufacturing-requirement records with source/location/applicability, interpretation/verification, impacts, operation/cost links, owner, clarification, and approval. | TEST-061–TEST-064 | Draft |
| REQ-F-052 | Requirement coverage shall distinguish recognized-and-costed, recognized-not-costed, clarification, not-applicable, deferred, conflict, and missing-source states. | TEST-061–TEST-064 | Draft |
| REQ-F-053 | Quote readiness shall block a complete representation when critical requirements remain unresolved unless an authorized, reasoned, audited exception is recorded. | TEST-063–TEST-065 | Draft |
| REQ-F-054 | The system shall import supported structured PMI/MBD as revision-scoped evidence, distinguishing semantic from graphical PMI and requiring applicability/completeness/user validation. | TEST-066–TEST-069 | Draft |
| REQ-F-055 | The viewer shall map features to operations, setups, tools, machines, time, cost, inspection, risk, confidence, accessibility, revision deltas, and removal intensity without inventing unavailable allocation precision. | TEST-070–TEST-073 | Draft |
| REQ-F-056 | The system shall capture structured estimator corrections with automatic and approved values, actor/time/reason, model/algorithm/shop versions, and quote-specific versus reusable scope. | TEST-074–TEST-076 | Draft |
| REQ-F-057 | The system may propose evidence-backed, versioned rule/default improvements from correction cohorts, but shall require review, approval, comparison, and rollback and shall never update approved settings automatically. | TEST-075–TEST-078 | Draft |
| REQ-F-058 | The system shall import CAM-neutral or adapter-supplied setup/operation/tool/stock/timing metadata and reconcile estimate, CAM plan, machine result, and final actual without a core dependency on one vendor. | TEST-079–TEST-083 | Draft |
| REQ-F-059 | CAM/actual variance shall be categorized by geometry, recognition, setup, tool, feeds/speeds, runtime, programmer, machine, operator, quality, material, rework, vendor, and schedule causes. | TEST-080–TEST-084 | Draft |
| REQ-F-060 | Availability-aware estimation shall preserve both the theoretical baseline and a current readiness-adjusted snapshot with source, as-of time, freshness, reservations, restrictions, cost, and lead-time effects. | TEST-085–TEST-089 | Draft |
| REQ-F-061 | Users shall compare make, buy, mixed-operation outsourcing, near-net/blanking, commercial-component, customer-mandated supplier, benchmark, and decline alternatives using economic, capacity, quality, schedule, and data-handling evidence. | TEST-090–TEST-093 | Draft |
| REQ-F-062 | External marketplace prices shall be manually enterable as sourced benchmarks; controlled files or derived technical data shall not be transmitted automatically. | TEST-091–TEST-094 | Draft |
| REQ-F-063 | Bid/no-bid decisions shall expose configurable factor scores, weights, reasons, blockers, required approvals, recommendation, and override without replacing management judgment. | TEST-095–TEST-098 | Draft |
| REQ-F-064 | Quote-priority scoring shall distinguish win likelihood, strategic value, contribution, bottleneck efficiency, delivery feasibility, quote effort, technical/security risk, and cash-flow evidence. | TEST-095–TEST-099 | Draft |
| REQ-F-065 | Every advanced result shall pin input/configuration and algorithm versions, preserve approved historical snapshots, and require explicit adoption into a quote revision. | TEST-044, TEST-050, TEST-055, TEST-060, TEST-065, TEST-069, TEST-078, TEST-084, TEST-089, TEST-099 | Draft |

No functional requirement is Complete; executable evidence does not yet exist.
