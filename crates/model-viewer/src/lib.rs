//! Native-only rendering boundary for PartProbe model review.
//!
//! VIS-1 deliberately renders a bounded synthetic scene. It proves the GPU path and visual
//! language without accepting CAD bytes, source paths, estimate authority, or WebView geometry.

use std::{fmt, path::Path, sync::mpsc};

use wgpu::util::DeviceExt;

/// Stable identifier for the non-authoritative VIS-1 scene.
pub const SYNTHETIC_SCENE_REFERENCE: &str = "synthetic-viewer-spike-v1";

const MIN_FRAME_EDGE: u32 = 64;
const MAX_FRAME_EDGE: u32 = 2_048;
const MAX_SURFACE_EDGE: u32 = 4_096;
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const FLOATS_PER_VERTEX: usize = 7;
const VIEWPORT_BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.025,
    g: 0.038,
    b: 0.055,
    a: 1.0,
};
const MODEL_FACE_COLORS: [[f32; 4]; 6] = [
    [0.18, 0.78, 0.78, 1.0],
    [0.12, 0.60, 0.67, 1.0],
    [0.23, 0.86, 0.76, 1.0],
    [0.10, 0.52, 0.62, 1.0],
    [0.31, 0.91, 0.82, 1.0],
    [0.14, 0.65, 0.72, 1.0],
];
const STOCK_FACE_COLOR: [f32; 4] = [0.98, 0.59, 0.16, 0.13];
const STOCK_EDGE_COLOR: [f32; 4] = [1.0, 0.69, 0.22, 0.95];

/// Standard camera orientations required by the first model-and-stock workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardView {
    Isometric,
    Front,
    Top,
    Right,
}

/// Content-minimized proof about a rendered frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderReport {
    pub scene_reference: &'static str,
    pub view: StandardView,
    pub width: u32,
    pub height: u32,
    pub backend: String,
    pub model_visible: bool,
    pub stock_visible: bool,
    pub stock_is_translucent: bool,
}

/// RGBA pixels returned by the native renderer for local validation and future surface adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedFrame {
    pub report: RenderReport,
    pub rgba8: Vec<u8>,
}

impl RenderedFrame {
    /// Writes a simple binary PPM without adding an image-codec dependency.
    pub fn write_ppm(&self, path: impl AsRef<Path>) -> Result<(), ViewerError> {
        let mut bytes =
            format!("P6\n{} {}\n255\n", self.report.width, self.report.height).into_bytes();
        bytes.reserve(self.rgba8.len() / 4 * 3);
        for pixel in self.rgba8.chunks_exact(4) {
            bytes.extend_from_slice(&pixel[..3]);
        }
        std::fs::write(path, bytes).map_err(ViewerError::Output)
    }
}

#[derive(Debug)]
pub enum ViewerError {
    InvalidFrameSize { width: u32, height: u32 },
    InvalidSurfaceSize { width: u32, height: u32 },
    InvalidSurfaceViewport,
    SurfaceCreation(String),
    SurfaceUnsupported,
    SurfaceUnavailable(String),
    AdapterUnavailable,
    DeviceUnavailable(String),
    GpuPoll(String),
    ReadbackUnavailable,
    ReadbackFailed,
    Output(std::io::Error),
}

impl fmt::Display for ViewerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameSize { width, height } => write!(
                formatter,
                "viewer frame must be {MIN_FRAME_EDGE}..={MAX_FRAME_EDGE} pixels per edge; received {width}x{height}"
            ),
            Self::InvalidSurfaceSize { width, height } => write!(
                formatter,
                "viewer surface must be {MIN_FRAME_EDGE}..={MAX_SURFACE_EDGE} pixels per edge; received {width}x{height}"
            ),
            Self::InvalidSurfaceViewport => {
                formatter.write_str("viewer viewport is outside the bounded native surface")
            }
            Self::AdapterUnavailable => {
                formatter.write_str("no compatible native graphics adapter is available")
            }
            Self::SurfaceCreation(reason) => {
                write!(formatter, "native viewer surface creation failed: {reason}")
            }
            Self::SurfaceUnsupported => {
                formatter.write_str("native graphics adapter cannot present to the viewer surface")
            }
            Self::SurfaceUnavailable(reason) => {
                write!(formatter, "native viewer surface is unavailable: {reason}")
            }
            Self::DeviceUnavailable(reason) => {
                write!(
                    formatter,
                    "native graphics device creation failed: {reason}"
                )
            }
            Self::GpuPoll(reason) => {
                write!(formatter, "native graphics completion failed: {reason}")
            }
            Self::ReadbackUnavailable => {
                formatter.write_str("native graphics readback did not complete")
            }
            Self::ReadbackFailed => formatter.write_str("native graphics readback failed"),
            Self::Output(error) => write!(formatter, "viewer image output failed: {error}"),
        }
    }
}

