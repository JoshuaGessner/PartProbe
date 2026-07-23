use std::error::Error;
use std::fmt::{Display, Formatter};

/// A validation or arithmetic failure at the domain boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    /// A value violated a named invariant.
    InvalidValue {
        /// The field or type that failed validation.
        field: &'static str,
        /// A stable explanation suitable for a calculation trace.
        reason: &'static str,
    },
    /// Arithmetic exceeded the bounded representation.
    ArithmeticOverflow {
        /// The operation that overflowed.
        operation: &'static str,
    },
    /// An exact result cannot be represented without an approved rounding policy.
    RoundingRequired {
        /// The operation that requires a rounding decision.
        operation: &'static str,
    },
    /// Money operands used different currencies.
    CurrencyMismatch {
        /// Currency on the left operand.
        left: String,
        /// Currency on the right operand.
        right: String,
    },
}

impl Display for DomainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue { field, reason } => {
                write!(formatter, "invalid {field}: {reason}")
            }
            Self::ArithmeticOverflow { operation } => {
                write!(formatter, "arithmetic overflow during {operation}")
            }
            Self::RoundingRequired { operation } => {
                write!(
                    formatter,
                    "explicit rounding policy required for {operation}"
                )
            }
            Self::CurrencyMismatch { left, right } => {
                write!(formatter, "currency mismatch: {left} versus {right}")
            }
        }
    }
}

impl Error for DomainError {}
