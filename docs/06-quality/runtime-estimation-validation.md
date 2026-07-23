# Runtime Estimation Validation

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** TIME-001–TIME-008, TEST-005, TEST-019  
> **Related architecture decision IDs:** ADR-0007, ADR-0008  
> **Open questions:** Acceptance bands by workcenter/process, actual-time source, and who approves calibration versions  
> **Dependencies:** runtime engine, feeds/speeds library, fixtures, actuals/revision data  
> **Supersedes:** None

## Purpose

Validate calculation reproducibility, error characterization, and calibration governance. Do not treat an aggregate accuracy number as proof of suitability for all parts.

## Test corpus

Maintain reviewed, sanitized fixtures across: prismatic milling; pockets/holes/deep holes; simple and multi-operation turning; live-tool/mill-turn; Swiss/bar feed; fixture-heavy/tight-tolerance work; high-removal aerospace material; mesh-only/ambiguous units; difficult access; and quantities 1, small batch, repeat. Each case snapshots model hash/revision, material condition, stock, machine/tool/fixture data, setup plan, feeds/speeds library version, formula version, expected component times, reviewed estimate, actual source/scope, and exclusions.

## Test types

| Test | Acceptance evidence |
| --- | --- |
| Unit/equation | conversions, RPM/feed/MRR, route summation, rounding boundaries, zero/negative rejection, machine caps |
| Golden calculation | exact deterministic intermediate trace and final components for a fixed snapshot |
| Property | more positive time/cost cannot lower result absent named nonlinear rule; approved routing is not overwritten; limits produce warnings |
| Differential | compare revised calculation/library against prior approved cases; classify all changes |
| Actual comparison | same scope/revision only; report bias and percentiles per segment, with outlier investigation |
| Human review | estimator confirms routing/setup/tool/access and reasons for variance before calibration |

## Metrics and reporting

Report cutting, non-cutting, attended, setup/prove-out, programming, and inspection separately. By segment report sample count, median signed error, median absolute error, selected percentile bands, bias, and error contributors. Use absolute-percent error only with an adequate nonzero denominator; retain elapsed time and scope changes for interpretation. Never pool mesh-only, unknown-material, or unreviewed cases with validated machine-specific cases without labels.

## Calibration change control

1. Ingest actuals with source, scope, reviewer, and variance classification.
2. Propose—not apply—a new rate/template/library version.
3. Run differential and holdout validation, preserving all old golden results.
4. Obtain designated shop approval; publish immutable version, effective date, rationale, and rollback reference.
5. New estimates may select it; approved estimates stay pinned and only receive a comparison.

## Release gate

No runtime model/library release if it is nondeterministic, lacks trace/versions, changes a golden result without documented expected reason, masks a machine/tool constraint, or lacks required human-review flags. “Production CAM accurate” is never a release claim.

References: [runtime research](../01-research/runtime-estimation.md), [feeds/speeds model](../02-domain/feeds-and-speeds-model.md).
