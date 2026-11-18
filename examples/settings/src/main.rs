//! A deliberately single-window consumer with application-owned GPU resources.
use egui_prism::{
    compositing::{self, Group, Item},
    egui,
};
use prism_settings_example::SettingsApp;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum Notice {
    Repaint { generation: u64, at: Instant },
    Failed { generation: u64, message: String },
}

fn main() -> Result<()> {
    let events = EventLoop::<Notice>::with_user_event().build()?;
    let mut application = Application {
        settings: SettingsApp::default(),
        runtime: None,
        proxy: events.create_proxy(),
        generation: 0,
        repaint_at: None,
        failure: None,
    };
    events.run_app(&mut application)?;
    match application.failure {
        Some(message) => Err(message.into()),
        None => Ok(()),
    }
}

struct Application {
    // Settings survive window/GPU recreation; egui and GPU state do not.
    settings: SettingsApp,
    runtime: Option<Runtime>,
    proxy: EventLoopProxy<Notice>,
    generation: u64,
    repaint_at: Option<Instant>,
    failure: Option<String>,
}

impl Application {
    fn fail(&mut self, events: &ActiveEventLoop, error: impl ToString) {
        self.failure = Some(error.to_string());
        self.runtime.take();
        events.exit();
    }
}

impl ApplicationHandler<Notice> for Application {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        if self.runtime.is_none() {
            self.generation += 1;
            match Runtime::create(events, self.proxy.clone(), self.generation) {
                Ok(runtime) => {
                    runtime.window.request_redraw();
                    self.runtime = Some(runtime);
                }
                Err(error) => self.fail(events, error),
            }
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.runtime.take();
        self.repaint_at = None;
    }

