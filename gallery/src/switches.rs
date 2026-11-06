use egui_prism::{
    egui,
    switch::{Switch, SwitchSize},
    theme::Theme,
};

pub(super) struct SwitchCase {
    values: [bool; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
    small: bool,
    invalid: bool,
    compositing: bool,
}

impl Default for SwitchCase {
    fn default() -> Self {
        Self {
            values: [false, true],
            changes: [0; 2],
            disabled: false,
            reversed: false,
            long: false,
            small: false,
            invalid: false,
            compositing: false,
        }
    }
}

impl SwitchCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism switches");
        ui.label("Click the track or its label, or focus with Tab and toggle with Space or Enter. Space activates on release; Enter activates on press. Pointer focus has no ring.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable switches");
            ui.checkbox(&mut self.reversed, "Reverse switch instances");
            ui.checkbox(&mut self.long, "Long labels");
            ui.checkbox(&mut self.small, "Small switches");
            ui.checkbox(&mut self.invalid, "Invalid switches");
            ui.checkbox(&mut self.compositing, "Compositing samples");
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
                let response = ui.add_enabled(!self.disabled, Switch::new(theme, &mut self.values[index], label).size(if self.small { SwitchSize::Small } else { SwitchSize::Default }).invalid(self.invalid));
                if response.changed() { self.changes[index] += 1; }
                ui.label(format!("{name}: {} · Changes: {}", self.values[index], self.changes[index]));
                if response.has_focus() { ui.small("This switch has keyboard focus."); }
            });
        }
        if self.compositing {
            for background in [
                egui::Color32::WHITE,
                egui::Color32::from_gray(128),
                egui::Color32::from_rgb(52, 120, 246),
            ] {
                egui::Frame::new()
                    .fill(background)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (mut value, label) in [(false, "Off"), (true, "On")] {
                                ui.add_enabled(
                                    false,
                                    Switch::new(theme, &mut value, label).size(if self.small {
                                        SwitchSize::Small
                                    } else {
                                        SwitchSize::Default
                                    }),
                                );
                            }
                        });
                    });
            }
        }
        ui.small("Default: 32 × 18.4 / 16pt thumb. Small: 24 × 14 / 12pt thumb. Invalid has no extra visual treatment in the pinned source.");
        ui.small("Values and counters belong to this example. The component does not start asynchronous work or persist values.");
    }
}
