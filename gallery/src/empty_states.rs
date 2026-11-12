//! Caller-owned empty, loading and inline-error presentations, with no network work.
use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{button::Button, egui, empty::Empty, theme::Theme};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum State {
    #[default]
    Empty,
    Loading,
    Error,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    State(State),
    Long,
    Disabled,
    Action,
    Count,
}

pub(super) struct EmptyCase {
    state: State,
    long: bool,
    disabled: bool,
    count: u64,
    icon: egui::TextureHandle,
}
impl EmptyCase {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut image = egui::ColorImage::filled([24, 24], egui::Color32::TRANSPARENT);
        for y in 4..20 {
            for x in 4..20 {
                if (11..13).contains(&x) || (11..13).contains(&y) {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            }
        }
        Self {
            state: State::Empty,
            long: false,
            disabled: false,
            count: 0,
            icon: ctx.load_texture("empty-add", image, egui::TextureOptions::NEAREST),
        }
    }
    pub fn activated(&mut self, control: Control) {
        if matches!(control, Control::Action) {
            self.count += 1;
            self.state = State::Loading;
        }
    }
    pub fn control(&mut self, ui: &mut egui::Ui, theme: &Theme, control: Control) -> Draw {
        match control {
            Control::State(state) => ui
                .selectable_value(
                    &mut self.state,
                    state,
                    match state {
                        State::Empty => "Empty",
                        State::Loading => "Loading",
                        State::Error => "Inline error",
                    },
                )
                .into(),
            Control::Long => ui.checkbox(&mut self.long, "Long localized copy").into(),
            Control::Disabled => ui
                .checkbox(&mut self.disabled, "Disable caller action")
                .into(),
            Control::Action => {
                if self.disabled {
                    ui.disable();
                }
                Button::new(
                    theme,
                    if self.state == State::Error {
                        "Retry"
                    } else {
                        "Create project"
                    },
                )
                .register(ui)
                .into()
            }
            Control::Count => ui.label(format!("Caller actions: {}", self.count)).into(),
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Empty states"));
        frame.label(
            ui,
            "Caller selects empty, loading or failed state. Actions only change local demo state.",
        );
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::State(State::Empty),
                Control::State(State::Loading),
                Control::State(State::Error),
                Control::Long,
                Control::Disabled,
            ] {
                frame.widget(ui, input::Control::Empty(control), |ui| {
                    self.control(ui, theme, control)
                });
            }
        });
        frame.label(ui,"Padding follows viewport width: 24pt below 768pt, 48pt at or above it, including in a narrow container.");
        match self.state {
            State::Empty => {
                let icon = self.icon.clone();
                let output=Empty::new(theme,"projects-empty")
                    .title(if self.long {"Noch keine gespeicherten Projekte vorhanden"} else {"No projects yet"})
                    .description(if self.long {"Vos projets enregistrés apparaîtront ici. Créez votre premier projet ou importez un projet existant pour commencer à travailler ensemble."} else {"Create a project to get started."})
                    .icon(&icon).show(ui,|body| {
                        frame.widget(body.ui(),input::Control::Empty(Control::Action),|ui| self.control(ui,theme,Control::Action));
                    });
                frame.track(match frame.phase {
                    Phase::Register => output.response,
                    Phase::Paint => {
                        output
                            .paint(ui.painter())
                            .expect("Empty compositor")
                            .response
                    }
                });
            }
            State::Loading => {
                frame.label(
                    ui,
                    "Loading projects... The caller owns completion or cancellation.",
                );
            }
            State::Error => {
                frame.track(ui.colored_label(
                    theme.colors.destructive.background,
                    "Projects could not be loaded. Your existing work is unchanged.",
                ));
                frame.widget(ui, input::Control::Empty(Control::Action), |ui| {
                    self.control(ui, theme, Control::Action)
                });
            }
        }
        frame.widget(ui, input::Control::Empty(Control::Count), |ui| {
            self.control(ui, theme, Control::Count)
        });
        frame.label(
            ui,
            "140pt container, short title and unbreakable text under the parent clip",
        );
        ui.scope(|ui| {
            ui.set_max_width(140.);
            let output = Empty::new(theme, "narrow-empty")
                .title("No items")
                .description("UnbreakableProjectIdentifier")
                .show(ui, |_| {});
            frame.track(match frame.phase {
                Phase::Register => output.response,
                Phase::Paint => {
                    output
                        .paint(ui.painter())
                        .expect("Empty compositor")
                        .response
                }
            });
        });
        frame.label(ui, "60% opacity over a tinted surface");
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(52, 120, 246))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.multiply_opacity(0.6);
                let output = Empty::new(theme, "tinted-empty")
                    .title("No items")
                    .description("Nothing has been added yet.")
                    .show(ui, |_| {});
                frame.track(match frame.phase {
                    Phase::Register => output.response,
                    Phase::Paint => {
                        output
                            .paint(ui.painter())
                            .expect("Empty compositor")
                            .response
                    }
                });
            });
    }
}
