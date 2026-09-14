use std::path::PathBuf;

use partprobe_desktop_contract::{
    ActivateShopResourceSelectionRequest, AnalysisCancellationAcknowledgement,
    AnalyzeModelSourceRequest, CancelModelAnalysisRequest, DesktopContract,
    DraftEstimateEvaluation, DraftEstimateProposalEvaluation, EVENT_MODEL_SOURCE_SELECTED,
    EvaluateDraftEstimateRequest, HostCommandError, ModelAnalysisResult, ModelSourceSelectedEvent,
    ModelSourceSelection, ModelViewerWorkspaceResult, PrepareDraftEstimateProposalRequest,
    SaveShopResourceCatalogDraftRequest, SaveShopSettingsRequest, SetModelViewerViewRequest,
    SetModelViewerWorkspaceRequest, ShopResourceCatalogActivationResult,
    ShopResourceCatalogDraftSaveResult, ShopSettingsState,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::analysis::DesktopAnalysisConfiguration;
use crate::viewer::DesktopModelViewerState;
use crate::{DesktopSessionState, DesktopSettingsState};

const MODEL_SOURCE_DIALOG_TITLE: &str = "Select model file";

#[tauri::command]
fn desktop_contract(app: tauri::AppHandle) -> DesktopContract {
    DesktopContract::current()
        .with_model_viewer(app.state::<DesktopModelViewerState>().availability())
}

#[tauri::command]
fn set_model_viewer_workspace(
    app: tauri::AppHandle,
    request: SetModelViewerWorkspaceRequest,
) -> Result<ModelViewerWorkspaceResult, HostCommandError> {
    app.state::<DesktopModelViewerState>()
        .set_workspace(request)
}

#[tauri::command]
fn set_model_viewer_view(
    app: tauri::AppHandle,
    request: SetModelViewerViewRequest,
) -> Result<ModelViewerWorkspaceResult, HostCommandError> {
    app.state::<DesktopModelViewerState>().set_view(request)
}

#[tauri::command]
async fn select_model_source(
    app: tauri::AppHandle,
) -> Result<ModelSourceSelection, HostCommandError> {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    app.dialog()
        .file()
        .set_title(MODEL_SOURCE_DIALOG_TITLE)
        .add_filter("Supported model", &["step", "stp", "stl", "3mf"])
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
    app.state::<DesktopModelViewerState>()
        .begin_selection(&source.selection_id);
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
    let analysis_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        analysis_app
            .state::<DesktopSessionState>()
            .analyze_selected_source(&request.selection_id)
    })
    .await
    .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ANALYSIS-TASK"))??;
    let scene = app
        .state::<DesktopSessionState>()
        .retained_display_scene(&result.selection_id, &result.analysis_id)?;
    app.state::<DesktopModelViewerState>()
        .accept_analysis_scene(&result.selection_id, &result.analysis_id, scene);
    Ok(result)
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
        let settings = if request.proposal_adoption.is_some() {
            Some(
                app.state::<DesktopSettingsState>()
                    .current_settings_draft()?,
            )
        } else {
            None
        };
        app.state::<DesktopSessionState>()
            .evaluate_draft_estimate(&request, settings.as_ref())
    })
    .await
    .map_err(|_| HostCommandError::host_state_unavailable("GUI4-ESTIMATE-TASK"))?
}

#[tauri::command]
async fn prepare_draft_estimate_proposal(
    app: tauri::AppHandle,
    request: PrepareDraftEstimateProposalRequest,
) -> Result<DraftEstimateProposalEvaluation, HostCommandError> {
    app.state::<DesktopModelViewerState>()
        .clear_proposed_stock();
    let task_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let settings = task_app
            .state::<DesktopSettingsState>()
            .current_settings_draft()?;
        task_app
            .state::<DesktopSessionState>()
            .prepare_draft_estimate_proposal(&request, &settings)
    })
    .await
    .map_err(|_| HostCommandError::host_state_unavailable("USE3-PROPOSAL-TASK"))??;
    if let DraftEstimateProposalEvaluation::Available { proposal } = &result
        && let Some(dimensions_mm) = proposal_blank_dimensions(&proposal.blank_dimensions_mm)
    {
        app.state::<DesktopModelViewerState>()
            .accept_proposed_stock(&proposal.selection_id, &proposal.analysis_id, dimensions_mm);
    }
    Ok(result)
}

