#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const DESKTOP_CONTRACT_VERSION: u16 = 1;
pub const COMMAND_DESKTOP_CONTRACT: &str = "desktop_contract";
pub const COMMAND_SELECT_MODEL_SOURCE: &str = "select_model_source";
pub const EVENT_MODEL_SOURCE_SELECTED: &str = "partprobe:model-source-selected";
pub const APPLICATION_COMMANDS: [&str; 2] = [COMMAND_DESKTOP_CONTRACT, COMMAND_SELECT_MODEL_SOURCE];
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostErrorCode {
    UnsupportedModelFormat,
    InvalidSelection,
    HostStateUnavailable,
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
}
