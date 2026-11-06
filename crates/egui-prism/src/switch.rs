//! Controlled boolean switches using the pinned shadcn geometry and behavior.

use crate::{egui, theme::Theme};

mod transition;
use transition::{animate, color, color_components};

/// The two sizes supported by the pinned shadcn switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchSize {
    /// A 32 by 18.4 point track with a 16 point thumb.
    #[default]
    Default,
    /// A 24 by 14 point track with a 12 point thumb.
    Small,
}

/// A labeled switch borrowing the caller's boolean and theme for one frame.
///
/// Consume `Response::changed()` in the caller; the switch does not start work
/// or infer that an application operation succeeded.
///
/// ```
/// use egui_prism::{egui, switch::Switch, theme::Theme};
/// let theme = Theme::light();
/// let mut previews = false;
/// egui::__run_test_ui(|ui| {
///     if ui.add(Switch::new(&theme, &mut previews, "Show previews")).changed() {
///         // Apply application policy to the new value.
///     }
/// });
/// ```
#[must_use = "Add the switch to a Ui with ui.add(switch)"]
pub struct Switch<'theme, 'value> {
    theme: &'theme Theme,
    checked: &'value mut bool,
    label: egui::WidgetText,
    size: SwitchSize,
    invalid: bool,
}

impl<'theme, 'value> Switch<'theme, 'value> {
    /// Create a switch with a visible label; an empty label is a programmer error.
    pub fn new(
        theme: &'theme Theme,
        checked: &'value mut bool,
        label: impl Into<egui::WidgetText>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.text().trim().is_empty(),
            "Switch requires a nonempty label"
        );
        Self {
            theme,
            checked,
            label,
            size: SwitchSize::Default,
            invalid: false,
        }
    }

    /// Select the default or small reference geometry.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Expose invalid semantics without adding styling absent from the reference.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

#[derive(Clone)]
struct Interaction {
    keyboard: bool,
    space_armed: bool,
    pointer_armed: bool,
}

