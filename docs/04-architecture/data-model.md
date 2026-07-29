# Initial Data Model

> **Status:** In Review
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-001–REQ-F-065; DATA-001–DATA-042
> **Related ADRs:** ADR-0006–ADR-0014
> **Open questions:** OQ-019–OQ-024
> **Dependencies:** Persistence spike and domain review
> **Supersedes:** None

| ID | Aggregate/record | Required invariants |
|---|---|---|
| DATA-001 | Customer/contact | Stable ID; effective-dated policies; no quote history cascade delete |
| DATA-002 | RFQ/input package | Revision, requested quantities/dates, attachments, classification, hashes |
| DATA-003 | Part/part revision | Stable part identity; immutable source revisions; parent/assembly links optional |
| DATA-004 | Model analysis snapshot | Source hash, units/scale/healing, body-level representation basis, per-measurement derivation/tolerance/confidence ceiling and reasons, stage results, warnings, algorithm versions, approval |
| DATA-005 | Material/offer/stock | Definition version separate from supplier offer/effective date |
| DATA-006 | Machine/workcenter/rate card | Capability separate from finance; typed organization/site/cost-center/workcenter/machine/operation/labor-class scope; amount/currency/basis; effective interval; approval and immutable versions |
| DATA-007 | Routing/setup/operation | Ordered revision; provenance; all time/cost categories; override history |
| DATA-008 | Estimate/quantity scenario | Immutable published revision; calculation graph/result references; pinned rate entry/card, selector, pricing and rounding-policy versions |
| DATA-009 | Risk/requirement | Category, evidence, impact, owner, acceptance/resolution/customer visibility |
| DATA-010 | Quote/quote revision | References approved estimate; template/version; issued artifact hash |
| DATA-011 | Audit event | Append-preserving actor/time/action/object/context/result and before/after references; sensitive-safe details for rate/policy/override decisions |
| DATA-012 | Actuals/calibration proposal | Links original versions; category variance; cohort; authorization |
| DATA-013 | Attachment/document asset | Source/derivative hashes, media/format, revision, classification, locator, integrity state |
| DATA-014 | Unit/healing/import record | Declared/detected/confirmed units, scale, parser transfer, healing actions, warnings |
| DATA-015 | Feature analysis snapshot | Geometry snapshot, recognizer versions, candidates/evidence/review lineage, source-body/measurement representation basis and enforced confidence ceiling |
| DATA-016 | Setup/process alternative | Orientation/access/workholding assumptions, routing proposal, confidence, rejection/adoption |
| DATA-017 | Version registry | Calculation, geometry, feature, runtime, rate-card/entry/selector, pricing, rounding, feed/speed, schema, and template versions |
| DATA-018 | Confidence/warning/review decision | Dimension/reason chain, severity/blocking state, actor/time/reason/resolution |
| DATA-019 | Routing alternative/set | Immutable generated/manual route, source/config versions, feasibility, adoption/rejection, parent estimate revision |
| DATA-020 | Routing comparison | Candidate snapshot IDs, comparison policy/version, metrics, filters, score trace, selected route and approval |
| DATA-021 | Capacity calendar/snapshot | Resource, shifts/downtime/backlog/as-of/freshness, source, confidence and access classification |
| DATA-022 | Capacity reservation/demand | Alternative/operation resource intervals, occupancy/touch/parallel/precedence basis, status |
| DATA-023 | Bottleneck designation | Resource or class, effective interval, short/long-term reason, owner and approval |
| DATA-024 | Profitability/opportunity metric | Cost basis, contribution, denominator resource/time, displacement scenario, policy/version |
| DATA-025 | Uncertain input | Target node, bounds/distribution, source, units, correlation group/assumption, provenance, review |
| DATA-026 | Estimate distribution/scenario/sensitivity | Baseline, method/seed/sample/version, percentiles/scenarios, dominant inputs, warnings |
| DATA-027 | Model revision comparison | Immutable old/new snapshot IDs, alignment/tolerance/method/version, geometry/feature/access changes |
| DATA-028 | Cost delta explanation | Old/new calculation nodes, fixed/current rate basis, category delta, causal evidence, approximation state |
| DATA-029 | Manufacturing requirement/source/coverage | Stable requirement and source location/applicability, interpretation/verification, impacts, links, owner, readiness state |
| DATA-030 | PMI record | Semantic/graphical kind, AP/schema/importer/revision, geometry reference, completeness/applicability/confirmation |
| DATA-031 | Estimator correction event | Automatic/approved values, actor/time/reason, model/algorithm/shop versions, reusable scope |
| DATA-032 | Rule suggestion/approval | Cohort/evidence, proposed diff, bias/fairness review, approvers, effective version, rollback state |
| DATA-033 | CAM import/operation | Source adapter/schema/hash, setups/tools/stock/timing/notes, completeness, mapping and classification |
| DATA-034 | CAM reconciliation/variance | Estimate/CAM/machine/actual snapshot links, comparable terms, categorized deltas, review state |
| DATA-035 | Tool availability snapshot | Owned/loaded/reserved/order/regrind/holder/custom status, cost/lead/life/as-of/source |
| DATA-036 | Fixture availability snapshot | Design/physical asset/location/compatibility/condition/reservation/design-build-inspect effort |
| DATA-037 | Material availability snapshot | On-hand/certified/remnant/lot/reserved/supplier/lead/country/cert/as-of/source |
| DATA-038 | Resource/vendor readiness snapshot | Machine/personnel qualification/restriction/downtime and vendor approval/quote/capacity/performance/freshness |
| DATA-039 | Sourcing alternative/make-buy analysis | Make/buy/mixed/decline option, supplier evidence, cost bases, capacity, quality, schedule, security, adoption |
| DATA-040 | Bid decision | Factors, blockers, recommendation, required approvals, override, outcome and policy/version |
| DATA-041 | Quote-priority score | Factor values/weights, contribution/bottleneck basis, data quality, rank cohort, reasons and override |
| DATA-042 | Advanced-engine version manifest | Routing, setup/tool, capacity, opportunity, uncertainty, requirement, revision, learning, CAM, availability and scoring versions |

## Persistence conventions

Use opaque UUID-compatible IDs, UTC timestamps plus display timezone, explicit unit/currency codes, optimistic version numbers, soft lifecycle transitions rather than destructive deletion, and join records for many-to-many assignment. Store large source/render artifacts in a content-addressed document store; SQLite stores manifests and integrity metadata.

Snapshots use normalized searchable metadata plus a versioned serialized payload only where immutability and forward compatibility justify it. Representation basis is attached to each body, measurement, and feature evidence item; a mixed snapshot is only aggregate metadata and cannot raise an item's confidence above its own basis. Advanced records are grouped into bounded aggregates rather than automatically becoming one table per noun: routing comparison owns alternatives; capacity analysis references immutable resource snapshots; estimate analysis owns uncertainty/scenarios; part revision owns comparison evidence; estimate revision owns requirement coverage; governance owns correction proposals; integration owns CAM imports/reconciliations. Persistence rows are not public interchange DTOs.

TASK-003 provides a preproduction in-memory DATA-013 contract for governed worker derivatives: artifact/source revisions, derivative reference, classification, access/retention policy references, authorization correlation, schema/media type, verified SHA-256/length, actor/time, and opaque locator. This is partial type-level evidence only. TASK-006 must define the persisted schema, migration, integrity-on-open, retention/disposition, and backup/restore behavior before DATA-013 can be treated as implemented.
