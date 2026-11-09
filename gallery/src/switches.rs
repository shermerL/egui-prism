use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{
    egui,
    switch::{Switch, SwitchRegistration, SwitchSize},
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Reversed,
    Long,
    Small,
    Invalid,
    Compositing,
    Instance(usize),
    Count(usize),
    Sample { background: usize, checked: bool },
}

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
    pub(super) fn control(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        control: Control,
        phase: Phase,
    ) -> Draw {
        let size = if self.small {
            SwitchSize::Small
        } else {
            SwitchSize::Default
        };
        let draw = |ui: &mut egui::Ui, switch: Switch<'_, '_>| match phase {
            Phase::Register => switch.register(ui).into(),
            Phase::Paint => switch.show(ui).into(),
        };
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable switches").into(),
            Control::Reversed => ui
                .checkbox(&mut self.reversed, "Reverse switch instances")
                .into(),
            Control::Long => ui.checkbox(&mut self.long, "Long labels").into(),
            Control::Small => ui.checkbox(&mut self.small, "Small switches").into(),
            Control::Invalid => ui.checkbox(&mut self.invalid, "Invalid switches").into(),
            Control::Compositing => ui
                .checkbox(&mut self.compositing, "Compositing samples")
                .into(),
            Control::Instance(index) => {
                let name = ['A', 'B'][index];
                let label = if self.long {
                    format!(
                        "Enable option {name} with a deliberately long label that wraps inside narrow content and remains part of the same clickable control"
                    )
                } else {
                    format!("Enable option {name}")
                };
                if self.disabled {
                    ui.disable();
                }
                draw(
                    ui,
                    Switch::new(theme, &self.values[index], label)
                        .id(egui::Id::unique(("gallery-switch", index)))
                        .size(size)
                        .invalid(self.invalid),
                )
            }
            Control::Count(index) => ui
                .label(format!(
                    "{}: {} · Changes: {}",
                    ['A', 'B'][index],
                    self.values[index],
                    self.changes[index]
                ))
                .into(),
            Control::Sample {
                background,
                checked,
            } => {
                ui.disable();
                let label = if checked { "On" } else { "Off" };
                let id = egui::Id::unique(("gallery-switch-sample", background, checked));
                draw(ui, Switch::new(theme, &checked, label).id(id).size(size))
            }
        }
    }

    pub(super) fn handle(
        &mut self,
        control: Control,
        registration: &SwitchRegistration,
        event: &egui::RoutedInput,
    ) -> egui::Response {
        let Control::Instance(index) = control else {
            unreachable!("only enabled switch instances receive input");
        };
        let response = registration.handle(event, &mut self.values[index]);
        if response.changed() {
            self.changes[index] += 1;
        }
        response
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Prism switches"));
        frame.label(
            ui,
            "Click the track or its label, or focus with Tab and toggle with Space or Enter. Space activates on release. Enter activates on press.",
        );
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Disabled,
                Control::Reversed,
                Control::Long,
                Control::Small,
                Control::Invalid,
                Control::Compositing,
            ] {
                self.widget(ui, theme, frame, control);
            }
        });
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("switch-instance", index), |ui| {
                self.widget(ui, theme, frame, Control::Instance(index));
                self.widget(ui, theme, frame, Control::Count(index));
            });
        }
        if self.compositing {
            for (index, background) in [
                egui::Color32::WHITE,
                egui::Color32::from_gray(128),
                egui::Color32::from_rgb(52, 120, 246),
            ]
            .into_iter()
            .enumerate()
            {
                egui::Frame::new()
                    .fill(background)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for checked in [false, true] {
                                self.widget(
                                    ui,
                                    theme,
                                    frame,
                                    Control::Sample {
                                        background: index,
                                        checked,
                                    },
                                );
                            }
                        });
                    });
            }
        }
        frame.label(
            ui,
            egui::RichText::new(
                "Default: 32 × 18.4 / 16pt thumb. Small: 24 × 14 / 12pt thumb. Invalid has no extra visual treatment in the pinned source.",
            )
            .small(),
        );
        frame.label(
            ui,
            egui::RichText::new(
                "Values and counters belong to this example. The component does not start asynchronous work or persist values.",
            )
            .small(),
        );
    }

    fn widget(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout, control: Control) {
        let phase = frame.phase;
        frame.widget(ui, input::Control::Switch(control), |ui| {
            self.control(ui, theme, control, phase)
        });
    }
}
