use std::collections::BTreeMap;

use partprobe_domain::{
    DensityKilogramsPerCubicMillimeter, ItemQuantity, MassKilograms, Money, RoundedMoney, RuleRef,
    RuleVersion, SchemaVersion, SourceRef, ValueState, VolumeCubicMillimeters,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{CalculationError, NodeId, ResolvedRate};

/// Typed value preserved in a calculation trace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SnapshotValue {
    /// Dimensionless exact decimal.
    Decimal(#[serde(with = "partprobe_domain::decimal_serde")] Decimal),
    /// Whole item count.
    Quantity(ItemQuantity),
    /// Cubic-millimeter volume.
    Volume(VolumeCubicMillimeters),
    /// Kilograms-per-cubic-millimeter density.
    Density(DensityKilogramsPerCubicMillimeter),
    /// Kilogram mass.
    Mass(MassKilograms),
    /// Fixed-precision currency.
    Money(Money),
    /// Pinned resolved rate and its card/entry versions.
    ResolvedRate(Box<ResolvedRate>),
    /// Both sides and policy metadata of a governed rounding boundary.
    RoundedMoney(RoundedMoney),
    /// Versioned textual evidence.
    Text(String),
}

/// Immutable result record for one graph node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSnapshot {
    /// Exact rule and behavior version.
    pub rule: RuleRef,
    /// Inputs in stable key order.
    pub inputs: BTreeMap<String, SnapshotValue>,
    /// Explicit result state.
    pub result: ValueState<SnapshotValue>,
    /// Named intermediate values in deterministic key order.
    pub intermediate_trace: BTreeMap<String, SnapshotValue>,
    /// Source provenance for the evaluated node.
    pub provenance: SourceRef,
    /// Stable review warnings.
    pub warnings: Vec<String>,
}

/// Versioned calculation result serialized for replay.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalculationSnapshot {
    /// Serialization contract version.
    pub schema_version: SchemaVersion,
    /// Version of the graph structure.
    pub graph_version: RuleVersion,
    /// Node records in stable ID order.
    pub nodes: BTreeMap<NodeId, NodeSnapshot>,
}

impl CalculationSnapshot {
    /// Serializes the snapshot to compact canonical JSON.
    ///
    /// Determinism relies on struct field order, `BTreeMap` key order, normalized
    /// decimal-as-string serialization, and the pinned serializer version in `Cargo.lock`.
    pub fn to_canonical_json(&self) -> Result<String, CalculationError> {
        Ok(serde_json::to_string(self)?)
    }
}
