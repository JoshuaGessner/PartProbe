#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use partprobe_desktop_contract::{
    AnalysisCancellationAcknowledgement, AnalysisStatus, CancelModelAnalysisRequest,
    DraftEstimateEvaluation, DraftEstimateProposalEvaluation, EvaluateDraftEstimateRequest,
    HostCommandError, ModelAnalysisResult, ModelSourceFormat, PersistenceAvailability,
    PrepareDraftEstimateProposalRequest, SelectedModelSource,
};
use partprobe_domain::ShopSettingsDraft;
use partprobe_geometry_import::GeometryWorkerSupervisor;

mod analysis;
mod estimate;
mod proposal;
mod settings;

use analysis::DesktopAnalysisAdapter;
pub use settings::{DesktopCatalogAuthorizationPolicy, DesktopSettingsState};

#[derive(Debug)]
pub struct DesktopSessionState {
    next_selection_number: Mutex<u64>,
    next_analysis_number: Mutex<u64>,
    selected_source: Mutex<Option<RetainedModelSource>>,
    analysis_session: Mutex<Option<RetainedAnalysisSession>>,
    active_analysis: Mutex<Option<ActiveAnalysis>>,
    analysis_adapter: Option<DesktopAnalysisAdapter<GeometryWorkerSupervisor>>,
}

impl Default for DesktopSessionState {
    fn default() -> Self {
        Self {
            next_selection_number: Mutex::new(1),
            next_analysis_number: Mutex::new(1),
            selected_source: Mutex::new(None),
            analysis_session: Mutex::new(None),
            active_analysis: Mutex::new(None),
            analysis_adapter: None,
        }
    }
}

impl DesktopSessionState {
    #[cfg(feature = "desktop-host")]
    fn with_analysis_adapter(
        analysis_adapter: DesktopAnalysisAdapter<GeometryWorkerSupervisor>,
    ) -> Self {
        Self {
            analysis_adapter: Some(analysis_adapter),
            ..Self::default()
        }
    }

