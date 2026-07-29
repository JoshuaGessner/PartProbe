use partprobe_domain::{
    CostCategory, EffectiveDate, Money, RateBasis, RateCard, RateCardId, RateEntry, RateScope,
    RateVersion, ValueState,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::CalculationError;

/// Exact approved rate selected from a pinned card version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResolvedRate {
    /// Stable selector behavior ID.
    pub selector_id: String,
    /// Pinned selector behavior version.
    pub selector_version: partprobe_domain::RuleVersion,
    /// Pinned rate-card ID.
    pub card_id: RateCardId,
    /// Pinned rate-card version.
    pub card_version: RateVersion,
    /// Selected immutable entry.
    pub entry: RateEntry,
    /// Date against which the effective interval was evaluated.
    pub effective_on: EffectiveDate,
    /// Complete caller-declared scope order used by the selector.
    pub requested_scopes: Vec<RateScope>,
    /// Zero-based position in the caller-declared scope order.
    pub scope_rank: usize,
    /// Stable explanation for the selection trace.
    pub reason: String,
}

/// Resolves one approved effective rate without guessing or hidden fallback.
#[must_use]
pub fn resolve_rate(
    card: &RateCard,
    category: CostCategory,
    basis: RateBasis,
    effective_on: &EffectiveDate,
    ordered_scopes: &[RateScope],
) -> ValueState<ResolvedRate> {
    for (scope_rank, scope) in ordered_scopes.iter().enumerate() {
        let matching = card
            .entries()
            .iter()
            .filter(|entry| {
                entry.category() == category
                    && entry.basis() == basis
                    && entry.scope() == scope
                    && entry.is_approved_and_effective(effective_on)
            })
            .collect::<Vec<_>>();

        match matching.as_slice() {
            [] => {}
            [entry] => {
                return ValueState::available(ResolvedRate {
                    selector_id: "ordered_scope_approved_effective".to_owned(),
                    selector_version: partprobe_domain::RuleVersion::new(1, 0, 0),
                    card_id: card.id().clone(),
                    card_version: card.version(),
                    entry: (*entry).clone(),
                    effective_on: effective_on.clone(),
                    requested_scopes: ordered_scopes.to_vec(),
                    scope_rank,
                    reason: format!(
                        "selected the sole approved effective {:?} rate at scope rank {scope_rank}",
                        category
                    ),
                });
            }
            _ => {
                let ids = matching
                    .iter()
                    .map(|entry| entry.id().as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return ValueState::Blocked {
                    reason: format!(
                        "multiple approved effective {:?} rates match scope rank {scope_rank}: {ids}",
                        category
                    ),
                };
            }
        }
    }

    ValueState::Unavailable {
        reason: format!(
            "no approved {:?} rate with basis {:?} is effective on {} in the requested scopes",
            category,
            basis,
            effective_on.as_str()
        ),
    }
}

/// Extends a resolved rate by an exact nonnegative quantity in its declared basis.
pub fn extend_rate(
    rate: &ResolvedRate,
    expected_basis: RateBasis,
    quantity: Decimal,
) -> Result<Money, CalculationError> {
    if rate.entry.basis() != expected_basis {
        return Err(partprobe_domain::DomainError::InvalidValue {
            field: "rate basis",
            reason: "resolved rate basis does not match the calculation input",
        }
        .into());
    }
    if quantity.is_sign_negative() {
        return Err(partprobe_domain::DomainError::InvalidValue {
            field: "rate quantity",
            reason: "must be nonnegative",
        }
        .into());
    }
    Ok(rate.entry.amount().checked_mul(quantity)?)
}
