//! Shared builders for deterministic PartProbe tests.

use std::str::FromStr;

use partprobe_domain::{CurrencyCode, Money, RuleId, RuleRef, RuleVersion, SourceKind, SourceRef};
use partprobe_estimation_engine::{NodeDefinition, NodeId, ValueType};
use rust_decimal::Decimal;

/// Parses an exact decimal and panics with a fixture-oriented message on invalid text.
#[must_use]
pub fn decimal(value: &str) -> Decimal {
    Decimal::from_str(value).expect("test fixture must contain a valid decimal")
}

/// Creates exact USD money for a test fixture.
#[must_use]
pub fn usd(value: &str) -> Money {
    Money::new(
        decimal(value),
        CurrencyCode::new("USD").expect("USD is a valid currency code"),
    )
}

/// Creates a version 1.0.0 rule reference for a stable rule ID.
#[must_use]
pub fn rule(rule_id: &str) -> RuleRef {
    RuleRef::new(
        RuleId::new(rule_id).expect("test rule ID must be nonempty"),
        RuleVersion::new(1, 0, 0),
    )
}

/// Creates a source reference for deterministic calculated test evidence.
#[must_use]
pub fn calculated_source() -> SourceRef {
    SourceRef::new(
        SourceKind::Calculated,
        "test-engine",
        Some("1".to_owned()),
        None,
    )
    .expect("test source ID must be nonempty")
}

/// Creates a source node with no dependencies.
#[must_use]
pub fn source_node(node_id: &str, rule_id: &str, output_type: ValueType) -> NodeDefinition {
    NodeDefinition::new(
        NodeId::new(node_id).expect("test node ID must be nonempty"),
        rule(rule_id),
        Vec::new(),
        output_type,
    )
}