    pub fn retain_selected_path(
        &self,
        path: PathBuf,
    ) -> Result<SelectedModelSource, HostCommandError> {
        if let Some(active) = self
            .active_analysis
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-ACTIVE"))?
            .as_ref()
        {
            active.cancellation.store(true, Ordering::Release);
        }
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
        let retained = RetainedModelSource {
            selection_id,
            path,
            format,
        };
        *self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI3-SELECTION-STATE"))? =
            Some(retained);
        *self
            .analysis_session
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-STATE"))? = None;

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

    pub fn analyze_selected_source(
        &self,
        selection_id: &str,
    ) -> Result<ModelAnalysisResult, HostCommandError> {
        let selected = self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-SELECTION"))?
            .clone()
            .filter(|source| source.selection_id == selection_id)
            .ok_or_else(|| HostCommandError::stale_selection("GUI4-ANALYSIS-STALE-SELECTION"))?;
        let adapter = self.analysis_adapter.as_ref().ok_or_else(|| {
            HostCommandError::analysis_unavailable("GUI4-ANALYSIS-NOT-CONFIGURED")
        })?;
        let analysis_number = {
            let mut next_number = self
                .next_analysis_number
                .lock()
                .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-COUNTER"))?;
            let number = *next_number;
            *next_number = next_number.checked_add(1).unwrap_or(1);
            number
        };
        let cancellation = Arc::new(AtomicBool::new(false));
        {
            let mut active = self
                .active_analysis
                .lock()
                .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-ACTIVE"))?;
            if active.is_some() {
                return Err(HostCommandError::analysis_in_progress(
                    "GUI4-ANALYSIS-ALREADY-RUNNING",
                ));
            }
            *active = Some(ActiveAnalysis {
                selection_id: selection_id.to_owned(),
                analysis_number,
                cancellation: Arc::clone(&cancellation),
            });
        }
        let outcome = adapter.analyze(
            selection_id,
            selected.format,
            &selected.path,
            analysis_number,
            &cancellation,
        );
        self.finish_analysis(analysis_number)?;
        let (session, result) = outcome?;

        let current_selection = self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-SELECTION"))?;
        if current_selection
            .as_ref()
            .is_none_or(|current| current.selection_id != selection_id)
        {
            return Err(HostCommandError::stale_selection(
                "GUI4-ANALYSIS-REPLACED-SELECTION",
            ));
        }
        *self
            .analysis_session
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-STATE"))? =
            Some(RetainedAnalysisSession {
                selection_id: selection_id.to_owned(),
                analysis_id: result.analysis_id.clone(),
                session,
            });
        Ok(result)
    }

    pub fn cancel_model_analysis(
        &self,
        request: &CancelModelAnalysisRequest,
    ) -> Result<AnalysisCancellationAcknowledgement, HostCommandError> {
        let active = self
            .active_analysis
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-ACTIVE"))?;
        let cancellation_requested = active
            .as_ref()
            .filter(|active| active.selection_id == request.selection_id)
            .is_some_and(|active| {
                active.cancellation.store(true, Ordering::Release);
                true
            });
        Ok(AnalysisCancellationAcknowledgement {
            selection_id: request.selection_id.clone(),
            cancellation_requested,
        })
    }

    pub fn evaluate_draft_estimate(
        &self,
        request: &EvaluateDraftEstimateRequest,
        settings: Option<&ShopSettingsDraft>,
    ) -> Result<DraftEstimateEvaluation, HostCommandError> {
        let current_selection = self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ESTIMATE-SELECTION"))?;
        if current_selection
            .as_ref()
            .is_none_or(|selected| selected.selection_id != request.selection_id)
        {
            return Err(HostCommandError::stale_selection(
                "GUI4-ESTIMATE-STALE-SELECTION",
            ));
        }
        // Keep the selection guard until the retained session is locked so source
        // replacement cannot race a valid evaluation onto the superseded session.
        let mut retained = self
            .analysis_session
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ESTIMATE-SESSION"))?;
        let retained = retained
            .as_mut()
            .filter(|retained| {
                retained.selection_id == request.selection_id
                    && retained.analysis_id == request.analysis_id
            })
            .ok_or_else(|| HostCommandError::stale_selection("GUI4-ESTIMATE-STALE-ANALYSIS"))?;
        estimate::evaluate_draft_estimate(&mut retained.session, request, settings)
    }

    pub fn prepare_draft_estimate_proposal(
        &self,
        request: &PrepareDraftEstimateProposalRequest,
        settings: &ShopSettingsDraft,
    ) -> Result<DraftEstimateProposalEvaluation, HostCommandError> {
        let current_selection = self
            .selected_source
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("USE3-PROPOSAL-SELECTION"))?;
        if current_selection
            .as_ref()
            .is_none_or(|selected| selected.selection_id != request.selection_id)
        {
            return Err(HostCommandError::stale_selection(
                "USE3-PROPOSAL-STALE-SELECTION",
            ));
        }
        let retained = self
            .analysis_session
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("USE3-PROPOSAL-SESSION"))?;
        let retained = retained
            .as_ref()
            .filter(|retained| {
                retained.selection_id == request.selection_id
                    && retained.analysis_id == request.analysis_id
            })
            .ok_or_else(|| HostCommandError::stale_selection("USE3-PROPOSAL-STALE-ANALYSIS"))?;
        Ok(proposal::prepare_draft_estimate_proposal(
            &retained.session,
            settings,
            &request.selection_id,
            &request.analysis_id,
        ))
    }

    fn finish_analysis(&self, analysis_number: u64) -> Result<(), HostCommandError> {
        let mut active = self
            .active_analysis
            .lock()
            .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-ACTIVE"))?;
        if active
            .as_ref()
            .is_some_and(|active| active.analysis_number == analysis_number)
        {
            *active = None;
        }
        Ok(())
    }

    #[cfg(test)]
    fn retained_analysis_identity(&self) -> Option<(String, String)> {
        self.analysis_session.lock().ok().and_then(|retained| {
            retained.as_ref().map(|retained| {
                let _ = retained.session.geometry();
                (retained.selection_id.clone(), retained.analysis_id.clone())
            })
        })
    }
}

#[derive(Clone, Debug)]
struct RetainedModelSource {
    selection_id: String,
    path: PathBuf,
    format: ModelSourceFormat,
}

#[derive(Clone, Debug)]
struct RetainedAnalysisSession {
    selection_id: String,
    analysis_id: String,
    session: partprobe_application::DraftEstimateSession,
}

#[derive(Clone, Debug)]
struct ActiveAnalysis {
    selection_id: String,
    analysis_number: u64,
    cancellation: Arc<AtomicBool>,
}

