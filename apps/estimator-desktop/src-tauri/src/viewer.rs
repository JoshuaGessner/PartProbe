#[cfg(any(feature = "viewer-spike", test))]
use std::ffi::OsString;
#[cfg(feature = "viewer-spike")]
use std::sync::{Arc, Mutex};

use partprobe_desktop_contract::{
    HostCommandError, ModelViewerAvailability, ModelViewerWorkspaceResult,
    SetModelViewerViewRequest, SetModelViewerWorkspaceRequest,
};
#[cfg(feature = "viewer-spike")]
use partprobe_desktop_contract::{ModelViewerStandardView, ModelViewerWorkspaceMode};
#[cfg(feature = "viewer-spike")]
use partprobe_geometry_core::GEOMETRY_DISPLAY_SCENE_REFERENCE;
use partprobe_geometry_import::ValidatedGeometryDisplayScene;
#[cfg(feature = "viewer-spike")]
use partprobe_model_viewer::{
    NativeSurfaceRenderer, StandardView, SurfaceFrameStatus, SurfaceViewport,
};
#[cfg(feature = "viewer-spike")]
use tauri::{Manager, WebviewUrl, webview::WebviewBuilder};

#[cfg(any(feature = "viewer-spike", test))]
const SYNTHETIC_VIEWER_FLAG: &str = "--vis1-synthetic-viewer";
#[cfg(feature = "viewer-spike")]
const SYNTHETIC_SCENE_REFERENCE: &str = "synthetic-viewer-spike-v1";
#[cfg(feature = "viewer-spike")]
const SYNTHETIC_VIEWER_NOTICE: &str =
    "Synthetic renderer validation scene; it is not the selected model or stock authority.";
#[cfg(feature = "viewer-spike")]
const PENDING_VIEWER_NOTICE: &str = "Selected-model display is waiting for a validated analysis derivative; no prior model is shown.";
#[cfg(feature = "viewer-spike")]
const SOURCE_VIEWER_NOTICE: &str = "Selected exact B-rep model, tessellated for display from the current validated analysis. Stock is hidden until placement is governed.";
#[cfg(feature = "viewer-spike")]
const UNAVAILABLE_VIEWER_NOTICE: &str = "Selected-model display is unavailable; accepted analysis and estimate evidence remain usable. Stock is hidden until placement is governed.";
#[cfg(feature = "viewer-spike")]
const VIEWER_SIDEBAR_LOGICAL_WIDTH: f64 = 400.0;
#[cfg(feature = "viewer-spike")]
const VIEWER_WORKSPACE_WEBVIEW_LABEL: &str = "model-viewer-workspace";

#[derive(Clone)]
#[cfg(feature = "viewer-spike")]
pub(super) struct DesktopModelViewerState {
    inner: Arc<Mutex<ModelViewerRuntime>>,
}

#[derive(Clone)]
#[cfg(not(feature = "viewer-spike"))]
pub(super) struct DesktopModelViewerState;

#[cfg(feature = "viewer-spike")]
struct ModelViewerRuntime {
    #[cfg(feature = "viewer-spike")]
    active: bool,
    #[cfg(feature = "viewer-spike")]
    window: Option<tauri::Window>,
    #[cfg(feature = "viewer-spike")]
    workspace: Option<tauri::Webview>,
    #[cfg(feature = "viewer-spike")]
    renderer: Option<NativeSurfaceRenderer>,
    #[cfg(feature = "viewer-spike")]
    scene_status: ViewerSceneStatus,
}

#[cfg(feature = "viewer-spike")]
enum ViewerSceneStatus {
    Synthetic,
    Pending {
        selection_id: String,
    },
    SourceBound {
        selection_id: String,
        analysis_id: String,
    },
    Unavailable {
        selection_id: String,
    },
}

#[cfg(feature = "viewer-spike")]
impl ViewerSceneStatus {
    fn selection_matches(&self, selection_id: &str) -> bool {
        match self {
            Self::Pending {
                selection_id: current,
            }
            | Self::SourceBound {
                selection_id: current,
                ..
            }
            | Self::Unavailable {
                selection_id: current,
            } => current == selection_id,
            Self::Synthetic => false,
        }
    }

