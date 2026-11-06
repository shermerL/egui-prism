//! Labeled single-value sliders following the pinned shadcn/Radix contract.

use std::ops::RangeInclusive;

use crate::{egui, theme::Theme};

/// A horizontal slider borrowing the caller's value and theme for one frame.
///
/// ```
/// use egui_prism::{egui, slider::Slider, theme::Theme};
/// let theme = Theme::light();
/// let mut amount = 0.5;
/// egui::__run_test_ui(|ui| {
///     ui.add(Slider::new(&theme, &mut amount, "Amount").step_by(0.1));
/// });
/// ```
#[must_use = "Add the slider to a Ui with ui.add(slider)"]
pub struct Slider<'theme, 'value> {
    theme: &'theme Theme,
    value: &'value mut f64,
    label: egui::WidgetText,
    range: RangeInclusive<f64>,
    step: f64,
}

impl<'theme, 'value> Slider<'theme, 'value> {
    /// Create a labeled slider over 0..=1; a nonfinite value or empty label is invalid.
    pub fn new(
        theme: &'theme Theme,
        value: &'value mut f64,
        label: impl Into<egui::WidgetText>,
    ) -> Self {
        let label = label.into();
        assert!(value.is_finite(), "Slider requires a finite value");
        assert!(
            !label.text().trim().is_empty(),
            "Slider requires a nonempty label"
        );
        Self {
            theme,
            value,
            label,
            range: 0.0..=1.0,
            step: 1.0,
        }
    }

    /// Set finite increasing bounds; only user edits are clamped to them.
    pub fn range(mut self, range: RangeInclusive<f64>) -> Self {
        assert!(
            range.start().is_finite()
                && range.end().is_finite()
                && range.start() < range.end()
                && (range.end() - range.start()).is_finite(),
            "Slider requires finite increasing bounds"
        );
        self.range = range;
        self
    }

    /// Snap edits to a positive finite step from the range start (default: 1).
    pub fn step_by(mut self, step: f64) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "Slider requires a positive finite step"
        );
        self.step = step;
        self
    }
}

// Radix maps pointer coordinates over the whole rail, but insets the painted
// thumb by 8 * (1 - 2 * fraction). Keep these deliberately distinct.
fn thumb_center(rect: egui::Rect, value: f64, min: f64, max: f64) -> egui::Pos2 {
    let fraction = ((value.clamp(min, max) - min) / (max - min)) as f32;
    egui::pos2(
        rect.left() + 8.0 + (rect.width() - 16.0) * fraction,
        rect.center().y,
    )
}

fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let snapped = ((value - min) / step + 0.5).floor() * step + min;
    // Decimal rounding removes binary drift in repeated fractional key steps.
    let digits = step
        .to_string()
        .split_once('.')
        .map_or(0, |(_, tail)| tail.len());
    let scale = 10.0_f64.powi(digits.min(308) as i32);
    let rounded = if (snapped * scale).is_finite() {
        (snapped * scale + 0.5).floor() / scale
    } else {
        snapped
    };
    rounded.clamp(min, max)
}

#[derive(Clone, Copy)]
struct Capture {
    rail_global: egui::Rect,
    last_pointer: egui::Pos2,
    frame: u64,
}

