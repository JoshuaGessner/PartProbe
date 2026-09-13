use std::str::FromStr;

use partprobe_application::{
    DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_ID,
    DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_VERSION, DeveloperEstimateProposalApplication,
    DeveloperEstimateProposalReview, DraftBaseCostInputs, DraftEstimateInputs,
    DraftEstimateSession, DraftGeometryReview, DraftMaterialCostInputs, DraftOperationCostInputs,
    DraftQuantityInputs, DraftRateContext, DraftStockInputs, DraftTimeInputs,
};
#[cfg(test)]
use partprobe_desktop_contract::GeometryReviewInput;
use partprobe_desktop_contract::{
    DeveloperPricingInputFields, DeveloperRateInputFields, DraftEstimateEvaluation,
    DraftEstimateEvaluationState, DraftEstimateInputFields, DraftEstimateProposalAdoptionSummary,
    DraftEstimateResultSummary, EvaluateDraftEstimateRequest, HostCommandError,
    PricingPolicySummary, ResolvedRateSummary,
};
use partprobe_domain::{
    ActorId, CostCategory, CurrencyCode, DensityKilogramsPerCubicMillimeter, EffectiveDate,
    ItemQuantity, Money, PricingMethod, PricingPolicy, PricingPolicyId, RateApprovalState,
    RateBasis, RateCard, RateCardId, RateComposition, RateEntry, RateEvent, RateGovernance, RateId,
    RateScope, RateVersion, RecordedAt, RoundingBoundary, RoundingMode, RoundingPolicy,
    RoundingPolicyId, ShopSettingsDraft, SourceKind, SourceRef, ValueState, VolumeCubicMillimeters,
};
use rust_decimal::Decimal;

use crate::analysis::{DEVELOPER_ACTOR_ID, trusted_recorded_at};

pub fn evaluate_draft_estimate(
    session: &mut DraftEstimateSession,
    request: &EvaluateDraftEstimateRequest,
    settings: Option<&ShopSettingsDraft>,
) -> Result<DraftEstimateEvaluation, HostCommandError> {
    let review = DraftGeometryReview::new(
        request.review.canonical_units_reviewed,
        request.review.warnings_reviewed,
    );
    let recorded_at = trusted_recorded_at()?;
    let (inputs, input_trace, proposal_adoption) =
        if let Some(adoption) = request.proposal_adoption.as_ref() {
            proposed_estimate_inputs(session, settings, adoption, &recorded_at)?
        } else {
            (
                estimate_inputs(&request.inputs, &request.rates.currency)?,
                request.inputs.clone(),
                None,
            )
        };
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
            result: Some(Box::new(result_summary(
                value,
                input_trace,
                proposal_adoption,
            ))),
        },
        ValueState::Unavailable { reason } => unavailable_evaluation(request, reason),
        ValueState::Blocked { reason } => blocked_evaluation(request, reason),
        ValueState::Unknown { reason } => blocked_evaluation(request, reason),
        ValueState::Stale { reason, .. } => blocked_evaluation(request, reason),
    })
}

fn proposed_estimate_inputs(
    session: &DraftEstimateSession,
    settings: Option<&ShopSettingsDraft>,
    adoption: &partprobe_desktop_contract::DraftEstimateProposalAdoptionInput,
    recorded_at: &RecordedAt,
) -> Result<
    (
        DraftEstimateInputs,
        DraftEstimateInputFields,
        Option<DraftEstimateProposalAdoptionSummary>,
    ),
    HostCommandError,
