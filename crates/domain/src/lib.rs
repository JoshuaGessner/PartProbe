//! UI-independent domain primitives for PartProbe.

#[doc(hidden)]
pub mod decimal_serde;
pub mod error;
pub mod money;
pub mod provenance;
pub mod units;
pub mod value_state;

pub use error::DomainError;
pub use money::{CurrencyCode, Money};
pub use provenance::{
    RecordedAt, RuleId, RuleRef, RuleVersion, SchemaVersion, SourceKind, SourceRef,
};
pub use units::{
    DensityKilogramsPerCubicMillimeter, ItemQuantity, MassKilograms, VolumeCubicMillimeters,
};
pub use value_state::ValueState;