impl egui::Widget for Slider<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (min, max) = (*self.range.start(), *self.range.end());
        ui.vertical(|ui| {
            let label = ui.add(egui::Label::new(self.label).wrap());
            // A 16-point minimum is the thumb's intrinsic footprint. At smaller
            // offered widths it may overflow, but its travel can never reverse.
            let width = ui
                .spacing()
                .slider_width
                .min(ui.available_width())
                .max(16.0);
            let (id, bounds) = ui.allocate_space(egui::vec2(width, 16.0));
            let rail = egui::Rect::from_center_size(bounds.center(), egui::vec2(width, 6.0));
            let center = thumb_center(rail, *self.value, min, max);
            let thumb = egui::Rect::from_center_size(center, egui::Vec2::splat(16.0));
            let track_response = ui.interact(
                rail,
                id.with("track"),
                egui::Sense::CLICK | egui::Sense::DRAG,
            );
            let thumb_response = ui.interact(thumb, id, egui::Sense::click_and_drag());
            let mut response = thumb_response
                .clone()
                .union(track_response.clone())
                .labelled_by(label.id);
            let enabled = ui.is_enabled();
            let before = *self.value;
            let events = ui.input(|input| input.events.clone());
            let focus_id = id.with("focus-visible");
            let mut focus_visible = ui.data(|data| data.get_temp::<bool>(focus_id).unwrap_or(true));
            for event in &events {
                match event {
                    egui::Event::PointerButton { pressed: true, .. } => focus_visible = false,
                    egui::Event::Key {
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.alt && !modifiers.ctrl && !modifiers.mac_cmd => {
                        focus_visible = true
                    }
                    _ => {}
                }
            }
            ui.data_mut(|data| data.insert_temp(focus_id, focus_visible));
            let capture_id = id.with("capture");
            let frame = ui.ctx().cumulative_frame_nr();
            // Keep the initial rail in the same global space as pointer events.
            // This also preserves Radix's cached bounds if layout changes mid-drag.
            let mut capture = ui
                .data(|data| data.get_temp::<Capture>(capture_id))
                .filter(|state| state.frame == frame || state.frame.checked_add(1) == Some(frame));
            let interactive =
                enabled && ui.memory(|memory| memory.allows_interaction(ui.layer_id()));
            let primary_down = ui.input(|input| input.pointer.primary_down());
            if !interactive {
                capture = None;
            }
            if interactive {
                let to_global = ui
                    .ctx()
                    .layer_transform_to_global(ui.layer_id())
                    .unwrap_or(egui::emath::TSTransform::IDENTITY);
                let rail_global = to_global * rail;
                let thumb_global = to_global * thumb;
                let clip_global = to_global * ui.clip_rect();
                let occupied = ui.ctx().interaction_snapshot(|snapshot| {
                    snapshot
                        .clicked
                        .or(snapshot.dragged)
                        .is_some_and(|owner| owner != id && owner != track_response.id)
                });
                let value_at = |rect: egui::Rect, pos: egui::Pos2| {
                    min + f64::from((pos.x - rect.left()) / rect.width()) * (max - min)
                };
                for event in &events {
                    let next = match event {
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            ..
                        } => {
                            // egui has no drag response for a gesture completed in
                            // one frame. Hit-test that press before replaying its
                            // ordered movement/release, while respecting layers,
                            // clipping and any known competing widget owner.
                            let hit = !occupied
                                && (!primary_down || response.is_pointer_button_down_on())
                                && ui.ctx().layer_id_at(*pos) == Some(ui.layer_id())
                                && clip_global.contains(*pos)
                                && (rail_global.contains(*pos) || thumb_global.contains(*pos));
                            capture = hit.then_some(Capture {
                                rail_global,
                                last_pointer: *pos,
                                frame,
                            });
                            if hit {
                                response.request_focus();
                            }
                            (hit && !thumb_global.contains(*pos))
                                .then(|| value_at(rail_global, *pos))
                        }
                        egui::Event::PointerMoved(pos) => capture.as_mut().and_then(|state| {
                            if state.last_pointer == *pos {
                                return None;
                            }
                            state.last_pointer = *pos;
                            Some(value_at(state.rail_global, *pos))
                        }),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            ..
                        } => capture.take().and_then(|state| {
                            // Only a new press may acquire focus. Releasing an
                            // existing capture must respect the host's surrender.
                            (state.last_pointer != *pos).then(|| value_at(state.rail_global, *pos))
                        }),
                        egui::Event::PointerGone => {
                            capture = None;
                            None
                        }

                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } if response.has_focus() => {
                            use egui::Key;
                            let multiplier =
                                if modifiers.shift || matches!(key, Key::PageUp | Key::PageDown) {
                                    10.0
                                } else {
                                    1.0
                                };
                            match key {
                                Key::Home => Some(min),
                                Key::End => Some(max),
                                Key::ArrowRight | Key::ArrowUp | Key::PageUp => {
                                    Some(*self.value + self.step * multiplier)
                                }
                                Key::ArrowLeft | Key::ArrowDown | Key::PageDown => {
                                    Some(*self.value - self.step * multiplier)
                                }
                                _ => None,
                            }
                        }
                        egui::Event::AccessKitActionRequest(request)
                            if request.target_node == id.accesskit_id() =>
                        {
                            use egui::accesskit::{Action, ActionData};
                            match (&request.action, &request.data) {
                                (Action::SetValue, Some(ActionData::NumericValue(value)))
                                    if value.is_finite() =>
                                {
                                    Some(*value)
                                }
                                (Action::Increment, _) => Some(*self.value + self.step),
                                (Action::Decrement, _) => Some(*self.value - self.step),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(next) = next {
                        *self.value = snap(next, min, max, self.step);
                    }
                }
                if response.has_focus() {
                    // request_focus resets egui's filter, so install it after
                    // pointer presses/releases have transferred thumb focus.
                    ui.memory_mut(|memory| {
                        memory.set_focus_lock_filter(
                            id,
                            egui::EventFilter {
                                horizontal_arrows: true,
                                vertical_arrows: true,
                                ..Default::default()
                            },
                        )
                    });
                }
            }
            ui.data_mut(|data| {
                if primary_down && let Some(mut state) = capture {
                    state.frame = frame;
                    data.insert_temp(capture_id, state);
                } else {
                    data.remove::<Capture>(capture_id);
                }
            });
            if *self.value != before {
                response.mark_changed();
            }
            response.widget_info(|| egui::WidgetInfo::slider(enabled, *self.value, ""));
            ui.ctx().accesskit_node_builder(id, |node| {
                use egui::accesskit::Action;
                node.set_min_numeric_value(min);
                node.set_max_numeric_value(max);
                node.set_numeric_value_step(self.step);
                if enabled {
                    node.add_action(Action::SetValue);
                    if *self.value < max {
                        node.add_action(Action::Increment);
                    }
                    if *self.value > min {
                        node.add_action(Action::Decrement);
                    }
                }
            });
            if ui.is_rect_visible(bounds) {
                let center = thumb_center(rail, *self.value, min, max);
                let thumb = egui::Rect::from_center_size(center, egui::Vec2::splat(16.0));
                let hovered = enabled && ui.rect_contains_pointer(thumb);
                let ring = ui.ctx().animate_bool_with_time(
                    id.with("ring"),
                    hovered || (enabled && response.has_focus() && focus_visible),
                    0.15,
                );
                let mut painter = ui.painter().clone();
                // Replace egui's disabled alpha with shadcn's root opacity.
                if !enabled {
                    painter.set_opacity(if ui.visuals().disabled_alpha() > 0.0 {
                        ui.painter().opacity() / ui.visuals().disabled_alpha() * 0.5
                    } else {
                        0.0
                    });
                }
                let palette = &self.theme.colors;
                let fraction = ((*self.value).clamp(min, max) - min) / (max - min);
                let split = rail.left() + width * fraction as f32;
                let fill = egui::Rect::from_min_max(rail.min, egui::pos2(split, rail.bottom()));
                let remainder = egui::Rect::from_min_max(egui::pos2(split, rail.top()), rail.max);
                let mut shapes = vec![
                    egui::epaint::ClippedShape::new(
                        remainder.intersect(painter.clip_rect()),
                        egui::Shape::rect_filled(rail, 3, palette.muted.background),
                    ),
                    egui::epaint::ClippedShape::new(
                        fill.intersect(painter.clip_rect()),
                        egui::Shape::rect_filled(rail, 3, palette.primary.background),
                    ),
                ];
                for (blur, inset) in [(3, 0.0), (2, 1.0)] {
                    let shadow = egui::epaint::Shadow {
                        offset: [0, 1],
                        blur,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(26),
                    };
                    shapes.push(egui::epaint::ClippedShape::new(
                        painter.clip_rect(),
                        shadow.as_shape(thumb.shrink(inset), (8.0 - inset) as u8),
                    ));
                }
                for shape in shapes {
                    painter.with_clip_rect(shape.clip_rect).add(shape.shape);
                }
                if ring > 0.0 {
                    painter.circle_stroke(
                        center,
                        8.0 + 2.0 * ring,
                        egui::Stroke::new(4.0 * ring, palette.ring.gamma_multiply(0.5)),
                    );
                }
                painter.circle(
                    center,
                    7.5,
                    egui::Color32::WHITE,
                    egui::Stroke::new(1.0, palette.primary.background),
                );
            }
            response
        })
        .inner
    }
}