#[cfg(feature = "desktop-host")]
mod runtime;
#[cfg(feature = "desktop-host")]
mod viewer;

#[cfg(feature = "desktop-host")]
pub use runtime::run;

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::Path};

    use partprobe_desktop_contract::{
        APPLICATION_COMMANDS, APPLICATION_EVENTS, ModelSourceSelection,
    };
    #[cfg(feature = "desktop-host")]
    use partprobe_desktop_contract::{
        DeveloperPricingInputFields, DeveloperRateInputFields, DraftEstimateProposalAdoptionInput,
        SaveShopSettingsRequest, ShopResourceInputFields,
    };
    #[cfg(feature = "desktop-host")]
    use partprobe_test_support::TestDirectory;
    use serde_json::Value;

    use super::*;

    const BUILD_SCRIPT: &str = include_str!("../build.rs");
    const CAPABILITY: &str = include_str!("../capabilities/main.json");
    const CONFIG: &str = include_str!("../tauri.conf.json");
    const RUNTIME: &str = include_str!("runtime.rs");
    const SETTINGS_ADAPTER: &str = include_str!("settings.rs");
    const VIEWER_ADAPTER: &str = include_str!("viewer.rs");
    const WEBVIEW: &str = include_str!("../../src/web.rs");

    #[test]
    fn selected_path_remains_native_and_summary_is_explicitly_provisional() {
        let state = DesktopSessionState::default();
        let path = PathBuf::from("/sensitive/customer/gearbox.step");

        let summary = state.retain_selected_path(path.clone()).unwrap();

        assert_eq!(summary.selection_id, "selection-1");
        assert_eq!(summary.display_name, "gearbox.step");
        assert_eq!(summary.format, ModelSourceFormat::Step);
        assert_eq!(summary.analysis_status, AnalysisStatus::NotStarted);
        assert_eq!(summary.persistence, PersistenceAvailability::SessionOnly);
        assert_eq!(state.retained_path(&summary.selection_id), Some(path));

        let serialized =
            serde_json::to_string(&ModelSourceSelection::Selected { source: summary }).unwrap();
        assert!(!serialized.contains("/sensitive"));
        assert!(!serialized.contains("customer"));
    }

    #[test]
    fn governed_mesh_extensions_are_retained_only_behind_native_tokens() {
        for (name, expected_format) in [
            ("fixture.stl", ModelSourceFormat::Stl),
            ("fixture.3mf", ModelSourceFormat::ThreeMf),
        ] {
            let state = DesktopSessionState::default();
            let path = PathBuf::from(format!("/sensitive/customer/{name}"));
            let summary = state.retain_selected_path(path.clone()).unwrap();

            assert_eq!(summary.format, expected_format);
            assert_eq!(state.retained_path(&summary.selection_id), Some(path));
            let serialized = serde_json::to_string(&summary).unwrap();
            assert!(!serialized.contains("/sensitive"));
            assert!(!serialized.contains("customer"));
        }
    }

    #[test]
    fn unsupported_source_is_rejected_without_retaining_a_path() {
        let state = DesktopSessionState::default();
        let error = state
            .retain_selected_path(PathBuf::from("/private/customer/mesh.obj"))
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
    fn runtime_handler_exposes_only_contract_commands() {
        let start = RUNTIME.find("tauri::generate_handler![").unwrap();
        let section = &RUNTIME[start + "tauri::generate_handler![".len()..];
        let end = section.find("])").unwrap();
        let registered = section[..end]
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        let expected = APPLICATION_COMMANDS
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();

        assert_eq!(registered, expected);
    }

    #[test]
    fn developer_viewer_reuses_the_main_window_and_keeps_geometry_native() {
        assert!(VIEWER_ADAPTER.contains("get_window(\"main\")"));
        assert!(VIEWER_ADAPTER.contains("add_child"));
        assert!(VIEWER_ADAPTER.contains("set_bounds"));
        assert!(!VIEWER_ADAPTER.contains("WindowBuilder"));
        assert!(!VIEWER_ADAPTER.contains("WebviewWindowBuilder"));
        assert!(!VIEWER_ADAPTER.contains("source_path"));
        assert!(!VIEWER_ADAPTER.contains("vertex_buffer"));
        assert!(!VIEWER_ADAPTER.contains("index_buffer"));
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
                "allow-analyze-model-source",
                "allow-activate-shop-resource-selection",
                "allow-cancel-model-analysis",
                "allow-desktop-contract",
                "allow-evaluate-draft-estimate",
                "allow-load-shop-settings",
                "allow-prepare-draft-estimate-proposal",
                "allow-save-shop-resource-catalog-draft",
                "allow-save-shop-settings",
                "allow-select-model-source",
                "allow-set-model-viewer-workspace",
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
        assert_eq!(config["bundle"]["active"], false);
        assert_eq!(
            config["bundle"]["icon"],
            serde_json::json!([
                "icons/32x32.png",
                "icons/128x128.png",
                "icons/128x128@2x.png",
                "icons/icon.png",
                "icons/icon.icns",
                "icons/icon.ico"
            ])
        );
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
        assert!(RUNTIME.contains(".set_title(MODEL_SOURCE_DIALOG_TITLE)"));
        assert!(RUNTIME.contains("Select model file"));
        assert!(!RUNTIME.contains("blocking_pick_file"));
    }

    #[test]
    fn geometry_analysis_is_async_and_accepts_only_the_path_free_contract_request() {
        assert!(RUNTIME.contains("async fn analyze_model_source"));
        assert!(RUNTIME.contains("AnalyzeModelSourceRequest"));
        assert!(RUNTIME.contains("spawn_blocking"));
        assert!(!RUNTIME.contains("source_path:"));
        assert!(!RUNTIME.contains("request: PathBuf"));
    }

    #[test]
    fn production_host_resolves_native_runtime_from_tauri_resources() {
        assert!(RUNTIME.contains(".path()"));
        assert!(RUNTIME.contains(".resource_dir()"));
        assert!(RUNTIME.contains("from_deployment_resource_directory"));
        assert!(!RUNTIME.contains("read_dir("));
        assert!(!RUNTIME.contains("PARTPROBE_GEOMETRY_WORKER"));
        assert!(!RUNTIME.contains("PARTPROBE_OCCT_ROOT"));
    }

    #[test]
    fn estimate_evaluation_and_cancellation_stay_in_typed_native_commands() {
        assert!(RUNTIME.contains("async fn evaluate_draft_estimate"));
        assert!(RUNTIME.contains("EvaluateDraftEstimateRequest"));
        assert!(RUNTIME.contains("async fn prepare_draft_estimate_proposal"));
        assert!(RUNTIME.contains("PrepareDraftEstimateProposalRequest"));
        assert!(RUNTIME.contains("fn cancel_model_analysis"));
        assert!(RUNTIME.contains("CancelModelAnalysisRequest"));
        assert!(!RUNTIME.contains("apply_pricing_policy"));
        assert!(!RUNTIME.contains("resolve_rate"));
    }

    #[test]
    fn settings_storage_is_host_owned_and_runs_off_the_ui_task() {
        assert!(RUNTIME.contains("async fn load_shop_settings"));
        assert!(RUNTIME.contains("async fn save_shop_settings"));
        assert!(RUNTIME.contains("app_data_dir()"));
        assert!(RUNTIME.contains("DesktopSettingsState::open"));
        assert!(RUNTIME.matches("spawn_blocking").count() >= 4);
        assert!(!RUNTIME.contains("database_path:"));
        assert!(!RUNTIME.contains("request.database"));
    }

    #[test]
    fn catalog_activation_command_uses_the_native_policy_audit_and_identity_boundary() {
        assert!(SETTINGS_ADAPTER.contains("DesktopCatalogAuthorizationPolicy"));
        assert!(SETTINGS_ADAPTER.contains("SqliteShopResourceCatalogAuthorizationAudit"));
        assert!(SETTINGS_ADAPTER.contains("unconfigured_catalog_policy"));
        assert!(RUNTIME.contains("async fn activate_shop_resource_selection"));
        assert!(RUNTIME.contains("async fn save_shop_resource_catalog_draft"));
        assert!(RUNTIME.contains("ActivateShopResourceSelectionRequest"));
        assert!(RUNTIME.contains("spawn_blocking"));
        assert!(SETTINGS_ADAPTER.contains("catalog_actor: Option<ActorId>"));
        assert!(SETTINGS_ADAPTER.contains("catalog_operation_evidence"));
        assert!(!RUNTIME.contains("actor_id"));
        assert!(!RUNTIME.contains("correlation_id"));
        assert!(!WEBVIEW.contains("COMMAND_ACTIVATE_SHOP_RESOURCE_SELECTION"));
        assert!(!WEBVIEW.contains("activate_shop_resource_selection"));
        assert!(!WEBVIEW.contains("save_shop_resource_catalog_draft"));
        assert!(WEBVIEW.contains("catalog-editor-boundary"));
        assert!(WEBVIEW.contains("Editing unavailable"));
        assert!(WEBVIEW.contains("settings-save-status"));
        assert!(WEBVIEW.contains("Save catalog draft"));
        assert!(WEBVIEW.contains("trusted operator identity is not configured"));
        assert!(!SETTINGS_ADAPTER.contains("std::env"));
        assert!(!SETTINGS_ADAPTER.contains("PARTPROBE_CATALOG"));
        assert_eq!(APPLICATION_COMMANDS.len(), 11);
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

    #[test]
    fn analysis_requires_the_current_selection_and_explicit_worker_configuration() {
        let state = DesktopSessionState::default();
        let source = state
            .retain_selected_path(std::env::temp_dir().join("fixture.step"))
            .expect("STEP selection must be retained");

        let stale = state
            .analyze_selected_source("selection-stale")
            .expect_err("stale selection must fail before configuration lookup");
        assert_eq!(
            stale.code,
            partprobe_desktop_contract::HostErrorCode::StaleSelection
        );

        let unavailable = state
            .analyze_selected_source(&source.selection_id)
            .expect_err("unconfigured developer worker must stay unavailable");
        assert_eq!(
            unavailable.code,
            partprobe_desktop_contract::HostErrorCode::AnalysisUnavailable
        );
        assert!(state.retained_analysis_identity().is_none());
    }

    #[test]
    fn cancellation_is_token_bound_and_signals_only_the_matching_active_analysis() {
        let state = DesktopSessionState::default();
        let cancellation = Arc::new(AtomicBool::new(false));
        *state.active_analysis.lock().expect("active state") = Some(ActiveAnalysis {
            selection_id: "selection-1".to_owned(),
            analysis_number: 7,
            cancellation: Arc::clone(&cancellation),
        });

        let stale = state
            .cancel_model_analysis(&CancelModelAnalysisRequest {
                selection_id: "selection-stale".to_owned(),
            })
            .expect("stale cancellation is an explicit no-op");
        assert!(!stale.cancellation_requested);
        assert!(!cancellation.load(Ordering::Acquire));

        let matching = state
            .cancel_model_analysis(&CancelModelAnalysisRequest {
                selection_id: "selection-1".to_owned(),
            })
            .expect("matching cancellation must be acknowledged");
        assert!(matching.cancellation_requested);
        assert!(cancellation.load(Ordering::Acquire));
    }

    #[cfg(feature = "desktop-host")]
    #[test]
    #[ignore = "requires a verified pinned native runtime and worker workspace"]
    fn gui5_configured_worker_runs_real_step_through_retained_estimate_session() {
        let adapter = crate::analysis::DesktopAnalysisConfiguration::from_environment()
            .and_then(crate::analysis::DesktopAnalysisConfiguration::build_adapter)
            .expect("GUI-5 requires verified native-runtime/workspace configuration");
        assert_real_step_reaches_retained_estimate(adapter);
    }

    #[cfg(feature = "desktop-host")]
    #[test]
    #[ignore = "requires an explicit verified native runtime and worker workspace"]
    fn gui5_configured_worker_returns_real_mesh_results_without_estimate_authority() {
        let adapter = crate::analysis::DesktopAnalysisConfiguration::from_environment()
            .and_then(crate::analysis::DesktopAnalysisConfiguration::build_adapter)
            .expect("GUI-5 requires verified native-runtime/workspace configuration");
        assert_real_meshes_reach_unavailable_estimates(adapter);
    }

    #[cfg(feature = "desktop-host")]
    #[test]
    #[ignore = "requires an extracted packaged native runtime and worker workspace"]
    fn packaged_resource_worker_runs_real_step_through_retained_estimate_session() {
        let resource_directory = std::env::var_os("PARTPROBE_DESKTOP_RESOURCE_DIRECTORY")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .expect("packaged smoke requires an explicit extracted resource directory");
        let adapter =
            crate::analysis::DesktopAnalysisConfiguration::from_deployment_resource_directory(
                &resource_directory,
            )
            .and_then(crate::analysis::DesktopAnalysisConfiguration::build_adapter)
            .expect("packaged resource runtime/workspace must verify before analysis");
        assert_real_step_reaches_retained_estimate(adapter);
    }

    #[cfg(feature = "desktop-host")]
    #[test]
    #[ignore = "requires an extracted packaged native runtime and worker workspace"]
    fn packaged_resource_worker_reaches_reviewed_model_sensitive_proposal_estimate() {
        let resource_directory = std::env::var_os("PARTPROBE_DESKTOP_RESOURCE_DIRECTORY")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .expect("packaged smoke requires an explicit extracted resource directory");
        let adapter =
            crate::analysis::DesktopAnalysisConfiguration::from_deployment_resource_directory(
                &resource_directory,
            )
            .and_then(crate::analysis::DesktopAnalysisConfiguration::build_adapter)
            .expect("packaged resource runtime/workspace must verify before analysis");
        assert_real_step_reaches_reviewed_proposal_estimate(adapter);
    }

    #[cfg(feature = "desktop-host")]
    fn assert_real_step_reaches_retained_estimate(
        adapter: DesktopAnalysisAdapter<GeometryWorkerSupervisor>,
    ) {
        let state = DesktopSessionState::with_analysis_adapter(adapter);
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/rectangular_prism_12x8x5.step")
            .canonicalize()
            .expect("GUI-5 STEP fixture must exist");
        let source = state
            .retain_selected_path(fixture.clone())
            .expect("fixture must be retained behind an opaque selection token");

        let analysis = state
            .analyze_selected_source(&source.selection_id)
            .expect("configured worker must analyze the real STEP fixture");

        let partprobe_desktop_contract::ProvisionalGeometryFacts::ExactBrep(geometry) =
            &analysis.geometry
        else {
            panic!("configured STEP analysis must retain exact-B-rep evidence");
        };
        assert_eq!(geometry.surface_area_mm2, "392");
        assert_eq!(geometry.enclosed_volume_mm3, "480");
        assert_eq!(geometry.center_of_mass_mm, ["6", "4", "2.5"]);
        let request =
            crate::estimate::complete_test_request(&source.selection_id, &analysis.analysis_id);
        let evaluation = state
            .evaluate_draft_estimate(&request, None)
            .expect("complete inputs must evaluate through the retained native session");
        assert_eq!(
            evaluation.state,
            partprobe_desktop_contract::DraftEstimateEvaluationState::Available
        );
        assert_eq!(
            evaluation
                .result
                .expect("available result must contain a trace")
                .rounded_selling_price,
            "702"
        );
        let serialized = serde_json::to_string(&analysis).expect("analysis must serialize");
        assert!(!serialized.contains(fixture.to_string_lossy().as_ref()));
        assert!(!serialized.contains("fixtures/models"));
    }

    #[cfg(feature = "desktop-host")]
    fn assert_real_step_reaches_reviewed_proposal_estimate(
        adapter: DesktopAnalysisAdapter<GeometryWorkerSupervisor>,
    ) {
        let directory = TestDirectory::create("desktop-packaged-proposal").unwrap();
        let settings_state = DesktopSettingsState::open(&directory.path().join("settings.sqlite3"))
            .expect("temporary Settings repository must open");
        let settings_request = live_test_settings_request();
        settings_state
            .save(&settings_request)
            .expect("reviewed synthetic Settings draft must save");
        let settings = settings_state
            .current_settings_draft()
            .expect("saved Settings draft must reload");

        let state = DesktopSessionState::with_analysis_adapter(adapter);
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/rectangular_prism_12x8x5.step")
            .canonicalize()
            .expect("proposal STEP fixture must exist");
        let source = state
            .retain_selected_path(fixture.clone())
            .expect("fixture must be retained behind an opaque selection token");
        let analysis = state
            .analyze_selected_source(&source.selection_id)
            .expect("packaged worker must analyze the real STEP fixture");
        let proposal = state
            .prepare_draft_estimate_proposal(
                &PrepareDraftEstimateProposalRequest {
                    selection_id: source.selection_id.clone(),
                    analysis_id: analysis.analysis_id.clone(),
                },
                &settings,
            )
            .expect("proposal command must return a typed state");
        let DraftEstimateProposalEvaluation::Available { proposal } = proposal else {
            panic!("one-solid STEP plus starter resources must produce a reviewable proposal");
        };
        assert_eq!(proposal.model_extents_mm, ["12", "8", "5"]);
        assert_eq!(proposal.blank_dimensions_mm, ["15", "11", "7"]);
        assert_eq!(proposal.blank_volume_mm3, "1155");
        assert_eq!(proposal.removed_volume_mm3, "675");
        assert_eq!(proposal.unit_stock_material_cost, "0.02650725");

        let mut request =
            crate::estimate::complete_test_request(&source.selection_id, &analysis.analysis_id);
        request.rates = settings_request.rates;
        request.pricing = settings_request.pricing;
        request.proposal_adoption = Some(DraftEstimateProposalAdoptionInput {
            settings_revision: proposal.settings_revision,
            library_id: proposal.library_id.clone(),
            library_version: proposal.library_version,
            proposal_values_reviewed: true,
            coarse_limitations_accepted: true,
            review_reason: "reviewed synthetic values and named exclusions for package smoke"
                .to_owned(),
            deliver_quantity: "1".to_owned(),
            planned_spares: "0".to_owned(),
            destructive_samples: "0".to_owned(),
        });
        let evaluation = state
            .evaluate_draft_estimate(&request, Some(&settings))
            .expect("explicit proposal adoption must evaluate through the retained session");
        assert_eq!(
            evaluation.state,
            partprobe_desktop_contract::DraftEstimateEvaluationState::Available
        );
        let serialized = serde_json::to_string(&evaluation).expect("result must serialize");
        assert!(!serialized.contains(fixture.to_string_lossy().as_ref()));
        assert!(!serialized.contains("fixtures/models"));
        let result = evaluation
            .result
            .expect("available proposal estimate must contain a trace");
        assert_ne!(
            result.rounded_selling_price, "702",
            "proposal adoption must not fall back to the fixed manual fixture"
        );
        assert_eq!(result.make_quantity, 1);
        assert_eq!(result.input_trace.stock_volume_mm3, "1155");
        assert_eq!(result.input_trace.density_kg_per_mm3, "0.0000027");
        assert_eq!(result.input_trace.purchased_material, "0.02650725");
        let adoption = result
            .proposal_adoption
            .expect("proposal estimate must retain adoption evidence");
        assert_eq!(adoption.settings_revision, 1);
        assert!(
            adoption
                .excluded_inputs
                .contains(&"tooling_fixture_and_outside_processing".to_owned())
        );
    }

    #[cfg(feature = "desktop-host")]
    fn live_test_settings_request() -> SaveShopSettingsRequest {
        SaveShopSettingsRequest {
            expected_revision: None,
            changed_by: "package-smoke-reviewer".to_owned(),
            change_reason: "reviewed synthetic values for governed package smoke".to_owned(),
            rates: DeveloperRateInputFields {
                confirmed_for_session: true,
                rate_card_id: "synthetic-demo-rates".to_owned(),
                rate_card_version: "1".to_owned(),
                effective_on: "2026-09-10".to_owned(),
                currency: "USD".to_owned(),
                setup_labor_per_hour: "85".to_owned(),
                programming_per_hour: "95".to_owned(),
                run_labor_per_hour: "45".to_owned(),
                machine_per_hour: "110".to_owned(),
                quality_inspection_per_hour: "75".to_owned(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: true,
                pricing_policy_id: "synthetic-demo-pricing".to_owned(),
                pricing_policy_version: "1".to_owned(),
                markup_rate: "0.25".to_owned(),
                optional_price_floor: String::new(),
                optional_minimum_order: String::new(),
                rounding_decimal_places: "2".to_owned(),
            },
            resources: Some(ShopResourceInputFields {
                confirmed_for_draft: true,
                library_id: "synthetic-demo-resources".to_owned(),
                library_version: "1".to_owned(),
                material_id: "synthetic-al-6061-t6".to_owned(),
                material_version: "1".to_owned(),
                material_family: "Aluminum".to_owned(),
                material_grade: "6061".to_owned(),
                optional_material_specification: String::new(),
                optional_material_condition: "T6".to_owned(),
                density_kg_per_m3: "2700".to_owned(),
                material_source: "synthetic-demo-handbook".to_owned(),
                offer_id: "synthetic-al-6061-offer".to_owned(),
                offer_version: "1".to_owned(),
                supplier: "Synthetic demo supplier".to_owned(),
                material_price_per_kg: "8.50".to_owned(),
                offer_effective_on: "2026-09-10".to_owned(),
                offer_source: "synthetic-demo-offer".to_owned(),
                stock_profile_id: "synthetic-rectangular-stock".to_owned(),
                stock_profile_version: "1".to_owned(),
                stock_form: "rectangular".to_owned(),
                stock_allowance_x_mm: "3".to_owned(),
                stock_allowance_y_mm: "3".to_owned(),
                stock_allowance_z_mm: "2".to_owned(),
                stock_source: "synthetic-demo-stock-policy".to_owned(),
                machine_id: "synthetic-vmc".to_owned(),
                machine_version: "1".to_owned(),
                machine_name: "Synthetic VMC".to_owned(),
                process_class: "milling".to_owned(),
                machine_envelope_x_mm: "762".to_owned(),
                machine_envelope_y_mm: "508".to_owned(),
                machine_envelope_z_mm: "508".to_owned(),
                machine_source: "synthetic-demo-machine-profile".to_owned(),
                runtime_profile_id: "synthetic-vmc-al-runtime".to_owned(),
                runtime_profile_version: "1".to_owned(),
                removal_rate_mm3_per_minute: "12000".to_owned(),
                setup_minutes: "60".to_owned(),
                programming_minutes: "45".to_owned(),
                load_unload_minutes: "3".to_owned(),
                inspection_minutes: "15".to_owned(),
                runtime_source: "synthetic-demo-runtime-profile".to_owned(),
            }),
        }
    }

    #[cfg(feature = "desktop-host")]
    fn assert_real_meshes_reach_unavailable_estimates(
        adapter: DesktopAnalysisAdapter<GeometryWorkerSupervisor>,
    ) {
        let state = DesktopSessionState::with_analysis_adapter(adapter);
        for (fixture_name, expected_format, expected_units, expected_basis) in [
            (
                "cube_10mm_ascii.stl",
                ModelSourceFormat::Stl,
                partprobe_desktop_contract::ModelLengthUnit::Unknown,
                partprobe_desktop_contract::MeshMeasurementBasis::SourceCoordinates,
            ),
            (
                "cube_1cm_translated.3mf",
                ModelSourceFormat::ThreeMf,
                partprobe_desktop_contract::ModelLengthUnit::Centimeter,
                partprobe_desktop_contract::MeshMeasurementBasis::CanonicalMillimeters,
            ),
        ] {
            let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../fixtures/models")
                .join(fixture_name)
                .canonicalize()
                .expect("governed mesh fixture must exist");
            let source = state
                .retain_selected_path(fixture.clone())
                .expect("fixture must be retained behind an opaque selection token");
            let analysis = state
                .analyze_selected_source(&source.selection_id)
                .expect("configured worker must analyze the governed mesh fixture");

            let partprobe_desktop_contract::ProvisionalGeometryFacts::Mesh(geometry) =
                &analysis.geometry
            else {
                panic!("configured mesh analysis must retain mesh evidence");
            };
            assert_eq!(geometry.detected_format, expected_format);
            assert_eq!(geometry.source_units, expected_units);
            assert_eq!(geometry.measurement_basis, expected_basis);
            assert_eq!(
                geometry.welding_status,
                partprobe_desktop_contract::MeshWeldingStatus::NotApplied
            );
            assert!(geometry.enclosed_volume.is_some());
            assert!(geometry.center_of_mass.is_some());
            assert!(analysis.estimate.reason.contains("not authorized"));

            let request =
                crate::estimate::complete_test_request(&source.selection_id, &analysis.analysis_id);
            let evaluation = state
                .evaluate_draft_estimate(&request, None)
                .expect("mesh estimate request must return an explicit unavailable state");
            assert_eq!(
                evaluation.state,
                partprobe_desktop_contract::DraftEstimateEvaluationState::Unavailable
            );
            assert!(
                evaluation
                    .reason
                    .expect("mesh estimate must explain unavailability")
                    .contains("not authorized")
            );

            let serialized = serde_json::to_string(&analysis).expect("analysis must serialize");
            assert!(!serialized.contains(fixture.to_string_lossy().as_ref()));
            assert!(!serialized.contains("fixtures/models"));
        }
    }
}
