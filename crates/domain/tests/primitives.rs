use partprobe_domain::{
    CurrencyCode, DensityKilogramsPerCubicMillimeter, DomainError, Money, RuleId, SchemaVersion,
    SourceRef, VolumeCubicMillimeters,
};
use rust_decimal::Decimal;
use std::str::FromStr;

fn decimal(mantissa: i64, scale: u32) -> Decimal {
    Decimal::new(mantissa, scale)
}

#[test]
fn money_addition_is_exact_and_currency_checked() {
    let usd = CurrencyCode::new("USD").expect("valid currency");
    let eur = CurrencyCode::new("EUR").expect("valid currency");
    let left = Money::new(decimal(1_001, 2), usd.clone());
    let right = Money::new(decimal(2_002, 2), usd);

    let total = left.checked_add(&right).expect("same currency adds");
    assert_eq!(total.amount(), decimal(3_003, 2));

    let mismatch = total
        .checked_add(&Money::new(Decimal::ONE, eur))
        .expect_err("different currencies must fail");
    assert!(matches!(mismatch, DomainError::CurrencyMismatch { .. }));
}

#[test]
fn decimal_money_serializes_as_a_string() {
    let money = Money::new(
        decimal(1_234_567, 4),
        CurrencyCode::new("USD").expect("valid currency"),
    );
    let serialized = serde_json::to_string(&money).expect("money serializes");

    assert_eq!(serialized, r#"{"amount":"123.4567","currency":"USD"}"#);
}

#[test]
fn physical_quantities_reject_negative_values() {
    assert!(VolumeCubicMillimeters::new(decimal(-1, 0)).is_err());
    assert!(DensityKilogramsPerCubicMillimeter::new(decimal(-1, 9)).is_err());
}

#[test]
fn currency_code_requires_three_uppercase_ascii_letters() {
    assert!(CurrencyCode::new("USD").is_ok());
    assert!(CurrencyCode::new("usd").is_err());
    assert!(CurrencyCode::new("US").is_err());
    assert!(CurrencyCode::new("EURO").is_err());
}

#[test]
fn deserialization_cannot_bypass_validated_constructors() {
    assert!(serde_json::from_str::<CurrencyCode>(r#""usd""#).is_err());
    assert!(serde_json::from_str::<VolumeCubicMillimeters>(r#""-1""#).is_err());
    assert!(serde_json::from_str::<RuleId>(r#""""#).is_err());
    assert!(serde_json::from_str::<SchemaVersion>("0").is_err());
    assert!(
        serde_json::from_str::<SourceRef>(
            r#"{"kind":"manual","source_id":"","revision":null,"recorded_at":null}"#
        )
        .is_err()
    );
}

#[test]
fn decimal_serialization_normalizes_scale_and_negative_zero() {
    let usd = CurrencyCode::new("USD").expect("valid currency");
    let first = Money::new(decimal(10_000, 2), usd.clone());
    let second = Money::new(decimal(1_000_000, 4), usd.clone());
    let mut negative_zero_decimal = decimal(0, 2);
    negative_zero_decimal.set_sign_negative(true);
    let negative_zero = Money::new(negative_zero_decimal, usd);

    assert_eq!(
        serde_json::to_string(&first).expect("money serializes"),
        serde_json::to_string(&second).expect("money serializes")
    );
    assert_eq!(
        serde_json::to_string(&negative_zero).expect("money serializes"),
        r#"{"amount":"0","currency":"USD"}"#
    );
}

#[test]
fn exact_multiplication_cancels_scale_before_bounded_mantissa_check() {
    let currency = CurrencyCode::new("USD").expect("valid currency");
    let amount = Decimal::from_i128_with_scale(2_i128.pow(41), 20);
    let factor = Decimal::from_i128_with_scale(5_i128.pow(41), 21);
    let money = Money::new(amount, currency);

    let product = money
        .checked_mul(factor)
        .expect("cross-operand decimal factors make the exact product representable");

    assert_eq!(product.amount(), Decimal::ONE);
}

#[test]
fn addition_and_subtraction_reject_precision_losing_rescale() {
    let currency = CurrencyCode::new("USD").expect("valid currency");
    let large = Money::new(
        Decimal::from_str("108053.27500000000000000000000").expect("valid decimal"),
        currency.clone(),
    );
    let tiny = Money::new(
        Decimal::from_str("0.000000000000000000000001").expect("valid decimal"),
        currency,
    );

    assert!(matches!(
        large.checked_add(&tiny),
        Err(DomainError::RoundingRequired { .. })
    ));
    assert!(matches!(
        large.checked_sub(&tiny),
        Err(DomainError::RoundingRequired { .. })
    ));
}
