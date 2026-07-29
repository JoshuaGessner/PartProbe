use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{CurrencyCode, DomainError, Money, RateVersion};

macro_rules! policy_id {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates the identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must not be empty",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the identifier text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

policy_id!(
    /// Stable identity of a rounding policy.
    RoundingPolicyId,
    "rounding-policy ID"
);
policy_id!(
    /// Stable identity of a pricing policy.
    PricingPolicyId,
    "pricing-policy ID"
);

/// Named boundary at which an exact amount may be rounded.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundingBoundary {
    /// A sourced supplier charge.
    SupplierCharge,
    /// An authoritative internal or quote line.
    LineItem,
    /// The final quote total.
    QuoteTotal,
    /// Display only; the stored authoritative amount remains exact.
    Presentation,
}

/// Explicit midpoint or directed rounding behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundingMode {
    /// Round a midpoint to the nearest even final digit.
    HalfEven,
    /// Round a midpoint away from zero.
    HalfAwayFromZero,
    /// Round a midpoint toward zero.
    HalfTowardZero,
    /// Always round toward zero.
    TowardZero,
    /// Always round away from zero.
    AwayFromZero,
    /// Always round toward negative infinity.
    TowardNegativeInfinity,
    /// Always round toward positive infinity.
    TowardPositiveInfinity,
}

impl RoundingMode {
    fn strategy(self) -> RoundingStrategy {
        match self {
            Self::HalfEven => RoundingStrategy::MidpointNearestEven,
            Self::HalfAwayFromZero => RoundingStrategy::MidpointAwayFromZero,
            Self::HalfTowardZero => RoundingStrategy::MidpointTowardZero,
            Self::TowardZero => RoundingStrategy::ToZero,
            Self::AwayFromZero => RoundingStrategy::AwayFromZero,
            Self::TowardNegativeInfinity => RoundingStrategy::ToNegativeInfinity,
            Self::TowardPositiveInfinity => RoundingStrategy::ToPositiveInfinity,
        }
    }
}

/// Versioned currency rounding policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoundingPolicy {
    id: RoundingPolicyId,
    version: RateVersion,
    currency: CurrencyCode,
    scale: u32,
    mode: RoundingMode,
    boundary: RoundingBoundary,
}

#[derive(Deserialize)]
struct RoundingPolicyWire {
    id: RoundingPolicyId,
    version: RateVersion,
    currency: CurrencyCode,
    scale: u32,
    mode: RoundingMode,
    boundary: RoundingBoundary,
}

impl<'de> Deserialize<'de> for RoundingPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RoundingPolicyWire::deserialize(deserializer)?;
        Self::new(
            wire.id,
            wire.version,
            wire.currency,
            wire.scale,
            wire.mode,
            wire.boundary,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl RoundingPolicy {
    /// Validates and creates a policy.
    pub fn new(
        id: RoundingPolicyId,
        version: RateVersion,
        currency: CurrencyCode,
        scale: u32,
        mode: RoundingMode,
        boundary: RoundingBoundary,
    ) -> Result<Self, DomainError> {
        if scale > Decimal::MAX_SCALE {
            return Err(DomainError::InvalidValue {
                field: "rounding scale",
                reason: "must not exceed the decimal representation maximum",
            });
        }
        Ok(Self {
            id,
            version,
            currency,
            scale,
            mode,
            boundary,
        })
    }

    /// Applies this exact policy and preserves the input alongside the result.
    pub fn apply(&self, amount: &Money) -> Result<RoundedMoney, DomainError> {
        if amount.currency() != &self.currency {
            return Err(DomainError::CurrencyMismatch {
                left: self.currency.as_str().to_owned(),
                right: amount.currency().as_str().to_owned(),
            });
        }
        let rounded = amount
            .amount()
            .round_dp_with_strategy(self.scale, self.mode.strategy());
        let rounded = if rounded.is_zero() {
            Decimal::ZERO
        } else {
            rounded
        };
        Ok(RoundedMoney {
            unrounded: amount.clone(),
            rounded: Money::new(rounded, self.currency.clone()),
            policy_id: self.id.clone(),
            policy_version: self.version,
            scale: self.scale,
            mode: self.mode,
            boundary: self.boundary,
        })
    }

    /// Returns the policy ID.
    #[must_use]
    pub const fn id(&self) -> &RoundingPolicyId {
        &self.id
    }

    /// Returns the policy version.
    #[must_use]
    pub const fn version(&self) -> RateVersion {
        self.version
    }

    /// Returns the policy currency.
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Returns the target decimal scale.
    #[must_use]
    pub const fn scale(&self) -> u32 {
        self.scale
    }

    /// Returns the rounding mode.
    #[must_use]
    pub const fn mode(&self) -> RoundingMode {
        self.mode
    }

    /// Returns the named boundary.
    #[must_use]
    pub const fn boundary(&self) -> RoundingBoundary {
        self.boundary
    }
}

/// Both sides of one governed rounding operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RoundedMoney {
    /// Exact authoritative input.
    pub unrounded: Money,
    /// Rounded result for the named boundary.
    pub rounded: Money,
    /// Stable policy ID.
    pub policy_id: RoundingPolicyId,
    /// Exact policy version.
    pub policy_version: RateVersion,
    /// Applied decimal scale.
    pub scale: u32,
    /// Applied rounding mode.
    pub mode: RoundingMode,
    /// Applied rounding boundary.
    pub boundary: RoundingBoundary,
}

