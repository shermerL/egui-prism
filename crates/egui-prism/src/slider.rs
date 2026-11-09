//! Labeled single-value sliders with host-owned input and drawing phases.

use std::{ops::RangeInclusive, sync::Arc};

use crate::{
    compositing::{self, Error, Group, Item},
    egui,
    theme::Theme,
};

/// A horizontal slider configured before the host dispatches input.
///
/// ```
/// use egui_prism::{egui, slider::Slider, theme::Theme};
/// let theme = Theme::light();
/// let context = egui::Context::default();
/// context.options_mut(|o| o.ordered_input = true);
/// let mut amount = 0.5;
/// context.run_ui(Default::default(), |ui| {
///     context.begin_input();
///     let slider = Slider::new(&theme, "Amount").step_by(0.1).register(ui, amount);
///     while let Some(event) = context.next_input_event() {
///         if event.target == Some(slider.id()) {
///             slider.handle(ui, &mut amount, &event);
///         }
///     }
///     let output = slider.finish(ui, amount);
///     // A real host submits output.drawing through its installed compositor.
///     assert!(!output.response.changed());
/// }).drop_without_applying_deltas();
/// ```
#[must_use = "register the slider before dispatching input"]
pub struct Slider<'theme> {
    theme: &'theme Theme,
    label: egui::WidgetText,
    range: RangeInclusive<f64>,
    step: f64,
}
impl<'theme> Slider<'theme> {
    /// Configure a labeled slider over 0..=1 with a default step of one.
    pub fn new(theme: &'theme Theme, label: impl Into<egui::WidgetText>) -> Self {
        let label = label.into();
        assert!(
            !label.text().trim().is_empty(),
            "Slider requires a nonempty label"
        );
        Self {
            theme,
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
    /// Snap edits to a positive finite step from the range start.
    pub fn step_by(mut self, step: f64) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "Slider requires a positive finite step"
        );
        self.step = step;
        self
    }
    /// Register this frame's geometry and focus policy without editing the value.
    pub fn register(self, ui: &mut egui::Ui, value: f64) -> SliderRegistration {
        assert!(value.is_finite(), "Slider requires a finite value");
        assert!(
            ui.ctx().options(|o| o.ordered_input),
            "Slider requires ordered input"
        );
        ui.vertical(|ui| {
            let (label_pos, label, label_response) = egui::Label::new(self.label)
                .wrap()
                .selectable(false)
                .layout_in_ui(ui);
            label_response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), label.text())
            });
            let width = ui
                .spacing()
                .slider_width
                .min(ui.available_width())
                .max(16.0);
            let (id, bounds) = ui.allocate_space(egui::vec2(width, 16.0));
            let registration = SliderRegistration {
                muted: self.theme.colors.muted.background,
                primary: self.theme.colors.primary.background,
                ring: self.theme.colors.ring,
                range: self.range,
                step: self.step,
                id,
                rail: egui::Rect::from_center_size(bounds.center(), egui::vec2(width, 6.0)),
                layer: ui.layer_id(),
                clip: ui.clip_rect(),
                enabled: ui.is_enabled(),
                visible: ui.is_visible(),
                label,
                label_pos,
                label_id: label_response.id,
                label_color: ui.visuals().text_color(),
            };
            let response = registration.register_parts(ui, value);
            if !registration.enabled
                || !registration.visible
                || !ui.memory(|m| m.allows_interaction(registration.layer))
                || (!response.is_pointer_button_down_on()
                    && ui.ctx().dragged_id() != Some(id)
                    && !ui.input(|i| i.pointer.primary_down()))
            {
                ui.data_mut(|d| d.remove::<DragMapping>(id.with("mapping")));
            }
            registration
        })
        .inner
    }
}

/// This frame's registered geometry and drawing colors, separate from the caller's value.
#[must_use = "handle routed events, then finish the slider"]
pub struct SliderRegistration {
    muted: egui::Color32,
    primary: egui::Color32,
    ring: egui::Color32,
    range: RangeInclusive<f64>,
    step: f64,
    id: egui::Id,
    rail: egui::Rect,
    layer: egui::LayerId,
    clip: egui::Rect,
    enabled: bool,
    visible: bool,
    label: Arc<egui::Galley>,
    label_pos: egui::Pos2,
    label_id: egui::Id,
    label_color: egui::Color32,
}

