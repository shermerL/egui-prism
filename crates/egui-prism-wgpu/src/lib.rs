#![doc = include_str!("../../../docs/compositing.md")]
//! Native frame integration for isolated egui drawing groups.
//!
//! Upload current texture deltas, call [`Host::prepare_frame`], then
//! [`Frame::encode`] before [`Frame::render`]. Submit returned user command
//! buffers before the main encoder. Components never acquire renderer locks.
mod pipeline;
use egui_prism::{
    compositing::{self, Backend, Error, Group, Installation, Item},
    egui::{
        self, Rect,
        emath::TSTransform,
        epaint::{Mesh, Primitive},
    },
};
pub use egui_wgpu;
use egui_wgpu::{ScreenDescriptor, wgpu};
use pipeline::{Draw, Pipeline, Target};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
fn failure(message: impl Into<String>) -> Error {
    Error::Renderer(message.into())
}
struct Registration(Arc<AtomicBool>);
struct Submission {
    alive: Weak<AtomicBool>,
    group: Group,
}
impl Backend for Registration {
    fn callback(
        &self,
        _context: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, Error> {
        if !self.0.load(Ordering::Acquire) {
            return Err(Error::ContextLost);
        }
        Ok(egui::PaintCallback {
            rect: group.bounds(),
            callback: Arc::new(Submission {
                alive: Arc::downgrade(&self.0),
                group,
            }),
        })
    }
}
/// One device's compositor installation and bounded reusable render targets.
///
/// The host owns device creation and loss notification. Call [`Self::invalidate`]
/// on loss and recreate the complete host for a new device.
pub struct Host {
    context: egui::Context,
    installation: Option<Installation>,
    alive: Arc<AtomicBool>,
    device: wgpu::Device,
    pipeline: Pipeline,
    pool: Vec<Target>,
}
impl Host {
    /// Install a compositor for normal hardware-filtered, single-sample egui drawing.
    /// Use a gamma Rgba8Unorm/Bgra8Unorm final attachment and premultiplied colors.
    pub fn new(context: &egui::Context, device: &wgpu::Device) -> Result<Self, Error> {
        let alive = Arc::new(AtomicBool::new(true));
        let installation = compositing::install(context, Arc::new(Registration(alive.clone())))?;
        Ok(Self {
            context: context.clone(),
            installation: Some(installation),
            alive,
            device: device.clone(),
            pipeline: Pipeline::new(device),
            pool: Vec::new(),
        })
    }
    /// Revoke all old callbacks and remove this context's capability.
    pub fn invalidate(&mut self) {
        self.alive.store(false, Ordering::Release);
        self.installation.take();
        self.pool.clear();
    }
    /// Check that this installation has not been invalidated.
    pub fn check(&self) -> Result<(), Error> {
        if self.alive.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Error::ContextLost)
        }
    }
    /// Borrow current uploaded bindings and replace isolated groups with images.
    ///
    /// Call after all texture uploads. The renderer must belong to this device.
    /// Retain caller texture handles through submission. The returned frame removes
    /// only its own temporary registrations on drop, including preparation errors.
    pub fn prepare_frame<'a>(
        &'a mut self,
        renderer: &'a mut egui_wgpu::Renderer,
        jobs: Vec<egui::ClippedPrimitive>,
        screen: ScreenDescriptor,
    ) -> Result<Frame<'a>, Error> {
        self.check()?;
        if !screen.pixels_per_point.is_finite()
            || screen.pixels_per_point <= 0.0
            || screen.size_in_pixels.contains(&0)
        {
            return Err(failure("invalid screen extent or density"));
        }
        let mut frame = Frame {
            host: self,
            renderer,
            jobs: Vec::with_capacity(jobs.len()),
            screen,
            registered: Vec::new(),
            passes: Vec::new(),
            used: 0,
            encoded: false,
        };
        for mut job in jobs {
            if let Primitive::Callback(callback) = &job.primitive
                && let Some(submission) = callback.callback.downcast_ref::<Submission>()
            {
                let Some(alive) = submission.alive.upgrade() else {
                    continue;
                };
                if !alive.load(Ordering::Acquire) {
                    continue;
                }
                if !Arc::ptr_eq(&alive, &frame.host.alive) {
                    return Err(failure("group belongs to another host"));
                }
                let bounds = submission.group.bounds();
                let scale = callback.rect.width() / bounds.width();
                if !scale.is_finite() || scale <= 0.0 {
                    return Err(failure("invalid callback transform"));
                }
                let transform = TSTransform::new(
                    callback.rect.min.to_vec2() - bounds.min.to_vec2() * scale,
                    scale,
                );
                let ppp = frame.screen.pixels_per_point;
                let screen_rect = Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(
                        frame.screen.size_in_pixels[0] as f32,
                        frame.screen.size_in_pixels[1] as f32,
                    ) / ppp,
                );
                if let Some((index, rect)) = frame.group(
                    &frame.host.context.clone(),
                    &submission.group,
                    transform,
                    job.clip_rect.intersect(screen_rect),
                )? {
                    let mut mesh = Mesh::with_texture(frame.registered[index]);
                    mesh.add_rect_with_uv(
                        rect,
                        unit_uv(),
                        egui::Color32::WHITE.gamma_multiply(submission.group.opacity()),
                    );
                    job.primitive = Primitive::Mesh(mesh);
                } else {
                    continue;
                }
            }
            frame.jobs.push(job);
        }
        Ok(frame)
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.invalidate();
    }
}
fn unit_uv() -> Rect {
    Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0))
}
struct Pass {
    target: usize,
    draws: Vec<Draw>,
}
/// Frame-local exclusive host/renderer borrow with automatic temporary cleanup.
pub struct Frame<'a> {
    host: &'a mut Host,
    renderer: &'a mut egui_wgpu::Renderer,
    jobs: Vec<egui::ClippedPrimitive>,
    screen: ScreenDescriptor,
    registered: Vec<egui::TextureId>,
    passes: Vec<Pass>,
    used: usize,
    encoded: bool,
}
impl Frame<'_> {
    fn group(
        &mut self,
        context: &egui::Context,
        group: &Group,
        transform: TSTransform,
        inherited: Rect,
    ) -> Result<Option<(usize, Rect)>, Error> {
        let clip = (transform * group.bounds()).intersect(inherited);
        if !clip.is_positive() || group.opacity() == 0.0 {
            return Ok(None);
        }
        let ppp = self.screen.pixels_per_point;
        let min = (clip.min.to_vec2() * ppp).floor();
        let max = (clip.max.to_vec2() * ppp).ceil();
        let size = [(max.x - min.x) as u32, (max.y - min.y) as u32];
        if size.contains(&0)
            || size
                .iter()
                .any(|&side| side > self.host.device.limits().max_texture_dimension_2d)
        {
            return Err(failure("group exceeds device extent"));
        }
        let rect = Rect::from_min_max((min / ppp).to_pos2(), (max / ppp).to_pos2());
        let index = self.used;
        self.used += 1;
        if index == self.host.pool.len() {
            self.host.pool.push(Target::new(&self.host.device, size));
        } else if self.host.pool[index].size != size {
            self.host.pool[index] = Target::new(&self.host.device, size);
        }
        let id = self.renderer.register_native_texture(
            &self.host.device,
            &self.host.pool[index].view,
            wgpu::FilterMode::Linear,
        );
        self.registered.push(id);
        let mut draws = Vec::new();
        for item in group.items() {
            match item {
                Item::Shape(shape) => {
                    let mut shape = shape.clone();
                    shape.transform(transform);
                    for primitive in context.tessellate(vec![shape], ppp) {
                        let Primitive::Mesh(mesh) = primitive.primitive else {
                            unreachable!("core rejects callbacks")
                        };
                        let binding = self
                            .renderer
                            .texture(&mesh.texture_id)
                            .ok_or_else(|| {
                                failure(format!(
                                    "missing current-frame texture {:?}",
                                    mesh.texture_id
                                ))
                            })?
                            .bind_group
                            .clone();
                        if let Some(draw) = self.host.pipeline.mesh(
                            &self.host.device,
                            &mesh,
                            binding,
                            rect,
                            clip.intersect(primitive.clip_rect),
                            ppp,
                            1.0,
                        )? {
                            draws.push(draw);
                        }
                    }
                }
                Item::Group(child) => {
                    if let Some((child_index, child_rect)) =
                        self.group(context, child, transform, clip)?
                    {
                        let id = self.registered[child_index];
                        let binding = self
                            .renderer
                            .texture(&id)
                            .expect("owned registration")
                            .bind_group
                            .clone();
                        let mut mesh = Mesh::with_texture(id);
                        mesh.add_rect_with_uv(child_rect, unit_uv(), egui::Color32::WHITE);
                        if let Some(draw) = self.host.pipeline.mesh(
                            &self.host.device,
                            &mesh,
                            binding,
                            rect,
                            clip,
                            ppp,
                            child.opacity(),
                        )? {
                            draws.push(draw);
                        }
                    }
                }
            }
        }
        self.passes.push(Pass {
            target: index,
            draws,
        });
        Ok(Some((index, rect)))
    }
    /// Prepare ordinary callbacks, then encode offscreen groups in the main encoder.
    ///
    /// Submit returned buffers before the main encoder. Caller image producers in
    /// prepare/finish_prepare precede these groups; writes later in paint do not.
    pub fn encode(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Vec<wgpu::CommandBuffer>, Error> {
        self.host.check()?;
        if self.encoded {
            return Err(failure("frame already encoded"));
        }
        let commands = self.renderer.update_buffers(
            &self.host.device,
            queue,
            encoder,
            &self.jobs,
            &self.screen,
        );
        for pass in &self.passes {
            self.host
                .pipeline
                .draw(encoder, &self.host.pool[pass.target], &pass.draws);
        }
        self.encoded = true;
        Ok(commands)
    }
    /// Render final UI into the host's single-sample gamma-color render pass.
    pub fn render(&self, pass: &mut wgpu::RenderPass<'static>) -> Result<(), Error> {
        self.host.check()?;
        if !self.encoded {
            return Err(failure("encode before render"));
        }
        self.renderer.render(pass, &self.jobs, &self.screen);
        Ok(())
    }
}
impl Drop for Frame<'_> {
    fn drop(&mut self) {
        for id in self.registered.drain(..) {
            self.renderer.free_texture(&id);
        }
        self.host.pool.truncate(self.used);
    }
}

#[cfg(test)]
mod tests;
