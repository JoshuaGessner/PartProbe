use std::collections::{BTreeMap, BTreeSet};

use partprobe_domain::{CurrencyCode, DomainError, RuleRef, ValueState};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{CalculationError, SnapshotValue};

/// Stable identifier for a calculation node.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct NodeId(String);

impl NodeId {
    /// Validates and creates a node identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "node ID",
                reason: "must not be empty",
            });
        }
        Ok(Self(value))
    }

    /// Returns the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Physical or commercial dimension of a graph value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// Dimensionless decimal value.
    Unitless,
    /// Whole item count.
    Quantity,
    /// Cubic-millimeter volume.
    Volume,
    /// Kilograms per cubic millimeter.
    Density,
    /// Kilogram mass.
    Mass,
    /// Fixed-precision money.
    Money,
}

/// Runtime graph type, including currency when the dimension is money.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValueType {
    dimension: Dimension,
    currency: Option<CurrencyCode>,
}

impl ValueType {
    /// Creates a non-money value type.
    pub fn physical(dimension: Dimension) -> Result<Self, DomainError> {
        if dimension == Dimension::Money {
            return Err(DomainError::InvalidValue {
                field: "value type",
                reason: "money requires an explicit currency",
            });
        }
        Ok(Self {
            dimension,
            currency: None,
        })
    }

    /// Creates a money value type.
    #[must_use]
    pub const fn money(currency: CurrencyCode) -> Self {
        Self {
            dimension: Dimension::Money,
            currency: Some(currency),
        }
    }

    /// Returns the dimension.
    #[must_use]
    pub const fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// Returns the currency for a money value.
    #[must_use]
    pub const fn currency(&self) -> Option<&CurrencyCode> {
        self.currency.as_ref()
    }
}

#[derive(Deserialize)]
struct ValueTypeWire {
    dimension: Dimension,
    currency: Option<CurrencyCode>,
}

impl<'de> Deserialize<'de> for ValueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ValueTypeWire::deserialize(deserializer)?;
        match (wire.dimension, wire.currency) {
            (Dimension::Money, Some(currency)) => Ok(Self::money(currency)),
            (Dimension::Money, None) => Err(serde::de::Error::custom(
                "money value type requires an explicit currency",
            )),
            (dimension, None) => Self::physical(dimension).map_err(serde::de::Error::custom),
            (_, Some(_)) => Err(serde::de::Error::custom(
                "physical value type must not carry a currency",
            )),
        }
    }
}

/// A typed edge from a dependency node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InputDefinition {
    source: NodeId,
    expected_type: ValueType,
}

impl InputDefinition {
    /// Creates a typed input edge.
    #[must_use]
    pub const fn new(source: NodeId, expected_type: ValueType) -> Self {
        Self {
            source,
            expected_type,
        }
    }

    /// Returns the dependency node.
    #[must_use]
    pub const fn source(&self) -> &NodeId {
        &self.source
    }

    /// Returns the expected dependency output type.
    #[must_use]
    pub const fn expected_type(&self) -> &ValueType {
        &self.expected_type
    }
}

/// Serializable definition of a graph node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeDefinition {
    id: NodeId,
    rule: RuleRef,
    inputs: Vec<InputDefinition>,
    output_type: ValueType,
}

impl NodeDefinition {
    /// Creates a node definition.
    #[must_use]
    pub const fn new(
        id: NodeId,
        rule: RuleRef,
        inputs: Vec<InputDefinition>,
        output_type: ValueType,
    ) -> Self {
        Self {
            id,
            rule,
            inputs,
            output_type,
        }
    }

    /// Returns the node identifier.
    #[must_use]
    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    /// Returns the rule reference.
    #[must_use]
    pub const fn rule(&self) -> &RuleRef {
        &self.rule
    }

    /// Returns the typed dependency edges.
    #[must_use]
    pub fn inputs(&self) -> &[InputDefinition] {
        &self.inputs
    }