impl std::error::Error for ViewerError {}

/// Render the bounded VIS-1 model-and-stock scene through the platform-native wgpu backend.
pub fn render_synthetic_model_and_stock(
    width: u32,
    height: u32,
    view: StandardView,
) -> Result<RenderedFrame, ViewerError> {
    validate_frame_size(width, height)?;
    pollster::block_on(render(width, height, view))
}

/// Outcome of attempting to draw one native window frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceFrameStatus {
    Presented,
    Skipped,
}

/// Physical-pixel region of the native surface reserved for the 3D scene.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceViewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SurfaceViewport {
    #[must_use]
    pub const fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

/// A bounded native-window renderer for the synthetic VIS-1 review scene.
///
/// The owned surface retains the native window handle supplied to [`Self::attach`]. This type
/// accepts no model bytes, source paths, estimate data, or WebView-owned geometry.
pub struct NativeSurfaceRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    view: StandardView,
    viewport: SurfaceViewport,
    _depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    opaque_pipeline: wgpu::RenderPipeline,
    translucent_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    model_buffer: wgpu::Buffer,
    stock_buffer: wgpu::Buffer,
    edge_buffer: wgpu::Buffer,
    model_vertex_count: u32,
    stock_vertex_count: u32,
    edge_vertex_count: u32,
    backend: String,
}

impl NativeSurfaceRenderer {
    /// Attach the bounded synthetic renderer to an owned native window handle.
    ///
    /// On macOS this must be called from the main thread because Metal surface creation has that
    /// platform requirement.
    pub fn attach<W>(
        window: W,
        width: u32,
        height: u32,
        view: StandardView,
    ) -> Result<Self, ViewerError>
    where
        W: wgpu::WindowHandle + wgpu::rwh::HasDisplayHandle + 'static,
    {
        validate_surface_size(width, height)?;
        pollster::block_on(Self::attach_async(window, width, height, view))
    }

