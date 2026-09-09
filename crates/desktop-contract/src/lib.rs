#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DESKTOP_CONTRACT_VERSION: u16 = 6;
pub const COMMAND_DESKTOP_CONTRACT: &str = "desktop_contract";
pub const COMMAND_SELECT_MODEL_SOURCE: &str = "select_model_source";
pub const COMMAND_ANALYZE_MODEL_SOURCE: &str = "analyze_model_source";
pub const COMMAND_CANCEL_MODEL_ANALYSIS: &str = "cancel_model_analysis";
pub const COMMAND_EVALUATE_DRAFT_ESTIMATE: &str = "evaluate_draft_estimate";
pub const COMMAND_LOAD_SHOP_SETTINGS: &str = "load_shop_settings";
pub const COMMAND_SAVE_SHOP_SETTINGS: &str = "save_shop_settings";
pub const EVENT_MODEL_SOURCE_SELECTED: &str = "partprobe:model-source-selected";
pub const APPLICATION_COMMANDS: [&str; 7] = [
    COMMAND_DESKTOP_CONTRACT,
    COMMAND_SELECT_MODEL_SOURCE,
    COMMAND_ANALYZE_MODEL_SOURCE,
    COMMAND_CANCEL_MODEL_ANALYSIS,
    COMMAND_EVALUATE_DRAFT_ESTIMATE,
    COMMAND_LOAD_SHOP_SETTINGS,
    COMMAND_SAVE_SHOP_SETTINGS,
];
pub const APPLICATION_EVENTS: [&str; 1] = [EVENT_MODEL_SOURCE_SELECTED];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DesktopContract {
    pub contract_version: u16,
    pub commands: Vec<String>,
    pub events: Vec<String>,
    pub analysis_authority: AnalysisAuthority,
    pub persistence: PersistenceAvailability,
    pub shop_settings_persistence: ShopSettingsPersistenceAvailability,
}

impl DesktopContract {
    #[must_use]
    pub fn current() -> Self {
        Self {
            contract_version: DESKTOP_CONTRACT_VERSION,
            commands: APPLICATION_COMMANDS.map(str::to_owned).to_vec(),
            events: APPLICATION_EVENTS.map(str::to_owned).to_vec(),
            analysis_authority: AnalysisAuthority::NativeApplicationService,
            persistence: PersistenceAvailability::SessionOnly,
            shop_settings_persistence: ShopSettingsPersistenceAvailability::LocalDrafts,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisAuthority {
    NativeApplicationService,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceAvailability {
    SessionOnly,
}

/// Narrow durable boundary available only for non-authoritative shop-settings drafts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopSettingsPersistenceAvailability {
    LocalDrafts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ModelSourceSelection {
    Cancelled,
    Selected { source: SelectedModelSource },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectedModelSource {
    pub selection_id: String,
    pub display_name: String,
    pub format: ModelSourceFormat,
    pub analysis_status: AnalysisStatus,
    pub persistence: PersistenceAvailability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSourceFormat {
    Step,
    Stl,
    ThreeMf,
}

impl ModelSourceFormat {
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "step" | "stp" => Some(Self::Step),
            "stl" => Some(Self::Stl),
            "3mf" => Some(Self::ThreeMf),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    NotStarted,
    ProvisionalAvailable,
}

/// Path-free request to analyze the model retained behind one native session token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalyzeModelSourceRequest {
    pub selection_id: String,
}

/// Token-bound cancellation request; no source authority crosses the bridge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CancelModelAnalysisRequest {
    pub selection_id: String,
}

/// Explicit acknowledgement of whether a matching active analysis was signalled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisCancellationAcknowledgement {
    pub selection_id: String,
    pub cancellation_requested: bool,
}

/// Complete, exact-text, session-only request for the GUI-4 deterministic estimate path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvaluateDraftEstimateRequest {
    pub selection_id: String,
    pub analysis_id: String,
    pub review: GeometryReviewInput,
    pub inputs: DraftEstimateInputFields,
    pub rates: DeveloperRateInputFields,
    pub pricing: DeveloperPricingInputFields,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GeometryReviewInput {
    pub canonical_units_reviewed: bool,
    pub warnings_reviewed: bool,
}

/// Exact decimal and whole-number text parsed only by the trusted native adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DraftEstimateInputFields {
    pub stock_volume_mm3: String,
    pub density_kg_per_mm3: String,
    pub deliver_quantity: String,
    pub planned_spares: String,
    pub destructive_samples: String,
    pub setup_hours: String,
    pub programming_hours: String,
    pub cutting_hours_per_item: String,
    pub non_cutting_hours_per_item: String,
    pub load_unload_hours_per_item: String,
    pub in_cycle_inspection_hours_per_item: String,
    pub quality_inspection_hours: String,
    pub purchased_material: String,
    pub cut_charge: String,
    pub material_certificate: String,
    pub inbound_freight: String,
    pub approved_remnant_credit: String,
    pub prove_out: String,
    pub tooling: String,
    pub consumables: String,
    pub fixture: String,
    pub outside_processing: String,
    pub operation_freight: String,
    pub nonrecurring_engineering: String,
    pub administration: String,
    pub overhead: String,
    pub accepted_risk_impact: String,
    pub expected_rework: String,
}

/// Five required organization-scope hourly rates supplied and confirmed for this session only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeveloperRateInputFields {
    pub confirmed_for_session: bool,
    pub rate_card_id: String,
    pub rate_card_version: String,
    pub effective_on: String,
    pub currency: String,
    pub setup_labor_per_hour: String,
    pub programming_per_hour: String,
    pub run_labor_per_hour: String,
    pub machine_per_hour: String,
    pub quality_inspection_per_hour: String,
}

/// Explicit session pricing policy; it is never installed as a production default.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeveloperPricingInputFields {
    pub confirmed_for_session: bool,
    pub pricing_policy_id: String,
    pub pricing_policy_version: String,
    pub markup_rate: String,
    pub optional_price_floor: String,
    pub optional_minimum_order: String,
    pub rounding_decimal_places: String,
}

