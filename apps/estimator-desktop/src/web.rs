use leptos::prelude::*;
use partprobe_desktop_contract::{
    AnalysisCancellationAcknowledgement, AnalyzeModelSourceRequest, COMMAND_ANALYZE_MODEL_SOURCE,
    COMMAND_CANCEL_MODEL_ANALYSIS, COMMAND_DESKTOP_CONTRACT, COMMAND_EVALUATE_DRAFT_ESTIMATE,
    COMMAND_LOAD_SHOP_SETTINGS, COMMAND_PREPARE_DRAFT_ESTIMATE_PROPOSAL,
    COMMAND_SAVE_SHOP_SETTINGS, COMMAND_SELECT_MODEL_SOURCE, COMMAND_SET_MODEL_VIEWER_WORKSPACE,
    CancelModelAnalysisRequest, DesktopContract, DeveloperPricingInputFields,
    DeveloperRateInputFields, DraftEstimateEvaluation, DraftEstimateEvaluationState,
    DraftEstimateInputFields, DraftEstimateProposalAdoptionInput, DraftEstimateProposalEvaluation,
    DraftEstimateProposalSummary, EvaluateDraftEstimateRequest, GeometryConfidenceLevel,
    GeometryReviewInput, HostCommandError, MeshMeasurementBasis, MeshSelfIntersectionState,
    MeshTopologyIdentity, ModelAnalysisResult, ModelSourceSelection, ModelViewerAvailability,
    ModelViewerWorkspaceMode, ModelViewerWorkspaceResult, PrepareDraftEstimateProposalRequest,
    ProvisionalGeometryFacts, SaveShopSettingsRequest, SelectedModelSource,
    SetModelViewerWorkspaceRequest, ShopResourceCatalogSnapshot, ShopResourceInputFields,
    ShopResourceRecordState, ShopResourceSelectionState, ShopSettingsSnapshot, ShopSettingsState,
    StlEncoding, UnitResolution,
};
use wasm_bindgen::prelude::*;