    async fn attach_async<W>(
        window: W,
        width: u32,
        height: u32,
        view: StandardView,
    ) -> Result<Self, ViewerError>
    where
        W: wgpu::WindowHandle + wgpu::rwh::HasDisplayHandle + 'static,
    {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|error| ViewerError::SurfaceCreation(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|_| ViewerError::AdapterUnavailable)?;
        let backend = format!("{:?}", adapter.get_info().backend);
        let supported_surface_edge = adapter.limits().max_texture_dimension_2d;
        if width > supported_surface_edge || height > supported_surface_edge {
            return Err(ViewerError::InvalidSurfaceSize { width, height });
        }
        let mut required_limits = wgpu::Limits::downlevel_defaults();
        required_limits.max_texture_dimension_2d = MAX_SURFACE_EDGE.min(supported_surface_edge);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("PartProbe VIS-1 native surface device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                ..Default::default()
            })
            .await
            .map_err(|error| ViewerError::DeviceUnavailable(error.to_string()))?;
        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or(ViewerError::SurfaceUnsupported)?;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PartProbe VIS-1 native surface shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PartProbe VIS-1 native surface pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let opaque_pipeline = create_pipeline(
            &device,
            &shader,
            &pipeline_layout,
            PipelineSpec {
                label: "PartProbe VIS-1 native opaque model pipeline",
                color_format: config.format,
                topology: wgpu::PrimitiveTopology::TriangleList,
                blend: None,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
            },
        );
        let translucent_pipeline = create_pipeline(
            &device,
            &shader,
            &pipeline_layout,
            PipelineSpec {
                label: "PartProbe VIS-1 native translucent stock pipeline",
                color_format: config.format,
                topology: wgpu::PrimitiveTopology::TriangleList,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
            },
        );
        let line_pipeline = create_pipeline(
            &device,
            &shader,
            &pipeline_layout,
            PipelineSpec {
                label: "PartProbe VIS-1 native stock edge pipeline",
                color_format: config.format,
                topology: wgpu::PrimitiveTopology::LineList,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
            },
        );
        let (depth_texture, depth_view) = create_depth_target(&device, width, height);
        let viewport = SurfaceViewport::full(width, height);
        let scene = create_scene_vertex_buffers(&device, view, viewport.width, viewport.height);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            view,
            viewport,
            _depth_texture: depth_texture,
            depth_view,
            opaque_pipeline,
            translucent_pipeline,
            line_pipeline,
            model_buffer: scene.model_buffer,
            stock_buffer: scene.stock_buffer,
            edge_buffer: scene.edge_buffer,
            model_vertex_count: scene.model_vertex_count,
            stock_vertex_count: scene.stock_vertex_count,
            edge_vertex_count: scene.edge_vertex_count,
            backend,
        })
    }

    /// Graphics backend selected for the current native surface.
    #[must_use]
    pub fn backend(&self) -> &str {
        &self.backend
    }

    /// Reconfigure and redraw after a nonzero native window resize.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<SurfaceFrameStatus, ViewerError> {
        self.resize_with_viewport(width, height, SurfaceViewport::full(width, height))
    }

    /// Reconfigure the native surface and its in-window viewport in one redraw.
    pub fn resize_with_viewport(
        &mut self,
        width: u32,
        height: u32,
        viewport: SurfaceViewport,
    ) -> Result<SurfaceFrameStatus, ViewerError> {
        if width == 0 || height == 0 {
            return Ok(SurfaceFrameStatus::Skipped);
        }
        validate_surface_size(width, height)?;
        validate_surface_viewport(width, height, viewport)?;
        self.config.width = width;
        self.config.height = height;
        self.viewport = viewport;
        self.surface.configure(&self.device, &self.config);
        let (depth_texture, depth_view) = create_depth_target(&self.device, width, height);
        self._depth_texture = depth_texture;
        self.depth_view = depth_view;
        self.rebuild_scene();
        self.render()
    }

    /// Move or resize the drawing region without changing the native surface dimensions.
    pub fn set_viewport(
        &mut self,
        viewport: SurfaceViewport,
    ) -> Result<SurfaceFrameStatus, ViewerError> {
        validate_surface_viewport(self.config.width, self.config.height, viewport)?;
        self.viewport = viewport;
        self.rebuild_scene();
        self.render()
    }

    /// Change to one of the governed standard views and redraw.
    pub fn set_view(&mut self, view: StandardView) -> Result<SurfaceFrameStatus, ViewerError> {
        self.view = view;
        self.rebuild_scene();
        self.render()
    }

    fn rebuild_scene(&mut self) {
        let scene = create_scene_vertex_buffers(
            &self.device,
            self.view,
            self.viewport.width,
            self.viewport.height,
        );
        self.model_buffer = scene.model_buffer;
        self.stock_buffer = scene.stock_buffer;
        self.edge_buffer = scene.edge_buffer;
        self.model_vertex_count = scene.model_vertex_count;
        self.stock_vertex_count = scene.stock_vertex_count;
        self.edge_vertex_count = scene.edge_vertex_count;
    }

    /// Draw one frame. Occlusion and presentation timeouts are deliberate non-errors.
    pub fn render(&mut self) -> Result<SurfaceFrameStatus, ViewerError> {
        let (frame, reconfigure_after_present) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(SurfaceFrameStatus::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(SurfaceFrameStatus::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(ViewerError::SurfaceUnavailable("surface-lost".to_owned()));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ViewerError::SurfaceUnavailable(
                    "surface-validation".to_owned(),
                ));
            }
        };
        let color_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("PartProbe VIS-1 native surface encoder"),
            });
        {
            let color_attachment = Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(VIEWPORT_BACKGROUND),
                    store: wgpu::StoreOp::Store,
                },
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PartProbe VIS-1 native model and stock pass"),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_viewport(
                self.viewport.x as f32,
                self.viewport.y as f32,
                self.viewport.width as f32,
                self.viewport.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                self.viewport.x,
                self.viewport.y,
                self.viewport.width,
                self.viewport.height,
            );
            pass.set_pipeline(&self.translucent_pipeline);
            pass.set_vertex_buffer(0, self.stock_buffer.slice(..));
            pass.draw(0..self.stock_vertex_count, 0..1);
            pass.set_pipeline(&self.opaque_pipeline);
            pass.set_vertex_buffer(0, self.model_buffer.slice(..));
            pass.draw(0..self.model_vertex_count, 0..1);
            pass.set_pipeline(&self.line_pipeline);
            pass.set_vertex_buffer(0, self.edge_buffer.slice(..));
            pass.draw(0..self.edge_vertex_count, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
        }
        Ok(SurfaceFrameStatus::Presented)
    }
}

