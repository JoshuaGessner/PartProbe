# Feeds and Speeds Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-007, REQ-F-017, REQ-NF-003, TIME-003  
> **Related architecture decision IDs:** ADR-0007  
> **Open questions:** Which tool families, coatings, holders, coolant modes, and material conditions are standard in the target shop?  
> **Dependencies:** tool, material, machine, and operation-template libraries  
> **Supersedes:** None

## Recommendation

Use a versioned, deterministic recommendation library with shop-owned policies. A recommendation is a range plus a selected value and rationale; it is not a universal feed/speed table, CAM output, or operating instruction.

The canonical schema is defined in `02-domain/feeds-and-speeds-model.md`.

## Inputs that must participate in selection

1. Operation and strategy: face/shoulder/slot/pocket/profile, rough/semi/finish, drilling/peck/ream/tap/boring, turning/grooving/threading.
2. Work material specification, condition, hardness range, stock form, and machinability note.
3. Tool identity or class: substrate, grade, coating, geometry, diameter, flute/effective-edge count, reach/stickout, holder, and condition.
4. Engagement: axial/radial depth, lead/approach angle, interrupted cut, path curvature, hole depth/diameter, and coolant/chip evacuation.
5. Workholding/setup rigidity and accessibility.
6. Machine speed, feed, power, torque, kinematics, and coolant capability.

## Equations and units

Use canonical SI internally; preserve original entered units and conversion provenance.

| Operation | Equations | Notes |
| --- | --- | --- |
| Milling | `n = 1000vc/(πD)`; `vf = fz n zc` | `zc` is effective teeth, not nominal flutes; engagement correction must be explicit. |
| Turning | `vc = πDn/1000`; `vf = fn n` | D is instantaneous or segmented diameter under CSS. |
| Drilling | `vf = fn n`; in-cut time derives from full travel and feed | add approach, breakthrough, pecks and retracts visibly. |
| Tapping | feed synchronizes with thread pitch × RPM | require controller/process capability; no inferred rigid-tap promise. |
| MRR | milling `ap ae vf / 1000` cm³/min | use only with documented engagement/coverage. |

Sandvik publishes these milling relationships, including MRR, net power, and torque, while Kennametal describes the equivalent inch-based relationships. They are vendor guidance; tool-maker recommendations and shop trials may supersede them by a governed version. [Sandvik formulas](https://cdn.sandvik.coromant.com/files/sitecollectiondocuments/services/metal-cutting-e-learning/formulas-and-definitions/formulas-and-deinitions-for-milling-metric-enu.pdf), [Kennametal FAQ](https://www.kennametal.com/us/en/resources/engineering-calculators/miscellaneous/speed-and-feed.html) (accessed 2026-07-22).

## Guardrails

- Return `recommended`, `conservative`, and `aggressive` profiles only when a shop has approved all three. Default new/unknown conditions to conservative.
- Evaluate speed, feed, power, torque, tool reach/deflection proxy, and machine limits independently. Explain each limiter and selected derating.
- Do not derive tool life solely from material removal. Tool-life allowance is an explicit library model or manual input, calibrated from actual usage.
- Require human approval for deep holes, thin-wall/long-reach work, exotic material, hard/heat-treated condition, unstable fixturing, interrupted cuts, customer-defined process limits, or conflicting sources.
- Keep source type: `shop_test`, `tool_vendor`, `machine_vendor`, `historical_actual`, `manual`, or `derived`. Vendor and historical data are not automatically authoritative.

## Data stewardship

Every version is immutable after approval and contains effective date, approver, source links/documents, unit system, selection predicates, calculations, rounding, and change reason. An estimate snapshots the exact version; re-analysis may offer a new result but never replaces an approved estimate. Vendor guidance must retain manufacturer, catalog/application revision, accessed date, and applicability limits.

## Research basis

- Kennametal explicitly identifies steel hardness, heat treatment, alloy composition, tool material/condition, coolant, and fixture stability as factors affecting selection. [Kennametal speed/feed FAQ](https://www.kennametal.com/us/en/resources/engineering-calculators/miscellaneous/speed-and-feed.html) (accessed 2026-07-22).
- Sandvik gives formula definitions but does not make them a universal process approval; record the manufacturer source and application scope. [Sandvik formula reference](https://cdn.sandvik.coromant.com/files/sitecollectiondocuments/services/metal-cutting-e-learning/formulas-and-definitions/formulas-and-deinitions-for-milling-inch-enu.pdf) (accessed 2026-07-22).
