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
    NativeSurfaceRenderer, ProposedStockDimensions, StandardView, SurfaceFrameStatus,
    SurfaceViewport, ViewerError,
};
#[cfg(feature = "viewer-spike")]
use tauri::{Manager, WebviewUrl, webview::WebviewBuilder};

#[cfg(feature = "viewer-spike")]
const SYNTHETIC_SCENE_REFERENCE: &str = "synthetic-viewer-spike-v1";
#[cfg(feature = "viewer-spike")]
const SYNTHETIC_VIEWER_NOTICE: &str =
    "Synthetic renderer validation scene; it is not the selected model or stock authority.";
#[cfg(feature = "viewer-spike")]
const PENDING_VIEWER_NOTICE: &str = "Selected-model display is waiting for a validated analysis derivative; no prior model is shown.";
#[cfg(feature = "viewer-spike")]
const SOURCE_VIEWER_NOTICE: &str = "Selected exact B-rep model, tessellated for display from the current validated analysis. Proposed stock will appear after automatic estimate-input preparation succeeds.";
#[cfg(feature = "viewer-spike")]
const SOURCE_WITH_STOCK_VIEWER_NOTICE: &str = "Selected exact B-rep model with review-only proposed rectangular stock. Stock is source-axis aligned and centered under proposed-stock-display-placement-v1; standard size and availability remain unresolved.";
#[cfg(feature = "viewer-spike")]
const UNAVAILABLE_VIEWER_NOTICE: &str = "Selected-model display is unavailable; accepted analysis and estimate evidence remain usable. Proposed stock cannot be shown without the matching source display.";
#[cfg(feature = "viewer-spike")]
const STARTUP_LIMITED_VIEWER_NOTICE: &str = "3D display could not start with the available graphics configuration. PartProbe is using text-only model and proposed-stock review; analysis and estimate evidence remain usable.";
#[cfg(feature = "viewer-spike")]
const GRAPHICS_FAILURE_VIEWER_NOTICE: &str = "3D display stopped after a graphics failure. PartProbe switched to text-only model and proposed-stock review; accepted analysis and estimate evidence were preserved. Reopen the app to retry graphics.";
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
    #[cfg(feature = "viewer-spike")]
    view: StandardView,
}

#[cfg(feature = "viewer-spike")]
#[derive(Clone, Copy)]
enum ViewerLimitedModeCause {
    StartupUnavailable,
    GraphicsFailure,
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
        proposed_stock_visible: bool,
    },
    Unavailable {
        selection_id: String,
    },
    Limited {
        selection_id: Option<String>,
        cause: ViewerLimitedModeCause,
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
            Self::Limited {
                selection_id: Some(current),
                ..
            } => current == selection_id,
            Self::Limited {
                selection_id: None, ..
            } => false,
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
            Self::SourceBound {
                analysis_id,
                proposed_stock_visible,
                ..
            } => {
                let _ = analysis_id;
                (
                    Some(GEOMETRY_DISPLAY_SCENE_REFERENCE.to_owned()),
                    if *proposed_stock_visible {
                        SOURCE_WITH_STOCK_VIEWER_NOTICE
                    } else {
                        SOURCE_VIEWER_NOTICE
                    },
                )
            }
            Self::Unavailable { .. } => (None, UNAVAILABLE_VIEWER_NOTICE),
            Self::Limited { cause, .. } => (
                None,
                match cause {
                    ViewerLimitedModeCause::StartupUnavailable => STARTUP_LIMITED_VIEWER_NOTICE,
                    ViewerLimitedModeCause::GraphicsFailure => GRAPHICS_FAILURE_VIEWER_NOTICE,
                },
            ),
        }
    }
}

