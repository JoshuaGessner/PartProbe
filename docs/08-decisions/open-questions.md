# Open Questions

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** All  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Shop, security, legal, and technical review  
> **Supersedes:** None

## Shop and process

- **OQ-001:** Which machines/workcenters and physical capabilities are current?
- **OQ-002:** Which operations are internal versus outsourced, and who validates routing?
- **OQ-003:** Which CAD formats/revisions occur and at what frequency?
- **OQ-004:** Are assemblies quoted, or primarily single detail parts?
- **OQ-005:** How often are STEP versus mesh files available?
- **OQ-006:** What real unit/scale/import errors recur?
- **OQ-007:** Which unsupported entities/healing failures are common?
- **OQ-008:** Which proprietary formats justify translator cost?
- **OQ-009:** Which materials/forms/specifications and supplier restrictions dominate?
- **OQ-010:** Which tool brands/families and tool-life practices are used?
- **OQ-011:** Are approved shop feeds/speeds documented, and who owns changes?
- **OQ-012:** How are machine, labor, setup, run, and burden rates calculated?
- **OQ-013:** When is unattended time credited, and how is overlap costed?
- **OQ-014:** How are material yield, remnants, minimums, freight, and certificates costed?
- **OQ-015:** Does the shop use markup, margin, floors, or customer-specific policy?
- **OQ-016:** How are scrap, rework, risk, and contingency separated?
- **OQ-017:** Which inspection/documentation costs and quality clauses recur?
- **OQ-018:** Who approves technical assumptions, pricing, overrides, and exceptions?
- **OQ-019:** Which ERP/CAM/QMS/purchasing/time systems and export formats exist?
- **OQ-020:** Which historical quote/job actuals are complete enough for validation?

## Deployment, governance, and product

- **OQ-021:** Is offline or air-gapped operation required, and on which OS versions?
- **OQ-022:** Is team/LAN mode needed in the first release, and what identity system exists?
- **OQ-023:** Who may view each class of customer/defense technical data?
- **OQ-024:** What separation-of-duties and audit-retention rules apply?
- **OQ-025:** What customer quote/PDF format, terms, and revision controls are required?
- **OQ-026:** What data-retention, backup, restore, print/export, and deletion policy applies?
- **OQ-027:** What first-release accuracy/coverage target is acceptable by estimation method?
- **OQ-028:** How should confidence and blocking uncertainty be displayed to estimators?
- **OQ-029:** Which parts may become public/private validation fixtures and under what rights/classification?
- **OQ-030:** Which operations or requirements must never be inferred automatically?

## Advanced estimation, capacity, revision, and learning

- **OQ-031:** Which routing alternatives are common enough to generate and compare, and which must remain manual?
- **OQ-032:** Which resources are current versus long-term bottlenecks, and who designates them?
- **OQ-033:** What schedule, backlog, downtime, staffing, vendor, and reservation systems are available and how accurate/current are they?
- **OQ-034:** How does the shop define contribution, incremental/direct-cash/fully burdened cost, opportunity cost, and acceptable bottleneck economics?
- **OQ-035:** What capacity horizon, granularity, calendars, overtime/weekend, unattended, and parallel-resource policies are decision-useful?
- **OQ-036:** Which inputs should initially use ranges, who supplies them, and is three-point/scenario analysis sufficient before simulation?
- **OQ-037:** Which uncertainty ranges/percentiles and sensitivity explanations will estimators and approvers actually use?
- **OQ-038:** What geometric/revision comparison tolerance constitutes a meaningful manufacturing change by process/format?
- **OQ-039:** Should revision cost deltas hold historical rates constant, refresh current rates, or display both?
- **OQ-040:** What is the shop/customer-specific priority among model, semantic PMI, drawing, PO/RFQ clauses, specifications, portals, and user interpretation when sources conflict?
- **OQ-041:** Which unresolved requirements block quote readiness, and who may grant which documented exceptions?
- **OQ-042:** How often is usable STEP AP242 semantic or graphical PMI received, and what MBD workflows/customers require it?
- **OQ-043:** Which translator/kernel versions preserve needed PMI, under what licenses and packaging constraints?
- **OQ-044:** What correction-event cohort size, segmentation, privacy safeguards, reviewer roles, and rollback policy govern reusable rule suggestions?
- **OQ-045:** Which CAM systems, versions, setup sheets, reports, APIs, post metadata, and simulation outputs exist?
- **OQ-046:** Which CAM/machine/actual data may enter PartProbe, and which integrations may transmit technical data under each deployment profile?
- **OQ-047:** What systems own tool, fixture, material, machine, personnel, and vendor availability; who owns freshness and reservations?
- **OQ-048:** Which sourcing alternatives, suppliers, quality restrictions, data-transfer constraints, and manual marketplace benchmarks are allowed?
- **OQ-049:** Which bid/no-bid and quote-priority factors, weights, blockers, approvals, and override roles reflect management policy?
- **OQ-050:** Which advanced metrics belong in customer output, internal approval only, or restricted management views?

Do not block all work on unanswered questions; use visible assumptions and validation tasks. Do block approval where an answer materially affects safety, legality, data handling, or financial integrity.
