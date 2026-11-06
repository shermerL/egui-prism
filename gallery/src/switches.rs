use egui_prism::{egui, switch::Switch, theme::Theme};

#[derive(Default)]
pub(super) struct SwitchCase {
    values: [bool; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
}

impl SwitchCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism switches");
        ui.label("Click the track or its label, or focus with Tab and toggle with Space or Enter. Each activation changes one caller-owned value once.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable switches");
            ui.checkbox(&mut self.reversed, "Reverse switch instances");
            ui.checkbox(&mut self.long, "Long labels");
        });
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            let id = ui.make_persistent_id(("switch-instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                let name = if index == 0 { "A" } else { "B" };
                let label = if self.long {
                    format!("Enable option {name} with a deliberately long label that wraps inside narrow content and remains part of the same clickable control")
                } else {
                    format!("Enable option {name}")
                };
                let response = ui.add_enabled(!self.disabled, Switch::new(theme, &mut self.values[index], label));
                if response.changed() { self.changes[index] += 1; }
                ui.label(format!("{name}: {} · Changes: {}", self.values[index], self.changes[index]));
                if response.has_focus() { ui.small("This switch has keyboard focus."); }
            });
        }
        ui.small("Values and counters belong to this example. The component does not start asynchronous work or persist values.");
    }
}