    fn visible_summary(&self) -> (Option<String>, &'static str) {
        match self {
            Self::Synthetic => (
                Some(SYNTHETIC_SCENE_REFERENCE.to_owned()),
                SYNTHETIC_VIEWER_NOTICE,
            ),
            Self::Pending { .. } => (None, PENDING_VIEWER_NOTICE),
            Self::SourceBound { analysis_id, .. } => {
                let _ = analysis_id;
                (
                    Some(GEOMETRY_DISPLAY_SCENE_REFERENCE.to_owned()),
                    SOURCE_VIEWER_NOTICE,
                )
            }
            Self::Unavailable { .. } => (None, UNAVAILABLE_VIEWER_NOTICE),
        }
    }
}

impl DesktopModelViewerState {
    pub(super) fn unavailable() -> Self {
        #[cfg(not(feature = "viewer-spike"))]
        {
            Self
        }
        #[cfg(feature = "viewer-spike")]
        Self {
            inner: Arc::new(Mutex::new(ModelViewerRuntime {
                #[cfg(feature = "viewer-spike")]
                active: false,
                #[cfg(feature = "viewer-spike")]
                window: None,
                #[cfg(feature = "viewer-spike")]
                workspace: None,
                #[cfg(feature = "viewer-spike")]
                renderer: None,
                #[cfg(feature = "viewer-spike")]
                scene_status: ViewerSceneStatus::Synthetic,
            })),
        }
    }

    pub(super) fn availability(&self) -> ModelViewerAvailability {
        #[cfg(feature = "viewer-spike")]
        if self.inner.lock().is_ok_and(|runtime| {
            runtime.renderer.is_some() && runtime.window.is_some() && runtime.workspace.is_some()
        }) {
            return ModelViewerAvailability::DeveloperPreview;
        }
        ModelViewerAvailability::Unavailable
    }

