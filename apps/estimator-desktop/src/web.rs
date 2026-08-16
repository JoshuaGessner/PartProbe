use leptos::prelude::*;
use partprobe_desktop_contract::{
    AnalysisCancellationAcknowledgement, AnalyzeModelSourceRequest, COMMAND_ANALYZE_MODEL_SOURCE,
    COMMAND_CANCEL_MODEL_ANALYSIS, COMMAND_EVALUATE_DRAFT_ESTIMATE, COMMAND_SELECT_MODEL_SOURCE,
    CancelModelAnalysisRequest, DeveloperPricingInputFields, DeveloperRateInputFields,
    DraftEstimateEvaluation, DraftEstimateEvaluationState, DraftEstimateInputFields,
    EvaluateDraftEstimateRequest, GeometryReviewInput, HostCommandError, ModelAnalysisResult,
    ModelSourceSelection, SelectedModelSource,
};
use wasm_bindgen::prelude::*;

use crate::{
    AnalysisPanelState, DraftEstimatePanelState, GeometryReviewConfirmation, ModelPanelState,
    selected_source_accessible_label,
};

#[wasm_bindgen(inline_js = r#"
export async function invokePartProbe(command, args) {
  return await window.__TAURI__.core.invoke(command, args ?? {});
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = invokePartProbe)]
    async fn invoke_partprobe(command: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(serde::Serialize)]
struct AnalyzeModelSourceArgs {
    request: AnalyzeModelSourceRequest,
}

#[derive(serde::Serialize)]
struct CancelModelAnalysisArgs {
    request: CancelModelAnalysisRequest,
}

#[derive(serde::Serialize)]
struct EvaluateDraftEstimateArgs {
    request: EvaluateDraftEstimateRequest,
}

#[derive(Clone, Debug, Default)]
struct DeveloperEstimateForm {
    geometry_review: GeometryReviewConfirmation,
    stock_volume_mm3: String,
    density_kg_per_mm3: String,
    deliver_quantity: String,
    planned_spares: String,
    destructive_samples: String,
    setup_hours: String,
    programming_hours: String,
    cutting_hours_per_item: String,
    non_cutting_hours_per_item: String,
    load_unload_hours_per_item: String,
    in_cycle_inspection_hours_per_item: String,
    quality_inspection_hours: String,
    purchased_material: String,
    cut_charge: String,
    material_certificate: String,
    inbound_freight: String,
    approved_remnant_credit: String,
    prove_out: String,
    tooling: String,
    consumables: String,
    fixture: String,
    outside_processing: String,
    operation_freight: String,
    nonrecurring_engineering: String,
    administration: String,
    overhead: String,
    accepted_risk_impact: String,
    expected_rework: String,
    rates_confirmed: bool,
    rate_card_id: String,
    rate_card_version: String,
    effective_on: String,
    currency: String,
    setup_labor_per_hour: String,
    programming_per_hour: String,
    run_labor_per_hour: String,
    machine_per_hour: String,
    quality_inspection_per_hour: String,
    pricing_confirmed: bool,
    pricing_policy_id: String,
    pricing_policy_version: String,
    markup_rate: String,
    optional_price_floor: String,
    optional_minimum_order: String,
    rounding_decimal_places: String,
}

impl DeveloperEstimateForm {
    fn new() -> Self {
        Self {
            rate_card_id: "developer-session-rates".to_owned(),
            rate_card_version: "1".to_owned(),
            currency: "USD".to_owned(),
            pricing_policy_id: "developer-session-pricing".to_owned(),
            pricing_policy_version: "1".to_owned(),
            rounding_decimal_places: "2".to_owned(),
            ..Self::default()
        }
    }

