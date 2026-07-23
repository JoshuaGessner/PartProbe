# Runtime Estimation Research

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirement IDs:** REQ-F-006–REQ-F-008, REQ-NF-003, TIME-001–TIME-008  
> **Related architecture decision IDs:** ADR-0007, ADR-0008  
> **Open questions:** Which controllers/machines expose measured rapid, tool-change, probe, and spindle-ramp times?  
> **Dependencies:** `01-research/feeds-and-speeds.md`, `02-domain/feeds-and-speeds-model.md`, routing and machine libraries  
> **Supersedes:** None

## Position

Runtime is a reproducible estimate, not production CAM verification and not a safety assertion. Emit cutting, non-cutting, attended, and elapsed components separately, with method, inputs, library versions, rounding, confidence, and warnings.

## Calculation layers

| Layer | Method | Confidence ceiling |
| --- | --- | --- |
| Coarse removal | removable volume ÷ approved baseline MRR, plus named allowances | Low; no feature/tool access certainty |
| Feature/operation | path length or feature volume divided by feed/MRR; cycle elements per operation | Medium only after reviewable features and machine/tool data |
| Program-informed | imported CAM/NC summary or reviewed path segments mapped to machine kinematics | High only when scope, post, machine, and revision match |
| Actual-informed | historical actual distribution used as a comparison or explicitly approved adjustment | Does not replace calculation without a new versioned rule |

## Runtime decomposition

`cycle_elapsed = cutting + rapid/positioning + tool_change + spindle/coolant state + probing + indexing + transfer + dwell + operator_required_cycle_events`.

`job_elapsed = programming + setup/prove_out + cycle_elapsed × make_quantity + post_cycle_operator_inspection + quality_inspection_elapsed + outside_process_lead + queue/contingency`, where queue and contingency are business-policy values, not machine cycle time. `cycle_elapsed` may contain in-cycle probing/inspection; `post_cycle_operator_inspection` is off-machine/operator work after a cycle; `quality_inspection_elapsed` is a separate quality/CMM/FAI operation. A time occurrence belongs to exactly one term even when separate machine and labor cost rates apply.

Every term is zero only when the selected operation template explicitly says so. Avoid a global percentage “overhead” inside cycle time because it destroys diagnostics.

### Required physical constraints

- Clamp commanded speed to machine spindle maximum and account for constant-surface-speed diameter changes in turning.
- Clamp feed by machine axis/feed capability, tool/process policy, power/torque estimate, stability, and an explicit derating rule. A constraint hit is a warning, not an invisible recalculation.
- Distinguish programmed rapid from effective rapid: use a machine-profile acceleration/deceleration model where known; otherwise a conservative, versioned effective rapid rate.
- Attribute tool-change and probe/index times to a specific machine configuration. Published values are configuration claims, not defaults for every machine; Haas, for example, advertises a specific DC-series tool-to-tool time and separately describes pre-staging as affecting non-cut time. [Haas DC tool changer](https://www.haascnc.com/productivity/tool-changer/tc-21.html), [Haas side-mount changer](https://www.haascnc.com/productivity/tool-changer/smtc-20.html) (vendor sources accessed 2026-07-22).

## Base equations (metric)

- Milling: `n = 1000 × vc / (π × D)`; `vf = fz × n × effective_teeth`; `MRR = ap × ae × vf / 1000` in cm³/min.
- Linear cut: `t = length / vf`.
- Volume removal: `t = removed_volume / MRR`, with coverage/engagement factor recorded rather than hidden.
- Drilling: `vf = fn × n`; include approach, breakthrough, pecks/retracts, chip-clear, and depth allowance as separate terms.
- Turning: use work diameter for surface speed, `vf = fn × n`, integrate or segment when CSS changes materially by diameter.

Sandvik publishes equivalent milling equations and power/torque relationships; use them as cited vendor guidance and bind any constants to the material/tool library version. [Sandvik milling formulas](https://cdn.sandvik.coromant.com/files/sitecollectiondocuments/services/metal-cutting-e-learning/formulas-and-definitions/formulas-and-deinitions-for-milling-metric-enu.pdf)

## Required inputs and fallbacks

| Input absent | Safe fallback | Warning |
| --- | --- | --- |
| Drawing/tolerance | broad operation class only | Inspection and finishing cannot be determined from model alone |
| Exact solid | no exact volume/feature confidence | mesh-only runtime cap and manual review |
| Machine profile | named generic profile | no machine-specific kinematic claim |
| Tool data | shop-approved conservative template | tool/holder and engagement unverified |
| Setup/fixture | count/effort proposed, not confirmed | access and rigidity uncertain |
| Material condition | family baseline only | heat treatment/hardness unknown |

## Acceptance targets and analysis

Do not promise one accuracy percentage. Segment validation by process, material family, machine, quantity, and estimation level. Measure signed error, absolute-percent error only where actual time is nonzero and meaningful, median, 80th/90th percentile, and systematic bias. Review outliers for scope changes before using them to calibrate. Store golden examples immutably with calculation/library versions.

## Research basis

- Vendor data identifies chip load as material removed per cutting edge and expresses milling feed as RPM × chip load × teeth; it also calls out material hardness, tool condition, coolant, and fixture stability as inputs. Treat this as vendor application guidance. [Kennametal speed/feed FAQ](https://www.kennametal.com/us/en/resources/engineering-calculators/miscellaneous/speed-and-feed.html) (accessed 2026-07-22).
- Kennametal’s holemaking calculator labels outputs as theoretical/planning values and includes torque, thrust, power, feed, and in-cut time; runtime must therefore retain constraints and uncertainty rather than report false precision. [Kennametal drilling calculator](https://www.kennametal.com/be/fr/resources/engineering-calculators/holemaking-calculators/torque--thrust--and-power.html) (accessed 2026-07-22).