use crate::{
    AnalysisPanelState, DraftEstimatePanelState, GeometryReviewConfirmation, ModelPanelState,
    analysis_supports_draft_estimate, length_unit_label,
    provisional_analysis_failure_accessible_label, provisional_geometry_accessible_label,
    selected_source_accessible_label, source_format_label,
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

#[derive(serde::Serialize)]
struct PrepareDraftEstimateProposalArgs {
    request: PrepareDraftEstimateProposalRequest,
}

#[derive(serde::Serialize)]
struct SaveShopSettingsArgs {
    request: SaveShopSettingsRequest,
}

#[derive(serde::Serialize)]
struct SetModelViewerWorkspaceArgs {
    request: SetModelViewerWorkspaceRequest,
}

#[derive(Clone, Debug, Default)]
enum SettingsPanelState {
    #[default]
    Loading,
    NotConfigured,
    Available(Box<ShopSettingsSnapshot>),
    Saving,
    Failed {
        error: HostCommandError,
        revision: Option<u32>,
    },
}

#[derive(Clone, Debug, Default)]
enum ProposalPanelState {
    #[default]
    NotRequested,
    Loading,
    Evaluated(Box<DraftEstimateProposalEvaluation>),
    Failed(HostCommandError),
}

impl ProposalPanelState {
    fn is_available(&self) -> bool {
        matches!(
            self,
            Self::Evaluated(evaluation)
                if matches!(
                    evaluation.as_ref(),
                    DraftEstimateProposalEvaluation::Available { .. }
                )
        )
    }
}

impl SettingsPanelState {
    fn revision(&self) -> Option<u32> {
        match self {
            Self::Available(settings) => Some(settings.revision),
            Self::Failed { revision, .. } => *revision,
            Self::Loading | Self::NotConfigured | Self::Saving => None,
        }
    }

    fn resource_catalog(&self) -> Option<&ShopResourceCatalogSnapshot> {
        match self {
            Self::Available(settings) => settings.resource_catalog.as_ref(),
            Self::Loading | Self::NotConfigured | Self::Saving | Self::Failed { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum WorkspaceView {
    #[default]
    Estimate,
    Settings,
    ModelAndStock,
}

impl WorkspaceView {
    const fn title(self) -> &'static str {
        match self {
            Self::Estimate => "Estimate workspace",
            Self::Settings => "Shop settings",
            Self::ModelAndStock => "Model & stock",
        }
    }
}

#[derive(Clone, Debug, Default)]
enum ModelViewerPanelState {
    #[default]
    Loading,
    Unavailable,
    Ready,
    Switching,
    Active(ModelViewerWorkspaceResult),
    Failed(HostCommandError),
}

impl ModelViewerPanelState {
    fn is_available(&self) -> bool {
        matches!(
            self,
            Self::Ready | Self::Switching | Self::Active(_) | Self::Failed(_)
        )
    }

    fn is_switching(&self) -> bool {
        matches!(self, Self::Switching)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsSection {
    #[default]
    RatesAndPricing,
    Resources,
}

#[derive(Clone, Debug, Default)]
struct DeveloperEstimateForm {
    geometry_review: GeometryReviewConfirmation,
    proposal_values_reviewed: bool,
    coarse_limitations_accepted: bool,
    proposal_review_reason: String,
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
    settings_changed_by: String,
    settings_change_reason: String,
    resources: DeveloperResourceForm,
}

#[derive(Clone, Debug, Default)]
struct DeveloperResourceForm {
    enabled: bool,
    confirmed_for_draft: bool,
    library_id: String,
    library_version: String,
    material_id: String,
    material_version: String,
    material_family: String,
    material_grade: String,
    optional_material_specification: String,
    optional_material_condition: String,
    density_kg_per_m3: String,
    material_source: String,
    offer_id: String,
    offer_version: String,
    supplier: String,
    material_price_per_kg: String,
    offer_effective_on: String,
    offer_source: String,
    stock_profile_id: String,
    stock_profile_version: String,
    stock_form: String,
    stock_allowance_x_mm: String,
    stock_allowance_y_mm: String,
    stock_allowance_z_mm: String,
    stock_source: String,
    machine_id: String,
    machine_version: String,
    machine_name: String,
    process_class: String,
    machine_envelope_x_mm: String,
    machine_envelope_y_mm: String,
    machine_envelope_z_mm: String,
    machine_source: String,
    runtime_profile_id: String,
    runtime_profile_version: String,
    removal_rate_mm3_per_minute: String,
    setup_minutes: String,
    programming_minutes: String,
    load_unload_minutes: String,
    inspection_minutes: String,
    runtime_source: String,
}

impl DeveloperResourceForm {
    fn new() -> Self {
        Self {
            library_version: "1".to_owned(),
            material_version: "1".to_owned(),
            offer_version: "1".to_owned(),
            stock_profile_version: "1".to_owned(),
            stock_form: "rectangular".to_owned(),
            machine_version: "1".to_owned(),
            process_class: "milling".to_owned(),
            runtime_profile_version: "1".to_owned(),
            ..Self::default()
        }
    }

    fn complete(&self) -> bool {
        !self.enabled
            || self.confirmed_for_draft
                && [
                    &self.library_id,
                    &self.library_version,
                    &self.material_id,
                    &self.material_version,
                    &self.material_family,
                    &self.material_grade,
                    &self.density_kg_per_m3,
                    &self.material_source,
                    &self.offer_id,
                    &self.offer_version,
                    &self.supplier,
                    &self.material_price_per_kg,
                    &self.offer_effective_on,
                    &self.offer_source,
                    &self.stock_profile_id,
                    &self.stock_profile_version,
                    &self.stock_form,
                    &self.stock_allowance_x_mm,
                    &self.stock_allowance_y_mm,
                    &self.stock_allowance_z_mm,
                    &self.stock_source,
                    &self.machine_id,
                    &self.machine_version,
                    &self.machine_name,
                    &self.process_class,
                    &self.machine_envelope_x_mm,
                    &self.machine_envelope_y_mm,
                    &self.machine_envelope_z_mm,
                    &self.machine_source,
                    &self.runtime_profile_id,
                    &self.runtime_profile_version,
                    &self.removal_rate_mm3_per_minute,
                    &self.setup_minutes,
                    &self.programming_minutes,
                    &self.load_unload_minutes,
                    &self.inspection_minutes,
                    &self.runtime_source,
                ]
                .into_iter()
                .all(|value| !value.trim().is_empty())
    }

    fn fields(&self) -> Option<ShopResourceInputFields> {
        self.enabled.then(|| ShopResourceInputFields {
            confirmed_for_draft: self.confirmed_for_draft,
            library_id: self.library_id.clone(),
            library_version: self.library_version.clone(),
            material_id: self.material_id.clone(),
            material_version: self.material_version.clone(),
            material_family: self.material_family.clone(),
            material_grade: self.material_grade.clone(),
            optional_material_specification: self.optional_material_specification.clone(),
            optional_material_condition: self.optional_material_condition.clone(),
            density_kg_per_m3: self.density_kg_per_m3.clone(),
            material_source: self.material_source.clone(),
            offer_id: self.offer_id.clone(),
            offer_version: self.offer_version.clone(),
            supplier: self.supplier.clone(),
            material_price_per_kg: self.material_price_per_kg.clone(),
            offer_effective_on: self.offer_effective_on.clone(),
            offer_source: self.offer_source.clone(),
            stock_profile_id: self.stock_profile_id.clone(),
            stock_profile_version: self.stock_profile_version.clone(),
            stock_form: self.stock_form.clone(),
            stock_allowance_x_mm: self.stock_allowance_x_mm.clone(),
            stock_allowance_y_mm: self.stock_allowance_y_mm.clone(),
            stock_allowance_z_mm: self.stock_allowance_z_mm.clone(),
            stock_source: self.stock_source.clone(),
            machine_id: self.machine_id.clone(),
            machine_version: self.machine_version.clone(),
            machine_name: self.machine_name.clone(),
            process_class: self.process_class.clone(),
            machine_envelope_x_mm: self.machine_envelope_x_mm.clone(),
            machine_envelope_y_mm: self.machine_envelope_y_mm.clone(),
            machine_envelope_z_mm: self.machine_envelope_z_mm.clone(),
            machine_source: self.machine_source.clone(),
            runtime_profile_id: self.runtime_profile_id.clone(),
            runtime_profile_version: self.runtime_profile_version.clone(),
            removal_rate_mm3_per_minute: self.removal_rate_mm3_per_minute.clone(),
            setup_minutes: self.setup_minutes.clone(),
            programming_minutes: self.programming_minutes.clone(),
            load_unload_minutes: self.load_unload_minutes.clone(),
            inspection_minutes: self.inspection_minutes.clone(),
            runtime_source: self.runtime_source.clone(),
        })
    }

    fn apply(&mut self, fields: Option<&ShopResourceInputFields>) {
        let Some(fields) = fields else {
            *self = Self::new();
            return;
        };
        self.enabled = true;
        self.confirmed_for_draft = false;
        self.library_id.clone_from(&fields.library_id);
        self.library_version.clone_from(&fields.library_version);
        self.material_id.clone_from(&fields.material_id);
        self.material_version.clone_from(&fields.material_version);
        self.material_family.clone_from(&fields.material_family);
        self.material_grade.clone_from(&fields.material_grade);
        self.optional_material_specification
            .clone_from(&fields.optional_material_specification);
        self.optional_material_condition
            .clone_from(&fields.optional_material_condition);
        self.density_kg_per_m3.clone_from(&fields.density_kg_per_m3);
        self.material_source.clone_from(&fields.material_source);
        self.offer_id.clone_from(&fields.offer_id);
        self.offer_version.clone_from(&fields.offer_version);
        self.supplier.clone_from(&fields.supplier);
        self.material_price_per_kg
            .clone_from(&fields.material_price_per_kg);
        self.offer_effective_on
            .clone_from(&fields.offer_effective_on);
        self.offer_source.clone_from(&fields.offer_source);
        self.stock_profile_id.clone_from(&fields.stock_profile_id);
        self.stock_profile_version
            .clone_from(&fields.stock_profile_version);
        self.stock_form.clone_from(&fields.stock_form);
        self.stock_allowance_x_mm
            .clone_from(&fields.stock_allowance_x_mm);
        self.stock_allowance_y_mm
            .clone_from(&fields.stock_allowance_y_mm);
        self.stock_allowance_z_mm
            .clone_from(&fields.stock_allowance_z_mm);
        self.stock_source.clone_from(&fields.stock_source);
        self.machine_id.clone_from(&fields.machine_id);
        self.machine_version.clone_from(&fields.machine_version);
        self.machine_name.clone_from(&fields.machine_name);
        self.process_class.clone_from(&fields.process_class);
        self.machine_envelope_x_mm
            .clone_from(&fields.machine_envelope_x_mm);
        self.machine_envelope_y_mm
            .clone_from(&fields.machine_envelope_y_mm);
        self.machine_envelope_z_mm
            .clone_from(&fields.machine_envelope_z_mm);
        self.machine_source.clone_from(&fields.machine_source);
        self.runtime_profile_id
            .clone_from(&fields.runtime_profile_id);
        self.runtime_profile_version
            .clone_from(&fields.runtime_profile_version);
        self.removal_rate_mm3_per_minute
            .clone_from(&fields.removal_rate_mm3_per_minute);
        self.setup_minutes.clone_from(&fields.setup_minutes);
        self.programming_minutes
            .clone_from(&fields.programming_minutes);
        self.load_unload_minutes
            .clone_from(&fields.load_unload_minutes);
        self.inspection_minutes
            .clone_from(&fields.inspection_minutes);
        self.runtime_source.clone_from(&fields.runtime_source);
    }
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
            resources: DeveloperResourceForm::new(),
            ..Self::default()
        }
    }

    /// Loads the versioned Huntsville aerospace testing baseline into the unsaved form.
    ///
    /// The user must still review, confirm, and save them; this action never creates shop or
    /// production authority and is intentionally unavailable as an automatic startup default.
    fn load_huntsville_test_values(&mut self) {
        self.rate_card_id = "huntsville-2026q3-test-rates".to_owned();
        self.rate_card_version = "1".to_owned();
        self.effective_on = "2026-09-11".to_owned();
        self.currency = "USD".to_owned();
        self.setup_labor_per_hour = "47.00".to_owned();
        self.programming_per_hour = "50.90".to_owned();
        self.run_labor_per_hour = "47.50".to_owned();
        self.machine_per_hour = "45.00".to_owned();
        self.quality_inspection_per_hour = "37.00".to_owned();
        self.pricing_policy_id = "huntsville-2026q3-test-pricing".to_owned();
        self.pricing_policy_version = "1".to_owned();
        self.markup_rate = "0.25".to_owned();
        self.optional_price_floor.clear();
        self.optional_minimum_order.clear();
        self.rounding_decimal_places = "2".to_owned();

        self.resources = DeveloperResourceForm {
            enabled: true,
            confirmed_for_draft: false,
            library_id: "huntsville-2026q3-test-resources".to_owned(),
            library_version: "1".to_owned(),
            material_id: "test-al-6061-t651".to_owned(),
            material_version: "1".to_owned(),
            material_family: "Aluminum".to_owned(),
            material_grade: "6061".to_owned(),
            optional_material_specification: String::new(),
            optional_material_condition: "T651".to_owned(),
            density_kg_per_m3: "2700".to_owned(),
            material_source: "aluminum-association-density-2025".to_owned(),
            offer_id: "public-6061-plate-test-offer-2026q3".to_owned(),
            offer_version: "1".to_owned(),
            supplier: "Public 6061 benchmark (test only)".to_owned(),
            material_price_per_kg: "10.00".to_owned(),
            offer_effective_on: "2026-09-11".to_owned(),
            offer_source: "public-6061-plate-benchmark-2026q3".to_owned(),
            stock_profile_id: "test-rectangular-stock-allowance".to_owned(),
            stock_profile_version: "1".to_owned(),
            stock_form: "rectangular".to_owned(),
            stock_allowance_x_mm: "6.4".to_owned(),
            stock_allowance_y_mm: "6.4".to_owned(),
            stock_allowance_z_mm: "6.4".to_owned(),
            stock_source: "generic-aerospace-test-assumption".to_owned(),
            machine_id: "test-reference-vmc".to_owned(),
            machine_version: "1".to_owned(),
            machine_name: "Reference 3-axis VMC (VF-2 envelope)".to_owned(),
            process_class: "milling".to_owned(),
            machine_envelope_x_mm: "762".to_owned(),
            machine_envelope_y_mm: "406".to_owned(),
            machine_envelope_z_mm: "508".to_owned(),
            machine_source: "haas-vf2-public-spec".to_owned(),
            runtime_profile_id: "test-aluminum-coarse-runtime".to_owned(),
            runtime_profile_version: "1".to_owned(),
            removal_rate_mm3_per_minute: "250000".to_owned(),
            setup_minutes: "90".to_owned(),
            programming_minutes: "120".to_owned(),
            load_unload_minutes: "5".to_owned(),
            inspection_minutes: "15".to_owned(),
            runtime_source: "generic-aluminum-test-assumption-not-cam".to_owned(),
        };
        self.rates_confirmed = false;
        self.pricing_confirmed = false;
        self.settings_changed_by = "local-test-operator".to_owned();
        self.settings_change_reason =
            "Load research-informed Huntsville aerospace testing baseline".to_owned();
    }

    fn estimate_inputs_complete(&self) -> bool {
        [
            &self.stock_volume_mm3,
            &self.density_kg_per_mm3,
            &self.deliver_quantity,
            &self.planned_spares,
            &self.destructive_samples,
            &self.setup_hours,
            &self.programming_hours,
            &self.cutting_hours_per_item,
            &self.non_cutting_hours_per_item,
            &self.load_unload_hours_per_item,
            &self.in_cycle_inspection_hours_per_item,
            &self.quality_inspection_hours,
            &self.purchased_material,
            &self.cut_charge,
            &self.material_certificate,
            &self.inbound_freight,
            &self.approved_remnant_credit,
            &self.prove_out,
            &self.tooling,
            &self.consumables,
            &self.fixture,
            &self.outside_processing,
            &self.operation_freight,
            &self.nonrecurring_engineering,
            &self.administration,
            &self.overhead,
            &self.accepted_risk_impact,
            &self.expected_rework,
        ]
        .into_iter()
        .all(|value| !value.trim().is_empty())
    }

    fn proposal_adoption_ready(&self) -> bool {
        self.proposal_values_reviewed
            && self.coarse_limitations_accepted
            && !self.proposal_review_reason.trim().is_empty()
            && [
                &self.deliver_quantity,
                &self.planned_spares,
                &self.destructive_samples,
            ]
            .into_iter()
            .all(|value| !value.trim().is_empty())
    }

    fn session_settings_ready(&self) -> bool {
        self.rates_confirmed
            && self.pricing_confirmed
            && [
                &self.rate_card_id,
                &self.rate_card_version,
                &self.effective_on,
                &self.currency,
                &self.setup_labor_per_hour,
                &self.programming_per_hour,
                &self.run_labor_per_hour,
                &self.machine_per_hour,
                &self.quality_inspection_per_hour,
                &self.pricing_policy_id,
                &self.pricing_policy_version,
                &self.markup_rate,
                &self.rounding_decimal_places,
            ]
            .into_iter()
            .all(|value| !value.trim().is_empty())
    }

    fn settings_save_ready(&self) -> bool {
        self.session_settings_ready()
            && self.resources.complete()
            && !self.settings_changed_by.trim().is_empty()
            && !self.settings_change_reason.trim().is_empty()
    }

    fn rates(&self) -> DeveloperRateInputFields {
        DeveloperRateInputFields {
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
        }
    }

    fn pricing(&self) -> DeveloperPricingInputFields {
        DeveloperPricingInputFields {
            confirmed_for_session: self.pricing_confirmed,
            pricing_policy_id: self.pricing_policy_id.clone(),
            pricing_policy_version: self.pricing_policy_version.clone(),
            markup_rate: self.markup_rate.clone(),
            optional_price_floor: self.optional_price_floor.clone(),
            optional_minimum_order: self.optional_minimum_order.clone(),
            rounding_decimal_places: self.rounding_decimal_places.clone(),
        }
    }

    fn settings_request(&self, expected_revision: Option<u32>) -> SaveShopSettingsRequest {
        SaveShopSettingsRequest {
            expected_revision,
            changed_by: self.settings_changed_by.clone(),
            change_reason: self.settings_change_reason.clone(),
            rates: self.rates(),
            pricing: self.pricing(),
            resources: self.resources.fields(),
        }
    }

    fn apply_settings(&mut self, settings: &ShopSettingsSnapshot) {
        self.currency.clone_from(&settings.currency);
        if let Some(rates) = settings.rates.as_ref() {
            self.rate_card_id.clone_from(&rates.rate_card_id);
            self.rate_card_version.clone_from(&rates.rate_card_version);
            self.effective_on.clone_from(&rates.effective_on);
            self.setup_labor_per_hour
                .clone_from(&rates.setup_labor_per_hour);
            self.programming_per_hour
                .clone_from(&rates.programming_per_hour);
            self.run_labor_per_hour
                .clone_from(&rates.run_labor_per_hour);
            self.machine_per_hour.clone_from(&rates.machine_per_hour);
            self.quality_inspection_per_hour
                .clone_from(&rates.quality_inspection_per_hour);
        } else {
            self.rate_card_id.clear();
            self.rate_card_version.clear();
            self.effective_on.clear();
            self.setup_labor_per_hour.clear();
            self.programming_per_hour.clear();
            self.run_labor_per_hour.clear();
            self.machine_per_hour.clear();
            self.quality_inspection_per_hour.clear();
        }
        if let Some(pricing) = settings.pricing.as_ref() {
            self.pricing_policy_id
                .clone_from(&pricing.pricing_policy_id);
            self.pricing_policy_version
                .clone_from(&pricing.pricing_policy_version);
            self.markup_rate.clone_from(&pricing.markup_rate);
            self.optional_price_floor
                .clone_from(&pricing.optional_price_floor);
            self.optional_minimum_order
                .clone_from(&pricing.optional_minimum_order);
            self.rounding_decimal_places
                .clone_from(&pricing.rounding_decimal_places);
        } else {
            self.pricing_policy_id.clear();
            self.pricing_policy_version.clear();
            self.markup_rate.clear();
            self.optional_price_floor.clear();
            self.optional_minimum_order.clear();
            self.rounding_decimal_places.clear();
        }
        self.rates_confirmed = false;
        self.pricing_confirmed = false;
        self.resources.apply(settings.resources.as_ref());
        self.settings_change_reason.clear();
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
            rates: self.rates(),
            pricing: self.pricing(),
            proposal_adoption: None,
        }
    }

    fn proposal_request(
        &self,
        proposal: &DraftEstimateProposalSummary,
    ) -> EvaluateDraftEstimateRequest {
        let mut request = self.request(proposal.selection_id.clone(), proposal.analysis_id.clone());
        request.proposal_adoption = Some(DraftEstimateProposalAdoptionInput {
            settings_revision: proposal.settings_revision,
            library_id: proposal.library_id.clone(),
            library_version: proposal.library_version,
            proposal_values_reviewed: self.proposal_values_reviewed,
            coarse_limitations_accepted: self.coarse_limitations_accepted,
            review_reason: self.proposal_review_reason.clone(),
            deliver_quantity: self.deliver_quantity.clone(),
            planned_spares: self.planned_spares.clone(),
            destructive_samples: self.destructive_samples.clone(),
        });
        request
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
    let (proposal_state, set_proposal_state) = signal(ProposalPanelState::NotRequested);
    let (is_selecting, set_is_selecting) = signal(false);
    let (active_view, set_active_view) = signal(WorkspaceView::Estimate);
    let (viewer_state, set_viewer_state) = signal(ModelViewerPanelState::Loading);
    let (settings_state, set_settings_state) = signal(SettingsPanelState::Loading);
    let form = RwSignal::new(DeveloperEstimateForm::new());

    leptos::task::spawn_local(async move {
        match invoke_partprobe(COMMAND_DESKTOP_CONTRACT, JsValue::UNDEFINED).await {
            Ok(value) => match serde_wasm_bindgen::from_value::<DesktopContract>(value) {
                Ok(contract)
                    if contract.model_viewer == ModelViewerAvailability::DeveloperPreview =>
                {
                    set_viewer_state.set(ModelViewerPanelState::Ready);
                }
                Ok(_) => set_viewer_state.set(ModelViewerPanelState::Unavailable),
                Err(_) => set_viewer_state.set(ModelViewerPanelState::Failed(
                    HostCommandError::host_state_unavailable("VIS1-CONTRACT-RESULT"),
                )),
            },
            Err(error) => {
                let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                    .unwrap_or_else(|_| {
                        HostCommandError::host_state_unavailable("VIS1-CONTRACT-INVOKE")
                    });
                set_viewer_state.set(ModelViewerPanelState::Failed(error));
            }
        }
    });

    let switch_workspace = Callback::new(move |target: WorkspaceView| {
        let current = active_view.get_untracked();
        if current == target {
            return;
        }
        let viewer_transition =
            current == WorkspaceView::ModelAndStock || target == WorkspaceView::ModelAndStock;
        if !viewer_transition {
            set_active_view.set(target);
            return;
        }
        if target == WorkspaceView::ModelAndStock && !viewer_state.get_untracked().is_available() {
            return;
        }
        let requested_mode = if target == WorkspaceView::ModelAndStock {
            ModelViewerWorkspaceMode::Visible
        } else {
            ModelViewerWorkspaceMode::Hidden
        };
        set_viewer_state.set(ModelViewerPanelState::Switching);
        leptos::task::spawn_local(async move {
            let request = SetModelViewerWorkspaceRequest {
                mode: requested_mode,
            };
            let args = serde_wasm_bindgen::to_value(&SetModelViewerWorkspaceArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_SET_MODEL_VIEWER_WORKSPACE, args).await {
                Ok(value) => {
                    match serde_wasm_bindgen::from_value::<ModelViewerWorkspaceResult>(value) {
                        Ok(result) if result.mode == requested_mode => {
                            set_active_view.set(target);
                            if target == WorkspaceView::ModelAndStock {
                                set_viewer_state.set(ModelViewerPanelState::Active(result));
                            } else {
                                set_viewer_state.set(ModelViewerPanelState::Ready);
                            }
                        }
                        _ => set_viewer_state.set(ModelViewerPanelState::Failed(
                            HostCommandError::host_state_unavailable("VIS1-WORKSPACE-RESULT"),
                        )),
                    }
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::host_state_unavailable("VIS1-WORKSPACE-INVOKE")
                        });
                    set_viewer_state.set(ModelViewerPanelState::Failed(error));
                }
            }
        });
    });

    let load_settings = Callback::new(move |()| {
        set_settings_state.set(SettingsPanelState::Loading);
        leptos::task::spawn_local(async move {
            match invoke_partprobe(COMMAND_LOAD_SHOP_SETTINGS, JsValue::UNDEFINED).await {
                Ok(value) => match serde_wasm_bindgen::from_value::<ShopSettingsState>(value) {
                    Ok(ShopSettingsState::NotConfigured) => {
                        set_settings_state.set(SettingsPanelState::NotConfigured);
                    }
                    Ok(ShopSettingsState::Available { settings }) => {
                        form.update(|current| current.apply_settings(&settings));
                        set_settings_state.set(SettingsPanelState::Available(settings));
                    }
                    Err(_) => set_settings_state.set(SettingsPanelState::Failed {
                        error: HostCommandError::settings_unavailable("USE2-SETTINGS-RESULT"),
                        revision: None,
                    }),
                },
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::settings_unavailable("USE2-SETTINGS-INVOKE")
                        });
                    set_settings_state.set(SettingsPanelState::Failed {
                        error,
                        revision: None,
                    });
                }
            }
        });
    });
    load_settings.run(());

    let save_settings = Callback::new(move |()| {
        if !form.with(DeveloperEstimateForm::settings_save_ready) {
            return;
        }
        let expected_revision = settings_state.get_untracked().revision();
        let request = form.with(|current| current.settings_request(expected_revision));
        set_settings_state.set(SettingsPanelState::Saving);
        leptos::task::spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&SaveShopSettingsArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_SAVE_SHOP_SETTINGS, args).await {
                Ok(value) => match serde_wasm_bindgen::from_value::<ShopSettingsState>(value) {
                    Ok(ShopSettingsState::Available { settings }) => {
                        form.update(|current| current.apply_settings(&settings));
                        set_settings_state.set(SettingsPanelState::Available(settings));
                    }
                    Ok(ShopSettingsState::NotConfigured) | Err(_) => {
                        set_settings_state.set(SettingsPanelState::Failed {
                            error: HostCommandError::settings_unavailable(
                                "USE2-SETTINGS-SAVE-RESULT",
                            ),
                            revision: expected_revision,
                        });
                    }
                },
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::settings_unavailable("USE2-SETTINGS-SAVE-INVOKE")
                        });
                    set_settings_state.set(SettingsPanelState::Failed {
                        error,
                        revision: expected_revision,
                    });
                }
            }
        });
    });

    Effect::new(move |_| {
        let _ = form.get();
        set_estimate_state.set(DraftEstimatePanelState::NotReady);
    });
    Effect::new(move |_| {
        let _ = settings_state.get();
        set_proposal_state.set(ProposalPanelState::NotRequested);
    });

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
                    set_proposal_state.set(ProposalPanelState::NotRequested);
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
        set_proposal_state.set(ProposalPanelState::NotRequested);
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
                <p class="eyebrow">"PARTPROBE"</p>
                <h1>{move || active_view.get().title()}</h1>
            </div>
            <div class="app-header-actions">
                <nav class="primary-navigation" aria-label="Primary">
                    <button
                        type="button"
                        class=move || if active_view.get() == WorkspaceView::Estimate {
                            "navigation-action active"
                        } else {
                            "navigation-action"
                        }
                        aria-pressed=move || active_view.get() == WorkspaceView::Estimate
                        disabled=move || viewer_state.get().is_switching()
                        on:click=move |_| switch_workspace.run(WorkspaceView::Estimate)
                    >
                        "Estimate"
                    </button>
                    <button
                        type="button"
                        class=move || if active_view.get() == WorkspaceView::Settings {
                            "navigation-action active"
                        } else {
                            "navigation-action"
                        }
                        aria-pressed=move || active_view.get() == WorkspaceView::Settings
                        disabled=move || viewer_state.get().is_switching()
                        on:click=move |_| switch_workspace.run(WorkspaceView::Settings)
                    >
                        "Settings"
                    </button>
                    <Show when=move || viewer_state.get().is_available()>
                        <button
                            type="button"
                            class=move || if active_view.get() == WorkspaceView::ModelAndStock {
                                "navigation-action active"
                            } else {
                                "navigation-action"
                            }
                            aria-pressed=move || active_view.get() == WorkspaceView::ModelAndStock
                            disabled=move || viewer_state.get().is_switching()
                            on:click=move |_| switch_workspace.run(WorkspaceView::ModelAndStock)
                        >
                            "Model & stock"
                        </button>
                    </Show>
                </nav>
                <div class="session-state" aria-label="Application state">
                    <span class="state-dot" aria-hidden="true"></span>
                    <span>"Internal test · Local only · Estimate not saved"</span>
                </div>
            </div>
        </header>

        <Show
            when=move || active_view.get() == WorkspaceView::Estimate
            fallback=move || view! {
                <Show
                    when=move || active_view.get() == WorkspaceView::Settings
                    fallback=move || view! {
                        <ModelViewerWorkspace model_state analysis_state proposal_state viewer_state />
                    }
                >
                    <SettingsWorkspace form settings_state set_active_view save_settings load_settings />
                </Show>
            }
        >
            <main id="workspace" class="workspace">
                <section class="model-panel" aria-labelledby="model-heading">
                    <div class="panel-heading">
                        <div>
                            <p class="section-index">"MODEL"</p>
                            <h2 id="model-heading">"Upload model"</h2>
                        </div>
                        <span class="status-chip">"STEP estimating · STL/3MF analysis only"</span>
                    </div>

                    <div class="drop-zone">
                        <div class="model-mark" aria-hidden="true">"P"</div>
                        <p class="status-heading" aria-live="polite">
                            {move || if is_selecting.get() {
                                "Choose a model in the file window"
                            } else {
                                model_state.get().status_heading()
                            }}
                        </p>
                        <p class="status-detail">
                            {move || if is_selecting.get() {
                                "Files stay on this computer."
                            } else {
                                model_state.get().status_detail()
                            }}
                        </p>
                        <button
                            type="button"
                            class="primary-action"
                            disabled=move || is_selecting.get()
                            on:click=select_model
                        >
                            {move || if is_selecting.get() { "File window open" } else { "Choose model" }}
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
                                _ => "Analyze model",
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
                    proposal_state
                    set_proposal_state
                    estimate_state
                    set_estimate_state
                    form
                    set_active_view
                />
            </main>
        </Show>
    }
}

