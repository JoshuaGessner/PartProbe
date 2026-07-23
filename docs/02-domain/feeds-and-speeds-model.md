# Feeds and Speeds Domain Model

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-007, REQ-F-017, TIME-003, DATA-017  
> **Related architecture decision IDs:** ADR-0007  
> **Open questions:** Canonical material/tool taxonomy, approval roles, and initial shop-library migration source  
> **Dependencies:** material, tool, machine, operation, units, and audit models  
> **Supersedes:** None

## Canonical model

`FeedsSpeedsLibraryVersion` is immutable after approval.

| Entity | Required fields |
| --- | --- |
| `LibraryVersion` | id, semantic version, status, effective interval, parent version, change reason, approver, source manifest, units convention, hash |
| `RecommendationRule` | id, operation/strategy predicates, material condition predicates, tool/holder predicates, engagement bounds, machine capability predicates, profile, outputs, limits, precedence, source refs |
| `Recommendation` | selected rule/version, vc, rpm, fz/fn, feed, ap/ae, profile, derivation trace, constraint results, confidence, warnings |
| `MachineCapabilityProfile` | spindle speed/power/torque curves, axis/feed/rapid limits, coolant, tool-change/probe/index values, evidence/date |
| `ToolAssembly` | cutter and holder identity, geometry, effective teeth, reach/stickout, condition, approved usage envelope |
| `MaterialCondition` | specification/family, condition/heat treatment, hardness range, stock form, traceability state |
| `Override` | original value/trace, replacement, reason, actor, time, approval requirement/status |

Numeric values are fixed-precision decimals with dimensioned units; retain original input unit and conversion rule. No floating-point-only business boundary.

## Evaluation order

1. Validate operation, material, tool assembly, machine, units, and requested profile.
2. Find matching rules in the snapshot version; deterministic order is priority, specificity, then stable rule ID. Ambiguous equal-priority matches fail with a review warning.
3. Calculate base speed/feed from the selected rule and equations; record every intermediate value.
4. Apply only named, ordered modifiers: material condition, engagement/chip-thinning, reach/rigidity, coolant/chip evacuation, machine cap, conservative/aggressive profile.
5. Evaluate constraints independently. Return selected values, limiter list, and prohibited/needs-review status.
6. Apply documented rounding at output/presentation only. Snapshot all inputs and source version to the estimate.

## Formula trace

For milling, canonical metric relationships are `n = 1000vc/(πD)` and `vf = fz×n×effective_teeth`; for turning/drilling use feed per revolution × RPM. Store formula ID, symbol values, unit conversions, rounding, and source. The authoritative calculation behavior must eventually live in `calculation-rules.md`; this document owns the data model, not a second copy of rules.

## Source and governance policy

Source manifest entries carry `shop_test`, `tool_vendor`, `machine_vendor`, `historical_actual`, `manual`, or `derived`, plus publisher, reference/revision, access date, scope, and reviewer. Vendor material is guidance and cannot overwrite shop-approved data automatically. Actuals enter a proposed version only after variance review, approver sign-off, and validation examples.

## Confidence and safety boundaries

- A missing tool assembly, uncertain material condition, unverified holder/stickout, unknown machine profile, or out-of-range engagement returns a visible warning and limits confidence.
- A recommendation has no authority to execute a machine program. Deep holes, long reach, thin walls, tapping, high-power, exotic material, or conflicting constraints require explicit human review.
- Approved quote snapshots retain prior versions; reanalysis may produce a comparison but must never silently replace approved parameters.

## Example result contract

```text
recommendation_id: REC-...
source: rule FS-AL-EM-001 @ library 1.4.0
values: vc=..., rpm=..., fz=..., feed=..., ap=..., ae=...
constraints: [machine_spindle_cap, holder_reach_derate]
status: needs_review
trace: [formula TIME-FS-001, modifier MOD-REACH-002]
```

References: [research: feeds and speeds](../01-research/feeds-and-speeds.md), [research: runtime](../01-research/runtime-estimation.md).
