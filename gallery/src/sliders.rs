use egui_prism::{
    egui,
    slider::{Slider, SliderRegistration},
    theme::Theme,
};

#[derive(Clone, Copy)]
#[allow(dead_code, reason = "the ordered Gallery adapter is wired separately")]
pub(super) enum Control {
    Heading,
    Help,
    Disabled,
    Reversed,
    Long,
    Tinted,
    WidthLabel,
    Width(f32),
    Slider(usize),
    Value(usize),
    Focus(bool),
    Footer,
}

#[allow(dead_code, reason = "the ordered Gallery adapter is wired separately")]
pub(super) struct Draw {
    pub response: egui::Response,
    pub ids: Vec<egui::Id>,
}

impl From<egui::Response> for Draw {
    fn from(response: egui::Response) -> Self {
        Self {
            response,
            ids: Vec::new(),
        }
    }
}

/// Owned by the Gallery's current registration layout, not by persistent state.
#[allow(dead_code, reason = "the ordered Gallery adapter is wired separately")]
pub(super) struct Live {
    sliders: [Option<(SliderRegistration, egui::Rect)>; 3],
}

impl Default for Live {
    fn default() -> Self {
        Self {
            sliders: std::array::from_fn(|_| None),
        }
    }
}

#[allow(dead_code, reason = "the ordered Gallery adapter is wired separately")]
pub(super) struct SliderCase {
    values: [f64; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
    width: f32,
    tinted: bool,
}

impl Default for SliderCase {
    fn default() -> Self {
        Self {
            values: [50.0, 0.0],
            changes: [0; 2],
            disabled: false,
            reversed: false,
            long: false,
            width: 448.0,
            tinted: false,
        }
    }
}

#[allow(dead_code, reason = "the ordered Gallery adapter is wired separately")]
impl SliderCase {
    /// Temporary compatibility with the existing immediate Gallery match arm.
    ///
    /// Deliberately display-only: no local begin/drain or legacy Slider path.
    pub(super) fn show(&mut self, ui: &mut egui::Ui, _theme: &Theme) {
        ui.heading("Prism sliders");
        ui.label("Slider interaction is unavailable in this preview.");
        ui.monospace(format!(
            "A: {:.1} · Changes: {}\nB: {:.1} · Changes: {}",
            self.values[0], self.changes[0], self.values[1], self.changes[1],
        ));
    }

