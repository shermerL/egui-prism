//! Root-window native host. OS input/IME/clipboard stay in egui-winit.
use egui_prism::egui;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
enum Wake {
    Repaint(Instant),
    GpuError,
}
pub fn run() -> Result<()> {
    let event_loop = EventLoop::<Wake>::with_user_event().build()?;
    let mut app = App {
        proxy: event_loop.create_proxy(),
        runtime: None,
        deadline: None,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error.into());
    }
    Ok(())
}
struct App {
    proxy: EventLoopProxy<Wake>,
    runtime: Option<Runtime>,
    deadline: Option<Instant>,
    error: Option<String>,
}
impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        self.runtime.take();
        event_loop.exit();
    }
}
impl ApplicationHandler<Wake> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.runtime.is_some() {
            return;
        }
        match Runtime::new(event_loop, self.proxy.clone()) {
            Ok(runtime) => {
                runtime.window.request_redraw();
                self.runtime = Some(runtime);
            }
            Err(error) => self.fail(event_loop, error),
        }
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.runtime.take();
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Wake) {
        match event {
            Wake::Repaint(when) => {
                self.deadline = Some(self.deadline.map_or(when, |old| old.min(when)));
            }
            Wake::GpuError => {
                if let Some(runtime) = &mut self.runtime {
                    let error = runtime.gpu_error.lock().expect("GPU error lock").clone();
                    if let Some(error) = error {
                        runtime.gallery.compositor.invalidate();
                        self.fail(event_loop, error);
                    }
                }
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        if runtime.window.id() != id {
            return;
        }
        let response = runtime.input.on_window_event(&runtime.window, &event);
        match event {
            WindowEvent::RedrawRequested => {
                self.deadline = None;
                match runtime.paint(event_loop) {
                    Ok(retry) => self.deadline = retry,
                    Err(error) => self.fail(event_loop, error),
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                runtime.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                runtime.close(event_loop);
            }
            _ => {
                if response.repaint {
                    runtime.window.request_redraw();
                }
            }
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(when) = self.deadline {
            if when <= Instant::now() {
                self.deadline = None;
                if let Some(runtime) = &self.runtime {
                    runtime.window.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Wait);
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(when));
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.runtime.take();
    }
}
struct Runtime {
    gallery: super::Gallery,
    input: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    capture: Option<egui_wgpu::capture::CaptureState>,
    surface: wgpu::Surface<'static>,
    instance: wgpu::Instance,
    reconfigure: bool,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    context: egui::Context,
    gpu_error: Arc<Mutex<Option<String>>>,
    window: Arc<Window>,
}
impl Runtime {
    fn new(event_loop: &ActiveEventLoop, proxy: EventLoopProxy<Wake>) -> Result<Self> {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("egui-prism gallery")
                    .with_inner_size(winit::dpi::LogicalSize::new(960.0, 900.0)),
            )?,
        );
        let backends = if cfg!(target_os = "macos") {
            wgpu::Backends::METAL
        } else if cfg!(target_os = "windows") {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::VULKAN
        };
        let mut descriptor = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        ));
        descriptor.backends = backends;
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        println!("Native adapter: {:?}", adapter.get_info());
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
        let gpu_error = Arc::new(Mutex::new(None));
        let lost = gpu_error.clone();
        let wake = proxy.clone();
        device.set_device_lost_callback(move |reason, message| {
            if reason != wgpu::DeviceLostReason::Destroyed {
                *lost.lock().expect("GPU error lock") = Some(format!(
                    "device lost: {reason:?}: {message}; restart the host"
                ));
                let _ = wake.send_event(Wake::GpuError);
            }
        });
        let errors = gpu_error.clone();
        let wake = proxy.clone();
        device.on_uncaptured_error(Arc::new(move |error| {
            *errors.lock().expect("GPU error lock") = Some(format!("GPU error: {error}"));
            let _ = wake.send_event(Wake::GpuError);
        }));
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("no surface configuration")?;
        config.format = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .ok_or("gamma-color surface format unavailable")?;
        surface.configure(&device, &config);
        let context = egui::Context::default();
        let wake = proxy;
        context.set_request_repaint_callback(move |info| {
            let _ = wake.send_event(Wake::Repaint(Instant::now() + info.delay));
        });
        let input = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            Some(window.scale_factor() as f32),
            event_loop.system_theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let gallery = super::Gallery::new(&context, &device)?;
        let renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions {
                dithering: false,
                ..Default::default()
            },
        );
        Ok(Self {
            gallery,
            input,
            renderer,
            capture: None,
            surface,
            instance,
            reconfigure: false,
            config,
            device,
            queue,
            context,
            gpu_error,
            window,
        })
    }
    // Closing must run even when Wayland has withheld the next frame callback.
    fn close(&mut self, event_loop: &ActiveEventLoop) {
        let (mut output, actions, close) = self.update(event_loop, true);
        for id in output.textures_delta.free.drain() {
            self.renderer.free_texture(&id);
        }
        output.textures_delta.clear();
        if close {
            event_loop.exit();
        } else {
            // This UI pass did not paint. Preserve captures for the next painted pass.
            for action in actions {
                if let egui_winit::ActionRequested::Screenshot(callback) = action {
                    self.context
                        .send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
                }
            }
            self.window.request_redraw();
        }
    }
    fn update(
        &mut self,
        event_loop: &ActiveEventLoop,
        close_requested: bool,
    ) -> (egui::FullOutput, Vec<egui_winit::ActionRequested>, bool) {
        let mut raw = self.input.take_egui_input(&self.window);
        egui_winit::update_viewport_info(
            raw.viewports.entry(egui::ViewportId::ROOT).or_default(),
            &self.context,
            &self.window,
            false,
        );
        if close_requested {
            raw.viewports
                .entry(egui::ViewportId::ROOT)
                .or_default()
                .events
                .push(egui::ViewportEvent::Close);
        }

        let mut output = self
            .context
            .run_ui(raw, |ui| self.gallery.ui(ui, &self.device));
        self.input.handle_platform_output_with_event_loop(
            &self.window,
            event_loop,
            std::mem::take(&mut output.platform_output),
        );
        let mut actions = Vec::new();
        let mut close = false;
        if let Some(viewport) = output.viewport_output.remove(&egui::ViewportId::ROOT) {
            let mut info = self.context.input(|i| i.viewport().clone());
            close = should_close(&info, &viewport.commands);
            egui_winit::process_viewport_commands(
                &self.context,
                &mut info,
                viewport.commands,
                &self.window,
                &mut actions,
            );
        }
        for (id, deltas) in output.textures_delta.set.drain() {
            for delta in deltas {
                self.renderer
                    .update_texture(&self.device, &self.queue, id, &delta);
            }
        }
        (output, actions, close)
    }
    fn paint(&mut self, event_loop: &ActiveEventLoop) -> Result<Option<Instant>> {
        if let Some(error) = self.gpu_error.lock().expect("GPU error lock").clone() {
            return Err(error.into());
        }
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(None);
        }
        if self.reconfigure || self.config.width != size.width || self.config.height != size.height
        {
            self.reconfigure = false;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
        let surface_frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.reconfigure = true;
                frame
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone())?;
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                // No UI pass ran, so egui may still coalesce repaint requests.
                // The host must schedule every failed acquisition independently.
                return Ok(Some(Instant::now() + std::time::Duration::from_millis(50)));
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation failed".into());
            }
        };
        let (mut output, actions, close) = self.update(event_loop, false);
        let screenshots = actions
            .into_iter()
            .filter_map(|action| match action {
                egui_winit::ActionRequested::Screenshot(callback) => Some(callback),
                _ => None,
            })
            .collect::<Vec<_>>();
        let free = output.textures_delta.free.drain().collect::<Vec<_>>();
        output.textures_delta.clear();
        let rendered = (|| -> Result<()> {
            let screen = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: output.pixels_per_point,
            };
            let mut frame =
                self.gallery
                    .compositor
                    .prepare_frame(&mut self.renderer, output.shapes, screen)?;
            let mut encoder = self.device.create_command_encoder(&Default::default());
            let commands = frame.encode(&self.queue, &mut encoder)?;
            if !screenshots.is_empty() {
                let capture = self.capture.get_or_insert_with(|| {
                    egui_wgpu::capture::CaptureState::new(
                        &self.device,
                        surface_frame.texture.size(),
                        self.config.format,
                    )
                });
                capture.update(&self.device, surface_frame.texture.size());
            }
            let view = if screenshots.is_empty() {
                surface_frame.texture.create_view(&Default::default())
            } else {
                self.capture
                    .as_ref()
                    .expect("capture requested")
                    .texture
                    .create_view(&Default::default())
            };
            {
                let mut pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("gallery frame"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
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
            let buffer = if screenshots.is_empty() {
                None
            } else {
                Some(
                    self.capture
                        .as_mut()
                        .expect("capture requested")
                        .copy_textures(&self.device, Some(&surface_frame), &mut encoder),
                )
            };
            self.queue
                .submit(commands.into_iter().chain([encoder.finish()]));
            drop(frame);
            if let Some(buffer) = buffer {
                self.capture
                    .as_ref()
                    .expect("capture requested")
                    .read_screen_rgba(buffer, screenshots);
                self.device.poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(5)),
                })?;
            }
            self.window.pre_present_notify();
            self.queue.present(surface_frame);
            self.device.poll(wgpu::PollType::Poll)?;
            Ok(())
        })();
        for id in free {
            self.renderer.free_texture(&id);
        }
        rendered?;
        if close {
            event_loop.exit();
        }
        Ok(None)
    }
}