    fn user_event(&mut self, events: &ActiveEventLoop, notice: Notice) {
        match notice {
            Notice::Repaint { generation, at }
                if generation == self.generation && self.runtime.is_some() =>
            {
                self.repaint_at = Some(self.repaint_at.map_or(at, |old| old.min(at)));
            }
            Notice::Failed {
                generation,
                message,
            } if generation == self.generation && self.runtime.is_some() => {
                self.fail(events, message);
            }
            // Notifications from a disposed window cannot affect its replacement.
            _ => {}
        }
    }

    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(runtime) = self
            .runtime
            .as_mut()
            .filter(|runtime| runtime.window.id() == id)
        else {
            return;
        };
        let response = runtime.input.on_window_event(&runtime.window, &event);
        match event {
            // This recipe's close policy is unconditional; it has no close-confirmation dialog.
            WindowEvent::CloseRequested => events.exit(),
            WindowEvent::RedrawRequested => {
                self.repaint_at = None;
                if let Err(error) = runtime.draw(events, &mut self.settings) {
                    self.fail(events, error);
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                runtime.configure = true;
                runtime.window.request_redraw();
            }
            _ if response.repaint => runtime.window.request_redraw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, events: &ActiveEventLoop) {
        match self.repaint_at {
            Some(at) if at <= Instant::now() => {
                self.repaint_at = None;
                if let Some(runtime) = &self.runtime {
                    runtime.window.request_redraw();
                }
                events.set_control_flow(ControlFlow::Wait);
            }
            Some(at) => events.set_control_flow(ControlFlow::WaitUntil(at)),
            None => events.set_control_flow(ControlFlow::Wait),
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.runtime.take();
    }
}

struct Runtime {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    configuration: wgpu::SurfaceConfiguration,
    configure: bool,
    context: egui::Context,
    input: egui_winit::State,
    drawing: Drawing,
    started: Instant,
}

impl Runtime {
    fn create(
        events: &ActiveEventLoop,
        proxy: EventLoopProxy<Notice>,
        generation: u64,
    ) -> Result<Self> {
        let window = Arc::new(
            events.create_window(
                Window::default_attributes()
                    .with_title("Prism external settings recipe")
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 620.0)),
            )?,
        );
        let mut descriptor = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
            events.owned_display_handle(),
        ));
        descriptor.backends = if cfg!(target_os = "macos") {
            wgpu::Backends::METAL
        } else if cfg!(target_os = "windows") {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::VULKAN
        };
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
        let errors = proxy.clone();
        device.on_uncaptured_error(Arc::new(move |error| {
            let _ = errors.send_event(Notice::Failed {
                generation,
                message: error.to_string(),
            });
        }));
        let lost = proxy.clone();
        device.set_device_lost_callback(move |reason, message| {
            if reason != wgpu::DeviceLostReason::Destroyed {
                let _ = lost.send_event(Notice::Failed {
                    generation,
                    message: format!("Device lost ({reason:?}): {message}. Restart the recipe."),
                });
            }
        });
        let size = window.inner_size();
        let mut configuration = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("No supported surface configuration")?;
        configuration.format = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .ok_or("The compositor requires a gamma Rgba8Unorm/Bgra8Unorm surface")?;
        surface.configure(&device, &configuration);
        let context = egui::Context::default();
        SettingsApp::configure_context(&context);
        context.set_request_repaint_callback(move |info| {
            let _ = proxy.send_event(Notice::Repaint {
                generation,
                at: Instant::now() + info.delay,
            });
        });
        let input = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            events,
            Some(window.scale_factor() as f32),
            events.system_theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let drawing = Drawing::new(&context, device, queue, configuration.format)?;
        Ok(Self {
            window,
            surface,
            configuration,
            configure: false,
            context,
            input,
            drawing,
            started: Instant::now(),
        })
    }

    fn draw(&mut self, events: &ActiveEventLoop, settings: &mut SettingsApp) -> Result<()> {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        if self.configure {
            self.configuration.width = size.width;
            self.configuration.height = size.height;
            self.surface
                .configure(&self.drawing.device, &self.configuration);
            self.configure = false;
        }
        let surface_image = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(image) => image,
            wgpu::CurrentSurfaceTexture::Suboptimal(image) => {
                self.configure = true;
                image
            }
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err("Surface lost; restart the recipe".into());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return Err("Surface outdated; restart the recipe".into());
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err("Surface acquisition timed out".into());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("Surface validation failed".into());
            }
        };
        let mut input = self.input.take_egui_input(&self.window);
        egui_winit::update_viewport_info(
            input.viewports.entry(egui::ViewportId::ROOT).or_default(),
            &self.context,
            &self.window,
            false,
        );
        let mut failure = None;
        let mut output = self.context.run_ui(input, |ui| {
            if let Err(error) = settings.ui(ui).and_then(|()| self.drawing.image.paint(ui)) {
                failure = Some(error);
            }
        });
        self.input.handle_platform_output_with_event_loop(
            &self.window,
            events,
            std::mem::take(&mut output.platform_output),
        );
        let mut actions = Vec::new();
        let mut platform_failure = None;
        let mut quit = false;
        for (id, viewport) in std::mem::take(&mut output.viewport_output) {
            if id != egui::ViewportId::ROOT {
                platform_failure = Some("This recipe supports only the root viewport");
                continue;
            }
            quit |= requests_quit(&viewport.commands);
            let mut info = self.context.input(|input| input.viewport().clone());
            egui_winit::process_viewport_commands(
                &self.context,
                &mut info,
                viewport.commands,
                &self.window,
                &mut actions,
            );
        }
        // Settings does not request screenshots or additional native actions.
        if !actions.is_empty() {
            platform_failure =
                Some("Unsupported native action requested by the single-window recipe");
        }
        if let Some(error) = failure {
            output.drop_without_applying_deltas();
            return Err(error.into());
        }
        if let Some(error) = platform_failure {
            output.drop_without_applying_deltas();
            return Err(error.into());
        }
        if quit {
            // Keyboard/app Close has the same unconditional policy as the OS close event.
            output.drop_without_applying_deltas();
            events.exit();
            return Ok(());
        }
        let view = surface_image.texture.create_view(&Default::default());
        self.drawing.render(
            output,
            self.started.elapsed().as_secs_f64(),
            Target {
                view: &view,
                size: [size.width, size.height],
            },
        )?;
        self.window.pre_present_notify();
        self.drawing.queue.present(surface_image);
        self.drawing.device.poll(wgpu::PollType::Poll)?;
        // A bounded synthetic producer also demonstrates an unrelated redraw during editing.
        self.context
            .request_repaint_after(Duration::from_millis(500));
        Ok(())
    }
}

// Closing is application policy, not a side effect of mutating a temporary ViewportInfo.
fn requests_quit(commands: &[egui::ViewportCommand]) -> bool {
    commands
        .iter()
        .any(|command| matches!(command, egui::ViewportCommand::Close))
}

struct Target<'a> {
    view: &'a wgpu::TextureView,
    size: [u32; 2],
}

