use std::{collections::BTreeMap, str::FromStr};

use partprobe_domain::{
    CurrencyCode, DensityKilogramsPerCubicMillimeter, ItemQuantity, MassKilograms, RuleId, RuleRef,
    RuleVersion, SchemaVersion, SourceKind, SourceRef, ValueState, VolumeCubicMillimeters,
};
use partprobe_estimation_engine::{
    CalculationError, CalculationGraph, CalculationSnapshot, Dimension, GeometryBasis,
    InputDefinition, NodeDefinition, NodeId, NodeSnapshot, SnapshotValue, ValueType, make_quantity,
    part_mass, price_from_margin, price_from_markup, removed_volume,
};
use rust_decimal::Decimal;

fn decimal(mantissa: i64, scale: u32) -> Decimal {
    Decimal::new(mantissa, scale)
}

fn rule(id: &str) -> RuleRef {
    RuleRef::new(
        RuleId::new(id).expect("valid rule ID"),
        RuleVersion::new(1, 0, 0),
    )
}

fn node(
    id: &str,
    rule_id: &str,
    inputs: Vec<InputDefinition>,
    output_type: ValueType,
) -> NodeDefinition {
    NodeDefinition::new(
        NodeId::new(id).expect("valid node ID"),
        rule(rule_id),
        inputs,
        output_type,
    )
}

#[test]
fn calc_001_part_mass_is_exact_after_explicit_units() {
    let volume = VolumeCubicMillimeters::new(decimal(1_000, 0)).expect("valid volume");
    let density = DensityKilogramsPerCubicMillimeter::new(decimal(27, 7)).expect("valid density");

    let mass = part_mass(volume, density).expect("mass calculates");

    assert_eq!(
        mass,
        MassKilograms::new(decimal(27_000, 7)).expect("valid mass")
    );
}

#[test]
fn calc_001_rejects_implicit_precision_loss() {
    let volume = VolumeCubicMillimeters::new(Decimal::new(1, 28)).expect("valid tiny volume");
    let density = DensityKilogramsPerCubicMillimeter::new(decimal(1, 1)).expect("valid density");

    assert!(matches!(
        part_mass(volume, density),
        Err(CalculationError::Domain(
            partprobe_domain::DomainError::RoundingRequired { .. }
        ))
    ));
}

#[test]
fn calc_003_removed_volume_obeys_bounds_for_a_grid_of_values() {
    let basis = GeometryBasis {
        enclosed: true,
        multi_body_resolved: true,
    };

    for stock in 0_u64..=20 {
        for part in 0_u64..=20 {
            let outcome = removed_volume(
                VolumeCubicMillimeters::new(Decimal::from(stock)).expect("valid stock"),
                VolumeCubicMillimeters::new(Decimal::from(part)).expect("valid part"),
                basis,
            )
            .expect("removed volume calculates");
            let expected = Decimal::from(stock.saturating_sub(part));

            assert_eq!(outcome.value.value(), expected);
            assert!(outcome.value.value() >= Decimal::ZERO);
            assert!(outcome.value.value() <= Decimal::from(stock));
        }
    }
}

#[test]
fn calc_003_rejects_implicit_precision_loss() {
    let stock = VolumeCubicMillimeters::new(
        Decimal::from_str("108053.27500000000000000000000").expect("valid stock"),
    )
    .expect("valid stock");
    let part = VolumeCubicMillimeters::new(
        Decimal::from_str("0.000000000000000000000001").expect("valid part"),
    )
    .expect("valid part");

    assert!(matches!(
        removed_volume(
            stock,
            part,
            GeometryBasis {
                enclosed: true,
                multi_body_resolved: true,
            },
        ),
        Err(CalculationError::Domain(
            partprobe_domain::DomainError::RoundingRequired { .. }
        ))
    ));
}

