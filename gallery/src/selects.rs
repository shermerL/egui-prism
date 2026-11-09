//! Controlled Select examples using the Gallery's single ordered input cursor.

use crate::input::{self, Draw, Layout, Phase};
use egui_prism::{
    egui,
    select::{Select, SelectItem, SelectRegistration, SelectSize},
    theme::Theme,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Small,
    Disabled,
    Invalid,
    Long,
    Reversed,
    Empty,
    AllDisabled,
    Tinted,
    Reset,
    Instance,
    Count(usize),
    Outside,
    OutsideCount,
}

/// Immutable resources for the entire input batch, including replacement layouts.
/// Preparing every variant lets a configuration event change which slice is
/// borrowed without modifying storage borrowed by an earlier registration.
pub(super) struct Resources {
    themes: [Theme; 2],
    options: [Vec<SelectItem<String>>; 5],
}

impl Resources {
    pub fn new() -> Self {
        let short = [
            ("apple", "Apple"),
            ("apricot", "Apricot"),
            ("banana", "Banana"),
            ("blueberry", "Blueberry"),
            ("cherry", "Cherry"),
        ]
        .into_iter()
        .map(|(value, label)| SelectItem::new(value.to_owned(), label).disabled(value == "banana"))
        .collect::<Vec<_>>();
        let long = (1..=30)
            .map(|index| {
                SelectItem::new(format!("option-{index}"), format!("Option {index:02}"))
                    .disabled(index == 7 || index == 14)
            })
            .collect::<Vec<_>>();
        let all_disabled = |items: &[SelectItem<String>]| {
            items
                .iter()
                .cloned()
                .map(|item| item.disabled(true))
                .collect::<Vec<_>>()
        };
        let short_disabled = all_disabled(&short);
        let long_disabled = all_disabled(&long);
        Self {
            themes: [Theme::light(), Theme::dark()],
            options: [short, short_disabled, long, long_disabled, Vec::new()],
        }
    }

    fn theme(&self, dark: bool) -> &Theme {
        &self.themes[usize::from(dark)]
    }

    fn items(&self, case: &SelectCase) -> &[SelectItem<String>] {
        &self.options[if case.empty {
            4
        } else {
            usize::from(case.long) * 2 + usize::from(case.all_disabled)
        }]
    }
}

struct Registered<'a> {
    select: SelectRegistration<'a, String>,
    bounds: egui::Rect,
    painter: egui::Painter,
}

/// Only the current Gallery pass owns these borrows and drawing resources.
#[derive(Default)]
pub(super) struct Live<'a> {
    instances: [Option<Registered<'a>>; 3],
}

impl Live<'_> {
    /// Retire before replacement registration, while the input cursor is live.
    pub fn unregister(&mut self) {
        for slot in &mut self.instances {
            if let Some(mut registered) = slot.take() {
                registered.select.unregister();
            }
        }
    }
}

pub(super) struct SelectCase {
    values: [Option<String>; 2],
    changes: [u64; 2],
    outside: u64,
    small: bool,
    disabled: bool,
    invalid: bool,
    long: bool,
    reversed: bool,
    empty: bool,
    all_disabled: bool,
    tinted: bool,
}

impl Default for SelectCase {
    fn default() -> Self {
        Self {
            values: [None, Some("cherry".into())],
            changes: [0; 2],
            outside: 0,
            small: false,
            disabled: false,
            invalid: false,
            long: false,
            reversed: false,
            empty: false,
            all_disabled: false,
            tinted: false,
        }
    }
}

fn instance_id(index: usize) -> egui::Id {
    egui::Id::unique(("gallery-select", index))
}

impl SelectCase {
    /// Every registration sees every event before the Gallery offers it to a
    /// background scroll shell or an ordinary targeted control.
    pub fn handle_all(&mut self, live: &mut Live<'_>, event: &egui::RoutedInput) -> bool {
        let mut changed = false;
        for (index, slot) in live.instances.iter_mut().enumerate() {
            let Some(registered) = slot else {
                continue;
            };
            if index < 2 {
                let response = registered.select.handle(event, &mut self.values[index]);
                if response.changed() {
                    self.changes[index] += 1;
                    changed = true;
                }
            } else {
                // The permanently disabled trigger borrows A's authoritative
                // value for this event and must never change it.
                let response = registered.select.handle(event, &mut self.values[0]);
                debug_assert!(!response.changed());
            }
        }
        changed
    }

