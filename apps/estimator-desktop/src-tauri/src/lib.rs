#![forbid(unsafe_code)]

use std::{path::PathBuf, sync::Mutex};

use partprobe_desktop_contract::{
    AnalysisStatus, HostCommandError, ModelSourceFormat, PersistenceAvailability,
    SelectedModelSource,
};

#[derive(Debug)]
pub struct DesktopSessionState {
    next_selection_number: Mutex<u64>,
    selected_source: Mutex<Option<RetainedModelSource>>,
}

impl Default for DesktopSessionState {
    fn default() -> Self {
        Self {
            next_selection_number: Mutex::new(1),
            selected_source: Mutex::new(None),
        }
    }
}

impl DesktopSessionState {
    pub fn retain_selected_path(
        &self,
        path: PathBuf,
    ) -> Result<SelectedModelSource, HostCommandError> {
        let display_name = path
            .file_name()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| HostCommandError::invalid_selection("GUI3-SELECTION-NAME"))?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| HostCommandError::unsupported_model_format("GUI3-SELECTION-FORMAT"))?;
        let format = ModelSourceFormat::from_extension(extension)
            .ok_or_else(|| HostCommandError::unsupported_model_format("GUI3-SELECTION-FORMAT"))?;

        let mut next_number = self
            .next_selection_number
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI3-SELECTION-COUNTER"))?;
        let selection_id = format!("selection-{next_number}");
        *next_number = next_number.checked_add(1).unwrap_or(1);

        let summary = SelectedModelSource {
            selection_id: selection_id.clone(),
            display_name,
            format,
            analysis_status: AnalysisStatus::NotStarted,
            persistence: PersistenceAvailability::SessionOnly,
        };
        let retained = RetainedModelSource { selection_id, path };
        *self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI3-SELECTION-STATE"))? =
            Some(retained);

        Ok(summary)
    }

    #[must_use]
    pub fn retained_path(&self, selection_id: &str) -> Option<PathBuf> {
        self.selected_source
            .lock()
            .ok()
            .and_then(|source| source.clone())
            .filter(|source| source.selection_id == selection_id)
            .map(|source| source.path)
    }
}

#[derive(Clone, Debug)]
struct RetainedModelSource {
    selection_id: String,
    path: PathBuf,
}

#[cfg(feature = "desktop-host")]
mod runtime;

