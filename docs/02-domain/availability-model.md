# Availability Model

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-060, REQ-F-065; REQ-NF-017–REQ-NF-021; TEST-085–TEST-089  
> **Related ADRs:** ADR-0006, ADR-0007  
> **Open questions:** Inventory/schedule systems, freshness owners, reservation authority, and acceptable stale-data thresholds  
> **Dependencies:** Shop-library records, integration policy, permissions, and source-system discovery  
> **Supersedes:** None

## Purpose and boundary

Availability describes whether a theoretically capable resource is ready for a proposed routing at a stated time. It does not change machine, tool, material, fixture, personnel, or vendor capability. It is quote-time evidence, not a production reservation, finite-capacity schedule, ERP inventory balance, or promise that a resource will still be available when the job is won.

Every comparison keeps two named views:

- **Baseline theoretical estimate:** uses approved capability, standard sourcing, and normal lead-time assumptions without claiming current readiness.
- **Readiness-adjusted estimate:** uses an identified availability snapshot and shows additions, substitutions, lead-time effects, cash costs, and blockers.

The application never silently replaces the baseline with the readiness-adjusted view. An approved estimate pins both views or explicitly records why only the baseline was available.

## Aggregate and records

`AvailabilitySnapshot` is the aggregate boundary for a project/RFQ, site, requested-delivery scenario, and observation time. It contains immutable observations rather than copying mutable source-system rows.

| Record | Required content |
|---|---|
| `AvailabilitySnapshot` | stable ID, project/site, requested window, captured/effective time, source-set/config versions, snapshot state, classification, owner, and content digest |
| `AvailabilityObservation` | resource reference, observed status and quantity/capacity, unit, location, time basis, source/evidence, captured time, expiry/freshness policy, confidence, warnings, and any reservation reference |
| `ToolAvailability` | owned/available/loaded/assigned/order/regrind/custom status, assembly/holder compatibility, quantity, condition, tool-life basis, location, lead time, and incremental cost |
| `FixtureAvailability` | design and physical-asset identities, compatibility, condition, storage location, reservation, required design/build/inspection work, lead time, and cost |
| `MaterialAvailability` | form/dimensions/quantity, certified status, heat/lot/country evidence, remnant identity, reservation, supplier offer, minimum, lead time, expedite option, and expiry |
| `MachineAvailability` | machine/workcenter, calendar window, maintenance/down state, qualification/restrictions, reservation/source, and observed capacity; capability remains elsewhere |
| `PersonnelAvailability` | role/qualification/familiarity and aggregate availability window; avoid unnecessary personal data |
| `VendorAvailability` | approved scope, offer/quote ID, effective/expiry dates, quoted lead time/capacity, expedite option, performance-evidence period, restrictions, and current status |
| `ReadinessAdjustment` | baseline component, adjusted component, cause, evidence observation, cost/time/risk delta, confidence, and user decision |

An observation may reference a source record without importing all source-system content. Source authority, mapping version, polling/import time, and conflict policy are mandatory. Derived rollups cannot be fresher or more authoritative than their inputs.

## Freshness and missing-data semantics

Freshness is configured per resource/source because a machine-down signal, a vendor quote, and certified stock have different useful lives. Each observation evaluates to one of:

- `Current`: within the approved freshness interval and no newer contradictory evidence is known.
- `Aging`: still usable under policy but visibly approaching expiry.
- `Stale`: older than policy permits for an authoritative readiness claim.
- `Unknown`: never observed, incomplete, or source unavailable.
- `Conflicted`: two sources or observations disagree and authority rules cannot resolve them.

`Stale`, `Unknown`, and `Conflicted` are not numeric zero and do not imply unavailable. They produce a visible assumption, confidence reduction, and—when policy marks the resource critical—a readiness or quote-approval blocker. The UI shows `as of`, source, owner, and next refresh action. A manual observation requires actor, time, reason, evidence, and expiry.

If no usable current data exists, the baseline estimate remains calculable. The readiness-adjusted estimate becomes `Incomplete` or `Blocked`, lists missing inputs, and must not claim delivery feasibility. Authorized users may accept a bounded assumption for a quote revision; the acceptance is audited and does not make the underlying observation current.

