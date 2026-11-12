//! Caller-owned Card content and actions through the existing host input path.
use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{button::Button, card::Card, egui, theme::Theme};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Long,
    Text,
    Save,
    Count,
}

#[derive(Default)]
pub(super) struct CardCase {
    disabled: bool,
    long: bool,
    text: String,
    count: u64,
}

impl CardCase {
    pub fn activated(&mut self, control: Control) {
        if matches!(control, Control::Save) {
            self.count += 1;
        }
    }

    pub fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::Disabled => ui
                .checkbox(&mut self.disabled, "Disable nested action")
                .into(),
            Control::Long => ui.checkbox(&mut self.long, "Long header and body").into(),
            Control::Text => ui.text_edit_singleline(&mut self.text).into(),
            Control::Save => {
                if self.disabled {
                    ui.disable();
                }
                Button::new(theme, "Save").register(ui).into()
            }
            Control::Count => ui.label(format!("Saved: {}", self.count)).into(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Cards"));
        frame.label(
            ui,
            "Caller-owned sections, text and actions. Standard egui text editor.",
        );
        ui.horizontal(|ui| {
            for control in [Control::Disabled, Control::Long] {
                frame.widget(ui, input::Control::Card(control), |ui| {
                    self.control(ui, theme, control)
                });
            }
        });
        let painter = ui.painter().clone();
        let output = Card::new(theme).show(ui, |card| {
            card.header(
                if self.long {
                    "A deliberately long card title wraps within the caller's narrow layout"
                } else {
                    "Project"
                },
                Some("Caller-owned content and actions."),
            );
            card.content(|body| {
                if self.long {
                    for i in 1..=8 {
                        frame.label(body.ui(), format!("Caller-owned content row {i}"));
                    }
                }
                frame.widget(body.ui(), input::Control::Card(Control::Text), |ui| {
                    self.control(ui, theme, Control::Text)
                });
            });
            card.footer(36.0, |footer| {
                frame.widget(footer.ui(), input::Control::Card(Control::Save), |ui| {
                    self.control(ui, theme, Control::Save)
                });
                footer.ui().add_space(12.0);
                frame.widget(footer.ui(), input::Control::Card(Control::Count), |ui| {
                    self.control(ui, theme, Control::Count)
                });
            });
        });
        frame.track(match frame.phase {
            Phase::Register => output.response,
            Phase::Paint => output.paint(&painter).expect("card compositor").response,
        });
        frame.label(ui, "Empty Card");
        let empty = Card::new(theme).show(ui, |_| {});
        frame.track(match frame.phase {
            Phase::Register => empty.response,
            Phase::Paint => empty.paint(ui.painter()).expect("card compositor").response,
        });
        frame.label(ui, "60% isolated shell and header over a tinted surface");
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(52, 120, 246))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.multiply_opacity(0.6);
                let output = Card::new(theme).show(ui, |card| {
                    card.header("Preview", Some("Two-layer shadow-sm."));
                });
                frame.track(match frame.phase {
                    Phase::Register => output.response,
                    Phase::Paint => {
                        output
                            .paint(ui.painter())
                            .expect("card compositor")
                            .response
                    }
                });
            });
    }
}
