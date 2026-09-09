//! Shared builders for deterministic PartProbe tests.

pub mod geometry_fixtures;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Process-owned temporary directory used by portable filesystem tests.
#[derive(Debug)]
pub struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    /// Creates a unique empty directory beneath the host temporary directory.
    pub fn create(label: &str) -> std::io::Result<Self> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let safe_label: String = label
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '-' {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let path = std::env::temp_dir().join(format!(
            "partprobe-test-{}-{}-{}",
            safe_label,
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        Ok(Self { path })
    }

    /// Returns the owned directory path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Removes the owned tree after callers have dropped every filesystem-backed handle.
    pub fn cleanup(self) -> std::io::Result<()> {
        std::fs::remove_dir_all(self.path)
    }
}
