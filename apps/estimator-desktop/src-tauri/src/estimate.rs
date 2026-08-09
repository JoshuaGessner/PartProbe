use std::str::FromStr;

use partprobe_application::{
    DraftBaseCostInputs, DraftEstimateInputs, DraftEstimateSession, DraftGeometryReview,
    DraftMaterialCostInputs, DraftOperationCostInputs, DraftQuantityInputs, DraftRateContext,
    DraftStockInputs, DraftTimeInputs,
};
use partprobe_desktop_contract::{
    DeveloperPricingInputFields, DeveloperRateInputFields, DraftEstimateEvaluation,
    DraftEstimateEvaluationState, DraftEstimateInputFields, DraftEstimateResultSummary,
    EvaluateDraftEstimateRequest, HostCommandError, PricingPolicySummary, ResolvedRateSummary,
};
use partprobe_domain::{
    CostCategory, CurrencyCode, DensityKilogramsPerCubicMillimeter, EffectiveDate, ItemQuantity,
    Money, PricingMethod, PricingPolicy, PricingPolicyId, RateApprovalState, RateBasis, RateCard,
    RateCardId, RateComposition, RateEntry, RateEvent, RateGovernance, RateId, RateScope,
    RateVersion, RecordedAt, RoundingBoundary, RoundingMode, RoundingPolicy, RoundingPolicyId,
    SourceKind, SourceRef, ValueState, VolumeCubicMillimeters,
};
use rust_decimal::Decimal;

use crate::analysis::{DEVELOPER_ACTOR_ID, trusted_recorded_at};

pub fn evaluate_draft_estimate(
    session: &mut DraftEstimateSession,
    request: &EvaluateDraftEstimateRequest,
) -> Result<DraftEstimateEvaluation, HostCommandError> {
    let review = DraftGeometryReview::new(
        request.review.canonical_units_reviewed,
        request.review.warnings_reviewed,
    );
    let inputs = estimate_inputs(&request.inputs, &request.rates.currency)?;
    let recorded_at = trusted_recorded_at()?;
    let rate_context = rate_context(&request.rates, &request.analysis_id, &recorded_at)?;
    let pricing_policy = pricing_policy(&request.pricing, &request.rates.currency)?;

    session.set_geometry_review(review);
    session.set_inputs(inputs);
    session.set_rate_context(rate_context);
    session.set_pricing_policy(pricing_policy);

    Ok(match session.evaluate() {
        ValueState::Available { value } => DraftEstimateEvaluation {
            selection_id: request.selection_id.clone(),
            analysis_id: request.analysis_id.clone(),
            state: DraftEstimateEvaluationState::Available,
            reason: None,
            result: Some(Box::new(result_summary(value, request.inputs.clone()))),
        },
        ValueState::Unavailable { reason } => unavailable_evaluation(request, reason),
        ValueState::Blocked { reason } => blocked_evaluation(request, reason),
        ValueState::Unknown { reason } => blocked_evaluation(request, reason),
        ValueState::Stale { reason, .. } => blocked_evaluation(request, reason),
    })
}

fn estimate_inputs(
    fields: &DraftEstimateInputFields,
    currency: &str,
) -> Result<DraftEstimateInputs, HostCommandError> {
    let currency =
        CurrencyCode::new(currency).map_err(|_| invalid_input("GUI4-ESTIMATE-CURRENCY"))?;
    Ok(DraftEstimateInputs {
        stock: DraftStockInputs {
            stock_volume: VolumeCubicMillimeters::new(decimal(&fields.stock_volume_mm3)?)
                .map_err(|_| invalid_input("GUI4-ESTIMATE-STOCK-VOLUME"))?,
            density: DensityKilogramsPerCubicMillimeter::new(decimal(&fields.density_kg_per_mm3)?)
                .map_err(|_| invalid_input("GUI4-ESTIMATE-DENSITY"))?,
        },
        quantities: DraftQuantityInputs {
            deliver: ItemQuantity::new(whole(&fields.deliver_quantity)?),
            planned_spares: ItemQuantity::new(whole(&fields.planned_spares)?),
            destructive_samples: ItemQuantity::new(whole(&fields.destructive_samples)?),
        },
        times: DraftTimeInputs {
            setup_hours: decimal(&fields.setup_hours)?,
            programming_hours: decimal(&fields.programming_hours)?,
            cutting_hours_per_item: decimal(&fields.cutting_hours_per_item)?,
            non_cutting_hours_per_item: decimal(&fields.non_cutting_hours_per_item)?,
            load_unload_hours_per_item: decimal(&fields.load_unload_hours_per_item)?,
            in_cycle_inspection_hours_per_item: decimal(
                &fields.in_cycle_inspection_hours_per_item,
            )?,
            quality_inspection_hours: decimal(&fields.quality_inspection_hours)?,
        },
        material: DraftMaterialCostInputs {
            purchased: money(&fields.purchased_material, &currency)?,
            cut: money(&fields.cut_charge, &currency)?,
            certificate: money(&fields.material_certificate, &currency)?,
            inbound_freight: money(&fields.inbound_freight, &currency)?,
            approved_remnant_credit: money(&fields.approved_remnant_credit, &currency)?,
        },
        operation: DraftOperationCostInputs {
            prove_out: money(&fields.prove_out, &currency)?,
            tooling: money(&fields.tooling, &currency)?,
            consumables: money(&fields.consumables, &currency)?,
            fixture: money(&fields.fixture, &currency)?,
            outside: money(&fields.outside_processing, &currency)?,
            freight: money(&fields.operation_freight, &currency)?,
        },
        base: DraftBaseCostInputs {
            nonrecurring_engineering: money(&fields.nonrecurring_engineering, &currency)?,
            administration: money(&fields.administration, &currency)?,
            overhead: money(&fields.overhead, &currency)?,
            accepted_risk_impacts: vec![money(&fields.accepted_risk_impact, &currency)?],
            expected_rework: money(&fields.expected_rework, &currency)?,
        },
    })
}