    fn request(&self, selection_id: String, analysis_id: String) -> EvaluateDraftEstimateRequest {
        EvaluateDraftEstimateRequest {
            selection_id,
            analysis_id,
            review: GeometryReviewInput {
                canonical_units_reviewed: self.geometry_review.canonical_units_reviewed,
                warnings_reviewed: self.geometry_review.warnings_reviewed,
            },
            inputs: DraftEstimateInputFields {
                stock_volume_mm3: self.stock_volume_mm3.clone(),
                density_kg_per_mm3: self.density_kg_per_mm3.clone(),
                deliver_quantity: self.deliver_quantity.clone(),
                planned_spares: self.planned_spares.clone(),
                destructive_samples: self.destructive_samples.clone(),
                setup_hours: self.setup_hours.clone(),
                programming_hours: self.programming_hours.clone(),
                cutting_hours_per_item: self.cutting_hours_per_item.clone(),
                non_cutting_hours_per_item: self.non_cutting_hours_per_item.clone(),
                load_unload_hours_per_item: self.load_unload_hours_per_item.clone(),
                in_cycle_inspection_hours_per_item: self.in_cycle_inspection_hours_per_item.clone(),
                quality_inspection_hours: self.quality_inspection_hours.clone(),
                purchased_material: self.purchased_material.clone(),
                cut_charge: self.cut_charge.clone(),
                material_certificate: self.material_certificate.clone(),
                inbound_freight: self.inbound_freight.clone(),
                approved_remnant_credit: self.approved_remnant_credit.clone(),
                prove_out: self.prove_out.clone(),
                tooling: self.tooling.clone(),
                consumables: self.consumables.clone(),
                fixture: self.fixture.clone(),
                outside_processing: self.outside_processing.clone(),
                operation_freight: self.operation_freight.clone(),
                nonrecurring_engineering: self.nonrecurring_engineering.clone(),
                administration: self.administration.clone(),
                overhead: self.overhead.clone(),
                accepted_risk_impact: self.accepted_risk_impact.clone(),
                expected_rework: self.expected_rework.clone(),
            },
            rates: DeveloperRateInputFields {
                confirmed_for_session: self.rates_confirmed,
                rate_card_id: self.rate_card_id.clone(),
                rate_card_version: self.rate_card_version.clone(),
                effective_on: self.effective_on.clone(),
                currency: self.currency.clone(),
                setup_labor_per_hour: self.setup_labor_per_hour.clone(),
                programming_per_hour: self.programming_per_hour.clone(),
                run_labor_per_hour: self.run_labor_per_hour.clone(),
                machine_per_hour: self.machine_per_hour.clone(),
                quality_inspection_per_hour: self.quality_inspection_per_hour.clone(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: self.pricing_confirmed,
                pricing_policy_id: self.pricing_policy_id.clone(),
                pricing_policy_version: self.pricing_policy_version.clone(),
                markup_rate: self.markup_rate.clone(),
                optional_price_floor: self.optional_price_floor.clone(),
                optional_minimum_order: self.optional_minimum_order.clone(),
                rounding_decimal_places: self.rounding_decimal_places.clone(),
            },
        }
    }
}

pub fn mount() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (model_state, set_model_state) = signal(ModelPanelState::Empty);
    let (analysis_state, set_analysis_state) = signal(AnalysisPanelState::NotStarted);
    let (estimate_state, set_estimate_state) = signal(DraftEstimatePanelState::NotReady);
    let (is_selecting, set_is_selecting) = signal(false);

    let select_model = move |_| {
        set_is_selecting.set(true);
        leptos::task::spawn_local(async move {
            let next = invoke_partprobe(COMMAND_SELECT_MODEL_SOURCE, JsValue::UNDEFINED)
                .await
                .ok()
                .and_then(|value| serde_wasm_bindgen::from_value(value).ok());

            match next {
                Some(selection @ ModelSourceSelection::Selected { .. }) => {
                    set_model_state.update(|state| state.apply_selection(selection));
                    set_analysis_state.set(AnalysisPanelState::NotStarted);
                    set_estimate_state.set(DraftEstimatePanelState::NotReady);
                }
                Some(ModelSourceSelection::Cancelled) => {}
                None => set_model_state.set(ModelPanelState::Failed),
            }
            set_is_selecting.set(false);
        });
    };