    /// Returns the node output type.
    #[must_use]
    pub const fn output_type(&self) -> &ValueType {
        &self.output_type
    }
}

/// Evaluated output and trace from a calculation node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeEvaluation {
    /// Explicit result state.
    pub result: ValueState<SnapshotValue>,
    /// Named intermediate values in deterministic key order.
    pub intermediate_trace: BTreeMap<String, SnapshotValue>,
    /// Stable warning text for review and replay.
    pub warnings: Vec<String>,
}

/// Interface implemented by an executable calculation node.
pub trait CalculationNode {
    /// Returns the immutable node definition.
    fn definition(&self) -> &NodeDefinition;

    /// Evaluates from already-resolved dependency values.
    fn evaluate(
        &self,
        inputs: &BTreeMap<NodeId, ValueState<SnapshotValue>>,
    ) -> Result<NodeEvaluation, CalculationError>;
}

/// Validated directed acyclic graph with deterministic evaluation order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalculationGraph {
    nodes: BTreeMap<NodeId, NodeDefinition>,
    evaluation_order: Vec<NodeId>,
}

impl CalculationGraph {
    /// Validates node uniqueness, references, types, and acyclicity.
    pub fn try_new(definitions: Vec<NodeDefinition>) -> Result<Self, CalculationError> {
        let mut nodes = BTreeMap::new();
        for definition in definitions {
            let node_id = definition.id.clone();
            if nodes.insert(node_id.clone(), definition).is_some() {
                return Err(CalculationError::DuplicateNode { node: node_id });
            }
        }

        Self::validate_edges(&nodes)?;
        let evaluation_order = Self::topological_order(&nodes)?;

        Ok(Self {
            nodes,
            evaluation_order,
        })
    }

    /// Returns a node definition by ID.
    #[must_use]
    pub fn node(&self, node_id: &NodeId) -> Option<&NodeDefinition> {
        self.nodes.get(node_id)
    }

    /// Returns the deterministic topological order.
    #[must_use]
    pub fn evaluation_order(&self) -> &[NodeId] {
        &self.evaluation_order
    }

    /// Returns all definitions in stable ID order.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeMap<NodeId, NodeDefinition> {
        &self.nodes
    }

    fn validate_edges(nodes: &BTreeMap<NodeId, NodeDefinition>) -> Result<(), CalculationError> {
        for (node_id, definition) in nodes {
            for input in &definition.inputs {
                let dependency = nodes.get(&input.source).ok_or_else(|| {
                    CalculationError::MissingDependency {
                        node: node_id.clone(),
                        dependency: input.source.clone(),
                    }
                })?;
                if dependency.output_type != input.expected_type {
                    return Err(CalculationError::TypeMismatch {
                        node: node_id.clone(),
                        dependency: input.source.clone(),
                        expected: input.expected_type.clone(),
                        actual: dependency.output_type.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn topological_order(
        nodes: &BTreeMap<NodeId, NodeDefinition>,
    ) -> Result<Vec<NodeId>, CalculationError> {
        let mut indegree = BTreeMap::new();
        let mut dependents: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();

        for (node_id, definition) in nodes {
            indegree.insert(node_id.clone(), definition.inputs.len());
            for input in &definition.inputs {
                dependents
                    .entry(input.source.clone())
                    .or_default()
                    .push(node_id.clone());
            }
        }

        let mut ready: BTreeSet<NodeId> = indegree
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(node_id, _)| node_id.clone())
            .collect();
        let mut order = Vec::with_capacity(nodes.len());

        while let Some(node_id) = ready.pop_first() {
            order.push(node_id.clone());
            if let Some(children) = dependents.get(&node_id) {
                for child in children {
                    let count = indegree
                        .get_mut(child)
                        .expect("validated dependency must have indegree");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(child.clone());
                    }
                }
            }
        }

        if order.len() == nodes.len() {
            return Ok(order);
        }

        let nodes = indegree
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(node_id, _)| node_id)
            .collect();
        Err(CalculationError::CycleDetected { nodes })
    }
}
