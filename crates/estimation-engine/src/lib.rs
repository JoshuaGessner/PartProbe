//! Deterministic, UI-independent calculation foundations.

pub mod error;
pub mod graph;
pub mod rules;
pub mod snapshot;

pub use error::CalculationError;
pub use graph::{
    CalculationGraph, CalculationNode, Dimension, InputDefinition, NodeDefinition, NodeEvaluation,
    NodeId, ValueType,
};
pub use rules::{
    GeometryBasis, RuleOutcome, make_quantity, part_mass, price_from_margin, price_from_markup,
    removed_volume,
};
pub use snapshot::{CalculationSnapshot, NodeSnapshot, SnapshotValue};
