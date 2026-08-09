#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DESKTOP_CONTRACT_VERSION: u16 = 2;
pub const COMMAND_DESKTOP_CONTRACT: &str = "desktop_contract";
pub const COMMAND_SELECT_MODEL_SOURCE: &str = "select_model_source";
pub const COMMAND_ANALYZE_MODEL_SOURCE: &str = "analyze_model_source";
pub const EVENT_MODEL_SOURCE_SELECTED: &str = "partprobe:model-source-selected";
pub const APPLICATION_COMMANDS: [&str; 3] = [
    COMMAND_DESKTOP_CONTRACT,
    COMMAND_SELECT_MODEL_SOURCE,
    COMMAND_ANALYZE_MODEL_SOURCE,
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
