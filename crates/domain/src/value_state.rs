use serde::{Deserialize, Serialize};

/// Explicit availability state for an input or result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ValueState<T> {
    /// A valid value is available.
    Available {
        /// The available value.
        value: T,
    },
    /// Evidence is currently absent.
    Unavailable {
        /// Why the value cannot be supplied.
        reason: String,
    },
    /// A prerequisite prevents evaluation.
    Blocked {
        /// Why evaluation is blocked.
        reason: String,
    },
    /// The state cannot yet be determined.
    Unknown {
        /// Why the state is unknown.
        reason: String,
    },
    /// A last-known value exists but violates freshness policy.
    Stale {
        /// The preserved last-known value.
        last_known: T,
        /// Source timestamp or version text.
        as_of: String,
        /// Why the evidence is stale.
        reason: String,
    },
}

impl<T> ValueState<T> {
    /// Wraps an available value.
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self::Available { value }
    }
}