#[component]
fn ModelViewerWorkspace(
    model_state: ReadSignal<ModelPanelState>,
    analysis_state: ReadSignal<AnalysisPanelState>,
    proposal_state: ReadSignal<ProposalPanelState>,
    viewer_state: ReadSignal<ModelViewerPanelState>,
) -> impl IntoView {
    let source = move || {
        model_state.get().selected_source().map_or_else(
            || "No model selected".to_owned(),
            |source| source.display_name.clone(),
        )
    };
    let analysis = move || match analysis_state.get() {
        AnalysisPanelState::Available(_) => "Provisional analysis available",
        AnalysisPanelState::Running => "Analysis running",
        AnalysisPanelState::Cancelling => "Analysis cancelling",
        AnalysisPanelState::Cancelled => "Analysis cancelled",
        AnalysisPanelState::Failed(_) => "Analysis failed",
        AnalysisPanelState::NotStarted => "Not analyzed",
    };
    let proposal = move || match proposal_state.get() {
        ProposalPanelState::Evaluated(evaluation)
            if matches!(
                evaluation.as_ref(),
                DraftEstimateProposalEvaluation::Available { .. }
            ) =>
        {
            "Reviewable proposal available"
        }
        ProposalPanelState::Loading => "Proposal loading",
        ProposalPanelState::Failed(_) => "Proposal failed",
        ProposalPanelState::Evaluated(_) => "Proposal unavailable",
        ProposalPanelState::NotRequested => "Not prepared",
    };
    let viewer_notice = move || match viewer_state.get() {
        ModelViewerPanelState::Active(result) => result.notice,
        ModelViewerPanelState::Failed(error) => {
            format!("{} Diagnostic: {}", error.message, error.diagnostic_id)
        }
        _ => "Native viewport is changing state.".to_owned(),
    };

    view! {
        <main id="workspace" class="model-viewer-workspace">
            <section class="model-viewer-inspector" aria-labelledby="model-viewer-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"VISUAL REVIEW"</p>
                        <h2 id="model-viewer-heading">"Model & proposed stock"</h2>
                    </div>
                    <span class="status-chip blocked">"Synthetic preview"</span>
                </div>

                <div class="viewer-preview-notice" role="status">
                    <p class="blocked-title">"Renderer validation scene"</p>
                    <p>{viewer_notice}</p>
                </div>

                <dl class="viewer-summary">
                    <div><dt>"Selected source"</dt><dd>{source}</dd></div>
                    <div><dt>"Analysis"</dt><dd>{analysis}</dd></div>
                    <div><dt>"Stock proposal"</dt><dd>{proposal}</dd></div>
                    <div><dt>"View"</dt><dd>"Isometric · model opaque · stock translucent"</dd></div>
                </dl>

                <p class="evidence-note">
                    "The native viewport is inside this PartProbe window. Its current shapes are a fixed renderer test—not the selected model, stock authority, workholding, or CAM evidence."
                </p>
                <p class="viewer-navigation-note">
                    "Use Estimate or Settings above to return to the full workspace."
                </p>
            </section>
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
    let format = source_format_label(source.format);
    view! {
        <dl class="source-summary" aria-label=accessible_label>
            <div><dt>"File"</dt><dd>{source.display_name}</dd></div>
            <div><dt>"Format"</dt><dd>{format}</dd></div>
        </dl>
    }
}

const fn unit_resolution_label(resolution: UnitResolution) -> &'static str {
    match resolution {
        UnitResolution::Declared => "Declared by the format",
        UnitResolution::Confirmed => "Confirmed by the user",
        UnitResolution::Inferred => "Inferred; review required",
        UnitResolution::Unresolved => "Unresolved; confirmation required",
    }
}