    pub fn show<'a>(
        &mut self,
        ui: &mut egui::Ui,
        resources: &'a Resources,
        dark: bool,
        frame: &mut Layout,
        live: &mut Live<'a>,
    ) {
        let theme = resources.theme(dark);
        let items = resources.items(self);
        frame.track(ui.heading("Prism selects"));
        frame.label(
            ui,
            "Open with a primary press, Enter, Space or arrows. Navigate, type to search, Enter to select; Escape restores trigger focus.",
        );
        ui.horizontal_wrapped(|ui| {
            for control in [
                Control::Small,
                Control::Disabled,
                Control::Invalid,
                Control::Long,
                Control::Reversed,
                Control::Empty,
                Control::AllDisabled,
                Control::Tinted,
            ] {
                self.widget(ui, frame, control);
            }
        });
        self.widget(ui, frame, Control::Reset);

        egui::Frame::NONE
            .fill(if self.tinted {
                egui::Color32::from_rgb(52, 120, 246)
            } else {
                theme.colors.background.background
            })
            .inner_margin(16.0)
            .show(ui, |ui| {
                for index in if self.reversed { [1, 0] } else { [0, 1] } {
                    // Reordering changes position, not trigger or option identity.
                    ui.push_id(("select-instance", index), |ui| {
                        frame.label(ui, format!("Value {}", ['A', 'B'][index]));
                        self.instance(ui, theme, items, frame, live, index);
                        self.widget(ui, frame, Control::Count(index));
                    });
                    ui.add_space(8.0);
                }
                ui.push_id(("select-instance", 2), |ui| {
                    frame.label(ui, "Disabled trigger");
                    self.instance(ui, theme, items, frame, live, 2);
                });
            });
        ui.horizontal_wrapped(|ui| {
            self.widget(ui, frame, Control::Outside);
            self.widget(ui, frame, Control::OutsideCount);
        });
        frame.track(ui.small(
            "192pt wide · Default 36pt / Small 32pt · Values and counts belong to this example.",
        ));
    }

    fn instance<'a>(
        &self,
        ui: &mut egui::Ui,
        theme: &'a Theme,
        items: &'a [SelectItem<String>],
        frame: &mut Layout,
        live: &mut Live<'a>,
        index: usize,
    ) {
        let phase = frame.phase;
        let value = &self.values[if index == 2 { 0 } else { index }];
        frame.widget(ui, input::Control::Select(Control::Instance), |ui| {
            match phase {
                Phase::Register => {
                    // Capture before entering the disabled scope. The component
                    // applies its own disabled group opacity exactly once.
                    let painter = ui.painter().clone();
                    let output = ui.add_enabled_ui(index < 2 && !self.disabled, |ui| {
                        Select::new(theme, instance_id(index), items)
                            .placeholder("Choose an option")
                            .width(192.0)
                            .size(if self.small {
                                SelectSize::Small
                            } else {
                                SelectSize::Default
                            })
                            .invalid(self.invalid)
                            .register(ui, value)
                    });
                    let response = output.inner.response.clone();
                    live.instances[index] = Some(Registered {
                        select: output.inner,
                        bounds: output.response.rect,
                        painter,
                    });
                    response.into()
                }
                Phase::Paint => {
                    let registered = live.instances[index].take().expect("live Select layout");
                    // Preserve the allocation made by add_enabled_ui without
                    // registering a trigger or popup after the cursor is drained.
                    ui.advance_cursor_after_rect(registered.bounds);
                    registered
                        .select
                        .paint(value, &registered.painter)
                        .expect("select compositor")
                        .into()
                }
            }
        });
    }

    fn widget(&mut self, ui: &mut egui::Ui, frame: &mut Layout, control: Control) {
        frame.widget(ui, input::Control::Select(control), |ui| {
            self.control(ui, control)
        });
    }

    pub fn control(&mut self, ui: &mut egui::Ui, control: Control) -> Draw {
        match control {
            Control::Small => ui.checkbox(&mut self.small, "Small").into(),
            Control::Disabled => ui.checkbox(&mut self.disabled, "Disabled").into(),
            Control::Invalid => ui.checkbox(&mut self.invalid, "Invalid").into(),
            Control::Long => ui.checkbox(&mut self.long, "Long list").into(),
            Control::Reversed => ui.checkbox(&mut self.reversed, "Reverse").into(),
            Control::Empty => ui.checkbox(&mut self.empty, "Empty").into(),
            Control::AllDisabled => ui.checkbox(&mut self.all_disabled, "All disabled").into(),
            Control::Tinted => ui.checkbox(&mut self.tinted, "Tinted").into(),
            Control::Reset => {
                let response = ui.button("Reset values");
                if response.clicked() {
                    self.values = [
                        None,
                        Some(if self.long { "option-30" } else { "cherry" }.into()),
                    ];
                }
                response.into()
            }
            Control::Count(index) => ui
                .add(
                    egui::Label::new(format!(
                        "{}: {} · Changes: {}",
                        ['A', 'B'][index],
                        self.values[index].as_deref().unwrap_or("None"),
                        self.changes[index],
                    ))
                    .truncate(),
                )
                .into(),
            Control::Outside => {
                let response = ui.button("Outside action");
                if response.clicked() {
                    self.outside += 1;
                }
                response.into()
            }
            Control::OutsideCount => ui.label(format!("Count: {}", self.outside)).into(),
            Control::Instance => unreachable!("Select instances use live registrations"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, Gallery};
    use egui::accesskit::{Action, ActionRequest, NodeId, TreeId};
    use std::{collections::HashMap, time::Duration};

    /// A real native Host, with all input dispatched by production Gallery::ui.
    /// These tests do not substitute a collector or another UI event pipeline.
    struct NativeGallery {
        context: egui::Context,
        device: wgpu::Device,
        gallery: Gallery,
        time: f64,
        responses: HashMap<egui::Id, egui::Response>,
    }

    impl NativeGallery {
        fn new() -> Self {
            let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
            descriptor.backends = wgpu::Backends::PRIMARY;
            let instance = wgpu::Instance::new(descriptor);
            let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
                .expect("native adapter");
            let (device, _) =
                pollster::block_on(adapter.request_device(&Default::default())).unwrap();
            let context = egui::Context::default();
            context.enable_accesskit();
            context.options_mut(|options| options.max_passes = std::num::NonZeroUsize::MIN);
            let gallery = Gallery::new(&context, &device).unwrap();
            Self {
                context,
                device,
                gallery,
                time: 0.0,
                responses: HashMap::new(),
            }
        }

        fn run(&mut self, elapsed: f64, events: Vec<egui::Event>) -> egui::FullOutput {
            self.time += elapsed;
            self.context.run_ui(
                egui::RawInput {
                    // Match the Gallery's default native viewport, including
                    // enough body space for the outside action to be clickable.
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(960.0, 900.0),
                    )),
                    time: Some(self.time),
                    events,
                    ..Default::default()
                },
                |ui| {
                    self.gallery.ui(ui, &self.device);

                    // end_pass swaps this_pass and prev_pass. Read responses
                    // here, while current registrations still take precedence,
                    // rather than querying the Context after run_ui returns.
                    // This storage belongs only to the test wrapper.
                    self.responses.clear();
                    if self.gallery.case == Case::Selects {
                        for index in 0..3 {
                            let trigger = instance_id(index);
                            let popup_key =
                                trigger.with(("egui-prism/select", egui::ViewportId::ROOT));
                            for id in [
                                trigger,
                                popup_key.with("popup-focus"),
                                popup_key.with("scroll-up"),
                                popup_key.with("scroll-down"),
                            ] {
                                if let Some(response) = ui.ctx().read_response(id) {
                                    self.responses.insert(id, response);
                                }
                            }
                        }
                    }
                },
            )
        }

        fn send(&mut self, events: Vec<egui::Event>) {
            self.run(0.02, events).drop_without_applying_deltas();
        }

        // Only query registrations known to be live; read_response can fall
        // back to the previous pass for an absent registration.
        fn response(&self, id: egui::Id) -> &egui::Response {
            self.responses.get(&id).expect("captured live response")
        }

        fn rect(&self, id: egui::Id) -> egui::Rect {
            self.response(id).rect
        }

        fn modal(&self) -> Option<egui::LayerId> {
            self.context.memory(|memory| memory.top_modal_layer())
        }

        fn assert_idle(&mut self) {
            let values = self.gallery.selects.values.clone();
            let changes = self.gallery.selects.changes;
            let outside = self.gallery.selects.outside;
            // Allow animation and already requested repaint opportunities to
            // expire, then inspect the viewport's actual scheduling output.
            for _ in 0..3 {
                self.run(1.0, Vec::new()).drop_without_applying_deltas();
            }
            let output = self.run(1.0, Vec::new());
            let repaint_delay = output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .map(|viewport| viewport.repaint_delay);
            output.drop_without_applying_deltas();
            assert_eq!(repaint_delay, Some(Duration::MAX));
            assert_eq!(self.gallery.selects.values, values);
            assert_eq!(self.gallery.selects.changes, changes);
            assert_eq!(self.gallery.selects.outside, outside);
            assert_eq!(self.modal(), None);
        }
    }

    // Extract fallible output facts without panicking. Callers dispose of the
    // texture deltas before requiring those facts or asserting their values.
    fn node(output: &egui::FullOutput, label: &str) -> Option<NodeId> {
        output
            .platform_output
            .accesskit_update
            .as_ref()?
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .map(|(id, _)| *id)
    }

    fn bounds(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
        let bounds = output
            .platform_output
            .accesskit_update
            .as_ref()?
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))?
            .1
            .bounds()?;
        Some(egui::Rect::from_min_max(
            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
        ))
    }

    fn action(target_node: NodeId, action: Action) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        })
    }

    fn activate(index: usize) -> egui::Event {
        action(instance_id(index).accesskit_id(), Action::Click)
    }

    fn key(key: egui::Key) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }

    fn click(pos: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }

    fn wheel(delta: f32) -> egui::Event {
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, delta),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Wheel,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    #[ignore = "requires a native wgpu adapter"]
    fn production_navigation_values_and_current_batch_retirement() {
        let mut h = NativeGallery::new();
        let output = h.run(0.0, Vec::new());
        let selects = node(&output, "Selects");
        let theme = node(&output, "Theme");
        output.drop_without_applying_deltas();
        let selects = selects.expect("accessible Selects tab");
        let theme = theme.expect("accessible Theme tab");

        // The trigger does not exist at the start of this batch. Navigation
        // must register it before the very next event can open and commit it.
        let output = h.run(
            0.02,
            vec![
                action(selects, Action::Click),
                activate(0),
                key(egui::Key::ArrowDown),
                key(egui::Key::Enter),
            ],
        );
        let reverse = node(&output, "Reverse");
        let disabled = node(&output, "Disabled");
        let submits_drawing = output
            .shapes
            .iter()
            .any(|shape| matches!(shape.shape, egui::Shape::Callback(_)));
        output.drop_without_applying_deltas();
        let reverse = reverse.expect("accessible Reverse control");
        let disabled = disabled.expect("accessible Disabled control");
        assert_eq!(h.gallery.case, Case::Selects);
        assert_eq!(h.gallery.selects.values[0].as_deref(), Some("apricot"));
        assert_eq!(h.gallery.selects.values[1].as_deref(), Some("cherry"));
        assert_eq!(h.gallery.selects.changes, [1, 0]);
        assert_eq!(
            h.context.memory(|memory| memory.focused()),
            Some(instance_id(0)),
        );
        assert!(submits_drawing);

        h.send(vec![
            activate(1),
            key(egui::Key::Home),
            key(egui::Key::Enter),
            activate(0),
            key(egui::Key::Enter),
        ]);
        assert_eq!(h.gallery.selects.changes, [1, 1]);
        assert_eq!(h.gallery.selects.values[1].as_deref(), Some("apple"));
        h.send(vec![action(reverse, Action::Click)]);
        assert!(h.gallery.selects.reversed);
        assert_eq!(h.gallery.selects.values[0].as_deref(), Some("apricot"));
        assert_eq!(h.gallery.selects.values[1].as_deref(), Some("apple"));
        assert!(h.rect(instance_id(1)).top() < h.rect(instance_id(0)).top());
        h.send(vec![
            activate(0),
            key(egui::Key::End),
            key(egui::Key::Enter),
        ]);
        assert_eq!(h.gallery.selects.values[0].as_deref(), Some("cherry"));
        assert_eq!(h.gallery.selects.changes, [2, 1]);
        h.send(vec![activate(2)]);
        assert_eq!(h.modal(), None);

        // A configuration change replaces live geometry before the attempted
        // activation of the now-disabled control in this same batch.
        h.send(vec![
            action(disabled, Action::Click),
            activate(0),
            egui::Event::Text("a".into()),
        ]);
        assert!(h.gallery.selects.disabled);
        assert!(!h.response(instance_id(0)).enabled());
        assert_eq!(h.gallery.selects.changes, [2, 1]);
        assert_eq!(h.modal(), None);

        h.send(vec![
            action(disabled, Action::Click),
            activate(0),
            key(egui::Key::Escape),
            action(theme, Action::Click),
            activate(0),
            egui::Event::Text("a".into()),
        ]);
        assert_eq!(h.gallery.case, Case::Theme);
        assert_eq!(h.gallery.selects.changes, [2, 1]);
        assert_eq!(h.modal(), None);
        h.send(vec![
            action(selects, Action::Click),
            activate(0),
            key(egui::Key::Escape),
        ]);
        assert_eq!(
            h.context.memory(|memory| memory.focused()),
            Some(instance_id(0)),
        );
        h.assert_idle();
    }

    #[test]
    #[ignore = "requires a native wgpu adapter"]
    fn production_pointer_open_survives_idle_and_split_press_release() {
        for split_release in [false, true] {
            let mut h = NativeGallery::new();
            h.gallery.case = Case::Selects;
            h.send(Vec::new());
            let trigger = h.rect(instance_id(0)).center();
            let mut events = click(trigger);
            let release = events.pop().expect("pointer release");
            if split_release {
                h.send(events);
                assert!(h.modal().is_some(), "primary press must open the popup");
                h.run(0.02, Vec::new()).drop_without_applying_deltas();
                assert!(h.modal().is_some(), "held press must survive an idle frame");
                h.send(vec![release]);
            } else {
                events.push(release);
                h.send(events);
            }
            assert!(
                h.modal().is_some(),
                "opening release must not dismiss the popup"
            );
            h.run(0.3, Vec::new()).drop_without_applying_deltas();
            assert!(
                h.modal().is_some(),
                "popup must remain open after animation"
            );
            assert_eq!(h.gallery.selects.values[0], None);
            assert_eq!(h.gallery.selects.changes, [0, 0]);
            h.send(vec![key(egui::Key::ArrowDown), key(egui::Key::Enter)]);
            assert_eq!(h.gallery.selects.values[0].as_deref(), Some("apricot"));
            assert_eq!(h.gallery.selects.changes, [1, 0]);
            assert_eq!(h.modal(), None);
            h.assert_idle();
        }
    }

    #[test]
    #[ignore = "requires a native wgpu adapter"]
    fn production_popup_shields_outside_action_and_reopens() {
        let mut h = NativeGallery::new();
        h.gallery.case = Case::Selects;
        let output = h.run(0.0, Vec::new());
        let outside = bounds(&output, "Outside action");
        output.drop_without_applying_deltas();
        let outside = outside.expect("accessible Outside action bounds").center();
        let trigger = h.rect(instance_id(0));
        h.send(click(trigger.center()));
        assert!(h.modal().is_some());
        h.send(vec![egui::Event::PointerMoved(outside), wheel(-120.0)]);
        assert!(h.modal().is_some());
        assert_eq!(h.rect(instance_id(0)), trigger);
        h.send(click(outside));
        assert_eq!(h.modal(), None);
        assert_eq!(h.gallery.selects.outside, 0);
        h.send(click(outside));
        assert_eq!(h.gallery.selects.outside, 1);
        assert_eq!(h.gallery.selects.changes, [0, 0]);

        h.send(vec![
            action(instance_id(0).accesskit_id(), Action::Focus),
            key(egui::Key::Enter),
            key(egui::Key::Escape),
        ]);
        assert_eq!(
            h.context.memory(|memory| memory.focused()),
            Some(instance_id(0)),
        );
        // Register and final Paint must not replay the outside click or any
        // selection action, and dismissal must not leave a repaint loop.
        h.assert_idle();
        assert_eq!(h.gallery.selects.outside, 1);
    }

    #[test]
    #[ignore = "requires a native wgpu adapter"]
    fn production_frame_local_variants_and_indicator_timer_cancellation() {
        let mut h = NativeGallery::new();
        h.gallery.case = Case::Selects;
        let output = h.run(0.0, Vec::new());
        let long = node(&output, "Long list");
        let small = node(&output, "Small");
        let invalid = node(&output, "Invalid");
        let tinted = node(&output, "Tinted");
        let dark = node(&output, "Dark");
        let reset = node(&output, "Reset values");
        let empty = node(&output, "Empty");
        let all_disabled = node(&output, "All disabled");
        output.drop_without_applying_deltas();
        let long = long.expect("accessible Long list control");
        let small = small.expect("accessible Small control");
        let invalid = invalid.expect("accessible Invalid control");
        let tinted = tinted.expect("accessible Tinted control");
        let dark = dark.expect("accessible Dark control");
        let reset = reset.expect("accessible Reset values control");
        let empty = empty.expect("accessible Empty control");
        let all_disabled = all_disabled.expect("accessible All disabled control");
        h.send(vec![
            action(long, Action::Click),
            action(small, Action::Click),
            action(invalid, Action::Click),
            action(tinted, Action::Click),
            action(dark, Action::Click),
            action(reset, Action::Click),
            activate(1),
        ]);
        assert!(h.gallery.dark);
        assert!(h.gallery.selects.small);
        assert!(h.gallery.selects.long);
        assert!(h.gallery.selects.tinted && h.gallery.selects.invalid);
        assert_eq!(h.rect(instance_id(1)).height(), 32.0);
        assert_eq!(h.gallery.selects.values[1].as_deref(), Some("option-30"));
        assert_eq!(h.gallery.selects.changes, [0, 0]);
        assert!(h.modal().is_some());
        let popup_key = instance_id(1).with(("egui-prism/select", egui::ViewportId::ROOT));
        let popup = popup_key.with("popup-focus");
        let up = popup_key.with("scroll-up");
        let before = h.rect(popup);
        let indicator = h.rect(up).center();
        assert!(before.contains(indicator));
        let output = h.run(0.01, vec![egui::Event::PointerMoved(indicator)]);
        let repaint_delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay);
        output.drop_without_applying_deltas();
        assert!(repaint_delay.expect("root viewport") <= Duration::from_millis(50));
        let hovered = h.rect(popup);

        let output = h.run(0.06, Vec::new());
        let repaint_delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay);
        output.drop_without_applying_deltas();
        let repeated = h.rect(popup);
        // Compare current registrations rather than assuming a fixed popup
        // height for a particular viewport. Scrolling up from option 30 reveals
        // earlier rows and expands this popup. The time-only frame must make
        // progress beyond the initial hover, not merely schedule a repaint.
        assert!(repeated.height() > before.height());
        assert!(repeated.height() > hovered.height());
        assert!(repaint_delay.expect("root viewport") <= Duration::from_millis(50));
        assert_eq!(h.gallery.selects.values[0], None);
        assert_eq!(h.gallery.selects.values[1].as_deref(), Some("option-30"));
        assert_eq!(h.gallery.selects.changes, [0, 0]);
        h.send(vec![egui::Event::PointerGone, key(egui::Key::Escape)]);
        h.assert_idle();

        // Empty and all-disabled variants are selected within one input batch;
        // they borrow different prebuilt slices and cannot commit a value.
        h.send(vec![
            action(empty, Action::Click),
            activate(0),
            key(egui::Key::Enter),
            key(egui::Key::Escape),
            action(empty, Action::Click),
            action(all_disabled, Action::Click),
            activate(0),
            key(egui::Key::End),
            key(egui::Key::Enter),
            key(egui::Key::Escape),
            action(all_disabled, Action::Click),
            activate(1),
        ]);
        assert_eq!(h.gallery.selects.values[0], None);
        assert_eq!(h.gallery.selects.changes, [0, 0]);
        assert!(h.modal().is_some());
        let indicator = h.rect(up).center();
        h.send(vec![egui::Event::PointerMoved(indicator)]);
        h.send(vec![egui::Event::WindowFocused(false)]);
        assert_eq!(h.modal(), None);
        h.assert_idle();
    }
}