    let analyze_model = move |_| {
        let Some(selection_id) = model_state
            .get_untracked()
            .selected_source()
            .map(|source| source.selection_id.clone())
        else {
            return;
        };
        set_estimate_state.set(DraftEstimatePanelState::NotReady);
        set_analysis_state.set(AnalysisPanelState::Running);
        leptos::task::spawn_local(async move {
            let request = AnalyzeModelSourceRequest { selection_id };
            let args = serde_wasm_bindgen::to_value(&AnalyzeModelSourceArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_ANALYZE_MODEL_SOURCE, args).await {
                Ok(value) => {
                    let result = serde_wasm_bindgen::from_value::<ModelAnalysisResult>(value)
                        .map(Box::new)
                        .map(AnalysisPanelState::Available)
                        .unwrap_or_else(|_| {
                            AnalysisPanelState::Failed(HostCommandError::analysis_failed(
                                "GUI4-ANALYSIS-RESULT",
                            ))
                        });
                    set_analysis_state.set(result);
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::analysis_failed("GUI4-ANALYSIS-INVOKE")
                        });
                    set_analysis_state.set(AnalysisPanelState::from_host_error(error));
                }
            }
        });
    };

    let cancel_analysis = move |_| {
        let Some(selection_id) = model_state
            .get_untracked()
            .selected_source()
            .map(|source| source.selection_id.clone())
        else {
            return;
        };
        if !matches!(analysis_state.get_untracked(), AnalysisPanelState::Running) {
            return;
        }
        set_analysis_state.set(AnalysisPanelState::Cancelling);
        leptos::task::spawn_local(async move {
            let request = CancelModelAnalysisRequest { selection_id };
            let args = serde_wasm_bindgen::to_value(&CancelModelAnalysisArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_CANCEL_MODEL_ANALYSIS, args).await {
                Ok(value) => {
                    let acknowledged = serde_wasm_bindgen::from_value::<
                        AnalysisCancellationAcknowledgement,
                    >(value)
                    .is_ok_and(|acknowledgement| acknowledgement.cancellation_requested);
                    if !acknowledged {
                        set_analysis_state.set(AnalysisPanelState::Running);
                    }
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::analysis_failed("GUI4-CANCELLATION-INVOKE")
                        });
                    set_analysis_state.set(AnalysisPanelState::Failed(error));
                }
            }
        });
    };

    view! {
        <a class="skip-link" href="#workspace">"Skip to workspace"</a>
        <header class="app-header">
            <div>
                <p class="eyebrow">"PARTPROBE / DEVELOPER ALPHA"</p>
                <h1>"Estimate workspace"</h1>
            </div>
            <div class="session-state" aria-label="Application state">
                <span class="state-dot" aria-hidden="true"></span>
                <span>"Session only · not saved"</span>
            </div>
        </header>

        <main id="workspace" class="workspace">
            <section class="model-panel" aria-labelledby="model-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"01 / SOURCE"</p>
                        <h2 id="model-heading">"Model intake"</h2>
                    </div>
                    <span class="status-chip">"STEP only"</span>
                </div>

                <div class="drop-zone">
                    <div class="model-mark" aria-hidden="true">"P"</div>
                    <p class="status-heading" aria-live="polite">
                        {move || if is_selecting.get() {
                            "Waiting for model selection"
                        } else {
                            model_state.get().status_heading()
                        }}
                    </p>
                    <p class="status-detail">
                        {move || if is_selecting.get() {
                            "PartProbe is showing the native file picker."
                        } else {
                            model_state.get().status_detail()
                        }}
                    </p>
                    <button
                        type="button"
                        class="secondary-action"
                        disabled=move || is_selecting.get()
                        on:click=select_model
                    >
                        {move || if is_selecting.get() { "Picker open" } else { "Choose STEP model" }}
                    </button>
                </div>

                <SelectedSource state=model_state />
                <div class="analysis-actions">
                    <button
                        type="button"
                        class="primary-action analyze-action"
                        disabled=move || {
                            model_state.get().selected_source().is_none()
                                || matches!(
                                    analysis_state.get(),
                                    AnalysisPanelState::Running | AnalysisPanelState::Cancelling
                                )
                        }
                        on:click=analyze_model
                    >
                        {move || match analysis_state.get() {
                            AnalysisPanelState::Running => "Analyzing model",
                            AnalysisPanelState::Cancelling => "Cancelling analysis",
                            _ => "Analyze provisional geometry",
                        }}
                    </button>
                    <button
                        type="button"
                        class="secondary-action cancel-action"
                        disabled=move || !matches!(analysis_state.get(), AnalysisPanelState::Running)
                        on:click=cancel_analysis
                    >
                        "Cancel analysis"
                    </button>
                </div>
                <AnalysisEvidence state=analysis_state />
            </section>

            <EstimateWorkspace
                analysis_state
                estimate_state
                set_estimate_state
            />
        </main>
    }
}

