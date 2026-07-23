# Sourcing and Make/Buy Model

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-061–REQ-F-062, REQ-F-065; REQ-NF-017–REQ-NF-021; TEST-090–TEST-094  
> **Related ADRs:** ADR-0006, ADR-0007, ADR-0014  
> **Open questions:** Sourcing authority, approved suppliers, cost policy, controlled-data destinations, and benchmark use  
> **Dependencies:** Routing, pricing, availability, security, requirement coverage, and supplier data  
> **Supersedes:** None

## Purpose and boundary

`MakeBuyAnalysis` compares manufacturing and sourcing alternatives for one RFQ/part/quantity/revision. It informs a human sourcing decision; it is not procurement authorization, an online marketplace client, or a replacement for supplier approval and purchase-order controls.

Supported alternatives may include complete in-house manufacture, complete outsource, selected-operation outsource, near-net blank, external waterjet/laser blank, casting/forging, commercial component, customer-mandated supplier, manually entered marketplace benchmark, and decline. Each alternative is a complete scenario: it cannot omit the retained in-house operations, receiving/inspection, freight, packaging, documentation, or customer restrictions surrounding outsourced work.

## Aggregate and records

| Record | Required content |
|---|---|
| `MakeBuyAnalysis` | stable ID, RFQ/part/estimate revision, quantity/date scenario, alternatives, comparison-policy version, current recommendation, review state, and approvals |
| `SourcingAlternative` | type, route/operation coverage, source and destination sites, stock/near-net basis, retained internal work, supplier/customer constraints, costs, lead-time, capacity usage, risks, confidence, and blockers |
| `SupplierOffer` | supplier and approval scope, quote/evidence hash, currency/unit, quantities/minimums, price breaks, tooling/NRE, freight/packaging, validity window, lead time, expedite, certifications, assumptions, and actor/source |
| `ManualBenchmark` | provider/category without transmitted technical data, lookup time, entered value/range, currency/quantity basis, user, public-input description, caveat, and expiry |
| `SourcingDecision` | selected/rejected alternatives, reasons, reviewer/approver, time, required concurrence, and any authorization to release data or issue procurement |

Alternatives are children of an analysis aggregate, not independent master data. Supplier identity, approval status, customer mandates, and reusable commercial offers remain versioned library records. An offer is evidence, not a permanent price.

## Comparison semantics

Every alternative exposes, without collapsing them into one unlabeled number:

- supplier or purchase cash cost;
- retained internal incremental and fully burdened cost;
- accounting cost and explicit risk-adjusted cost;
- opportunity cost and bottleneck consumption;
- freight, packaging, receiving, inspection, certification, tooling/NRE, and rework exposure;
- calendar lead time and schedule risk;
- internal capacity consumed/released;
- resale selling price, margin/markup, and approval thresholds;
- quality, supplier-approval, customer-mandate, security, and data-release constraints.

The comparison policy may rank eligible alternatives for objectives such as lowest cash cost, fastest delivery, lowest risk, or highest contribution per bottleneck hour. Eligibility blockers are evaluated before ranking. Missing or expired supplier evidence yields `Unknown`/`Stale`, not zero cost or immediate rejection. Opportunity cost never changes accounting cost. New formulas become authoritative only after addition to `calculation-rules.md`, versioning, worked examples, and tests.

The system recommends at most; a user approves the sourcing decision. Partial outsourcing must map each sourced operation and retained operation exactly once so cost and lead-time are neither omitted nor double counted.

## Controlled and proprietary data

The source package's classification and customer restrictions determine which suppliers, adapters, exports, and users may receive technical data. No external marketplace, supplier portal, email/API, cloud benchmark, or AI service receives CAD, drawings, specifications, requirements, prices, or estimate content by default.

If controlled or customer-proprietary data cannot be transmitted to a destination, the alternative may use:

1. an approved internal supplier offer already in scope;
2. a manual benchmark based only on non-sensitive public descriptors;
3. an authorized redacted request package with explicit review; or
4. `Unavailable` with the restriction shown as a blocker.

A manual benchmark must never be represented as a binding quote or supplier availability. Any export/request package requires destination policy, minimum-necessary scope preview, classification inheritance, authorization, audit, and a returned-offer linkage. The product makes no export-control or supplier-compliance determination.

## Lifecycle, versions, and audit

Analysis lifecycle is `Draft → Compared → Reviewed → Approved`, with `Superseded`, `Rejected`, or `Expired` states. Approved analyses and decisions are immutable. A new model, quantity, date, offer, availability snapshot, route, pricing rule, capacity policy, or security decision creates a new analysis revision or explicit refresh comparison.

Each alternative pins route, calculation, rate, pricing, availability, supplier-offer, exchange-rate, requirement, and policy versions. Preserve rejected alternatives and reasons so a later quote revision explains why the decision changed. Audit offer import/manual entry, mapping, data-release authorization, ranking, selection, override, expiry acceptance, and procurement handoff. Approval of the estimate does not itself authorize a purchase order.

## Ownership and approval

- Estimating owns scenario completeness and comparability.
- Manufacturing owns technical route feasibility and retained operations.
- Purchasing owns supplier offers and commercial sourcing evidence.
- Quality owns supplier approval/certification scope and incoming/outsourced inspection.
- Security/export-control authorities own classification and release policy where applicable.
- Commercial approvers own price/margin implications; designated management owns make/buy selection.

Conflicting ownership is a recorded blocker. Customer-mandated suppliers, special-process approvals, and quote-expiry exceptions require the concurrence configured by shop policy.

## Staged delivery

- **Early foundation:** manual alternatives, complete cost-category schema, evidence/expiry, restrictions, and reviewed decision.
- **Initial production:** manual make/buy comparison and manual benchmark entry with no external transmission.
- **Intermediate:** partial-operation mapping, availability/capacity context, read-only supplier/ERP imports, and structured comparison.
- **Advanced:** approved supplier adapters, alternative ranking, and governed request/response packages.
- **Deferred/research:** autonomous RFQ submission, public-marketplace price harvesting, automatic purchasing, and unrestricted cross-company exchange.

## Validation and acceptance

- **TEST-090:** full make, full buy, and partial buy alternatives reconcile all operations and cost categories exactly once.
- **TEST-091:** expired/partial/multi-currency supplier offers remain visibly stale or blocked; no absent value becomes zero.
- **TEST-092:** supplier approval, customer mandate, certification, and quality restrictions remove ineligible options before ranking and remain explainable.
- **TEST-093:** controlled-data policy blocks external transmission, while a manual non-sensitive benchmark remains available and clearly non-binding.
- **TEST-094:** approval, override, refresh, export, and revision audit preserve old alternatives, evidence hashes, versions, and permissions.

Golden scenarios must cover near-net blanks, minimum order, freight, incoming inspection, external blank plus internal finish, one operation outsourced, supplier delay, and decline. Acceptance also requires access-control/export tests and proof that a selected option cannot silently mutate the routing or issue procurement.

## Shop decisions required

1. Who may request supplier quotes, release technical packages, approve suppliers, select make/buy, and authorize purchase orders?
2. Which suppliers/processes are customer-mandated or approved only for specific specifications?
3. Which internal-cost, opportunity-cost, freight, risk, and resale-margin policies belong in the comparison?
4. Which marketplace or benchmark sources are permitted, and what non-sensitive descriptors may be entered?
5. How should foreign currency, taxes/duties, quote expiry, minimums, split quantities, and supplier performance be treated?

