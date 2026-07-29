use partprobe_domain::money::{
    checked_decimal_add_exact, checked_decimal_mul_exact, checked_decimal_sub_exact,
};
use partprobe_domain::{
    CostCategory, CurrencyCode, DensityKilogramsPerCubicMillimeter, DomainError, ItemQuantity,
    MassKilograms, Money, PricingMethod, PricingPolicy, PricingPolicyId, RateBasis,
    RateComposition, RateVersion, RoundedMoney, VolumeCubicMillimeters,
};
use rust_decimal::Decimal;
use std::collections::BTreeSet;

use crate::{CalculationError, ResolvedRate, extend_rate};

/// A rule output plus stable review warnings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleOutcome<T> {
    /// Calculated value.
    pub value: T,
    /// Conditions that do not invalidate the value but require review.
    pub warnings: Vec<String>,
}

/// Geometry evidence needed to qualify removed-volume output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeometryBasis {
    /// Whether the analyzed body is enclosed.
    pub enclosed: bool,
    /// Whether multi-body interpretation has been resolved.
    pub multi_body_resolved: bool,
}

/// Explicit components of CALC-007 material cost.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterialCostComponents {
    /// Purchased stock extended cost.
    pub purchased: Money,
    /// Supplier or internal cut charge.
    pub cut: Money,
    /// Certificate charge.
    pub certificate: Money,
    /// Inbound freight.
    pub inbound_freight: Money,
    /// Approved nonnegative remnant credit.
    pub approved_remnant_credit: Money,
}

/// Explicit components of CALC-012 operation cost.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationCostComponents {
    /// Lot setup.
    pub setup: Money,
    /// Programming.
    pub programming: Money,
    /// Prove-out.
    pub prove_out: Money,
    /// Recurring run.
    pub run: Money,
    /// Tooling.
    pub tooling: Money,
    /// Consumables.
    pub consumables: Money,
    /// Fixture.
    pub fixture: Money,
    /// Quality and inspection.
    pub quality_inspection: Money,
    /// Outside processing.
    pub outside: Money,
    /// Operation-specific freight.
    pub freight: Money,
}

/// Explicit components of CALC-013 base internal cost.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseInternalCostComponents {
    /// Material cost.
    pub material: Money,
    /// Ordered operation totals.
    pub operations: Vec<Money>,
    /// Nonrecurring engineering.
    pub nonrecurring_engineering: Money,
    /// Administrative effort.
    pub administration: Money,
    /// Explicit overhead allocation.
    pub overhead: Money,
}

/// Result of applying a versioned pricing policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricingOutcome {
    /// Pinned pricing-policy ID.
    pub pricing_policy_id: PricingPolicyId,
    /// Pinned pricing-policy version.
    pub pricing_policy_version: RateVersion,
    /// Direct formula output before floors and minimums.
    pub formula_price: Money,
    /// Price after configured floors and minimums, before rounding.
    pub governed_price: Money,
    /// Quote-total rounding trace.
    pub rounded_price: RoundedMoney,
    /// Whether the configured floor changed the result.
    pub floor_applied: bool,
    /// Whether the configured minimum order changed the result.
    pub minimum_order_applied: bool,
}

/// CALC-001: mass equals volume times density after explicit unit conversion.
pub fn part_mass(
    volume: VolumeCubicMillimeters,
    density: DensityKilogramsPerCubicMillimeter,
) -> Result<MassKilograms, CalculationError> {
    let value = checked_decimal_mul_exact(volume.value(), density.value(), "part mass")?;
    Ok(MassKilograms::new(value)?)
}