fn should_close(info: &egui::ViewportInfo, commands: &[egui::ViewportCommand]) -> bool {
    !commands
        .iter()
        .any(|command| matches!(command, egui::ViewportCommand::CancelClose))
        && (info.close_requested()
            || commands
                .iter()
                .any(|command| matches!(command, egui::ViewportCommand::Close)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn close_decisions_run_without_a_surface_or_redraw() {
        let context = egui::Context::default();
        for (os_close, command, expected) in [
            (true, None, true),
            (true, Some(egui::ViewportCommand::CancelClose), false),
            (false, None, false),
            (false, Some(egui::ViewportCommand::Close), true),
        ] {
            let mut raw = egui::RawInput::default();
            if os_close {
                raw.viewports
                    .entry(egui::ViewportId::ROOT)
                    .or_default()
                    .events
                    .push(egui::ViewportEvent::Close);
            }
            let mut output = context.run_ui(raw, |ui| {
                if let Some(command) = command.clone() {
                    ui.ctx().send_viewport_cmd(command);
                }
            });
            let info = context.input(|input| input.viewport().clone());
            let commands = &output.viewport_output[&egui::ViewportId::ROOT].commands;
            assert_eq!(should_close(&info, commands), expected);
            output.textures_delta.clear();
        }
    }
}
