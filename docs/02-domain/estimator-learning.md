# Estimator Correction Learning

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-056–REQ-F-057, REQ-F-065; REQ-NF-015, REQ-NF-017–REQ-NF-021; TEST-074–TEST-078  
> **Related ADRs:** ADR-0007, ADR-0008, ADR-0013  
> **Open questions:** Correction taxonomy, learning cohort, approval roles, retention, and acceptable evidence thresholds  
> **Dependencies:** Override history, estimate/analysis versions, actuals, permissions, and rule registry  
> **Supersedes:** None

## Purpose and boundary

Estimator correction learning captures structured differences between a generated proposal and an authorized human decision before production actuals exist. It may identify repeatable bias and propose a rule/default change. It never edits approved shop settings, retrains or deploys a model, changes historical estimates, scores employee performance, or treats repeated preference as manufacturing truth without evidence.

Correction evidence complements rather than replaces `ActualsRecord`: a correction shows what a reviewer believed should change; an actual shows what production observed. Analytics label those evidence types separately and may compare them only through a declared cohort and method.

## Correction event

`CorrectionEvent` is append-only and records:

- stable event ID and correction taxonomy;
- affected RFQ, part/model, analysis, estimate, routing/setup/operation/feature, and quote scope as applicable;
- automatic/generated value, human-approved value, units/type, and the unchanged source proposal reference;
- actor, timestamp, reason code, free-text rationale policy, review/approval, and authorization;
- whether scope is quote-specific, customer/material/machine/process-specific, or proposed broadly applicable;
- geometry, feature, setup, routing, tool-selection, feeds/speeds, runtime, calculation, pricing, shop-library, and schema versions used;
- confidence/evidence before and after, known missing inputs, and context fingerprint;
- security classification, access scope, and retention/disposition policy.

The taxonomy covers process, machine, stock, setup/orientation, tool/cutting parameters, programming/cycle time, fixtures/jaws, inspection/risk, feature acceptance/classification, turning suitability, axis strategy, and outsourcing. Compound edits generate linked atomic events so analytics do not infer which field caused a result.

Editing a draft may replace its active value, but the correction event and generated proposal remain queryable. Deleting or merging an event requires an append-only supersession/tombstone reason; it cannot erase approved audit evidence.

## Rule suggestions

`RuleSuggestion` contains a human-readable hypothesis, targeted rule/default and scope, cohort query/version, included and excluded event IDs, sample counts, period, distribution/variance, comparison baseline, expected effect, counterexamples, data-quality/bias warnings, security scope, proposer/algorithm version, and reproducible evidence digest.

Suggested patterns may include machine/material/runtime bias, deep-pocket setup burden, customer inspection work, mesh feature omissions, or rejected process suitability. A correlation does not establish cause. The system shows supporting and contradicting cases and separates:

- estimator correction frequency;
- approved corrections;
- CAM-plan differences;
- machine/production actual variance; and
- outcome after a candidate rule was trialed.

Initial analytics are deterministic grouped summaries and threshold rules. More advanced statistical or learned methods are research until their validation, drift, security, explainability, and operational ownership are approved.

## Governance lifecycle

Correction lifecycle is `Captured → Validated → Eligible/Excluded → Superseded`, with immutable history. Suggestion lifecycle is:

`Draft → Evidence review → Technical review → Trial approved → Evaluated → Approved for activation → Active → Retired/Rolled back`, with `Rejected` available from any review state.

No suggestion can activate itself. `RuleApproval` records proposal/reviewer separation where staffing permits, reviewers, evidence snapshot, decision, reason, scope, activation/expiry, rollback target, and required validation. Activation creates a new effective-dated rule or library version through that domain's normal approval path. Existing approved estimates remain pinned to old versions. A trial is opt-in, separately labeled, and cannot change quote output without review.

Rollback deactivates the new version for future selection and restores a named prior default; it does not delete events, results, or quotes made under either version. Rule migration and replay decisions are explicit.

## Bias, quality, and missing data

Analytics must expose cohort selection, exclusions, missingness, and dominance by customer, estimator, machine, material, process, file type, or time period. Minimum sample, diversity, recency, effect-size, and validation requirements are versioned policy—not universal constants. Small or skewed cohorts yield `Insufficient evidence`, not a weak recommendation disguised as confidence.

Events with unresolved unit/version identity, compound unmapped changes, unauthorized edits, or missing original values are quarantined from broad suggestions but retained for correction of the data. Quote-specific changes default to excluded from broad generalization. Individual-estimator patterns require a documented legitimate purpose and restricted access; the product must not repurpose estimation evidence for personnel decisions by default.

## Ownership, security, and audit

- Estimators own accurate reason capture.
- Manufacturing/process owners review technical applicability.
- Quality reviews inspection/compliance implications.
- Shop-library/rule owners approve the target change.
- A designated approver authorizes activation and rollback.
- Security/data owners set access, retention, export, and cross-project aggregation policy.

Use minimum necessary context in analytic records. Do not copy CAD, drawings, customer text, prices, user free text, or controlled requirements into general telemetry or external analytics. Processing is local/in-scope by default; no external AI or learning service receives events without a separate ADR and deployment policy. Capture, exclusion, cohort generation, suggestion, review, trial, activation, rejection, export, and rollback are audited.

NIST's manufacturing-data guidance emphasizes traceability/trustworthiness and deliberate collection, curation, and reuse; those goals support preserving source lineage and cohort evidence rather than treating event volume as authority ([NIST AMS 300-10](https://www.nist.gov/publications/recommendations-ensuring-traceability-and-trustworthiness-manufacturing-related-data), [NIST AMS 300-11](https://www.nist.gov/publications/recommendations-collecting-curating-and-re-using-manufacturing-data)).

## Staged delivery

- **Early foundation:** correction taxonomy, append-only event capture, version/context links, and permissions.
- **Initial production:** user-visible correction history and quote-specific/general-scope flag; no rule suggestions.
- **Intermediate:** local aggregate dashboards, governed cohorts, and evidence bundles for manual review.
- **Advanced:** controlled suggestions, trials, approval/activation, monitoring, and rollback.
- **Research/deferred:** adaptive models, automatic personalization, cross-shop learning, and any external hosted analytics.

## Validation and acceptance

- **TEST-074:** every correction type preserves generated/approved values, units, actor/reason, scope, and all relevant versions.
- **TEST-075:** invalid, duplicate, compound, missing-version, and unauthorized events are rejected or quarantined without corrupting history.
- **TEST-076:** cohort analytics are reproducible; exclusions, missingness, counterexamples, estimator/customer dominance, and insufficient evidence are visible.
- **TEST-077:** suggestion review, separation of duties, trial, activation, rejection, and rollback enforce authorization and append-preserving audit.
- **TEST-078:** activating or rolling back a rule creates versions, leaves approved estimates unchanged, and replays golden cohorts cross-platform without content leakage.

## Shop decisions required

1. Which correction types and reason codes are required, and when is free text permitted?
2. Who may see correction analytics, define cohorts, approve trials, activate rules, and roll back?
3. What minimum evidence, diversity, recency, and actuals confirmation is required by rule class?
4. How long are events/suggestions retained, and may evidence cross customer, program, site, or security boundaries?
5. Which patterns are useful for estimator assistance but prohibited for personnel evaluation or automatic personalization?

