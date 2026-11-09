//! Caller-owned Dialog examples in the Gallery's single ordered input cursor.

use crate::input::{self, Draw, Layout};
use egui_prism::{
    button::{Button, ButtonOutput, ButtonRole},
    compositing::{Group, Item},
    dialog::{Dialog, DialogRegistration, DialogUi},
    egui,
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Long,
    Reverse,
    KeepOpen,
    Tinted,
    Open(usize),
    Count(usize),
    Outside,
    OutsideCount,
}

#[derive(Clone, Copy)]
enum Action {
    Done,
    Apply,
}

type Children = Vec<(Action, ButtonOutput)>;
pub(super) struct Live<'a> {
    registrations: Vec<DialogRegistration<'a, Children>>,
    open: [bool; 2],
    long: bool,
}
impl Live<'_> {
    pub fn unregister(self) {
        for registration in self.registrations {
            registration.unregister();
        }
    }
}

#[derive(Default)]
pub(super) struct DialogCase {
    open: [bool; 2],
    actions: [u64; 2],
    dismissals: [u64; 2],
    outside: u64,
    long: bool,
    reverse: bool,
    keep_open: bool,
    tinted: bool,
}

fn body(slot: &mut DialogUi<'_>, theme: &Theme, long: bool) -> Children {
    if long {
        slot.ui().spacing_mut().item_spacing.y = 8.0;
        for index in 1..=30 {
            let ui = slot.ui();
            let galley = ui.painter().layout_no_wrap(
                format!("Row {index:02}"),
                egui::FontId::proportional(14.0),
                theme.colors.background.foreground,
            );
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 20.0), egui::Sense::hover());
            let shape = egui::Shape::galley(rect.min, galley, theme.colors.background.foreground);
            let clip = ui.clip_rect();
            slot.draw(
                Group::new(
                    rect,
                    1.0,
                    vec![Item::Shape(egui::epaint::ClippedShape::new(clip, shape))],
                )
                .expect("dialog text group"),
            );
        }
        slot.ui().add_space(8.0);
    }
    let narrow = slot.ui().ctx().content_rect().width() < 640.0;
    let layout = if narrow {
        egui::Layout::top_down(egui::Align::Min)
    } else {
        egui::Layout::right_to_left(egui::Align::Center)
    };
    let available_width = slot.ui().available_width();
    let buttons = slot
        .ui()
        .allocate_ui_with_layout(
            egui::vec2(available_width, if narrow { 80.0 } else { 36.0 }),
            layout,
            |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                [
                    (
                        Action::Apply,
                        ui.push_id("apply", |ui| Button::new(theme, "Apply").register(ui))
                            .inner,
                    ),
                    (
                        Action::Done,
                        ui.push_id("done", |ui| {
                            Button::new(theme, "Done")
                                .role(ButtonRole::Outline)
                                .register(ui)
                        })
                        .inner,
                    ),
                ]
            },
        )
        .inner;
    buttons
        .into_iter()
        .map(|(action, mut output)| {
            slot.focusable(&output.response);
            if let Some(group) = output.drawing.take() {
                slot.draw(group);
            }
            (action, output)
        })
        .collect()
}

