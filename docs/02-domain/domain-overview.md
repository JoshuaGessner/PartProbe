# Domain Overview

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001–REQ-F-065; DATA-001–DATA-042  
> **Related ADRs:** ADR-0006–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** Shop discovery  
> **Supersedes:** None

## Aggregate boundaries

- **RFQ:** customer request, input package, requested quantities/dates, bid/no-bid history.
- **Part revision:** identity, source files, hashes, unit decisions, immutable analysis snapshots.
- **Estimate revision:** quantity scenarios, proposed routing, requirements, risks, calculations, overrides, approval state.
- **Shop library:** versioned material, tool, machine, rate-card, feeds/speeds, vendor, and template records.
- **Quote:** approved commercial presentation derived from—but not identical to—an estimate revision.
- **Actuals record:** production outcomes linked to the original immutable estimate and versions.
- **Analysis proposal:** route, uncertainty, capacity/economic, revision, availability, sourcing, or bid view derived from a pinned estimate baseline; separately adopted or retained.
- **Requirement set:** revision-bound source evidence, interpretation, applicability, coverage, verification, blockers, and exceptions.
- **Learning evidence:** immutable correction, CAM, machine, and actual observations grouped into governed cohorts and rule suggestions; activation is a separate authorized record.

## Provenance vocabulary

Every significant value is `Measured`, `Imported`, `Calculated`, `Suggested`, `Historical`, or `Manual`. Suggested and calculated values may be superseded by an override, but the original value remains queryable. Approvals freeze a snapshot; re-analysis creates a new proposal and never mutates the approved routing.

## Dependency direction

Geometry facts feed stock, feature, setup, and runtime proposals. Drawing/contract/PMI requirements augment—but never silently emerge from—geometry. Routings feed the deterministic cost baseline. Advanced analyses consume immutable baselines and source snapshots, then emit proposals or explanatory mappings rather than mutating estimates. Risk, uncertainty, readiness, opportunity cost, and price remain distinct. UI, persistence, and vendor integrations are adapters around domain/application services.
