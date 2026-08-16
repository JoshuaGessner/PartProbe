use std::path::PathBuf;

use partprobe_desktop_contract::{
    AnalysisCancellationAcknowledgement, AnalyzeModelSourceRequest, CancelModelAnalysisRequest,
    DesktopContract, DraftEstimateEvaluation, EVENT_MODEL_SOURCE_SELECTED,
    EvaluateDraftEstimateRequest, HostCommandError, ModelAnalysisResult, ModelSourceSelectedEvent,
    ModelSourceSelection,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::DesktopSessionState;
use crate::analysis::DesktopAnalysisConfiguration;

const MODEL_SOURCE_DIALOG_TITLE: &str = "Select STEP model";

#[tauri::command]
fn desktop_contract() -> DesktopContract {
    DesktopContract::current()
}

#[tauri::command]
async fn select_model_source(
    app: tauri::AppHandle,
) -> Result<ModelSourceSelection, HostCommandError> {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    app.dialog()
        .file()
        .set_title(MODEL_SOURCE_DIALOG_TITLE)
        .add_filter("STEP model", &["step", "stp"])
        .pick_file(move |selected| {
            let _ = sender.try_send(selected);
        });
    let selected = receiver
        .recv()
        .await
        .ok_or_else(|| HostCommandError::host_state_unavailable("GUI3-SELECTION-DIALOG"))?;
    let Some(selected) = selected else {
        return Ok(ModelSourceSelection::Cancelled);
    };
    let path = desktop_path(selected)?;
    let state = app.state::<DesktopSessionState>();
    let source = state.retain_selected_path(path)?;
    let event = ModelSourceSelectedEvent {
        source: source.clone(),
    };
    app.emit_to("main", EVENT_MODEL_SOURCE_SELECTED, event)
        .map_err(|_| HostCommandError::host_state_unavailable("GUI3-SELECTION-EVENT"))?;

    Ok(ModelSourceSelection::Selected { source })
}

#[tauri::command]
async fn analyze_model_source(
    app: tauri::AppHandle,
    request: AnalyzeModelSourceRequest,
) -> Result<ModelAnalysisResult, HostCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<DesktopSessionState>()
            .analyze_selected_source(&request.selection_id)
    })
    .await
    .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-TASK"))?
}

#[tauri::command]
fn cancel_model_analysis(
    app: tauri::AppHandle,
    request: CancelModelAnalysisRequest,
) -> Result<AnalysisCancellationAcknowledgement, HostCommandError> {
    app.state::<DesktopSessionState>()
        .cancel_model_analysis(&request)
}

#[tauri::command]
async fn evaluate_draft_estimate(
    app: tauri::AppHandle,
    request: EvaluateDraftEstimateRequest,
) -> Result<DraftEstimateEvaluation, HostCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<DesktopSessionState>()
            .evaluate_draft_estimate(&request)
    })
    .await
    .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ESTIMATE-TASK"))?
}

fn desktop_path(selected: FilePath) -> Result<PathBuf, HostCommandError> {
    match selected {
        FilePath::Path(path) => Ok(path),
        FilePath::Url(_) => Err(HostCommandError::invalid_selection(
            "GUI3-SELECTION-NONLOCAL",
        )),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let session_state = app
                .path()
                .resource_dir()
                .map_err(|_| ())
                .and_then(|resource_directory| {
                    DesktopAnalysisConfiguration::from_deployment_resource_directory(
                        &resource_directory,
                    )
                    .map_err(|_| ())
                })
                .and_then(|configuration| configuration.build_adapter().map_err(|_| ()))
                .map_or_else(
                    |_| DesktopSessionState::default(),
                    DesktopSessionState::with_analysis_adapter,
                );
            if !app.manage(session_state) {
                return Err("desktop session state is already managed".into());
            }
            let window = app
                .get_webview_window("main")
                .ok_or("configured main window is missing")?;
            window.set_title("PartProbe")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_contract,
            select_model_source,
            analyze_model_source,
            cancel_model_analysis,
            evaluate_draft_estimate
        ])
        .run(tauri::generate_context!())
        .expect("PartProbe desktop host failed");
}