impl egui::Widget for Switch<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());
        let (track_width, track_height, diameter) = match self.size {
            SwitchSize::Default => (32.0, 18.4, 16.0),
            SwitchSize::Small => (24.0, 14.0, 12.0),
        };
        let label_text = self.label.text().to_owned();
        let mut job = std::sync::Arc::unwrap_or_clone(self.label.into_layout_job(
            ui.style(),
            egui::TextStyle::Body.into(),
            egui::Align::Center,
        ));
        job.wrap.max_width = (ui.available_width() - track_width - 8.0).max(0.0);
        for section in &mut job.sections {
            if section.format.line_height.is_none() {
                section.format.line_height = Some(20.0);
            }
        }
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let size = egui::vec2(
            track_width + 8.0 + galley.size().x,
            20.0_f32.max(galley.size().y),
        );
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
        let state_id = response.id.with("switch-interaction");
        let mut state = ui
            .data_mut(|data| data.get_temp::<Interaction>(state_id))
            .unwrap_or(Interaction {
                keyboard: true,
                space_armed: false,
                pointer_armed: false,
            });
        // Track modality even before focus arrives; pointer movement does not
        // erase keyboard focus-visible, while a pointer press does.
        let events = ui.input(|input| input.events.clone());
        for event in &events {
            match event {
                egui::Event::PointerButton { pressed: true, .. } => state.keyboard = false,
                egui::Event::Key { pressed: true, .. } => state.keyboard = true,
                _ => {}
            }
        }
        let mut activations = 0;
        if !ui.is_enabled() || !ui.input(|input| input.focused) {
            state.pointer_armed = false;
        } else {
            let eligible = response.hovered()
                || response.is_pointer_button_down_on()
                || response.clicked_by(egui::PointerButton::Primary);
            let from_global = ui
                .ctx()
                .layer_transform_from_global(response.layer_id)
                .unwrap_or_default();
            for event in &events {
                if let egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    ..
                } = event
                {
                    let inside = response.interact_rect.contains(from_global * *pos);
                    if *pressed {
                        state.pointer_armed = eligible && inside;
                        if state.pointer_armed {
                            response.request_focus();
                        }
                    } else {
                        if state.pointer_armed && eligible && inside {
                            activations += 1;
                        }
                        state.pointer_armed = false;
                    }
                }
            }
        }
        let focused = response.has_focus() && ui.is_enabled();
        if !focused {
            state.space_armed = false;
        }
        if focused {
            ui.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                );
                // egui cannot install a filter on the frame focus first arrives.
                // Cancel an already scheduled arrow move on the following frame.
                if events
                    .iter()
                    .rev()
                    .find_map(|event| match event {
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } if !modifiers.any()
                            && matches!(
                                key,
                                egui::Key::ArrowUp
                                    | egui::Key::ArrowDown
                                    | egui::Key::ArrowLeft
                                    | egui::Key::ArrowRight
                                    | egui::Key::Tab
                            ) =>
                        {
                            Some(*key)
                        }
                        _ => None,
                    })
                    .is_some_and(|key| key != egui::Key::Tab)
                {
                    memory.move_focus(egui::FocusDirection::None);
                }
            });
        }
        if focused {
            for event in events {
                match event {
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    } => activations += 1,
                    egui::Event::Key {
                        key: egui::Key::Space,
                        pressed,
                        ..
                    } => {
                        if pressed {
                            state.space_armed = true;
                        } else if state.space_armed {
                            activations += 1;
                            state.space_armed = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        if ui.is_enabled()
            && ui.input(|input| {
                input.has_accesskit_action_request(response.id, egui::accesskit::Action::Click)
            })
        {
            activations += 1;
        }
        if ui.is_enabled() && activations > 0 {
            if activations % 2 == 1 {
                *self.checked = !*self.checked;
            }
            response.mark_changed();
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::Switch,
                ui.is_enabled(),
                *self.checked,
                &label_text,
            )
        });
        if self.invalid {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_invalid(egui::accesskit::Invalid::True)
            });
        }
        let focus_visible = focused && state.keyboard;
        ui.data_mut(|data| data.insert_temp(state_id, state));

        let [progress] = animate(
            ui.ctx(),
            response.id.with("switch-thumb"),
            [f32::from(*self.checked)],
        );
        let [focus] = animate(
            ui.ctx(),
            response.id.with("switch-focus"),
            [f32::from(focus_visible)],
        );
        let [opacity] = animate(
            ui.ctx(),
            response.id.with("switch-opacity"),
            [if ui.is_enabled() { 1.0 } else { 0.5 }],
        );
        let colors = self.theme.colors;
        let dark = ui.visuals().dark_mode;
        let target_fill = if *self.checked {
            colors.primary.background
        } else if dark {
            colors.input.gamma_multiply(0.8)
        } else {
            colors.input
        };
        let fill = color(animate(
            ui.ctx(),
            response.id.with("switch-fill"),
            color_components(target_fill),
        ));

        if ui.is_rect_visible(rect) {
            let thumb = if dark {
                if *self.checked {
                    colors.primary.foreground
                } else {
                    colors.background.foreground
                }
            } else {
                colors.background.background
            };
            let track = egui::Rect::from_center_size(
                egui::pos2(rect.left() + track_width * 0.5, rect.center().y),
                egui::vec2(track_width, track_height),
            );
            let center = egui::pos2(
                track.left() + 1.0 + diameter * 0.5 + (diameter - 2.0) * progress,
                track.center().y,
            );
            // add_enabled applies the host's disabled opacity to the painter.
            // Replace that multiplier with the reference's 50%, preserving any
            // deliberate ancestor opacity. Label opacity is independent in HTML.
            let mut painter = ui.painter().clone();
            if !ui.is_enabled() {
                painter.set_opacity(painter.opacity() / ui.visuals().disabled_alpha());
            }
            painter.galley(
                egui::pos2(track.right() + 8.0, rect.center().y - galley.size().y * 0.5)
                    - galley.rect.min.to_vec2(),
                galley,
                colors.background.foreground,
            );
            painter.multiply_opacity(opacity);
            if !ui.is_enabled()
                && response.contains_pointer()
                && ui.input(|input| {
                    input
                        .pointer
                        .hover_pos()
                        .is_some_and(|pos| track.contains(pos))
                })
            {
                ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::NotAllowed);
            }
            painter.add(
                egui::epaint::Shadow {
                    offset: [0, 1],
                    blur: 2,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(13),
                }
                .as_shape(track, 255),
            );
            if focus > 0.0 {
                painter.rect_stroke(
                    track.expand(1.5 * focus),
                    255,
                    egui::Stroke::new(3.0 * focus, colors.ring.gamma_multiply(0.5 * focus)),
                    egui::StrokeKind::Middle,
                );
            }
            painter.rect_filled(track, 255, fill);
            painter.circle_filled(center, diameter * 0.5, thumb);
            if focus > 0.0 {
                painter.rect_stroke(
                    track,
                    255,
                    egui::Stroke::new(1.0, colors.ring.gamma_multiply(focus)),
                    egui::StrokeKind::Inside,
                );
            }
        }
        ui.set_style(previous_style);
        response
    }
}