> {
    let settings = settings.ok_or_else(|| invalid_input("USE3-ADOPTION-SETTINGS-MISSING"))?;
    if settings.revision().value() != adoption.settings_revision {
        return Err(invalid_input("USE3-ADOPTION-SETTINGS-STALE"));
    }
    let proposal = match DeveloperEstimateProposalApplication.propose(session.geometry(), settings)
    {
        ValueState::Available { value } => value,
        ValueState::Unavailable { .. } => {
            return Err(invalid_input("USE3-ADOPTION-PROPOSAL-UNAVAILABLE"));
        }
        ValueState::Blocked { .. } | ValueState::Unknown { .. } | ValueState::Stale { .. } => {
            return Err(invalid_input("USE3-ADOPTION-PROPOSAL-BLOCKED"));
        }
    };
    if proposal.library_id().as_str() != adoption.library_id
        || proposal.library_version().value() != adoption.library_version
    {
        return Err(invalid_input("USE3-ADOPTION-PROPOSAL-STALE"));
    }
    let quantities = DraftQuantityInputs {
        deliver: ItemQuantity::new(whole(&adoption.deliver_quantity)?),
        planned_spares: ItemQuantity::new(whole(&adoption.planned_spares)?),
        destructive_samples: ItemQuantity::new(whole(&adoption.destructive_samples)?),
    };
    let proposal_rule_id = proposal.rule_id().to_owned();
    let (proposal_major, proposal_minor, proposal_patch) = proposal.rule_version();
    let adopted = DeveloperEstimateProposalApplication
        .adopt(
            proposal,
            quantities,
            DeveloperEstimateProposalReview {
                proposal_values_reviewed: adoption.proposal_values_reviewed,
                coarse_limitations_accepted: adoption.coarse_limitations_accepted,
                actor: ActorId::new(DEVELOPER_ACTOR_ID)
                    .map_err(|_| invalid_input("USE3-ADOPTION-ACTOR"))?,
                recorded_at: recorded_at.clone(),
                reason: adoption.review_reason.clone(),
            },
        )
        .map_err(|_| invalid_input("USE3-ADOPTION-REVIEW"))?;
    let trace = input_trace(&adopted.inputs);
    let (adoption_major, adoption_minor, adoption_patch) =
        DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_VERSION;
    let adoption_summary = DraftEstimateProposalAdoptionSummary {
        settings_revision: adoption.settings_revision,
        library_id: adoption.library_id.clone(),
        library_version: adoption.library_version,
        proposal_rule_id,
        proposal_rule_version: format!("{proposal_major}.{proposal_minor}.{proposal_patch}"),
        adoption_rule_id: DEVELOPER_ESTIMATE_PROPOSAL_ADOPTION_RULE_ID.to_owned(),
        adoption_rule_version: format!("{adoption_major}.{adoption_minor}.{adoption_patch}"),
        reviewed_by: adopted.review.actor.as_str().to_owned(),
        reviewed_at: adopted.review.recorded_at.as_str().to_owned(),
        review_reason: adopted.review.reason.clone(),
        excluded_inputs: vec![
            "non_cutting_time".to_owned(),
            "in_cycle_inspection_time".to_owned(),
            "cut_certificate_and_freight".to_owned(),
            "tooling_fixture_and_outside_processing".to_owned(),
            "administration_and_overhead".to_owned(),
            "risk_and_rework".to_owned(),
        ],
    };
    Ok((adopted.inputs, trace, Some(adoption_summary)))
}

fn input_trace(inputs: &DraftEstimateInputs) -> DraftEstimateInputFields {
    DraftEstimateInputFields {
        stock_volume_mm3: decimal_text(inputs.stock.stock_volume.value()),
        density_kg_per_mm3: decimal_text(inputs.stock.density.value()),
        deliver_quantity: inputs.quantities.deliver.value().to_string(),
        planned_spares: inputs.quantities.planned_spares.value().to_string(),
        destructive_samples: inputs.quantities.destructive_samples.value().to_string(),
        setup_hours: decimal_text(inputs.times.setup_hours),
        programming_hours: decimal_text(inputs.times.programming_hours),
        cutting_hours_per_item: decimal_text(inputs.times.cutting_hours_per_item),
        non_cutting_hours_per_item: decimal_text(inputs.times.non_cutting_hours_per_item),
        load_unload_hours_per_item: decimal_text(inputs.times.load_unload_hours_per_item),
        in_cycle_inspection_hours_per_item: decimal_text(
            inputs.times.in_cycle_inspection_hours_per_item,
        ),
        quality_inspection_hours: decimal_text(inputs.times.quality_inspection_hours),
        purchased_material: money_text(&inputs.material.purchased),
        cut_charge: money_text(&inputs.material.cut),
        material_certificate: money_text(&inputs.material.certificate),
        inbound_freight: money_text(&inputs.material.inbound_freight),
        approved_remnant_credit: money_text(&inputs.material.approved_remnant_credit),
        prove_out: money_text(&inputs.operation.prove_out),
        tooling: money_text(&inputs.operation.tooling),
        consumables: money_text(&inputs.operation.consumables),
        fixture: money_text(&inputs.operation.fixture),
        outside_processing: money_text(&inputs.operation.outside),
        operation_freight: money_text(&inputs.operation.freight),
        nonrecurring_engineering: money_text(&inputs.base.nonrecurring_engineering),
        administration: money_text(&inputs.base.administration),
        overhead: money_text(&inputs.base.overhead),
        accepted_risk_impact: inputs
            .base
            .accepted_risk_impacts
            .first()
            .map_or_else(|| "0".to_owned(), money_text),
        expected_rework: money_text(&inputs.base.expected_rework),
    }
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
    let (card, effective_on) = rate_card(
        fields,
        analysis_id,
        recorded_at,
        DEVELOPER_ACTOR_ID,
        "entered for a session-only developer estimate",
        "explicitly confirmed for this session-only calculation",
    )?;
    DraftRateContext::new(card, effective_on, vec![RateScope::organization()])
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-CONTEXT"))
}