impl DialogCase {
    pub fn register<'a>(&self, ctx: &egui::Context, theme: &'a Theme) -> Live<'a> {
        let registrations = (0..2)
            .map(|index| {
                Dialog::new(
                    theme,
                    egui::Id::unique(("gallery-dialog", index)),
                    if index == 0 { "Dialog A" } else { "Dialog B" },
                )
                .description("Actions belong to the application.")
                .register(ctx, self.open[index], |slot| body(slot, theme, self.long))
            })
            .collect();
        Live {
            registrations,
            open: self.open,
            long: self.long,
        }
    }

    pub fn handle(&mut self, live: &mut Live<'_>, theme: &Theme, event: &egui::RoutedInput) {
        for index in (0..live.registrations.len()).rev() {
            let registration = &mut live.registrations[index];
            let mut dismiss = false;
            if let Some(children) = &registration.inner {
                for (action, button) in children {
                    button.cancel_unfocused();
                    if event.target == Some(button.response.id) && button.handle(event).clicked() {
                        match action {
                            Action::Apply => self.actions[index] += 1,
                            Action::Done => dismiss = true,
                        }
                    }
                }
            }
            let intent = registration.handle(event);
            dismiss |= intent.close_requested.is_some();
            if dismiss {
                self.dismissals[index] += 1;
                if !self.keep_open {
                    self.open[index] = false;
                }
            }
            if intent.layout_changed || live.open[index] != self.open[index] {
                registration.refresh(self.open[index], |slot| body(slot, theme, self.long));
                live.open[index] = self.open[index];
            }
        }
    }

    pub fn sync(&self, live: &mut Live<'_>, theme: &Theme) {
        for (index, registration) in live.registrations.iter_mut().enumerate() {
            if live.open[index] != self.open[index] || live.long != self.long {
                registration.refresh(self.open[index], |slot| body(slot, theme, self.long));
            }
        }
        live.open = self.open;
        live.long = self.long;
    }

    pub fn paint(&self, live: Live<'_>, theme: &Theme) {
        for registration in live.registrations {
            registration
                .paint(|slot| body(slot, theme, self.long))
                .expect("dialog compositor");
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Dialogs"));
        frame.label(ui, "Open either instance. Tab stays inside; Escape, the backdrop and X request a neutral dismissal.");
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Long,
                Control::Reverse,
                Control::KeepOpen,
                Control::Tinted,
            ] {
                frame.widget(ui, input::Control::Dialog(control), |ui| {
                    self.control(ui, control)
                });
            }
        });
        egui::Frame::NONE
            .fill(if self.tinted {
                egui::Color32::from_rgb(52, 120, 246)
            } else {
                theme.colors.background.background
            })
            .inner_margin(16.0)
            .show(ui, |ui| {
                for index in if self.reverse { [1, 0] } else { [0, 1] } {
                    for control in [Control::Open(index), Control::Count(index)] {
                        frame.widget(ui, input::Control::Dialog(control), |ui| {
                            self.control(ui, control)
                        });
                    }
                }
                for control in [Control::Outside, Control::OutsideCount] {
                    frame.widget(ui, input::Control::Dialog(control), |ui| {
                        self.control(ui, control)
                    });
                }
            });
    }

    pub fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
        match control {
            Control::Long => ui.checkbox(&mut self.long, "Long body").into(),
            Control::Reverse => ui.checkbox(&mut self.reverse, "Reverse instances").into(),
            Control::KeepOpen => ui.checkbox(&mut self.keep_open, "Decline dismissal").into(),
            Control::Tinted => ui.checkbox(&mut self.tinted, "Tinted surface").into(),
            Control::Open(index) => {
                let response = ui.button(format!("Open dialog {}", ['A', 'B'][index]));
                if response.clicked() {
                    self.open[index] = true;
                }
                response.into()
            }
            Control::Count(index) => ui
                .label(format!(
                    "Actions: {} · Dismissals: {}",
                    self.actions[index], self.dismissals[index]
                ))
                .into(),
            Control::Outside => {
                let response = ui.button("Outside action");
                if response.clicked() {
                    self.outside += 1;
                }
                response.into()
            }
            Control::OutsideCount => ui
                .label(format!("Outside actions: {}", self.outside))
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_body_keeps_natural_action_row_height() {
        let context = egui::Context::default();
        context.options_mut(|o| {
            o.ordered_input = true;
            o.max_passes = 1.try_into().unwrap();
        });
        egui_prism::theme::install_fonts(&context, egui::FontDefinitions::default());
        let theme = Theme::light();
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(960.0, 900.0),
                )),
                ..Default::default()
            },
            |ui| {
                ui.ctx().begin_input();
                let mut dialog = Dialog::new(&theme, egui::Id::unique("natural"), "Dialog A")
                    .description("Actions belong to the application.")
                    .register(ui.ctx(), true, |slot| body(slot, &theme, false));
                let button = &dialog.inner.as_ref().unwrap()[0].1.response;
                assert!((button.rect.top() - 463.0).abs() < 1.0, "{:?}", button.rect);
                while let Some(event) = ui.ctx().next_input_event() {
                    dialog.handle(&event);
                }
            },
        );
        output.drop_without_applying_deltas();
    }
}
