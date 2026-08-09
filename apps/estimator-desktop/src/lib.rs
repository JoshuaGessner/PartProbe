#![forbid(unsafe_code)]

use partprobe_desktop_contract::{
    DraftEstimateEvaluation, HostCommandError, ModelAnalysisResult, ModelSourceSelection,
    SelectedModelSource,
};

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
            Self::Available(result) => &result.estimate.reason,
            Self::Failed(error) => &error.message,
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
    fn rejected_estimate_inputs_never_imply_a_numeric_result() {
        let state = DraftEstimatePanelState::Failed(HostCommandError::invalid_estimate_input(
            "GUI4-ESTIMATE-TEST",
        ));

        assert!(state.status_heading().contains("unavailable"));
        assert!(state.status_detail().contains("missing or invalid"));
    }
}