/// One authoritative pricing method per policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum PricingMethod {
    /// Cost multiplied by one plus this markup rate.
    Markup {
        /// Exact markup rate.
        #[serde(with = "crate::decimal_serde")]
        rate: Decimal,
    },
    /// Cost divided by one minus this target margin.
    TargetMargin {
        /// Exact target margin.
        #[serde(with = "crate::decimal_serde")]
        rate: Decimal,
    },
}

impl PricingMethod {
    /// Validates method-specific rate boundaries.
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::Markup { rate } if rate < &Decimal::NEGATIVE_ONE => {
                Err(DomainError::InvalidValue {
                    field: "markup rate",
                    reason: "must be greater than or equal to -1",
                })
            }
            Self::TargetMargin { rate } if rate >= &Decimal::ONE => {
                Err(DomainError::InvalidValue {
                    field: "target margin",
                    reason: "must be less than 1",
                })
            }
            _ => Ok(()),
        }
    }
}

/// Versioned pricing policy separated from operational rates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PricingPolicy {
    id: PricingPolicyId,
    version: RateVersion,
    currency: CurrencyCode,
    method: PricingMethod,
    floor: Option<Money>,
    minimum_order: Option<Money>,
    quote_total_rounding: RoundingPolicy,
}

#[derive(Deserialize)]
struct PricingPolicyWire {
    id: PricingPolicyId,
    version: RateVersion,
    currency: CurrencyCode,
    method: PricingMethod,
    floor: Option<Money>,
    minimum_order: Option<Money>,
    quote_total_rounding: RoundingPolicy,
}

impl<'de> Deserialize<'de> for PricingPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PricingPolicyWire::deserialize(deserializer)?;
        Self::new(
            wire.id,
            wire.version,
            wire.currency,
            wire.method,
            wire.floor,
            wire.minimum_order,
            wire.quote_total_rounding,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl PricingPolicy {
    /// Validates and creates a pricing policy.
    pub fn new(
        id: PricingPolicyId,
        version: RateVersion,
        currency: CurrencyCode,
        method: PricingMethod,
        floor: Option<Money>,
        minimum_order: Option<Money>,
        quote_total_rounding: RoundingPolicy,
    ) -> Result<Self, DomainError> {
        method.validate()?;
        for amount in [floor.as_ref(), minimum_order.as_ref()]
            .into_iter()
            .flatten()
        {
            if amount.amount().is_sign_negative() {
                return Err(DomainError::InvalidValue {
                    field: "pricing threshold",
                    reason: "must be nonnegative",
                });
            }
            ensure_currency(&currency, amount.currency())?;
        }
        ensure_currency(&currency, quote_total_rounding.currency())?;
        if quote_total_rounding.boundary() != RoundingBoundary::QuoteTotal {
            return Err(DomainError::InvalidValue {
                field: "pricing rounding policy",
                reason: "must use the quote-total boundary",
            });
        }
        Ok(Self {
            id,
            version,
            currency,
            method,
            floor,
            minimum_order,
            quote_total_rounding,
        })
    }

    /// Returns the policy ID.
    #[must_use]
    pub const fn id(&self) -> &PricingPolicyId {
        &self.id
    }

    /// Returns the policy version.
    #[must_use]
    pub const fn version(&self) -> RateVersion {
        self.version
    }

    /// Returns the policy currency.
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Returns the authoritative pricing method.
    #[must_use]
    pub const fn method(&self) -> &PricingMethod {
        &self.method
    }

    /// Returns the optional price floor.
    #[must_use]
    pub const fn floor(&self) -> Option<&Money> {
        self.floor.as_ref()
    }

    /// Returns the optional minimum-order threshold.
    #[must_use]
    pub const fn minimum_order(&self) -> Option<&Money> {
        self.minimum_order.as_ref()
    }

    /// Returns the quote-total rounding policy.
    #[must_use]
    pub const fn quote_total_rounding(&self) -> &RoundingPolicy {
        &self.quote_total_rounding
    }
}

fn ensure_currency(expected: &CurrencyCode, actual: &CurrencyCode) -> Result<(), DomainError> {
    if expected == actual {
        return Ok(());
    }
    Err(DomainError::CurrencyMismatch {
        left: expected.as_str().to_owned(),
        right: actual.as_str().to_owned(),
    })
}
