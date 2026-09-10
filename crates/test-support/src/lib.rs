//! Shared builders for deterministic PartProbe tests.

pub mod geometry_fixtures;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CurrencyCode, DensityKilogramsPerCubicMeter, EffectiveDate,
    LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile, MachineProfileId,
    MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId, Money, ProcessClass,
    RemovalRateCubicMillimetersPerMinute, ResourceSelection, ResourceSelectionId,
    ResourceSelectionState, RuleId, RuleRef, RuleVersion, RuntimeMinutes, RuntimeProfileId,
    ShopResourceCatalog, ShopResourceCatalogId, ShopResourceVersion, SourceKind, SourceRef,
    StockAllowanceMillimeters, StockAllowanceProfile, StockAllowanceProfileId, StockForm,
};
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

/// Creates a deterministic synthetic reviewed catalog for persistence/application tests.
#[must_use]
pub fn resource_catalog_fixture(
    catalog_version: u32,
    material_version: u32,
    density_kg_per_m3: &str,
    selection_version: u32,
    selection_state: ResourceSelectionState,
) -> ShopResourceCatalog {
    let catalog_version = ShopResourceVersion::new(catalog_version).expect("catalog version");
    let material_version = ShopResourceVersion::new(material_version).expect("material version");
    let other_version = ShopResourceVersion::new(1).expect("record version");
    let selection_version = ShopResourceVersion::new(selection_version).expect("selection version");
    let material_id = MaterialDefinitionId::new("test-al-6061-t6").expect("material ID");
    let machine_id = MachineProfileId::new("test-vmc").expect("machine ID");
    let source = |id: &str| {
        SourceRef::new(
            SourceKind::Manual,
            id,
            None,
            Some(partprobe_domain::RecordedAt::new("2026-09-09T12:00:00Z").expect("recorded at")),
        )
        .expect("source")
    };
    let material = MaterialDefinition::new(
        material_id.clone(),
        material_version,
        "Aluminum",
        "6061",
        Some("ASTM B221".to_owned()),
        Some("T6".to_owned()),
        DensityKilogramsPerCubicMeter::new(decimal(density_kg_per_m3)).expect("density"),
        source("test-material-source"),
        LibraryRecordState::Reviewed,
    )
    .expect("material");
    let offer = MaterialOffer::new(
        MaterialOfferId::new("test-material-offer").expect("offer ID"),
        other_version,
        material_id.clone(),
        material_version,
        "Synthetic supplier",
        usd("8.50"),
        EffectiveDate::new("2026-09-09").expect("effective date"),
        source("test-offer-source"),
        LibraryRecordState::Reviewed,
    )
    .expect("offer");
    let stock = StockAllowanceProfile::new(
        StockAllowanceProfileId::new("test-stock-allowance").expect("stock ID"),
        other_version,
        StockForm::Rectangular,
        StockAllowanceMillimeters::new(decimal("3")).expect("x allowance"),
        StockAllowanceMillimeters::new(decimal("3")).expect("y allowance"),
        StockAllowanceMillimeters::new(decimal("2")).expect("z allowance"),
        source("test-stock-source"),
        LibraryRecordState::Reviewed,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        other_version,
        "Synthetic VMC",
        ProcessClass::Milling,
        MachineEnvelopeMillimeters::new(decimal("762")).expect("x envelope"),
        MachineEnvelopeMillimeters::new(decimal("508")).expect("y envelope"),
        MachineEnvelopeMillimeters::new(decimal("508")).expect("z envelope"),
        source("test-machine-source"),
        LibraryRecordState::Reviewed,
    )
    .expect("machine");
    let runtime = CoarseRuntimeProfile::new(
        RuntimeProfileId::new("test-runtime").expect("runtime ID"),
        other_version,
        machine_id.clone(),
        other_version,
        material_id.clone(),
        material_version,
        RemovalRateCubicMillimetersPerMinute::new(decimal("16000")).expect("removal rate"),
        RuntimeMinutes::new(decimal("60")).expect("setup minutes"),
        RuntimeMinutes::new(decimal("45")).expect("programming minutes"),
        RuntimeMinutes::new(decimal("3")).expect("load minutes"),
        RuntimeMinutes::new(decimal("15")).expect("inspection minutes"),
        source("test-runtime-source"),
        LibraryRecordState::Reviewed,
    );
    let selection = ResourceSelection::new(
        ResourceSelectionId::new("test-resource-selection").expect("selection ID"),
        selection_version,
        material_id,
        material_version,
        offer.id().clone(),
        offer.version(),
        stock.id().clone(),
        stock.version(),
        machine_id,
        machine.version(),
        runtime.id().clone(),
        runtime.version(),
        selection_state,
        ActorId::new("test-resource-reviewer").expect("reviewer"),
        partprobe_domain::RecordedAt::new("2026-09-09T14:00:00Z").expect("decision time"),
        "reviewed synthetic resource selection",
    )
    .expect("selection");
    ShopResourceCatalog::new(
        ShopResourceCatalogId::new("test-resource-catalog").expect("catalog ID"),
        catalog_version,
        CurrencyCode::new("USD").expect("currency"),
        vec![material],
        vec![offer],
        vec![stock],
        vec![machine],
        vec![runtime],
        vec![selection],
    )
    .expect("resource catalog")
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
