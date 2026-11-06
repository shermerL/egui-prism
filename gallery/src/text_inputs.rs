use egui_prism::{egui, text_input::TextInput, theme::Theme};

#[derive(Default)]
pub(super) struct InputCase {
    values: [String; 2],
    edits: [u64; 2],
    disabled: bool,
    reversed: bool,
    show_error: bool,
}

impl InputCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Prism text inputs");
        ui.label("Type, select, paste, undo, or compose with the host input method. The caller owns both values and validation.");
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.disabled, "Disable editing");
            ui.checkbox(&mut self.show_error, "Show caller error");
            ui.checkbox(&mut self.reversed, "Reverse input instances");
        });
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            let id = ui.make_persistent_id(("text-input-instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                let mut input = TextInput::new(theme, &mut self.values[index], format!("Value {}", if index == 0 { "A" } else { "B" }))
                    .hint("Type or paste here");
                if self.show_error {
                    input = input.error("Caller-provided error: this description may wrap, and does not modify the value.");
                }
                if ui.add_enabled(!self.disabled, input).changed() {
                    self.edits[index] += 1;
                }
                ui.small(format!("Edit frames: {} · Characters: {}", self.edits[index], self.values[index].chars().count()));
            });
        }
        ui.collapsing("Inspect Unicode values", |ui| {
            ui.label(
                "Escaped values help verify composition even when the host font lacks a glyph.",
            );
            for (index, value) in self.values.iter().enumerate() {
                ui.monospace(format!(
                    "{}: {}",
                    if index == 0 { "A" } else { "B" },
                    value.escape_unicode()
                ));
            }
        });
        ui.small("IME integration and glyph coverage depend on the host platform and fonts; synthetic events do not establish OS input-method acceptance.");
    }
}
