use std::str::FromStr;

use partprobe_domain::{
    CostCategory, CurrencyCode, EffectiveDate, ItemQuantity, Money, PricingPolicy, RateBasis,
    RateCard, RateEntry, RateScope, RateScopeKind, ValueState,
};
use partprobe_estimation_engine::{
    BaseInternalCostComponents, MaterialCostComponents, OperationCostComponents,
    apply_pricing_policy, base_internal_cost, cycle_time, extend_rate, material_cost,
    operation_cost, resolve_rate, risk_reserve, run_cost, setup_lot_cost, setup_unit_cost,
    total_internal_cost,
};
use rust_decimal::Decimal;
use serde::Deserialize;

const FIXTURE_JSON: &str = include_str!("fixtures/task_002/golden_estimates.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GoldenFixture {
    schema_version: u16,
    classification: String,
    rate_card: RateCard,
    pricing_policy: PricingPolicy,
    examples: Vec<ExampleFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExampleFixture {
    id: String,
    effective_on: EffectiveDate,
    deliver_quantity: u64,
    make_quantity: u64,
    setup_hours: String,
    programming_hours: String,
    cutting_hours: String,
    non_cutting_hours: String,
    load_unload_hours: String,
    in_cycle_inspection_hours: String,
    inspection_hours: String,
    material: [String; 5],
    operation_other: [String; 6],
    nre: String,
    administration: String,
    overhead: String,
    risk: String,
    rework: String,
    expected: ExpectedTrace,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedTrace {
    setup: String,
    setup_unit: String,
    programming: String,
    cycle_hours: String,
    run: String,
    quality: String,
    material: String,
    operation: String,
    base: String,
    risk: String,
    total: String,
    formula_price: String,
    rounded_price: String,
}

fn fixture() -> GoldenFixture {
    serde_json::from_str(FIXTURE_JSON).expect("TASK-002 fixture must satisfy domain invariants")
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str(value).expect("fixture decimal must be valid")
}

fn usd(value: &str) -> Money {
    Money::new(
        decimal(value),
        CurrencyCode::new("USD").expect("USD must be valid"),
    )
}

fn available<T: std::fmt::Debug>(state: ValueState<T>) -> T {
    match state {
        ValueState::Available { value } => value,
        other => panic!("expected available rate, got {other:?}"),
    }
}

fn resolved_rate(
    fixture: &GoldenFixture,
    example: &ExampleFixture,
    category: CostCategory,
) -> partprobe_estimation_engine::ResolvedRate {
    available(resolve_rate(
        &fixture.rate_card,
        category,
        RateBasis::PerHour,
        &example.effective_on,
        &[RateScope::organization()],
    ))
}

#[test]
fn synthetic_fixture_is_explicitly_nonproduction_and_versioned() {
    let fixture = fixture();

    assert_eq!(fixture.schema_version, 2);
    assert_eq!(fixture.classification, "synthetic_test_only");
    assert_eq!(fixture.rate_card.id().as_str(), "synthetic-task-002");
    assert!(!fixture.rate_card.entries().is_empty());
}

#[test]
fn empty_card_keeps_missing_rates_unavailable() {
    let fixture = fixture();
    let empty = RateCard::empty(
        fixture.rate_card.id().clone(),
        fixture.rate_card.version(),
        fixture.rate_card.currency().clone(),
    );

    let result = resolve_rate(
        &empty,
        CostCategory::Machine,
        RateBasis::PerHour,
        &EffectiveDate::new("2026-07-29").expect("date must be valid"),
        &[RateScope::organization()],
    );

    assert!(matches!(result, ValueState::Unavailable { .. }));
}

#[test]
fn equally_applicable_approved_rates_block_resolution() {
    let fixture = fixture();
    let first = fixture
        .rate_card
        .entries()
        .iter()
        .find(|entry| entry.category() == CostCategory::Machine)
        .expect("fixture has a machine rate");
    let mut second_value = serde_json::to_value(first).expect("serialize entry");
    second_value["id"] = serde_json::Value::String("machine-conflict".to_owned());
    let second: RateEntry = serde_json::from_value(second_value).expect("valid conflicting rate");
    let conflict = RateCard::new(
        fixture.rate_card.id().clone(),
        fixture.rate_card.version(),
        fixture.rate_card.currency().clone(),
        vec![first.clone(), second],
    )
    .expect("conflicting applicability is retained for explicit resolution");

    let result = resolve_rate(
        &conflict,
        CostCategory::Machine,
        RateBasis::PerHour,
        &EffectiveDate::new("2026-07-29").expect("date must be valid"),
        &[RateScope::organization()],
    );

    assert!(matches!(result, ValueState::Blocked { .. }));
}

#[test]
fn ordered_scope_selection_is_explicit_and_deterministic() {
    let fixture = fixture();
    let organization = fixture
        .rate_card
        .entries()
        .iter()
        .find(|entry| entry.category() == CostCategory::SetupLabor)
        .expect("fixture has a setup rate");
    let mut machine_value = serde_json::to_value(organization).expect("serialize entry");
    machine_value["id"] = serde_json::Value::String("setup-machine-1".to_owned());
    machine_value["scope"] = serde_json::json!({
        "kind": "machine",
        "reference_id": "machine-1"
    });
    machine_value["amount"]["amount"] = serde_json::Value::String("40".to_owned());
    let machine: RateEntry =
        serde_json::from_value(machine_value).expect("machine rate must be valid");
    let card = RateCard::new(
        fixture.rate_card.id().clone(),
        fixture.rate_card.version(),
        fixture.rate_card.currency().clone(),
        vec![organization.clone(), machine],
    )
    .expect("card must be valid");
    let machine_scope = RateScope::new(RateScopeKind::Machine, Some("machine-1".to_owned()))
        .expect("scope must be valid");

    let selected = available(resolve_rate(
        &card,
        CostCategory::SetupLabor,
        RateBasis::PerHour,
        &EffectiveDate::new("2026-07-29").expect("date must be valid"),
        &[machine_scope.clone(), RateScope::organization()],
    ));

    assert_eq!(selected.entry.id().as_str(), "setup-machine-1");
    assert_eq!(selected.scope_rank, 0);
    assert_eq!(
        selected.requested_scopes,
        vec![machine_scope, RateScope::organization()]
    );
}

#[test]
fn unapproved_or_out_of_period_rates_are_unavailable() {
    let fixture = fixture();
    let first = fixture
        .rate_card
        .entries()
        .iter()
        .find(|entry| entry.category() == CostCategory::Machine)
        .expect("fixture has a machine rate");
    let mut draft_value = serde_json::to_value(first).expect("serialize entry");
    draft_value["governance"]["state"] = serde_json::Value::String("draft".to_owned());
    draft_value["governance"]["decision"] = serde_json::Value::Null;
    let draft: RateEntry = serde_json::from_value(draft_value).expect("draft rate must be valid");
    let draft_card = RateCard::new(
        fixture.rate_card.id().clone(),
        fixture.rate_card.version(),
        fixture.rate_card.currency().clone(),
        vec![draft],
    )
    .expect("draft card must be valid");

    let draft_result = resolve_rate(
        &draft_card,
        CostCategory::Machine,
        RateBasis::PerHour,
        &EffectiveDate::new("2026-07-29").expect("date must be valid"),
        &[RateScope::organization()],
    );
    let prior_result = resolve_rate(
        &fixture.rate_card,
        CostCategory::Machine,
        RateBasis::PerHour,
        &EffectiveDate::new("2025-12-31").expect("date must be valid"),
        &[RateScope::organization()],
    );

    assert!(matches!(draft_result, ValueState::Unavailable { .. }));
    assert!(matches!(prior_result, ValueState::Unavailable { .. }));
}

#[test]
fn pinned_rate_card_version_replays_after_a_new_version_exists() {
    let fixture = fixture();
    let example = &fixture.examples[0];
    let original = resolved_rate(&fixture, example, CostCategory::SetupLabor);
    let original_cost =
        setup_lot_cost(decimal("3"), &original).expect("original extension must be exact");
    let mut replacement_value =
        serde_json::to_value(&original.entry).expect("serialize original entry");
    replacement_value["version"] = serde_json::json!(2);
    replacement_value["amount"]["amount"] = serde_json::Value::String("40".to_owned());
    let replacement: RateEntry =
        serde_json::from_value(replacement_value).expect("replacement must be valid");
    let replacement_card = RateCard::new(
        fixture.rate_card.id().clone(),
        partprobe_domain::RateVersion::new(2).expect("version must be valid"),
        fixture.rate_card.currency().clone(),
        vec![replacement],
    )
    .expect("replacement card must be valid");
    let replacement_resolved = available(resolve_rate(
        &replacement_card,
        CostCategory::SetupLabor,
        RateBasis::PerHour,
        &example.effective_on,
        &[RateScope::organization()],
    ));

    assert_eq!(original.card_version.value(), 1);
    assert_eq!(replacement_resolved.card_version.value(), 2);
    assert_eq!(original_cost, usd("75"));
    assert_eq!(
        setup_lot_cost(decimal("3"), &replacement_resolved)
            .expect("replacement extension must be exact"),
        usd("120")
    );
    assert_eq!(
        setup_lot_cost(decimal("3"), &original).expect("replay must remain exact"),
        usd("75")
    );

    let snapshot_value =
        partprobe_estimation_engine::SnapshotValue::ResolvedRate(Box::new(original));
    let serialized = serde_json::to_string(&snapshot_value).expect("rate trace must serialize");
    assert!(serialized.contains("\"card_version\":1"));
    assert!(serialized.contains("\"version\":1"));
    assert!(serialized.contains("\"selector_id\":\"ordered_scope_approved_effective\""));
    assert!(serialized.contains("\"selector_version\":{\"major\":1,\"minor\":0,\"patch\":0}"));
    assert!(serialized.contains("\"requested_scopes\":[{\"kind\":\"organization\""));
}

#[test]
fn operation_cost_rejects_hidden_negative_components() {
    let result = operation_cost(&OperationCostComponents {
        setup: usd("0"),
        programming: usd("0"),
        prove_out: usd("0"),
        run: usd("0"),
        tooling: usd("-1"),
        consumables: usd("0"),
        fixture: usd("0"),
        quality_inspection: usd("0"),
        outside: usd("0"),
        freight: usd("0"),
    });

    assert!(result.is_err());
}

#[test]
fn run_cost_rejects_duplicate_and_mixed_composite_charges() {
    let fixture = fixture();
    let example = &fixture.examples[0];
    let run_labor = resolved_rate(&fixture, example, CostCategory::RunLabor);
    let machine = resolved_rate(&fixture, example, CostCategory::Machine);
    let duplicate = run_cost(
        decimal("1"),
        ItemQuantity::new(1),
        &[run_labor.clone(), run_labor.clone()],
    );

    let mut composite_value = serde_json::to_value(&machine.entry).expect("serialize entry");
    composite_value["composition"] = serde_json::Value::String("composite".to_owned());
    let mut composite = machine;
    composite.entry =
        serde_json::from_value(composite_value).expect("composite rate must be valid");
    let mixed = run_cost(decimal("1"), ItemQuantity::new(1), &[run_labor, composite]);

    assert!(duplicate.is_err());
    assert!(mixed.is_err());
}

#[test]
fn setup_unit_cost_requires_explicit_rounding_when_division_is_nonterminating() {
    let result = setup_unit_cost(&usd("1"), ItemQuantity::new(3));

    assert!(result.is_err());
}

#[test]
fn ex_01_ex_03_and_ex_12_quantity_breaks_reconcile_exactly() {
    let fixture = fixture();

    for example in &fixture.examples {
        reconcile_example(&fixture, example);
    }
}

fn reconcile_example(fixture: &GoldenFixture, example: &ExampleFixture) {
    let setup_rate = resolved_rate(fixture, example, CostCategory::SetupLabor);
    let programming_rate = resolved_rate(fixture, example, CostCategory::Programming);
    let run_labor_rate = resolved_rate(fixture, example, CostCategory::RunLabor);
    let machine_rate = resolved_rate(fixture, example, CostCategory::Machine);
    let inspection_rate = resolved_rate(fixture, example, CostCategory::QualityInspection);

    let setup = setup_lot_cost(decimal(&example.setup_hours), &setup_rate)
        .expect("setup extension must be exact");
    let setup_unit = setup_unit_cost(&setup, ItemQuantity::new(example.deliver_quantity))
        .expect("setup amortization must be exact for fixture quantities");
    let programming = extend_rate(
        &programming_rate,
        RateBasis::PerHour,
        decimal(&example.programming_hours),
    )
    .expect("programming extension must be exact");
    let cycle = cycle_time(
        decimal(&example.cutting_hours),
        decimal(&example.non_cutting_hours),
        decimal(&example.load_unload_hours),
        decimal(&example.in_cycle_inspection_hours),
    )
    .expect("cycle time must be exact");
    let run = run_cost(
        cycle,
        ItemQuantity::new(example.make_quantity),
        &[run_labor_rate, machine_rate],
    )
    .expect("run extension must be exact");
    let quality = extend_rate(
        &inspection_rate,
        RateBasis::PerHour,
        decimal(&example.inspection_hours),
    )
    .expect("inspection extension must be exact");

    let material = material_cost(&MaterialCostComponents {
        purchased: usd(&example.material[0]),
        cut: usd(&example.material[1]),
        certificate: usd(&example.material[2]),
        inbound_freight: usd(&example.material[3]),
        approved_remnant_credit: usd(&example.material[4]),
    })
    .expect("material total must reconcile");
    assert_eq!(setup, usd(&example.expected.setup), "{} setup", example.id);
    assert_eq!(
        setup_unit,
        usd(&example.expected.setup_unit),
        "{} setup unit",
        example.id
    );
    assert_eq!(
        programming,
        usd(&example.expected.programming),
        "{} programming",
        example.id
    );
    assert_eq!(
        cycle,
        decimal(&example.expected.cycle_hours),
        "{} cycle",
        example.id
    );
    assert_eq!(run, usd(&example.expected.run), "{} run", example.id);
    assert_eq!(
        quality,
        usd(&example.expected.quality),
        "{} quality",
        example.id
    );
    assert_eq!(
        material,
        usd(&example.expected.material),
        "{} material",
        example.id
    );
    let operation = operation_cost(&OperationCostComponents {
        setup,
        programming,
        prove_out: usd(&example.operation_other[0]),
        run,
        tooling: usd(&example.operation_other[1]),
        consumables: usd(&example.operation_other[2]),
        fixture: usd(&example.operation_other[3]),
        quality_inspection: quality,
        outside: usd(&example.operation_other[4]),
        freight: usd(&example.operation_other[5]),
    })
    .expect("operation total must reconcile");
    assert_eq!(
        operation,
        usd(&example.expected.operation),
        "{} operation",
        example.id
    );
    let base = base_internal_cost(&BaseInternalCostComponents {
        material,
        operations: vec![operation],
        nonrecurring_engineering: usd(&example.nre),
        administration: usd(&example.administration),
        overhead: usd(&example.overhead),
    })
    .expect("base total must reconcile");
    let risk = risk_reserve(
        &CurrencyCode::new("USD").expect("USD must be valid"),
        &[usd(&example.risk)],
    )
    .expect("risk reserve must reconcile");
    let total = total_internal_cost(&base, &risk, &usd(&example.rework))
        .expect("total internal cost must reconcile");
    let pricing =
        apply_pricing_policy(&total, &fixture.pricing_policy).expect("pricing must reconcile");

    assert_eq!(base, usd(&example.expected.base), "{} base", example.id);
    assert_eq!(risk, usd(&example.expected.risk), "{} risk", example.id);
    assert_eq!(total, usd(&example.expected.total), "{} total", example.id);
    assert_eq!(
        pricing.formula_price,
        usd(&example.expected.formula_price),
        "{} formula price",
        example.id
    );
    assert_eq!(
        pricing.rounded_price.rounded,
        usd(&example.expected.rounded_price),
        "{} rounded price",
        example.id
    );
    assert_eq!(pricing.pricing_policy_id.as_str(), "synthetic-markup");
    assert_eq!(pricing.pricing_policy_version.value(), 1);
}