## Lifecycle, ownership, and immutability

Snapshot lifecycle is `Draft → Validated → Frozen`, with `Superseded`, `Expired`, or `Quarantined` as later states. Validation checks units, resource identity, time basis, conflicts, classification, and source integrity. Published estimates reference a frozen snapshot; refresh creates a sibling snapshot and a comparison, never an in-place rewrite.

- Resource-library owners define capability and identity.
- Purchasing owns supplier/material/vendor evidence unless shop policy assigns another role.
- Manufacturing/operations owns machine, tool, fixture, and aggregate labor readiness.
- Quality owns qualification, certification, and approved-supplier restrictions.
- Integration owners own source mappings and freshness monitoring.
- Estimate approvers decide whether stale or missing evidence is acceptable for the quoted scenario.

Availability does not reserve inventory or capacity unless a separately authorized source system returns a durable reservation identifier. A local planning hold must be labeled advisory and cannot impersonate an ERP/MES reservation.

## Calculation and explainability rules

Readiness adjustments are typed calculation inputs with source snapshot and policy versions. They may add procurement, regrind, fixture design/build, qualification, expedite, substitution, or schedule effects, but cannot overwrite accounting cost, approved rate, capability, or the baseline trace. Opportunity cost and delivery feasibility remain separately labeled outputs.

For every changed result the user can trace:

`resource observation → freshness/policy decision → readiness action → cost/time/risk delta → affected route/estimate node`.

Substitution must revalidate compatibility, certification, customer restrictions, and routing assumptions. No substitute material, machine, tool, fixture, person, or vendor is selected without review.

## Security, retention, and audit

Availability inherits the highest applicable project/source classification. Supplier quotes, heat/lot evidence, personnel qualification, machine status, locations, and production-load information may be commercially or operationally sensitive. Enforce project/role/classification authorization at the service boundary; minimize personnel detail; exclude quantities, prices, paths, and customer content from diagnostics.

External refresh is disabled by default. Controlled-data profiles may import approved local files or query approved internal services, but cannot transmit model, drawing, requirement, or estimate content merely to check availability. Snapshot creation, refresh, manual correction, assumption acceptance, conflict resolution, reservation linkage, and export are audited. Retention follows the estimate/quote evidence policy; source credentials are never stored in snapshots.

## Staged delivery

- **Early foundation:** snapshot/value types, manual observations, provenance, freshness states, and separate baseline/readiness outputs.
- **Initial production:** basic stock and machine readiness; manual tool/fixture/vendor fields; visible stale/unknown behavior; no automatic reservation.
- **Intermediate:** read-only inventory, purchasing, and calendar adapters; comparison between snapshots; expiry notifications.
- **Advanced:** broader near-real-time adapters, governed reservation links, and full route-level readiness analysis.
- **Deferred/research:** finite-capacity scheduling, autonomous procurement, automatic rescheduling, and ERP/MES replacement behavior.

## Validation and acceptance

- **TEST-085:** owned tool is unavailable or assigned; baseline remains and adjusted plan explains order/regrind/substitution effects.
- **TEST-086:** certified stock is partly reserved or remnant dimensions do not enclose stock; no double allocation or false availability.
- **TEST-087:** fixture is damaged, machine is down, or qualification is missing; affected routing is blocked or explicitly assumed under policy.
- **TEST-088:** vendor quote expires and two source observations conflict; status becomes stale/conflicted with no silent fallback.
- **TEST-089:** snapshot refresh and migration preserve the approved quote's pinned snapshot, digest, trace, access policy, and cross-platform replay.

Acceptance requires golden cases for each resource class, time-zone/clock-boundary tests, unit/quantity invariants, source outage and partial-import cases, authorization/audit tests, and proof that stale or missing data is never coerced to zero or `Available`.

## Shop decisions required

1. Which inventory, crib, maintenance, scheduling, purchasing, HR/qualification, and vendor systems are authoritative?
2. Who owns refresh and exception decisions for each resource class, and what age is `Aging` or `Stale`?
3. Are any source reservations available, and may PartProbe create them or only reference them?
4. Which certification, heat/lot, country-of-origin, customer-approval, or personnel restrictions block substitution?
5. What availability evidence is required before a delivery date may be represented as feasible?

