#![forbid(unsafe_code)]

use partprobe_desktop_contract::{ModelSourceSelection, SelectedModelSource};

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
                "The source is retained only for this session. Geometry analysis and estimating remain unavailable in this shell checkpoint."
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
        assert!(state.status_detail().contains("unavailable"));
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
}
