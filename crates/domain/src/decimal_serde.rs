//! Canonical decimal-as-string Serde support.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serializer};

/// Normalizes a decimal for deterministic persisted representation.
#[must_use]
pub fn canonical(value: Decimal) -> Decimal {
    if value.is_zero() {
        Decimal::ZERO
    } else {
        value.normalize()
    }
}

/// Serializes a decimal as a normalized string.
pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&canonical(*value).to_string())
}

/// Deserializes a decimal string without applying a domain-specific sign/range policy.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}
