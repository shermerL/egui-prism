use crate::input::{self, Draw, Layout};
use egui_prism::{
    checkbox::{Checkbox, CheckboxRegistration},
    egui,
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Disabled,
    Reversed,
    Long,
    Invalid,
    Compositing,
    Instance(usize),
    Count(usize),
    Sample { background: usize, checked: bool },
}

pub(super) struct CheckboxCase {
    values: [bool; 2],
    changes: [u64; 2],
    disabled: bool,
    reversed: bool,
    long: bool,
    invalid: bool,
    compositing: bool,
}

impl Default for CheckboxCase {
    fn default() -> Self {
        Self {
            values: [false, true],
            changes: [0; 2],
            disabled: false,
            reversed: false,
            long: false,
            invalid: false,
            compositing: false,
        }
    }
}

impl CheckboxCase {
    pub(super) fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
        match control {
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disable checkboxes").into(),
            Control::Reversed => ui
                .checkbox(&mut self.reversed, "Reverse checkbox instances")
                .into(),
            Control::Long => ui.checkbox(&mut self.long, "Long labels").into(),
            Control::Invalid => ui.checkbox(&mut self.invalid, "Invalid checkboxes").into(),
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
                unreachable!("Checkboxes use their registered instance")
            }
        }
    }

    fn checkbox<'a>(
        &'a self,
        ui: &mut egui::Ui,
        theme: &'a Theme,
        control: &'a Control,
    ) -> Checkbox<'a, 'a> {
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
                Checkbox::new(theme, &self.values[index], label)
                    .id(egui::Id::unique(("gallery-checkbox", index)))
                    .invalid(self.invalid)
            }
            Control::Sample {
                background,
                ref checked,
            } => {
                ui.disable();
                let label = if *checked { "On" } else { "Off" };
                let id = egui::Id::unique(("gallery-checkbox-sample", background, *checked));
                Checkbox::new(theme, checked, label).id(id)
            }
            _ => unreachable!("only Checkbox instances use the phase adapter"),
        }
    }

    pub(super) fn handle(
        &mut self,
        control: Control,
        registration: &CheckboxRegistration,
        event: &egui::RoutedInput,
    ) -> egui::Response {
        let Control::Instance(index) = control else {
            unreachable!("only enabled checkbox instances receive input");
        };
        let response = registration.handle(event, &mut self.values[index]);
        if response.changed() {
            self.changes[index] += 1;
        }
        response
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
        frame.track(ui.heading("Prism checkboxes"));
        frame.label(
            ui,
            "Click the box or its label, or focus with Tab and toggle with Space. Space activates on release; Enter does not toggle.",
        );
        // Give each setting a full row so its local input UI can lay out
        // the complete checkbox label even in the narrow comparison fixture.
        for control in [
            Control::Disabled,
            Control::Reversed,
            Control::Long,
            Control::Invalid,
            Control::Compositing,
        ] {
            self.widget(ui, theme, frame, control);
        }
        for index in if self.reversed { [1, 0] } else { [0, 1] } {
            ui.push_id(("checkbox-instance", index), |ui| {
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
                "16 × 16pt box, 4pt corners and a 14pt check indicator. Invalid and keyboard focus match the pinned reference.",
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
                frame.checkbox_widget(ui, control, |ui| self.checkbox(ui, theme, &control));
            }
            _ => {
                frame.widget(ui, input::Control::Checkbox(control), |ui| {
                    self.control(ui, control)
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Control as CheckboxControl, egui};
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
                let resources = crate::selects::Resources::new();
                let mut select_live = crate::selects::Live::default();
                context.begin_input();
                let root = LocalUi::named(ui, ui.make_persistent_id("gallery-root"));
                let page = gallery.layout(
                    &mut root.open(context),
                    Phase::Register,
                    Default::default(),
                    &resources,
                    &mut select_live,
                );
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
    fn checkbox_gallery_routes_navigation_and_layout_changes_in_one_pass() {
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
            matches!(control, Control::Page(Case::Checkboxes))
        });
        gallery.case = Case::Checkboxes;
        let checkbox_rects = rects(&mut gallery, &context);
        let row = |index| {
            rect(
                &checkbox_rects,
                |control| matches!(control, Control::Checkbox(CheckboxControl::Instance(actual)) if *actual == index),
            )
        };
        let a_track = row(0).left_center() + egui::vec2(8.0, 0.0);
        let b_track = row(1).left_center() + egui::vec2(8.0, 0.0);
        let b_label = row(1).right_center() - egui::vec2(4.0, 0.0);
        let disabled = rect(&checkbox_rects, |control| {
            matches!(control, Control::Checkbox(CheckboxControl::Disabled))
        })
        .center();
        let reverse = rect(&checkbox_rects, |control| {
            matches!(control, Control::Checkbox(CheckboxControl::Reversed))
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
        assert_eq!(gallery.case, Case::Checkboxes);
        assert_eq!(gallery.checkboxes.values, [true, false]);
        assert_eq!(gallery.checkboxes.changes, [1, 1]);

        frame(
            &mut gallery,
            click(disabled).into_iter().chain(click(a_track)).collect(),
        );
        assert!(gallery.checkboxes.disabled);
        assert_eq!(gallery.checkboxes.changes, [1, 1]);

        frame(
            &mut gallery,
            click(disabled)
                .into_iter()
                .chain(click(reverse))
                .chain(click(b_track))
                .collect(),
        );
        assert!(!gallery.checkboxes.disabled);
        assert!(gallery.checkboxes.reversed);
        assert_eq!(gallery.checkboxes.values, [false, false]);
        assert_eq!(gallery.checkboxes.changes, [2, 1]);
        frame(&mut gallery, Vec::new());
        assert_eq!(gallery.checkboxes.changes, [2, 1]);

        gallery.narrow = true;
        gallery.checkboxes.long = true;
        let narrow = rects(&mut gallery, &context);
        let samples =
            |control: &Control| matches!(control, Control::Checkbox(CheckboxControl::Compositing));
        assert!(
            rect(&narrow, samples).height() <= rect(&checkbox_rects, samples).height() + 0.5,
            "setting label should take the same height in narrow and wide layouts"
        );
        let long_label = rect(&narrow, |control| {
            matches!(control, Control::Checkbox(CheckboxControl::Instance(0)))
        });
        assert!(long_label.height() > 30.0);
        assert!(long_label.width() <= 320.0, "{long_label:?}");
    }
}
