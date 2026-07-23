# Bid/No-Bid and Quote-Priority Model

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-063–REQ-F-065; REQ-NF-015, REQ-NF-017–REQ-NF-021; TEST-095–TEST-099  
> **Related ADRs:** ADR-0006, ADR-0007  
> **Open questions:** Scoring factors/weights, blockers, approval thresholds, win-probability evidence, and priority owner  
> **Dependencies:** RFQ lifecycle, routing/estimate, pricing, requirements, availability/capacity, security, and permissions  
> **Supersedes:** None

## Purpose and decision boundaries

The system supports two related but different decisions:

- `BidDecision`: whether to pursue, decline, hold, or escalate one RFQ revision.
- `QuotePriorityScore`: the relative order in which eligible RFQs should receive estimating attention at a stated snapshot time.

Neither decision is a quote price, accounting result, employee work assignment, or autonomous management decision. A configurable, explainable recommendation informs authorized judgment. A human records the decision or override.

## Eligibility before scoring

Hard or approval-required concerns are evaluated before any weighted score. Examples include unsupported/classified data, prohibited customer or supplier relationships, missing authority to handle the package, impossible required process/certification, unresolved critical requirements, customer-mandated restrictions, or a date that cannot be assessed.

A `Block` prevents an ordinary bid recommendation. `Escalate` requires a named authority. `Unknown` is never coerced to a favorable or unfavorable numeric value. Policy may allow a time-bounded assumption, but the recommendation must show the resulting coverage and confidence.

## Factors and explanation

Candidate factors include strategic customer, payment/relationship history, revenue, margin, contribution per bottleneck hour, delivery feasibility, availability, process/machine familiarity, quality/security burden, outside-process dependency, tooling/fixture effort, quote effort, win probability, competitive pressure, repeat potential, technical/schedule risk, cash-flow effect, and completeness of customer-furnished information.

Each `ScoreFactor` stores value/range, direction, normalization rule, weight, source/evidence, `as of`, freshness, confidence, missing-data policy, applicability, and contribution. Calculation is deterministic and versioned:

`eligible weighted score = Σ(weight × normalized factor)`, accompanied by factor coverage and uncertainty; it is not emitted when policy says missing critical factors block scoring.

Weights and normalization are shop policy. Positive and negative contributions, excluded factors, threshold crossings, blockers, and the top reasons are all visible. Customer identity must not act as an unexplained proxy; factors must have a legitimate commercial/operational purpose and authorized source.

`BidRecommendation` is `Bid`, `No bid`, `Hold`, or `Escalate`. `QuotePriority` is a separately versioned tier/rank that may consider response deadline and quote effort in addition to bid attractiveness. A high-value RFQ does not automatically outrank an imminent low-effort response unless policy explicitly says so.

## Aggregate and lifecycle

`BidDecision` belongs to the RFQ revision and contains eligibility results, score snapshot, recommendation, required approvals, final human decision, reasons, override, actor/time, and follow-up/expiry. Lifecycle:

`Draft → Evaluated → Review required/Ready → Decided`, then `Superseded`, `Expired`, or `Reopened` through a new revision.

`QuotePriorityScore` is an immutable queue snapshot with policy version, population/filter, capacity/availability snapshot, source times, score components, rank/tier, warnings, and digest. Priority is inherently time-sensitive; refreshing the queue creates a new snapshot. It does not rewrite the bid decision or approved quote.

Policy changes create effective-dated versions. Issued/approved quote evidence retains the decision, score, and inputs used at the time. Recalculation under a new policy is a labeled comparison, never a rewrite.

## Missing, stale, and conflicting data

- Missing payment, win-probability, capacity, or availability data yields `Unknown` plus a declared fallback or blocker.
- Stale data retains its `as of` date, cannot claim current delivery feasibility, and lowers factor coverage/confidence.
- Conflicting customer, security, or requirements facts force escalation until the source authority resolves them.
- A partial estimate may support triage but cannot present final margin or contribution as known.

The UI shows score coverage (known applicable weight versus total applicable weight) separately from score. A recommendation with low coverage is labeled provisional.

## Ownership, security, and audit

Commercial leadership owns factor definitions, weights, thresholds, and final bid policy. Estimating owns quote-effort inputs; manufacturing/quality own technical and quality feasibility; finance owns authorized commercial/payment inputs; operations owns capacity evidence; security/classification authorities own handling restrictions. Only designated approvers may override blockers or no-bid policy.

Customer financial history, prices, strategy, security restrictions, and win-probability notes are restricted commercial data. Enforce least privilege, minimize free text, avoid sensitive content in logs, and prevent customer-facing exports. Audit policy changes, scoring, source refresh, blocker resolution, decision, override, reprioritization, and export. Do not transmit RFQ content to external scoring or AI services by default.

## Staged delivery

- **Early foundation:** structured bid decision, reason taxonomy, eligibility blockers, manual priority tier, audit/version fields.
- **Initial production:** explainable manual-factor recommendation and human override; no automatic queue ordering required.
- **Intermediate:** versioned weighted scoring, requirements/readiness/capacity inputs, queue snapshots, and outcome reporting.
- **Advanced:** capacity-adjusted priority scenarios, calibrated win-probability ranges, and sensitivity analysis.
- **Deferred/research:** automated bidding, opaque learned scores, external credit/marketplace enrichment, and automatic price changes.

## Validation and acceptance

- **TEST-095:** known policy fixtures reproduce factor normalization, weighted contributions, thresholds, coverage, reasons, and deterministic rank ties.
- **TEST-096:** hard blockers and approval-required concerns are enforced before scoring; no score bypasses security or requirement readiness.
- **TEST-097:** missing, stale, conflicting, and partial data produce `Unknown`/provisional/escalated results, never silent zero/default facts.
- **TEST-098:** authorized override, reopening, policy upgrade, and queue refresh preserve old decisions/snapshots and complete audit.
- **TEST-099:** access/export tests protect financial/strategic data; sensitivity and outcome tests detect unstable or systematically skewed recommendations.

## Shop decisions required

1. Which factors are legitimate, who owns each source, and which are blockers versus weighted considerations?
2. What normalizations, weights, thresholds, tie-breakers, expiry, and minimum coverage are acceptable?
3. Who may decide, override, reopen, and reprioritize, and which decisions require technical/quality/security concurrence?
4. How are win probability, payment history, strategic value, quote effort, and cash-flow impact evidenced and retained?
5. Which outcome metrics review policy quality without pressuring users to ignore risk, requirements, or data-handling rules?

