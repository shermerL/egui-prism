//! Shared interaction cases exercised by the gallery and synthetic-input tests.

use egui_prism::egui;

#[derive(Default)]
struct Instance {
    text: String,
    checked: bool,
    activations: u64,
}

#[derive(Default)]
pub(super) struct InteractionCase {
    pub(super) disabled: bool,
    pub(super) reversed: bool,
    instances: [Instance; 2],
}

impl InteractionCase {
    pub(super) fn show(&mut self, ui: &mut egui::Ui, long_labels: bool) -> [egui::Response; 2] {
        let order = if self.reversed { [1, 0] } else { [0, 1] };
        order.map(|index| {
            // Explicit scopes keep automatic child IDs stable when instances move.
            let id = ui.make_persistent_id(("instance", index));
            ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    let instance = &mut self.instances[index];
                    ui.strong(format!("Instance {}", if index == 0 { "A" } else { "B" }));
                    let label = ui.label("Caller-owned text");
                    ui.add(egui::TextEdit::singleline(&mut instance.text)
                        .id_salt("text")
                        .hint_text("Type here, then switch case or theme")
                        .desired_width(ui.available_width()))
                        .labelled_by(label.id);
                    ui.checkbox(&mut instance.checked, "Caller-owned checkbox");
                    let text = if long_labels {
                        "Activate this instance with a deliberately long label that must wrap inside the available width"
                    } else {
                        "Activate"
                    };
                    let response = ui.add_enabled(!self.disabled, egui::Button::new(text).wrap());
                    if response.clicked() {
                        instance.activations += 1;
                    }
                    ui.label(format!("Activations: {}", instance.activations));
                    response
                }).inner
            }).inner
        })
    }
}

#[cfg(test)]
mod tests;
