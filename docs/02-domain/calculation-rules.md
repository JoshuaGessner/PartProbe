# Calculation Rules

> **Status:** In Review  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-F-006–REQ-F-010, REQ-F-040–REQ-F-065, REQ-NF-003, REQ-NF-015–REQ-NF-021; TEST-001–TEST-006, TEST-040–TEST-099  
> **Related ADRs:** ADR-0007, ADR-0009–ADR-0014  
> **Open questions:** OQ-012–OQ-018  
> **Dependencies:** Shop-approved rate policy  
> **Supersedes:** None

All currency uses fixed-precision decimal arithmetic. Geometry and physical calculations may use floating point internally with dimensional types and documented tolerance, but currency conversion occurs through explicit, tested boundaries. Inputs never carry implicit units.

| ID | Rule | Boundary |
|---|---|---|
| CALC-001 | `part_mass = net_volume × material_density` | Convert compatible volume/density units first; display mass rounds only at presentation. |
| CALC-002 | `stock_volume` is shape-specific (box, cylinder, tube, etc.) | Exact dimensions and units required. |
| CALC-003 | `removed_volume = max(stock_volume - net_part_volume, 0)` | Warning if model is not enclosed or multi-body policy unresolved. |
| CALC-004 | `buy_to_fly = stock_mass / delivered_part_mass` | Undefined, not zero, when denominator is zero. |
| CALC-005 | `make_qty = deliver_qty + planned_spares + destructive_samples` | Must be at least deliver quantity. |
| CALC-006 | `expected_input_qty = make_qty / (1 - expected_scrap_rate)` | Scrap model and rounding-to-whole policy must be explicit. |
| CALC-007 | `material_cost = purchased_qty × unit_price + cut + cert + inbound_freight - approved_remnant_credit` | Preserve vendor quote and effective date. |
| CALC-008 | `setup_lot_cost = setup_hours × approved_setup_rate` | Setup rate version required. |
| CALC-009 | `setup_unit_cost = setup_lot_cost / deliver_qty` | Presentation only; total remains authoritative. |
| CALC-010 | `cycle_time = cutting + non_cutting + load_unload + in_cycle_probe_or_inspection` | Each occurrence is classified once. Post-cycle operator inspection and quality/CMM/FAI are separate operation times, not repeated here. |
| CALC-011 | `run_cost = cycle_time × make_qty × applicable_machine/labor rates` | Unattended overlap policy explicit; no silent double count. |
| CALC-012 | `operation_cost = setup + programming + prove_out + run + tooling + consumables + fixture + quality_inspection + outside + freight` | In-cycle inspection is already inside run; post-cycle operator inspection is a separate labor operation; quality/CMM/FAI uses this category. No occurrence may be charged twice. |
| CALC-013 | `base_internal_cost = material + Σ operation_cost + NRE + admin + overhead` | Overhead method is versioned shop policy. |
| CALC-014 | `risk_reserve = Σ accepted risk expected impacts` | Do not use one universal complexity multiplier. |
| CALC-015 | `total_internal_cost = base_internal_cost + risk_reserve + expected_rework` | No hidden contingency. |
| CALC-016 | `price_from_markup = cost × (1 + markup_rate)` | Markup rate ≥ -1; override below floor requires approval. |
| CALC-017 | `price_from_margin = cost / (1 - target_margin)` | Target margin must be < 1. |
| CALC-018 | `selling_price = pricing_policy(total_internal_cost, floor, minimums, adjustments)` | Policy returns detailed trace, never only a scalar. |
| CALC-019 | `lead_time = critical-path calendar duration`, not sum of all durations | Queue assumptions and calendars versioned. |
| CALC-020 | `variance = actual - estimate`; `% variance` undefined when estimate is zero | Compare category by category with version context. |
| CALC-021 | `contribution_amount = selling_price - policy_defined_incremental_or_variable_cost` | Name the included cost basis; never label it gross profit or accounting margin without that policy. |
| CALC-022 | `contribution_per_spindle_hour = contribution_amount / constrained_spindle_hours` | Undefined when hours are zero; scenario and resource class required. |
| CALC-023 | `contribution_per_occupancy_hour = contribution_amount / machine_occupancy_hours` | Occupancy includes configured non-cutting possession; do not substitute cycle time. |
| CALC-024 | `contribution_per_bottleneck_hour = contribution_amount / bottleneck_resource_hours` | Resource, calendar snapshot, and time basis required. |
| CALC-025 | `opportunity_cost = max(credible_displaced_contribution - alternative_recovery, 0)` | Scenario/decision-support value only; never silently added to accounting cost. |
| CALC-026 | `capacity_adjusted_score = versioned_weighted_policy(normalized economics, delivery, risk, confidence, strategic factors)` | Expose every component/weight; score is not an approval. |
| CALC-027 | Three-point uncertain input stores `minimum ≤ most_likely ≤ maximum` and source | Values are scenario bounds, not measured probabilities. |
| CALC-028 | Triangular or PERT expected value uses the explicitly selected distribution and parameters | Method/version required; PERT weighting is configurable and reviewed. |
| CALC-029 | Percentiles/scenarios are produced only by the named deterministic scenario set or seeded simulation | Preserve seed, sample count, correlations/independence assumptions, and numerical error; maintain P10 ≤ P50 ≤ P90. |
| CALC-030 | `revision_delta(category) = new_revision_result(category) - prior_revision_result(category)` | Compare immutable snapshots on a declared price/rate basis; never recalculate the prior approved quote in place. |
| CALC-031 | Incremental, direct-cash, fully burdened, accounting, risk-adjusted, and selling-price values are separate typed totals | A policy explicitly maps categories to each basis; no aliasing. |
| CALC-032 | `make_or_buy_landed_cost = quoted_price + freight + packaging + internal_receiving_quality + expected_risk_cost + approved_other_cash_costs` | Opportunity/capacity and resale pricing remain separate comparison dimensions. |
| CALC-033 | Delivery feasibility compares resource demand intervals/precedence with the selected calendar and availability snapshot | Missing schedules yield `Unavailable`, not feasible. |
| CALC-034 | `readiness_adjusted = theoretical_baseline + explicit_current_availability_deltas` | Preserve both results, as-of time, sources, and freshness; never mutate baseline inputs. |
| CALC-035 | `bid_priority_score = versioned_weighted_policy(factors, blockers)` | Factors/weights/reasons visible; blockers and authorized overrides remain separate from score. |

## Rounding

Do not round intermediate physical values for calculation. Currency nodes retain configured internal decimal scale; round using an explicit strategy only at documented invoice/quote presentation or supplier-charge boundaries. Store the unrounded value, rounded value, scale, and strategy. Negative-zero display is forbidden.

## Overrides and versioning

An override is a new node referencing the original, actor, timestamp, reason, and authorization. Rule changes increment a calculation-rules version; published estimates retain the old version. Migration may rehydrate old results but never silently recalculate an approved estimate.