// Own one device/renderer/compositor/source combination. Both the real window
// and the readback regression use this exact production and submission path.
struct Drawing {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: egui_wgpu::Renderer,
    compositor: egui_prism_wgpu::Host,
    image: GpuImage,
}

impl Drawing {
    fn new(
        context: &egui::Context,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let mut renderer = egui_wgpu::Renderer::new(
            &device,
            format,
            egui_wgpu::RendererOptions {
                dithering: false,
                ..Default::default()
            },
        );
        let compositor = egui_prism_wgpu::Host::new(context, &device)?;
        let image = GpuImage::new(&device, &mut renderer);
        Ok(Self {
            device,
            queue,
            renderer,
            compositor,
            image,
        })
    }

    fn render(
        &mut self,
        mut output: egui::FullOutput,
        seconds: f64,
        target: Target<'_>,
    ) -> Result<()> {
        for (id, deltas) in output.textures_delta.set.drain() {
            for delta in deltas {
                self.renderer
                    .update_texture(&self.device, &self.queue, id, &delta);
            }
        }
        let free: Vec<_> = output.textures_delta.free.drain().collect();
        output.textures_delta.clear();
        let result = (|| -> Result<()> {
            let mut encoder = self.device.create_command_encoder(&Default::default());
            // Production is in this encoder before any compositor pass samples the image.
            self.image.produce(&mut encoder, seconds);
            let screen = egui_wgpu::ScreenDescriptor {
                size_in_pixels: target.size,
                pixels_per_point: output.pixels_per_point,
            };
            let mut frame =
                self.compositor
                    .prepare_frame(&mut self.renderer, output.shapes, screen)?;
            let callback_buffers = frame.encode(&self.queue, &mut encoder)?;
            {
                let mut pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("settings final output"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target.view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    })
                    .forget_lifetime();
                frame.render(&mut pass)?;
            }
            self.queue
                .submit(callback_buffers.into_iter().chain([encoder.finish()]));
            drop(frame);
            Ok(())
        })();
        for id in free {
            self.renderer.free_texture(&id);
        }
        result
    }
}

impl Drop for Drawing {
    fn drop(&mut self) {
        self.compositor.invalidate();
        self.renderer.free_texture(&self.image.id);
        // Dropping handles does not explicitly destroy resources retained by submitted work.
    }
}

struct GpuImage {
    // Keep the application-owned source alive until the runtime is disposed.
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    id: egui::TextureId,
}