#[test]
fn calc_003_preserves_geometry_warnings() {
    let outcome = removed_volume(
        VolumeCubicMillimeters::new(decimal(100, 0)).expect("valid stock"),
        VolumeCubicMillimeters::new(decimal(40, 0)).expect("valid part"),
        GeometryBasis {
            enclosed: false,
            multi_body_resolved: false,
        },
    )
    .expect("removed volume calculates");

    assert_eq!(outcome.warnings.len(), 2);
}

#[test]
fn calc_005_make_quantity_is_never_below_deliver_quantity() {
    for deliver in 0_u64..=20 {
        for spares in 0_u64..=5 {
            for samples in 0_u64..=5 {
                let result = make_quantity(
                    ItemQuantity::new(deliver),
                    ItemQuantity::new(spares),
                    ItemQuantity::new(samples),
                )
                .expect("quantity calculates");
                assert!(result.value() >= deliver);
                assert_eq!(result.value(), deliver + spares + samples);
            }
        }
    }
}

#[test]
fn calc_005_detects_quantity_overflow() {
    let error = make_quantity(
        ItemQuantity::new(u64::MAX),
        ItemQuantity::new(1),
        ItemQuantity::new(0),
    )
    .expect_err("overflow must fail");

    assert!(matches!(error, CalculationError::Domain(_)));
}

#[test]
fn calc_016_and_017_match_exact_golden_prices() {
    let cost = partprobe_domain::Money::new(
        decimal(10_000, 2),
        CurrencyCode::new("USD").expect("valid currency"),
    );

    let markup = price_from_markup(&cost, decimal(25, 2)).expect("valid markup");
    let margin = price_from_margin(&cost, decimal(20, 2)).expect("valid margin");

    assert_eq!(markup.amount(), decimal(12_500, 2));
    assert_eq!(margin.amount(), decimal(12_500, 2));
}

#[test]
fn pricing_rejects_invalid_rate_boundaries() {
    let cost = partprobe_domain::Money::new(
        decimal(100, 0),
        CurrencyCode::new("USD").expect("valid currency"),
    );

    assert!(price_from_markup(&cost, decimal(-101, 2)).is_err());
    assert!(price_from_margin(&cost, Decimal::ONE).is_err());
    assert!(matches!(
        price_from_margin(&cost, decimal(10, 2)),
        Err(CalculationError::Domain(
            partprobe_domain::DomainError::RoundingRequired { .. }
        ))
    ));

    let tiny_cost = partprobe_domain::Money::new(
        Decimal::new(1, 28),
        CurrencyCode::new("USD").expect("valid currency"),
    );
    assert!(matches!(
        price_from_markup(&tiny_cost, decimal(-9, 1)),
        Err(CalculationError::Domain(
            partprobe_domain::DomainError::RoundingRequired { .. }
        ))
    ));
}

#[test]
fn graph_rejects_runtime_dimension_mismatch() {
    let volume_type = ValueType::physical(Dimension::Volume).expect("valid type");
    let mass_type = ValueType::physical(Dimension::Mass).expect("valid type");
    let source_id = NodeId::new("volume").expect("valid ID");
    let source = node("volume", "CALC-INPUT", Vec::new(), volume_type);
    let consumer = node(
        "price",
        "CALC-001",
        vec![InputDefinition::new(source_id, mass_type.clone())],
        mass_type,
    );

    let error = CalculationGraph::try_new(vec![source, consumer]).expect_err("type must fail");

    assert!(matches!(error, CalculationError::TypeMismatch { .. }));
}

#[test]
fn graph_rejects_missing_dependencies() {
    let volume_type = ValueType::physical(Dimension::Volume).expect("valid type");
    let consumer = node(
        "removed",
        "CALC-003",
        vec![InputDefinition::new(
            NodeId::new("missing").expect("valid ID"),
            volume_type.clone(),
        )],
        volume_type,
    );

    let error = CalculationGraph::try_new(vec![consumer]).expect_err("missing edge must fail");

    assert!(matches!(error, CalculationError::MissingDependency { .. }));
}