/// Host-owned durable Settings state. Absence is explicit and never filled with numeric defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ShopSettingsState {
    NotConfigured,
    Available { settings: Box<ShopSettingsSnapshot> },
}

/// Path-free current Settings revision returned by the typed application service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShopSettingsSnapshot {
    pub revision: u32,
    pub currency: String,
    pub rates: Option<DeveloperRateInputFields>,
    pub pricing: Option<DeveloperPricingInputFields>,
    pub resources: Option<ShopResourceInputFields>,
    pub changed_by: String,
    pub changed_at: String,
    pub change_reason: String,
}

/// Complete draft append request with explicit optimistic-concurrency and audit evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SaveShopSettingsRequest {
    pub expected_revision: Option<u32>,
    pub changed_by: String,
    pub change_reason: String,
    pub rates: DeveloperRateInputFields,
    pub pricing: DeveloperPricingInputFields,
    pub resources: Option<ShopResourceInputFields>,
}

/// Exact-text draft fields for one internally consistent material/stock/machine/runtime bundle.
///
/// These values are persisted as reusable draft evidence only. They do not authorize an estimate
/// or claim production calibration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShopResourceInputFields {
    pub confirmed_for_draft: bool,
    pub library_id: String,
    pub library_version: String,
    pub material_id: String,
    pub material_version: String,
    pub material_family: String,
    pub material_grade: String,
    pub optional_material_specification: String,
    pub optional_material_condition: String,
    pub density_kg_per_m3: String,
    pub material_source: String,
    pub offer_id: String,
    pub offer_version: String,
    pub supplier: String,
    pub material_price_per_kg: String,
    pub offer_effective_on: String,
    pub offer_source: String,
    pub stock_profile_id: String,
    pub stock_profile_version: String,
    pub stock_form: String,
    pub stock_allowance_x_mm: String,
    pub stock_allowance_y_mm: String,
    pub stock_allowance_z_mm: String,
    pub stock_source: String,
    pub machine_id: String,
    pub machine_version: String,
    pub machine_name: String,
    pub process_class: String,
    pub machine_envelope_x_mm: String,
    pub machine_envelope_y_mm: String,
    pub machine_envelope_z_mm: String,
    pub machine_source: String,
    pub runtime_profile_id: String,
    pub runtime_profile_version: String,
    pub removal_rate_mm3_per_minute: String,
    pub setup_minutes: String,
    pub programming_minutes: String,
    pub load_unload_minutes: String,
    pub inspection_minutes: String,
    pub runtime_source: String,
}

