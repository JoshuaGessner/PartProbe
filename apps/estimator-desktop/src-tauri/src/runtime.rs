use std::path::PathBuf;

use partprobe_desktop_contract::{
    AnalyzeModelSourceRequest, DesktopContract, EVENT_MODEL_SOURCE_SELECTED, HostCommandError,
    ModelAnalysisResult, ModelSourceSelectedEvent, ModelSourceSelection,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::DesktopSessionState;
use crate::analysis::DesktopAnalysisConfiguration;

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

fn desktop_path(selected: FilePath) -> Result<PathBuf, HostCommandError> {
    match selected {
        FilePath::Path(path) => Ok(path),
        FilePath::Url(_) => Err(HostCommandError::invalid_selection(
            "GUI3-SELECTION-NONLOCAL",
        )),
    }
}

pub fn run() {
    let session_state = DesktopAnalysisConfiguration::from_environment()
        .and_then(DesktopAnalysisConfiguration::build_adapter)
        .map_or_else(
            |_| DesktopSessionState::default(),
            DesktopSessionState::with_analysis_adapter,
        );
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(session_state)
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .ok_or("configured main window is missing")?;
            window.set_title("PartProbe")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_contract,
            select_model_source,
            analyze_model_source
        ])
        .run(tauri::generate_context!())
        .expect("PartProbe desktop host failed");
}
