#![forbid(unsafe_code)]

use partprobe_desktop_contract::{
    DraftEstimateEvaluation, HostCommandError, HostErrorCode, ModelAnalysisResult,
    ModelSourceSelection, SelectedModelSource,
};

#[must_use]
pub fn selected_source_accessible_label(display_name: &str) -> String {
    format!("Selected model source: {display_name}")
}

#[must_use]
pub fn provisional_geometry_accessible_label(
    surface_area_mm2: &str,
    enclosed_volume_mm3: &str,
    center_of_mass_mm: [&str; 3],
    geometry_engine: &str,
) -> String {
    format!(
        "Provisional geometry available: surface area {surface_area_mm2} square millimeters; \
         enclosed volume {enclosed_volume_mm3} cubic millimeters; centroid {} millimeters; \
         engine {geometry_engine}",
        center_of_mass_mm.join(", ")
    )
}

#[must_use]
pub fn provisional_analysis_failure_accessible_label(diagnostic_id: &str) -> String {
    format!("Provisional analysis failed safely: diagnostic {diagnostic_id}")
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
            Self::Selected(_) => "Model selected — analysis not started",
            Self::Failed => "Model selection failed",
        }
    }

    #[must_use]
    pub const fn status_detail(&self) -> &'static str {
        match self {
            Self::Empty => "Choose a local STEP file to begin the provisional analysis workflow.",
            Self::Selected(_) => {
                "The source is retained only for this session. Analysis requires the explicit local worker, and estimating requires complete reviewed inputs."
            }
            Self::Failed => {
                "PartProbe did not retain a source path. Choose another local STEP file or restart the session."
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
            Self::Failed(_) => "Provisional analysis failed safely",
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

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::mount;

#[cfg(test)]
mod tests {
    use partprobe_desktop_contract::{AnalysisStatus, ModelSourceFormat, PersistenceAvailability};

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
        assert!(state.status_heading().contains("analysis not started"));
        assert!(state.status_detail().contains("session"));
        assert!(state.status_detail().contains("requires"));
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
        assert_eq!(
            provisional_geometry_accessible_label("392", "480", ["6", "4", "2.5"], "OCCT 8.0.0",),
            "Provisional geometry available: surface area 392 square millimeters; enclosed volume \
             480 cubic millimeters; centroid 6, 4, 2.5 millimeters; engine OCCT 8.0.0"
        );
    }

    #[test]
    fn provisional_analysis_failure_label_carries_only_the_diagnostic() {
        assert_eq!(
            provisional_analysis_failure_accessible_label("GUI4-ANALYSIS-TEST"),
            "Provisional analysis failed safely: diagnostic GUI4-ANALYSIS-TEST"
        );
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

        assert!(state.status_heading().contains("failed safely"));
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
}