#[component]
fn SelectedSource(state: ReadSignal<ModelPanelState>) -> impl IntoView {
    move || match state.get() {
        ModelPanelState::Selected(source) => view! { <SourceSummary source /> }.into_any(),
        _ => ().into_any(),
    }
}

#[component]
fn SourceSummary(source: SelectedModelSource) -> impl IntoView {
    let accessible_label = selected_source_accessible_label(&source.display_name);
    view! {
        <dl class="source-summary" aria-label=accessible_label>
            <div><dt>"File"</dt><dd>{source.display_name}</dd></div>
            <div><dt>"Format"</dt><dd>"STEP"</dd></div>
            <div><dt>"Authority"</dt><dd>"Native session token"</dd></div>
            <div><dt>"Storage"</dt><dd>"Session only"</dd></div>
        </dl>
    }
}

#[component]
fn AnalysisEvidence(state: ReadSignal<AnalysisPanelState>) -> impl IntoView {
    move || match state.get() {
        AnalysisPanelState::Available(result) => {
            let geometry = result.geometry;
            let centroid = geometry.center_of_mass_mm.join(", ");
            let warning_count = result
                .stages
                .iter()
                .map(|stage| stage.warning_codes.len())
                .sum::<usize>();
            view! {
                <section class="analysis-evidence" aria-labelledby="analysis-evidence-heading">
                    <div class="analysis-evidence-heading">
                        <div>
                            <p class="section-index">"PROVISIONAL / SESSION ONLY"</p>
                            <h3 id="analysis-evidence-heading">"Geometry evidence"</h3>
                        </div>
                        <span class="status-chip">"Review required"</span>
                    </div>
                    <dl class="geometry-facts">
                        <div><dt>"Surface area"</dt><dd>{geometry.surface_area_mm2}" mm²"</dd></div>
                        <div><dt>"Enclosed volume"</dt><dd>{geometry.enclosed_volume_mm3}" mm³"</dd></div>
                        <div><dt>"Centroid"</dt><dd>{centroid}" mm"</dd></div>
                        <div><dt>"Solid bodies"</dt><dd>{geometry.solid_body_count}</dd></div>
                        <div><dt>"Canonical units"</dt><dd>"Millimeter"</dd></div>
                        <div><dt>"Warnings"</dt><dd>{warning_count}</dd></div>
                        <div><dt>"Engine"</dt><dd>{geometry.geometry_engine}</dd></div>
                        <div><dt>"Analysis ID"</dt><dd>{result.analysis_id}</dd></div>
                    </dl>
                    <p class="evidence-note">
                        "These exact-B-rep measurements are provisional spike evidence. They are not a supported importer result or an approved estimate."
                    </p>
                </section>
            }
            .into_any()
        }
        AnalysisPanelState::Failed(error) => view! {
            <section class="analysis-error" role="alert">
                <p class="blocked-title">"Analysis failed safely"</p>
                <p>{error.message}</p>
                <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::Running => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Isolated worker analysis in progress"</p>
                <p>"The selected source remains session-only. Cancellation is available above."</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::Cancelling => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Cancellation requested"</p>
                <p>"PartProbe is waiting for acknowledgement or bounded forced cleanup."</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::Cancelled => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Analysis cancelled"</p>
                <p>"The selected source remains available. Retry analysis when you are ready."</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::NotStarted => ().into_any(),
    }
}

#[component]
fn EstimateWorkspace(
    analysis_state: ReadSignal<AnalysisPanelState>,
    estimate_state: ReadSignal<DraftEstimatePanelState>,
    set_estimate_state: WriteSignal<DraftEstimatePanelState>,
) -> impl IntoView {
    let form = RwSignal::new(DeveloperEstimateForm::new());
    Effect::new(move |_| {
        let _ = analysis_state.get();
        form.update(|form| form.geometry_review.clear());
    });
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some((selection_id, analysis_id)) = (match analysis_state.get_untracked() {
            AnalysisPanelState::Available(result) => {
                Some((result.selection_id.clone(), result.analysis_id.clone()))
            }
            _ => None,
        }) else {
            return;
        };
        let request = form.with(|form| form.request(selection_id, analysis_id));
        set_estimate_state.set(DraftEstimatePanelState::Evaluating);
        leptos::task::spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&EvaluateDraftEstimateArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_EVALUATE_DRAFT_ESTIMATE, args).await {
                Ok(value) => {
                    let result = serde_wasm_bindgen::from_value::<DraftEstimateEvaluation>(value)
                        .map(Box::new)
                        .map(DraftEstimatePanelState::Evaluated)
                        .unwrap_or_else(|_| {
                            DraftEstimatePanelState::Failed(
                                HostCommandError::invalid_estimate_input("GUI4-ESTIMATE-RESULT"),
                            )
                        });
                    set_estimate_state.set(result);
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::invalid_estimate_input("GUI4-ESTIMATE-INVOKE")
                        });
                    set_estimate_state.set(DraftEstimatePanelState::Failed(error));
                }
            }
        });
    };

    view! {
        <aside class="estimate-panel" aria-labelledby="estimate-heading">
            <div class="panel-heading">
                <div>
                    <p class="section-index">"02 / ESTIMATE"</p>
                    <h2 id="estimate-heading">"Draft estimate"</h2>
                </div>
                <span class=move || match estimate_state.get() {
                    DraftEstimatePanelState::Evaluated(ref evaluation)
                        if evaluation.state == DraftEstimateEvaluationState::Available =>
                            "status-chip available",
                    _ => "status-chip blocked",
                }>
                    {move || match estimate_state.get() {
                        DraftEstimatePanelState::Evaluated(ref evaluation)
                            if evaluation.state == DraftEstimateEvaluationState::Available => "Available",
                        DraftEstimatePanelState::Evaluating => "Evaluating",
                        _ => "Unavailable",
                    }}
                </span>
            </div>

            {move || if matches!(analysis_state.get(), AnalysisPanelState::Available(_)) {
                view! {
                    <form class="estimate-form" on:submit=submit>
                        <fieldset>
                            <legend>"Geometry review"</legend>
                            <ReviewCheckbox
                                form
                                label="I reviewed the canonical millimeter interpretation."
                                read=|form| form.geometry_review.canonical_units_reviewed
                                write=|form, value| form.geometry_review.canonical_units_reviewed = value
                            />
                            <ReviewCheckbox
                                form
                                label="I reviewed the complete warning set, including an empty set."
                                read=|form| form.geometry_review.warnings_reviewed
                                write=|form, value| form.geometry_review.warnings_reviewed = value
                            />
                        </fieldset>

                        <fieldset>
                            <legend>"Stock and quantity"</legend>
                            <div class="form-grid">
                                <ExactInput form label="Stock volume" unit="mm³" read=|f| &f.stock_volume_mm3 write=|f, v| f.stock_volume_mm3 = v />
                                <ExactInput form label="Material density" unit="kg/mm³" read=|f| &f.density_kg_per_mm3 write=|f, v| f.density_kg_per_mm3 = v />
                                <ExactInput form label="Deliver quantity" unit="items" read=|f| &f.deliver_quantity write=|f, v| f.deliver_quantity = v />
                                <ExactInput form label="Planned spares" unit="items" read=|f| &f.planned_spares write=|f, v| f.planned_spares = v />
                                <ExactInput form label="Destructive samples" unit="items" read=|f| &f.destructive_samples write=|f, v| f.destructive_samples = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Time inputs"</legend>
                            <div class="form-grid">
                                <ExactInput form label="Setup" unit="hr/lot" read=|f| &f.setup_hours write=|f, v| f.setup_hours = v />
                                <ExactInput form label="Programming" unit="hr/lot" read=|f| &f.programming_hours write=|f, v| f.programming_hours = v />
                                <ExactInput form label="Cutting" unit="hr/item" read=|f| &f.cutting_hours_per_item write=|f, v| f.cutting_hours_per_item = v />
                                <ExactInput form label="Non-cutting" unit="hr/item" read=|f| &f.non_cutting_hours_per_item write=|f, v| f.non_cutting_hours_per_item = v />
                                <ExactInput form label="Load / unload" unit="hr/item" read=|f| &f.load_unload_hours_per_item write=|f, v| f.load_unload_hours_per_item = v />
                                <ExactInput form label="In-cycle inspection" unit="hr/item" read=|f| &f.in_cycle_inspection_hours_per_item write=|f, v| f.in_cycle_inspection_hours_per_item = v />
                                <ExactInput form label="Quality inspection" unit="hr/lot" read=|f| &f.quality_inspection_hours write=|f, v| f.quality_inspection_hours = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Material and operation costs"</legend>
                            <p class="fieldset-note">"Enter every monetary value in the rate-card currency; use an explicit 0 where applicable."</p>
                            <div class="form-grid">
                                <ExactInput form label="Purchased material" unit="currency" read=|f| &f.purchased_material write=|f, v| f.purchased_material = v />
                                <ExactInput form label="Cut charge" unit="currency" read=|f| &f.cut_charge write=|f, v| f.cut_charge = v />
                                <ExactInput form label="Material certificate" unit="currency" read=|f| &f.material_certificate write=|f, v| f.material_certificate = v />
                                <ExactInput form label="Inbound freight" unit="currency" read=|f| &f.inbound_freight write=|f, v| f.inbound_freight = v />
                                <ExactInput form label="Approved remnant credit" unit="currency" read=|f| &f.approved_remnant_credit write=|f, v| f.approved_remnant_credit = v />
                                <ExactInput form label="Prove-out" unit="currency" read=|f| &f.prove_out write=|f, v| f.prove_out = v />
                                <ExactInput form label="Tooling" unit="currency" read=|f| &f.tooling write=|f, v| f.tooling = v />
                                <ExactInput form label="Consumables" unit="currency" read=|f| &f.consumables write=|f, v| f.consumables = v />
                                <ExactInput form label="Fixture" unit="currency" read=|f| &f.fixture write=|f, v| f.fixture = v />
                                <ExactInput form label="Outside processing" unit="currency" read=|f| &f.outside_processing write=|f, v| f.outside_processing = v />
                                <ExactInput form label="Operation freight" unit="currency" read=|f| &f.operation_freight write=|f, v| f.operation_freight = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Base cost and risk"</legend>
                            <div class="form-grid">
                                <ExactInput form label="Nonrecurring engineering" unit="currency" read=|f| &f.nonrecurring_engineering write=|f, v| f.nonrecurring_engineering = v />
                                <ExactInput form label="Administration" unit="currency" read=|f| &f.administration write=|f, v| f.administration = v />
                                <ExactInput form label="Overhead" unit="currency" read=|f| &f.overhead write=|f, v| f.overhead = v />
                                <ExactInput form label="Accepted risk impact" unit="currency" read=|f| &f.accepted_risk_impact write=|f, v| f.accepted_risk_impact = v />
                                <ExactInput form label="Expected rework" unit="currency" read=|f| &f.expected_rework write=|f, v| f.expected_rework = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Session rate card"</legend>
                            <p class="fieldset-note">"These user-entered values are ephemeral and are not installed as production rates."</p>
                            <div class="form-grid">
                                <ExactInput form label="Rate-card ID" unit="ID" read=|f| &f.rate_card_id write=|f, v| f.rate_card_id = v />
                                <ExactInput form label="Rate-card version" unit="version" read=|f| &f.rate_card_version write=|f, v| f.rate_card_version = v />
                                <ExactInput form label="Effective on" unit="YYYY-MM-DD" read=|f| &f.effective_on write=|f, v| f.effective_on = v />
                                <ExactInput form label="Currency" unit="ISO code" read=|f| &f.currency write=|f, v| f.currency = v />
                                <ExactInput form label="Setup labor" unit="/hr" read=|f| &f.setup_labor_per_hour write=|f, v| f.setup_labor_per_hour = v />
                                <ExactInput form label="Programming" unit="/hr" read=|f| &f.programming_per_hour write=|f, v| f.programming_per_hour = v />
                                <ExactInput form label="Run labor" unit="/hr" read=|f| &f.run_labor_per_hour write=|f, v| f.run_labor_per_hour = v />
                                <ExactInput form label="Machine" unit="/hr" read=|f| &f.machine_per_hour write=|f, v| f.machine_per_hour = v />
                                <ExactInput form label="Quality inspection" unit="/hr" read=|f| &f.quality_inspection_per_hour write=|f, v| f.quality_inspection_per_hour = v />
                            </div>
                            <ReviewCheckbox form label="I confirm these five rates for this session-only developer calculation." read=|f| f.rates_confirmed write=|f, v| f.rates_confirmed = v />
                        </fieldset>

                        <fieldset>
                            <legend>"Session pricing policy"</legend>
                            <div class="form-grid">
                                <ExactInput form label="Pricing-policy ID" unit="ID" read=|f| &f.pricing_policy_id write=|f, v| f.pricing_policy_id = v />
                                <ExactInput form label="Policy version" unit="version" read=|f| &f.pricing_policy_version write=|f, v| f.pricing_policy_version = v />
                                <ExactInput form label="Markup rate" unit="decimal" read=|f| &f.markup_rate write=|f, v| f.markup_rate = v />
                                <OptionalInput form label="Price floor" unit="currency" read=|f| &f.optional_price_floor write=|f, v| f.optional_price_floor = v />
                                <OptionalInput form label="Minimum order" unit="currency" read=|f| &f.optional_minimum_order write=|f, v| f.optional_minimum_order = v />
                                <ExactInput form label="Rounding places" unit="decimals" read=|f| &f.rounding_decimal_places write=|f, v| f.rounding_decimal_places = v />
                            </div>
                            <ReviewCheckbox form label="I confirm this pricing policy for this session-only developer calculation." read=|f| f.pricing_confirmed write=|f, v| f.pricing_confirmed = v />
                        </fieldset>

                        <button
                            type="submit"
                            class="primary-action calculate-action"
                            disabled=move || matches!(estimate_state.get(), DraftEstimatePanelState::Evaluating)
                        >
                            {move || if matches!(estimate_state.get(), DraftEstimatePanelState::Evaluating) {
                                "Evaluating estimate"
                            } else {
                                "Calculate deterministic draft"
                            }}
                        </button>
                    </form>
                    <EstimateResult state=estimate_state />
                }
                .into_any()
            } else {
                view! {
                    <div class="blocked-state" aria-live="polite">
                        <p class="blocked-title">"Provisional geometry required"</p>
                        <p>"Analyze a selected STEP model before reviewing units or entering estimate inputs."</p>
                        <dl>
                            <div><dt>"Geometry"</dt><dd>"Not available"</dd></div>
                            <div><dt>"Units"</dt><dd>"Not reviewed"</dd></div>
                            <div><dt>"Rate basis"</dt><dd>"Not selected"</dd></div>
                            <div><dt>"Selling price"</dt><dd>"Unavailable"</dd></div>
                        </dl>
                    </div>
                }
                .into_any()
            }}
        </aside>
    }
}