// Only Radix's cached coordinate mapping lives here. egui owns capture and eligibility.
#[derive(Clone, Copy)]
struct DragMapping {
    rail: egui::Rect,
    last: egui::Pos2,
}

impl SliderRegistration {
    /// The authoritative thumb owner to match against a routed event's target.
    pub fn id(&self) -> egui::Id {
        self.id
    }

    /// The registered label and owner IDs for host-owned retirement.
    /// Retiring the owner also removes its bound track.
    pub fn input_ids(&self) -> [egui::Id; 2] {
        [self.label_id, self.id]
    }

    fn response(&self, ui: &mut egui::Ui, value: f64) -> egui::Response {
        assert_eq!(
            ui.layer_id(),
            self.layer,
            "slider handling must use its registered layer"
        );
        let mut child = ui.new_child(egui::UiBuilder::new().id_salt(self.id).max_rect(self.rail));
        child.set_clip_rect(self.clip);
        if !self.enabled {
            child.disable();
        }
        if !self.visible {
            child.set_invisible();
        }
        let rail = child.interact(
            self.rail,
            self.id.with("track"),
            egui::Sense::CLICK | egui::Sense::DRAG,
        );
        let thumb = child.interact(self.thumb(value), self.id, egui::Sense::click_and_drag());
        thumb.union(rail).labelled_by(self.label_id)
    }
    fn thumb(&self, value: f64) -> egui::Rect {
        egui::Rect::from_center_size(
            thumb_center(self.rail, value, *self.range.start(), *self.range.end()),
            egui::Vec2::splat(16.0),
        )
    }
    fn register_parts(&self, ui: &mut egui::Ui, value: f64) -> egui::Response {
        let response = self.response(ui, value);
        ui.ctx().set_input_owner(self.id.with("track"), self.id);
        ui.ctx().set_input_rounding(self.id.with("track"), 3.0);
        ui.ctx().set_input_rounding(self.id, 8.0);
        if self.enabled {
            ui.memory_mut(|m| {
                m.register_focus_policy(
                    self.id,
                    self.layer,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        }
        response
    }
    /// Apply one routed event and return its change response before the next event.
    pub fn handle(
        &self,
        ui: &mut egui::Ui,
        value: &mut f64,
        routed: &egui::RoutedInput,
    ) -> egui::Response {
        assert!(value.is_finite(), "Slider requires a finite value");
        assert_eq!(
            ui.layer_id(),
            self.layer,
            "slider handling must use its registered layer"
        );
        assert_eq!(
            routed.target,
            Some(self.id),
            "route only the slider owner's events"
        );
        let ctx = ui.ctx();
        let mut response = ctx
            .read_response(self.id)
            .expect("slider owner is registered");
        let mapping_id = self.id.with("mapping");
        let mut mapping = ui.data(|d| d.get_temp::<DragMapping>(mapping_id));
        let before = *value;
        let (min, max) = (*self.range.start(), *self.range.end());
        let value_at = |rect: egui::Rect, pos: egui::Pos2| {
            min + f64::from((pos.x - rect.left()) / rect.width()) * (max - min)
        };
        let mut next = None;
        let eligible =
            self.enabled && self.visible && ui.memory(|m| m.allows_interaction(self.layer));
        let owns_pointer =
            response.is_pointer_button_down_on() || ctx.dragged_id() == Some(self.id);
        if eligible && let egui::InputTrigger::Event { event, .. } = &routed.trigger {
            match event {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    ..
                } if owns_pointer && ui.input(|i| i.pointer.primary_down()) => {
                    let transform = ui
                        .ctx()
                        .layer_transform_to_global(self.layer)
                        .unwrap_or_default();
                    mapping = Some(DragMapping {
                        rail: transform * self.rail,
                        last: *pos,
                    });
                    // Ownership is already resolved. This only distinguishes the thumb's
                    // no-jump press from a rail press; it does not choose the input target.
                    if (transform.inverse() * *pos).distance(self.thumb(*value).center()) > 8.0 {
                        next = Some(value_at(transform * self.rail, *pos));
                    }
                }
                egui::Event::PointerMoved(pos)
                    if owns_pointer && ui.input(|i| i.pointer.primary_down()) =>
                {
                    if let Some(state) = mapping.as_mut()
                        && state.last != *pos
                    {
                        state.last = *pos;
                        next = Some(value_at(state.rail, *pos));
                    }
                }
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                } => {
                    // Radix ends the drag on release. Only press and motion edit the value.
                    mapping = None;
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
                    next = match key {
                        Key::Home => Some(min),
                        Key::End => Some(max),
                        Key::ArrowRight | Key::ArrowUp | Key::PageUp => {
                            Some(*value + self.step * multiplier)
                        }
                        Key::ArrowLeft | Key::ArrowDown | Key::PageDown => {
                            Some(*value - self.step * multiplier)
                        }
                        _ => None,
                    };
                }
                egui::Event::AccessKitActionRequest(request) => {
                    use egui::accesskit::{Action, ActionData};
                    next = match (&request.action, &request.data) {
                        (Action::SetValue, Some(ActionData::NumericValue(v))) if v.is_finite() => {
                            Some(*v)
                        }
                        (Action::Increment, _) => Some(*value + self.step),
                        (Action::Decrement, _) => Some(*value - self.step),
                        _ => None,
                    };
                }
                _ => {}
            }
        }
        if let Some(next) = next {
            *value = snap(next, min, max, self.step);
        }
        if response.drag_stopped()
            || !eligible
            || !owns_pointer
            || !ui.input(|i| i.pointer.primary_down())
        {
            mapping = None;
        }
        ui.data_mut(|d| {
            if let Some(mapping) = mapping {
                d.insert_temp(mapping_id, mapping);
            } else {
                d.remove::<DragMapping>(mapping_id);
            }
        });
        if *value != before {
            response.mark_changed();
            // A later event in this batch must hit the newly positioned thumb.
            ui.ctx()
                .clone()
                .input_layout(|| self.register_parts(ui, *value));
        }
        response
    }
    /// Build final unfaded drawing after the host has drained input.
    pub fn finish(self, ui: &mut egui::Ui, value: f64) -> SliderOutput {
        assert!(value.is_finite(), "Slider requires a finite value");
        let response = self.response(ui, value);
        let thumb = self.thumb(value);
        if !response.is_pointer_button_down_on()
            && ui.ctx().dragged_id() != Some(self.id)
            && !ui.input(|i| i.pointer.primary_down())
        {
            ui.data_mut(|d| d.remove::<DragMapping>(self.id.with("mapping")));
        }
        response.widget_info(|| egui::WidgetInfo::slider(self.enabled, value, ""));
        ui.ctx().accesskit_node_builder(self.label_id, |node| {
            node.set_label(self.label.text());
        });
        ui.ctx().accesskit_node_builder(self.id, |node| {
            use egui::accesskit::Action;
            node.set_min_numeric_value(*self.range.start());
            node.set_max_numeric_value(*self.range.end());
            node.set_numeric_value_step(self.step);
            if self.enabled {
                node.add_action(Action::SetValue);
                if value < *self.range.end() {
                    node.add_action(Action::Increment);
                }
                if value > *self.range.start() {
                    node.add_action(Action::Decrement);
                }
            }
        });
        // The label sits outside the reference Slider root and its disabled opacity.
        if self.visible {
            ui.painter().with_clip_rect(self.clip).galley(
                self.label_pos,
                self.label,
                self.label_color,
            );
        }
        let center = thumb.center();
        let hovered = self.enabled
            && response.hovered()
            && ui.input(|i| i.pointer.hover_pos()).is_some_and(|p| {
                let transform = ui
                    .ctx()
                    .layer_transform_to_global(self.layer)
                    .unwrap_or_default();
                (transform.inverse() * p).distance(center) <= 8.0
            });
        // Navigation events may be untargeted after the dispatcher moves focus.
        // Raw events are observed only for focus-ring modality, never replayed as edits.
        let modality = ui.input(|i| {
            i.raw.events.iter().rev().find_map(|event| match event {
                egui::Event::PointerButton { pressed: true, .. } => Some(false),
                egui::Event::Key {
                    pressed: true,
                    modifiers,
                    ..
                } if !modifiers.alt && !modifiers.ctrl && !modifiers.mac_cmd => Some(true),
                _ => None,
            })
        });
        let focus_visible = modality.unwrap_or_else(|| {
            ui.data(|d| {
                d.get_temp::<bool>(self.id.with("focus-visible"))
                    .unwrap_or(true)
            })
        });
        ui.data_mut(|d| d.insert_temp(self.id.with("focus-visible"), focus_visible));
        let ring = ui.ctx().animate_bool_with_time(
            self.id.with("ring"),
            hovered || (self.enabled && response.has_focus() && focus_visible),
            0.15,
        );
        let bounds = self.rail.union(thumb.expand(5.0));
        let drawing = if self.visible && bounds.intersects(self.clip) {
            let fraction = (value.clamp(*self.range.start(), *self.range.end())
                - self.range.start())
                / (self.range.end() - self.range.start());
            let split = self.rail.left() + self.rail.width() * fraction as f32;
            let fill =
                egui::Rect::from_min_max(self.rail.min, egui::pos2(split, self.rail.bottom()));
            let remainder =
                egui::Rect::from_min_max(egui::pos2(split, self.rail.top()), self.rail.max);
            let mut shapes = vec![
                egui::epaint::ClippedShape::new(
                    remainder.intersect(self.clip),
                    egui::Shape::rect_filled(self.rail, 3, self.muted),
                ),
                egui::epaint::ClippedShape::new(
                    fill.intersect(self.clip),
                    egui::Shape::rect_filled(self.rail, 3, self.primary),
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
                    self.clip,
                    shadow.as_shape(thumb.shrink(inset), (8.0 - inset) as u8),
                ));
            }
            if ring > 0.0 {
                shapes.push(egui::epaint::ClippedShape::new(
                    self.clip,
                    egui::Shape::circle_stroke(
                        center,
                        8.0 + 2.0 * ring,
                        egui::Stroke::new(4.0 * ring, self.ring.gamma_multiply(0.5)),
                    ),
                ));
            }
            shapes.push(egui::epaint::ClippedShape::new(
                self.clip,
                egui::Shape::Circle(egui::epaint::CircleShape {
                    center,
                    radius: 7.5,
                    fill: egui::Color32::WHITE,
                    stroke: egui::Stroke::new(1.0, self.primary),
                }),
            ));
            Some(
                Group::new(
                    bounds,
                    if self.enabled { 1.0 } else { 0.5 },
                    shapes.into_iter().map(Item::Shape).collect(),
                )
                .expect("valid slider drawing"),
            )
        } else {
            None
        };
        SliderOutput { response, drawing }
    }
}