fn validate_frame_size(width: u32, height: u32) -> Result<(), ViewerError> {
    if !(MIN_FRAME_EDGE..=MAX_FRAME_EDGE).contains(&width)
        || !(MIN_FRAME_EDGE..=MAX_FRAME_EDGE).contains(&height)
    {
        return Err(ViewerError::InvalidFrameSize { width, height });
    }
    Ok(())
}

fn validate_surface_size(width: u32, height: u32) -> Result<(), ViewerError> {
    if width < MIN_FRAME_EDGE
        || height < MIN_FRAME_EDGE
        || width > MAX_SURFACE_EDGE
        || height > MAX_SURFACE_EDGE
    {
        return Err(ViewerError::InvalidSurfaceSize { width, height });
    }
    Ok(())
}

fn validate_surface_viewport(
    surface_width: u32,
    surface_height: u32,
    viewport: SurfaceViewport,
) -> Result<(), ViewerError> {
    let within_surface = viewport
        .x
        .checked_add(viewport.width)
        .is_some_and(|right| right <= surface_width)
        && viewport
            .y
            .checked_add(viewport.height)
            .is_some_and(|bottom| bottom <= surface_height);
    if viewport.width < MIN_FRAME_EDGE || viewport.height < MIN_FRAME_EDGE || !within_surface {
        return Err(ViewerError::InvalidSurfaceViewport);
    }
    Ok(())
}