/// Explicit value state and safe deterministic trace returned from the application session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DraftEstimateEvaluation {
    pub selection_id: String,
    pub analysis_id: String,
    pub state: DraftEstimateEvaluationState,
    pub reason: Option<String>,
    pub result: Option<Box<DraftEstimateResultSummary>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftEstimateEvaluationState {
    Unavailable,
    Blocked,
    Available,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DraftEstimateResultSummary {
    pub currency: String,
    pub net_part_volume_mm3: String,
    pub part_mass_kg: String,
    pub stock_mass_kg: String,
    pub removed_volume_mm3: String,
    pub removed_volume_warnings: Vec<String>,
    pub make_quantity: u64,
    pub material_cost: String,
    pub setup_cost: String,
    pub programming_cost: String,
    pub cycle_hours_per_item: String,
    pub run_cost: String,
    pub quality_inspection_cost: String,
    pub operation_cost: String,
    pub base_internal_cost: String,
    pub risk_reserve: String,
    pub total_internal_cost: String,
    pub formula_price: String,
    pub governed_price: String,
    pub rounded_selling_price: String,
    pub floor_applied: bool,
    pub minimum_order_applied: bool,
    pub input_trace: DraftEstimateInputFields,
    pub resolved_rates: Vec<ResolvedRateSummary>,
    pub pricing_policy: PricingPolicySummary,
    pub calculation_rule_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedRateSummary {
    pub category: String,
    pub entry_id: String,
    pub amount_per_hour: String,
    pub card_id: String,
    pub card_version: u32,
    pub effective_on: String,
    pub scope_rank: usize,
    pub selector_id: String,
    pub selector_version: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PricingPolicySummary {
    pub policy_id: String,
    pub policy_version: u32,
    pub method: String,
    pub method_rate: String,
    pub rounding_decimal_places: u32,
    pub rounding_mode: String,
}

/// Session-only provisional analysis returned by the native application adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelAnalysisResult {
    pub selection_id: String,
    pub analysis_id: String,
    pub analysis_status: AnalysisStatus,
    pub evidence_state: GeometryEvidenceState,
    pub persistence: PersistenceAvailability,
    pub geometry: ProvisionalGeometryFacts,
    pub stages: Vec<GeometryStageSummary>,
    pub estimate: EstimateReadiness,
}

/// Explicit authority level of geometry shown by the developer slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryEvidenceState {
    ProvisionalExactBrepSpike,
    ProvisionalMeshSpike,
}

/// Path-free geometry facts validated and content-minimized by the native adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "representation", content = "facts", rename_all = "snake_case")]
pub enum ProvisionalGeometryFacts {
    ExactBrep(ProvisionalExactBrepFacts),
    Mesh(ProvisionalMeshFacts),
}

/// Exact-B-rep measurements retained by the existing STEP developer path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProvisionalExactBrepFacts {
    pub canonical_units: CanonicalLengthUnit,
    pub surface_area_mm2: String,
    pub enclosed_volume_mm3: String,
    pub center_of_mass_mm: [String; 3],
    pub solid_body_count: u64,
    pub transferred_roots: u64,
    pub source_hash_sha256: String,
    pub output_hash_sha256: String,
    pub output_byte_length: u64,
    pub geometry_engine: String,
    pub adapter_abi_version: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalLengthUnit {
    Millimeter,
}

/// Content-minimized, non-authoritative mesh facts for review in the WebView.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProvisionalMeshFacts {
    pub detected_format: ModelSourceFormat,
    pub stl_encoding: Option<StlEncoding>,
    pub source_units: ModelLengthUnit,
    pub unit_resolution: UnitResolution,
    pub unit_was_explicit: Option<bool>,
    pub measurement_basis: MeshMeasurementBasis,
    pub aabb_extents: [String; 3],
    pub surface_area: String,
    pub enclosed_volume: Option<String>,
    pub center_of_mass: Option<[String; 3]>,
    pub triangle_count: u64,
    pub manifold: bool,
    pub watertight: bool,
    pub consistently_wound: bool,
    pub self_intersection: MeshSelfIntersectionState,
    pub confidence_level: GeometryConfidenceLevel,
    pub confidence_reason_codes: Vec<String>,
    pub algorithm_version: String,
    pub self_intersection_algorithm_version: String,
    pub confidence_policy_version: String,
    pub topology_policy_version: String,
    pub topology_identity: MeshTopologyIdentity,
    pub welding_status: MeshWeldingStatus,
    pub warning_codes: Vec<String>,
    pub source_hash_sha256: String,
    pub output_hash_sha256: String,
    pub output_byte_length: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StlEncoding {
    Ascii,
    Binary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLengthUnit {
    Micrometer,
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitResolution {
    Declared,
    Confirmed,
    Inferred,
    Unresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshMeasurementBasis {
    SourceCoordinates,
    CanonicalMillimeters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryConfidenceLevel {
    High,
    Medium,
    Low,
    NeedsReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshTopologyIdentity {
    ExactSourceCoordinates,
    SourceVertexIndices,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshWeldingStatus {
    NotApplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshSelfIntersectionState {
    NotDetected,
    Detected,
    Indeterminate,
}

/// Content-minimized outcome of one requested geometry stage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GeometryStageSummary {
    pub stage: String,
    pub status: String,
    pub warning_codes: Vec<String>,
}

/// Estimate availability remains explicit until all GUI-2 prerequisites are supplied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EstimateReadiness {
    pub state: EstimateAvailability,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateAvailability {
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelSourceSelectedEvent {
    pub source: SelectedModelSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HostCommandError {
    pub code: HostErrorCode,
    pub message: String,
    pub diagnostic_id: String,
}

impl HostCommandError {
    #[must_use]
    pub fn unsupported_model_format(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::UnsupportedModelFormat,
            message: "Choose a STEP, STL, or 3MF model with a supported extension.".to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_selection(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::InvalidSelection,
            message: "The selected model could not be accepted. Choose another local model file."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn host_state_unavailable(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::HostStateUnavailable,
            message: "The desktop session could not retain the selected model. Restart PartProbe and try again."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn stale_selection(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::StaleSelection,
            message:
                "That model is no longer the active session selection. Choose the model again."
                    .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn analysis_unavailable(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::AnalysisUnavailable,
            message: "Provisional geometry analysis is not configured for this developer session."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn analysis_failed(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::AnalysisFailed,
            message: "The provisional geometry analysis failed safely. The selected source remains available for retry."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn analysis_in_progress(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::AnalysisInProgress,
            message: "A provisional analysis is already running for this session.".to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn analysis_cancelled(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::AnalysisCancelled,
            message: "The provisional geometry analysis was cancelled. The selected source remains available for retry."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_estimate_input(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::InvalidEstimateInput,
            message: "One or more required estimate values are missing or invalid. Review every field and confirmation."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn settings_unavailable(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::SettingsUnavailable,
            message: "Local shop-settings storage is unavailable. No settings were changed."
                .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_settings_input(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::InvalidSettingsInput,
            message: "One or more shop-settings values are missing or invalid. Review every field and confirmation.".to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn settings_conflict(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::SettingsConflict,
            message:
                "Shop settings changed since this screen was loaded. Reload before saving again."
                    .to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn settings_version_conflict(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::SettingsVersionConflict,
            message: "A changed rate card or pricing policy must use a new version. Review both version fields before saving.".to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostErrorCode {
    UnsupportedModelFormat,
    InvalidSelection,
    HostStateUnavailable,
    StaleSelection,
    AnalysisUnavailable,
    AnalysisFailed,
    AnalysisInProgress,
    AnalysisCancelled,
    InvalidEstimateInput,
    SettingsUnavailable,
    InvalidSettingsInput,
    SettingsConflict,
    SettingsVersionConflict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_contract_distinguishes_session_and_settings_persistence() {
        let contract = DesktopContract::current();

        assert_eq!(contract.contract_version, DESKTOP_CONTRACT_VERSION);
        assert_eq!(contract.contract_version, 6);
        assert_eq!(contract.commands, APPLICATION_COMMANDS);
        assert_eq!(contract.events, APPLICATION_EVENTS);
        assert_eq!(
            contract.analysis_authority,
            AnalysisAuthority::NativeApplicationService
        );
        assert_eq!(contract.persistence, PersistenceAvailability::SessionOnly);
        assert_eq!(
            contract.shop_settings_persistence,
            ShopSettingsPersistenceAvailability::LocalDrafts
        );
    }

    #[test]
    fn settings_contract_is_path_free_and_preserves_first_run_absence() {
        assert_eq!(
            serde_json::to_value(ShopSettingsState::NotConfigured).unwrap(),
            serde_json::json!({"state": "not_configured"})
        );
        let request = SaveShopSettingsRequest {
            expected_revision: None,
            changed_by: "operator-1".to_owned(),
            change_reason: "initial test rates".to_owned(),
            rates: DeveloperRateInputFields {
                confirmed_for_session: true,
                rate_card_id: "shop-rates".to_owned(),
                rate_card_version: "1".to_owned(),
                effective_on: "2026-09-04".to_owned(),
                currency: "USD".to_owned(),
                setup_labor_per_hour: "75".to_owned(),
                programming_per_hour: "90".to_owned(),
                run_labor_per_hour: "65".to_owned(),
                machine_per_hour: "110".to_owned(),
                quality_inspection_per_hour: "80".to_owned(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: true,
                pricing_policy_id: "shop-pricing".to_owned(),
                pricing_policy_version: "1".to_owned(),
                markup_rate: "0.25".to_owned(),
                optional_price_floor: String::new(),
                optional_minimum_order: String::new(),
                rounding_decimal_places: "2".to_owned(),
            },
            resources: None,
        };
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(!serialized.contains("path"));
        assert!(!serialized.contains("sqlite"));
    }

    #[test]
    fn governed_model_extensions_are_recognized() {
        assert_eq!(
            ModelSourceFormat::from_extension("STEP"),
            Some(ModelSourceFormat::Step)
        );
        assert_eq!(
            ModelSourceFormat::from_extension("stp"),
            Some(ModelSourceFormat::Step)
        );
        assert_eq!(
            ModelSourceFormat::from_extension("stl"),
            Some(ModelSourceFormat::Stl)
        );
        assert_eq!(
            ModelSourceFormat::from_extension("3MF"),
            Some(ModelSourceFormat::ThreeMf)
        );
        assert_eq!(ModelSourceFormat::from_extension(""), None);
    }

    #[test]
    fn cancellation_is_a_distinct_retryable_host_result() {
        let error = HostCommandError::analysis_cancelled("GUI4-ANALYSIS-CANCELLED-TEST");

        assert_eq!(error.code, HostErrorCode::AnalysisCancelled);
        assert!(error.message.contains("cancelled"));
        assert!(error.message.contains("retry"));
        assert_eq!(
            serde_json::to_value(error).expect("host error must serialize")["code"],
            "analysis_cancelled"
        );
    }

    #[test]
    fn provisional_analysis_contract_is_path_free_and_keeps_estimate_unavailable() {
        let result = ModelAnalysisResult {
            selection_id: "selection-1".to_owned(),
            analysis_id: "analysis-1".to_owned(),
            analysis_status: AnalysisStatus::ProvisionalAvailable,
            evidence_state: GeometryEvidenceState::ProvisionalExactBrepSpike,
            persistence: PersistenceAvailability::SessionOnly,
            geometry: ProvisionalGeometryFacts::ExactBrep(ProvisionalExactBrepFacts {
                canonical_units: CanonicalLengthUnit::Millimeter,
                surface_area_mm2: "600".to_owned(),
                enclosed_volume_mm3: "1000".to_owned(),
                center_of_mass_mm: ["5".to_owned(), "5".to_owned(), "5".to_owned()],
                solid_body_count: 1,
                transferred_roots: 1,
                source_hash_sha256: "a".repeat(64),
                output_hash_sha256: "b".repeat(64),
                output_byte_length: 256,
                geometry_engine: "OCCT 8.0.0".to_owned(),
                adapter_abi_version: 3,
            }),
            stages: vec![GeometryStageSummary {
                stage: "basic_properties".to_owned(),
                status: "succeeded".to_owned(),
                warning_codes: Vec::new(),
            }],
            estimate: EstimateReadiness {
                state: EstimateAvailability::Unavailable,
                reason: "geometry units and warnings have not been reviewed".to_owned(),
            },
        };

        let serialized = serde_json::to_string(&result).expect("analysis result must serialize");
        assert!(!serialized.contains("/sensitive"));
        assert!(!serialized.contains("source_path"));
        assert!(serialized.contains("provisional_exact_brep_spike"));
        assert!(serialized.contains("unavailable"));
    }

    #[test]
    fn mesh_analysis_contract_keeps_withheld_measurements_absent_and_path_free() {
        let facts = ProvisionalGeometryFacts::Mesh(ProvisionalMeshFacts {
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
            algorithm_version: "partprobe-ascii-stl-spike-v3".to_owned(),
            self_intersection_algorithm_version: "partprobe-exact-mesh-intersection-spike-v1"
                .to_owned(),
            confidence_policy_version: "partprobe-mesh-confidence-policy-v1".to_owned(),
            topology_policy_version: "partprobe-mesh-topology-policy-v1".to_owned(),
            topology_identity: MeshTopologyIdentity::ExactSourceCoordinates,
            welding_status: MeshWeldingStatus::NotApplied,
            warning_codes: vec!["UNITS_MISSING_REQUIRES_CONFIRMATION".to_owned()],
            source_hash_sha256: "a".repeat(64),
            output_hash_sha256: "b".repeat(64),
            output_byte_length: 512,
        });

        let serialized = serde_json::to_string(&facts).expect("mesh facts must serialize");
        assert!(serialized.contains("\"representation\":\"mesh\""));
        assert!(serialized.contains("\"enclosed_volume\":null"));
        assert!(serialized.contains("\"center_of_mass\":null"));
        assert!(!serialized.contains("source_path"));
        assert!(!serialized.contains("/private"));
        assert_eq!(
            serde_json::from_str::<ProvisionalGeometryFacts>(&serialized).unwrap(),
            facts
        );
    }
}
