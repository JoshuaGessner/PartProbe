use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::DomainError;

/// An uppercase ISO-style three-letter currency identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Validates and creates a currency code.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.len() != 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase()) {
            return Err(DomainError::InvalidValue {
                field: "currency",
                reason: "must contain exactly three uppercase ASCII letters",
            });
        }
        Ok(Self(value))
    }

    /// Returns the validated code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for CurrencyCode {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Fixed-precision money with an explicit currency.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Money {
    #[serde(with = "crate::decimal_serde")]
    amount: Decimal,
    currency: CurrencyCode,
}

impl Money {
    /// Creates a monetary value. Negative values are allowed for explicit deltas and credits.
    #[must_use]
    pub const fn new(amount: Decimal, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    /// Returns the exact decimal amount.
    #[must_use]
    pub const fn amount(&self) -> Decimal {
        self.amount
    }

    /// Returns the currency.
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Adds same-currency values without implicit conversion.
    pub fn checked_add(&self, other: &Self) -> Result<Self, DomainError> {
        self.ensure_same_currency(other)?;
        let amount = checked_decimal_add_exact(self.amount, other.amount, "money addition")?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Subtracts same-currency values without implicit conversion.
    pub fn checked_sub(&self, other: &Self) -> Result<Self, DomainError> {
        self.ensure_same_currency(other)?;
        let amount = checked_decimal_sub_exact(self.amount, other.amount, "money subtraction")?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Multiplies by a factor only when the exact result is representable.
    pub fn checked_mul(&self, factor: Decimal) -> Result<Self, DomainError> {
        let amount = checked_decimal_mul_exact(self.amount, factor, "money multiplication")?;
        Ok(Self::new(amount, self.currency.clone()))
    }

    /// Divides by a nonzero factor only when the exact result is representable.
    ///
    /// A nonterminating or precision-losing result is rejected until the caller supplies
    /// an approved, versioned rounding policy.
    pub fn checked_div_exact(&self, divisor: Decimal) -> Result<Self, DomainError> {
        if divisor.is_zero() {
            return Err(DomainError::InvalidValue {
                field: "money divisor",
                reason: "must not be zero",
            });
        }
        if !has_terminating_decimal(self.amount, divisor) {
            return Err(DomainError::RoundingRequired {
                operation: "money division",
            });
        }
        let amount = self
            .amount
            .checked_div(divisor)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "money division",
            })?;
        let reconstructed =
            exact_product_representation(amount, divisor).ok_or(DomainError::RoundingRequired {
                operation: "money division",
            })?;
        let expected = crate::decimal_serde::canonical(self.amount);
        if reconstructed.0 != expected.mantissa() || reconstructed.1 != expected.scale() {
            return Err(DomainError::RoundingRequired {
                operation: "money division",
            });
        }
        Ok(Self::new(amount, self.currency.clone()))
    }

    fn ensure_same_currency(&self, other: &Self) -> Result<(), DomainError> {
        if self.currency == other.currency {
            return Ok(());
        }
        Err(DomainError::CurrencyMismatch {
            left: self.currency.as_str().to_owned(),
            right: other.currency.as_str().to_owned(),
        })
    }
}

/// Adds decimals without permitting implicit precision loss.
#[doc(hidden)]
pub fn checked_decimal_add_exact(
    left: Decimal,
    right: Decimal,
    operation: &'static str,
) -> Result<Decimal, DomainError> {
    let actual = left
        .checked_add(right)
        .ok_or(DomainError::ArithmeticOverflow { operation })?;
    ensure_exact_sum(left, right, false, actual, operation)?;
    Ok(actual)
}

/// Subtracts decimals without permitting implicit precision loss.
#[doc(hidden)]
pub fn checked_decimal_sub_exact(
    left: Decimal,
    right: Decimal,
    operation: &'static str,
) -> Result<Decimal, DomainError> {
    let actual = left
        .checked_sub(right)
        .ok_or(DomainError::ArithmeticOverflow { operation })?;
    ensure_exact_sum(left, right, true, actual, operation)?;
    Ok(actual)
}

/// Multiplies decimals without permitting implicit precision loss.
#[doc(hidden)]
pub fn checked_decimal_mul_exact(
    left: Decimal,
    right: Decimal,
    operation: &'static str,
) -> Result<Decimal, DomainError> {
    let expected = exact_product_representation(left, right)
        .ok_or(DomainError::RoundingRequired { operation })?;
    let actual = left
        .checked_mul(right)
        .ok_or(DomainError::ArithmeticOverflow { operation })?;
    let canonical = crate::decimal_serde::canonical(actual);
    if canonical.mantissa() == expected.0 && canonical.scale() == expected.1 {
        Ok(actual)
    } else {
        Err(DomainError::RoundingRequired { operation })
    }
}

fn has_terminating_decimal(dividend: Decimal, divisor: Decimal) -> bool {
    let mut denominator = divisor.mantissa().unsigned_abs();
    let numerator = dividend.mantissa().unsigned_abs();
    denominator /= greatest_common_divisor(numerator, denominator);

    while denominator.is_multiple_of(2) {
        denominator /= 2;
    }
    while denominator.is_multiple_of(5) {
        denominator /= 5;
    }
    denominator == 1
}

fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn exact_product_representation(left: Decimal, right: Decimal) -> Option<(i128, u32)> {
    let left = crate::decimal_serde::canonical(left);
    let right = crate::decimal_serde::canonical(right);
    if left.is_zero() || right.is_zero() {
        return Some((0, 0));
    }

    let mut left_mantissa = left.mantissa().unsigned_abs();
    let mut right_mantissa = right.mantissa().unsigned_abs();
    let mut scale = left.scale().checked_add(right.scale())?;
    let paired_factors = count_factor(left_mantissa, 2)
        .checked_add(count_factor(right_mantissa, 2))?
        .min(count_factor(left_mantissa, 5).checked_add(count_factor(right_mantissa, 5))?)
        .min(scale);
    remove_factors(&mut left_mantissa, &mut right_mantissa, 2, paired_factors);
    remove_factors(&mut left_mantissa, &mut right_mantissa, 5, paired_factors);
    scale -= paired_factors;

    let product = left_mantissa.checked_mul(right_mantissa)?;
    if scale > Decimal::MAX_SCALE || product > Decimal::MAX.mantissa().unsigned_abs() {
        return None;
    }

    let mantissa = i128::try_from(product).ok()?;
    let signed_mantissa = if left.is_sign_negative() ^ right.is_sign_negative() {
        -mantissa
    } else {
        mantissa
    };
    Some((signed_mantissa, scale))
}

fn count_factor(mut value: u128, factor: u128) -> u32 {
    let mut count = 0;
    while value.is_multiple_of(factor) {
        value /= factor;
        count += 1;
    }
    count
}

fn remove_factors(left: &mut u128, right: &mut u128, factor: u128, mut count: u32) {
    while count > 0 && left.is_multiple_of(factor) {
        *left /= factor;
        count -= 1;
    }
    while count > 0 {
        debug_assert!(right.is_multiple_of(factor));
        *right /= factor;
        count -= 1;
    }
}

fn ensure_exact_sum(
    left: Decimal,
    right: Decimal,
    subtract_right: bool,
    actual: Decimal,
    operation: &'static str,
) -> Result<(), DomainError> {
    let expected = exact_sum_representation(left, right, subtract_right)
        .ok_or(DomainError::RoundingRequired { operation })?;
    let actual = crate::decimal_serde::canonical(actual);
    if actual.mantissa() == expected.0 && actual.scale() == expected.1 {
        Ok(())
    } else {
        Err(DomainError::RoundingRequired { operation })
    }
}

fn exact_sum_representation(
    left: Decimal,
    right: Decimal,
    subtract_right: bool,
) -> Option<(i128, u32)> {
    let left = crate::decimal_serde::canonical(left);
    let right = crate::decimal_serde::canonical(right);
    let mut scale = left.scale().max(right.scale());
    let left_magnitude = align_mantissa(left, scale)?;
    let right_magnitude = align_mantissa(right, scale)?;
    let left_negative = left.is_sign_negative();
    let right_negative = right.is_sign_negative() ^ subtract_right;

    let (mut magnitude, negative) = if left_negative == right_negative {
        (left_magnitude.checked_add(right_magnitude)?, left_negative)
    } else if left_magnitude >= right_magnitude {
        (left_magnitude - right_magnitude, left_negative)
    } else {
        (right_magnitude - left_magnitude, right_negative)
    };

    if magnitude == 0 {
        return Some((0, 0));
    }
    while scale > 0 && magnitude.is_multiple_of(10) {
        magnitude /= 10;
        scale -= 1;
    }
    if magnitude > Decimal::MAX.mantissa().unsigned_abs() {
        return None;
    }

    let mantissa = i128::try_from(magnitude).ok()?;
    Some((if negative { -mantissa } else { mantissa }, scale))
}

fn align_mantissa(value: Decimal, target_scale: u32) -> Option<u128> {
    let scale_difference = target_scale.checked_sub(value.scale())?;
    let multiplier = 10_u128.checked_pow(scale_difference)?;
    value.mantissa().unsigned_abs().checked_mul(multiplier)
}
