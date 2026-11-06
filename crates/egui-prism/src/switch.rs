//! Controlled boolean switches without application-side effects.

use crate::{egui, theme::Theme};

/// A labeled switch borrowing the caller's boolean and theme for one frame.
///
/// The track and label share a hit target. Consume `Response::changed()` in the
/// caller; the widget does not start work or infer that an operation succeeded.
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
        }
    }
}

impl egui::Widget for Switch<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());
        let metrics = self.theme.metrics;
        let track_height = metrics.control_height * 0.625;
        let track_width = track_height * 1.8;
        let gap = metrics.item_spacing.x;
        let label_text = self.label.text().to_owned();
        let galley = self.label.into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            (ui.available_width() - track_width - gap).max(0.0),
            egui::TextStyle::Body,
        );
        let size = egui::vec2(
            track_width + gap + galley.size().x,
            metrics.control_height.max(galley.size().y),
        );
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
        if response.clicked() {
            *self.checked = !*self.checked;
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
        if ui.is_rect_visible(rect) {
            let colors = self.theme.colors;
            let pair = if *self.checked {
                colors.primary
            } else {
                colors.muted
            };
            let track = egui::Rect::from_center_size(
                egui::pos2(rect.left() + track_width * 0.5, rect.center().y),
                egui::vec2(track_width, track_height),
            );
            let stroke = if response.has_focus() || response.is_pointer_button_down_on() {
                egui::Stroke::new(2.0, colors.focus.foreground)
            } else if response.hovered() {
                egui::Stroke::new(1.0, colors.surface.foreground)
            } else {
                egui::Stroke::new(1.0, colors.border)
            };
            ui.painter().rect(
                track,
                track_height * 0.5,
                pair.background,
                stroke,
                egui::StrokeKind::Inside,
            );
            let radius = track_height * 0.4;
            let inset = track_height * 0.5;
            let center = egui::pos2(
                if *self.checked {
                    track.right() - inset
                } else {
                    track.left() + inset
                },
                track.center().y,
            );
            ui.painter().circle_filled(center, radius, pair.foreground);
            ui.painter().galley(
                egui::pos2(track.right() + gap, rect.center().y - galley.size().y * 0.5)
                    - galley.rect.min.to_vec2(),
                galley,
                colors.surface.foreground,
            );
        }
        ui.set_style(previous_style);
        response
    }
}