    /// Shared structural traversal for Register and Paint.
    ///
    /// The host's visitor creates its stable leaf scope, captures LocalUi before
    /// drawing, and records the returned IDs in the current Layout.
    pub(super) fn layout(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        mut visit: impl FnMut(&mut Self, &mut egui::Ui, Control) -> egui::Response,
    ) {
        visit(self, ui, Control::Heading);
        visit(self, ui, Control::Help);

        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Disabled,
                Control::Reversed,
                Control::Long,
                Control::Tinted,
            ] {
                visit(self, ui, control);
            }
        });
        ui.horizontal_wrapped(|ui| {
            visit(self, ui, Control::WidthLabel);
            for width in [448.0, 200.0, 20.0, 16.0] {
                visit(self, ui, Control::Width(width));
            }
        });

        let surface = if self.tinted {
            egui::Color32::from_rgb(25, 51, 65)
        } else {
            theme.colors.background.background
        };

        egui::Frame::NONE
            .fill(surface)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.spacing_mut().slider_width = self.width;

                for index in if self.reversed { [1, 0] } else { [0, 1] } {
                    // Identity is semantic, never the current visual position.
                    let id = ui.make_persistent_id(("slider-instance", index));
                    ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                        let response = visit(self, ui, Control::Slider(index));
                        visit(self, ui, Control::Value(index));
                        // Always reserve one line; focus changes cannot move B.
                        visit(self, ui, Control::Focus(response.has_focus()));
                    });
                }

                ui.add_space(16.0);
                let id = ui.make_persistent_id(("slider-instance", 2));
                ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                    visit(self, ui, Control::Slider(2));
                });
            });

        visit(self, ui, Control::Footer);
    }

    pub(super) fn register_control(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        control: Control,
        live: &mut Live,
    ) -> Draw {
        let Control::Slider(index) = control else {
            return self.stock(ui, control).into();
        };

        let (slider, bounds) = self.register_slider(ui, theme, index);
        let ids = slider.input_ids().to_vec();
        // Only used for routing identity/focus. It is not a replacement hit rect.
        let response = ui
            .ctx()
            .read_response(slider.id())
            .expect("registered slider");
        live.sliders[index] = Some((slider, bounds));
        Draw { response, ids }
    }

    /// Handle exactly one host-routed event using the retained registration.
    ///
    /// Returns whether the host must rebuild this case's layout before dispatching
    /// another event. Slider value changes refresh their own thumb geometry.
    pub(super) fn handle(
        &mut self,
        ui: &mut egui::Ui,
        control: Control,
        live: &Live,
        event: &egui::RoutedInput,
    ) -> bool {
        if let Control::Slider(index) = control {
            let (slider, _) = live.sliders[index].as_ref().expect("current slider layout");
            assert_eq!(event.target, Some(slider.id()));

            if index < 2 {
                if slider.handle(ui, &mut self.values[index], event).changed() {
                    self.changes[index] += 1;
                }
            } else {
                let mut amount = 35.0;
                let response = slider.handle(ui, &mut amount, event);
                debug_assert!(!response.changed());
                debug_assert_eq!(amount, 35.0);
            }
            return false;
        }

        match control {
            Control::Disabled
            | Control::Reversed
            | Control::Long
            | Control::Tinted
            | Control::Width(_) => self.stock(ui, control).changed(),
            _ => false,
        }
    }

    pub(super) fn paint_control(
        &mut self,
        ui: &mut egui::Ui,
        control: Control,
        live: &mut Live,
    ) -> Draw {
        let Control::Slider(index) = control else {
            return self.stock(ui, control).into();
        };

        // Caller-owned, same-layer, locally clipped, with true parent opacity.
        // Save BEFORE entering the Slider's disabled scope.
        let painter = ui.painter().clone();
        let (slider, bounds) = live.sliders[index].take().expect("current slider layout");
        ui.advance_cursor_after_rect(bounds);
        let ids = slider.input_ids().to_vec();
        let output = slider.finish(ui, self.value(index));
        let response = output.paint(&painter).expect("slider compositor");
        Draw { response, ids }
    }

    fn value(&self, index: usize) -> f64 {
        match index {
            0 | 1 => self.values[index],
            2 => 35.0,
            _ => unreachable!("slider instance"),
        }
    }

    fn register_slider(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        index: usize,
    ) -> (SliderRegistration, egui::Rect) {
        let (label, range, step) = match index {
            0 | 1 => {
                let name = if index == 0 { "A" } else { "B" };
                let label = if self.long {
                    format!(
                        "Value {name} with a deliberately long label that wraps \
                         above the rail inside narrow content"
                    )
                } else {
                    format!("Value {name}")
                };
                let (range, step) = if index == 0 {
                    (0.0..=100.0, 5.0)
                } else {
                    (-1.0..=1.0, 0.1)
                };
                (label, range, step)
            }
            2 => ("Disabled amount".into(), 0.0..=100.0, 1.0),
            _ => unreachable!("slider instance"),
        };

        let output = ui.add_enabled_ui(index < 2 && !self.disabled, |ui| {
            Slider::new(theme, label)
                .range(range)
                .step_by(step)
                .register(ui, self.value(index))
        });
        (output.inner, output.response.rect)
    }

    fn stock(&mut self, ui: &mut egui::Ui, control: Control) -> egui::Response {
        match control {
            Control::Heading => ui.heading("Prism sliders"),
            Control::Help => ui.label(
                "Click or drag the rail. Use Tab, arrows, Home/End, and \
                 Shift+arrows or PageUp/Down (ten steps). A uses 0 to 100 \
                 in steps of 5; B uses -1 to 1 in steps of 0.1.",
            ),
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable sliders"),
            Control::Reversed => ui.checkbox(&mut self.reversed, "Reverse slider instances"),
            Control::Long => ui.checkbox(&mut self.long, "Long labels"),
            Control::Tinted => ui.checkbox(&mut self.tinted, "Tinted surface"),
            Control::WidthLabel => ui.label("Rail width:"),
            Control::Width(width) => {
                ui.selectable_value(&mut self.width, width, format!("{width}pt"))
            }
            Control::Value(index) => {
                let name = if index == 0 { "A" } else { "B" };
                ui.add(
                    egui::Label::new(format!(
                        "{name}: {:.1} · Changes: {}",
                        self.values[index], self.changes[index],
                    ))
                    .truncate(),
                )
            }
            Control::Focus(focused) => ui.add(
                egui::Label::new(
                    egui::RichText::new(if focused {
                        "This slider has keyboard focus."
                    } else {
                        "\u{a0}"
                    })
                    .small(),
                )
                .truncate(),
            ),
            Control::Footer => ui.small(
                "Values and change counters belong to this example. \
                 Sliders do not implement application side effects.",
            ),
            Control::Slider(_) => unreachable!("use Slider phase methods"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_prism::compositing::{self, Backend, Error, Group};
    use std::sync::Arc;

    struct TestHost;
    impl Backend for TestHost {
        fn callback(&self, _: &egui::Context, group: Group) -> Result<egui::PaintCallback, Error> {
            Ok(egui::PaintCallback {
                rect: group.bounds(),
                callback: Arc::new(()),
            })
        }
    }

    #[test]
    fn case_dispatches_its_registered_slider_and_paints_after_drain() {
        let context = egui::Context::default();
        context.options_mut(|o| o.ordered_input = true);
        let _host = compositing::install(&context, Arc::new(TestHost)).unwrap();
        let theme = Theme::light();
        let mut case = SliderCase::default();
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        context
            .run_ui(
                egui::RawInput {
                    events: vec![key(egui::Key::ArrowRight)],
                    ..Default::default()
                },
                |ui| {
                    context.begin_input();
                    let rect = ui.available_rect_before_wrap();
                    let scope_id = egui::Id::unique("slider-case-test");
                    let mut live = Live::default();
                    let mut first = None;
                    let mut layout =
                        ui.new_child(egui::UiBuilder::new().scope_id(scope_id).max_rect(rect));
                    case.layout(&mut layout, &theme, |case, ui, control| {
                        let draw = case.register_control(ui, &theme, control, &mut live);
                        if matches!(control, Control::Slider(0)) {
                            first = Some(draw.response.id);
                            assert!(draw.ids.contains(&draw.response.id));
                        }
                        draw.response
                    });
                    context.memory_mut(|memory| memory.request_focus(first.unwrap()));
                    while let Some(event) = context.next_input_event() {
                        if event.target == first {
                            assert!(!case.handle(&mut layout, Control::Slider(0), &live, &event));
                        }
                    }
                    let mut paint =
                        ui.new_child(egui::UiBuilder::new().scope_id(scope_id).max_rect(rect));
                    case.layout(&mut paint, &theme, |case, ui, control| {
                        case.paint_control(ui, control, &mut live).response
                    });
                    assert!(live.sliders.iter().all(Option::is_none));
                },
            )
            .drop_without_applying_deltas();
        assert_eq!(case.values, [55.0, 0.0]);
        assert_eq!(case.changes, [1, 0]);
    }
}
