# ADR-0013 — Estimator-Learning Governance

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-056–REQ-F-057, REQ-F-065; REQ-NF-015, REQ-NF-017–REQ-NF-021; TEST-074–TEST-078  
> **Related ADRs:** ADR-0006–ADR-0008, ADR-0014  
> **Open questions:** Cohort thresholds, rule-owner/approval matrix, trial policy, retention, and cross-project scope  
> **Dependencies:** Correction-event capture, rule/version registry, authorization/audit, and representative evidence  
> **Supersedes:** None

## Decision proposed

Use an append-only, human-governed learning pipeline. Structured correction, CAM-reconciliation, and actuals evidence may produce immutable, explainable `RuleSuggestion` records. Suggestions cannot modify active settings. A change reaches production only through the target rule/library's normal validation, approval, effective-dated versioning, trial/monitoring where required, and explicit rollback path.

Default implementation is deterministic cohort analytics, not adaptive machine learning. Processing remains local or inside the approved deployment boundary. Historical estimates and quotes retain the exact rule versions originally used.

## Rationale

Estimator corrections are valuable early evidence but may encode quote-specific judgment, missing context, inconsistent habits, or bias. Separating evidence capture, cohort analysis, suggestion review, and rule activation preserves explainability and prevents feedback loops from turning frequency into authority. NIST manufacturing-data guidance emphasizes traceability/trustworthiness plus deliberate collection, curation, and reuse, which supports immutable lineage and governed cohorts ([NIST AMS 300-10](https://www.nist.gov/publications/recommendations-ensuring-traceability-and-trustworthiness-manufacturing-related-data), [NIST AMS 300-11](https://www.nist.gov/publications/recommendations-collecting-curating-and-re-using-manufacturing-data)).

## Alternatives considered

- **Automatically tune defaults from corrections:** rejected because it can amplify bias, lacks causal evidence, and silently changes financial/manufacturing behavior.
- **Capture corrections but never aggregate them:** safe for the first release but insufficient as the long-term boundary; retained as the first delivery stage.
- **Use an opaque hosted learning service:** rejected as the default because it weakens reproducibility, version control, controlled-data boundaries, and offline operation.
- **Allow each domain module to learn independently:** rejected because cohort, permission, audit, trial, and rollback policy would diverge.

## Consequences

The system stores more event/version/evidence data and requires named owners, review queues, cohort policy, deterministic replay, bias checks, trial design, and rollback operations. Suggestions may take longer to activate, but every change remains attributable and reversible. Advanced statistical or machine-learned analysis requires a later method-specific decision and validation; this ADR does not authorize it.

## Required invariants

- Source proposal, correction, cohort, suggestion, approval, target rule, activation, and rollback remain traceable.
- Missing lineage or insufficient/skewed evidence cannot become an active rule.
- Suggestion generation and rule activation are separate permissions; separation of duties is applied where staffing permits.
- Aggregation never broadens project/classification access or declassifies derived evidence.
- No external service receives technical, commercial, or correction data by default.
- Activation creates a new rule/library version; rollback names a prior version; neither rewrites approved estimates.
- Trial/candidate results are visibly distinct from authoritative quote results.

## Acceptance gate

TEST-074–TEST-078 pass with representative correction, skew, missingness, unauthorized access, trial, activation, and rollback fixtures. Evidence must demonstrate deterministic cohort replay, visible inclusions/exclusions/counterexamples, target-module validation, append-preserving audit, access/export controls, and proof that a suggestion cannot write an active rule directly. Product, manufacturing, quality, commercial, and security owners must approve their responsibility matrix before this ADR can be Accepted.

