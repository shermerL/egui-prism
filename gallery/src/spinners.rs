//! Caller-controlled animation and visibility, not a background task manager.
use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{button::Button, egui, spinner::Spinner, theme::Theme};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Hidden,
    Reduced,
    DisabledButton,
}
#[derive(Default)]
pub(super) struct SpinnerCase {
    hidden: bool,
    reduced: bool,
}
impl SpinnerCase {
    pub fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::Hidden => ui.checkbox(&mut self.hidden, "Hide indicators").into(),
            Control::Reduced => ui
                .checkbox(&mut self.reduced, "Reduced motion (caller policy)")
                .into(),
            Control::DisabledButton => {
                ui.disable();
                Button::new(theme, "Saving").register(ui).into()
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Spinners"));
        frame.label(ui,"16pt status, one-second linear rotation. Caller owns busy state and accessibility text.");
        for control in [Control::Hidden, Control::Reduced] {
            frame.widget(ui, input::Control::Spinner(control), |ui| {
                self.control(ui, theme, control)
            });
        }
        if !self.hidden {
            ui.horizontal(|ui| {
                draw(
                    Spinner::new(theme, "inline", "Loading projects").animate(!self.reduced),
                    ui,
                    frame,
                );
                frame.label(ui, "Loading projects");
            });
            ui.horizontal(|ui| {
                frame.widget(ui, input::Control::Spinner(Control::DisabledButton), |ui| {
                    self.control(ui, theme, Control::DisabledButton)
                });
                draw(
                    Spinner::new(theme, "adjacent", "Saving").animate(!self.reduced),
                    ui,
                    frame,
                );
            });
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(52, 120, 246))
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.multiply_opacity(0.6);
                    draw(
                        Spinner::new(theme, "opacity", "Loading preview").animate(!self.reduced),
                        ui,
                        frame,
                    );
                });
        }
        frame.label(ui,"Hide and reinsert, switch pages, scroll out of view, and compare real browser/native timing and status announcements. No progress percentage is implied.");
    }
}
fn draw(spinner: Spinner<'_>, ui: &mut egui::Ui, frame: &mut Layout) {
    let output = spinner.show(ui);
    frame.track(match frame.phase {
        Phase::Register => output.response,
        Phase::Paint => output.paint(ui.painter()).expect("spinner compositor"),
    });
}