/// CALC-003: removed volume is clamped to zero with explicit evidence warnings.
pub fn removed_volume(
    stock: VolumeCubicMillimeters,
    net_part: VolumeCubicMillimeters,
    basis: GeometryBasis,
) -> Result<RuleOutcome<VolumeCubicMillimeters>, CalculationError> {
    let mut warnings = Vec::new();
    if !basis.enclosed {
        warnings.push("source geometry is not enclosed".to_owned());
    }
    if !basis.multi_body_resolved {
        warnings.push("multi-body interpretation is unresolved".to_owned());
    }

    let value = if stock.value() < net_part.value() {
        warnings.push("net part volume exceeds stock volume; result clamped to zero".to_owned());
        Decimal::ZERO
    } else {
        checked_decimal_sub_exact(stock.value(), net_part.value(), "removed volume")?
    };

    Ok(RuleOutcome {
        value: VolumeCubicMillimeters::new(value)?,
        warnings,
    })
}

/// CALC-005: make quantity equals deliver quantity plus spares and destructive samples.
pub fn make_quantity(
    deliver: ItemQuantity,
    planned_spares: ItemQuantity,
    destructive_samples: ItemQuantity,
) -> Result<ItemQuantity, CalculationError> {
    Ok(deliver
        .checked_add(planned_spares)?
        .checked_add(destructive_samples)?)
}

/// CALC-007: purchased material plus explicit charges minus approved remnant credit.
pub fn material_cost(components: &MaterialCostComponents) -> Result<Money, CalculationError> {
    for (field, value) in [
        ("purchased material", &components.purchased),
        ("cut charge", &components.cut),
        ("certificate charge", &components.certificate),
        ("inbound freight", &components.inbound_freight),
    ] {
        ensure_nonnegative_money(value, field)?;
    }
    if components
        .approved_remnant_credit
        .amount()
        .is_sign_negative()
    {
        return Err(DomainError::InvalidValue {
            field: "approved remnant credit",
            reason: "must be nonnegative",
        }
        .into());
    }
    let subtotal = components
        .purchased
        .checked_add(&components.cut)?
        .checked_add(&components.certificate)?
        .checked_add(&components.inbound_freight)?;
    let result = subtotal.checked_sub(&components.approved_remnant_credit)?;
    if result.amount().is_sign_negative() {
        return Err(DomainError::InvalidValue {
            field: "material cost",
            reason: "approved remnant credit must not exceed sourced material charges",
        }
        .into());
    }
    Ok(result)
}

/// CALC-008: setup hours multiplied by the selected approved setup rate.
pub fn setup_lot_cost(
    setup_hours: Decimal,
    rate: &ResolvedRate,
) -> Result<Money, CalculationError> {
    ensure_rate_category(rate, CostCategory::SetupLabor)?;
    extend_rate(
        rate,
        RateBasis::PerHour,
        nonnegative(setup_hours, "setup hours")?,
    )
}

/// CALC-009: setup lot cost divided by deliver quantity for presentation.
pub fn setup_unit_cost(
    setup_lot_cost: &Money,
    deliver_quantity: ItemQuantity,
) -> Result<Money, CalculationError> {
    if deliver_quantity.value() == 0 {
        return Err(DomainError::InvalidValue {
            field: "deliver quantity",
            reason: "must be greater than zero for setup amortization",
        }
        .into());
    }
    Ok(setup_lot_cost.checked_div_exact(Decimal::from(deliver_quantity.value()))?)
}

/// CALC-010: each classified cycle-time occurrence contributes exactly once.
pub fn cycle_time(
    cutting_hours: Decimal,
    non_cutting_hours: Decimal,
    load_unload_hours: Decimal,
    in_cycle_probe_or_inspection_hours: Decimal,
) -> Result<Decimal, CalculationError> {
    [
        ("cutting hours", cutting_hours),
        ("non-cutting hours", non_cutting_hours),
        ("load/unload hours", load_unload_hours),
        (
            "in-cycle probe or inspection hours",
            in_cycle_probe_or_inspection_hours,
        ),
    ]
    .into_iter()
    .try_fold(Decimal::ZERO, |total, (field, value)| {
        Ok(checked_decimal_add_exact(
            total,
            nonnegative(value, field)?,
            "cycle time",
        )?)
    })
}

