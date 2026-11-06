//! Labeled boolean editing with native egui interaction and caller-owned state.

use crate::{egui, theme::Theme};

/// A themed checkbox that borrows the caller's boolean for one frame.
///
/// Both the box and label activate the same control. Use `ui.add_enabled(false,
/// checkbox)` to disable it and stable caller scopes for reorderable instances.
///
/// ```
/// use egui_prism::{checkbox::Checkbox, egui, theme::Theme};
/// let theme = Theme::light();
/// let mut enabled = false;
/// egui::__run_test_ui(|ui| {
///     if ui.add(Checkbox::new(&theme, &mut enabled, "Enable previews")).changed() {
///         // The application decides what the new value means.
///     }
/// });
/// ```
#[must_use = "Add the checkbox to a Ui with ui.add(checkbox)"]
pub struct Checkbox<'theme, 'value> {
    theme: &'theme Theme,
    checked: &'value mut bool,
    label: egui::WidgetText,
}

impl<'theme, 'value> Checkbox<'theme, 'value> {
    /// Create a checkbox with a visible label; an empty label is a programmer error.
    pub fn new(
        theme: &'theme Theme,
        checked: &'value mut bool,
        label: impl Into<egui::WidgetText>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.text().trim().is_empty(),
            "Checkbox requires a nonempty label"
        );
        Self {
            theme,
            checked,
            label,
        }
    }
}

impl egui::Widget for Checkbox<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Keep the caller's layout and automatic ID sequence without leaking style.
        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());
        let response = ui.add(egui::Checkbox::new(self.checked, self.label));
        ui.set_style(previous_style);
        response
    }
}
