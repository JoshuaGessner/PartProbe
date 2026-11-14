#![forbid(unsafe_code)]

use partprobe_desktop_contract::{
    DeveloperPricingInputFields, DeveloperRateInputFields, DraftEstimateEvaluation,
    GeometryConfidenceLevel, HostCommandError, HostErrorCode, ModelAnalysisResult, ModelLengthUnit,
    ModelSourceFormat, ModelSourceSelection, ModelViewerStandardView, ProvisionalGeometryFacts,
    SaveShopSettingsRequest, SelectedModelSource, ShopResourceInputFields,
};

#[must_use]
pub const fn model_view_label(view: ModelViewerStandardView) -> &'static str {
    match view {
        ModelViewerStandardView::Isometric => "Isometric",
        ModelViewerStandardView::Front => "Front",
        ModelViewerStandardView::Top => "Top",
        ModelViewerStandardView::Right => "Right",
    }
}

#[must_use]
pub fn selected_source_accessible_label(display_name: &str) -> String {
    format!("Selected model source: {display_name}")
}

#[must_use]
pub fn provisional_geometry_accessible_label(geometry: &ProvisionalGeometryFacts) -> String {
    match geometry {
        ProvisionalGeometryFacts::ExactBrep(facts) => {
            let bounds = facts
                .aabb_extents_mm
                .as_ref()
                .map_or_else(|| "unavailable".to_owned(), |value| value.join(", "));
            format!(
                "Provisional exact B-rep geometry available: source-axis bounds {bounds} \
                 millimeters; surface area {} square millimeters; enclosed volume {} cubic \
                 millimeters; centroid {} millimeters; engine {}",
                facts.surface_area_mm2,
                facts.enclosed_volume_mm3,
                facts.center_of_mass_mm.join(", "),
                facts.geometry_engine,
            )
        }
        ProvisionalGeometryFacts::Mesh(facts) => {
            let format = source_format_label(facts.detected_format);
            let units = length_unit_label(facts.source_units);
            let volume = facts
                .enclosed_volume
                .as_deref()
                .map_or("withheld".to_owned(), |value| value.to_owned());
            format!(
                "Provisional mesh geometry available: detected format {format}; source units \
                 {units}; surface area {}; enclosed volume {volume}; confidence {}; warnings {}",
                facts.surface_area,
                confidence_level_label(facts.confidence_level),
                facts.warning_codes.len(),
            )
        }
    }
}

#[must_use]
pub const fn confidence_level_label(level: GeometryConfidenceLevel) -> &'static str {
    match level {
        GeometryConfidenceLevel::High => "high",
        GeometryConfidenceLevel::Medium => "medium",
        GeometryConfidenceLevel::Low => "low",
        GeometryConfidenceLevel::NeedsReview => "needs review",
    }
}

#[must_use]
pub const fn source_format_label(format: ModelSourceFormat) -> &'static str {
    match format {
        ModelSourceFormat::Step => "STEP",
        ModelSourceFormat::Stl => "STL",
        ModelSourceFormat::ThreeMf => "3MF",
    }
}

#[must_use]
pub const fn length_unit_label(unit: ModelLengthUnit) -> &'static str {
    match unit {
        ModelLengthUnit::Micrometer => "micrometer",
        ModelLengthUnit::Millimeter => "millimeter",
        ModelLengthUnit::Centimeter => "centimeter",
        ModelLengthUnit::Meter => "meter",
        ModelLengthUnit::Inch => "inch",
        ModelLengthUnit::Foot => "foot",
        ModelLengthUnit::Unknown => "unknown; confirmation required",
    }
}

