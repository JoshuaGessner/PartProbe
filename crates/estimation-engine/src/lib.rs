//! Deterministic, UI-independent calculation foundations.

pub mod error;
pub mod graph;
pub mod rate_resolution;
pub mod rules;
pub mod snapshot;

pub use error::CalculationError;
pub use graph::{
    CalculationGraph, CalculationNode, Dimension, InputDefinition, NodeDefinition, NodeEvaluation,
    NodeId, ValueType,
};
pub use rate_resolution::{ResolvedRate, extend_rate, resolve_rate};
pub use rules::{
    BaseInternalCostComponents, GeometryBasis, MaterialCostComponents, OperationCostComponents,
    PricingOutcome, RuleOutcome, apply_pricing_policy, base_internal_cost, cycle_time,
    make_quantity, material_cost, operation_cost, part_mass, price_from_margin, price_from_markup,
    removed_volume, risk_reserve, run_cost, setup_lot_cost, setup_unit_cost, total_internal_cost,
};
pub use snapshot::{CalculationSnapshot, NodeSnapshot, SnapshotValue};