#[tauri::command]
async fn load_shop_settings(app: tauri::AppHandle) -> Result<ShopSettingsState, HostCommandError> {
    tauri::async_runtime::spawn_blocking(move || app.state::<DesktopSettingsState>().load())
        .await
        .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOAD-TASK"))?
}

#[tauri::command]
async fn save_shop_settings(
    app: tauri::AppHandle,
    request: SaveShopSettingsRequest,
) -> Result<ShopSettingsState, HostCommandError> {
    let task_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        task_app.state::<DesktopSettingsState>().save(&request)
    })
    .await
    .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-SAVE-TASK"))??;
    app.state::<DesktopModelViewerState>()
        .clear_proposed_stock();
    Ok(result)
}

fn proposal_blank_dimensions(values: &[String; 3]) -> Option<[f32; 3]> {
    Some([
        values[0].parse().ok()?,
        values[1].parse().ok()?,
        values[2].parse().ok()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::proposal_blank_dimensions;

    #[test]
    fn proposal_stock_dimensions_map_only_complete_numeric_evidence() {
        assert_eq!(
            proposal_blank_dimensions(&["18.4".into(), "14.4".into(), "11.4".into()]),
            Some([18.4, 14.4, 11.4])
        );
        assert_eq!(
            proposal_blank_dimensions(&["18.4".into(), "not-a-number".into(), "11.4".into()]),
            None
        );
    }
}

#[tauri::command]
async fn activate_shop_resource_selection(
    app: tauri::AppHandle,
    request: ActivateShopResourceSelectionRequest,
) -> Result<ShopResourceCatalogActivationResult, HostCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<DesktopSettingsState>()
            .activate_shop_resource_selection(&request)
    })
    .await
    .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-ACTIVATION-TASK"))?
}

#[tauri::command]
async fn save_shop_resource_catalog_draft(
    app: tauri::AppHandle,
    request: SaveShopResourceCatalogDraftRequest,
) -> Result<ShopResourceCatalogDraftSaveResult, HostCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<DesktopSettingsState>()
            .save_shop_resource_catalog_draft(&request)
    })
    .await
    .map_err(|_| HostCommandError::settings_unavailable("USE2-CATALOG-DRAFT-TASK"))?
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
                .and_then(|configuration| {
                    configuration
                        .build_adapter_with_display_scene(crate::viewer::source_scene_requested())
                        .map_err(|_| ())
                })
                .map_or_else(
                    |_| DesktopSessionState::default(),
                    DesktopSessionState::with_analysis_adapter,
                );
            if !app.manage(session_state) {
                return Err("desktop session state is already managed".into());
            }
            let settings_state = app
                .path()
                .app_data_dir()
                .map_err(|_| ())
                .and_then(|directory| {
                    std::fs::create_dir_all(&directory).map_err(|_| ())?;
                    DesktopSettingsState::open(&directory.join("shop-settings.sqlite3"))
                        .map_err(|_| ())
                })
                .unwrap_or_else(|_| DesktopSettingsState::unavailable());
            if !app.manage(settings_state) {
                return Err("desktop settings state is already managed".into());
            }
            let window = app
                .get_webview_window("main")
                .ok_or("configured main window is missing")?;
            window.set_title("PartProbe")?;
            let viewer_state = crate::viewer::configure_in_window_viewer(app)?;
            if !app.manage(viewer_state) {
                return Err("desktop model-viewer state is already managed".into());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_contract,
            select_model_source,
            analyze_model_source,
            cancel_model_analysis,
            evaluate_draft_estimate,
            prepare_draft_estimate_proposal,
            load_shop_settings,
            save_shop_settings,
            activate_shop_resource_selection,
            save_shop_resource_catalog_draft,
            set_model_viewer_workspace,
            set_model_viewer_view
        ])
        .run(tauri::generate_context!())
        .expect("PartProbe desktop host failed");
}