/// CALC-011: cycle time times make quantity times each explicitly applicable hourly rate.
pub fn run_cost(
    cycle_hours_per_item: Decimal,
    make_quantity: ItemQuantity,
    rates: &[ResolvedRate],
) -> Result<Money, CalculationError> {
    if rates.is_empty() {
        return Err(DomainError::InvalidValue {
            field: "run rates",
            reason: "at least one explicit run, machine, or burden rate is required",
        }
        .into());
    }
    let hours_per_item = nonnegative(cycle_hours_per_item, "cycle hours per item")?;
    let total_hours = checked_decimal_mul_exact(
        hours_per_item,
        Decimal::from(make_quantity.value()),
        "total run hours",
    )?;
    let currency = rates[0].entry.amount().currency();
    let mut total = Money::new(Decimal::ZERO, currency.clone());
    let mut entry_ids = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let has_composite = rates
        .iter()
        .any(|rate| rate.entry.composition() == RateComposition::Composite);
    if has_composite && rates.len() != 1 {
        return Err(DomainError::InvalidValue {
            field: "run rate composition",
            reason: "a composite rate must be extended alone",
        }
        .into());
    }
    for rate in rates {
        if !entry_ids.insert(rate.entry.id().clone()) {
            return Err(DomainError::InvalidValue {
                field: "run rates",
                reason: "the same rate entry must not be charged more than once",
            }
            .into());
        }
        if !categories.insert(rate.entry.category()) {
            return Err(DomainError::InvalidValue {
                field: "run rate category",
                reason: "each cost category must be charged at most once",
            }
            .into());
        }
        if !matches!(
            rate.entry.category(),
            CostCategory::RunLabor | CostCategory::Machine | CostCategory::Burden
        ) {
            return Err(DomainError::InvalidValue {
                field: "run rate category",
                reason: "must be run labor, machine, or burden",
            }
            .into());
        }
        total = total.checked_add(&extend_rate(rate, RateBasis::PerHour, total_hours)?)?;
    }
    Ok(total)
}

/// CALC-012: sums each explicitly classified operation component once.
pub fn operation_cost(components: &OperationCostComponents) -> Result<Money, CalculationError> {
    sum_money(
        components.setup.currency(),
        [
            &components.setup,
            &components.programming,
            &components.prove_out,
            &components.run,
            &components.tooling,
            &components.consumables,
            &components.fixture,
            &components.quality_inspection,
            &components.outside,
            &components.freight,
        ],
    )
}

/// CALC-013: material plus operations, NRE, administration, and explicit overhead.
pub fn base_internal_cost(
    components: &BaseInternalCostComponents,
) -> Result<Money, CalculationError> {
    ensure_nonnegative_money(&components.material, "material cost")?;
    for operation in &components.operations {
        ensure_nonnegative_money(operation, "operation cost")?;
    }
    ensure_nonnegative_money(
        &components.nonrecurring_engineering,
        "nonrecurring engineering",
    )?;
    ensure_nonnegative_money(&components.administration, "administration")?;
    ensure_nonnegative_money(&components.overhead, "overhead")?;
    let mut total = components.material.clone();
    for operation in &components.operations {
        total = total.checked_add(operation)?;
    }
    Ok(total
        .checked_add(&components.nonrecurring_engineering)?
        .checked_add(&components.administration)?
        .checked_add(&components.overhead)?)
}

/// CALC-014: sum of separately accepted monetary risk impacts.
pub fn risk_reserve(
    currency: &CurrencyCode,
    accepted_impacts: &[Money],
) -> Result<Money, CalculationError> {
    accepted_impacts.iter().try_fold(
        Money::new(Decimal::ZERO, currency.clone()),
        |total, impact| {
            if impact.amount().is_sign_negative() {
                return Err(DomainError::InvalidValue {
                    field: "risk impact",
                    reason: "must be nonnegative",
                }
                .into());
            }
            Ok(total.checked_add(impact)?)
        },
    )
}

/// CALC-015: base cost plus explicit risk reserve and expected rework.
pub fn total_internal_cost(
    base_cost: &Money,
    risk_reserve: &Money,
    expected_rework: &Money,
) -> Result<Money, CalculationError> {
    ensure_nonnegative_money(base_cost, "base internal cost")?;
    ensure_nonnegative_money(risk_reserve, "risk reserve")?;
    ensure_nonnegative_money(expected_rework, "expected rework")?;
    Ok(base_cost
        .checked_add(risk_reserve)?
        .checked_add(expected_rework)?)
}

