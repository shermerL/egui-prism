//! Noninteractive hints using the Gallery's existing ordered cursor.
use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{
    egui,
    theme::Theme,
    tooltip::{Tooltip, TooltipRegistration},
};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Hidden,
    Long,
    Reverse,
    Delayed,
    Trigger(usize),
    Count(usize),
    Outside,
    OutsideCount,
}

pub(super) struct Resources {
    themes: [Theme; 2],
}
impl Resources {
    pub fn new() -> Self {
        Self {
            themes: [Theme::light(), Theme::dark()],
        }
    }
}

#[derive(Default)]
pub(super) struct Live<'a> {
    tips: Vec<TooltipRegistration<'a>>,
    painters: Vec<egui::Painter>,
    bounds: Vec<egui::Rect>,
}
impl Live<'_> {
    pub fn handle(&mut self, event: &egui::RoutedInput) {
        TooltipRegistration::handle_all(&mut self.tips, event);
    }
    pub fn unregister(&mut self) {
        for tip in &mut self.tips {
            tip.unregister();
        }
        self.tips.clear();
        self.painters.clear();
        self.bounds.clear();
    }
}

#[derive(Default)]
pub(super) struct TooltipCase {
    disabled: bool,
    hidden: bool,
    long: bool,
    reverse: bool,
    delayed: bool,
    counts: [u64; 2],
    outside: u64,
}

const LONG: &str = "This is a long plain-text hint. Move from the trigger into the hint and back without losing it. It is bounded by the viewport, contains no links or actions, and never replaces the trigger's accessible label. Extra detail wraps rather than widening the page.";
fn id(index: usize) -> egui::Id {
    egui::Id::unique(("gallery-tooltip-trigger", index))
}

impl TooltipCase {
    pub fn show<'a>(
        &mut self,
        ui: &mut egui::Ui,
        resources: &'a Resources,
        dark: bool,
        frame: &mut Layout,
        live: &mut Live<'a>,
    ) {
        let theme = &resources.themes[usize::from(dark)];
        frame.track(ui.heading("Prism tooltips"));
        frame.label(ui, "Hover or Tab to a trigger. Escape, activation and window blur dismiss. Hints never intercept clicks; move into the hint to keep reading.");
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Disabled,
                Control::Hidden,
                Control::Long,
                Control::Reverse,
                Control::Delayed,
            ] {
                self.widget(ui, frame, control);
            }
        });
        frame.label(
            ui,
            "Default delay: 0ms · optional delay: 500ms · animation: 150ms · viewport-bounded text",
        );
        ui.add_space(48.0);
        for index in if self.reverse { [1, 0] } else { [0, 1] } {
            if index == 1 && self.hidden {
                continue;
            }
            ui.push_id(("tooltip-instance", index), |ui| {
                self.instance(ui, theme, frame, live, index);
                self.widget(ui, frame, Control::Count(index));
            });
            ui.add_space(24.0);
        }
        ui.push_id(("tooltip-instance", 2), |ui| {
            self.instance(ui, theme, frame, live, 2)
        });
        ui.add_space(24.0);
        self.widget(ui, frame, Control::Outside);
        self.widget(ui, frame, Control::OutsideCount);
        frame.label(ui, "Hide/disable/reverse while a hint is pending. Scroll, use narrow content and 150% native scale, and compare light/dark against the browser fixture.");
        ui.add_space(320.0);
        frame.label(ui, "Scroll endpoint: moving an ancestor cancels its hints; wheel input is still owned by the scroll shell.");
    }

    fn instance<'a>(
        &self,
        ui: &mut egui::Ui,
        theme: &'a Theme,
        frame: &mut Layout,
        live: &mut Live<'a>,
        index: usize,
    ) {
        let phase = frame.phase;
        frame.widget(ui, input::Control::Tooltip(Control::Trigger(index)), |ui| {
            if phase == Phase::Register {
                let painter = ui.painter().clone();
                let enabled = index < 2 && !self.disabled;
                let allocated = ui.add_enabled_ui(enabled, |ui| {
                    let (_, rect) = ui.allocate_space(egui::vec2(192.0, 36.0));
                    let response = ui.interact(rect, id(index), egui::Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, enabled, label(index))
                    });
                    if enabled {
                        ui.memory_mut(|memory| {
                            memory.register_focus_policy(
                                id(index),
                                response.layer_id,
                                egui::EventFilter {
                                    escape: true,
                                    ..Default::default()
                                },
                            )
                        });
                    }
                    let tip = Tooltip::new(
                        theme,
                        &response,
                        if self.long { LONG } else { "Add to library" },
                    )
                    .enabled(enabled)
                    .delay(if self.delayed {
                        Duration::from_millis(500)
                    } else {
                        Duration::ZERO
                    })
                    .register(ui);
                    live.tips.push(tip);
                    response
                });
                live.painters.push(painter);
                live.bounds.push(allocated.response.rect);
                allocated.inner.into()
            } else {
                let slot = live
                    .tips
                    .iter()
                    .position(|tip| tip.response.id == id(index))
                    .expect("live Tooltip");
                let tip = &live.tips[slot];
                let response = ui
                    .ctx()
                    .read_response(id(index))
                    .unwrap_or_else(|| tip.response.clone());
                ui.advance_cursor_after_rect(live.bounds[slot]);
                let painter = &live.painters[slot];
                let color = if response.enabled() {
                    theme.colors.background.foreground
                } else {
                    theme.colors.muted.foreground
                };
                painter.rect_filled(
                    response.rect,
                    8,
                    if response.hovered() {
                        theme.colors.accent.background
                    } else {
                        theme.colors.background.background
                    },
                );
                painter.rect_stroke(
                    response.rect,
                    8,
                    egui::Stroke::new(1.0, theme.colors.border),
                    egui::StrokeKind::Inside,
                );
                if response.has_focus() {
                    painter.rect_stroke(
                        response.rect.expand(2.0),
                        8,
                        egui::Stroke::new(2.0, theme.colors.ring),
                        egui::StrokeKind::Outside,
                    );
                }
                painter.text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label(index),
                    egui::FontId::proportional(14.0),
                    color,
                );
                tip.paint(painter).expect("tooltip compositor");
                response.into()
            }
        });
    }

    fn widget(&mut self, ui: &mut egui::Ui, frame: &mut Layout, control: Control) {
        frame.widget(ui, input::Control::Tooltip(control), |ui| {
            self.control(ui, control)
        });
    }

    pub fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disabled").into(),
            Control::Hidden => ui.checkbox(&mut self.hidden, "Hide B").into(),
            Control::Long => ui.checkbox(&mut self.long, "Long text").into(),
            Control::Reverse => ui.checkbox(&mut self.reverse, "Reverse").into(),
            Control::Delayed => ui.checkbox(&mut self.delayed, "500ms delay").into(),
            Control::Trigger(index) => {
                let response = ui.ctx().read_response(id(index)).expect("ordinary trigger");
                if index < 2 && response.clicked() {
                    self.counts[index] += 1;
                }
                response.into()
            }
            Control::Count(index) => ui
                .label(format!(
                    "{} activations: {}",
                    ['A', 'B'][index],
                    self.counts[index]
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
                .label(format!("Outside activations: {}", self.outside))
                .into(),
        }
    }
}
fn label(index: usize) -> &'static str {
    match index {
        0 => "Action A",
        1 => "Action B",
        _ => "Disabled trigger",
    }
}
