# Uncertainty Validation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-047–REQ-F-048, REQ-F-065; REQ-NF-003–REQ-NF-004, REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-027–CALC-029; TEST-051–TEST-055  
> **Related ADRs:** ADR-0007, ADR-0011  
> **Open questions:** Calibration datasets, error thresholds, percentile policy, and supported distribution families  
> **Dependencies:** Executable calculation graph, uncertainty engine, reviewed synthetic/shop fixtures  
> **Supersedes:** None

## Validation claim boundary

Validation must support claims per method, output, process/material cohort, model version, and data quality. Passing numeric tests proves implementation semantics; it does not prove that a shop's input ranges or distributions predict reality. No universal accuracy or coverage claim is permitted.

Customer data is excluded from general fixtures unless rights, classification, access, retention, and expected-result review are documented. CI uses synthetic or approved sanitized data and makes no external analytics call.

## Test allocation

| ID | Scope | Required evidence |
|---|---|---|
| TEST-051 | Three-point inputs | Ordered bounds, units, sources, validation, dependencies, and manual/generated provenance |
| TEST-052 | Distribution methods | Triangular/PERT/scenario formulas and hand-calculable propagation through reviewed calculation traces |
| TEST-053 | Simulation reproducibility | Seed, RNG/sample and quantile policies, dependencies/correlation, moments/quantiles, and cross-platform tolerance |
| TEST-054 | Sensitivity and resources | Known-driver rank/direction/stability, convergence, run limits, cancellation, and cache keys |
| TEST-055 | Uncertainty invariants | P10 ≤ P50 ≤ P80 ≤ P90, no physically impossible negative values, and deterministic baseline preservation |

## Fixture schema

Each golden fixture records:

- fixture ID, purpose, license/classification, reviewer, and expected-result authority;
- deterministic graph/rule/library versions and route/quantity scope;
- uncertain and fixed inputs with units, source, provenance, credible bounds, scenarios/family, dependencies, and rationale;
- method, algorithm/schema, quantile/sensitivity policy, seed/RNG/sample count, resource limits, and currency rounding boundaries;
- expected exact outcomes or statistical acceptance bands;
- known limitations, invalid cases, and whether the fixture validates implementation only or empirical calibration.

Golden changes require an explained review. Do not blindly regenerate expected outputs after an engine change.

## Exact and property tests

Exact tests cover constant inputs; one uncertain additive/multiplicative term; two-point and weighted scenario trees; low/likely/high propagation through CALC rules; min/max support; fixed-decimal conversion; and analytically solvable uniform/triangular/PERT-style cases when supported.

Required properties include:

- `minimum ≤ P10 ≤ P50 ≤ P80 ≤ P90 ≤ maximum`, subject to the documented discrete-quantile convention;
- constant inputs produce a point result and zero sensitivity;
- increasing a nonnegative additive time/cost input cannot lower that output under otherwise identical trials;
- units/currencies cannot mix without an explicit conversion rule;
- time, cost, count, probability, scrap/yield, and bounded-rate domains reject invalid values;
- most-likely lies within credible bounds; probability-bearing scenarios normalize under the named policy;
- shared sources are evaluated once per trial and dependencies are honored;
- mutually exclusive scenarios cannot co-occur;
- deterministic baseline and uncertainty results share identical calculation rules;
- a risk event is represented in either probabilistic inputs or risk-reserve expected impact according to ownership—not both;
- missing evidence produces `NotModeled`/`Blocked`, never zero uncertainty;
- display precision and rounding do not alter stored summaries.

Edge cases include zero cost/time where valid, zero denominator, near-bound probabilities, single scenario, duplicate IDs, extreme but permitted values, overflow, high correlation, discontinuous floors/minimums, quantity rounding, all routes infeasible, and invalid trial behavior.

## Seeded and statistical tests

Seeded golden tests pin RNG algorithm/version, seed, sample order policy, worker-count behavior, sample count, and quantile estimator. Repeated runs on supported platforms must satisfy the declared bitwise or numeric-tolerance policy; a change requires a new algorithm version.

For each supported sampler/distribution, test generated bounds and use statistically justified checks for moments, quantiles, and categorical frequency across several fixed seeds. Thresholds account for expected sampling error and are reviewed to avoid flaky tests. Test the quantile estimator independently with fixed arrays, including discrete ties and interpolation boundaries.

Convergence diagnostics use predeclared representative outputs and increasing sample counts; they determine a run budget, not a guarantee of model truth. Performance fixtures cover large graphs/output sets, memory bounds, cancellation, partial-result labeling, and nested capacity-evaluation limits. Invalid samples are counted and fail authority under policy; none are silently dropped.

## Sensitivity validation

Use synthetic graphs with a known dominant input, monotonic directions, interaction, shared cause, irrelevant input, and correlated inputs. Verify rank and direction for each named method separately. One-at-a-time, rank correlation, and variance-based output cannot share expectations or labels.

Sensitivity stability tests vary seed and sample count. Unstable ordering is reported, not hidden. The UI must link each driver to its input source/range and never imply causation from a correlation-only method.

## Empirical calibration and backtesting

Before probabilistic claims guide approval, backtest frozen estimates against later actuals by comparable cohort: method, process, material, machine class, quantity, risk type, and scope change. Preserve estimate-time information to prevent leakage.

Evaluate interval/quantile coverage, calibration/reliability, sharpness, bias, and proper scoring measures selected in the validation plan. A nominal P90 whose realized outcomes fall below it materially less or more than the declared frequency is investigated for model/input/cohort drift; it is not automatically rescaled. Small cohorts show sample count and uncertainty and cannot authorize a learned default.

Separate implementation defects, geometry/requirement omissions, estimator range choices, process variation, planning change, and execution disruption. Calibration may propose versioned changes; it never rewrites approved settings or historical estimates.

## Workflow and human-factors validation

Tests prove an authorized user can enter/accept/override a range; inspect sources and dominant drivers; distinguish low/likely/high, expected, P50/P90, confidence, and high-risk scenario; identify `Not modeled`; see baseline versus risk-adjusted output; and approve only after policy blockers/exceptions are resolved.

At minimum, comprehension sessions ask users to explain a P90 modeled cost/delivery in their own words and reject interpretations such as warranty or “90% confidence that the model is correct.” The normal view must remain usable without opening advanced statistics. Every visualization has a keyboard-operable table equivalent and non-color-only states.

## Versioning, migration, security, and release gates

Replay fixtures load prior model/result schemas, preserve approved snapshots, and compare sibling algorithm versions without mutation. Migration tests cover unknown future fields, missing legacy fields, canonical digests, retained override/approval history, and raw-sample retention/compaction policy.

Permission tests enforce project/classification/commercial scope for models, results, actuals, and exports. Network-isolation tests prove local execution. Logs/support bundles exclude sources, customer/part identity, prices, schedule content, ranges, samples, and model values unless an authorized explicit diagnostic policy says otherwise.

Release gates:

1. Scenario/range storage ships only after TEST-051/052/055 pass.
2. Three-point probability families ship only after exact/statistical implementation tests and user terminology review pass.
3. Monte Carlo cannot support approval until TEST-053 performance/reproducibility gates and TEST-054 cohort-specific calibration plan/evidence pass.
4. Probabilistic capacity/delivery and learned distributions remain advanced until dependency, drift, and cross-engine validation pass.
5. Any changed method, quantile, sensitivity, clamping, dependency, RNG, or rounding behavior increments its version and repeats applicable gates.
