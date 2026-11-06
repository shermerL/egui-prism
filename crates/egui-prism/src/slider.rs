//! Labeled single-value sliders using egui interaction and geometry.

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
    step: Option<f64>,
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
            step: None,
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

    /// Round edits to positive finite step multiples, using egui's range-start grid.
    pub fn step_by(mut self, step: f64) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "Slider requires a positive finite step"
        );
        self.step = Some(step);
        self
    }
}

impl egui::Widget for Slider<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());
        let response = ui
            .vertical(|ui| {
                ui.spacing_mut().slider_width = ui.spacing().slider_width.min(ui.available_width());
                let enabled = ui.is_enabled();
                let (min, max) = (*self.range.start(), *self.range.end());
                let label = ui.add(egui::Label::new(self.label).wrap());
                let mut slider = egui::Slider::from_get_set(self.range, |next| {
                    // Accessibility requests use the same write boundary as pointer/keyboard edits.
                    if enabled
                        && let Some(next) = next
                        && next.is_finite()
                    {
                        // egui rounds after clamping, so a non-dividing step may overshoot.
                        *self.value = next.clamp(min, max);
                    }
                    *self.value
                })
                .clamping(egui::SliderClamping::Edits)
                .show_value(false)
                .trailing_fill(true);
                if let Some(step) = self.step {
                    slider = slider.step_by(step);
                }
                ui.add(slider).labelled_by(label.id)
            })
            .inner;
        ui.set_style(previous_style);
        response
    }
}