#[must_use]
pub fn provisional_analysis_failure_accessible_label(diagnostic_id: &str) -> String {
    format!("Analysis could not be completed: diagnostic {diagnostic_id}")
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GeometryReviewConfirmation {
    pub canonical_units_reviewed: bool,
    pub warnings_reviewed: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
impl GeometryReviewConfirmation {
    pub const fn clear(&mut self) {
        self.canonical_units_reviewed = false;
        self.warnings_reviewed = false;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ModelPanelState {
    #[default]
    Empty,
    Selected(SelectedModelSource),
    Failed,
}

impl ModelPanelState {
    #[must_use]
    pub const fn status_heading(&self) -> &'static str {
        match self {
            Self::Empty => "No model selected",
            Self::Selected(_) => "Model selected",
            Self::Failed => "Model selection failed",
        }
    }

    #[must_use]
    pub const fn status_detail(&self) -> &'static str {
        match self {
            Self::Empty => {
                "Choose a local STEP model to begin. STL and 3MF files can be analyzed, but not estimated yet. Files stay on this computer."
            }
            Self::Selected(_) => {
                "Available for local analysis. This file is available only for this session and is not saved."
            }
            Self::Failed => {
                "No model was retained. Choose another local file or restart the session."
            }
        }
    }

    pub fn apply_selection(&mut self, selection: ModelSourceSelection) {
        if let ModelSourceSelection::Selected { source } = selection {
            *self = Self::Selected(source);
        }
    }

    #[must_use]
    pub const fn selected_source(&self) -> Option<&SelectedModelSource> {
        match self {
            Self::Selected(source) => Some(source),
            Self::Empty | Self::Failed => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AnalysisPanelState {
    #[default]
    NotStarted,
    Running,
    Cancelling,
    Cancelled,
    Available(Box<ModelAnalysisResult>),
    Failed(HostCommandError),
}

impl AnalysisPanelState {
    #[must_use]
    pub const fn status_heading(&self) -> &'static str {
        match self {
            Self::NotStarted => "Provisional analysis not started",
            Self::Running => "Analyzing in the isolated worker",
            Self::Cancelling => "Requesting analysis cancellation",
            Self::Cancelled => "Provisional analysis cancelled",
            Self::Available(_) => "Provisional geometry available",
            Self::Failed(_) => "Analysis could not be completed",
        }
    }

    #[must_use]
    pub fn status_detail(&self) -> &str {
        match self {
            Self::NotStarted => {
                "Analysis requires a locally configured developer worker and does not save results."
            }
            Self::Running => {
                "The native application service is authorizing, hashing, and analyzing the selected source."
            }
            Self::Cancelling => {
                "The native host has signalled the active worker and is waiting for bounded cleanup."
            }
            Self::Cancelled => {
                "The selected source remains available. Retry analysis when you are ready."
            }
            Self::Available(result) => &result.estimate.reason,
            Self::Failed(error) => &error.message,
        }
    }

    #[must_use]
    pub fn from_host_error(error: HostCommandError) -> Self {
        if error.code == HostErrorCode::AnalysisCancelled {
            Self::Cancelled
        } else {
            Self::Failed(error)
        }
    }
}

#[must_use]
pub fn analysis_supports_draft_estimate(state: &AnalysisPanelState) -> bool {
    matches!(
        state,
        AnalysisPanelState::Available(result)
            if matches!(result.geometry, ProvisionalGeometryFacts::ExactBrep(_))
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DraftEstimatePanelState {
    #[default]
    NotReady,
    Evaluating,
    Evaluated(Box<DraftEstimateEvaluation>),
    Failed(HostCommandError),
}

impl DraftEstimatePanelState {
    #[must_use]
    pub const fn status_heading(&self) -> &'static str {
        match self {
            Self::NotReady => "Complete the governed estimate inputs",
            Self::Evaluating => "Evaluating deterministic estimate",
            Self::Evaluated(_) => "Draft estimate evaluated",
            Self::Failed(_) => "Draft estimate remains unavailable",
        }
    }

    #[must_use]
    pub fn status_detail(&self) -> &str {
        match self {
            Self::NotReady => {
                "Review geometry, enter every manual value, and confirm session-only rates and pricing."
            }
            Self::Evaluating => {
                "The native application session is validating inputs, resolving rates, and applying pinned rules."
            }
            Self::Evaluated(evaluation) => evaluation
                .reason
                .as_deref()
                .unwrap_or("A session-only deterministic result and trace are available."),
            Self::Failed(error) => &error.message,
        }
    }
}

/// The field values that were loaded from the last saved settings revision. Record versions are
/// immutable, so a changed value needs the exact next version; the form derives that
/// automatically from this baseline instead of asking the user to manage version numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsFieldBaseline {
    pub rates: DeveloperRateInputFields,
    pub pricing: DeveloperPricingInputFields,
    pub resources: Option<ShopResourceInputFields>,
}

/// Bumps each record version whose governed content changed against the loaded baseline while
/// its version field was left untouched. A hand-edited version is respected, a non-numeric
/// version is left for the host to validate, and unchanged records keep their exact reused
/// evidence. This chooses identity bookkeeping only; the host still validates every value.
pub fn apply_automatic_record_versions(
    request: &mut SaveShopSettingsRequest,
    baseline: &SettingsFieldBaseline,
) {
    if rates_content_changed(&request.rates, &baseline.rates)
        && request.rates.rate_card_version == baseline.rates.rate_card_version
    {
        bump_version(&mut request.rates.rate_card_version);
    }
    if pricing_content_changed(&request.pricing, &baseline.pricing)
        && request.pricing.pricing_policy_version == baseline.pricing.pricing_policy_version
    {
        bump_version(&mut request.pricing.pricing_policy_version);
    }
    if let (Some(resources), Some(baseline_resources)) =
        (request.resources.as_mut(), baseline.resources.as_ref())
        && resources_content_changed(resources, baseline_resources)
    {
        // Child records cross-reference each other's versions (an offer pins its material
        // version), so one content change advances the complete bundle together. Reusing
        // unchanged bytes under a new version stays a valid immutable record.
        for (version, baseline_version) in [
            (
                &mut resources.library_version,
                &baseline_resources.library_version,
            ),
            (
                &mut resources.material_version,
                &baseline_resources.material_version,
            ),
            (
                &mut resources.offer_version,
                &baseline_resources.offer_version,
            ),
            (
                &mut resources.stock_profile_version,
                &baseline_resources.stock_profile_version,
            ),
            (
                &mut resources.machine_version,
                &baseline_resources.machine_version,
            ),
            (
                &mut resources.runtime_profile_version,
                &baseline_resources.runtime_profile_version,
            ),
        ] {
            if version == baseline_version {
                bump_version(version);
            }
        }
    }
}

fn bump_version(version: &mut String) {
    if let Ok(current) = version.trim().parse::<u32>()
        && let Some(next) = current.checked_add(1)
    {
        *version = next.to_string();
    }
}

fn rates_content_changed(
    current: &DeveloperRateInputFields,
    baseline: &DeveloperRateInputFields,
) -> bool {
    let content = |fields: &DeveloperRateInputFields| {
        [
            fields.rate_card_id.clone(),
            fields.effective_on.clone(),
            fields.currency.clone(),
            fields.setup_labor_per_hour.clone(),
            fields.programming_per_hour.clone(),
            fields.run_labor_per_hour.clone(),
            fields.machine_per_hour.clone(),
            fields.quality_inspection_per_hour.clone(),
        ]
    };
    content(current) != content(baseline)
}

fn pricing_content_changed(
    current: &DeveloperPricingInputFields,
    baseline: &DeveloperPricingInputFields,
) -> bool {
    let content = |fields: &DeveloperPricingInputFields| {
        [
            fields.pricing_policy_id.clone(),
            fields.markup_rate.clone(),
            fields.optional_price_floor.clone(),
            fields.optional_minimum_order.clone(),
            fields.rounding_decimal_places.clone(),
        ]
    };
    content(current) != content(baseline)
}

fn resources_content_changed(
    current: &ShopResourceInputFields,
    baseline: &ShopResourceInputFields,
) -> bool {
    let content = |fields: &ShopResourceInputFields| {
        [
            fields.library_id.clone(),
            fields.material_id.clone(),
            fields.material_family.clone(),
            fields.material_grade.clone(),
            fields.optional_material_specification.clone(),
            fields.optional_material_condition.clone(),
            fields.density_kg_per_m3.clone(),
            fields.material_source.clone(),
            fields.offer_id.clone(),
            fields.supplier.clone(),
            fields.material_price_per_kg.clone(),
            fields.offer_effective_on.clone(),
            fields.offer_source.clone(),
            fields.stock_profile_id.clone(),
            fields.stock_form.clone(),
            fields.stock_allowance_x_mm.clone(),
            fields.stock_allowance_y_mm.clone(),
            fields.stock_allowance_z_mm.clone(),
            fields.stock_source.clone(),
            fields.machine_id.clone(),
            fields.machine_name.clone(),
            fields.process_class.clone(),
            fields.machine_envelope_x_mm.clone(),
            fields.machine_envelope_y_mm.clone(),
            fields.machine_envelope_z_mm.clone(),
            fields.machine_source.clone(),
            fields.runtime_profile_id.clone(),
            fields.removal_rate_mm3_per_minute.clone(),
            fields.setup_minutes.clone(),
            fields.programming_minutes.clone(),
            fields.load_unload_minutes.clone(),
            fields.inspection_minutes.clone(),
            fields.runtime_source.clone(),
        ]
    };
    content(current) != content(baseline)
}

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::mount;

#[cfg(test)]
mod tests {
    use partprobe_desktop_contract::{
        AnalysisStatus, CanonicalLengthUnit, MeshMeasurementBasis, MeshSelfIntersectionState,
        MeshTopologyIdentity, MeshWeldingStatus, ModelSourceFormat, PersistenceAvailability,
        ProvisionalExactBrepFacts, ProvisionalMeshFacts, StlEncoding, UnitResolution,
    };

    use super::*;

    #[test]
    fn selected_source_never_implies_analysis_or_persistence() {
        let source = SelectedModelSource {
            selection_id: "selection-1".to_owned(),
            display_name: "fixture.step".to_owned(),
            format: ModelSourceFormat::Step,
            analysis_status: AnalysisStatus::NotStarted,
            persistence: PersistenceAvailability::SessionOnly,
        };
        let mut state = ModelPanelState::Empty;

        state.apply_selection(ModelSourceSelection::Selected {
            source: source.clone(),
        });

        assert_eq!(state, ModelPanelState::Selected(source));
        assert_eq!(state.status_heading(), "Model selected");
        assert!(state.status_detail().contains("session"));
        assert!(state.status_detail().contains("not saved"));
    }

    #[test]
    fn model_view_labels_are_explicit() {
        assert_eq!(
            [
                ModelViewerStandardView::Isometric,
                ModelViewerStandardView::Front,
                ModelViewerStandardView::Top,
                ModelViewerStandardView::Right,
            ]
            .map(model_view_label),
            ["Isometric", "Front", "Top", "Right"]
        );
    }

    #[test]
    fn selected_source_accessible_label_includes_the_display_name() {
        assert_eq!(
            selected_source_accessible_label("fixture.step"),
            "Selected model source: fixture.step"
        );
    }

    #[test]
    fn provisional_geometry_accessible_label_carries_exact_path_free_evidence() {
        let geometry = ProvisionalGeometryFacts::ExactBrep(ProvisionalExactBrepFacts {
            canonical_units: CanonicalLengthUnit::Millimeter,
            aabb_extents_mm: Some(["12".to_owned(), "8".to_owned(), "5".to_owned()]),
            surface_area_mm2: "392".to_owned(),
            enclosed_volume_mm3: "480".to_owned(),
            center_of_mass_mm: ["6".to_owned(), "4".to_owned(), "2.5".to_owned()],
            solid_body_count: 1,
            transferred_roots: 1,
            source_hash_sha256: "a".repeat(64),
            output_hash_sha256: "b".repeat(64),
            output_byte_length: 256,
            geometry_engine: "OCCT 8.0.0".to_owned(),
            adapter_abi_version: 3,
        });
        assert_eq!(
            provisional_geometry_accessible_label(&geometry),
            "Provisional exact B-rep geometry available: source-axis bounds 12, 8, 5 millimeters; surface area 392 square millimeters; enclosed volume \
             480 cubic millimeters; centroid 6, 4, 2.5 millimeters; engine OCCT 8.0.0"
        );
    }

    #[test]
    fn provisional_analysis_failure_label_carries_only_the_diagnostic() {
        assert_eq!(
            provisional_analysis_failure_accessible_label("GUI4-ANALYSIS-TEST"),
            "Analysis could not be completed: diagnostic GUI4-ANALYSIS-TEST"
        );
    }

    #[test]
    fn mesh_accessible_label_preserves_unknown_units_and_withheld_volume() {
        let geometry = ProvisionalGeometryFacts::Mesh(ProvisionalMeshFacts {
            detected_format: ModelSourceFormat::Stl,
            stl_encoding: Some(StlEncoding::Ascii),
            source_units: ModelLengthUnit::Unknown,
            unit_resolution: UnitResolution::Unresolved,
            unit_was_explicit: None,
            measurement_basis: MeshMeasurementBasis::SourceCoordinates,
            aabb_extents: ["10".to_owned(), "10".to_owned(), "0".to_owned()],
            surface_area: "100".to_owned(),
            enclosed_volume: None,
            center_of_mass: None,
            triangle_count: 2,
            manifold: true,
            watertight: false,
            consistently_wound: false,
            self_intersection: MeshSelfIntersectionState::NotDetected,
            confidence_level: GeometryConfidenceLevel::NeedsReview,
            confidence_reason_codes: vec!["MESH_UNITS_UNRESOLVED".to_owned()],
            algorithm_version: "stl-v3".to_owned(),
            self_intersection_algorithm_version: "intersection-v1".to_owned(),
            confidence_policy_version: "confidence-v1".to_owned(),
            topology_policy_version: "topology-v1".to_owned(),
            topology_identity: MeshTopologyIdentity::ExactSourceCoordinates,
            welding_status: MeshWeldingStatus::NotApplied,
            warning_codes: vec!["UNITS_MISSING_REQUIRES_CONFIRMATION".to_owned()],
            source_hash_sha256: "a".repeat(64),
            output_hash_sha256: "b".repeat(64),
            output_byte_length: 512,
        });

        let label = provisional_geometry_accessible_label(&geometry);
        assert!(label.contains("detected format STL"));
        assert!(label.contains("source units unknown; confirmation required"));
        assert!(label.contains("enclosed volume withheld"));
        assert!(!label.contains("cubic millimeters"));
        assert!(!label.contains("source path"));
    }

    #[test]
    fn cancellation_returns_to_an_explicit_empty_state() {
        let mut state = ModelPanelState::Empty;
        state.apply_selection(ModelSourceSelection::Cancelled);
        assert_eq!(state, ModelPanelState::Empty);
        assert_eq!(state.status_heading(), "No model selected");
    }

    #[test]
    fn cancellation_preserves_the_current_session_selection() {
        let source = SelectedModelSource {
            selection_id: "selection-1".to_owned(),
            display_name: "fixture.step".to_owned(),
            format: ModelSourceFormat::Step,
            analysis_status: AnalysisStatus::NotStarted,
            persistence: PersistenceAvailability::SessionOnly,
        };
        let mut state = ModelPanelState::Selected(source.clone());

        state.apply_selection(ModelSourceSelection::Cancelled);

        assert_eq!(state, ModelPanelState::Selected(source));
    }

    #[test]
    fn analysis_failure_remains_explicit_and_recoverable() {
        let state =
            AnalysisPanelState::Failed(HostCommandError::analysis_failed("GUI4-ANALYSIS-TEST"));

        assert!(state.status_heading().contains("could not be completed"));
        assert!(state.status_detail().contains("remains available"));
    }

    #[test]
    fn cancellation_state_explains_bounded_worker_cleanup() {
        let state = AnalysisPanelState::Cancelling;

        assert!(state.status_heading().contains("cancellation"));
        assert!(state.status_detail().contains("bounded cleanup"));
    }

    #[test]
    fn cancelled_host_result_is_not_presented_as_an_analysis_failure() {
        let state = AnalysisPanelState::from_host_error(HostCommandError::analysis_cancelled(
            "GUI4-ANALYSIS-CANCELLED-TEST",
        ));

        assert_eq!(state, AnalysisPanelState::Cancelled);
        assert!(state.status_heading().contains("cancelled"));
        assert!(state.status_detail().contains("Retry"));
    }

    #[test]
    fn rejected_estimate_inputs_never_imply_a_numeric_result() {
        let state = DraftEstimatePanelState::Failed(HostCommandError::invalid_estimate_input(
            "GUI4-ESTIMATE-TEST",
        ));

        assert!(state.status_heading().contains("unavailable"));
        assert!(state.status_detail().contains("missing or invalid"));
    }

    #[test]
    fn geometry_review_confirmation_is_explicitly_revision_bound() {
        let mut review = GeometryReviewConfirmation {
            canonical_units_reviewed: true,
            warnings_reviewed: true,
        };

        review.clear();

        assert_eq!(review, GeometryReviewConfirmation::default());
    }

    fn baseline_request() -> SaveShopSettingsRequest {
        SaveShopSettingsRequest {
            expected_revision: Some(1),
            changed_by: "local-test-operator".to_owned(),
            change_reason: "test".to_owned(),
            rates: DeveloperRateInputFields {
                confirmed_for_session: true,
                rate_card_id: "shop-rates".to_owned(),
                rate_card_version: "3".to_owned(),
                effective_on: "2026-09-11".to_owned(),
                currency: "USD".to_owned(),
                setup_labor_per_hour: "47.00".to_owned(),
                programming_per_hour: "50.90".to_owned(),
                run_labor_per_hour: "47.50".to_owned(),
                machine_per_hour: "45.00".to_owned(),
                quality_inspection_per_hour: "37.00".to_owned(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: true,
                pricing_policy_id: "shop-pricing".to_owned(),
                pricing_policy_version: "2".to_owned(),
                markup_rate: "0.25".to_owned(),
                optional_price_floor: String::new(),
                optional_minimum_order: String::new(),
                rounding_decimal_places: "2".to_owned(),
            },
            resources: Some(ShopResourceInputFields {
                confirmed_for_draft: true,
                library_id: "shop-resources".to_owned(),
                library_version: "1".to_owned(),
                material_id: "shop-material".to_owned(),
                material_version: "1".to_owned(),
                material_family: "Aluminum".to_owned(),
                material_grade: "6061".to_owned(),
                optional_material_specification: String::new(),
                optional_material_condition: "T651".to_owned(),
                density_kg_per_m3: "2700".to_owned(),
                material_source: "manual-settings-entry".to_owned(),
                offer_id: "shop-material-offer".to_owned(),
                offer_version: "1".to_owned(),
                supplier: "Manual entry".to_owned(),
                material_price_per_kg: "10.00".to_owned(),
                offer_effective_on: "2026-09-11".to_owned(),
                offer_source: "manual-settings-entry".to_owned(),
                stock_profile_id: "shop-stock-allowance".to_owned(),
                stock_profile_version: "1".to_owned(),
                stock_form: "rectangular".to_owned(),
                stock_allowance_x_mm: "6.4".to_owned(),
                stock_allowance_y_mm: "6.4".to_owned(),
                stock_allowance_z_mm: "6.4".to_owned(),
                stock_source: "manual-settings-entry".to_owned(),
                machine_id: "shop-machine".to_owned(),
                machine_version: "1".to_owned(),
                machine_name: "Primary machine".to_owned(),
                process_class: "milling".to_owned(),
                machine_envelope_x_mm: "762".to_owned(),
                machine_envelope_y_mm: "406".to_owned(),
                machine_envelope_z_mm: "508".to_owned(),
                machine_source: "manual-settings-entry".to_owned(),
                runtime_profile_id: "shop-runtime".to_owned(),
                runtime_profile_version: "1".to_owned(),
                removal_rate_mm3_per_minute: "250000".to_owned(),
                setup_minutes: "90".to_owned(),
                programming_minutes: "120".to_owned(),
                load_unload_minutes: "5".to_owned(),
                inspection_minutes: "15".to_owned(),
                runtime_source: "manual-settings-entry".to_owned(),
            }),
        }
    }

    fn baseline_of(request: &SaveShopSettingsRequest) -> SettingsFieldBaseline {
        SettingsFieldBaseline {
            rates: request.rates.clone(),
            pricing: request.pricing.clone(),
            resources: request.resources.clone(),
        }
    }

    #[test]
    fn unchanged_settings_reuse_every_exact_record_version() {
        let original = baseline_request();
        let baseline = baseline_of(&original);
        let mut request = original.clone();

        apply_automatic_record_versions(&mut request, &baseline);

        assert_eq!(request, original);
    }

    #[test]
    fn changed_rate_and_pricing_values_advance_only_their_own_versions() {
        let original = baseline_request();
        let baseline = baseline_of(&original);
        let mut request = original.clone();
        request.rates.machine_per_hour = "55.00".to_owned();
        request.pricing.markup_rate = "0.30".to_owned();

        apply_automatic_record_versions(&mut request, &baseline);

        assert_eq!(request.rates.rate_card_version, "4");
        assert_eq!(request.pricing.pricing_policy_version, "3");
        let resources = request.resources.expect("resources retained");
        let original_resources = original.resources.expect("baseline resources");
        assert_eq!(
            resources.library_version,
            original_resources.library_version
        );
        assert_eq!(
            resources.material_version,
            original_resources.material_version
        );
    }

    #[test]
    fn one_changed_resource_value_advances_the_complete_bundle_once() {
        let original = baseline_request();
        let baseline = baseline_of(&original);
        let mut request = original.clone();
        request
            .resources
            .as_mut()
            .expect("resources enabled")
            .density_kg_per_m3 = "2810".to_owned();

        apply_automatic_record_versions(&mut request, &baseline);

        assert_eq!(request.rates.rate_card_version, "3");
        assert_eq!(request.pricing.pricing_policy_version, "2");
        let resources = request.resources.expect("resources retained");
        assert_eq!(resources.library_version, "2");
        assert_eq!(resources.material_version, "2");
        assert_eq!(resources.offer_version, "2");
        assert_eq!(resources.stock_profile_version, "2");
        assert_eq!(resources.machine_version, "2");
        assert_eq!(resources.runtime_profile_version, "2");
    }

    #[test]
    fn hand_edited_or_non_numeric_versions_are_respected_not_overwritten() {
        let original = baseline_request();
        let baseline = baseline_of(&original);
        let mut request = original.clone();
        request.rates.setup_labor_per_hour = "48.00".to_owned();
        request.rates.rate_card_version = "9".to_owned();
        request.pricing.markup_rate = "0.30".to_owned();
        request.pricing.pricing_policy_version = "not-a-number".to_owned();

        apply_automatic_record_versions(&mut request, &baseline);

        assert_eq!(request.rates.rate_card_version, "9");
        assert_eq!(request.pricing.pricing_policy_version, "not-a-number");
    }
}
