use std::error::Error;
use std::fmt::{Display, Formatter};

use partprobe_domain::DomainError;

use crate::graph::{NodeId, ValueType};

/// A graph, rule, or snapshot failure.
#[derive(Debug)]
pub enum CalculationError {
    /// A domain primitive rejected a value or operation.
    Domain(DomainError),
    /// Two graph nodes use the same stable identifier.
    DuplicateNode {
        /// The duplicated identifier.
        node: NodeId,
    },
    /// A dependency points to a node that is not in the graph.
    MissingDependency {
        /// The node containing the reference.
        node: NodeId,
        /// The missing dependency.
        dependency: NodeId,
    },
    /// A dependency output cannot satisfy an input's declared type.
    TypeMismatch {
        /// The consuming node.
        node: NodeId,
        /// The dependency node.
        dependency: NodeId,
        /// Type required by the consumer.
        expected: ValueType,
        /// Type produced by the dependency.
        actual: ValueType,
    },
    /// At least one dependency cycle exists.
    CycleDetected {
        /// Nodes that could not be placed in the topological order.
        nodes: Vec<NodeId>,
    },
    /// Snapshot serialization failed.
    Serialization(serde_json::Error),
}

impl Display for CalculationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => Display::fmt(error, formatter),
            Self::DuplicateNode { node } => write!(formatter, "duplicate node: {}", node.as_str()),
            Self::MissingDependency { node, dependency } => write!(
                formatter,
                "node {} references missing dependency {}",
                node.as_str(),
                dependency.as_str()
            ),
            Self::TypeMismatch {
                node,
                dependency,
                expected,
                actual,
            } => write!(
                formatter,
                "node {} expects {:?} from {}, but dependency produces {:?}",
                node.as_str(),
                expected,
                dependency.as_str(),
                actual
            ),
            Self::CycleDetected { nodes } => {
                write!(formatter, "calculation cycle includes")?;
                for node in nodes {
                    write!(formatter, " {}", node.as_str())?;
                }
                Ok(())
            }
            Self::Serialization(error) => {
                write!(formatter, "snapshot serialization failed: {error}")
            }
        }
    }
}

impl Error for CalculationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DomainError> for CalculationError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

impl From<serde_json::Error> for CalculationError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}
