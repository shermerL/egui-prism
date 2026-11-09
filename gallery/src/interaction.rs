//! Shared interaction cases exercised by the gallery and synthetic-input tests.

use crate::input::{self, Draw, Layout};
use egui_prism::egui;

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Reversed,
    Text(usize),
    Check(usize),
    Action(usize),
    Count(usize),
    Focus([egui::Id; 2]),
}

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
    pub(super) fn control(
        &mut self,
        ui: &mut egui::Ui,
        control: Control,
        long_labels: bool,
    ) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable actions").into(),
            Control::Reversed => ui.checkbox(&mut self.reversed, "Reverse instances").into(),
            Control::Text(index) => {
                let label = ui.label("Caller-owned text");
                let response = ui
                    .add(
                        egui::TextEdit::singleline(&mut self.instances[index].text)
                            .id_salt("text")
                            .hint_text("Type here, then switch case or theme")
                            .desired_width(ui.available_width()),
                    )
                    .labelled_by(label.id);
                Draw {
                    response,
                    ids: vec![label.id],
                    button: None,
                }
            }
            Control::Check(index) => ui
                .checkbox(&mut self.instances[index].checked, "Caller-owned checkbox")
                .into(),
            Control::Action(index) => {
                let text = if long_labels {
                    "Activate this instance with a deliberately long label that must wrap inside the available width"
                } else {
                    "Activate"
                };
                // This is intentionally still an ordinary stock egui button.
                let response = ui.add_enabled(!self.disabled, egui::Button::new(text).wrap());
                if response.clicked() {
                    self.instances[index].activations += 1;
                }
                response.into()
            }
            Control::Count(index) => ui
                .add(
                    egui::Label::new(format!(
                        "Activations: {}",
                        self.instances[index].activations
                    ))
                    .wrap_mode(egui::TextWrapMode::Extend),
                )
                .into(),
            Control::Focus(ids) => {
                let focused = ui.memory(|memory| ids.into_iter().any(|id| memory.has_focus(id)));
                ui.small(if focused {
                    "An action has keyboard focus."
                } else {
                    " "
                })
                .into()
            }
        }
    }

    pub(super) fn show_ordered(
        &mut self,
        ui: &mut egui::Ui,
        long_labels: bool,
        frame: &mut Layout,
    ) {
        let mut ids = [egui::Id::NULL; 2];
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("instance", index), |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    frame.track(
                        ui.strong(format!("Instance {}", if index == 0 { "A" } else { "B" })),
                    );
                    for control in [
                        Control::Text(index),
                        Control::Check(index),
                        Control::Action(index),
                        Control::Count(index),
                    ] {
                        let response =
                            frame.widget(ui, input::Control::Interaction(control), |ui| {
                                self.control(ui, control, long_labels)
                            });
                        if matches!(control, Control::Action(_)) {
                            ids[index] = response.id;
                        }
                    }
                });
            });
        }
        frame.widget(ui, input::Control::Interaction(Control::Focus(ids)), |ui| {
            self.control(ui, Control::Focus(ids), long_labels)
        });
    }

    #[cfg(test)]
    fn show(&mut self, ui: &mut egui::Ui, long_labels: bool) -> [egui::Response; 2] {
        let mut frame = Layout::new(input::Phase::Paint);
        self.show_ordered(ui, long_labels, &mut frame);
        (if self.reversed { [1, 0] } else { [0, 1] }).map(|index| {
            frame
                .entries
                .iter()
                .find(|entry| {
                    matches!(entry.control,
                        input::Control::Interaction(Control::Action(i)) if i == index
                    )
                })
                .expect("registered action")
                .response
                .clone()
        })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ordered_tests;
