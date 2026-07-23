# Test Strategy

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All; TEST-001–TEST-099  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** Hardware/performance and accuracy thresholds  
> **Dependencies:** Executable workspace and fixtures  
> **Supersedes:** None

Current executable evidence: [TASK-001 validation](task-001-validation.md) covers the local TEST-001 foundation. All other test IDs remain planned, and the first Windows/Linux CI evidence is pending.

| ID | Layer | Core evidence |
|---|---|---|
| TEST-001 | Unit/property | Money, units, rules, invariants, serialization |
| TEST-002 | Calculation golden | Reviewed worked estimate inputs and exact reconciled traces |
| TEST-003 | Geometry golden | Units, dimensions, volume/area, validity, warnings, failure corpus |
| TEST-004 | Feature recognition | Expected/ambiguous detections, false positives/negatives, confidence |
| TEST-005 | Runtime golden | Method, material/machine/tools, cutting/non-cutting, error band |
| TEST-006 | Workflow/domain | Overrides, approvals, re-analysis protection, lifecycle invariants |
| TEST-007 | Persistence | Migrations, concurrency, save/reopen, backup/restore, corruption |
| TEST-008 | Platform/package | Windows/Linux/macOS build, launch, signing/update staging |
| TEST-009 | Reporting | Internal/customer snapshot and leakage review |
| TEST-010 | Actuals/calibration | Variance, cohort bias, authorization, preserved estimate |
| TEST-011 | Security | Permissions, paths, malformed input, logs, isolation, update integrity |
| TEST-012 | UI/accessibility | Keyboard/focus, semantics, contrast, scaling, viewport linkage, HiDPI |
| TEST-013 | Units | Cross-system conversions, dimensional mismatch, ambiguous/missing units |
| TEST-014 | Shop libraries | Effective dating, approval, source, selection, and prior-version replay |
| TEST-015 | Routing | Operation order, setup/lot basis, source, alternatives, adoption, manual preservation |
| TEST-016 | Risk/approval | Category impacts, acceptance, thresholds, blockers, authorization |
| TEST-017 | Outside/quality | Vendor expiry/minimums/freight and FAI/CMM/documentation category reconciliation |
| TEST-018 | Import/export DTO | Schema compatibility, unknown/hostile fields, manifest/checksum, idempotency |
| TEST-019 | Performance/resource | Representative percentiles, memory, limits, cancellation, recovery |
| TEST-020 | Dependency/release provenance | Locks, license/SBOM, signing, update/rollback, offline package |
| TEST-021 | Intake identification | Hash, size, extension/signature mismatch, immutable source |
| TEST-022 | Unit resolution | Declared/absent/conflicting units, scale confirmation and blocking behavior |
| TEST-023 | Parser limits | File/entity/triangle/CPU/memory/depth/output limits and sanitized failures |
| TEST-024 | Exact geometry | STEP transfer, body/shell/validity, AABB/OBB, volume/area/centroid tolerances |
| TEST-025 | Mesh geometry | Watertight/manifold/degenerate/orientation, approximate volume/area, confidence ceiling |
| TEST-026 | Healing | Pre/post derivative hashes, action record, measurement delta, source preservation |
| TEST-027 | Stock/orientation | Enclosure, shape/dimensions/orientation/allowance, removed-volume guards |
| TEST-028 | Viewer mapping | Tessellation tolerance, stable picking references, overlays, clipping, large-model behavior |
| TEST-029 | Native packaging | Kernel/worker/render install/load/launch on every supported platform |
| TEST-030 | Geometry containment | Worker crash/hang/OOM/malformed corpus, cancellation, restart, output validation |
| TEST-031 | Primitive recognition | Plane/cylinder/cone/rotational candidates and evidence |
| TEST-032 | Hole recognition | Through/blind/counterbore/countersink/spotface and thread-candidate limits |
| TEST-033 | Prismatic recognition | Pockets/open pockets/slots/steps/bosses/chamfers/fillets/thin walls |
| TEST-034 | Turning recognition | OD/ID/face/shoulder/groove/bore/taper/cutoff/live-tool indicators |
| TEST-035 | Access/advanced indicators | Undercut, multi-direction, five-axis, EDM/grind/custom-tool ambiguity |
| TEST-036 | Candidate conflicts | Overlap, merge/split/edit/manual history and no evidence deletion |
| TEST-037 | Tool/process suggestions | Compatibility, missing evidence, alternatives, independent rejection |
| TEST-038 | Setup proposals | Orientation/access/workholding limitations, comparison, explicit adoption |
| TEST-039 | Feature regression | Fixture false positives/negatives, confidence reasons/ceilings, version diff |
| TEST-040 | Routing alternatives | Reviewed parts with multiple plausible internal/mixed/outsource/decline routes |
| TEST-041 | Route feasibility | Machine/stock/access/requirement constraints and explicit rejection reasons |
| TEST-042 | Route ranking | Policy component/weight trace, deterministic ties, user filters and explanation |
| TEST-043 | Route compute limits | Search bounds, cancellation, caching, timeout and partial-result policy |
| TEST-044 | Route adoption/versioning | Only explicit adoption changes active route; prior approved route preserved |
| TEST-045 | Capacity calendars | Shift/downtime/backlog/resource interval and timezone/calendar cases |
| TEST-046 | Bottleneck economics | Contribution per spindle/occupancy/bottleneck hour and zero denominator |
| TEST-047 | Capacity data quality | Missing, stale, contradictory and low-confidence schedules |
| TEST-048 | Parallel/overtime capacity | Parallel resources, attendance overlap, overtime/weekend/unattended policies |
| TEST-049 | Delivery feasibility | Precedence, queue, material/vendor lead and impossible-date cases |
| TEST-050 | Cost-basis separation | Accounting/incremental/burdened/opportunity/risk/price never alias or double count |
| TEST-051 | Three-point inputs | Ordered bounds, units, sources, validation and manual/generated provenance |
| TEST-052 | Distribution methods | Triangular/PERT/scenario formulas against reviewed analytical cases |
| TEST-053 | Simulation reproducibility | Seed, sample count, correlation/independence and cross-platform tolerance |
| TEST-054 | Sensitivity/resources | Dominant-input ranking, convergence/limits, cancellation and cache keys |
| TEST-055 | Uncertainty invariants | P10≤P50≤P80≤P90, no impossible negative values, baseline preserved |
| TEST-056 | Identical revisions | Hash/same-geometry outcomes and no false cost change |
| TEST-057 | Revision alignment | Unit-only, transform, tolerance and alignment ambiguity cases |
| TEST-058 | Geometry revision | Small/large added-removed geometry and measurement/feature/access changes |
| TEST-059 | Topology/mesh revision | Unstable B-rep topology, tessellation differences, false-change tolerance and limits |
| TEST-060 | Revision adoption | Cost-cause trace, manual override/routing preservation and quote reapproval |
| TEST-061 | Requirement identity | Stable IDs, duplicates, source location and applicability/revision scoping |
| TEST-062 | Requirement conflicts | Conflicting sources, priority/evidence, customer clarification and resolution |
| TEST-063 | Coverage linkage | Requirement-to-operation/cost/inspection/security linkage and reconciliation |
| TEST-064 | Coverage missing states | Recognized-not-costed, missing source, deferred and unresolved-critical cases |
| TEST-065 | Quote readiness | Blocking, authorized exception, reason/audit and customer-visible limitation |
| TEST-066 | PMI format coverage | AP203/AP214/AP242 fixtures and unsupported/absent PMI behavior |
| TEST-067 | PMI semantics | Semantic versus graphical dimensions/GD&T/datums/finish mapping and geometry refs |
| TEST-068 | PMI quality | Partial/ambiguous/stale/revision-mismatched/applicability failures |
| TEST-069 | PMI approval | Source priority, human confirmation, requirement conversion and immutable evidence |
| TEST-070 | Feature cost overlays | Operation/setup/tool/time/cost/risk/confidence/revision mode mappings |
| TEST-071 | Allocation honesty | Coarse/unallocated/approximate feature cost labels and totals reconciliation |
| TEST-072 | Visualization mapping | Stable feature selection, heatmap legend/range and snapshot version |
| TEST-073 | Visualization accessibility | Text/table equivalent, keyboard selection and non-color-only states |
| TEST-074 | Correction capture | Every supported correction records old/new/actor/time/reason/version |
| TEST-075 | Correction scope | Quote-specific versus reusable classification and snapshot immutability |
| TEST-076 | Correction analytics | Repeated-pattern cohorts, segmentation and reproducible evidence |
| TEST-077 | Learning safeguards | Sparse/biased/individual/customer leakage, permissions and non-recommendation states |
| TEST-078 | Rule governance | Suggestion comparison, approval, version activation, rollback and no silent update |
| TEST-079 | CAM partial import | Missing/unknown fields, source hash/schema/version and recoverable parsing |
| TEST-080 | CAM mismatch | Setup/tool/stock/time differences and incomparable-term handling |
| TEST-081 | CAM adapter neutrality | Equivalent neutral contract across at least two adapter fixture shapes |
| TEST-082 | CAM mapping quality | Estimate↔CAM↔machine↔actual mapping, completeness and user reconciliation |
| TEST-083 | CAM boundary | Resource limits, untrusted reports/APIs, secrets, controlled-data and cancellation |
| TEST-084 | Variance causes | Category/cause assignment, review, multi-cause evidence and historical preservation |
| TEST-085 | Tool availability | Owned/loaded/reserved/order/regrind/life/holder/custom and stale states |
| TEST-086 | Material availability | On-hand/cert/remnant/lot/reservation/supplier/lead/country cases |
| TEST-087 | Fixture availability | Design/physical/compatibility/location/condition/damaged/reserved cases |
| TEST-088 | Resource/vendor readiness | Machine down, qualification/restriction, vendor approval/expiry/capacity/history |
| TEST-089 | Readiness adjustment | Theoretical baseline retained, explicit deltas, as-of/freshness and fallback |
| TEST-090 | Make-buy economics | Incremental/burdened/cash/landed/risk/capacity/opportunity comparisons |
| TEST-091 | Supplier evidence | Quote expiry/minimum/freight/cert/lead and manual external benchmark entry |
| TEST-092 | Mixed sourcing | Partial operation, near-net/blank, commercial component and mandated supplier |
| TEST-093 | Sourcing security | Controlled-data restriction, no automatic marketplace upload, approved transfer |
| TEST-094 | Sourcing decision | Explainable selection/decline, approvals, override and quote versioning |
| TEST-095 | Bid factor model | Strategic/economic/delivery/quality/security/effort/win-probability inputs |
| TEST-096 | Bid blockers | Capability, data, compliance/security, schedule and approval blockers |
| TEST-097 | Bid policy version | Weights/scales/normalization/version and sensitivity to missing factors |
| TEST-098 | Bid judgment | Recommendation reasons, management override, audit and no auto-decline/approve |
| TEST-099 | Quote priority | Deterministic cohort ranking, tie/stale-data behavior and contribution basis |

## Policy

Fast pure tests run per change; integration/golden tests run per PR; platform/package/security/performance suites run on scheduled and release gates. Golden changes require an explained review, never blind regeneration. Flaky tests are quarantined only with owner, issue, and expiry. Sensitive customer data is never test data.
