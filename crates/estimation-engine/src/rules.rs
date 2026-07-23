use partprobe_domain::money::{
    checked_decimal_add_exact, checked_decimal_mul_exact, checked_decimal_sub_exact,
};
use partprobe_domain::{
    DensityKilogramsPerCubicMillimeter, DomainError, ItemQuantity, MassKilograms, Money,
    VolumeCubicMillimeters,
};
use rust_decimal::Decimal;

use crate::CalculationError;

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