#[component]
fn ExactInput(
    form: RwSignal<DeveloperEstimateForm>,
    label: &'static str,
    unit: &'static str,
    read: fn(&DeveloperEstimateForm) -> &String,
    write: fn(&mut DeveloperEstimateForm, String),
) -> impl IntoView {
    view! {
        <label class="form-field">
            <span>{label}</span>
            <span class="input-with-unit">
                <input
                    type="text"
                    inputmode="decimal"
                    required
                    prop:value=move || form.with(|form| read(form).clone())
                    on:input=move |event| write(&mut form.write(), event_target_value(&event))
                />
                <span>{unit}</span>
            </span>
        </label>
    }
}

#[component]
fn OptionalInput(
    form: RwSignal<DeveloperEstimateForm>,
    label: &'static str,
    unit: &'static str,
    read: fn(&DeveloperEstimateForm) -> &String,
    write: fn(&mut DeveloperEstimateForm, String),
) -> impl IntoView {
    view! {
        <label class="form-field">
            <span>{label}" (optional)"</span>
            <span class="input-with-unit">
                <input
                    type="text"
                    inputmode="decimal"
                    prop:value=move || form.with(|form| read(form).clone())
                    on:input=move |event| write(&mut form.write(), event_target_value(&event))
                />
                <span>{unit}</span>
            </span>
        </label>
    }
}