    pub(super) fn set_workspace(
        &self,
        request: SetModelViewerWorkspaceRequest,
    ) -> Result<ModelViewerWorkspaceResult, HostCommandError> {
        #[cfg(not(feature = "viewer-spike"))]
        {
            let _ = request;
            Err(HostCommandError::host_state_unavailable(
                "VIS1-VIEWER-NOT-CONFIGURED",
            ))
        }

        #[cfg(feature = "viewer-spike")]
        {
            let mut runtime = self
                .inner
                .lock()
                .map_err(|_| HostCommandError::host_state_unavailable("VIS1-VIEWER-STATE"))?;
            if runtime.renderer.is_none() || runtime.window.is_none() || runtime.workspace.is_none()
            {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-VIEWER-NOT-CONFIGURED",
                ));
            }
            let previous_active = runtime.active;
            runtime.active = request.mode == ModelViewerWorkspaceMode::Visible;
            if let Err(error) = apply_layout(&mut runtime) {
                runtime.active = previous_active;
                let _ = apply_layout(&mut runtime);
                return Err(error);
            }
            Ok(workspace_result(&runtime, request.mode))
        }
    }

    pub(super) fn set_view(
        &self,
        request: SetModelViewerViewRequest,
    ) -> Result<ModelViewerWorkspaceResult, HostCommandError> {
        #[cfg(not(feature = "viewer-spike"))]
        {
            let _ = request;
            Err(HostCommandError::host_state_unavailable(
                "VIS1-VIEWER-NOT-CONFIGURED",
            ))
        }

        #[cfg(feature = "viewer-spike")]
        {
            let mut runtime = self
                .inner
                .lock()
                .map_err(|_| HostCommandError::host_state_unavailable("VIS1-VIEWER-STATE"))?;
            if runtime.renderer.is_none() || runtime.window.is_none() || runtime.workspace.is_none()
            {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-VIEWER-NOT-CONFIGURED",
                ));
            }
            if !runtime.active {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-VIEWER-NOT-VISIBLE",
                ));
            }
            let renderer = runtime.renderer.as_mut().ok_or_else(|| {
                HostCommandError::host_state_unavailable("VIS1-VIEWER-NOT-CONFIGURED")
            })?;
            renderer
                .set_view(renderer_view(request.view))
                .map_err(|_| HostCommandError::host_state_unavailable("VIS1-SURFACE-RENDER"))?;
            Ok(workspace_result(
                &runtime,
                ModelViewerWorkspaceMode::Visible,
            ))
        }
    }

    pub(super) fn begin_selection(&self, selection_id: &str) {
        #[cfg(feature = "viewer-spike")]
        if let Ok(mut runtime) = self.inner.lock() {
            if let Some(renderer) = runtime.renderer.as_mut() {
                let _ = renderer.clear_source_scene();
            }
            runtime.scene_status = ViewerSceneStatus::Pending {
                selection_id: selection_id.to_owned(),
            };
        }
        #[cfg(not(feature = "viewer-spike"))]
        let _ = selection_id;
    }

    pub(super) fn accept_analysis_scene(
        &self,
        selection_id: &str,
        analysis_id: &str,
        scene: Option<ValidatedGeometryDisplayScene>,
    ) {
        #[cfg(feature = "viewer-spike")]
        if let Ok(mut runtime) = self.inner.lock() {
            if !runtime.scene_status.selection_matches(selection_id) {
                return;
            }
            let accepted = match (runtime.renderer.as_mut(), scene) {
                (Some(renderer), Some(scene)) => {
                    if renderer.set_source_scene(scene).is_ok() {
                        true
                    } else {
                        // A rejected or unrenderable replacement must not leave either the
                        // preceding selection or a partially installed source scene visible.
                        let _ = renderer.clear_source_scene();
                        false
                    }
                }
                (Some(renderer), None) => {
                    let _ = renderer.clear_source_scene();
                    false
                }
                (None, _) => false,
            };
            runtime.scene_status = if accepted {
                ViewerSceneStatus::SourceBound {
                    selection_id: selection_id.to_owned(),
                    analysis_id: analysis_id.to_owned(),
                }
            } else {
                ViewerSceneStatus::Unavailable {
                    selection_id: selection_id.to_owned(),
                }
            };
        }
        #[cfg(not(feature = "viewer-spike"))]
        let _ = (selection_id, analysis_id, scene);
    }
}

#[cfg(feature = "viewer-spike")]
fn workspace_result(
    runtime: &ModelViewerRuntime,
    mode: ModelViewerWorkspaceMode,
) -> ModelViewerWorkspaceResult {
    let (scene_reference, notice) = runtime.scene_status.visible_summary();
    let view = runtime
        .renderer
        .as_ref()
        .map_or(ModelViewerStandardView::Isometric, |renderer| {
            contract_view(renderer.view())
        });
    ModelViewerWorkspaceResult {
        mode,
        view,
        scene_reference: (mode == ModelViewerWorkspaceMode::Visible)
            .then_some(scene_reference)
            .flatten(),
        notice: notice.to_owned(),
    }
}

#[cfg(feature = "viewer-spike")]
const fn renderer_view(view: ModelViewerStandardView) -> StandardView {
    match view {
        ModelViewerStandardView::Isometric => StandardView::Isometric,
        ModelViewerStandardView::Front => StandardView::Front,
        ModelViewerStandardView::Top => StandardView::Top,
        ModelViewerStandardView::Right => StandardView::Right,
    }
}

#[cfg(feature = "viewer-spike")]
const fn contract_view(view: StandardView) -> ModelViewerStandardView {
    match view {
        StandardView::Isometric => ModelViewerStandardView::Isometric,
        StandardView::Front => ModelViewerStandardView::Front,
        StandardView::Top => ModelViewerStandardView::Top,
        StandardView::Right => ModelViewerStandardView::Right,
    }
}

pub(super) fn source_scene_requested() -> bool {
    #[cfg(feature = "viewer-spike")]
    {
        arguments_request_synthetic_viewer(&std::env::args_os().collect::<Vec<_>>())
    }
    #[cfg(not(feature = "viewer-spike"))]
    {
        false
    }
}