const fn measurement_basis_label(basis: MeshMeasurementBasis) -> &'static str {
    match basis {
        MeshMeasurementBasis::SourceCoordinates => "Unresolved source coordinates",
        MeshMeasurementBasis::CanonicalMillimeters => "Canonical millimeters",
    }
}

const fn confidence_label(level: GeometryConfidenceLevel) -> &'static str {
    match level {
        GeometryConfidenceLevel::High => "High",
        GeometryConfidenceLevel::Medium => "Medium",
        GeometryConfidenceLevel::Low => "Low",
        GeometryConfidenceLevel::NeedsReview => "Needs review",
    }
}

const fn topology_identity_label(identity: MeshTopologyIdentity) -> &'static str {
    match identity {
        MeshTopologyIdentity::ExactSourceCoordinates => "Exact source coordinates; no welding",
        MeshTopologyIdentity::SourceVertexIndices => "Retained source vertex indices",
    }
}

const fn self_intersection_label(state: MeshSelfIntersectionState) -> &'static str {
    match state {
        MeshSelfIntersectionState::NotDetected => "Not detected",
        MeshSelfIntersectionState::Detected => "Detected",
        MeshSelfIntersectionState::Indeterminate => "Indeterminate",
    }
}

const fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

const fn stl_encoding_label(encoding: Option<StlEncoding>) -> &'static str {
    match encoding {
        Some(StlEncoding::Ascii) => "ASCII STL",
        Some(StlEncoding::Binary) => "Binary STL",
        None => "3MF package",
    }
}

const fn unit_declaration_label(unit_was_explicit: Option<bool>) -> &'static str {
    match unit_was_explicit {
        Some(true) => "Explicit 3MF declaration",
        Some(false) => "3MF normative default",
        None => "Not declared by STL",
    }
}

#[component]
fn AnalysisEvidence(state: ReadSignal<AnalysisPanelState>) -> impl IntoView {
    move || match state.get() {
        AnalysisPanelState::Available(result) => {
            let accessible_label = provisional_geometry_accessible_label(&result.geometry);
            let warning_count = result
                .stages
                .iter()
                .map(|stage| stage.warning_codes.len())
                .sum::<usize>();
            match result.geometry {
                ProvisionalGeometryFacts::ExactBrep(geometry) => {
                    let centroid = geometry.center_of_mass_mm.join(", ");
                    let bounds = geometry.aabb_extents_mm.as_ref().map_or_else(
                        || "Unavailable in legacy analysis".to_owned(),
                        |value| format!("{} × {} × {} mm", value[0], value[1], value[2]),
                    );
                    view! {
                        <section class="analysis-evidence" aria-label=accessible_label>
                            <div class="analysis-evidence-heading">
                                <div>
                                    <p class="section-index">"MODEL ANALYSIS"</p>
                                    <h3 id="analysis-evidence-heading">"Model measurements"</h3>
                                </div>
                                <span class="status-chip">"Review required"</span>
                            </div>
                            <dl class="geometry-facts">
                                <div><dt>"Model size (X × Y × Z)"</dt><dd>{bounds}</dd></div>
                                <div><dt>"Surface area"</dt><dd>{geometry.surface_area_mm2}" mm²"</dd></div>
                                <div><dt>"Enclosed volume"</dt><dd>{geometry.enclosed_volume_mm3}" mm³"</dd></div>
                                <div><dt>"Solid bodies"</dt><dd>{geometry.solid_body_count}</dd></div>
                                <div><dt>"Units"</dt><dd>"Millimeters"</dd></div>
                                <div><dt>"Warnings"</dt><dd>{warning_count}</dd></div>
                            </dl>
                            <details>
                                <summary>"Technical details"</summary>
                                <dl class="geometry-facts">
                                    <div><dt>"Centroid"</dt><dd>{centroid}" mm"</dd></div>
                                    <div><dt>"Engine"</dt><dd>{geometry.geometry_engine}</dd></div>
                                    <div><dt>"Analysis ID"</dt><dd>{result.analysis_id}</dd></div>
                                </dl>
                            </details>
                            <p class="evidence-note">
                                "Provisional analysis—review the measurements and warnings before estimating."
                            </p>
                        </section>
                    }
                    .into_any()
                }
                ProvisionalGeometryFacts::Mesh(geometry) => {
                    let format = source_format_label(geometry.detected_format);
                    let encoding = stl_encoding_label(geometry.stl_encoding);
                    let units = length_unit_label(geometry.source_units);
                    let resolution = unit_resolution_label(geometry.unit_resolution);
                    let unit_declaration = unit_declaration_label(geometry.unit_was_explicit);
                    let basis = measurement_basis_label(geometry.measurement_basis);
                    let extents = geometry.aabb_extents.join(", ");
                    let volume = geometry
                        .enclosed_volume
                        .as_deref()
                        .map_or_else(|| "Unavailable / withheld".to_owned(), str::to_owned);
                    let centroid = geometry.center_of_mass.as_ref().map_or_else(
                        || "Unavailable / withheld".to_owned(),
                        |value| value.join(", "),
                    );
                    let confidence = confidence_label(geometry.confidence_level);
                    let reasons = geometry.confidence_reason_codes.join(", ");
                    let warnings = if geometry.warning_codes.is_empty() {
                        "None".to_owned()
                    } else {
                        geometry.warning_codes.join(", ")
                    };
                    let topology = topology_identity_label(geometry.topology_identity);
                    let intersection = self_intersection_label(geometry.self_intersection);
                    view! {
                        <section class="analysis-evidence" aria-label=accessible_label>
                            <div class="analysis-evidence-heading">
                                <div>
                                    <p class="section-index">"PROVISIONAL MESH / SESSION ONLY"</p>
                                    <h3 id="analysis-evidence-heading">"Mesh geometry evidence"</h3>
                                </div>
                                <span class="status-chip">"Estimate unavailable"</span>
                            </div>
                            <dl class="geometry-facts">
                                <div><dt>"Detected format"</dt><dd>{format}</dd></div>
                                <div><dt>"Encoding"</dt><dd>{encoding}</dd></div>
                                <div><dt>"Source units"</dt><dd>{units}</dd></div>
                                <div><dt>"Unit resolution"</dt><dd>{resolution}</dd></div>
                                <div><dt>"Unit declaration"</dt><dd>{unit_declaration}</dd></div>
                                <div><dt>"Measurement basis"</dt><dd>{basis}</dd></div>
                                <div><dt>"Bounds extents"</dt><dd>{extents}</dd></div>
                                <div><dt>"Surface area"</dt><dd>{geometry.surface_area}</dd></div>
                                <div><dt>"Enclosed volume"</dt><dd>{volume}</dd></div>
                                <div><dt>"Centroid"</dt><dd>{centroid}</dd></div>
                                <div><dt>"Triangles"</dt><dd>{geometry.triangle_count}</dd></div>
                                <div><dt>"Manifold"</dt><dd>{yes_no(geometry.manifold)}</dd></div>
                                <div><dt>"Watertight"</dt><dd>{yes_no(geometry.watertight)}</dd></div>
                                <div><dt>"Consistently wound"</dt><dd>{yes_no(geometry.consistently_wound)}</dd></div>
                                <div><dt>"Self-intersection"</dt><dd>{intersection}</dd></div>
                                <div><dt>"Confidence"</dt><dd>{confidence}</dd></div>
                                <div><dt>"Confidence reasons"</dt><dd>{reasons}</dd></div>
                                <div><dt>"Topology identity"</dt><dd>{topology}</dd></div>
                                <div><dt>"Topology policy"</dt><dd>{geometry.topology_policy_version}</dd></div>
                                <div><dt>"Welding"</dt><dd>"Not applied"</dd></div>
                                <div><dt>"Warnings"</dt><dd>{warnings}</dd></div>
                                <div><dt>"Parser"</dt><dd>{geometry.algorithm_version}</dd></div>
                                <div><dt>"Intersection detector"</dt><dd>{geometry.self_intersection_algorithm_version}</dd></div>
                                <div><dt>"Confidence policy"</dt><dd>{geometry.confidence_policy_version}</dd></div>
                                <div><dt>"Analysis ID"</dt><dd>{result.analysis_id}</dd></div>
                            </dl>
                            <p class="evidence-note">
                                "Mesh measurements are provisional comparison evidence. Missing measurements remain withheld, and this representation cannot authorize a deterministic draft estimate."
                            </p>
                        </section>
                    }
                    .into_any()
                }
            }
        }
        AnalysisPanelState::Failed(error) => {
            let accessible_label =
                provisional_analysis_failure_accessible_label(&error.diagnostic_id);
            view! {
                <section class="analysis-error" role="alert" aria-label=accessible_label>
                    <p class="blocked-title">"Analysis couldn’t be completed"</p>
                    <p>{error.message}</p>
                    <p>"Your selected model is unchanged."</p>
                    <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
                </section>
            }
            .into_any()
        }
        AnalysisPanelState::Running => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Analyzing model"</p>
                <p>"You can cancel without clearing the selected model."</p>
            </section>
        }
        .into_any(),
        AnalysisPanelState::Cancelling => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Cancellation requested"</p>
                <p>"The selected model will remain available when cancellation finishes."</p>
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
fn ProposalEvidence(
    state: ReadSignal<ProposalPanelState>,
    form: RwSignal<DeveloperEstimateForm>,
) -> impl IntoView {
    move || match state.get() {
        ProposalPanelState::NotRequested => view! {
            <p class="estimate-readiness">
                "Save complete test settings, then prepare estimate inputs."
            </p>
        }
        .into_any(),
        ProposalPanelState::Loading => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Preparing estimate inputs"</p>
                <p>"Applying the saved test settings to this model."</p>
            </section>
        }
        .into_any(),
        ProposalPanelState::Evaluated(evaluation) => match *evaluation {
            DraftEstimateProposalEvaluation::Available { proposal } => {
                let model = proposal.model_extents_mm.join(" × ");
                let blank = proposal.blank_dimensions_mm.join(" × ");
                let reasons = proposal.reason_codes.join(", ");
                view! {
                    <section class="analysis-evidence proposal-evidence" aria-label="Review-only model estimate proposal">
                        <div class="analysis-evidence-heading">
                            <div>
                                <p class="section-index">"REVIEW REQUIRED"</p>
                                <h3>"Proposed manufacturing inputs"</h3>
                            </div>
                            <span class="status-chip">"Not adopted"</span>
                        </div>
                        <dl class="geometry-facts">
                            <div><dt>"Model size"</dt><dd>{model}" mm"</dd></div>
                            <div><dt>"Stock size"</dt><dd>{blank}" mm"</dd></div>
                            <div><dt>"Blank volume"</dt><dd>{proposal.blank_volume_mm3}" mm³"</dd></div>
                            <div><dt>"Removed volume"</dt><dd>{proposal.removed_volume_mm3}" mm³"</dd></div>
                            <div><dt>"Material"</dt><dd>{proposal.material_grade}</dd></div>
                            <div><dt>"Blank mass"</dt><dd>{proposal.blank_mass_kg}" kg"</dd></div>
                            <div><dt>"Material cost / item"</dt><dd>{proposal.currency.clone()}" "{proposal.unit_stock_material_cost}</dd></div>
                            <div><dt>"Machine"</dt><dd>{proposal.machine_name}</dd></div>
                            <div><dt>"Estimated cutting time"</dt><dd>{proposal.cutting_hours_per_item}" hr/item"</dd></div>
                            <div><dt>"Setup + programming"</dt><dd>{proposal.setup_hours}" + "{proposal.programming_hours}" hr/lot"</dd></div>
                        </dl>
                        <p class="evidence-note">
                            "Uses the model’s current orientation and a rectangular stock allowance. Stock availability and detailed cycle analysis are not included."
                        </p>
                        <details>
                            <summary>"Proposal details"</summary>
                            <p>"Settings revision " {proposal.settings_revision}</p>
                            <p>"Rule " {proposal.rule_id}" v"{proposal.rule_version}</p>
                            <p class="diagnostic-id">"Reasons: " {reasons}</p>
                        </details>
                        <fieldset>
                            <legend>"Review proposal"</legend>
                            <ReviewCheckbox
                                form
                                label="The proposed stock, material, machine, and times look reasonable for this test."
                                read=|form| form.proposal_values_reviewed
                                write=|form, value| form.proposal_values_reviewed = value
                            />
                            <ReviewCheckbox
                                form
                                label="I understand this estimate excludes tooling, fixtures, outside processes, freight, overhead, risk, and detailed cycle analysis."
                                read=|form| form.coarse_limitations_accepted
                                write=|form, value| form.coarse_limitations_accepted = value
                            />
                            <TextInput
                                form
                                label="Review note"
                                read=|form| &form.proposal_review_reason
                                write=|form, value| form.proposal_review_reason = value
                            />
                        </fieldset>
                    </section>
                }
                .into_any()
            }
            DraftEstimateProposalEvaluation::Unavailable { reason } => view! {
                <section class="analysis-error" role="status">
                    <p class="blocked-title">"Proposal unavailable"</p>
                    <p>{reason}</p>
                    <p>"Open Settings to save and confirm a complete starter resource draft."</p>
                </section>
            }
            .into_any(),
            DraftEstimateProposalEvaluation::Blocked { reason } => view! {
                <section class="analysis-error" role="alert">
                    <p class="blocked-title">"Proposal blocked"</p>
                    <p>{reason}</p>
                </section>
            }
            .into_any(),
        },
        ProposalPanelState::Failed(error) => view! {
            <section class="analysis-error" role="alert">
                    <p class="blocked-title">"Couldn’t prepare estimate inputs"</p>
                <p>{error.message}</p>
                <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
            </section>
        }
        .into_any(),
    }
}

