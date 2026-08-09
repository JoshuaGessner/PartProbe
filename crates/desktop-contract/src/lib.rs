#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DESKTOP_CONTRACT_VERSION: u16 = 3;
pub const COMMAND_DESKTOP_CONTRACT: &str = "desktop_contract";
pub const COMMAND_SELECT_MODEL_SOURCE: &str = "select_model_source";
pub const COMMAND_ANALYZE_MODEL_SOURCE: &str = "analyze_model_source";
pub const COMMAND_CANCEL_MODEL_ANALYSIS: &str = "cancel_model_analysis";
pub const COMMAND_EVALUATE_DRAFT_ESTIMATE: &str = "evaluate_draft_estimate";
pub const EVENT_MODEL_SOURCE_SELECTED: &str = "partprobe:model-source-selected";
pub const APPLICATION_COMMANDS: [&str; 5] = [
    COMMAND_DESKTOP_CONTRACT,
    COMMAND_SELECT_MODEL_SOURCE,
    COMMAND_ANALYZE_MODEL_SOURCE,
    COMMAND_CANCEL_MODEL_ANALYSIS,
    COMMAND_EVALUATE_DRAFT_ESTIMATE,
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
}

impl ModelSourceFormat {
    #[must_use]
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "step" | "stp" => Some(Self::Step),
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
    ProvisionalSpike,
}

/// Exact, path-free geometry facts validated by the application service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProvisionalGeometryFacts {
    pub canonical_units: CanonicalLengthUnit,
    pub representation: GeometryRepresentation,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryRepresentation {
    ExactBrep,
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
            message: "Choose a STEP model with a .step or .stp extension.".to_owned(),
            diagnostic_id: diagnostic_id.into(),
        }
    }

    #[must_use]
    pub fn invalid_selection(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::InvalidSelection,
            message: "The selected model could not be accepted. Choose another local STEP file."
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
            message: "Provisional STEP analysis is not configured for this developer session."
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
    pub fn invalid_estimate_input(diagnostic_id: impl Into<String>) -> Self {
        Self {
            code: HostErrorCode::InvalidEstimateInput,
            message: "One or more required estimate values are missing or invalid. Review every field and confirmation."
                .to_owned(),
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
    InvalidEstimateInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_contract_is_explicit_and_session_only() {
        let contract = DesktopContract::current();

        assert_eq!(contract.contract_version, DESKTOP_CONTRACT_VERSION);
        assert_eq!(contract.commands, APPLICATION_COMMANDS);
        assert_eq!(contract.events, APPLICATION_EVENTS);
        assert_eq!(
            contract.analysis_authority,
            AnalysisAuthority::NativeApplicationService
        );
        assert_eq!(contract.persistence, PersistenceAvailability::SessionOnly);
    }

    #[test]
    fn only_step_extensions_are_recognized() {
        assert_eq!(
            ModelSourceFormat::from_extension("STEP"),
            Some(ModelSourceFormat::Step)
        );
        assert_eq!(
            ModelSourceFormat::from_extension("stp"),
            Some(ModelSourceFormat::Step)
        );
        assert_eq!(ModelSourceFormat::from_extension("stl"), None);
        assert_eq!(ModelSourceFormat::from_extension(""), None);
    }

    #[test]
    fn provisional_analysis_contract_is_path_free_and_keeps_estimate_unavailable() {
        let result = ModelAnalysisResult {
            selection_id: "selection-1".to_owned(),
            analysis_id: "analysis-1".to_owned(),
            analysis_status: AnalysisStatus::ProvisionalAvailable,
            evidence_state: GeometryEvidenceState::ProvisionalSpike,
            persistence: PersistenceAvailability::SessionOnly,
            geometry: ProvisionalGeometryFacts {
                canonical_units: CanonicalLengthUnit::Millimeter,
                representation: GeometryRepresentation::ExactBrep,
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
            },
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
        assert!(serialized.contains("provisional_spike"));
        assert!(serialized.contains("unavailable"));
    }
}
