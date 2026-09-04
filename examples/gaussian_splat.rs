use std::{borrow::Cow, error::Error, sync::Arc};

use wgpu::{CurrentSurfaceTexture, util::DeviceExt};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

// Back-to-front order keeps the alpha compositing deterministic for this first baseline.
const SPLAT_DATA: [[f32; 8]; 7] = [
    [-0.55, 0.35, -4.20, 0.52, 0.95, 0.35, 0.22, 0.70],
    [0.55, 0.32, -3.90, 0.47, 0.25, 0.58, 1.00, 0.72],
    [0.00, -0.15, -3.65, 0.70, 0.95, 0.72, 0.24, 0.62],
    [-0.72, -0.45, -3.35, 0.45, 0.34, 0.94, 0.55, 0.78],
    [0.70, -0.48, -3.10, 0.42, 1.00, 0.42, 0.58, 0.76],
    [-0.22, 0.62, -2.75, 0.28, 0.78, 0.42, 0.98, 0.82],
    [0.30, -0.02, -2.45, 0.34, 0.34, 0.92, 0.95, 0.78],
];
const SPLAT_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32,
        2 => Float32x3,
        3 => Float32
    ],
};

fn encode_rows<const N: usize>(rows: &[[f32; N]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(rows));
    for row in rows {
        for value in row {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    bytes
}

fn encode_values(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn camera_params(width: u32, height: u32) -> [f32; 4] {
    let width = u16::try_from(width.max(1)).unwrap_or(u16::MAX);
    let height = u16::try_from(height.max(1)).unwrap_or(u16::MAX);
    let aspect = f32::from(width) / f32::from(height);
    let focal = 1.0 / (0.5 * 55.0_f32.to_radians()).tan();
    [aspect, focal, 0.1, 100.0]
}

fn create_pipeline(
    device: &wgpu::Device,
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::RenderPipeline, wgpu::BindGroupLayout), String> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gaussian-splat shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "../shaders/gaussian-splat.wgsl"
        ))),
    });
    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gaussian-splat camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gaussian-splat pipeline layout"),
        bind_group_layouts: &[Some(&camera_bind_group_layout)],
        immediate_size: 0,
    });
    let format = surface
        .get_capabilities(adapter)
        .formats
        .first()
        .copied()
        .ok_or_else(|| "surface reported no supported formats".to_owned())?;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("gaussian-splat pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(SPLAT_LAYOUT)],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    Ok((pipeline, camera_bind_group_layout))
}

struct GpuState {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    splat_buffer: wgpu::Buffer,
    splat_count: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl GpuState {
    async fn new(
        window: Arc<Window>,
        display_handle: winit::event_loop::OwnedDisplayHandle,
    ) -> Result<Self, String> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(display_handle)),
        );
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| error.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("shader-lab gaussian-splat device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| error.to_string())?;
        let (pipeline, camera_bind_group_layout) = create_pipeline(&device, &surface, &adapter)?;

        let splat_bytes = encode_rows(&SPLAT_DATA);
        let splat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gaussian-splat instance buffer"),
            contents: &splat_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let splat_count = u32::try_from(SPLAT_DATA.len())
            .map_err(|_| "splat count exceeds the GPU draw range".to_owned())?;
        let camera_bytes = encode_values(&camera_params(size.width, size.height));
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gaussian-splat camera buffer"),
            contents: &camera_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gaussian-splat camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or_else(|| "surface has no default configuration".to_owned())?;
        surface.configure(&device, &config);

        Ok(Self {
            instance,
            window,
            device,
            queue,
            surface,
            config,
            pipeline,
            splat_buffer,
            splat_count,
            camera_buffer,
            camera_bind_group,
        })
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        let camera_bytes = encode_values(&camera_params(self.config.width, self.config.height));
        self.queue
            .write_buffer(&self.camera_buffer, 0, &camera_bytes);
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Lost => {
                match self.instance.create_surface(self.window.clone()) {
                    Ok(surface) => {
                        self.surface = surface;
                        self.surface.configure(&self.device, &self.config);
                        self.window.request_redraw();
                    }
                    Err(error) => eprintln!("failed to recreate GPU surface: {error}"),
                }
                return;
            }
            CurrentSurfaceTexture::Validation => {
                eprintln!("wgpu reported an unexpected surface validation failure");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gaussian-splat encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gaussian-splat render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.025,
                            b: 0.04,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.splat_buffer.slice(..));
            pass.draw(0..6, 0..self.splat_count);
        }

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

enum AppState {
    Uninitialized,
    Running(Box<GpuState>),
    Failed,
}

struct App {
    state: AppState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Uninitialized,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Uninitialized) {
            return;
        }

        let attributes = Window::default_attributes().with_title("shader-lab — gaussian splat");
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("failed to create window: {error}");
                self.state = AppState::Failed;
                event_loop.exit();
                return;
            }
        };
        let display_handle = event_loop.owned_display_handle();
        match pollster::block_on(GpuState::new(window, display_handle)) {
            Ok(state) => {
                state.window.request_redraw();
                self.state = AppState::Running(Box::new(state));
            }
            Err(error) => {
                eprintln!("failed to initialize wgpu: {error}");
                self.state = AppState::Failed;
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let AppState::Running(state) = &mut self.state {
                    state.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let AppState::Running(state) = &mut self.state {
                    state.render();
                }
            }
            WindowEvent::Occluded(false) => {
                if let AppState::Running(state) = &self.state {
                    state.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