#[cfg(feature = "desktop-host")]
pub use runtime::run;

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::Path};

    use partprobe_desktop_contract::{
        APPLICATION_COMMANDS, APPLICATION_EVENTS, ModelSourceSelection,
    };
    use serde_json::Value;

    use super::*;

    const BUILD_SCRIPT: &str = include_str!("../build.rs");
    const CAPABILITY: &str = include_str!("../capabilities/main.json");
    const CONFIG: &str = include_str!("../tauri.conf.json");
    const RUNTIME: &str = include_str!("runtime.rs");

    #[test]
    fn selected_path_remains_native_and_summary_is_explicitly_provisional() {
        let state = DesktopSessionState::default();
        let path = PathBuf::from("/sensitive/customer/gearbox.step");

        let summary = state.retain_selected_path(path.clone()).unwrap();

        assert_eq!(summary.selection_id, "selection-1");
        assert_eq!(summary.display_name, "gearbox.step");
        assert_eq!(summary.analysis_status, AnalysisStatus::NotStarted);
        assert_eq!(summary.persistence, PersistenceAvailability::SessionOnly);
        assert_eq!(state.retained_path(&summary.selection_id), Some(path));

        let serialized =
            serde_json::to_string(&ModelSourceSelection::Selected { source: summary }).unwrap();
        assert!(!serialized.contains("/sensitive"));
        assert!(!serialized.contains("customer"));
    }

    #[test]
    fn unsupported_source_is_rejected_without_retaining_a_path() {
        let state = DesktopSessionState::default();
        let error = state
            .retain_selected_path(PathBuf::from("/private/customer/mesh.stl"))
            .unwrap_err();

        assert_eq!(
            error.code,
            partprobe_desktop_contract::HostErrorCode::UnsupportedModelFormat
        );
        assert!(!error.message.contains("/private"));
        assert_eq!(state.retained_path("selection-1"), None);
    }

    #[test]
    fn application_manifest_exposes_only_contract_commands() {
        let build_commands = quoted_values_in_rust_slice(BUILD_SCRIPT, "const COMMANDS");
        let expected = APPLICATION_COMMANDS
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();

        assert_eq!(build_commands, expected);
    }

    #[test]
    fn main_capability_is_exact_and_has_no_remote_or_broad_plugin_permission() {
        let capability: Value = serde_json::from_str(CAPABILITY).unwrap();
        assert_eq!(capability["windows"], serde_json::json!(["main"]));
        assert_eq!(
            capability["platforms"],
            serde_json::json!(["linux", "macOS", "windows"])
        );
        assert!(capability.get("remote").is_none());

        let permissions = capability["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            permissions,
            BTreeSet::from([
                "allow-desktop-contract",
                "allow-select-model-source",
                "core:event:allow-listen",
                "core:event:allow-unlisten",
            ])
        );
        assert!(permissions.iter().all(|permission| {
            !permission.starts_with("dialog:")
                && !permission.starts_with("fs:")
                && !permission.starts_with("http:")
                && !permission.starts_with("opener:")
                && !permission.starts_with("shell:")
                && !permission.ends_with(":default")
        }));
    }

    #[test]
    fn production_config_uses_local_assets_restrictive_csp_and_one_capability() {
        let config: Value = serde_json::from_str(CONFIG).unwrap();
        let security = &config["app"]["security"];
        let csp = security["csp"].as_str().unwrap();

        assert_eq!(security["capabilities"], serde_json::json!(["main"]));
        assert_eq!(security["freezePrototype"], true);
        assert_eq!(config["app"]["windows"][0]["label"], "main");
        assert_eq!(config["app"]["windows"][0]["decorations"], true);
        assert_eq!(config["build"]["frontendDist"], "../dist");
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("script-src 'self' 'wasm-unsafe-eval'"));
        assert!(csp.contains("connect-src 'self' ipc: http://ipc.localhost"));
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-src 'none'"));
        assert!(!csp.contains("https:"));
        assert!(!csp.contains("'unsafe-inline'"));
    }

    #[test]
    fn shared_contract_lists_the_single_safe_event() {
        assert_eq!(APPLICATION_EVENTS, ["partprobe:model-source-selected"]);
    }

    #[test]
    fn native_picker_is_non_blocking() {
        assert!(RUNTIME.contains("async fn select_model_source"));
        assert!(RUNTIME.contains(".pick_file("));
        assert!(!RUNTIME.contains("blocking_pick_file"));
    }

    fn quoted_values_in_rust_slice(source: &str, anchor: &str) -> BTreeSet<String> {
        quoted_values_in_section(source, anchor, "];", false)
    }

    fn quoted_values_in_section(
        source: &str,
        anchor: &str,
        terminator: &str,
        skip_anchor_line: bool,
    ) -> BTreeSet<String> {
        let start = source.find(anchor).unwrap_or(0);
        let section = &source[start..];
        let end = section.find(terminator).unwrap_or(section.len());
        let section = &section[..end];

        section
            .lines()
            .skip(usize::from(skip_anchor_line))
            .flat_map(|line| line.split('"').skip(1).step_by(2))
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn path_accessor_is_bound_to_the_current_session_token() {
        let state = DesktopSessionState::default();
        let first = state
            .retain_selected_path(Path::new("first.step").to_path_buf())
            .unwrap();
        let second_path = Path::new("second.stp").to_path_buf();
        let second = state.retain_selected_path(second_path.clone()).unwrap();

        assert_eq!(state.retained_path(&first.selection_id), None);
        assert_eq!(state.retained_path(&second.selection_id), Some(second_path));
    }
}
