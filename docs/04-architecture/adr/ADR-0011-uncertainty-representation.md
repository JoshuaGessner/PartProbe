# ADR-0011 — Staged Uncertainty Representation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-047–REQ-F-048, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-027–CALC-029; TEST-051–TEST-055  
> **Related ADRs:** ADR-0007, ADR-0009, ADR-0010  
> **Open questions:** Distribution families, dependency/correlation governance, percentile language, and validation thresholds  
> **Dependencies:** Deterministic calculation graph, reviewed scenarios, actuals/calibration evidence  
> **Supersedes:** None

## Context

Single-point estimates suggest false precision, but an early Monte Carlo implementation can add numerical theater without defensible distributions, dependencies, or calibration. The existing ordinal confidence model describes evidence quality and cannot by itself supply probabilities.

## Decision proposed

Adopt a staged uncertainty model separate from ordinal confidence and the risk ledger:

1. Store sourced low/most-likely/high ranges and named coherent scenarios.
2. Initially propagate deterministic scenarios and provide named one-at-a-time sensitivity.
3. Add triangular or PERT-style three-point distributions only for reviewed use cases with explicit family rationale.
4. Add seeded Monte Carlo, correlation/dependency treatment, probabilistic capacity, and calibrated distributions only after advanced validation.

Every result pins varied and fixed assumptions, units, sources, provenance, model/method/version, dependency assumptions, seed/samples where applicable, output quantile semantics, sensitivity method, warnings, and exclusions. System suggestions require human acceptance. Approved estimates retain immutable result/model snapshots.

`P90` means the 90th percentile of the named output distribution, not “90% confidence,” and is never presented as a guarantee. Expected value is unavailable without probability semantics. Unknown evidence is not converted automatically to a distribution or numeric confidence.

## Options considered

| Option | Disposition |
|---|---|
| Continue single values plus ordinal confidence only | Rejected as long-term design: cannot expose asymmetric outcome ranges or sensitivities |
| Monte Carlo and learned distributions in initial release | Rejected: unjustified model sophistication and validation burden |
| Universal percentage bands from confidence labels | Rejected: confuses evidence quality with probability and hides causality |
| Staged scenarios, three-point methods, then governed simulation | Proposed: useful early explanations with an extensible validated boundary |

## Consequences

The domain must retain uncertain inputs, scenarios, dependencies, and result manifests. The UI must teach range/percentile semantics without crowding normal estimating. Some outputs remain `NotModeled` or unavailable. Advanced simulation requires reproducible RNG/quantile policies, computation limits, and calibration evidence.

## Numeric, security, and history rules

The uncertainty engine evaluates the typed deterministic graph; it does not duplicate calculation rules. Currency retains fixed-decimal authority at named boundaries. Invalid samples are not silently dropped. Risk expected impacts and uncertain-input effects have explicit ownership to prevent double counting.

Execution remains local/in-boundary by default. Models, prices, actuals, schedules, and samples are classified with the estimate and excluded from logs. Algorithm changes create sibling results; no approved quote is silently recalculated.

## Acceptance gate

- Exact scenario and analytically solvable distribution fixtures pass.
- Bounds, quantile ordering, unit/currency, no-invalid-output, dependency, double-count, and sensitivity properties pass.
- Seeded replay and the declared cross-platform numeric policy pass.
- Monte Carlo stays disabled for approval use until sampler/quantile statistical tests, convergence budgets, and backtesting/calibration evidence pass.
- Human-factors tests show estimators distinguish most likely, expected, P50/P90, confidence, and guarantee language.
- Overrides, approval snapshots, authorization, local-only execution, and log redaction pass.

Only authorized architecture/product/estimation reviewers may change this ADR to Accepted.
