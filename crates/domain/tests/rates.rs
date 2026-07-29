use std::str::FromStr;

use partprobe_domain::{
    CostCategory, CurrencyCode, EffectiveDate, Money, PricingMethod, PricingPolicy,
    PricingPolicyId, RateApprovalState, RateBasis, RateCard, RateCardId, RateComposition,
    RateEntry, RateEvent, RateGovernance, RateId, RateScope, RateVersion, RecordedAt,
    RoundingBoundary, RoundingMode, RoundingPolicy, RoundingPolicyId, SourceKind, SourceRef,
};
use rust_decimal::Decimal;

fn decimal(value: &str) -> Decimal {
    Decimal::from_str(value).expect("test decimal must be valid")
}

fn usd(value: &str) -> Money {
    Money::new(
        decimal(value),
        CurrencyCode::new("USD").expect("USD must be valid"),
    )
}

fn event(actor: &str, reason: &str) -> RateEvent {
    RateEvent::new(
        actor,
        RecordedAt::new("2026-07-29T12:00:00Z").expect("timestamp must be nonempty"),
        reason,
    )
    .expect("event must be valid")
}

fn approved_governance() -> RateGovernance {
    RateGovernance::new(
        RateApprovalState::Approved,
        event("estimator-1", "entered test-only rate"),
        Some(event("approver-1", "approved test-only rate")),
    )
    .expect("approved governance must be valid")
}

fn source() -> SourceRef {
    SourceRef::new(
        SourceKind::Manual,
        "synthetic-test-fixture",
        Some("1".to_owned()),
        None,
    )
    .expect("source must be valid")
}

fn rate_entry(amount: Money) -> RateEntry {
    RateEntry::new(
        RateId::new("setup-labor").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CostCategory::SetupLabor,
        RateComposition::Component,
        RateScope::organization(),
        "finance-owner",
        amount,
        RateBasis::PerHour,
        EffectiveDate::new("2026-01-01").expect("date must be valid"),
        None,
        approved_governance(),
        source(),
    )
    .expect("rate entry must be valid")
}

#[test]
fn production_rate_card_can_start_empty() {
    let card = RateCard::empty(
        RateCardId::new("production").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
    );

    assert!(card.entries().is_empty());
}

#[test]
fn effective_date_validates_calendar_boundaries() {
    assert!(EffectiveDate::new("2024-02-29").is_ok());
    assert!(EffectiveDate::new("2023-02-29").is_err());
    assert!(EffectiveDate::new("2026-13-01").is_err());
    assert!(EffectiveDate::new("0000-01-01").is_err());
    assert!(EffectiveDate::new("07/29/2026").is_err());
}

#[test]
fn approved_rate_requires_a_decision_record() {
    let result = RateGovernance::new(
        RateApprovalState::Approved,
        event("estimator-1", "entered"),
        None,
    );

    assert!(result.is_err());

    let transitioned = RateGovernance::with_prior_decisions(
        RateApprovalState::Superseded,
        event("estimator-1", "entered"),
        vec![event("approver-1", "approved")],
        Some(event("approver-1", "superseded by version 2")),
    )
    .expect("transition history must be retained");
    assert_eq!(transitioned.prior_decisions().len(), 1);
}

#[test]
fn rate_entry_rejects_negative_amount_and_reversed_dates() {
    let negative = RateEntry::new(
        RateId::new("negative").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CostCategory::Machine,
        RateComposition::Component,
        RateScope::organization(),
        "finance-owner",
        usd("-1"),
        RateBasis::PerHour,
        EffectiveDate::new("2026-01-01").expect("date must be valid"),
        None,
        approved_governance(),
        source(),
    );
    let reversed = RateEntry::new(
        RateId::new("reversed").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CostCategory::Machine,
        RateComposition::Component,
        RateScope::organization(),
        "finance-owner",
        usd("1"),
        RateBasis::PerHour,
        EffectiveDate::new("2026-02-01").expect("date must be valid"),
        Some(EffectiveDate::new("2026-01-01").expect("date must be valid")),
        approved_governance(),
        source(),
    );

    assert!(negative.is_err());
    assert!(reversed.is_err());
}

#[test]
fn card_rejects_currency_mismatch_and_duplicate_stable_ids() {
    let eur = Money::new(
        decimal("25"),
        CurrencyCode::new("EUR").expect("currency must be valid"),
    );
    let mismatch = RateCard::new(
        RateCardId::new("mismatch").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        vec![rate_entry(eur)],
    );
    let entry = rate_entry(usd("25"));
    let duplicate = RateCard::new(
        RateCardId::new("duplicate").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        vec![entry.clone(), entry],
    );

    assert!(mismatch.is_err());
    assert!(duplicate.is_err());
}

#[test]
fn rounding_policy_preserves_unrounded_value_and_half_even_result() {
    let policy = RoundingPolicy::new(
        RoundingPolicyId::new("usd-quote").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        2,
        RoundingMode::HalfEven,
        RoundingBoundary::QuoteTotal,
    )
    .expect("policy must be valid");

    let result = policy.apply(&usd("12.345")).expect("rounding must succeed");

    assert_eq!(result.unrounded, usd("12.345"));
    assert_eq!(result.rounded, usd("12.34"));
    assert_eq!(result.scale, 2);
    assert_eq!(result.mode, RoundingMode::HalfEven);
}

#[test]
fn pricing_policy_rejects_invalid_method_and_non_quote_rounding() {
    let presentation_rounding = RoundingPolicy::new(
        RoundingPolicyId::new("display").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        2,
        RoundingMode::HalfEven,
        RoundingBoundary::Presentation,
    )
    .expect("rounding policy must be valid");
    let invalid_margin = PricingPolicy::new(
        PricingPolicyId::new("invalid-margin").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        PricingMethod::TargetMargin { rate: Decimal::ONE },
        None,
        None,
        presentation_rounding.clone(),
    );
    let invalid_boundary = PricingPolicy::new(
        PricingPolicyId::new("invalid-boundary").expect("ID must be valid"),
        RateVersion::new(1).expect("version must be valid"),
        CurrencyCode::new("USD").expect("currency must be valid"),
        PricingMethod::Markup {
            rate: decimal("0.35"),
        },
        None,
        None,
        presentation_rounding,
    );

    assert!(invalid_margin.is_err());
    assert!(invalid_boundary.is_err());
}

#[test]
fn deserialization_revalidates_rate_invariants() {
    let mut value = serde_json::to_value(rate_entry(usd("25"))).expect("serialize rate");
    value["amount"]["amount"] = serde_json::Value::String("-25".to_owned());

    let result = serde_json::from_value::<RateEntry>(value);

    assert!(result.is_err());
}
