use std::{borrow::Cow, error::Error, sync::Arc};

use wgpu::CurrentSurfaceTexture;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

struct GpuState {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl GpuState {
    async fn new(
        window: Arc<Window>,
        display_handle: winit::event_loop::OwnedDisplayHandle,
    ) -> Result<Self, String> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
            Box::new(display_handle),
        ));
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
                label: Some("shader-lab device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| error.to_string())?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/triangle.wgsl"
            ))),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .copied()
            .ok_or_else(|| "surface reported no supported formats".to_owned())?;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
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
        })
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
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
                label: Some("triangle encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("triangle render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.03,
                            g: 0.04,
                            b: 0.06,
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
            pass.draw(0..3, 0..1);
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

        let attributes = Window::default_attributes().with_title("shader-lab — triangle");
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