#[test]
fn graph_rejects_cycles() {
    let quantity_type = ValueType::physical(Dimension::Quantity).expect("valid type");
    let first = node(
        "a",
        "CALC-A",
        vec![InputDefinition::new(
            NodeId::new("b").expect("valid ID"),
            quantity_type.clone(),
        )],
        quantity_type.clone(),
    );
    let second = node(
        "b",
        "CALC-B",
        vec![InputDefinition::new(
            NodeId::new("a").expect("valid ID"),
            quantity_type.clone(),
        )],
        quantity_type,
    );

    let error = CalculationGraph::try_new(vec![first, second]).expect_err("cycle must fail");

    assert!(matches!(error, CalculationError::CycleDetected { .. }));
}

#[test]
fn graph_order_is_deterministic_for_independent_nodes() {
    let quantity_type = ValueType::physical(Dimension::Quantity).expect("valid type");
    let graph = CalculationGraph::try_new(vec![
        node("z", "CALC-Z", Vec::new(), quantity_type.clone()),
        node("a", "CALC-A", Vec::new(), quantity_type),
    ])
    .expect("valid graph");

    let order: Vec<&str> = graph
        .evaluation_order()
        .iter()
        .map(NodeId::as_str)
        .collect();
    assert_eq!(order, vec!["a", "z"]);
}

#[test]
fn graph_deserialization_cannot_create_invalid_identifiers_or_value_types() {
    assert!(serde_json::from_str::<NodeId>(r#""""#).is_err());
    assert!(serde_json::from_str::<ValueType>(r#"{"dimension":"money","currency":null}"#).is_err());
    assert!(
        serde_json::from_str::<ValueType>(r#"{"dimension":"volume","currency":"USD"}"#).is_err()
    );
}

#[test]
fn snapshot_serialization_is_canonical_and_exact() {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "cost".to_owned(),
        SnapshotValue::Money(partprobe_domain::Money::new(
            decimal(10_000, 2),
            CurrencyCode::new("USD").expect("valid currency"),
        )),
    );
    let provenance = SourceRef::new(SourceKind::Calculated, "engine", Some("1".to_owned()), None)
        .expect("valid source");
    let mut intermediate_trace = BTreeMap::new();
    intermediate_trace.insert(
        "markup_factor".to_owned(),
        SnapshotValue::Decimal(decimal(125, 2)),
    );
    let node_snapshot = NodeSnapshot {
        rule: rule("CALC-016"),
        inputs,
        result: ValueState::available(SnapshotValue::Money(partprobe_domain::Money::new(
            decimal(12_500, 2),
            CurrencyCode::new("USD").expect("valid currency"),
        ))),
        intermediate_trace,
        provenance,
        warnings: Vec::new(),
    };
    let mut nodes = BTreeMap::new();
    nodes.insert(NodeId::new("price").expect("valid ID"), node_snapshot);
    let snapshot = CalculationSnapshot {
        schema_version: SchemaVersion::new(1).expect("valid schema"),
        graph_version: RuleVersion::new(1, 0, 0),
        nodes,
    };

    let canonical = snapshot.to_canonical_json().expect("snapshot serializes");

    assert_eq!(
        canonical,
        r#"{"schema_version":1,"graph_version":{"major":1,"minor":0,"patch":0},"nodes":{"price":{"rule":{"id":"CALC-016","version":{"major":1,"minor":0,"patch":0}},"inputs":{"cost":{"kind":"money","value":{"amount":"100","currency":"USD"}}},"result":{"state":"available","value":{"kind":"money","value":{"amount":"125","currency":"USD"}}},"intermediate_trace":{"markup_factor":{"kind":"decimal","value":"1.25"}},"provenance":{"kind":"calculated","source_id":"engine","revision":"1","recorded_at":null},"warnings":[]}}}"#
    );
}