async fn render(width: u32, height: u32, view: StandardView) -> Result<RenderedFrame, ViewerError> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        })
        .await
        .map_err(|_| ViewerError::AdapterUnavailable)?;
    let adapter_info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("PartProbe VIS-1 device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            ..Default::default()
        })
        .await
        .map_err(|error| ViewerError::DeviceUnavailable(error.to_string()))?;

    let color_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("PartProbe VIS-1 color target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("PartProbe VIS-1 depth target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("PartProbe VIS-1 shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("PartProbe VIS-1 pipeline layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let opaque_pipeline = create_pipeline(
        &device,
        &shader,
        &pipeline_layout,
        PipelineSpec {
            label: "PartProbe VIS-1 opaque model pipeline",
            color_format: COLOR_FORMAT,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: None,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
        },
    );
    let translucent_pipeline = create_pipeline(
        &device,
        &shader,
        &pipeline_layout,
        PipelineSpec {
            label: "PartProbe VIS-1 translucent stock pipeline",
            color_format: COLOR_FORMAT,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Less,
        },
    );
    let line_pipeline = create_pipeline(
        &device,
        &shader,
        &pipeline_layout,
        PipelineSpec {
            label: "PartProbe VIS-1 stock edge pipeline",
            color_format: COLOR_FORMAT,
            topology: wgpu::PrimitiveTopology::LineList,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
        },
    );

    let scene = build_scene(view, width as f32 / height as f32);
    let model_buffer =
        create_vertex_buffer(&device, "PartProbe VIS-1 model vertices", &scene.model);
    let stock_buffer = create_vertex_buffer(&device, "PartProbe VIS-1 stock faces", &scene.stock);
    let edge_buffer = create_vertex_buffer(&device, "PartProbe VIS-1 stock edges", &scene.edges);

    let padded_bytes_per_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("PartProbe VIS-1 readback"),
        size: u64::from(padded_bytes_per_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("PartProbe VIS-1 encoder"),
    });
    {
        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &color_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(VIEWPORT_BACKGROUND),
                store: wgpu::StoreOp::Store,
            },
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PartProbe VIS-1 model and stock pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&translucent_pipeline);
        pass.set_vertex_buffer(0, stock_buffer.slice(..));
        pass.draw(0..vertex_count(&scene.stock), 0..1);
        pass.set_pipeline(&opaque_pipeline);
        pass.set_vertex_buffer(0, model_buffer.slice(..));
        pass.draw(0..vertex_count(&scene.model), 0..1);
        pass.set_pipeline(&line_pipeline);
        pass.set_vertex_buffer(0, edge_buffer.slice(..));
        pass.draw(0..vertex_count(&scene.edges), 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &color_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let (sender, receiver) = mpsc::channel();
    readback.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| ViewerError::GpuPoll(error.to_string()))?;
    receiver
        .recv()
        .map_err(|_| ViewerError::ReadbackUnavailable)?
        .map_err(|_| ViewerError::ReadbackFailed)?;

    let mapped = readback
        .get_mapped_range(..)
        .map_err(|_| ViewerError::ReadbackFailed)?;
    let mut rgba8 = Vec::with_capacity(width as usize * height as usize * 4);
    for row in mapped.chunks_exact(padded_bytes_per_row as usize) {
        rgba8.extend_from_slice(&row[..width as usize * 4]);
    }
    drop(mapped);
    readback.unmap();

    Ok(RenderedFrame {
        report: RenderReport {
            scene_reference: SYNTHETIC_SCENE_REFERENCE,
            view,
            width,
            height,
            backend: format!("{:?}", adapter_info.backend),
            model_visible: true,
            stock_visible: true,
            stock_is_translucent: true,
        },
        rgba8,
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    spec: PipelineSpec,
) -> wgpu::RenderPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(spec.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex_main"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: (FLOATS_PER_VERTEX * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            })],
        },
        primitive: wgpu::PrimitiveState {
            topology: spec.topology,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(spec.depth_write_enabled),
            depth_compare: Some(spec.depth_compare),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: spec.color_format,
                blend: spec.blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[derive(Clone, Copy)]
struct PipelineSpec {
    label: &'static str,
    color_format: wgpu::TextureFormat,
    topology: wgpu::PrimitiveTopology,
    blend: Option<wgpu::BlendState>,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
}

fn create_vertex_buffer(
    device: &wgpu::Device,
    label: &'static str,
    vertices: &[f32],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn vertex_count(vertices: &[f32]) -> u32 {
    (vertices.len() / FLOATS_PER_VERTEX) as u32
}

fn create_depth_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("PartProbe VIS-1 native depth target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

struct SceneVertexBuffers {
    model_buffer: wgpu::Buffer,
    stock_buffer: wgpu::Buffer,
    edge_buffer: wgpu::Buffer,
    model_vertex_count: u32,
    stock_vertex_count: u32,
    edge_vertex_count: u32,
}

fn create_scene_vertex_buffers(
    device: &wgpu::Device,
    view: StandardView,
    width: u32,
    height: u32,
) -> SceneVertexBuffers {
    let scene = build_scene(view, width as f32 / height as f32);
    SceneVertexBuffers {
        model_buffer: create_vertex_buffer(
            device,
            "PartProbe VIS-1 native model vertices",
            &scene.model,
        ),
        stock_buffer: create_vertex_buffer(
            device,
            "PartProbe VIS-1 native stock faces",
            &scene.stock,
        ),
        edge_buffer: create_vertex_buffer(
            device,
            "PartProbe VIS-1 native stock edges",
            &scene.edges,
        ),
        model_vertex_count: vertex_count(&scene.model),
        stock_vertex_count: vertex_count(&scene.stock),
        edge_vertex_count: vertex_count(&scene.edges),
    }
}

struct SceneBuffers {
    model: Vec<f32>,
    stock: Vec<f32>,
    edges: Vec<f32>,
}

fn build_scene(view: StandardView, aspect: f32) -> SceneBuffers {
    let model_bounds = Box3::centered([0.0, 0.0, 0.0], [1.0, 0.64, 0.48]);
    let stock_bounds = Box3::centered([0.06, -0.03, 0.02], [1.34, 0.90, 0.72]);
    SceneBuffers {
        model: box_faces(model_bounds, view, aspect, MODEL_FACE_COLORS),
        stock: box_faces(stock_bounds, view, aspect, [STOCK_FACE_COLOR; 6]),
        edges: box_edges(stock_bounds, view, aspect, STOCK_EDGE_COLOR),
    }
}

#[derive(Clone, Copy)]
struct Box3 {
    corners: [[f32; 3]; 8],
}

impl Box3 {
    fn centered(center: [f32; 3], size: [f32; 3]) -> Self {
        let half = [size[0] / 2.0, size[1] / 2.0, size[2] / 2.0];
        Self {
            corners: [
                [
                    center[0] - half[0],
                    center[1] - half[1],
                    center[2] - half[2],
                ],
                [
                    center[0] + half[0],
                    center[1] - half[1],
                    center[2] - half[2],
                ],
                [
                    center[0] + half[0],
                    center[1] + half[1],
                    center[2] - half[2],
                ],
                [
                    center[0] - half[0],
                    center[1] + half[1],
                    center[2] - half[2],
                ],
                [
                    center[0] - half[0],
                    center[1] - half[1],
                    center[2] + half[2],
                ],
                [
                    center[0] + half[0],
                    center[1] - half[1],
                    center[2] + half[2],
                ],
                [
                    center[0] + half[0],
                    center[1] + half[1],
                    center[2] + half[2],
                ],
                [
                    center[0] - half[0],
                    center[1] + half[1],
                    center[2] + half[2],
                ],
            ],
        }
    }
}

fn box_faces(bounds: Box3, view: StandardView, aspect: f32, colors: [[f32; 4]; 6]) -> Vec<f32> {
    const FACES: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [4, 7, 6, 5],
        [0, 4, 5, 1],
        [1, 5, 6, 2],
        [2, 6, 7, 3],
        [3, 7, 4, 0],
    ];
    let mut output = Vec::with_capacity(6 * 6 * FLOATS_PER_VERTEX);
    for (face, color) in FACES.into_iter().zip(colors) {
        for corner in [face[0], face[1], face[2], face[0], face[2], face[3]] {
            push_vertex(
                &mut output,
                project(bounds.corners[corner], view, aspect),
                color,
            );
        }
    }
    output
}

fn box_edges(bounds: Box3, view: StandardView, aspect: f32, color: [f32; 4]) -> Vec<f32> {
    const EDGES: [[usize; 2]; 12] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    let mut output = Vec::with_capacity(12 * 2 * FLOATS_PER_VERTEX);
    for edge in EDGES {
        for corner in edge {
            push_vertex(
                &mut output,
                project(bounds.corners[corner], view, aspect),
                color,
            );
        }
    }
    output
}

fn push_vertex(output: &mut Vec<f32>, position: [f32; 3], color: [f32; 4]) {
    output.extend_from_slice(&position);
    output.extend_from_slice(&color);
}

fn project(point: [f32; 3], view: StandardView, aspect: f32) -> [f32; 3] {
    let (right, up, forward) = match view {
        StandardView::Isometric => {
            let forward = normalize([1.0, -1.0, 0.78]);
            let right = normalize(cross([0.0, 0.0, 1.0], forward));
            (right, cross(forward, right), forward)
        }
        StandardView::Front => ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]),
        StandardView::Top => ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        StandardView::Right => ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
    };
    let scale = 0.78;
    [
        dot(point, right) * scale / aspect.max(1.0),
        dot(point, up) * scale * aspect.min(1.0),
        0.5 + dot(point, forward) * 0.22,
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = dot(value, value).sqrt();
    [value[0] / length, value[1] / length, value[2] / length]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_scene_has_expected_layers_and_primitive_counts() {
        let scene = build_scene(StandardView::Isometric, 1.5);
        assert_eq!(vertex_count(&scene.model), 36);
        assert_eq!(vertex_count(&scene.stock), 36);
        assert_eq!(vertex_count(&scene.edges), 24);
    }

    #[test]
    fn every_standard_view_produces_finite_clip_coordinates() {
        for view in [
            StandardView::Isometric,
            StandardView::Front,
            StandardView::Top,
            StandardView::Right,
        ] {
            for aspect in [0.5, 1.0, 1.5] {
                let scene = build_scene(view, aspect);
                for vertex in scene
                    .model
                    .chunks_exact(FLOATS_PER_VERTEX)
                    .chain(scene.stock.chunks_exact(FLOATS_PER_VERTEX))
                    .chain(scene.edges.chunks_exact(FLOATS_PER_VERTEX))
                {
                    assert!(vertex[..3].iter().all(|coordinate| coordinate.is_finite()));
                    assert!((-1.0..=1.0).contains(&vertex[0]));
                    assert!((-1.0..=1.0).contains(&vertex[1]));
                    assert!((0.0..=1.0).contains(&vertex[2]));
                }
            }
        }
    }

    #[test]
    fn frame_limits_fail_before_gpu_initialization() {
        let error = render_synthetic_model_and_stock(32, 600, StandardView::Isometric)
            .expect_err("undersized frame must fail");
        assert!(matches!(error, ViewerError::InvalidFrameSize { .. }));
    }

    #[test]
    fn in_window_surface_and_viewport_limits_fail_before_gpu_work() {
        assert!(validate_surface_size(4_096, 2_048).is_ok());
        assert!(matches!(
            validate_surface_size(4_097, 2_048),
            Err(ViewerError::InvalidSurfaceSize { .. })
        ));
        assert!(
            validate_surface_viewport(
                2_360,
                1_520,
                SurfaceViewport {
                    x: 720,
                    y: 0,
                    width: 1_640,
                    height: 1_520,
                },
            )
            .is_ok()
        );
        assert!(matches!(
            validate_surface_viewport(
                2_360,
                1_520,
                SurfaceViewport {
                    x: 720,
                    y: 0,
                    width: 1_641,
                    height: 1_520,
                },
            ),
            Err(ViewerError::InvalidSurfaceViewport)
        ));
    }

    #[test]
    #[ignore = "requires a native GPU adapter; run explicitly for VIS-1 evidence"]
    fn native_renderer_covers_standard_views_and_resizing() {
        let mut prior_pixels: Option<Vec<u8>> = None;
        for view in [
            StandardView::Isometric,
            StandardView::Front,
            StandardView::Top,
            StandardView::Right,
        ] {
            let frame = render_synthetic_model_and_stock(256, 256, view)
                .expect("native render should succeed");
            assert_eq!(frame.rgba8.len(), 256 * 256 * 4);
            let first = &frame.rgba8[..4];
            assert!(frame.rgba8.chunks_exact(4).any(|pixel| pixel != first));
            assert!(frame.report.model_visible);
            assert!(frame.report.stock_is_translucent);
            if let Some(prior) = &prior_pixels {
                assert_ne!(prior, &frame.rgba8);
            }
            prior_pixels = Some(frame.rgba8);
        }

        let resized = render_synthetic_model_and_stock(640, 360, StandardView::Isometric)
            .expect("resized native render should succeed");
        assert_eq!(resized.rgba8.len(), 640 * 360 * 4);
        assert_eq!((resized.report.width, resized.report.height), (640, 360));
    }
}