pub(super) fn configure_in_window_viewer(
    app: &mut tauri::App,
) -> Result<DesktopModelViewerState, Box<dyn std::error::Error>> {
    #[cfg(not(feature = "viewer-spike"))]
    {
        let _ = app;
        Ok(DesktopModelViewerState::unavailable())
    }

    #[cfg(feature = "viewer-spike")]
    {
        if !source_scene_requested() {
            return Ok(DesktopModelViewerState::unavailable());
        }

        let original_webview = app
            .get_webview_window("main")
            .ok_or("configured main window is missing")?;
        let window = app
            .get_window("main")
            .ok_or("configured main native window is missing")?;
        let size = window.inner_size()?;
        let renderer = NativeSurfaceRenderer::attach(
            window.clone(),
            size.width,
            size.height,
            StandardView::Isometric,
        )?;
        let scale_factor = window.scale_factor()?;
        let logical_size = size.to_logical::<f64>(scale_factor);
        let workspace = window.add_child(
            WebviewBuilder::new(
                VIEWER_WORKSPACE_WEBVIEW_LABEL,
                WebviewUrl::App("index.html".into()),
            )
            .auto_resize(),
            tauri::LogicalPosition::new(0.0, 0.0),
            logical_size,
        )?;
        if let Err(error) = original_webview.as_ref().hide() {
            let _ = workspace.close();
            return Err(error.into());
        }
        let state = DesktopModelViewerState {
            inner: Arc::new(Mutex::new(ModelViewerRuntime {
                active: false,
                window: Some(window.clone()),
                workspace: Some(workspace),
                renderer: Some(renderer),
                scene_status: ViewerSceneStatus::Synthetic,
            })),
        };
        let state_for_events = state.clone();
        window.on_window_event(move |event| match event {
            tauri::WindowEvent::Resized(_) | tauri::WindowEvent::ScaleFactorChanged { .. } => {
                if let Ok(mut runtime) = state_for_events.inner.lock() {
                    let _ = apply_layout(&mut runtime);
                }
            }
            tauri::WindowEvent::Focused(true) => {
                if let Ok(mut runtime) = state_for_events.inner.lock()
                    && runtime.active
                    && let Some(renderer) = runtime.renderer.as_mut()
                {
                    let _ = renderer.render();
                }
            }
            tauri::WindowEvent::Destroyed => {
                if let Ok(mut runtime) = state_for_events.inner.lock() {
                    runtime.renderer.take();
                    runtime.workspace.take();
                    runtime.window.take();
                }
            }
            _ => {}
        });
        Ok(state)
    }
}