/// A final slider response and the isolated drawing submitted by its caller.
#[must_use = "paint the drawing or include it in a parent Group"]
pub struct SliderOutput {
    /// Final focus and hover state; changes were returned by the event handler.
    pub response: egui::Response,
    /// Unfaded root drawing, absent when invisible or clipped out.
    pub drawing: Option<Group>,
}
impl SliderOutput {
    /// Submit with a same-layer painter saved before entering a disabled scope.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "slider painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "slider painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
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
    fn round_up(x: f64) -> f64 {
        let lo = x.floor();
        lo + if x - lo >= 0.5 { 1.0 } else { 0.0 }
    }

    let offset = value - min;
    let ticks = if offset.is_finite() {
        offset / step
    } else {
        value / step - min / step
    };
    let snapped = if ticks.is_finite() {
        round_up(ticks).mul_add(step, min)
    } else if value.is_finite() {
        // The step count overflowed. Find the nearest grid point relative to
        // the edit so a distant minimum cannot erase its local precision.
        let remainder = (value % step - min % step) % step;
        let distance = remainder.abs();
        let other = step - distance;
        if distance < other || (distance == other && remainder <= 0.0) {
            value - remainder
        } else {
            value + other.copysign(remainder)
        }
    } else {
        value
    };

    // Both the origin and the step contribute to the grid's decimal precision.
    let decimal_places = |v: f64| v.to_string().split_once('.').map_or(0, |(_, s)| s.len());
    let digits = decimal_places(min).max(decimal_places(step));
    let scale = 10.0_f64.powi(digits as i32);
    let scaled = snapped * scale;
    let rounded = if scaled.is_finite() {
        round_up(scaled) / scale
    } else {
        snapped
    };
    rounded.clamp(min, max)
}
