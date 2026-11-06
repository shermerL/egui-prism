use egui_prism::{egui, slider::Slider, theme::Theme};

pub(super) struct SliderCase {
    values: [f64; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
    width: f32,
    tinted: bool,
}

impl Default for SliderCase {
    fn default() -> Self {
        Self {
            values: [50.0, 0.0],
            changes: [0; 2],
            disabled: false,
            reversed: false,
            long: false,
            width: 448.0,
            tinted: false,
        }
    }
}

impl SliderCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism sliders");
        ui.label("Click or drag the rail. Use Tab, arrows, Home/End, and Shift+arrows or PageUp/Down (ten steps). A uses 0 to 100 in steps of 5; B uses -1 to 1 in steps of 0.1.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable sliders");
            ui.checkbox(&mut self.reversed, "Reverse slider instances");
            ui.checkbox(&mut self.long, "Long labels");
            ui.checkbox(&mut self.tinted, "Tinted surface");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Rail width:");
            for width in [448.0, 200.0, 20.0, 16.0] {
                ui.selectable_value(&mut self.width, width, format!("{width}pt"));
            }
        });
        let surface = if self.tinted {
            egui::Color32::from_rgb(25, 51, 65)
        } else {
            theme.colors.background.background
        };
        egui::Frame::NONE.fill(surface).inner_margin(16.0).show(ui, |ui| {
        ui.spacing_mut().slider_width = self.width;
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
        ui.add_space(16.0);
        let mut disabled_amount = 35.0;
        ui.add_enabled(
            false,
            Slider::new(theme, &mut disabled_amount, "Disabled amount").range(0.0..=100.0),
        );
        });
        ui.small("Values and change counters belong to this example. Sliders do not implement application side effects.");
    }
}