fn rate_context(
    fields: &DeveloperRateInputFields,
    analysis_id: &str,
    recorded_at: &RecordedAt,
) -> Result<DraftRateContext, HostCommandError> {
    if !fields.confirmed_for_session {
        return Err(invalid_input("GUI4-ESTIMATE-RATE-CONFIRMATION"));
    }
    let currency = CurrencyCode::new(&fields.currency)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-CURRENCY"))?;
    let version = RateVersion::new(version(&fields.rate_card_version)?)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-VERSION"))?;
    let effective_on = EffectiveDate::new(&fields.effective_on)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-EFFECTIVE-DATE"))?;
    let entries = [
        (
            "setup-labor",
            CostCategory::SetupLabor,
            &fields.setup_labor_per_hour,
        ),
        (
            "programming",
            CostCategory::Programming,
            &fields.programming_per_hour,
        ),
        (
            "run-labor",
            CostCategory::RunLabor,
            &fields.run_labor_per_hour,
        ),
        ("machine", CostCategory::Machine, &fields.machine_per_hour),
        (
            "quality-inspection",
            CostCategory::QualityInspection,
            &fields.quality_inspection_per_hour,
        ),
    ]
    .into_iter()
    .map(|(id, category, amount)| {
        rate_entry(
            id,
            category,
            amount,
            &currency,
            version,
            &effective_on,
            analysis_id,
            recorded_at,
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    let card = RateCard::new(
        RateCardId::new(&fields.rate_card_id)
            .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-CARD-ID"))?,
        version,
        currency,
        entries,
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-CARD"))?;
    DraftRateContext::new(card, effective_on, vec![RateScope::organization()])
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-CONTEXT"))
}

#[allow(clippy::too_many_arguments)]
fn rate_entry(
    id: &str,
    category: CostCategory,
    amount: &str,
    currency: &CurrencyCode,
    version: RateVersion,
    effective_from: &EffectiveDate,
    analysis_id: &str,
    recorded_at: &RecordedAt,
) -> Result<RateEntry, HostCommandError> {
    let entered = RateEvent::new(
        DEVELOPER_ACTOR_ID,
        recorded_at.clone(),
        "entered for a session-only developer estimate",
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-EVENT"))?;
    let approved = RateEvent::new(
        DEVELOPER_ACTOR_ID,
        recorded_at.clone(),
        "explicitly confirmed for this session-only calculation",
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-APPROVAL"))?;
    let governance = RateGovernance::new(RateApprovalState::Approved, entered, Some(approved))
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-GOVERNANCE"))?;
    let source = SourceRef::new(
        SourceKind::Manual,
        "gui-4-session-rate-input",
        Some(analysis_id.to_owned()),
        Some(recorded_at.clone()),
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-SOURCE"))?;
    RateEntry::new(
        RateId::new(id).map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-ID"))?,
        version,
        category,
        RateComposition::Component,
        RateScope::organization(),
        DEVELOPER_ACTOR_ID,
        money(amount, currency)?,
        RateBasis::PerHour,
        effective_from.clone(),
        None,
        governance,
        source,
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-ENTRY"))
}

fn pricing_policy(
    fields: &DeveloperPricingInputFields,
    currency: &str,
) -> Result<PricingPolicy, HostCommandError> {
    if !fields.confirmed_for_session {
        return Err(invalid_input("GUI4-ESTIMATE-PRICING-CONFIRMATION"));
    }
    let currency =
        CurrencyCode::new(currency).map_err(|_| invalid_input("GUI4-ESTIMATE-PRICING-CURRENCY"))?;
    let version = RateVersion::new(version(&fields.pricing_policy_version)?)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-PRICING-VERSION"))?;
    let rounding_scale = fields
        .rounding_decimal_places
        .parse::<u32>()
        .map_err(|_| invalid_input("GUI4-ESTIMATE-ROUNDING-SCALE"))?;
    let rounding = RoundingPolicy::new(
        RoundingPolicyId::new(format!("{}-quote-total", fields.pricing_policy_id))
            .map_err(|_| invalid_input("GUI4-ESTIMATE-ROUNDING-ID"))?,
        version,
        currency.clone(),
        rounding_scale,
        RoundingMode::HalfEven,
        RoundingBoundary::QuoteTotal,
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-ROUNDING-POLICY"))?;
    PricingPolicy::new(
        PricingPolicyId::new(&fields.pricing_policy_id)
            .map_err(|_| invalid_input("GUI4-ESTIMATE-PRICING-ID"))?,
        version,
        currency.clone(),
        PricingMethod::Markup {
            rate: decimal(&fields.markup_rate)?,
        },
        optional_money(&fields.optional_price_floor, &currency)?,
        optional_money(&fields.optional_minimum_order, &currency)?,
        rounding,
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-PRICING-POLICY"))
}

fn result_summary(
    result: partprobe_application::DraftEstimateResult,
    input_trace: DraftEstimateInputFields,
) -> DraftEstimateResultSummary {
    let currency = result.total_internal_cost.currency().as_str().to_owned();
    let resolved_rates = [
        ("setup_labor", &result.trace.resolved_rates.setup_labor),
        ("programming", &result.trace.resolved_rates.programming),
        ("run_labor", &result.trace.resolved_rates.run_labor),
        ("machine", &result.trace.resolved_rates.machine),
        (
            "quality_inspection",
            &result.trace.resolved_rates.quality_inspection,
        ),
    ]
    .into_iter()
    .map(|(category, rate)| ResolvedRateSummary {
        category: category.to_owned(),
        entry_id: rate.entry.id().as_str().to_owned(),
        amount_per_hour: decimal_text(rate.entry.amount().amount()),
        card_id: rate.card_id.as_str().to_owned(),
        card_version: rate.card_version.value(),
        effective_on: rate.effective_on.as_str().to_owned(),
        scope_rank: rate.scope_rank,
        selector_id: rate.selector_id.clone(),
        selector_version: rate.selector_version.to_string(),
        reason: rate.reason.clone(),
    })
    .collect();
    let pricing_policy = pricing_summary(&result.trace.pricing_policy);
    DraftEstimateResultSummary {
        currency,
        net_part_volume_mm3: decimal_text(result.net_part_volume.value()),
        part_mass_kg: decimal_text(result.part_mass.value()),
        stock_mass_kg: decimal_text(result.stock_mass.value()),
        removed_volume_mm3: decimal_text(result.removed_volume.value.value()),
        removed_volume_warnings: result.removed_volume.warnings,
        make_quantity: result.make_quantity.value(),
        material_cost: money_text(&result.material_cost),
        setup_cost: money_text(&result.setup_cost),
        programming_cost: money_text(&result.programming_cost),
        cycle_hours_per_item: decimal_text(result.cycle_hours_per_item),
        run_cost: money_text(&result.run_cost),
        quality_inspection_cost: money_text(&result.quality_inspection_cost),
        operation_cost: money_text(&result.operation_cost),
        base_internal_cost: money_text(&result.base_internal_cost),
        risk_reserve: money_text(&result.risk_reserve),
        total_internal_cost: money_text(&result.total_internal_cost),
        formula_price: money_text(&result.pricing.formula_price),
        governed_price: money_text(&result.pricing.governed_price),
        rounded_selling_price: money_text(&result.pricing.rounded_price.rounded),
        floor_applied: result.pricing.floor_applied,
        minimum_order_applied: result.pricing.minimum_order_applied,
        input_trace,
        resolved_rates,
        pricing_policy,
        calculation_rule_ids: result
            .trace
            .calculation_rule_ids
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

fn pricing_summary(policy: &PricingPolicy) -> PricingPolicySummary {
    let (method, method_rate) = match policy.method() {
        PricingMethod::Markup { rate } => ("markup", decimal_text(*rate)),
        PricingMethod::TargetMargin { rate } => ("target_margin", decimal_text(*rate)),
    };
    PricingPolicySummary {
        policy_id: policy.id().as_str().to_owned(),
        policy_version: policy.version().value(),
        method: method.to_owned(),
        method_rate,
        rounding_decimal_places: policy.quote_total_rounding().scale(),
        rounding_mode: rounding_mode_name(policy.quote_total_rounding().mode()).to_owned(),
    }
}

fn rounding_mode_name(mode: RoundingMode) -> &'static str {
    match mode {
        RoundingMode::HalfEven => "half_even",
        RoundingMode::HalfAwayFromZero => "half_away_from_zero",
        RoundingMode::HalfTowardZero => "half_toward_zero",
        RoundingMode::TowardZero => "toward_zero",
        RoundingMode::AwayFromZero => "away_from_zero",
        RoundingMode::TowardNegativeInfinity => "toward_negative_infinity",
        RoundingMode::TowardPositiveInfinity => "toward_positive_infinity",
    }
}

fn unavailable_evaluation(
    request: &EvaluateDraftEstimateRequest,
    reason: String,
) -> DraftEstimateEvaluation {
    DraftEstimateEvaluation {
        selection_id: request.selection_id.clone(),
        analysis_id: request.analysis_id.clone(),
        state: DraftEstimateEvaluationState::Unavailable,
        reason: Some(reason),
        result: None,
    }
}

fn blocked_evaluation(
    request: &EvaluateDraftEstimateRequest,
    reason: String,
) -> DraftEstimateEvaluation {
    DraftEstimateEvaluation {
        selection_id: request.selection_id.clone(),
        analysis_id: request.analysis_id.clone(),
        state: DraftEstimateEvaluationState::Blocked,
        reason: Some(reason),
        result: None,
    }
}

fn decimal(value: &str) -> Result<Decimal, HostCommandError> {
    if value.trim().is_empty() {
        return Err(invalid_input("GUI4-ESTIMATE-DECIMAL-MISSING"));
    }
    Decimal::from_str(value).map_err(|_| invalid_input("GUI4-ESTIMATE-DECIMAL"))
}

fn whole(value: &str) -> Result<u64, HostCommandError> {
    value
        .parse::<u64>()
        .map_err(|_| invalid_input("GUI4-ESTIMATE-WHOLE-NUMBER"))
}

fn version(value: &str) -> Result<u32, HostCommandError> {
    value
        .parse::<u32>()
        .map_err(|_| invalid_input("GUI4-ESTIMATE-VERSION"))
}

fn money(value: &str, currency: &CurrencyCode) -> Result<Money, HostCommandError> {
    Ok(Money::new(decimal(value)?, currency.clone()))
}

fn optional_money(value: &str, currency: &CurrencyCode) -> Result<Option<Money>, HostCommandError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        money(value, currency).map(Some)
    }
}

fn decimal_text(value: Decimal) -> String {
    value.normalize().to_string()
}

fn money_text(value: &Money) -> String {
    decimal_text(value.amount())
}

fn invalid_input(diagnostic_id: &'static str) -> HostCommandError {
    HostCommandError::invalid_estimate_input(diagnostic_id)
}

#[cfg(test)]
mod tests {
    use partprobe_desktop_contract::{DeveloperPricingInputFields, DeveloperRateInputFields};

    use super::*;

    #[test]
    fn unconfirmed_session_rates_fail_before_domain_approval_is_constructed() {
        let fields = DeveloperRateInputFields {
            confirmed_for_session: false,
            rate_card_id: "developer-card".to_owned(),
            rate_card_version: "1".to_owned(),
            effective_on: "2026-08-09".to_owned(),
            currency: "USD".to_owned(),
            setup_labor_per_hour: "25".to_owned(),
            programming_per_hour: "30".to_owned(),
            run_labor_per_hour: "20".to_owned(),
            machine_per_hour: "40".to_owned(),
            quality_inspection_per_hour: "35".to_owned(),
        };

        let error = rate_context(
            &fields,
            "analysis-1",
            &RecordedAt::new("test-time").expect("recorded time"),
        )
        .expect_err("unconfirmed rates must fail closed");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::InvalidEstimateInput
        );
    }

    #[test]
    fn unconfirmed_pricing_policy_fails_closed() {
        let fields = DeveloperPricingInputFields {
            confirmed_for_session: false,
            pricing_policy_id: "developer-pricing".to_owned(),
            pricing_policy_version: "1".to_owned(),
            markup_rate: "0.35".to_owned(),
            optional_price_floor: String::new(),
            optional_minimum_order: String::new(),
            rounding_decimal_places: "2".to_owned(),
        };

        let error =
            pricing_policy(&fields, "USD").expect_err("unconfirmed pricing must fail closed");

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::InvalidEstimateInput
        );
    }
}
