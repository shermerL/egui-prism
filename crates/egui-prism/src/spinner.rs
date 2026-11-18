//! Caller-owned status drawing with the pinned one-second rotation.
//!
//! The caller owns visibility, the accessible label and reduced-motion policy.
//! Omit `show` when not busy, or use `animate(false)` for a static indicator.
//! Install a host compositor before submitting the returned drawing.

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};
use egui::{IdSalt, Response, Sense, Shape, pos2, vec2};

#[derive(Clone)]
struct Motion {
    start: f64,
    frame: u64,
    animate: bool,
}

/// A 16-point status indicator borrowing its theme and caller-provided label.
pub struct Spinner<'a> {
    theme: &'a Theme,
    id_salt: IdSalt,
    label: &'a str,
    animate: bool,
}

impl<'a> Spinner<'a> {
    /// Create an animated indicator with stable identity and caller-owned status text.
    pub fn new(theme: &'a Theme, id_salt: impl egui::AsIdSalt, label: &'a str) -> Self {
        Self {
            theme,
            id_salt: IdSalt::new(id_salt),
            label,
            animate: true,
        }
    }

    /// Disable rotation and repaint requests, for example for the host's reduced-motion preference.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Allocate a passive status and prepare its unfaded frame-local drawing.
    ///
    /// The host supplies time. Rotation begins when this ID first appears, on
    /// reentry after an omitted frame, or when animation is re-enabled. Clipping
    /// stops repaint requests but not elapsed time. No background timer is owned.
    pub fn show(self, ui: &mut egui::Ui) -> SpinnerOutput {
        let size = vec2(16.0, 16.0);
        let (_, allocated) = ui.allocate_space(size);
        let rect = ui.layout().align_size_within_rect(size, allocated);
        let id = ui.make_persistent_id(self.id_salt);
        let response = ui.interact(rect, id, Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Status, response.enabled(), self.label)
        });
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_live(egui::accesskit::Live::Polite);
            node.set_live_atomic();
        });
        let now = ui.input(|input| input.time);
        assert!(now.is_finite(), "Spinner requires finite host time");
        let frame = ui.ctx().cumulative_frame_nr();
        let key = id.with("spinner-motion");
        let mut motion = ui
            .ctx()
            .data_mut(|data| data.get_temp::<Motion>(key))
            .unwrap_or(Motion {
                start: now,
                frame,
                animate: self.animate,
            });
        if frame > motion.frame.saturating_add(1) || (self.animate && !motion.animate) {
            motion.start = now;
        }
        motion.frame = frame;
        motion.animate = self.animate;
        let angle = if self.animate {
            ((now - motion.start).rem_euclid(1.0) * std::f64::consts::TAU) as f32
        } else {
            0.0
        };
        ui.ctx().data_mut(|data| data.insert_temp(key, motion));
        let feather = ui.ctx().tessellation_options(|options| {
            if options.feathering {
                options.feathering_size_in_pixels / ui.pixels_per_point()
            } else {
                0.0
            }
        });
        let bounds = rect.expand(feather.max(1.0)).intersect(ui.clip_rect());
        let drawing = if ui.is_rect_visible(rect) && bounds.is_positive() {
            if self.animate && ui.painter().opacity() > 0.0 {
                ui.ctx().request_repaint();
            }
            // Lucide 1.21.0 LoaderCircle's exact SVG arc:
            // M21 12a9 9 0 1 1-6.219-8.56, viewBox 0 0 24 24.
            // Convert the endpoint arc to its circle, preserving the rounded caps.
            let start = pos2(21.0, 12.0);
            let end = pos2(14.781, 3.44);
            let half = (start - end) * 0.5;
            let factor = ((81.0 - half.length_sq()) / half.length_sq()).sqrt();
            let center = start.lerp(end, 0.5) + vec2(-half.y, half.x) * factor;
            let from = (start - center).angle();
            let sweep = ((end - center).angle() - from).rem_euclid(std::f32::consts::TAU);
            let rotation = egui::emath::Rot2::from_angle(angle);
            let color = self.theme.colors.background.foreground;
            let half_width = 2.0 / 3.0_f32;
            let inner = (half_width - feather * 0.5).max(0.0);
            let outer = half_width + feather * 0.5;
            let coverage = if feather > 0.0 {
                (2.0 * half_width / feather).min(1.0)
            } else {
                1.0
            };
            let solid = color.linear_multiply(coverage);
            let clear = egui::Color32::TRANSPARENT;
            let transform =
                |point| rect.center() + rotation * (point - pos2(12.0, 12.0)) * (2.0 / 3.0);
            let mut mesh = egui::Mesh::default();
            // Nonoverlapping strip and semicircular caps form one SVG stroke.
            // Adding circles over an open egui path would double-blend alpha
            // and overlap that path's existing feathered end caps.
            for index in 0..=64 {
                let normal = egui::Vec2::angled(from + sweep * index as f32 / 64.0);
                let point = transform(center + normal * 9.0);
                let normal = rotation * normal;
                for (offset, tint) in [
                    (-outer, clear),
                    (-inner, solid),
                    (inner, solid),
                    (outer, clear),
                ] {
                    mesh.colored_vertex(point + normal * offset, tint);
                }
                if index > 0 {
                    let previous = (index - 1) * 4;
                    let current = index * 4;
                    for band in 0..3 {
                        mesh.add_triangle(previous + band, current + band, current + band + 1);
                        mesh.add_triangle(previous + band, current + band + 1, previous + band + 1);
                    }
                }
            }
            for (angle, direction) in [(from, -1.0), (from + sweep, 1.0)] {
                let normal = egui::Vec2::angled(angle);
                let point = transform(center + normal * 9.0);
                let base = mesh.vertices.len() as u32;
                mesh.colored_vertex(point, solid);
                for index in 0..=16 {
                    let cap_angle = angle + direction * std::f32::consts::PI * index as f32 / 16.0;
                    let direction = rotation * egui::Vec2::angled(cap_angle);
                    mesh.colored_vertex(point + direction * inner, solid);
                    mesh.colored_vertex(point + direction * outer, clear);
                    if index > 0 {
                        let previous = base + 1 + (index - 1) * 2;
                        let current = base + 1 + index * 2;
                        mesh.add_triangle(base, previous, current);
                        mesh.add_triangle(previous, previous + 1, current + 1);
                        mesh.add_triangle(previous, current + 1, current);
                    }
                }
            }
            Some(
                Group::new(
                    bounds,
                    1.0,
                    vec![Item::Shape(egui::epaint::ClippedShape::new(
                        bounds,
                        Shape::mesh(mesh),
                    ))],
                )
                .expect("finite clipped spinner geometry"),
            )
        } else {
            None
        };
        SpinnerOutput { response, drawing }
    }
}

/// A passive status response and explicit, unfaded drawing.
#[must_use = "submit with paint or include the drawing in a parent Group"]
pub struct SpinnerOutput {
    /// A hover-only status, not a progress value or focus/activation target.
    pub response: Response,
    /// Visible drawing, absent when clipped out.
    pub drawing: Option<Group>,
}

impl SpinnerOutput {
    /// Submit using the host compositor and a same-context, same-layer painter.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "spinner painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "spinner painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}