pub(crate) fn rate_card(
    fields: &DeveloperRateInputFields,
    source_record_id: &str,
    recorded_at: &RecordedAt,
    actor: &str,
    entered_reason: &str,
    approval_reason: &str,
) -> Result<(RateCard, EffectiveDate), HostCommandError> {
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
            source_record_id,
            recorded_at,
            actor,
            entered_reason,
            approval_reason,
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
    Ok((card, effective_on))
}

#[allow(clippy::too_many_arguments)]
fn rate_entry(
    id: &str,
    category: CostCategory,
    amount: &str,
    currency: &CurrencyCode,
    version: RateVersion,
    effective_from: &EffectiveDate,
    source_record_id: &str,
    recorded_at: &RecordedAt,
    actor: &str,
    entered_reason: &str,
    approval_reason: &str,
) -> Result<RateEntry, HostCommandError> {
    let entered = RateEvent::new(actor, recorded_at.clone(), entered_reason)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-EVENT"))?;
    let approved = RateEvent::new(actor, recorded_at.clone(), approval_reason)
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-APPROVAL"))?;
    let governance = RateGovernance::new(RateApprovalState::Approved, entered, Some(approved))
        .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-GOVERNANCE"))?;
    let source = SourceRef::new(
        SourceKind::Manual,
        "desktop-shop-rate-input",
        Some(source_record_id.to_owned()),
        Some(recorded_at.clone()),
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-SOURCE"))?;
    RateEntry::new(
        RateId::new(id).map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-ID"))?,
        version,
        category,
        RateComposition::Component,
        RateScope::organization(),
        actor,
        money(amount, currency)?,
        RateBasis::PerHour,
        effective_from.clone(),
        None,
        governance,
        source,
    )
    .map_err(|_| invalid_input("GUI4-ESTIMATE-RATE-ENTRY"))
}

pub(crate) fn pricing_policy(
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
    proposal_adoption: Option<DraftEstimateProposalAdoptionSummary>,
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
        proposal_adoption,
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
pub(crate) fn complete_test_request(
    selection_id: &str,
    analysis_id: &str,
) -> EvaluateDraftEstimateRequest {
    EvaluateDraftEstimateRequest {
        selection_id: selection_id.to_owned(),
        analysis_id: analysis_id.to_owned(),
        review: GeometryReviewInput {
            canonical_units_reviewed: true,
            warnings_reviewed: true,
        },
        inputs: DraftEstimateInputFields {
            stock_volume_mm3: "1480".to_owned(),
            density_kg_per_mm3: "0.00000785".to_owned(),
            deliver_quantity: "1".to_owned(),
            planned_spares: "0".to_owned(),
            destructive_samples: "0".to_owned(),
            setup_hours: "3".to_owned(),
            programming_hours: "2".to_owned(),
            cutting_hours_per_item: "0.42".to_owned(),
            non_cutting_hours_per_item: "0.18".to_owned(),
            load_unload_hours_per_item: "0".to_owned(),
            in_cycle_inspection_hours_per_item: "0".to_owned(),
            quality_inspection_hours: "1".to_owned(),
            purchased_material: "90".to_owned(),
            cut_charge: "5".to_owned(),
            material_certificate: "0".to_owned(),
            inbound_freight: "5".to_owned(),
            approved_remnant_credit: "0".to_owned(),
            prove_out: "20".to_owned(),
            tooling: "25".to_owned(),
            consumables: "10".to_owned(),
            fixture: "20".to_owned(),
            outside_processing: "0".to_owned(),
            operation_freight: "5".to_owned(),
            nonrecurring_engineering: "50".to_owned(),
            administration: "25".to_owned(),
            overhead: "50".to_owned(),
            accepted_risk_impact: "35".to_owned(),
            expected_rework: "0".to_owned(),
        },
        rates: DeveloperRateInputFields {
            confirmed_for_session: true,
            rate_card_id: "developer-card".to_owned(),
            rate_card_version: "1".to_owned(),
            effective_on: "2026-08-09".to_owned(),
            currency: "USD".to_owned(),
            setup_labor_per_hour: "25".to_owned(),
            programming_per_hour: "30".to_owned(),
            run_labor_per_hour: "20".to_owned(),
            machine_per_hour: "40".to_owned(),
            quality_inspection_per_hour: "9".to_owned(),
        },
        pricing: DeveloperPricingInputFields {
            confirmed_for_session: true,
            pricing_policy_id: "developer-pricing".to_owned(),
            pricing_policy_version: "1".to_owned(),
            markup_rate: "0.35".to_owned(),
            optional_price_floor: String::new(),
            optional_minimum_order: String::new(),
            rounding_decimal_places: "2".to_owned(),
        },
        proposal_adoption: None,
    }
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