impl DesktopModelViewerState {
    #[cfg(any(not(feature = "viewer-spike"), test))]
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
                #[cfg(feature = "viewer-spike")]
                view: StandardView::Isometric,
            })),
        }
    }

    pub(super) fn availability(&self) -> ModelViewerAvailability {
        #[cfg(feature = "viewer-spike")]
        if self
            .inner
            .lock()
            .is_ok_and(|runtime| runtime.window.is_some() && runtime.workspace.is_some())
        {
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
            if runtime.window.is_none() || runtime.workspace.is_none() {
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
            if runtime.window.is_none() || runtime.workspace.is_none() {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-VIEWER-NOT-CONFIGURED",
                ));
            }
            if !runtime.active {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-VIEWER-NOT-VISIBLE",
                ));
            }
            let requested_view = renderer_view(request.view);
            runtime.view = requested_view;
            let render_result = runtime
                .renderer
                .as_mut()
                .map(|renderer| renderer.set_view(requested_view));
            if let Some(Err(error)) = render_result {
                if renderer_failure_requires_limited_mode(&error) {
                    enter_limited_mode(&mut runtime, ViewerLimitedModeCause::GraphicsFailure);
                    apply_layout(&mut runtime)?;
                } else {
                    return Err(HostCommandError::host_state_unavailable(
                        "VIS1-SURFACE-RENDER",
                    ));
                }
            }
            Ok(workspace_result(
                &runtime,
                ModelViewerWorkspaceMode::Visible,
            ))
        }
    }

    pub(super) fn begin_selection(&self, selection_id: &str) {
        #[cfg(feature = "viewer-spike")]
        if let Ok(mut runtime) = self.inner.lock() {
            let clear_result = runtime
                .renderer
                .as_mut()
                .map(NativeSurfaceRenderer::clear_source_scene);
            if matches!(clear_result, Some(Err(ref error)) if renderer_failure_requires_limited_mode(error))
            {
                enter_limited_mode(&mut runtime, ViewerLimitedModeCause::GraphicsFailure);
            }
            set_selection_transition_status(&mut runtime, selection_id, true);
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
            let mut graphics_failure = false;
            let accepted = match (runtime.renderer.as_mut(), scene) {
                (Some(renderer), Some(scene)) => {
                    match renderer.set_source_scene(scene) {
                        Ok(_) => true,
                        Err(error) => {
                            graphics_failure = renderer_failure_requires_limited_mode(&error);
                            // A rejected or unrenderable replacement must not leave either the
                            // preceding selection or a partially installed source scene visible.
                            if !graphics_failure {
                                let _ = renderer.clear_source_scene();
                            }
                            false
                        }
                    }
                }
                (Some(renderer), None) => {
                    let _ = renderer.clear_source_scene();
                    false
                }
                (None, _) => false,
            };
            if graphics_failure {
                enter_limited_mode(&mut runtime, ViewerLimitedModeCause::GraphicsFailure);
                set_selection_transition_status(&mut runtime, selection_id, false);
            } else if accepted {
                runtime.scene_status = ViewerSceneStatus::SourceBound {
                    selection_id: selection_id.to_owned(),
                    analysis_id: analysis_id.to_owned(),
                    proposed_stock_visible: false,
                };
            } else if runtime.renderer.is_some() {
                runtime.scene_status = ViewerSceneStatus::Unavailable {
                    selection_id: selection_id.to_owned(),
                };
            } else {
                set_selection_transition_status(&mut runtime, selection_id, false);
            }
        }
        #[cfg(not(feature = "viewer-spike"))]
        let _ = (selection_id, analysis_id, scene);
    }

    pub(super) fn accept_proposed_stock(
        &self,
        selection_id: &str,
        analysis_id: &str,
        dimensions_mm: [f32; 3],
    ) {
        #[cfg(feature = "viewer-spike")]
        if let Ok(mut runtime) = self.inner.lock() {
            let matches_current = matches!(
                &runtime.scene_status,
                ViewerSceneStatus::SourceBound {
                    selection_id: current_selection,
                    analysis_id: current_analysis,
                    ..
                } if current_selection == selection_id && current_analysis == analysis_id
            );
            if !matches_current {
                return;
            }
            let render_result =
                ProposedStockDimensions::new(dimensions_mm)
                    .ok()
                    .and_then(|stock| {
                        runtime
                            .renderer
                            .as_mut()
                            .map(|renderer| renderer.set_proposed_stock(stock))
                    });
            if matches!(render_result, Some(Err(ref error)) if renderer_failure_requires_limited_mode(error))
            {
                enter_limited_mode(&mut runtime, ViewerLimitedModeCause::GraphicsFailure);
                return;
            }
            let accepted = matches!(render_result, Some(Ok(_)));
            if accepted
                && let ViewerSceneStatus::SourceBound {
                    proposed_stock_visible,
                    ..
                } = &mut runtime.scene_status
            {
                *proposed_stock_visible = true;
            }
        }
        #[cfg(not(feature = "viewer-spike"))]
        let _ = (selection_id, analysis_id, dimensions_mm);
    }

    pub(super) fn clear_proposed_stock(&self) {
        #[cfg(feature = "viewer-spike")]
        if let Ok(mut runtime) = self.inner.lock() {
            let clear_result = runtime
                .renderer
                .as_mut()
                .map(NativeSurfaceRenderer::clear_proposed_stock);
            if matches!(clear_result, Some(Err(ref error)) if renderer_failure_requires_limited_mode(error))
            {
                enter_limited_mode(&mut runtime, ViewerLimitedModeCause::GraphicsFailure);
                return;
            }
            if let ViewerSceneStatus::SourceBound {
                proposed_stock_visible,
                ..
            } = &mut runtime.scene_status
            {
                *proposed_stock_visible = false;
            }
        }
    }
}

