# Probabilistic Estimation

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-047–REQ-F-048, REQ-F-065; REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-027–CALC-029; TEST-051–TEST-055  
> **Related ADRs:** ADR-0007, ADR-0011  
> **Open questions:** Shop percentile vocabulary, risk appetite, correlation policy, and minimum calibration evidence  
> **Dependencies:** Deterministic calculation graph, evidence/confidence model, actuals, route/capacity snapshots  
> **Supersedes:** None

## Purpose and boundary

Probabilistic estimation represents credible variation in time, cost, price-floor, and delivery outcomes without pretending that incomplete knowledge is precise. It supplements—not replaces—the deterministic trace, ordinal confidence, risk ledger, and human judgment.

An uncertainty result is an estimate under recorded assumptions. It is not a warranty, confidence interval about an unknown statistical parameter, production CAM simulation, or guaranteed delivery date.

## Concepts and owned records

| Record | Required meaning |
|---|---|
| `UncertainInput` | Stable ID; target input; minimum/most-likely/maximum or scenarios; optional family/parameters; source; rationale; owner; bounds; correlation group; provenance |
| `Scenario` | Named coherent set of input values/events, applicability, probability if justified, and source |
| `UncertaintyModelRevision` | Immutable inputs, dependency/correlation rules, method, algorithm/schema version, seed policy, and validation state |
| `EstimateDistribution` | Result variable, route/quantity scope, samples or exact scenarios, percentiles, expected/most-likely values, support, warnings, and digests |
| `SensitivityResult` | Input-to-output influence by named method, rank, direction, stability, and model revision |
| `UncertaintyReviewDecision` | Accepted/overridden ranges, actor, time, reason, authority, and effect on estimate approval |

Inputs distinguish inherent variation (for example measured cycle variation) from knowledge gaps (for example an unproven setup). The distinction is recorded because more data may reduce a knowledge range while ordinary process variation remains. An `Unknown` is not automatically converted to a distribution.

## Staged methods

1. Early foundation stores ranges, sources, and reasons while preserving ordinal confidence.
2. Initial production shows deterministic low/most-likely/high scenarios and one-at-a-time sensitivity; values are not assigned probabilities unless supported.
3. Intermediate releases add three-point triangular or PERT-style inputs and deterministic scenario propagation for approved use cases.
4. Advanced releases may add seeded Monte Carlo sampling, explicit dependencies/correlation, probabilistic capacity/lead-time treatment, and calibrated percentile policies.
5. Learned distributions and broad Bayesian/ML methods remain research until governed data volume, bias review, and explainability gates pass.

Method selection is per output/model revision and visible to the user. A three-point input does not justify a distribution family by itself; the chosen family and rationale are explicit.

## Values and display semantics

- Minimum and maximum are credible modeled bounds, not absolute guarantees.
- Most likely is the mode or named scenario; it need not equal the expected value.
- Expected value is the probability-weighted mean and is unavailable without probability semantics.
- `P10`, `P50`, `P80`, and `P90` are quantiles of the stated output distribution: for nonnegative cost/time, `P90` is a value at or below which 90% of modeled outcomes fall.
- A high-risk scenario is a named coherent scenario, not automatically the mathematical maximum or P90.
- A selling-price floor distribution is an advisory input to pricing policy; it is not the quoted price.

Every displayed percentile states output, units/currency, quantity, route, method, sample/scenario count, as-of time, and model version. Presentation precision cannot exceed input/model precision.

## Applicable variables and causality

Approved uncertainty may apply to programming, setup, prove-out, cutting/non-cutting, tool life, material yield/cost, scrap, rework, inspection, outside-process cost/lead time, total internal cost, price floor, capacity, and delivery date. Sources include representation limits, missing drawing/tolerance data, unfamiliar material/process/machine/vendor, unproven workholding, access/tooling difficulty, distortion/thin walls, expensive stock, and aggressive schedules.

Uncertain nodes feed the same typed calculation graph rules as deterministic inputs. Correlated quantities must not be sampled independently by default. Mutually exclusive events use scenarios or a categorical model. Risk-register expected impacts and probabilistic input effects must have an explicit ownership rule so the same event is not charged twice.

Physical samples may use floating point under documented tolerance; authoritative currency aggregation and presentation cross named fixed-decimal boundaries. Invalid samples are errors, not silently discarded. Time, cost, quantity, probability, and bounded rates enforce domain constraints; truncation/clamping is allowed only as a versioned named rule visible in the trace.

## Missing-data behavior

- Missing uncertainty evidence leaves the deterministic result intact and labels the range `Not modeled`; it does not imply zero uncertainty.
- Missing most-likely or credible bounds blocks three-point evaluation.
- Missing probability weights permits scenario comparison but not expected value or probabilistic percentiles.
- Unknown dependency/correlation is disclosed and tested through sensitivity scenarios; it is not silently assumed independent for authoritative use.
- An ordinal low-confidence claim may widen or prompt a range only through a named policy; confidence is never mechanically converted to a percentage.
- Stale historical/vendor evidence remains source-dated and may be blocked by approval policy.

## Explainability and human review

Every result lists assumptions that vary, fixed assumptions, ranges/distributions and sources, system-generated versus user-entered status, top sensitivity drivers, exclusions, warnings, and comparison with the deterministic baseline. Users can inspect the calculation path from an output quantile to contributing rules and inputs.

System-suggested ranges remain proposals. Authorized users accept or override them with a reason. Approval policy may require review when a cost/delivery spread exceeds a threshold, a dominant input is `Unknown`, correlation is unsupported, or a chosen selling price/delivery promise falls outside policy. Approved quotes preserve the complete model/result snapshot and never recalculate when methods change.

## Security, retention, and release gates

Uncertainty inputs and results inherit the estimate's classification and commercial access. Local execution is the default. No model, price, actual, vendor, customer, or schedule data is sent to an external analytics service without an explicit approved integration/deployment decision. Logs record safe IDs, versions, seed, duration, and error class—not sampled commercial values or source content.

Retain approved model definitions, seeds/scenarios, result summaries/digests, calculation/library versions, overrides, and validation state for replay. Large raw sample arrays may be omitted only when the same published result can be reproduced or when an approved retention policy explicitly records non-replayable compaction.

Release requires the validation in [uncertainty validation](../06-quality/uncertainty-validation.md): range/order invariants, seeded reproducibility, exact simple cases, sensitivity checks, no invalid outputs, calibration/backtesting where claims depend on frequency, and usability tests proving estimators understand what the ranges do and do not mean.