#[cfg(feature = "viewer-spike")]
fn apply_layout(runtime: &mut ModelViewerRuntime) -> Result<(), HostCommandError> {
    let window = runtime
        .window
        .clone()
        .ok_or_else(|| HostCommandError::host_state_unavailable("VIS1-MAIN-WINDOW"))?;
    let workspace = runtime
        .workspace
        .clone()
        .ok_or_else(|| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
    let surface_size = window
        .inner_size()
        .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WINDOW-SIZE"))?;
    if surface_size.width == 0 || surface_size.height == 0 {
        return Ok(());
    }
    let renderer = runtime
        .renderer
        .as_mut()
        .ok_or_else(|| HostCommandError::host_state_unavailable("VIS1-VIEWER-NOT-CONFIGURED"))?;
    if runtime.active {
        let scale_factor = window
            .scale_factor()
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WINDOW-SCALE"))?;
        let sidebar_width = (VIEWER_SIDEBAR_LOGICAL_WIDTH * scale_factor).round() as u32;
        let viewport_width = surface_size
            .width
            .checked_sub(sidebar_width)
            .ok_or_else(|| HostCommandError::host_state_unavailable("VIS1-WINDOW-TOO-SMALL"))?;
        workspace
            .set_auto_resize(false)
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
        let requested_bounds = tauri::Rect {
            position: tauri::PhysicalPosition::new(0, 0).into(),
            size: tauri::PhysicalSize::new(sidebar_width, surface_size.height).into(),
        };
        workspace
            .set_bounds(requested_bounds)
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
        renderer
            .resize_with_viewport(
                surface_size.width,
                surface_size.height,
                SurfaceViewport {
                    x: sidebar_width,
                    y: 0,
                    width: viewport_width,
                    height: surface_size.height,
                },
            )
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-SURFACE-RENDER"))?;
    } else {
        workspace
            .set_bounds(tauri::Rect {
                position: tauri::PhysicalPosition::new(0, 0).into(),
                size: surface_size.into(),
            })
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
        workspace
            .set_auto_resize(true)
            .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
        match renderer.resize(surface_size.width, surface_size.height) {
            Ok(SurfaceFrameStatus::Presented | SurfaceFrameStatus::Skipped) => {}
            Err(_) => {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-SURFACE-RENDER",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(any(feature = "viewer-spike", test))]
fn arguments_request_synthetic_viewer(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .any(|argument| argument == SYNTHETIC_VIEWER_FLAG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_developer_flag_requests_the_synthetic_viewer() {
        assert!(arguments_request_synthetic_viewer(&[
            "partprobe".into(),
            SYNTHETIC_VIEWER_FLAG.into(),
        ]));
        assert!(!arguments_request_synthetic_viewer(&[
            "partprobe".into(),
            "--vis1-synthetic-viewer-extra".into(),
        ]));
    }

    #[test]
    fn unconfigured_state_reports_no_viewer() {
        assert_eq!(
            DesktopModelViewerState::unavailable().availability(),
            ModelViewerAvailability::Unavailable
        );
    }

    #[test]
    fn unconfigured_state_rejects_standard_view_changes() {
        let error = DesktopModelViewerState::unavailable()
            .set_view(SetModelViewerViewRequest {
                view: partprobe_desktop_contract::ModelViewerStandardView::Front,
            })
            .expect_err("an unconfigured viewer must reject camera changes");

        assert_eq!(error.diagnostic_id, "VIS1-VIEWER-NOT-CONFIGURED");
        assert!(!error.message.contains('/'));
        assert!(!error.message.contains('\\'));
    }

    #[cfg(feature = "viewer-spike")]
    #[test]
    fn source_bound_workspace_summary_is_path_free_and_hides_stock_authority() {
        let status = ViewerSceneStatus::SourceBound {
            selection_id: "selection-1".to_owned(),
            analysis_id: "analysis-1".to_owned(),
        };

        let (reference, notice) = status.visible_summary();

        assert_eq!(reference.as_deref(), Some(GEOMETRY_DISPLAY_SCENE_REFERENCE));
        assert!(notice.contains("Selected exact B-rep model"));
        assert!(notice.contains("Stock is hidden"));
        assert!(!notice.contains('/'));
        assert!(!notice.contains('\\'));
    }

    #[cfg(feature = "viewer-spike")]
    #[test]
    fn viewer_scene_status_is_bound_to_the_current_selection() {
        let pending = ViewerSceneStatus::Pending {
            selection_id: "selection-2".to_owned(),
        };
        let source = ViewerSceneStatus::SourceBound {
            selection_id: "selection-2".to_owned(),
            analysis_id: "analysis-2".to_owned(),
        };

        assert!(pending.selection_matches("selection-2"));
        assert!(!pending.selection_matches("selection-1"));
        assert!(source.selection_matches("selection-2"));
        assert!(!ViewerSceneStatus::Synthetic.selection_matches("selection-2"));
    }

    #[cfg(feature = "viewer-spike")]
    #[test]
    fn contract_standard_views_map_exactly_to_renderer_views() {
        for view in [
            ModelViewerStandardView::Isometric,
            ModelViewerStandardView::Front,
            ModelViewerStandardView::Top,
            ModelViewerStandardView::Right,
        ] {
            assert_eq!(contract_view(renderer_view(view)), view);
        }
    }
}
