// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::Context as _;
use std::sync::Arc;
use std::time::Instant;
use topovium_diagnostics::{FrameLog, FrameSample};
use topovium_viewport::{Camera, FrameBudget};
use topovium_wgpu_common::describe_adapter;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

/// Everything that exists only once the window and GPU device are alive.
///
/// Held as an `Option` on [`Application`] because on Android and iPadOS the surface is
/// created and destroyed repeatedly across the application's life. Modelling that from
/// the start keeps the desktop path honest about a constraint the mobile shells have
/// to satisfy anyway.
struct Graphics {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

/// The desktop application.
pub struct Application {
    graphics: Option<Graphics>,
    frame_log: FrameLog,
    budget: FrameBudget,
    camera: Camera,
    last_frame_start: Option<Instant>,
    frames_rendered: u64,
}

impl Application {
    /// Creates the application. No window or device exists until `resumed`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graphics: None,
            frame_log: FrameLog::default(),
            budget: FrameBudget::for_refresh_rate(None),
            camera: Camera::looking_at_origin(1.0),
            last_frame_start: None,
            frames_rendered: 0,
        }
    }

    fn initialise(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let attributes = Window::default_attributes()
            .with_title("Topovium")
            .with_inner_size(PhysicalSize::new(1280, 720));
        let window = Arc::new(event_loop.create_window(attributes)?);

        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;

        let report = describe_adapter(
            &adapter.get_info(),
            adapter.features(),
            &adapter.limits(),
            env!("CARGO_PKG_VERSION"),
        );
        log::info!(
            "adapter: {} via {:?}, tier {:?}, unified memory: {}",
            report.adapter_name,
            report.backend,
            report.tier,
            report.unified_memory
        );

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("topovium-device"),
                required_features: wgpu::Features::empty(),
                // Request the downlevel baseline rather than the adapter's maximum, so a
                // desktop build cannot silently start depending on limits that no tablet
                // or browser provides.
                required_limits:
                    wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                ..Default::default()
            }))?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .context("the adapter cannot present to this surface")?;

        // Prefer an sRGB surface so shaders can write linear values and let the display
        // hardware encode, rather than encoding by hand in every shader.
        let capabilities = surface.get_capabilities(&adapter);
        if let Some(srgb) = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
        {
            config.format = srgb;
        }
        config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        // Two frames in flight. More would raise throughput and hurt input-to-pixel
        // latency, which is the metric that matters while the user is dragging.
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);

        let refresh_rate = window
            .current_monitor()
            .and_then(|monitor| monitor.refresh_rate_millihertz())
            .map(|millihertz| millihertz as f32 / 1000.0);
        self.budget = FrameBudget::for_refresh_rate(refresh_rate);
        log::info!(
            "frame budget: {:.2} ms ({:?} Hz)",
            self.budget.target().as_secs_f64() * 1000.0,
            refresh_rate
        );

        self.camera.aspect_ratio = config.width as f32 / config.height as f32;
        self.graphics = Some(Graphics {
            window,
            surface,
            device,
            queue,
            config,
        });
        Ok(())
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let Some(graphics) = self.graphics.as_mut() else {
            return;
        };
        if size.width == 0 || size.height == 0 {
            return;
        }
        graphics.config.width = size.width;
        graphics.config.height = size.height;
        graphics
            .surface
            .configure(&graphics.device, &graphics.config);
        self.camera.aspect_ratio = size.width as f32 / size.height as f32;
    }

    fn render(&mut self) {
        let Some(graphics) = self.graphics.as_ref() else {
            return;
        };
        let frame_start = Instant::now();

        let frame = match graphics.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            // Suboptimal still yields a usable texture. Present it so the window keeps
            // updating, and reconfigure so the next frame is correct.
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                graphics
                    .surface
                    .configure(&graphics.device, &graphics.config);
                frame
            }
            // Recoverable: the surface was resized or lost. Reconfigure and try again
            // next frame rather than treating it as fatal.
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                graphics
                    .surface
                    .configure(&graphics.device, &graphics.config);
                return;
            }
            // Nothing is visible, so there is nothing worth spending a frame on.
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            other => {
                log::error!("could not acquire a surface texture: {other:?}");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.043,
                            g: 0.047,
                            b: 0.055,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            });
        }

        graphics.queue.submit(Some(encoder.finish()));
        graphics.window.pre_present_notify();
        graphics.queue.present(frame);

        let elapsed = frame_start.elapsed();
        self.frame_log.record(FrameSample::cpu_only(elapsed));
        self.budget.record_frame(elapsed);
        self.frames_rendered += 1;
        self.last_frame_start = Some(frame_start);

        // Report every few seconds at 60 Hz, so a long session leaves a timing trail
        // in the log without flooding it.
        if self.frames_rendered.is_multiple_of(600)
            && let Some(percentiles) = self.frame_log.cpu_percentiles()
        {
            log::info!(
                "cpu frame p50 {:.2} ms, p95 {:.2} ms, p99 {:.2} ms, stalls {}, quality levers {}",
                percentiles.p50.as_secs_f64() * 1000.0,
                percentiles.p95.as_secs_f64() * 1000.0,
                percentiles.p99.as_secs_f64() * 1000.0,
                self.frame_log.stall_count(),
                self.budget.quality().engaged_count()
            );
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        if let Err(error) = self.initialise(event_loop) {
            log::error!("could not start the renderer: {error:#}");
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(graphics) = self.graphics.as_ref() {
                    graphics.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Mirrors the mobile contract: the surface can vanish at any moment, so
        // dropping it here keeps the desktop path exercising the same shape.
        self.graphics = None;
    }
}
