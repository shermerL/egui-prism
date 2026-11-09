use crate::input::{self, Draw, Layout};
use egui_prism::{checkbox::Checkbox, egui, theme::Theme};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Reversed,
    Long,
    Value(usize),
    Count(usize),
    Focus(egui::Id),
}

#[derive(Default)]
pub(super) struct CheckboxCase {
    values: [bool; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
}

impl CheckboxCase {
    pub(super) fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable checkboxes").into(),
            Control::Reversed => ui
                .checkbox(&mut self.reversed, "Reverse checkbox instances")
                .into(),
            Control::Long => ui.checkbox(&mut self.long, "Long labels").into(),
            Control::Value(index) => {
                let name = if index == 0 { "A" } else { "B" };
                let label = if self.long {
                    format!(
                        "Enable option {name} with a deliberately long label that wraps inside narrow content and remains part of the same clickable control"
                    )
                } else {
                    format!("Enable option {name}")
                };
                let response = ui.add_enabled(
                    !self.disabled,
                    Checkbox::new(theme, &mut self.values[index], label),
                );
                if response.changed() {
                    self.changes[index] += 1;
                }
                response.into()
            }
            Control::Count(index) => ui
                .add(
                    egui::Label::new(format!(
                        "{}: {} · Changes: {}",
                        if index == 0 { "A" } else { "B" },
                        self.values[index],
                        self.changes[index]
                    ))
                    .wrap_mode(egui::TextWrapMode::Extend),
                )
                .into(),
            Control::Focus(id) => {
                let focused = ui.memory(|memory| memory.has_focus(id));
                ui.small(if focused {
                    "This checkbox has keyboard focus."
                } else {
                    " "
                })
                .into()
            }
        }
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Prism checkboxes"));
        frame.label(ui, "Click the box or its label, or focus with Tab and toggle with Space or Enter. Each activation changes one caller-owned value once.");
        ui.horizontal_wrapped(|ui| {
            for control in [Control::Disabled, Control::Reversed, Control::Long] {
                self.widget(ui, theme, frame, control);
            }
        });
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("checkbox-instance", index), |ui| {
                let id = self.widget(ui, theme, frame, Control::Value(index)).id;
                self.widget(ui, theme, frame, Control::Count(index));
                // Reserve the line so focus alone does not move later controls.
                self.widget(ui, theme, frame, Control::Focus(id));
            });
        }
        frame.track(ui.small("Values and counters belong to this example. The component does not start asynchronous work or persist values."));
    }

    fn widget(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        frame: &mut Layout,
        c: Control,
    ) -> egui::Response {
        frame.widget(ui, input::Control::Checkbox(c), |ui| {
            self.control(ui, theme, c)
        })
    }
}
