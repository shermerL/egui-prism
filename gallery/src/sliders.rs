use egui_prism::{egui, slider::Slider, theme::Theme};

pub(super) struct SliderCase {
    values: [f64; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
}

impl Default for SliderCase {
    fn default() -> Self {
        Self {
            values: [50.0, 0.0],
            changes: [0; 2],
            disabled: false,
            reversed: false,
            long: false,
        }
    }
}

impl SliderCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism sliders");
        ui.label("Click or drag the rail. Use Tab and arrow keys. A uses 0 to 100 in steps of 5; B uses -1 to 1 in steps of 0.1.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable sliders");
            ui.checkbox(&mut self.reversed, "Reverse slider instances");
            ui.checkbox(&mut self.long, "Long labels");
        });
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            let id = ui.make_persistent_id(("slider-instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                let name = if index == 0 { "A" } else { "B" };
                let label = if self.long { format!("Value {name} with a deliberately long label that wraps above the rail inside narrow content") } else { format!("Value {name}") };
                let (range, step) = if index == 0 { (0.0..=100.0, 5.0) } else { (-1.0..=1.0, 0.1) };
                let response = ui.add_enabled(!self.disabled, Slider::new(theme, &mut self.values[index], label).range(range).step_by(step));
                if response.changed() { self.changes[index] += 1; }
                ui.label(format!("{name}: {:.1} · Changes: {}", self.values[index], self.changes[index]));
                if response.has_focus() { ui.small("This slider has keyboard focus."); }
            });
        }
        ui.small("Values and change counters belong to this example. Sliders do not implement application side effects.");
    }
}
