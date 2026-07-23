# Uncertainty Engine Architecture

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-047–REQ-F-048, REQ-F-065; REQ-NF-003–REQ-NF-004, REQ-NF-015–REQ-NF-018, REQ-NF-021–REQ-NF-022; CALC-027–CALC-029  
> **Related ADRs:** ADR-0007, ADR-0011  
> **Open questions:** Supported families, dependency model, run budgets, and authoritative percentile policy  
> **Dependencies:** Typed deterministic calculation graph and approved uncertain-input model  
> **Supersedes:** None

## Boundary and proposed module

The uncertainty engine evaluates an existing typed deterministic calculation graph under versioned scenarios or uncertain inputs. It owns model validation, scenario/sample generation, propagation, quantiles, sensitivity, diagnostics, and reproducibility metadata. It does not own baseline rules, risk acceptance, pricing approval, distribution suggestions, or quote decisions.

Use a UI-independent `crates/uncertainty-engine` depending on domain and public estimation-engine contracts. The estimation engine exposes a pure batch evaluation boundary; it must not depend back on uncertainty. Capacity evaluation is integrated through an optional adapter with an explicit computational budget rather than a cyclic crate dependency.

```text
deterministic graph snapshot + uncertain-input revision
                         │
                 validate model
                         ↓
            scenarios or seeded samples
                         ↓
            batch deterministic evaluation
                         ↓
        summaries + sensitivity + diagnostics
                         ↓
                  human review
```

## Engine contracts

`UncertaintyModel` identifies varied and fixed nodes, domains/bounds, scenarios or distribution family/parameters, dependencies/correlation, source, and version. `EvaluationPlan` identifies output nodes, method, exact or sampled execution, seed/RNG algorithm, sample count, convergence/reporting policy, and resource limits. `UncertaintyResult` stores valid/invalid counts, summaries, quantiles, support, sensitivity, diagnostics, and complete manifest/digests.

The initial exact scenario evaluator accepts named low/most-likely/high sets. A later distribution provider interface may supply triangular and PERT-style sampling without embedding one method into domain records. Monte Carlo is an advanced provider and cannot be enabled for approval use until its validation gate passes.

## Validation before execution

Reject missing/duplicate node IDs, unit or currency mismatch, non-finite physical values, invalid probability totals, inverted bounds, a most-likely value outside bounds, unsupported dependencies, cyclic scenario references, and output nodes absent from the graph. Zero and negative values are allowed only when the underlying typed domain permits them.

Do not infer independence. A model without dependency evidence either uses named coherent scenarios or explicitly records an independence assumption and warning. Mutually exclusive states use categorical/scenario selection. Shared uncertain sources are sampled once per trial and referenced by dependent nodes.

## Numeric and reproducibility policy

Scenario runs are exactly reproducible from canonical inputs. Sampled runs pin RNG algorithm/version, seed, sample count, ordering, sampler version, and worker-count-independent stream policy. Cross-platform tests establish bitwise or documented numeric-tolerance expectations. Authoritative currency values cross the existing fixed-decimal conversion/rounding boundaries; quantile interpolation and display rounding are named, versioned policies.

Quantiles use a documented estimator and stable ordering. Expected values are computed only from probability-bearing scenarios/samples. Sensitivity names its method; rank correlation, one-at-a-time deltas, and variance-based measures are not interchangeable. A change in sampler, quantile estimator, sensitivity method, clamping, or correlation semantics requires a new algorithm/rule version.

Invalid trials are never silently dropped. The result is `Failed` or `Incomplete` unless a versioned policy permits a disclosed invalid fraction for research; such output cannot support approval by default.

## Performance and failure containment

Limits cover scenario/sample count, graph evaluations, outputs, memory, CPU/wall time, and nested capacity calls. Deterministic chunking permits cancellation. Cancellation returns partial diagnostics labeled non-authoritative; it cannot publish requested quantiles as complete. Derived samples may stream through bounded summary structures, but replay/retention policy decides whether raw samples are required.

No-model and incomplete-model cases return the deterministic estimate with `UncertaintyStatus::NotModeled` or typed blockers. Engine failure never erases or modifies the baseline graph.

## Explainability and persistence

For each output, return method, units/basis, range and quantiles, baseline comparison, varied/fixed assumptions, source/provenance, dominant drivers, dependencies, excluded unknowns, warnings, and trace links to deterministic rule nodes. A percentile is always associated with one model revision and route/quantity scenario.

Approved estimates retain the uncertainty model, evaluation manifest, result summary, sensitivity, digests, overrides, and validation status. If raw samples are not retained, deterministic replay from the manifest must reproduce the published summary within the acceptance policy. New methods create sibling results and diffs; historical quotes remain unchanged.

## Security and release stages

Execution is local/in-boundary by default. Inputs inherit estimate classification and access. No external statistics, AI, or analytics service is called. Logs expose safe run counts/version/status only, never customer, geometry, price, schedule, or sampled-value content. Batch APIs return only outputs authorized for the caller.

Foundation defines types and exact scenarios. Initial production may show reviewed low/likely/high values and deterministic sensitivity. Intermediate adds validated three-point families. Seeded simulation, probabilistic capacity/delivery, calibrated distributions, and learned suggestions remain advanced/research.

Acceptance is governed by [uncertainty validation](../06-quality/uncertainty-validation.md), including exact fixtures, properties, seeded replay, statistical method checks, calibration qualification, resource limits, permission/log tests, and human comprehension.
