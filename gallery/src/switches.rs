use crate::input::{self, Draw, Layout};
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
    pub(super) fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
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
            Control::Count(index) => ui
                .label(format!(
                    "{}: {} · Changes: {}",
                    ['A', 'B'][index],
                    self.values[index],
                    self.changes[index]
                ))
                .into(),
            Control::Instance(_) | Control::Sample { .. } => {
                unreachable!("Switches use their registered instance")
            }
        }
    }

    fn switch<'a>(
        &'a self,
        ui: &mut egui::Ui,
        theme: &'a Theme,
        control: &'a Control,
    ) -> Switch<'a, 'a> {
        let size = if self.small {
            SwitchSize::Small
        } else {
            SwitchSize::Default
        };
        match *control {
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
                Switch::new(theme, &self.values[index], label)
                    .id(egui::Id::unique(("gallery-switch", index)))
                    .size(size)
                    .invalid(self.invalid)
            }
            Control::Sample {
                background,
                ref checked,
            } => {
                ui.disable();
                let label = if *checked { "On" } else { "Off" };
                let id = egui::Id::unique(("gallery-switch-sample", background, *checked));
                Switch::new(theme, checked, label).id(id).size(size)
            }
            _ => unreachable!("only Switch instances use the phase adapter"),
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
        // Give each setting a full row so its local input UI can lay out
        // the complete checkbox label even in the narrow comparison fixture.
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
        match control {
            Control::Instance(_) | Control::Sample { .. } => {
                frame.switch_widget(ui, control, |ui| self.switch(ui, theme, &control));
            }
            _ => {
                frame.widget(ui, input::Control::Switch(control), |ui| {
                    self.control(ui, control)
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Control as SwitchControl, egui};
    use crate::{
        Case, Gallery,
        input::{Control, LocalUi, Phase},
    };

    fn raw(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 800.0),
            )),
            events,
            ..Default::default()
        }
    }

    // Measure production registration in its own pass. The test then sends
    // raw events through Gallery::ui without retaining any layout from here.
    fn rects(gallery: &mut Gallery, context: &egui::Context) -> Vec<(Control, egui::Rect)> {
        let mut rects = Vec::new();
        context
            .run_ui(raw(Vec::new()), |ui| {
                context.begin_input();
                let root = LocalUi::named(ui, ui.make_persistent_id("gallery-root"));
                let page =
                    gallery.layout(&mut root.open(context), Phase::Register, Default::default());
                rects.extend(
                    page.chrome
                        .entries
                        .iter()
                        .chain(&page.body.entries)
                        .map(|entry| (entry.control.clone(), entry.response.rect)),
                );
                while context.next_input_event().is_some() {}
            })
            .drop_without_applying_deltas();
        rects
    }

    fn rect(rects: &[(Control, egui::Rect)], matches: impl Fn(&Control) -> bool) -> egui::Rect {
        rects
            .iter()
            .find(|(control, _)| matches(control))
            .expect("registered control")
            .1
    }

    fn click(pos: egui::Pos2) -> [egui::Event; 2] {
        [true, false].map(|pressed| egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: egui::Modifiers::NONE,
        })
    }

    #[test]
    #[ignore = "requires a native wgpu adapter"]
    fn switch_gallery_routes_navigation_and_layout_changes_in_one_pass() {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .expect("native adapter");
        let (device, _) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let context = egui::Context::default();
        context.options_mut(|options| options.max_passes = std::num::NonZeroUsize::MIN);
        let mut gallery = Gallery::new(&context, &device).unwrap();
        let tab = rect(&rects(&mut gallery, &context), |control| {
            matches!(control, Control::Page(Case::Switches))
        });
        gallery.case = Case::Switches;
        let switch_rects = rects(&mut gallery, &context);
        let row = |index| {
            rect(
                &switch_rects,
                |control| matches!(control, Control::Switch(SwitchControl::Instance(actual)) if *actual == index),
            )
        };
        let a_track = row(0).left_center() + egui::vec2(16.0, 0.0);
        let b_track = row(1).left_center() + egui::vec2(16.0, 0.0);
        let b_label = row(1).right_center() - egui::vec2(4.0, 0.0);
        let disabled = rect(&switch_rects, |control| {
            matches!(control, Control::Switch(SwitchControl::Disabled))
        })
        .center();
        let reverse = rect(&switch_rects, |control| {
            matches!(control, Control::Switch(SwitchControl::Reversed))
        })
        .center();
        gallery.case = Case::Theme;

        let frame = |gallery: &mut Gallery, events| {
            context
                .run_ui(raw(events), |ui| gallery.ui(ui, &device))
                .drop_without_applying_deltas();
        };
        frame(
            &mut gallery,
            click(tab.center())
                .into_iter()
                .chain(click(a_track))
                .chain(click(b_label))
                .collect(),
        );
        assert_eq!(gallery.case, Case::Switches);
        assert_eq!(gallery.switches.values, [true, false]);
        assert_eq!(gallery.switches.changes, [1, 1]);

        frame(
            &mut gallery,
            click(disabled).into_iter().chain(click(a_track)).collect(),
        );
        assert!(gallery.switches.disabled);
        assert_eq!(gallery.switches.changes, [1, 1]);

        frame(
            &mut gallery,
            click(disabled)
                .into_iter()
                .chain(click(reverse))
                .chain(click(b_track))
                .collect(),
        );
        assert!(!gallery.switches.disabled);
        assert!(gallery.switches.reversed);
        assert_eq!(gallery.switches.values, [false, false]);
        assert_eq!(gallery.switches.changes, [2, 1]);
        frame(&mut gallery, Vec::new());
        assert_eq!(gallery.switches.changes, [2, 1]);

        gallery.narrow = true;
        gallery.switches.long = true;
        let narrow = rects(&mut gallery, &context);
        let samples =
            |control: &Control| matches!(control, Control::Switch(SwitchControl::Compositing));
        assert!(
            rect(&narrow, samples).height() <= rect(&switch_rects, samples).height() + 0.5,
            "setting label should take the same height in narrow and wide layouts"
        );
        let long_label = rect(&narrow, |control| {
            matches!(control, Control::Switch(SwitchControl::Instance(0)))
        });
        assert!(long_label.height() > 30.0);
        assert!(long_label.width() <= 320.0, "{long_label:?}");
    }
}
