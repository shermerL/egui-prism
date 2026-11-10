use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{
    egui,
    text_input::{TextInput, TextInputOutput},
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Error,
    Reversed,
    Editor(usize),
    Count(usize),
    Inspector,
}

#[derive(Default)]
pub(super) struct InputCase {
    values: [String; 2],
    edits: [u64; 2],
    last_edit_frame: [Option<u64>; 2],
    disabled: bool,
    reversed: bool,
    show_error: bool,
}

impl InputCase {
    pub(super) fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable editing").into(),
            Control::Error => ui
                .checkbox(&mut self.show_error, "Show caller error")
                .into(),
            Control::Reversed => ui
                .checkbox(&mut self.reversed, "Reverse input instances")
                .into(),
            Control::Editor(index) => self.editor(ui, theme, index).into(),
            Control::Count(index) => ui
                .add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "Edit frames: {} · Characters: {}",
                            self.edits[index],
                            self.values[index].chars().count()
                        ))
                        .small(),
                    )
                    .wrap_mode(egui::TextWrapMode::Extend),
                )
                .into(),
            Control::Inspector => {
                let mut ids = Vec::new();
                let output = ui.collapsing("Inspect Unicode values", |ui| {
                    ids.push(
                        ui.label("Escaped values help verify composition even when the host font lacks a glyph.")
                            .id,
                    );
                    for (index, value) in self.values.iter().enumerate() {
                        ids.push(
                            ui.monospace(format!(
                                "{}: {}",
                                if index == 0 { "A" } else { "B" },
                                value.escape_unicode()
                            ))
                            .id,
                        );
                    }
                });
                if let Some(body) = output.body_response {
                    ids.push(body.id);
                }
                Draw {
                    response: output.header_response,
                    ids,
                    button: None,
                }
            }
        }
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Prism text inputs"));
        frame.label(ui, "Type, select, paste, undo, or compose with the host input method. The caller owns both values and validation.");
        for control in [Control::Disabled, Control::Error, Control::Reversed] {
            self.widget(ui, theme, frame, control);
        }
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("text-input-instance", index), |ui| {
                self.widget(ui, theme, frame, Control::Editor(index));
                self.widget(ui, theme, frame, Control::Count(index));
            });
        }
        self.widget(ui, theme, frame, Control::Inspector);
        frame.track(ui.small("IME integration and glyph coverage depend on the host platform and fonts; synthetic events do not establish OS input-method acceptance."));
    }

    fn editor(&mut self, ui: &mut egui::Ui, theme: &Theme, index: usize) -> TextInputOutput {
        if self.disabled {
            ui.disable();
        }
        let mut input = TextInput::new(
            theme,
            &mut self.values[index],
            format!("Value {}", if index == 0 { "A" } else { "B" }),
        )
        .hint("Type or paste here");
        if self.show_error {
            input = input.error(
                "Caller-provided error: this description may wrap, and does not modify the value.",
            );
        }
        let output = input.show(ui);
        let frame = ui.ctx().cumulative_frame_nr();
        if output.response.changed() && self.last_edit_frame[index] != Some(frame) {
            self.edits[index] += 1;
            self.last_edit_frame[index] = Some(frame);
        }
        output
    }

    fn widget(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout, c: Control) {
        let phase = frame.phase;
        frame.widget(ui, input::Control::Input(c), |ui| {
            if let Control::Editor(index) = c {
                let painter = ui.painter().clone();
                let mut output = self.editor(ui, theme, index);
                if phase == Phase::Paint
                    && let Some(drawing) = output.drawing.take()
                {
                    egui_prism::compositing::paint(&painter, drawing)
                        .expect("text input compositor");
                }
                output.into()
            } else {
                self.control(ui, theme, c)
            }
        });
    }
}

impl From<TextInputOutput> for Draw {
    fn from(output: TextInputOutput) -> Self {
        let mut ids = vec![output.label.id];
        if let Some(error) = output.error {
            ids.push(error.id);
        }
        Self {
            response: output.response,
            ids,
            button: None,
        }
    }
}