#[component]
fn EstimateWorkspace(
    analysis_state: ReadSignal<AnalysisPanelState>,
    proposal_state: ReadSignal<ProposalPanelState>,
    set_proposal_state: WriteSignal<ProposalPanelState>,
    estimate_state: ReadSignal<DraftEstimatePanelState>,
    set_estimate_state: WriteSignal<DraftEstimatePanelState>,
    form: RwSignal<DeveloperEstimateForm>,
    set_active_view: WriteSignal<WorkspaceView>,
) -> impl IntoView {
    Effect::new(move |_| {
        let _ = analysis_state.get();
        form.update(|form| {
            form.geometry_review.clear();
            form.proposal_values_reviewed = false;
            form.coarse_limitations_accepted = false;
            form.proposal_review_reason.clear();
        });
        set_proposal_state.set(ProposalPanelState::NotRequested);
    });
    let prepare_proposal = move |_| {
        let Some((selection_id, analysis_id)) = (match analysis_state.get_untracked() {
            AnalysisPanelState::Available(result) => {
                Some((result.selection_id.clone(), result.analysis_id.clone()))
            }
            _ => None,
        }) else {
            return;
        };
        set_proposal_state.set(ProposalPanelState::Loading);
        leptos::task::spawn_local(async move {
            let request = PrepareDraftEstimateProposalRequest {
                selection_id,
                analysis_id,
            };
            let args = serde_wasm_bindgen::to_value(&PrepareDraftEstimateProposalArgs { request })
                .unwrap_or(JsValue::UNDEFINED);
            match invoke_partprobe(COMMAND_PREPARE_DRAFT_ESTIMATE_PROPOSAL, args).await {
                Ok(value) => {
                    let state =
                        serde_wasm_bindgen::from_value::<DraftEstimateProposalEvaluation>(value)
                            .map(Box::new)
                            .map(ProposalPanelState::Evaluated)
                            .unwrap_or_else(|_| {
                                ProposalPanelState::Failed(
                                    HostCommandError::invalid_estimate_input(
                                        "USE3-PROPOSAL-RESULT",
                                    ),
                                )
                            });
                    set_proposal_state.set(state);
                }
                Err(error) => {
                    let error = serde_wasm_bindgen::from_value::<HostCommandError>(error)
                        .unwrap_or_else(|_| {
                            HostCommandError::invalid_estimate_input("USE3-PROPOSAL-INVOKE")
                        });
                    set_proposal_state.set(ProposalPanelState::Failed(error));
                }
            }
        });
    };
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
        let request = match proposal_state.get_untracked() {
            ProposalPanelState::Evaluated(evaluation) => match *evaluation {
                DraftEstimateProposalEvaluation::Available { proposal } => {
                    form.with(|form| form.proposal_request(&proposal))
                }
                DraftEstimateProposalEvaluation::Unavailable { .. }
                | DraftEstimateProposalEvaluation::Blocked { .. } => {
                    form.with(|form| form.request(selection_id, analysis_id))
                }
            },
            ProposalPanelState::NotRequested
            | ProposalPanelState::Loading
            | ProposalPanelState::Failed(_) => {
                form.with(|form| form.request(selection_id, analysis_id))
            }
        };
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
                    <p class="section-index">"ESTIMATE"</p>
                    <h2 id="estimate-heading">"Part estimate"</h2>
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

            {move || if analysis_supports_draft_estimate(&analysis_state.get()) {
                view! {
                    <form class="estimate-form" on:submit=submit>
                        <section class="settings-summary" aria-labelledby="proposal-heading">
                            <div>
                                <p class="section-index">"ESTIMATE BASIS"</p>
                                <h3 id="proposal-heading">"Estimated stock and run time"</h3>
                                <p>"Uses the model dimensions and saved test settings. Review these values before calculating."</p>
                            </div>
                            <button
                                type="button"
                                class="secondary-action"
                                disabled=move || matches!(proposal_state.get(), ProposalPanelState::Loading)
                                on:click=prepare_proposal
                            >
                                {move || if matches!(proposal_state.get(), ProposalPanelState::Loading) {
                                    "Preparing inputs"
                                } else {
                                    "Prepare estimate inputs"
                                }}
                            </button>
                        </section>
                        <ProposalEvidence state=proposal_state form />
                        <section class="settings-summary" aria-labelledby="settings-summary-heading">
                            <div>
                                <p class="section-index">"SHOP CONFIGURATION"</p>
                                <h3 id="settings-summary-heading">"Rates and pricing"</h3>
                                <p>
                                    {move || if form.with(DeveloperEstimateForm::session_settings_ready) {
                                        "Session settings are complete and confirmed."
                                    } else {
                                        "Configure and confirm rates and pricing before calculating."
                                    }}
                                </p>
                            </div>
                            <button
                                type="button"
                                class="secondary-action"
                                on:click=move |_| set_active_view.set(WorkspaceView::Settings)
                            >
                                "Open settings"
                            </button>
                        </section>
                        <fieldset>
                            <legend>"Geometry review"</legend>
                            <ReviewCheckbox
                                form
                                label="Units are correct: millimeters."
                                read=|form| form.geometry_review.canonical_units_reviewed
                                write=|form, value| form.geometry_review.canonical_units_reviewed = value
                            />
                            <ReviewCheckbox
                                form
                                label="I reviewed the analysis warnings."
                                read=|form| form.geometry_review.warnings_reviewed
                                write=|form, value| form.geometry_review.warnings_reviewed = value
                            />
                        </fieldset>

                        <fieldset>
                            <legend>"Essential estimate inputs"</legend>
                            <p class="fieldset-note">
                                {move || if proposal_state.get().is_available() {
                                    "Quantity remains explicit. Reviewed stock, material, and coarse runtime values will come from the proposal above."
                                } else {
                                    "Quantity and material cannot be taken safely from geometry alone. Without a prepared proposal, complete the manual path below."
                                }}
                            </p>
                            <div class="form-grid">
                                <Show when=move || !proposal_state.get().is_available()>
                                    <ExactInput form label="Stock volume" unit="mm³" read=|f| &f.stock_volume_mm3 write=|f, v| f.stock_volume_mm3 = v />
                                    <ExactInput form label="Material density" unit="kg/mm³" read=|f| &f.density_kg_per_mm3 write=|f, v| f.density_kg_per_mm3 = v />
                                </Show>
                                <ExactInput form label="Order quantity" unit="items" read=|f| &f.deliver_quantity write=|f, v| f.deliver_quantity = v />
                                <ExactInput form label="Planned spares" unit="items" read=|f| &f.planned_spares write=|f, v| f.planned_spares = v />
                                <ExactInput form label="Destructive samples" unit="items" read=|f| &f.destructive_samples write=|f, v| f.destructive_samples = v />
                            </div>
                        </fieldset>

                        <Show when=move || !proposal_state.get().is_available()>
                        <details class="manual-inputs">
                            <summary>"Manual manufacturing assumptions"</summary>
                            <p class="fieldset-note">
                                "These values are temporary manual inputs until stock, process, and runtime proposals are implemented. They are not derived from the model."
                            </p>
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
                                <p class="fieldset-note">"Enter every monetary value in the configured currency; use an explicit 0 where applicable."</p>
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
                        </details>
                        </Show>

                        <p class="estimate-readiness" role="status">
                            {move || if !form.with(DeveloperEstimateForm::session_settings_ready) {
                                "Complete your test settings to continue."
                            } else if proposal_state.get().is_available()
                                && !form.with(DeveloperEstimateForm::proposal_adoption_ready)
                            {
                                "Review the proposed inputs and add a review note."
                            } else if !proposal_state.get().is_available()
                                && !form.with(DeveloperEstimateForm::estimate_inputs_complete)
                            {
                                "Complete the required estimate inputs."
                            } else {
                                "Ready to calculate."
                            }}
                        </p>

                        <button
                            type="submit"
                            class="primary-action calculate-action"
                            disabled=move || {
                                matches!(estimate_state.get(), DraftEstimatePanelState::Evaluating)
                                    || !form.with(DeveloperEstimateForm::session_settings_ready)
                                    || if proposal_state.get().is_available() {
                                        !form.with(DeveloperEstimateForm::proposal_adoption_ready)
                                    } else {
                                        !form.with(DeveloperEstimateForm::estimate_inputs_complete)
                                    }
                            }
                        >
                            {move || if matches!(estimate_state.get(), DraftEstimatePanelState::Evaluating) {
                                "Calculating estimate"
                            } else {
                                "Calculate estimate"
                            }}
                        </button>
                    </form>
                    <EstimateResult state=estimate_state />
                }
                .into_any()
            } else {
                view! {
                    <div class="blocked-state" aria-live="polite">
                        <p class="blocked-title">"Choose and analyze a model"</p>
                        <p>"Select a STEP file to create an estimate. STL and 3MF analysis is available, but estimating is not yet supported for mesh files."</p>
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
fn SettingsWorkspace(
    form: RwSignal<DeveloperEstimateForm>,
    settings_state: ReadSignal<SettingsPanelState>,
    set_active_view: WriteSignal<WorkspaceView>,
    save_settings: Callback<()>,
    load_settings: Callback<()>,
) -> impl IntoView {
    let (settings_section, set_settings_section) = signal(SettingsSection::RatesAndPricing);

    view! {
        <main id="workspace" class="settings-workspace">
            <section class="settings-panel" aria-labelledby="settings-heading">
                <div class="panel-heading">
                    <div>
                        <p class="section-index">"SETTINGS"</p>
                        <h2 id="settings-heading">"Shop rates and resources"</h2>
                    </div>
                    <span class=move || if form.with(DeveloperEstimateForm::session_settings_ready) {
                        "status-chip available"
                    } else {
                        "status-chip blocked"
                    }>
                        {move || if form.with(DeveloperEstimateForm::session_settings_ready) {
                            "Ready"
                        } else {
                            "Setup required"
                        }}
                    </span>
                </div>

                <section class="settings-boundary" role="status" aria-live="polite">
                    <p class="blocked-title">"Saved test settings"</p>
                    <p>
                        {move || match settings_state.get() {
                            SettingsPanelState::Loading => "Loading saved settings.".to_owned(),
                            SettingsPanelState::NotConfigured => "No test settings are saved yet. Load the Huntsville test profile or enter your own reviewed values.".to_owned(),
                            SettingsPanelState::Available(settings) => format!(
                                "Saved local draft revision {}. Reloaded values require fresh confirmation before calculation.",
                                settings.revision,
                            ),
                            SettingsPanelState::Saving => "Saving a new local test-settings revision.".to_owned(),
                            SettingsPanelState::Failed { error, .. } => format!("{} Diagnostic: {}", error.message, error.diagnostic_id),
                        }}
                    </p>
                    <button
                        type="button"
                        class="secondary-action"
                        on:click=move |_| form.update(DeveloperEstimateForm::load_huntsville_test_values)
                    >
                        "Load Huntsville test profile"
                    </button>
                    <p class="fieldset-note">
                        "Research-informed testing baseline—not verified shop pricing. Review every field before saving or calculating."
                    </p>
                </section>

                <nav class="settings-section-nav" aria-label="Settings work areas">
                    <button
                        type="button"
                        class=move || if settings_section.get() == SettingsSection::RatesAndPricing {
                            "settings-section-action active"
                        } else {
                            "settings-section-action"
                        }
                        aria-pressed=move || settings_section.get() == SettingsSection::RatesAndPricing
                        on:click=move |_| set_settings_section.set(SettingsSection::RatesAndPricing)
                    >
                        <span>"Rates & pricing"</span>
                        <small>"Calculation basis"</small>
                    </button>
                    <button
                        type="button"
                        class=move || if settings_section.get() == SettingsSection::Resources {
                            "settings-section-action active"
                        } else {
                            "settings-section-action"
                        }
                        aria-pressed=move || settings_section.get() == SettingsSection::Resources
                        on:click=move |_| set_settings_section.set(SettingsSection::Resources)
                    >
                        <span>"Resources"</span>
                        <small>"Materials, stock & machines"</small>
                    </button>
                </nav>

                <Show when=move || settings_section.get() == SettingsSection::RatesAndPricing>
                    <section class="settings-section" aria-labelledby="rate-settings-heading">
                        <div class="section-heading">
                            <p class="section-index">"CALCULATION BASIS"</p>
                            <h3 id="rate-settings-heading">"Rates and selling-price policy"</h3>
                            <p>"Review the shop rates and pricing rule used by the deterministic estimate."</p>
                        </div>
                        <div class="settings-grid">
                            <fieldset>
                                <legend>"Rate card"</legend>
                                <p class="fieldset-note">"Enter the five hourly rates used by the current estimate."</p>
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
                                <ReviewCheckbox form label="I reviewed these rates for this test." read=|f| f.rates_confirmed write=|f, v| f.rates_confirmed = v />
                            </fieldset>

                            <fieldset>
                                <legend>"Pricing policy"</legend>
                                <div class="form-grid">
                                    <ExactInput form label="Pricing-policy ID" unit="ID" read=|f| &f.pricing_policy_id write=|f, v| f.pricing_policy_id = v />
                                    <ExactInput form label="Policy version" unit="version" read=|f| &f.pricing_policy_version write=|f, v| f.pricing_policy_version = v />
                                    <ExactInput form label="Markup rate" unit="decimal" read=|f| &f.markup_rate write=|f, v| f.markup_rate = v />
                                    <OptionalInput form label="Price floor" unit="currency" read=|f| &f.optional_price_floor write=|f, v| f.optional_price_floor = v />
                                    <OptionalInput form label="Minimum order" unit="currency" read=|f| &f.optional_minimum_order write=|f, v| f.optional_minimum_order = v />
                                    <ExactInput form label="Rounding places" unit="decimals" read=|f| &f.rounding_decimal_places write=|f, v| f.rounding_decimal_places = v />
                                </div>
                                <ReviewCheckbox form label="I reviewed this pricing policy for this test." read=|f| f.pricing_confirmed write=|f, v| f.pricing_confirmed = v />
                            </fieldset>
                        </div>
                    </section>
                </Show>

                <Show when=move || settings_section.get() == SettingsSection::Resources>
                    <section class="settings-section" aria-labelledby="resource-settings-heading">
                        <div class="section-heading">
                            <p class="section-index">"PROPOSAL RESOURCES"</p>
                            <h3 id="resource-settings-heading">"Materials, stock, machines, and runtime"</h3>
                            <p>"Review the material, stock, machine, and runtime assumptions used for test estimates."</p>
                        </div>

                        {move || match settings_state.get() {
                            SettingsPanelState::Available(settings) => settings.resource_catalog.clone()
                                .map(|catalog| view! { <ResourceCatalogReview catalog /> }.into_any())
                                .unwrap_or_else(|| ().into_any()),
                            SettingsPanelState::NotConfigured => ().into_any(),
                            SettingsPanelState::Loading
                            | SettingsPanelState::Saving
                            | SettingsPanelState::Failed { .. } => ().into_any(),
                        }}

                <fieldset>
                    <legend>"Material, stock, machine, and runtime draft"</legend>
                    <p class="fieldset-note">"These values prepare model-specific test estimates. You review the proposal before every calculation."</p>
                    <label class="review-check">
                        <input
                            type="checkbox"
                            prop:checked=move || form.with(|current| current.resources.enabled)
                            on:change=move |event| form.update(|current| {
                                current.resources.enabled = event_target_checked(&event);
                                current.resources.confirmed_for_draft = false;
                            })
                        />
                        <span>"Include material, stock, machine, and runtime values in these test settings."</span>
                    </label>
                </fieldset>

                <Show when=move || form.with(|current| current.resources.enabled)>
                    <div class="settings-grid">
                        <fieldset>
                            <legend>"Material and commercial offer"</legend>
                            <p class="fieldset-note">"Material identity and density remain separate from the time-bounded supplier price."</p>
                            <div class="form-grid">
                                <ExactInput form label="Resource-library ID" unit="ID" read=|f| &f.resources.library_id write=|f, v| f.resources.library_id = v />
                                <ExactInput form label="Library version" unit="version" read=|f| &f.resources.library_version write=|f, v| f.resources.library_version = v />
                                <ExactInput form label="Material ID" unit="ID" read=|f| &f.resources.material_id write=|f, v| f.resources.material_id = v />
                                <ExactInput form label="Material version" unit="version" read=|f| &f.resources.material_version write=|f, v| f.resources.material_version = v />
                                <TextInput form label="Material family" read=|f| &f.resources.material_family write=|f, v| f.resources.material_family = v />
                                <TextInput form label="Alloy / grade" read=|f| &f.resources.material_grade write=|f, v| f.resources.material_grade = v />
                                <OptionalInput form label="Specification" unit="optional" read=|f| &f.resources.optional_material_specification write=|f, v| f.resources.optional_material_specification = v />
                                <OptionalInput form label="Temper / condition" unit="optional" read=|f| &f.resources.optional_material_condition write=|f, v| f.resources.optional_material_condition = v />
                                <ExactInput form label="Density" unit="kg/m³" read=|f| &f.resources.density_kg_per_m3 write=|f, v| f.resources.density_kg_per_m3 = v />
                                <TextInput form label="Material source" read=|f| &f.resources.material_source write=|f, v| f.resources.material_source = v />
                                <ExactInput form label="Offer ID" unit="ID" read=|f| &f.resources.offer_id write=|f, v| f.resources.offer_id = v />
                                <ExactInput form label="Offer version" unit="version" read=|f| &f.resources.offer_version write=|f, v| f.resources.offer_version = v />
                                <TextInput form label="Supplier" read=|f| &f.resources.supplier write=|f, v| f.resources.supplier = v />
                                <ExactInput form label="Material price" unit="USD/kg" read=|f| &f.resources.material_price_per_kg write=|f, v| f.resources.material_price_per_kg = v />
                                <ExactInput form label="Offer effective on" unit="YYYY-MM-DD" read=|f| &f.resources.offer_effective_on write=|f, v| f.resources.offer_effective_on = v />
                                <TextInput form label="Offer source" read=|f| &f.resources.offer_source write=|f, v| f.resources.offer_source = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Stock allowance"</legend>
                            <p class="fieldset-note">"Current governed forms: rectangular, round, or plate. Allowances enlarge each model-envelope axis."</p>
                            <div class="form-grid">
                                <ExactInput form label="Stock-profile ID" unit="ID" read=|f| &f.resources.stock_profile_id write=|f, v| f.resources.stock_profile_id = v />
                                <ExactInput form label="Stock-profile version" unit="version" read=|f| &f.resources.stock_profile_version write=|f, v| f.resources.stock_profile_version = v />
                                <ExactInput form label="Stock form" unit="name" read=|f| &f.resources.stock_form write=|f, v| f.resources.stock_form = v />
                                <ExactInput form label="X allowance" unit="mm" read=|f| &f.resources.stock_allowance_x_mm write=|f, v| f.resources.stock_allowance_x_mm = v />
                                <ExactInput form label="Y allowance" unit="mm" read=|f| &f.resources.stock_allowance_y_mm write=|f, v| f.resources.stock_allowance_y_mm = v />
                                <ExactInput form label="Z allowance" unit="mm" read=|f| &f.resources.stock_allowance_z_mm write=|f, v| f.resources.stock_allowance_z_mm = v />
                                <TextInput form label="Stock-policy source" read=|f| &f.resources.stock_source write=|f, v| f.resources.stock_source = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Machine capability"</legend>
                            <p class="fieldset-note">"Physical capability remains separate from the machine hourly rate above."</p>
                            <div class="form-grid">
                                <ExactInput form label="Machine ID" unit="ID" read=|f| &f.resources.machine_id write=|f, v| f.resources.machine_id = v />
                                <ExactInput form label="Machine version" unit="version" read=|f| &f.resources.machine_version write=|f, v| f.resources.machine_version = v />
                                <TextInput form label="Machine name" read=|f| &f.resources.machine_name write=|f, v| f.resources.machine_name = v />
                                <ExactInput form label="Process class" unit="milling / turning / sawing / inspection" read=|f| &f.resources.process_class write=|f, v| f.resources.process_class = v />
                                <ExactInput form label="Envelope X" unit="mm" read=|f| &f.resources.machine_envelope_x_mm write=|f, v| f.resources.machine_envelope_x_mm = v />
                                <ExactInput form label="Envelope Y" unit="mm" read=|f| &f.resources.machine_envelope_y_mm write=|f, v| f.resources.machine_envelope_y_mm = v />
                                <ExactInput form label="Envelope Z" unit="mm" read=|f| &f.resources.machine_envelope_z_mm write=|f, v| f.resources.machine_envelope_z_mm = v />
                                <TextInput form label="Capability source" read=|f| &f.resources.machine_source write=|f, v| f.resources.machine_source = v />
                            </div>
                        </fieldset>

                        <fieldset>
                            <legend>"Coarse runtime profile"</legend>
                            <p class="fieldset-note">"This is versioned coarse-volumetric input, not CAM simulation or a machine-cycle guarantee."</p>
                            <div class="form-grid">
                                <ExactInput form label="Runtime-profile ID" unit="ID" read=|f| &f.resources.runtime_profile_id write=|f, v| f.resources.runtime_profile_id = v />
                                <ExactInput form label="Runtime-profile version" unit="version" read=|f| &f.resources.runtime_profile_version write=|f, v| f.resources.runtime_profile_version = v />
                                <ExactInput form label="Removal rate" unit="mm³/min" read=|f| &f.resources.removal_rate_mm3_per_minute write=|f, v| f.resources.removal_rate_mm3_per_minute = v />
                                <ExactInput form label="Setup" unit="min" read=|f| &f.resources.setup_minutes write=|f, v| f.resources.setup_minutes = v />
                                <ExactInput form label="Programming" unit="min" read=|f| &f.resources.programming_minutes write=|f, v| f.resources.programming_minutes = v />
                                <ExactInput form label="Load / unload" unit="min/item" read=|f| &f.resources.load_unload_minutes write=|f, v| f.resources.load_unload_minutes = v />
                                <ExactInput form label="Inspection" unit="min/lot" read=|f| &f.resources.inspection_minutes write=|f, v| f.resources.inspection_minutes = v />
                                <TextInput form label="Runtime source" read=|f| &f.resources.runtime_source write=|f, v| f.resources.runtime_source = v />
                            </div>
                            <ReviewCheckbox form label="I reviewed these material, stock, machine, and runtime values for this test." read=|f| f.resources.confirmed_for_draft write=|f, v| f.resources.confirmed_for_draft = v />
                        </fieldset>
                    </div>
                </Show>
                    </section>
                </Show>

                <fieldset>
                    <legend>"Change record"</legend>
                    <p class="fieldset-note">"Each save appends an immutable revision with an actor, host timestamp, and reason. Saving does not approve a quote or activate production authority."</p>
                    <div class="form-grid">
                        <TextInput form label="Changed by" read=|f| &f.settings_changed_by write=|f, v| f.settings_changed_by = v />
                        <TextInput form label="Reason for change" read=|f| &f.settings_change_reason write=|f, v| f.settings_change_reason = v />
                    </div>
                </fieldset>

                <div class="settings-footer">
                    <p id="settings-save-status" aria-live="polite">
                        {move || if settings_state.get().resource_catalog().is_some() {
                            "Catalog changes: none. Save unavailable—trusted operator identity is not configured. The current catalog remains unchanged."
                        } else if form.with(DeveloperEstimateForm::session_settings_ready) {
                            "Rate and pricing settings are confirmed for this session."
                        } else {
                            "Complete required rate/pricing fields and confirmations. If included, the starter resource bundle must also be complete."
                        }}
                    </p>
                    <Show when=move || matches!(settings_state.get(), SettingsPanelState::Failed { .. })>
                        <button
                            type="button"
                            class="secondary-action"
                            on:click=move |_| load_settings.run(())
                        >
                            "Reload saved draft"
                        </button>
                    </Show>
                    <button
                        type="button"
                        class="secondary-action"
                        aria-describedby="settings-save-status"
                        disabled=move || {
                            matches!(settings_state.get(), SettingsPanelState::Loading | SettingsPanelState::Saving)
                                || settings_state.get().resource_catalog().is_some()
                                || !form.with(DeveloperEstimateForm::settings_save_ready)
                        }
                        on:click=move |_| save_settings.run(())
                    >
                        {move || if settings_state.get().resource_catalog().is_some() {
                            "Save catalog draft"
                        } else if matches!(settings_state.get(), SettingsPanelState::Saving) {
                            "Saving settings"
                        } else {
                            "Save test settings"
                        }}
                    </button>
                    <button
                        type="button"
                        class="primary-action"
                        on:click=move |_| set_active_view.set(WorkspaceView::Estimate)
                    >
                        "Back to estimate"
                    </button>
                </div>
            </section>
        </main>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CatalogCategory {
    Materials,
    Offers,
    Stock,
    Machines,
    Runtime,
}

impl CatalogCategory {
    const fn label(self) -> &'static str {
        match self {
            Self::Materials => "Materials",
            Self::Offers => "Offers",
            Self::Stock => "Stock",
            Self::Machines => "Machines",
            Self::Runtime => "Runtime",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CatalogReviewRecord {
    key: String,
    title: String,
    subtitle: String,
    version: u32,
    state: ShopResourceRecordState,
    fields: Vec<(&'static str, String)>,
}

#[component]
fn ResourceCatalogReview(catalog: ShopResourceCatalogSnapshot) -> impl IntoView {
    let catalog = RwSignal::new(catalog);
    let (category, set_category) = signal(CatalogCategory::Materials);
    let (query, set_query) = signal(String::new());
    let (selected_key, set_selected_key) = signal(String::new());
    let choose_category = move |next| {
        set_category.set(next);
        set_query.set(String::new());
        set_selected_key.set(String::new());
    };

    let active_selection = catalog.with(|catalog| {
        catalog
            .selections
            .iter()
            .find(|selection| selection.state == ShopResourceSelectionState::ActiveForProposals)
            .cloned()
    });
    let selection_count = catalog.with(|catalog| catalog.selections.len());
    let counts = catalog.with(|catalog| {
        format!(
            "{} materials · {} offers · {} stock policies · {} machines · {} runtime profiles · {} selections",
            catalog.materials.len(),
            catalog.material_offers.len(),
            catalog.stock_allowances.len(),
            catalog.machines.len(),
            catalog.runtimes.len(),
            selection_count,
        )
    });
    let catalog_id = catalog.with(|catalog| catalog.catalog_id.clone());
    let catalog_version = catalog.with(|catalog| catalog.catalog_version);
    let currency = catalog.with(|catalog| catalog.currency.clone());

    let category_button = move |candidate: CatalogCategory, count: usize| {
        view! {
            <button
                type="button"
                class=move || if category.get() == candidate {
                    "catalog-category active"
                } else {
                    "catalog-category"
                }
                aria-pressed=move || category.get() == candidate
                on:click=move |_| choose_category(candidate)
            >
                <span>{candidate.label()}</span>
                <span class="catalog-category-count">{count}</span>
            </button>
        }
    };

    let material_count = catalog.with(|catalog| catalog.materials.len());
    let offer_count = catalog.with(|catalog| catalog.material_offers.len());
    let stock_count = catalog.with(|catalog| catalog.stock_allowances.len());
    let machine_count = catalog.with(|catalog| catalog.machines.len());
    let runtime_count = catalog.with(|catalog| catalog.runtimes.len());

    view! {
        <section class="resource-catalog-review" aria-labelledby="resource-catalog-heading">
            <div class="panel-heading compact-heading">
                <div>
                    <p class="section-index">"RESOURCE CATALOG / READ ONLY"</p>
                    <h3 id="resource-catalog-heading">"Governed proposal inputs"</h3>
                </div>
                <span class="status-chip blocked">"Review only"</span>
            </div>
            <p class="fieldset-note">
                "This immutable catalog is available for review. Editing is not enabled in this developer build because no trusted operator identity is configured. An active selection is proposal input—not estimate, routing, purchasing, or quote authority."
            </p>
            <dl class="catalog-summary">
                <div><dt>"Catalog"</dt><dd>{catalog_id} " · v" {catalog_version}</dd></div>
                <div><dt>"Currency"</dt><dd>{currency}</dd></div>
                <div><dt>"Contents"</dt><dd>{counts}</dd></div>
            </dl>
            <div class="catalog-browser">
                <nav class="catalog-categories" aria-label="Resource categories">
                    {category_button(CatalogCategory::Materials, material_count)}
                    {category_button(CatalogCategory::Offers, offer_count)}
                    {category_button(CatalogCategory::Stock, stock_count)}
                    {category_button(CatalogCategory::Machines, machine_count)}
                    {category_button(CatalogCategory::Runtime, runtime_count)}
                </nav>
                <section class="catalog-records" aria-labelledby="catalog-records-heading">
                    <div class="catalog-records-heading">
                        <h4 id="catalog-records-heading">{move || category.get().label()}</h4>
                        <label>
                            <span class="visually-hidden">"Search current resource category"</span>
                            <input
                                type="search"
                                placeholder="Search records"
                                prop:value=move || query.get()
                                on:input=move |event| {
                                    set_query.set(event_target_value(&event));
                                    set_selected_key.set(String::new());
                                }
                            />
                        </label>
                    </div>
                    <p class="catalog-result-count" aria-live="polite">
                        {move || {
                            let count = catalog.with(|catalog| {
                                filtered_catalog_records(catalog, category.get(), &query.get()).len()
                            });
                            format!("{count} matching records")
                        }}
                    </p>
                    <div class="catalog-record-list" aria-label="Resource records">
                        {move || {
                            let records = catalog.with(|catalog| {
                                filtered_catalog_records(catalog, category.get(), &query.get())
                            });
                            let selected = selected_catalog_key(&records, &selected_key.get());
                            if records.is_empty() {
                                view! {
                                    <p class="catalog-empty">"No records match this search."</p>
                                }.into_any()
                            } else {
                                records.into_iter().map(|record| {
                                    let key = record.key.clone();
                                    let click_key = key.clone();
                                    let is_selected = selected.as_deref() == Some(key.as_str());
                                    view! {
                                        <button
                                            type="button"
                                            class=if is_selected { "catalog-record selected" } else { "catalog-record" }
                                            aria-pressed=is_selected
                                            on:click=move |_| set_selected_key.set(click_key.clone())
                                        >
                                            <span class="catalog-record-title">{record.title}</span>
                                            <span>{record.subtitle}</span>
                                            <span class="catalog-record-meta">
                                                "v" {record.version} " · " {record_state_label(record.state)}
                                            </span>
                                        </button>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </div>
                </section>
                <section class="catalog-inspector" aria-labelledby="catalog-inspector-heading">
                    {move || {
                        let records = catalog.with(|catalog| {
                            filtered_catalog_records(catalog, category.get(), &query.get())
                        });
                        let selected = selected_catalog_key(&records, &selected_key.get());
                        records.into_iter().find(|record| Some(record.key.as_str()) == selected.as_deref())
                            .map(|record| view! {
                                <div>
                                    <p class="section-index">"SELECTED RECORD / REVIEW ONLY"</p>
                                    <h4 id="catalog-inspector-heading">{record.title}</h4>
                                    <p class="catalog-inspector-state">
                                        {record_state_label(record.state)} " · immutable version " {record.version}
                                    </p>
                                    <dl class="catalog-record-fields">
                                        {record.fields.into_iter().map(|(label, value)| view! {
                                            <div><dt>{label}</dt><dd>{value}</dd></div>
                                        }).collect_view()}
                                    </dl>
                                    <section
                                        class="catalog-editor-boundary"
                                        aria-labelledby="catalog-editor-heading"
                                    >
                                        <p class="section-index">"DRAFT SUCCESSOR"</p>
                                        <h5 id="catalog-editor-heading">"Editing unavailable"</h5>
                                        <p id="catalog-editor-unavailable-reason">
                                            "A trusted native operator identity is not configured. This immutable record remains unchanged, and no local successor is staged."
                                        </p>
                                        <button
                                            type="button"
                                            class="secondary-action"
                                            disabled=true
                                            aria-describedby="catalog-editor-unavailable-reason"
                                        >
                                            "Prepare successor draft"
                                        </button>
                                    </section>
                                </div>
                            }.into_any()).unwrap_or_else(|| view! {
                                <div>
                                    <p class="section-index">"SELECTED RECORD"</p>
                                    <h4 id="catalog-inspector-heading">"Nothing selected"</h4>
                                    <p class="fieldset-note">"Choose a record or clear the search."</p>
                                </div>
                            }.into_any())
                    }}
                </section>
            </div>
            {match active_selection {
                Some(selection) => view! {
                    <article class="catalog-selection" aria-label="Active proposal selection">
                        <p class="blocked-title">"Active for proposals: " {selection.selection_id} " · v" {selection.selection_version}</p>
                        <p>
                            "Material " {selection.material_id} " v" {selection.material_version}
                            " · Offer " {selection.material_offer_id} " v" {selection.material_offer_version}
                            " · Stock " {selection.stock_allowance_id} " v" {selection.stock_allowance_version}
                            " · Machine " {selection.machine_id} " v" {selection.machine_version}
                            " · Runtime " {selection.runtime_id} " v" {selection.runtime_version}
                        </p>
                        <p>"Reviewed by " {selection.decided_by} " at " {selection.decided_at} ". " {selection.reason}</p>
                    </article>
                }.into_any(),
                None => view! {
                    <article class="catalog-selection" aria-label="No active proposal selection">
                        <p class="blocked-title">"No active proposal selection"</p>
                        <p>"The catalog is retained, but nothing is currently selected for proposal generation."</p>
                    </article>
                }.into_any(),
            }}
        </section>
    }
}

fn selected_catalog_key(records: &[CatalogReviewRecord], requested: &str) -> Option<String> {
    records
        .iter()
        .find(|record| record.key == requested)
        .or_else(|| records.first())
        .map(|record| record.key.clone())
}

fn filtered_catalog_records(
    catalog: &ShopResourceCatalogSnapshot,
    category: CatalogCategory,
    query: &str,
) -> Vec<CatalogReviewRecord> {
    let query = query.trim().to_lowercase();
    catalog_review_records(catalog, category)
        .into_iter()
        .filter(|record| {
            query.is_empty()
                || record.title.to_lowercase().contains(&query)
                || record.subtitle.to_lowercase().contains(&query)
                || record.key.to_lowercase().contains(&query)
        })
        .collect()
}

fn record_state_label(state: ShopResourceRecordState) -> &'static str {
    match state {
        ShopResourceRecordState::Draft => "Draft",
        ShopResourceRecordState::Reviewed => "Reviewed",
        ShopResourceRecordState::Approved => "Approved",
        ShopResourceRecordState::Retired => "Retired",
        ShopResourceRecordState::Superseded => "Superseded",
    }
}

fn catalog_review_records(
    catalog: &ShopResourceCatalogSnapshot,
    category: CatalogCategory,
) -> Vec<CatalogReviewRecord> {
    match category {
        CatalogCategory::Materials => catalog
            .materials
            .iter()
            .map(|record| CatalogReviewRecord {
                key: format!("{}:v{}", record.material_id, record.material_version),
                title: format!("{} {}", record.family, record.grade),
                subtitle: record.material_id.clone(),
                version: record.material_version,
                state: record.state,
                fields: vec![
                    ("Material ID", record.material_id.clone()),
                    ("Family", record.family.clone()),
                    ("Grade", record.grade.clone()),
                    (
                        "Specification",
                        optional_text(record.specification.as_deref()),
                    ),
                    ("Condition", optional_text(record.condition.as_deref())),
                    ("Density", format!("{} kg/m³", record.density_kg_per_m3)),
                    ("Source", record.source_id.clone()),
                ],
            })
            .collect(),
        CatalogCategory::Offers => catalog
            .material_offers
            .iter()
            .map(|record| CatalogReviewRecord {
                key: format!("{}:v{}", record.offer_id, record.offer_version),
                title: record.supplier.clone(),
                subtitle: format!("{} · material {}", record.offer_id, record.material_id),
                version: record.offer_version,
                state: record.state,
                fields: vec![
                    ("Offer ID", record.offer_id.clone()),
                    (
                        "Material",
                        format!("{} v{}", record.material_id, record.material_version),
                    ),
                    (
                        "Price",
                        format!("{} {}/kg", record.currency, record.price_per_kg),
                    ),
                    ("Effective", record.effective_from.clone()),
                    ("Source", record.source_id.clone()),
                ],
            })
            .collect(),
        CatalogCategory::Stock => catalog
            .stock_allowances
            .iter()
            .map(|record| CatalogReviewRecord {
                key: format!(
                    "{}:v{}",
                    record.stock_allowance_id, record.stock_allowance_version
                ),
                title: record.stock_form.clone(),
                subtitle: record.stock_allowance_id.clone(),
                version: record.stock_allowance_version,
                state: record.state,
                fields: vec![
                    ("Stock policy ID", record.stock_allowance_id.clone()),
                    ("Form", record.stock_form.clone()),
                    ("X allowance", format!("{} mm", record.x_allowance_mm)),
                    ("Y allowance", format!("{} mm", record.y_allowance_mm)),
                    ("Z allowance", format!("{} mm", record.z_allowance_mm)),
                    ("Source", record.source_id.clone()),
                ],
            })
            .collect(),
        CatalogCategory::Machines => catalog
            .machines
            .iter()
            .map(|record| CatalogReviewRecord {
                key: format!("{}:v{}", record.machine_id, record.machine_version),
                title: record.name.clone(),
                subtitle: format!("{} · {}", record.machine_id, record.process_class),
                version: record.machine_version,
                state: record.state,
                fields: vec![
                    ("Machine ID", record.machine_id.clone()),
                    ("Process", record.process_class.clone()),
                    ("Envelope X", format!("{} mm", record.envelope_x_mm)),
                    ("Envelope Y", format!("{} mm", record.envelope_y_mm)),
                    ("Envelope Z", format!("{} mm", record.envelope_z_mm)),
                    ("Source", record.source_id.clone()),
                ],
            })
            .collect(),
        CatalogCategory::Runtime => catalog
            .runtimes
            .iter()
            .map(|record| CatalogReviewRecord {
                key: format!("{}:v{}", record.runtime_id, record.runtime_version),
                title: record.runtime_id.clone(),
                subtitle: format!("{} · {}", record.machine_id, record.material_id),
                version: record.runtime_version,
                state: record.state,
                fields: vec![
                    (
                        "Machine",
                        format!("{} v{}", record.machine_id, record.machine_version),
                    ),
                    (
                        "Material",
                        format!("{} v{}", record.material_id, record.material_version),
                    ),
                    (
                        "Removal rate",
                        format!("{} mm³/min", record.removal_rate_mm3_per_minute),
                    ),
                    ("Setup", format!("{} min", record.setup_minutes)),
                    ("Programming", format!("{} min", record.programming_minutes)),
                    (
                        "Load / unload",
                        format!("{} min/item", record.load_unload_minutes),
                    ),
                    (
                        "Inspection",
                        format!("{} min/lot", record.inspection_minutes),
                    ),
                    ("Source", record.source_id.clone()),
                ],
            })
            .collect(),
    }
}

fn optional_text(value: Option<&str>) -> String {
    value
        .filter(|value| !value.is_empty())
        .unwrap_or("Not recorded")
        .to_owned()
}

#[component]
fn TextInput(
    form: RwSignal<DeveloperEstimateForm>,
    label: &'static str,
    read: fn(&DeveloperEstimateForm) -> &String,
    write: fn(&mut DeveloperEstimateForm, String),
) -> impl IntoView {
    view! {
        <label class="form-field">
            <span>{label}</span>
            <input
                type="text"
                required
                prop:value=move || form.with(|form| read(form).clone())
                on:input=move |event| write(&mut form.write(), event_target_value(&event))
            />
        </label>
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
                let proposal_adoption = result.proposal_adoption.clone();
                let result_kind = if proposal_adoption.is_some() {
                    "TEST ESTIMATE · NOT A QUOTE"
                } else {
                    "MANUAL TEST ESTIMATE · NOT A QUOTE"
                };
                view! {
                    <section class="estimate-result" aria-labelledby="estimate-result-heading" aria-live="polite">
                        <div class="result-total">
                            <div>
                                <p class="section-index">{result_kind}</p>
                                <h3 id="estimate-result-heading">"Estimated selling price"</h3>
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
                            {proposal_adoption.map(|adoption| {
                                let excluded = adoption.excluded_inputs.join(", ");
                                view! {
                                    <div class="proposal-result-trace">
                                        <p><strong>"Proposal basis: "</strong>{adoption.library_id}" v"{adoption.library_version}", Settings revision "{adoption.settings_revision}</p>
                                        <p><strong>"Rules: "</strong>{adoption.proposal_rule_id}" v"{adoption.proposal_rule_version}"; "{adoption.adoption_rule_id}" v"{adoption.adoption_rule_version}</p>
                                        <p><strong>"Review: "</strong>{adoption.reviewed_by}" at "{adoption.reviewed_at}" — "{adoption.review_reason}</p>
                                        <p><strong>"Explicitly excluded: "</strong>{excluded}</p>
                                    </div>
                                }
                            })}
                        </details>
                        <p class="evidence-note">"Internal test estimate—not a customer quote. Detailed tooling, outside processing, overhead, risk, and CAM cycle analysis are not included."</p>
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
                <p class="blocked-title">"Check the estimate inputs"</p>
                <p>{error.message}</p>
                <p class="diagnostic-id">"Diagnostic: " {error.diagnostic_id}</p>
            </section>
        }
        .into_any(),
        DraftEstimatePanelState::Evaluating => view! {
            <section class="analysis-progress" role="status">
                <p class="blocked-title">"Calculating estimate"</p>
                <p>"Checking inputs, rates, and pricing."</p>
            </section>
        }
        .into_any(),
        DraftEstimatePanelState::NotReady => ().into_any(),
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn huntsville_test_profile_is_versioned_complete_and_requires_review() {
        let mut form = DeveloperEstimateForm::new();

        form.load_huntsville_test_values();

        assert_eq!(form.rate_card_id, "huntsville-2026q3-test-rates");
        assert_eq!(form.setup_labor_per_hour, "47.00");
        assert_eq!(form.programming_per_hour, "50.90");
        assert_eq!(form.run_labor_per_hour, "47.50");
        assert_eq!(form.machine_per_hour, "45.00");
        assert_eq!(form.quality_inspection_per_hour, "37.00");
        assert_eq!(form.markup_rate, "0.25");
        assert!(form.resources.enabled);
        assert_eq!(form.resources.material_price_per_kg, "10.00");
        assert_eq!(form.resources.stock_allowance_x_mm, "6.4");
        assert_eq!(form.resources.stock_allowance_y_mm, "6.4");
        assert_eq!(form.resources.stock_allowance_z_mm, "6.4");
        assert_eq!(form.resources.removal_rate_mm3_per_minute, "250000");
        assert_eq!(form.resources.machine_envelope_y_mm, "406");
        assert!(!form.rates_confirmed);
        assert!(!form.pricing_confirmed);
        assert!(!form.resources.confirmed_for_draft);
        assert!(!form.session_settings_ready());
    }

    #[test]
    fn loading_explicitly_missing_libraries_clears_stale_form_values() {
        let mut form = DeveloperEstimateForm::new();
        form.setup_labor_per_hour = "75".to_owned();
        form.markup_rate = "0.25".to_owned();
        form.resources.enabled = true;
        form.resources.library_id = "stale-resources".to_owned();
        let settings = ShopSettingsSnapshot {
            revision: 2,
            currency: "USD".to_owned(),
            rates: None,
            pricing: None,
            resources: None,
            resource_catalog: None,
            changed_by: "operator".to_owned(),
            changed_at: "2026-09-09T12:00:00Z".to_owned(),
            change_reason: "removed optional libraries".to_owned(),
        };

        form.apply_settings(&settings);

        assert!(form.rate_card_id.is_empty());
        assert!(form.setup_labor_per_hour.is_empty());
        assert!(form.pricing_policy_id.is_empty());
        assert!(form.markup_rate.is_empty());
        assert!(!form.resources.enabled);
        assert!(!form.rates_confirmed);
        assert!(!form.pricing_confirmed);
    }

    #[test]
    fn catalog_browser_maps_material_evidence_and_falls_back_to_first_record() {
        let catalog = ShopResourceCatalogSnapshot {
            catalog_id: "test-catalog".to_owned(),
            catalog_version: 3,
            currency: "USD".to_owned(),
            materials: vec![partprobe_desktop_contract::ShopMaterialSnapshot {
                material_id: "al-6061-t6".to_owned(),
                material_version: 2,
                family: "Aluminum".to_owned(),
                grade: "6061".to_owned(),
                specification: Some("ASTM B221".to_owned()),
                condition: Some("T6".to_owned()),
                density_kg_per_m3: "2700".to_owned(),
                source_id: "reviewed-handbook".to_owned(),
                state: ShopResourceRecordState::Reviewed,
            }],
            material_offers: Vec::new(),
            stock_allowances: Vec::new(),
            machines: Vec::new(),
            runtimes: Vec::new(),
            selections: Vec::new(),
        };

        let records = catalog_review_records(&catalog, CatalogCategory::Materials);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].key, "al-6061-t6:v2");
        assert_eq!(records[0].title, "Aluminum 6061");
        assert_eq!(records[0].version, 2);
        assert_eq!(records[0].state, ShopResourceRecordState::Reviewed);
        assert!(
            records[0]
                .fields
                .contains(&("Density", "2700 kg/m³".to_owned(),))
        );
        assert_eq!(
            selected_catalog_key(&records, "missing-record"),
            Some("al-6061-t6:v2".to_owned())
        );
        assert_eq!(
            filtered_catalog_records(&catalog, CatalogCategory::Materials, "6061").len(),
            1
        );
        assert!(filtered_catalog_records(&catalog, CatalogCategory::Materials, "steel").is_empty());
    }
}