#[cfg(feature = "viewer-spike")]
const fn renderer_failure_requires_limited_mode(error: &ViewerError) -> bool {
    error.requires_renderer_recreation()
}

#[cfg(feature = "viewer-spike")]
fn enter_limited_mode(runtime: &mut ModelViewerRuntime, cause: ViewerLimitedModeCause) {
    let selection_id = match &runtime.scene_status {
        ViewerSceneStatus::Pending { selection_id }
        | ViewerSceneStatus::SourceBound { selection_id, .. }
        | ViewerSceneStatus::Unavailable { selection_id } => Some(selection_id.clone()),
        ViewerSceneStatus::Limited { selection_id, .. } => selection_id.clone(),
        ViewerSceneStatus::Synthetic => None,
    };
    runtime.renderer.take();
    runtime.scene_status = ViewerSceneStatus::Limited {
        selection_id,
        cause,
    };
}

#[cfg(feature = "viewer-spike")]
fn set_selection_transition_status(
    runtime: &mut ModelViewerRuntime,
    selection_id: &str,
    pending: bool,
) {
    let limited_cause = match &runtime.scene_status {
        ViewerSceneStatus::Limited { cause, .. } => Some(*cause),
        _ => None,
    };
    if let Some(cause) = limited_cause {
        runtime.scene_status = ViewerSceneStatus::Limited {
            selection_id: Some(selection_id.to_owned()),
            cause,
        };
    } else if runtime.renderer.is_none() {
        runtime.scene_status = ViewerSceneStatus::Limited {
            selection_id: Some(selection_id.to_owned()),
            cause: ViewerLimitedModeCause::GraphicsFailure,
        };
    } else if pending {
        runtime.scene_status = ViewerSceneStatus::Pending {
            selection_id: selection_id.to_owned(),
        };
    } else {
        runtime.scene_status = ViewerSceneStatus::Unavailable {
            selection_id: selection_id.to_owned(),
        };
    }
}