#[component]
fn ReviewCheckbox(
    form: RwSignal<DeveloperEstimateForm>,
    label: &'static str,
    read: fn(&DeveloperEstimateForm) -> bool,
    write: fn(&mut DeveloperEstimateForm, bool),
) -> impl IntoView {
    view! {
        <label class="review-check">
            <input
                type="checkbox"
                required
                prop:checked=move || form.with(read)
                on:change=move |event| write(&mut form.write(), event_target_checked(&event))
            />
            <span>{label}</span>
        </label>
    }
}

#[component]
fn EstimateResult(state: ReadSignal<DraftEstimatePanelState>) -> impl IntoView {
    move || {
        match state.get() {
        DraftEstimatePanelState::Evaluated(evaluation) => match evaluation.result {
            Some(result) if evaluation.state == DraftEstimateEvaluationState::Available => {
                let currency = result.currency.clone();
                let rules = result.calculation_rule_ids.join(", ");
                view! {
                    <section class="estimate-result" aria-labelledby="estimate-result-heading" aria-live="polite">
                        <div class="result-total">
                            <div>
                                <p class="section-index">"DETERMINISTIC / SESSION ONLY"</p>
                                <h3 id="estimate-result-heading">"Draft selling price"</h3>
                            </div>
                            <p><span>{currency.clone()}</span> {result.rounded_selling_price}</p>
                        </div>
                        <dl class="cost-trace">
                            <div><dt>"Material"</dt><dd>{currency.clone()}" "{result.material_cost}</dd></div>
                            <div><dt>"Operation"</dt><dd>{currency.clone()}" "{result.operation_cost}</dd></div>
                            <div><dt>"Base internal"</dt><dd>{currency.clone()}" "{result.base_internal_cost}</dd></div>
                            <div><dt>"Risk reserve"</dt><dd>{currency.clone()}" "{result.risk_reserve}</dd></div>
                            <div><dt>"Total internal"</dt><dd>{currency.clone()}" "{result.total_internal_cost}</dd></div>
                            <div><dt>"Formula price"</dt><dd>{currency.clone()}" "{result.formula_price}</dd></div>
                        </dl>
                        <details>
                            <summary>"Calculation and rate trace"</summary>
                            <p>"Rules: " {rules}</p>
                            <p>"Pricing: " {result.pricing_policy.method}" "{result.pricing_policy.method_rate}
                                ", policy "{result.pricing_policy.policy_id}" v"{result.pricing_policy.policy_version}</p>
                            <ul class="rate-trace">
                                {result.resolved_rates.into_iter().map(|rate| view! {
                                    <li>
                                        <strong>{rate.category}</strong>
                                        ": "{currency.clone()}" "{rate.amount_per_hour}"/hr · "
                                        {rate.card_id}" v"{rate.card_version}" · "{rate.effective_on}
                                    </li>
                                }).collect_view()}
                            </ul>
                        </details>
                        <p class="evidence-note">"This draft is ephemeral, unapproved, and not a customer quote."</p>
                    </section>
                }
                .into_any()
            }
            _ => view! {
                <section class="analysis-error" role="alert">
                    <p class="blocked-title">"Estimate blocked"</p>
                    <p>{evaluation.reason.unwrap_or_else(|| "The deterministic result is unavailable.".to_owned())}</p>
                </section>
            }
            .into_any(),
        },
        DraftEstimatePanelState::Failed(error) => view! {
            <section class="analysis-error" role="alert">
                <p class="blocked-title">"Estimate input rejected safely"</p>
                <p>{error.message}</p>
                <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
            </section>
        }
        .into_any(),
        DraftEstimatePanelState::Evaluating => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Deterministic evaluation in progress"</p>
                <p>"The native application service is validating the complete input and policy set."</p>
            </section>
        }
        .into_any(),
        DraftEstimatePanelState::NotReady => ().into_any(),
    }
    }
}
