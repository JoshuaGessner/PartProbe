use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::DomainError;

macro_rules! non_negative_decimal_quantity {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(#[serde(with = "crate::decimal_serde")] Decimal);

        impl $name {
            /// Validates and creates the quantity.
            pub fn new(value: Decimal) -> Result<Self, DomainError> {
                if value.is_sign_negative() && !value.is_zero() {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must be non-negative",
                    });
                }
                Ok(Self(value))
            }

            /// Creates an exact zero quantity.
            #[must_use]
            pub const fn zero() -> Self {
                Self(Decimal::ZERO)
            }

            /// Returns the exact decimal magnitude in the type's declared unit.
            #[must_use]
            pub const fn value(self) -> Decimal {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = crate::decimal_serde::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

non_negative_decimal_quantity!(
    /// Volume expressed in cubic millimeters.
    ///
    /// ```compile_fail
    /// use partprobe_domain::{MassKilograms, VolumeCubicMillimeters};
    /// let volume = VolumeCubicMillimeters::zero();
    /// let _mass: MassKilograms = volume;
    /// ```
    VolumeCubicMillimeters,
    "volume"
);

non_negative_decimal_quantity!(
    /// Density expressed in kilograms per cubic millimeter.
    DensityKilogramsPerCubicMillimeter,
    "density"
);

non_negative_decimal_quantity!(
    /// Mass expressed in kilograms.
    MassKilograms,
    "mass"
);

/// A whole-number item quantity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemQuantity(u64);

impl ItemQuantity {
    /// Creates an item quantity.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the whole-number quantity.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Adds two quantities with overflow detection.
    pub fn checked_add(self, other: Self) -> Result<Self, DomainError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(DomainError::ArithmeticOverflow {
                operation: "item quantity addition",
            })
    }
}
