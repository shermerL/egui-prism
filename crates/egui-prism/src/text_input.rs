//! Labeled single-line editing with caller-owned values and validation.

use crate::{egui, theme::Theme};

/// A labeled single-line editor borrowing the caller's text and theme for a frame.
///
/// The caller owns validation and consumes the returned edit response. Reorderable
/// instances need stable UI scopes or an explicit unique [`Self::id`].
///
/// ```
/// use egui_prism::{egui, text_input::TextInput, theme::Theme};
/// let theme = Theme::light();
/// let mut name = String::new();
/// egui::__run_test_ui(|ui| {
///     let response = ui.add(TextInput::new(&theme, &mut name, "Name").hint("Your name"));
///     if response.changed() { /* validate the caller-owned value */ }
/// });
/// ```
#[must_use = "Add the input to a Ui with ui.add(input)"]
pub struct TextInput<'theme, 'value> {
    theme: &'theme Theme,
    value: &'value mut String,
    label: String,
    hint: String,
    error: Option<String>,
    id: Option<egui::Id>,
}

impl<'theme, 'value> TextInput<'theme, 'value> {
    /// Create a labeled editor; a missing visible label is a programmer error.
    pub fn new(theme: &'theme Theme, value: &'value mut String, label: impl Into<String>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextInput requires a nonempty label"
        );
        Self {
            theme,
            value,
            label,
            hint: String::new(),
            error: None,
            id: None,
        }
    }

    /// Show a hint while the caller's text is empty.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }

    /// Display the caller's validation message without changing or validating the value.
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Assign an application-unique ID to retain selection and undo across layout moves.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }
}

impl egui::Widget for TextInput<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("text-input"));
        ui.vertical(|ui| {
            self.theme.apply(ui.style_mut());
            if self.error.is_some() {
                // Keep selection's paired foreground and focus outline intact.
                let error_color = self.theme.colors.destructive.background;
                ui.visuals_mut().widgets.inactive.bg_stroke.color = error_color;
                ui.visuals_mut().widgets.hovered.bg_stroke.color = error_color;
            }
            let label = ui.label(self.label);
            let response = ui
                .add(
                    egui::TextEdit::singleline(self.value)
                        .interactive(ui.is_enabled())
                        .id(id)
                        .hint_text(self.hint)
                        .desired_width(ui.available_width()),
                )
                .labelled_by(label.id);
            // Selection foreground and focus outlines are separate shadcn tokens.
            // egui's default editor uses selection.stroke for both.
            if response.has_focus() && ui.is_enabled() {
                let color = if self.error.is_some() {
                    self.theme.colors.destructive.background
                } else {
                    self.theme.colors.ring
                };
                ui.painter().rect_stroke(
                    response.rect,
                    self.theme.metrics.corner_radius,
                    egui::Stroke::new(3.0, color.gamma_multiply(0.5)),
                    egui::StrokeKind::Outside,
                );
                ui.painter().rect_stroke(
                    response.rect,
                    self.theme.metrics.corner_radius,
                    egui::Stroke::new(1.0, color),
                    egui::StrokeKind::Inside,
                );
            }
            if let Some(error) = self.error {
                ui.label(
                    egui::RichText::new(&error)
                        .small()
                        .color(self.theme.colors.destructive.background),
                );
                ui.ctx().accesskit_node_builder(response.id, |node| {
                    node.set_description(error);
                    node.set_invalid(egui::accesskit::Invalid::True);
                });
            }
            response
        })
        .inner
    }
}