#[cfg(feature = "viewer-spike")]
fn workspace_result(
    runtime: &ModelViewerRuntime,
    mode: ModelViewerWorkspaceMode,
) -> ModelViewerWorkspaceResult {
    let (scene_reference, notice) = runtime.scene_status.visible_summary();
    let view = contract_view(runtime.view);
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
    cfg!(feature = "viewer-spike")
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
        let original_webview = app
            .get_webview_window("main")
            .ok_or("configured main window is missing")?;
        let window = app
            .get_window("main")
            .ok_or("configured main native window is missing")?;
        let size = window.inner_size()?;
        let (renderer, scene_status) = match NativeSurfaceRenderer::attach(
            window.clone(),
            size.width,
            size.height,
            StandardView::Isometric,
        ) {
            Ok(renderer) => (Some(renderer), ViewerSceneStatus::Synthetic),
            Err(_) => (
                None,
                ViewerSceneStatus::Limited {
                    selection_id: None,
                    cause: ViewerLimitedModeCause::StartupUnavailable,
                },
            ),
        };
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
                renderer,
                scene_status,
                view: StandardView::Isometric,
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
                if let Ok(mut runtime) = state_for_events.inner.lock() && runtime.active {
                    let render_result = runtime
                        .renderer
                        .as_mut()
                        .map(NativeSurfaceRenderer::render);
                    if matches!(render_result, Some(Err(ref error)) if renderer_failure_requires_limited_mode(error)) {
                        enter_limited_mode(
                            &mut runtime,
                            ViewerLimitedModeCause::GraphicsFailure,
                        );
                        let _ = apply_layout(&mut runtime);
                    }
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
    if runtime.active && runtime.renderer.is_some() {
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
        let render_result = runtime
            .renderer
            .as_mut()
            .expect("renderer presence was checked")
            .resize_with_viewport(
                surface_size.width,
                surface_size.height,
                SurfaceViewport {
                    x: sidebar_width,
                    y: 0,
                    width: viewport_width,
                    height: surface_size.height,
                },
            );
        if let Err(error) = render_result {
            if renderer_failure_requires_limited_mode(&error) {
                enter_limited_mode(runtime, ViewerLimitedModeCause::GraphicsFailure);
                set_full_workspace_bounds(&workspace, surface_size)?;
            } else {
                return Err(HostCommandError::host_state_unavailable(
                    "VIS1-SURFACE-RENDER",
                ));
            }
        }
    } else {
        set_full_workspace_bounds(&workspace, surface_size)?;
        if let Some(renderer) = runtime.renderer.as_mut() {
            match renderer.resize(surface_size.width, surface_size.height) {
                Ok(
                    SurfaceFrameStatus::Presented
                    | SurfaceFrameStatus::RecoveredAndPresented
                    | SurfaceFrameStatus::Skipped,
                ) => {}
                Err(error) if renderer_failure_requires_limited_mode(&error) => {
                    enter_limited_mode(runtime, ViewerLimitedModeCause::GraphicsFailure);
                }
                Err(_) => {
                    return Err(HostCommandError::host_state_unavailable(
                        "VIS1-SURFACE-RENDER",
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "viewer-spike")]
fn set_full_workspace_bounds(
    workspace: &tauri::Webview,
    surface_size: tauri::PhysicalSize<u32>,
) -> Result<(), HostCommandError> {
    workspace
        .set_bounds(tauri::Rect {
            position: tauri::PhysicalPosition::new(0, 0).into(),
            size: surface_size.into(),
        })
        .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))?;
    workspace
        .set_auto_resize(true)
        .map_err(|_| HostCommandError::host_state_unavailable("VIS1-WEBVIEW-BOUNDS"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_activation_matches_the_build_profile() {
        assert_eq!(source_scene_requested(), cfg!(feature = "viewer-spike"));
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
    fn source_bound_workspace_summary_is_path_free_and_awaits_stock_proposal() {
        let status = ViewerSceneStatus::SourceBound {
            selection_id: "selection-1".to_owned(),
            analysis_id: "analysis-1".to_owned(),
            proposed_stock_visible: false,
        };

        let (reference, notice) = status.visible_summary();

        assert_eq!(reference.as_deref(), Some(GEOMETRY_DISPLAY_SCENE_REFERENCE));
        assert!(notice.contains("Selected exact B-rep model"));
        assert!(notice.contains("automatic estimate-input preparation"));
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
            proposed_stock_visible: false,
        };

        assert!(pending.selection_matches("selection-2"));
        assert!(!pending.selection_matches("selection-1"));
        assert!(source.selection_matches("selection-2"));
        assert!(!ViewerSceneStatus::Synthetic.selection_matches("selection-2"));
    }

    #[cfg(feature = "viewer-spike")]
    #[test]
    fn proposed_stock_summary_keeps_review_and_availability_limits_visible() {
        let status = ViewerSceneStatus::SourceBound {
            selection_id: "selection-1".to_owned(),
            analysis_id: "analysis-1".to_owned(),
            proposed_stock_visible: true,
        };

        let (_, notice) = status.visible_summary();

        assert!(notice.contains("review-only proposed rectangular stock"));
        assert!(notice.contains("centered"));
        assert!(notice.contains("availability remain unresolved"));
        assert!(!notice.contains('/'));
        assert!(!notice.contains('\\'));
    }

    #[cfg(feature = "viewer-spike")]
    #[test]
    fn limited_mode_summary_is_path_free_and_preserves_text_review() {
        for (cause, expected) in [
            (
                ViewerLimitedModeCause::StartupUnavailable,
                "could not start with the available graphics configuration",
            ),
            (
                ViewerLimitedModeCause::GraphicsFailure,
                "switched to text-only",
            ),
        ] {
            let status = ViewerSceneStatus::Limited {
                selection_id: Some("selection-1".to_owned()),
                cause,
            };

            let (reference, notice) = status.visible_summary();

            assert_eq!(reference, None);
            assert!(notice.contains(expected));
            assert!(notice.contains("analysis and estimate evidence"));
            assert!(!notice.contains('/'));
            assert!(!notice.contains('\\'));
            assert!(status.selection_matches("selection-1"));
            assert!(!status.selection_matches("selection-2"));
        }
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
