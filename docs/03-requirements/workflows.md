# User Workflows

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-001–REQ-F-065; UX-001–UX-045  
> **Related ADRs:** ADR-0001, ADR-0009–ADR-0014  
> **Open questions:** OQ-001–OQ-050  
> **Dependencies:** User research  
> **Supersedes:** None

## WF-01 RFQ to issued quote

Receive package → classify/access-check → customer/RFQ/part revision → validate quantities/dates → hash/import model and drawing → resolve units/import warnings → review geometry → add drawing/contract requirements → choose material/stock → review process/setup/routing/runtime → add inspection/outside/risk → calculate quantity scenarios → technical review → commercial review → approve immutable estimate → generate customer quote → issue and record outcome.

## WF-02 Model revision

Import as new source revision → compare hashes/metadata/geometry → run new analysis → show proposal versus approved routing → user accepts selected changes → retain prior snapshot and decisions → require reapproval before issue.

## WF-03 Actuals and calibration

Import/enter actuals → reconcile job/operation identity → compare category variance → assign reason codes → review cohort/bias → propose versioned library/rule change → authorize → apply only to future draft estimates.

## WF-04 Exception paths

- Missing units block dimensional calculations but allow package triage.
- Malformed geometry preserves intake record and safe diagnostics.
- Missing drawing creates visible requirement uncertainty.
- Expired material/vendor quote creates freshness warning and approval policy check.
- Unauthorized data/classification prevents preview/export and records an audit event.

## WF-05 Advanced route and capacity analysis

Clone the accepted baseline into route candidates → validate feasibility and exclusions → compare deterministic cost/time/setup/risk → optionally apply a versioned capacity snapshot → show delivery, contribution, and opportunity-cost views separately → review assumptions and sensitivity → explicitly adopt one candidate or retain the baseline. Capacity analysis advises; it does not reserve work or become a scheduler.

## WF-06 Requirement coverage and revision response

Capture drawing/spec/customer/PMI requirements as revision-bound proposed evidence → confirm interpretation and applicability → map coverage and verification → block or exception unresolved quote-critical items → compare a new model/drawing revision → inspect geometry/feature/requirement/manufacturing deltas → calculate cost deltas on a named estimate and rate basis → accept selected changes into a new draft branch → reapprove before issue.

## WF-07 Corrections, CAM, and learning governance

Record an estimator correction without erasing the original → optionally import a bounded CAM artifact → reconcile estimate, CAM simulation, machine observation, and actual as distinct observations → classify variance → aggregate only approved cohorts → propose a versioned rule change → review bias, sample size, privacy, and backtest evidence → authorize for future drafts only.

## WF-08 Availability, sourcing, and quote priority

Start with theoretical feasibility → apply timestamped tool/fixture/material/workcenter readiness → preserve unknown/stale states → compare eligible make/buy alternatives using landed cost and constraints → evaluate bid blockers → calculate an explainable priority score → human records the decision and reason. No marketplace data upload or production reservation occurs by default.

Detailed screen interactions live in `docs/05-ux`.