/// CALC-016: price from a markup rate greater than or equal to negative one.
pub fn price_from_markup(cost: &Money, markup_rate: Decimal) -> Result<Money, CalculationError> {
    if markup_rate < Decimal::NEGATIVE_ONE {
        return Err(DomainError::InvalidValue {
            field: "markup rate",
            reason: "must be greater than or equal to -1",
        }
        .into());
    }
    let factor = checked_decimal_add_exact(Decimal::ONE, markup_rate, "markup factor")?;
    Ok(cost.checked_mul(factor)?)
}

/// CALC-017: price from a target margin strictly less than one.
pub fn price_from_margin(cost: &Money, target_margin: Decimal) -> Result<Money, CalculationError> {
    if target_margin >= Decimal::ONE {
        return Err(DomainError::InvalidValue {
            field: "target margin",
            reason: "must be less than 1",
        }
        .into());
    }
    let divisor = checked_decimal_sub_exact(Decimal::ONE, target_margin, "margin divisor")?;
    Ok(cost.checked_div_exact(divisor)?)
}

/// CALC-018: applies one versioned pricing method, thresholds, and quote-total rounding.
pub fn apply_pricing_policy(
    cost: &Money,
    policy: &PricingPolicy,
) -> Result<PricingOutcome, CalculationError> {
    if cost.currency() != policy.currency() {
        return Err(DomainError::CurrencyMismatch {
            left: cost.currency().as_str().to_owned(),
            right: policy.currency().as_str().to_owned(),
        }
        .into());
    }
    let formula_price = match policy.method() {
        PricingMethod::Markup { rate } => price_from_markup(cost, *rate)?,
        PricingMethod::TargetMargin { rate } => price_from_margin(cost, *rate)?,
    };
    let mut governed_price = formula_price.clone();
    let mut floor_applied = false;
    let mut minimum_order_applied = false;
    if let Some(floor) = policy.floor()
        && floor.amount() > governed_price.amount()
    {
        governed_price = floor.clone();
        floor_applied = true;
    }
    if let Some(minimum) = policy.minimum_order()
        && minimum.amount() > governed_price.amount()
    {
        governed_price = minimum.clone();
        minimum_order_applied = true;
    }
    let rounded_price = policy.quote_total_rounding().apply(&governed_price)?;
    Ok(PricingOutcome {
        pricing_policy_id: policy.id().clone(),
        pricing_policy_version: policy.version(),
        formula_price,
        governed_price,
        rounded_price,
        floor_applied,
        minimum_order_applied,
    })
}

fn ensure_rate_category(
    rate: &ResolvedRate,
    expected: CostCategory,
) -> Result<(), CalculationError> {
    if rate.entry.category() == expected {
        return Ok(());
    }
    Err(DomainError::InvalidValue {
        field: "rate category",
        reason: "resolved rate category does not match the calculation rule",
    }
    .into())
}

fn nonnegative(value: Decimal, field: &'static str) -> Result<Decimal, CalculationError> {
    if value.is_sign_negative() {
        return Err(DomainError::InvalidValue {
            field,
            reason: "must be nonnegative",
        }
        .into());
    }
    Ok(value)
}

fn sum_money<'a>(
    currency: &CurrencyCode,
    values: impl IntoIterator<Item = &'a Money>,
) -> Result<Money, CalculationError> {
    values.into_iter().try_fold(
        Money::new(Decimal::ZERO, currency.clone()),
        |total, value| {
            ensure_nonnegative_money(value, "operation cost component")?;
            Ok(total.checked_add(value)?)
        },
    )
}

fn ensure_nonnegative_money(value: &Money, field: &'static str) -> Result<(), CalculationError> {
    if value.amount().is_sign_negative() {
        return Err(DomainError::InvalidValue {
            field,
            reason: "must be nonnegative",
        }
        .into());
    }
    Ok(())
}
