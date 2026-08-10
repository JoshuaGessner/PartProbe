#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use partprobe_desktop_contract::{
    AnalysisCancellationAcknowledgement, AnalysisStatus, CancelModelAnalysisRequest,
    DraftEstimateEvaluation, EvaluateDraftEstimateRequest, HostCommandError, ModelAnalysisResult,
    ModelSourceFormat, PersistenceAvailability, SelectedModelSource,
};
use partprobe_geometry_import::GeometryWorkerSupervisor;

mod analysis;
mod estimate;

use analysis::DesktopAnalysisAdapter;

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
        let retained = RetainedModelSource { selection_id, path };
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
        let outcome = adapter.analyze(selection_id, &selected.path, analysis_number, &cancellation);
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
        estimate::evaluate_draft_estimate(&mut retained.session, request)
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
                "allow-analyze-model-source",
                "allow-cancel-model-analysis",
                "allow-desktop-contract",
                "allow-evaluate-draft-estimate",
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

    #[test]
    fn geometry_analysis_is_async_and_accepts_only_the_path_free_contract_request() {
        assert!(RUNTIME.contains("async fn analyze_model_source"));
        assert!(RUNTIME.contains("AnalyzeModelSourceRequest"));
        assert!(RUNTIME.contains("spawn_blocking"));
        assert!(!RUNTIME.contains("source_path:"));
        assert!(!RUNTIME.contains("request: PathBuf"));
    }

    #[test]
    fn estimate_evaluation_and_cancellation_stay_in_typed_native_commands() {
        assert!(RUNTIME.contains("async fn evaluate_draft_estimate"));
        assert!(RUNTIME.contains("EvaluateDraftEstimateRequest"));
        assert!(RUNTIME.contains("fn cancel_model_analysis"));
        assert!(RUNTIME.contains("CancelModelAnalysisRequest"));
        assert!(!RUNTIME.contains("apply_pricing_policy"));
        assert!(!RUNTIME.contains("resolve_rate"));
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

        assert_eq!(analysis.geometry.surface_area_mm2, "392");
        assert_eq!(analysis.geometry.enclosed_volume_mm3, "480");
        assert_eq!(analysis.geometry.center_of_mass_mm, ["6", "4", "2.5"]);
        let request =
            crate::estimate::complete_test_request(&source.selection_id, &analysis.analysis_id);
        let evaluation = state
            .evaluate_draft_estimate(&request)
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
}
