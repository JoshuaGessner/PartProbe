# Success Metrics

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-040–REQ-F-065; REQ-NF-002, REQ-NF-003, REQ-NF-006, REQ-NF-015–REQ-NF-022; TEST-001–TEST-099  
> **Related ADRs:** ADR-0001–ADR-0014  
> **Open questions:** OQ-027, OQ-028, OQ-031–OQ-050  
> **Dependencies:** Baseline studies and representative fixtures  
> **Supersedes:** None

## Phase 0/1 gates

- 100% of implementation tasks link to requirements.
- 100% of accepted calculation rules have worked examples and tests.
- Required CAD fixtures have reviewed expected units, dimensions, volume, area, validity, and warnings.
- Supported operating systems pass the same core test suite.
- Zero external network calls during the offline workflow test.

## Vertical-slice outcome metrics

- Median first-pass estimate preparation time improves against a measured manual baseline; target to be set after observation.
- Users can explain the source of every total in a moderated test.
- No re-analysis overwrites an approved manual routing.
- Currency golden tests reproduce exactly across platforms.
- Geometry measurements meet per-fixture tolerances documented in `geometry-validation.md`.
- At least 90% of critical usability tasks complete without moderator intervention; final threshold requires field validation.

Accuracy targets for runtime and quote totals remain unset until the shop supplies representative actuals. Inventing a percentage now would create false confidence.

## Advanced-system measures

Establish baselines and stratify by process, machine/material family, estimate method, quantity, customer/requirement class, and data quality before setting targets:

- Estimate-to-actual setup, total-cost, and delivery-date variance.
- Estimate-to-CAM and CAM-to-actual cutting/non-cutting/total runtime variance.
- Manual routing, machine, setup, feature, inspection, and risk correction frequency and reason.
- Requirement omission/conflict/recognized-not-costed rate at approval and after award.
- Quote-revision turnaround time and correctness of explained cost deltas.
- Route-comparison use, selected-alternative distribution, and expert rejection rate.
- Capacity-feasibility, material/tool/fixture/vendor availability, and delivery prediction accuracy at their recorded as-of times.
- Recommendation acceptance, override, later reversal, and approved rule-improvement frequency—segmented to detect sparse or biased cohorts.
- Quote preparation time and win rate alongside estimate accuracy, rework, exception, and margin-floor outcomes.
- Contribution amount and contribution per bottleneck hour using the approved cost/time basis; never optimize these alone.

Metrics are diagnostic and balanced. Faster quoting, higher recommendation acceptance, win rate, or contribution is not success when requirement coverage, accuracy, security, delivery reliability, or appropriate human review worsens.
