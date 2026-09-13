use std::{
    ffi::OsString,
    sync::{Arc, Mutex},
};

use partprobe_desktop_contract::{
    HostCommandError, ModelViewerAvailability, ModelViewerWorkspaceMode,
    ModelViewerWorkspaceResult, SetModelViewerWorkspaceRequest,
};
#[cfg(feature = "viewer-spike")]
use partprobe_model_viewer::{
    NativeSurfaceRenderer, StandardView, SurfaceFrameStatus, SurfaceViewport,
};
use tauri::Manager;
#[cfg(feature = "viewer-spike")]
use tauri::{WebviewUrl, webview::WebviewBuilder};

const SYNTHETIC_VIEWER_FLAG: &str = "--vis1-synthetic-viewer";
#[cfg(feature = "viewer-spike")]
const SYNTHETIC_SCENE_REFERENCE: &str = "synthetic-viewer-spike-v1";
#[cfg(feature = "viewer-spike")]
const VIEWER_NOTICE: &str = "Synthetic renderer preview; selected-model display and governed stock placement are not connected yet.";
#[cfg(feature = "viewer-spike")]
const VIEWER_SIDEBAR_LOGICAL_WIDTH: f64 = 400.0;
#[cfg(feature = "viewer-spike")]
const VIEWER_WORKSPACE_WEBVIEW_LABEL: &str = "model-viewer-workspace";

#[derive(Clone)]
pub(super) struct DesktopModelViewerState {
    inner: Arc<Mutex<ModelViewerRuntime>>,
}

struct ModelViewerRuntime {
    #[cfg(feature = "viewer-spike")]
    active: bool,
    #[cfg(feature = "viewer-spike")]
    window: Option<tauri::Window>,
    #[cfg(feature = "viewer-spike")]
    workspace: Option<tauri::Webview>,
    #[cfg(feature = "viewer-spike")]
    renderer: Option<NativeSurfaceRenderer>,
}

impl DesktopModelViewerState {
    pub(super) fn unavailable() -> Self {
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
            return Err(HostCommandError::host_state_unavailable(
                "VIS1-VIEWER-NOT-CONFIGURED",
            ));
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
            Ok(ModelViewerWorkspaceResult {
                mode: request.mode,
                scene_reference: (request.mode == ModelViewerWorkspaceMode::Visible)
                    .then(|| SYNTHETIC_SCENE_REFERENCE.to_owned()),
                notice: VIEWER_NOTICE.to_owned(),
            })
        }
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
        let arguments = std::env::args_os().collect::<Vec<_>>();
        if !arguments_request_synthetic_viewer(&arguments) {
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
}