impl GpuImage {
    fn new(device: &wgpu::Device, renderer: &mut egui_wgpu::Renderer) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("current-frame synthetic source"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);
        Self {
            _texture: texture,
            view,
            id,
        }
    }

    fn produce(&self, encoder: &mut wgpu::CommandEncoder, seconds: f64) {
        let color = wgpu::Color {
            r: 0.2,
            g: 0.35 + 0.25 * seconds.sin(),
            b: 0.8,
            a: 1.0,
        };
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("synthetic image producer before group consumption"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }

    fn paint(&self, ui: &egui::Ui) -> std::result::Result<(), compositing::Error> {
        let origin = ui.max_rect().right_bottom() - egui::vec2(190.0, 90.0);
        let grouped = egui::Rect::from_min_size(origin, egui::vec2(110.0, 60.0));
        let overlay =
            egui::Rect::from_min_size(origin + egui::vec2(125.0, 0.0), egui::vec2(50.0, 60.0));
        let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
        let group = Group::new(
            grouped,
            0.7,
            vec![
                Item::Shape(egui::epaint::ClippedShape::new(
                    ui.clip_rect(),
                    egui::Shape::image(self.id, grouped, uv, egui::Color32::WHITE),
                )),
                Item::Shape(egui::epaint::ClippedShape::new(
                    ui.clip_rect(),
                    egui::Shape::rect_filled(grouped.shrink(20.0), 0.0, egui::Color32::WHITE),
                )),
            ],
        )?;
        compositing::paint(ui.painter(), group)?;
        ui.painter()
            .image(self.id, overlay, uv, egui::Color32::WHITE);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_quit_shortcut_reaches_host_close_policy() {
        for consume in [false, true] {
            let context = egui::Context::default();
            context.options_mut(|options| {
                options.ordered_input = true;
                options.max_passes = 1.try_into().unwrap();
                options.quit_shortcuts = vec![egui::KeyboardShortcut::new(
                    egui::Modifiers::CTRL,
                    egui::Key::Q,
                )];
            });
            let input = egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Q,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::CTRL,
                }],
                time: Some(0.0),
                ..Default::default()
            };
            let output = context.run_ui(input, |_| {
                context.begin_input();
                while context.next_input_event().is_some() {
                    if consume {
                        context.input_mut(|input| input.events.clear());
                    }
                }
            });
            assert_eq!(
                requests_quit(&output.viewport_output[&egui::ViewportId::ROOT].commands),
                !consume
            );
            output.drop_without_applying_deltas();
            let output = context.run_ui(
                egui::RawInput {
                    time: Some(1.0),
                    ..Default::default()
                },
                |_| {
                    context.begin_input();
                    while context.next_input_event().is_some() {}
                },
            );
            assert!(
                !requests_quit(&output.viewport_output[&egui::ViewportId::ROOT].commands),
                "quit must not replay on an unrelated frame"
            );
            output.drop_without_applying_deltas();
        }
    }

    #[test]
    #[ignore = "requires a native adapter; Linux CI runs this with Mesa software Vulkan"]
    fn current_frame_image_reaches_group_and_overlay() {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .expect("GPU adapter required");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let context = egui::Context::default();
        let mut drawing =
            Drawing::new(&context, device, queue, wgpu::TextureFormat::Rgba8Unorm).unwrap();
        let target = drawing.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("recipe readback target"),
            size: wgpu::Extent3d {
                width: 512,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let mut previous: Option<([u8; 4], [u8; 4])> = None;
        for seconds in [0.0, std::f64::consts::FRAC_PI_2] {
            let mut points = [egui::Pos2::ZERO; 2];
            let output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(512.0, 256.0),
                    )),
                    time: Some(seconds),
                    ..Default::default()
                },
                |ui| {
                    drawing.image.paint(ui).unwrap();
                    let origin = ui.max_rect().right_bottom() - egui::vec2(190.0, 90.0);
                    points = [
                        origin + egui::vec2(8.0, 8.0),
                        origin + egui::vec2(133.0, 8.0),
                    ];
                },
            );
            drawing
                .render(
                    output,
                    seconds,
                    Target {
                        view: &view,
                        size: [512, 256],
                    },
                )
                .unwrap();
            let bytes = drawing.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("recipe pixel readback"),
                size: 512 * 256 * 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut copy = drawing.device.create_command_encoder(&Default::default());
            copy.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &bytes,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(512 * 4),
                        rows_per_image: Some(256),
                    },
                },
                wgpu::Extent3d {
                    width: 512,
                    height: 256,
                    depth_or_array_layers: 1,
                },
            );
            drawing.queue.submit([copy.finish()]);
            let (tx, rx) = std::sync::mpsc::channel();
            bytes
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
            drawing
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .unwrap();
            rx.recv().unwrap().unwrap();
            let pixels = bytes.slice(..).get_mapped_range().unwrap();
            let pixel = |point: egui::Pos2| -> [u8; 4] {
                let at = ((point.y as usize * 512) + point.x as usize) * 4;
                pixels[at..at + 4].try_into().unwrap()
            };
            let group = pixel(points[0]);
            let overlay = pixel(points[1]);
            assert_eq!(group[3], 255);
            assert_eq!(overlay[3], 255);
            assert!(overlay[0].abs_diff(51) <= 2, "source red: {overlay:?}");
            assert!(overlay[2].abs_diff(204) <= 2, "source blue: {overlay:?}");
            let expected_green = (255.0 * (0.35 + 0.25 * seconds.sin())).round() as u8;
            assert!(
                overlay[1].abs_diff(expected_green) <= 2,
                "current source green: {overlay:?}"
            );
            let expected_group_green =
                (255.0 * (0.3 + 0.7 * (0.35 + 0.25 * seconds.sin()))).round() as u8;
            assert!(
                group[1].abs_diff(expected_group_green) <= 2,
                "group sampled a non-current source: {group:?}"
            );
            assert!(
                group[0] > overlay[0],
                "group must fade over white: {group:?}"
            );
            if let Some((old_group, old_overlay)) = previous {
                assert!(group[1] > old_group[1] + 20, "group sampled stale source");
                assert!(
                    overlay[1] > old_overlay[1] + 40,
                    "overlay sampled stale source"
                );
            }
            previous = Some((group, overlay));
            drop(pixels);
            bytes.unmap();
        }
    }
}
