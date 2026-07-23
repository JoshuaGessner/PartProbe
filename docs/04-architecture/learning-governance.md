# Learning Governance Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-056–REQ-F-057, REQ-F-065; REQ-NF-015, REQ-NF-017–REQ-NF-021; TEST-074–TEST-078  
> **Related ADRs:** ADR-0006–ADR-0008, ADR-0013, ADR-0014  
> **Open questions:** Rule-owner matrix, cohort thresholds, trial policy, rollback authority, and cross-boundary aggregation  
> **Dependencies:** Correction model, audit, rule registry, permissions, actuals, and CAM reconciliation  
> **Supersedes:** None

## Boundary

Learning is a governed recommendation pipeline outside authoritative estimation evaluation. The estimation, geometry, routing, feature, tooling, feeds/speeds, and pricing modules expose versioned decisions and accept only approved versioned configuration through their existing ports. The learning subsystem cannot write their active rules directly.

```text
immutable correction/CAM/actual evidence
  → eligibility and quality checks
  → versioned cohort snapshot
  → deterministic analysis
  → immutable rule suggestion
  → human evidence/technical/trial review
  → normal rule-owner approval and new version
  → monitored activation or rollback
```

## Components and responsibilities

| Component | Responsibility | Explicitly cannot do |
|---|---|---|
| `CorrectionCaptureService` | validate and append structured correction events | infer broad applicability or delete source proposals |
| `EvidenceEligibilityService` | enforce scope, classification, consent/policy, required versions, and data-quality rules | reinterpret restricted evidence into a broader scope |
| `CohortBuilder` | create reproducible immutable cohorts with included/excluded IDs and query version | select hidden examples after results are known |
| `PatternAnalyzer` | produce deterministic summaries, comparisons, counterexamples, and warnings | activate settings or claim causation |
| `SuggestionRegistry` | persist suggestion lifecycle, evidence digest, approvals, trial, monitoring, and rollback target | mutate source events or historical results |
| `RuleChangePort` | hand an approved proposal to the owning module's normal version/approval workflow | bypass its validation, permissions, or migration policy |
| `OutcomeMonitor` | compare trial/new-version outcomes to baseline under a named method | silently disable or retune a rule |

CAM reconciliation and actuals are evidence adapters, not automatic labels. A reviewer-confirmed cause and mapping status determine eligibility. Quote-specific correction events remain excluded from general cohorts unless an authorized reviewer broadens scope with a recorded reason.

## Reproducibility and versioning

Each run stores engine/code/schema version, cohort/query/policy versions, included/excluded event digests, feature/aggregation definitions, thresholds, random seed if any, environment-independent canonical serialization, outputs, and warnings. Named analytical methods are immutable; a changed method creates a new version and comparison.

Rule suggestions bind to a target rule type/version and explicit applicability scope. Approval produces a new candidate version, never an in-place patch. Activation is effective-dated, reversible, and recorded in the version registry. Approved estimates pin their original versions; evaluation under a candidate is a separately labeled replay.

## Evidence and bias gates

Before a suggestion may reach technical review, the pipeline checks:

- original and corrected values are typed and version-linked;
- actor and authorization are valid;
- cohort purpose and permitted security scope are declared;
- minimum sample, diversity, recency, and missingness policy is met;
- customer, estimator, machine, material, process, file type, and time-period concentration is reported;
- repeated/linked edits are not double counted;
- quote-specific preferences and actual production evidence remain distinguishable;
- counterexamples and adverse effects are surfaced;
- target rule ownership, validation plan, rollback target, and monitoring window exist.

Failure yields `Ineligible`, `Insufficient evidence`, or a warning requiring explicit review; it never lowers the gate silently. Thresholds are shop-approved policy and vary by rule risk. A feeds/speeds or security-related rule may require stronger evidence than a display default.

## Approval, trial, and rollback

Suggestion author/engine, technical reviewer, rule owner, and activation approver are distinct roles where staffing permits. Higher-risk changes require manufacturing, quality, commercial, or security concurrence based on impact. An approved trial has bounded scope, start/end, target version, comparison baseline, success/harm metrics, users/projects permitted, and stop authority.

Activation cannot occur until the target module's tests, worked examples, documentation, version/migration decision, and approval pass. Monitoring never edits the rule. A threshold breach raises a review/rollback recommendation. Authorized rollback restores a named prior default for future drafts and preserves all trial/results/audit evidence.

## Security and privacy

The learning store uses project/classification-aware authorization and purpose-limited access. Cohorts may cross projects, customers, sites, or users only when policy explicitly permits it; aggregation does not declassify source evidence. Derived statistics inherit classification unless a reviewed release rule proves otherwise.

Minimize identifiers and free text in analytics; retain source links behind access checks. Do not copy CAD/drawing content, customer requirements, pricing, paths, secrets, or personal notes into logs or generic feature stores. External analytics, hosted models, cross-shop contributions, or AI services are disabled and out of scope absent a separate ADR and deployment policy. Audit cohort access/export, suggestion decisions, trials, rule handoff, activation, and rollback.

## Failure and recovery

- Missing versions, broken lineage, unauthorized records, or digest mismatch quarantine evidence.
- A failed analyzer run writes no partial authoritative suggestion; reproducible diagnostics remain content-minimized.
- Rule-registry or activation failure leaves the prior active version unchanged.
- Restore/migration verifies evidence and suggestion digests and preserves lifecycle history.
- Revoked access removes future cohort eligibility and access according to retention policy; it does not silently falsify historical approval evidence.

## Release sequence

1. Append-only correction capture and audit.
2. Read-only local cohorts and deterministic dashboards.
3. Immutable suggestions with evidence bundles and rejection workflow.
4. Bounded opt-in trials and monitored version activation.
5. Only after validated need: advanced statistical methods with separate method ADR/evidence.

Acceptance requires the workflow and tests in [estimator correction learning](../02-domain/estimator-learning.md), plus threat modeling, permissions tests, deterministic replay, representative bias fixtures, and proof that no suggestion can mutate an active rule without the owning module's approval path.

