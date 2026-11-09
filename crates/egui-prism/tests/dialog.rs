//! Ordered-context tests only: these do not assert GPU, native integration, or
//! browser-reference acceptance.

use std::sync::{Arc, Mutex};

use compositing::{Backend, Error, Group, Item};
use egui::{
    Color32, Context, Event, EventFilter, FontFamily, Id, InputTrigger, Key, LayerId, Modifiers,
    MouseWheelUnit, Order, PaintCallback, PointerButton, RawInput, Rect, Response, Sense, Shape,
    Ui, UiBuilder, Vec2, emath::TSTransform, epaint::ClippedShape, pos2, vec2,
};
use egui_prism::{compositing, dialog, theme};
use theme::Theme;

use dialog::{Dialog, DialogClose, DialogUi};

#[derive(Default)]
struct Collector(Mutex<Vec<Group>>);

impl Backend for Collector {
    fn callback(&self, _: &Context, group: Group) -> Result<PaintCallback, Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}

fn setup() -> (Context, Arc<Collector>, compositing::Installation) {
    let context = Context::default();
    context.options_mut(|options| {
        options.ordered_input = true;
        options.max_passes = 1.try_into().unwrap();
    });
    let collector = Arc::new(Collector::default());
    let installation = compositing::install(&context, collector.clone()).unwrap();
    (context, collector, installation)
}

fn frame(
    context: &Context,
    time: f64,
    size: Vec2,
    events: Vec<Event>,
    mut run: impl FnMut(&Context),
) -> Vec<ClippedShape> {
    let mut output = context.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), size)),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ui| run(ui.ctx()),
    );
    let shapes = std::mem::take(&mut output.shapes);
    output.drop_without_applying_deltas();
    shapes
}

fn key(key: Key, pressed: bool, shift: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers {
            shift,
            ..Modifiers::NONE
        },
    }
}

fn click(point: egui::Pos2, button: PointerButton) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos: point,
            button,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ]
}

fn background(context: &Context, id: Id, enabled: bool) -> Response {
    context.input_layout(|| {
        let layer = LayerId::new(Order::Middle, Id::unique("background-layer"));
        let mut ui = Ui::new(
            context.clone(),
            Id::unique("background-ui"),
            UiBuilder::new()
                .layer_id(layer)
                .max_rect(context.content_rect()),
        );
        if !enabled {
            ui.disable();
        }
        let response = ui.interact(
            Rect::from_min_size(pos2(20.0, 20.0), vec2(100.0, 32.0)),
            id,
            Sense::click(),
        );
        if enabled {
            context.memory_mut(|memory| {
                memory.register_focus_policy(id, layer, EventFilter::default());
            });
        }
        response
    })
}

fn drawing(rect: Rect, color: Color32) -> Group {
    Group::new(
        rect.expand(3.0),
        1.0,
        vec![Item::Shape(ClippedShape {
            clip_rect: rect.expand(3.0),
            shape: Shape::rect_filled(rect, 3.0, color),
            transform_after_tessellation: TSTransform::IDENTITY,
        })],
    )
    .unwrap()
}

/// A deliberately small egui-only control. Business handling remains outside
/// the callback, just as it does for a returned Prism ButtonOutput.
fn child(dialog: &mut DialogUi<'_>, id: Id, height: f32) -> Response {
    let ui = dialog.ui();
    let (rect, _) = ui.allocate_exact_size(vec2(100.0, height), Sense::hover());
    let response = ui.interact(rect, id, Sense::click());
    if response.enabled() {
        ui.ctx().memory_mut(|memory| {
            memory.register_focus_policy(id, ui.layer_id(), EventFilter::default());
        });
    }
    dialog.focusable(&response);
    dialog.draw(drawing(rect, Color32::from_rgb(20, 90, 160)));
    response
}

fn drain(context: &Context, registration: &mut dialog::DialogRegistration<'_, Response>) {
    while let Some(event) = context.next_input_event() {
        let intent = registration.handle(&event);
        assert!(!intent.layout_changed);
    }
}

fn content_group(root: &Group) -> &Group {
    match &root.items()[1] {
        Item::Group(group) => group,
        _ => panic!("content must remain an isolated group"),
    }
}

fn all_shapes<'a>(group: &'a Group, output: &mut Vec<&'a ClippedShape>) {
    for item in group.items() {
        match item {
            Item::Shape(shape) => output.push(shape),
            Item::Group(child) => all_shapes(child, output),
        }
    }
}

#[test]
fn same_batch_open_blocks_later_background_actions_and_ignores_right_click() {
    let (context, collector, _installation) = setup();
    let theme = Theme::light();
    let opener = Id::unique("opener");
    let body = Id::unique("body");
    let mut events = vec![key(Key::Enter, true, false), key(Key::Enter, false, false)];
    events.extend(click(pos2(40.0, 40.0), PointerButton::Secondary));
    events.extend(click(pos2(40.0, 40.0), PointerButton::Primary));

    let mut open = false;
    let mut background_clicks = 0;
    let mut requests = Vec::new();

    frame(&context, 0.0, vec2(800.0, 600.0), events, |context| {
        context.begin_input();
        let trigger = background(context, opener, true);
        trigger.request_focus();

        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title")
                .register(context, open, |ui| child(ui, body, 32.0));

        while let Some(event) = context.next_input_event() {
            let intent = registration.handle(&event);
            if let Some(close) = intent.close_requested {
                requests.push(close); // Deliberately decline.
            }
            if event.target == Some(opener) {
                if matches!(
                    event.trigger,
                    InputTrigger::Event {
                        event: Event::Key {
                            key: Key::Enter,
                            pressed: true,
                            ..
                        },
                        ..
                    }
                ) {
                    open = true;
                    registration.refresh(open, |ui| child(ui, body, 32.0));
                }
                if context.read_response(opener).unwrap().clicked() {
                    background_clicks += 1;
                }
            }
        }

        assert!(open);
        assert_eq!(context.memory(|memory| memory.focused()), Some(body));
        registration.paint(|ui| child(ui, body, 32.0)).unwrap();
    });

    assert_eq!(background_clicks, 0);
    assert_eq!(requests, vec![DialogClose::Backdrop]);
    assert!(!collector.0.lock().unwrap().is_empty());
}

#[test]
fn accepted_escape_keeps_exit_barrier_then_restores_live_opener() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let opener = Id::unique("opener");
    let body = Id::unique("body");
    let dialog_id = Id::unique("dialog");
    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        background(context, opener, true).request_focus();
        let mut dialog = Dialog::new(&theme, dialog_id, "Title")
            .register(context, true, |ui| child(ui, body, 32.0));
        drain(context, &mut dialog);
        dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
    let mut events = vec![key(Key::Escape, true, false), key(Key::Enter, true, false)];
    events.extend(click(pos2(40.0, 40.0), PointerButton::Primary));
    frame(&context, 0.3, vec2(800.0, 600.0), events, |context| {
        context.begin_input();
        background(context, opener, true);
        let mut dialog = Dialog::new(&theme, dialog_id, "Title")
            .register(context, true, |ui| child(ui, body, 32.0));
        let mut closes = 0;
        while let Some(event) = context.next_input_event() {
            assert_ne!(event.target, Some(opener));
            if dialog.handle(&event).close_requested == Some(DialogClose::Escape) {
                closes += 1;
                dialog.refresh(false, |ui| child(ui, body, 32.0));
                assert!(dialog.inner.is_none());
                assert_ne!(context.memory(|m| m.focused()), Some(opener));
                assert!(context.memory(|m| m.top_modal_layer().is_some()));
            }
        }
        assert_eq!(closes, 1);
        dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
    for (time, exiting) in [(0.46, true), (0.51, false)] {
        frame(&context, time, vec2(800.0, 600.0), vec![], |context| {
            context.begin_input();
            background(context, opener, true);
            let mut dialog =
                Dialog::new(&theme, dialog_id, "Title")
                    .register(context, false, |ui| child(ui, body, 32.0));
            assert_eq!(context.memory(|m| m.top_modal_layer().is_some()), exiting);
            if !exiting {
                assert_eq!(context.memory(|m| m.focused()), Some(opener));
            }
            drain(context, &mut dialog);
            dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
        });
    }
}

#[test]
fn programmatic_reopen_during_exit_preserves_original_opener() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let opener = Id::unique("opener");
    let body = Id::unique("body");
    let id = Id::unique("dialog");
    for time in [0.0, 0.3, 0.6] {
        frame(&context, time, vec2(800.0, 600.0), vec![], |context| {
            context.begin_input();
            let response = background(context, opener, true);
            if time == 0.0 {
                response.request_focus();
            }
            let mut dialog = Dialog::new(&theme, id, "Title")
                .register(context, true, |ui| child(ui, body, 32.0));
            if time == 0.3 {
                dialog.refresh(false, |ui| child(ui, body, 32.0));
                dialog.refresh(true, |ui| child(ui, body, 32.0));
                assert_eq!(context.memory(|m| m.focused()), Some(body));
            } else if time == 0.6 {
                dialog.refresh(false, |ui| child(ui, body, 32.0));
            }
            drain(context, &mut dialog);
            dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
        });
    }
    frame(&context, 0.81, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        background(context, opener, true);
        let mut dialog =
            Dialog::new(&theme, id, "Title").register(context, false, |ui| child(ui, body, 32.0));
        assert_eq!(context.memory(|m| m.focused()), Some(opener));
        drain(context, &mut dialog);
        dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
}

#[test]
fn ignored_escape_keeps_barrier_and_focus() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let body = Id::unique("body");
    let opener = Id::unique("opener");
    let mut events = vec![
        key(Key::Escape, true, false),
        key(Key::Escape, false, false),
    ];
    events.extend(click(pos2(40.0, 40.0), PointerButton::Primary));

    frame(&context, 0.0, vec2(800.0, 600.0), events, |context| {
        context.begin_input();
        background(context, opener, true).request_focus();
        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title")
                .register(context, true, |ui| child(ui, body, 32.0));
        let mut escape = false;

        while let Some(event) = context.next_input_event() {
            let intent = registration.handle(&event);
            escape |= intent.close_requested == Some(DialogClose::Escape);
            assert_ne!(event.target, Some(opener));
        }

        assert!(escape);
        assert_eq!(context.memory(|memory| memory.focused()), Some(body));
        assert!(context.memory(|memory| memory.top_modal_layer().is_some()));
        registration.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
}

#[test]
fn independent_instances_survive_registration_reordering() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let a = Id::unique("a");
    let b = Id::unique("b");
    let a_child = Id::unique("a-child");
    let b_child = Id::unique("b-child");

    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut lower =
            Dialog::new(&theme, a, "A").register(context, true, |ui| child(ui, a_child, 32.0));
        drain(context, &mut lower);
        lower.paint(|ui| child(ui, a_child, 32.0)).unwrap();
    });

    frame(&context, 0.1, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut lower =
            Dialog::new(&theme, a, "A").register(context, true, |ui| child(ui, a_child, 32.0));
        let mut upper =
            Dialog::new(&theme, b, "B").register(context, true, |ui| child(ui, b_child, 32.0));
        while let Some(event) = context.next_input_event() {
            lower.handle(&event);
            upper.handle(&event);
        }
        assert_eq!(context.memory(|memory| memory.focused()), Some(b_child));
        lower.paint(|ui| child(ui, a_child, 32.0)).unwrap();
        upper.paint(|ui| child(ui, b_child, 32.0)).unwrap();
    });

    frame(
        &context,
        0.2,
        vec2(800.0, 600.0),
        vec![key(Key::Escape, true, false)],
        |context| {
            context.begin_input();
            // Reverse registration order without changing instance identities.
            let mut upper =
                Dialog::new(&theme, b, "B").register(context, true, |ui| child(ui, b_child, 32.0));
            let mut lower =
                Dialog::new(&theme, a, "A").register(context, true, |ui| child(ui, a_child, 32.0));

            while let Some(event) = context.next_input_event() {
                assert_eq!(lower.handle(&event).close_requested, None);
                if upper.handle(&event).close_requested == Some(DialogClose::Escape) {
                    upper.refresh(false, |ui| child(ui, b_child, 32.0));
                    assert_ne!(context.memory(|memory| memory.focused()), Some(a_child));
                    // Offering that same Escape again must not cascade.
                    assert_eq!(lower.handle(&event).close_requested, None);
                }
            }
            lower.paint(|ui| child(ui, a_child, 32.0)).unwrap();
            upper.paint(|ui| child(ui, b_child, 32.0)).unwrap();
        },
    );
}

#[test]
fn modal_dispatcher_cycles_body_then_x_in_both_directions() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let a = Id::unique("first");
    let b = Id::unique("second");
    let content = |ui: &mut DialogUi<'_>| {
        let first = child(ui, a, 28.0);
        child(ui, b, 28.0);
        first
    };
    let mut events = Vec::new();
    for shift in [false, false, false, true] {
        events.push(key(Key::Tab, true, shift));
        events.push(key(Key::Tab, false, shift));
    }

    frame(&context, 0.0, vec2(800.0, 600.0), events, |context| {
        context.begin_input();
        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title").register(context, true, content);
        assert_eq!(context.memory(|memory| memory.focused()), Some(a));

        let mut focus = Vec::new();
        while let Some(event) = context.next_input_event() {
            registration.handle(&event);
            if matches!(
                event.trigger,
                InputTrigger::Event {
                    event: Event::Key {
                        key: Key::Tab,
                        pressed: true,
                        ..
                    },
                    ..
                }
            ) {
                focus.push(context.memory(|memory| memory.focused()).unwrap());
            }
        }

        assert_eq!(focus.len(), 4);
        assert_eq!(focus[0], b);
        assert_ne!(focus[1], a);
        assert_ne!(focus[1], b); // X
        assert_eq!(focus[2], a);
        assert_eq!(focus[3], focus[1]);
        registration.paint(content).unwrap();
    });
}

#[test]
fn long_body_scrolls_and_clips_hits_in_a_small_viewport() {
    let (context, collector, _installation) = setup();
    let theme = Theme::light();
    let content = |ui: &mut DialogUi<'_>| {
        (0..12)
            .map(|index| child(ui, Id::unique(("row", index)), 32.0))
            .collect::<Vec<_>>()
    };

    frame(&context, 0.0, vec2(320.0, 240.0), vec![], |context| {
        context.begin_input();
        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title").register(context, true, content);
        while let Some(event) = context.next_input_event() {
            registration.handle(&event);
        }
        registration.paint(content).unwrap();
    });

    let mut old_first = 0.0;
    let mut new_first = 0.0;
    frame(
        &context,
        0.3,
        vec2(320.0, 240.0),
        vec![
            Event::PointerMoved(pos2(160.0, 140.0)),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: vec2(0.0, -1000.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
                modifiers: Modifiers::NONE,
            },
        ],
        |context| {
            context.begin_input();
            let mut registration =
                Dialog::new(&theme, Id::unique("dialog"), "Title").register(context, true, content);
            old_first = registration.inner.as_ref().unwrap()[0].rect.top();

            while let Some(event) = context.next_input_event() {
                if registration.handle(&event).layout_changed {
                    registration.refresh(true, content);
                }
            }

            let rows = registration.inner.as_ref().unwrap();
            new_first = rows[0].rect.top();
            assert!(!rows[0].interact_rect.is_positive());
            assert!(rows.last().unwrap().interact_rect.is_positive());
            registration.paint(content).unwrap();
        },
    );
    assert!(new_first < old_first - 100.0);

    let groups = collector.0.lock().unwrap();
    let content = content_group(groups.last().unwrap());
    let mut shapes = Vec::new();
    all_shapes(content, &mut shapes);
    let panel = shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Rect(rect) if rect.fill == theme.colors.background.background => Some(rect.rect),
            _ => None,
        })
        .unwrap();
    assert!((panel.width() - 288.0).abs() < 0.01);
    assert!(panel.height() <= 208.01);
    assert!(panel.left() >= 15.99 && panel.top() >= 15.99);
    assert!(content.bounds().width() > panel.width()); // Unclipped shadow extent.
}

#[test]
fn nested_groups_use_host_time_and_exit_reconstructs_caller_data() {
    let (context, collector, _installation) = setup();
    let theme = Theme::light();
    let id = Id::unique("dialog");
    let mut calls = 0usize;
    let mut callbacks = Vec::new();

    for (time, open, color) in [
        (0.0, true, Color32::RED),
        (0.1, true, Color32::GREEN),
        (0.2, false, Color32::BLUE),
        (0.3, false, Color32::YELLOW),
    ] {
        let output = frame(&context, time, vec2(800.0, 600.0), vec![], |context| {
            context.begin_input();
            let mut content = |ui: &mut DialogUi<'_>| {
                calls += 1;
                let (rect, _) = ui
                    .ui()
                    .allocate_exact_size(vec2(100.0, 32.0), Sense::hover());
                let leaf = drawing(rect, color);
                let nested = Group::new(leaf.bounds(), 0.6, vec![Item::Group(leaf)]).unwrap();
                ui.draw(nested);
            };
            let mut registration =
                Dialog::new(&theme, id, "Title").register(context, open, &mut content);
            // Multiple refreshes at the same host timestamp must not advance time.
            registration.refresh(open, &mut content);
            while let Some(event) = context.next_input_event() {
                registration.handle(&event);
            }
            registration.paint(&mut content).unwrap();
        });
        callbacks.push(
            output
                .into_iter()
                .find_map(|shape| {
                    if let Shape::Callback(callback) = shape.shape {
                        Some(callback.rect)
                    } else {
                        None
                    }
                })
                .expect("compositor callback"),
        );
    }

    assert!(calls > 4 && calls < 40);
    let groups = collector.0.lock().unwrap();
    assert_eq!(groups.len(), 4);
    // CSS cubic-bezier(0.25, 0.1, 0.25, 1) at half duration.
    let expected = [0.0, 0.8024, 1.0, 0.1976];
    for ((root, callback), expected_opacity) in groups.iter().zip(callbacks).zip(expected) {
        let content = content_group(root);
        assert!((content.opacity() - expected_opacity).abs() < 0.001);
        assert!(content.items().iter().any(|item| {
            matches!(item, Item::Group(group)
                if (group.opacity() - 0.6).abs() < 0.001
                    && matches!(group.items().first(), Some(Item::Group(_))))
        }));
        let mut shapes = Vec::new();
        all_shapes(content, &mut shapes);
        let expected_scale = 0.95 + 0.05 * expected_opacity;
        // Payload stays in local coordinates; the actual emitted callback carries
        // the layer transform consumed by the renderer exactly once.
        assert!(
            shapes
                .iter()
                .all(|shape| shape.transform_after_tessellation == TSTransform::IDENTITY)
        );
        assert!((callback.width() / root.bounds().width() - expected_scale).abs() < 0.001);
        let transform = TSTransform::new(
            callback.min.to_vec2() - root.bounds().min.to_vec2() * expected_scale,
            expected_scale,
        );
        let Item::Group(overlay) = &root.items()[0] else {
            panic!("overlay group")
        };
        assert!(((transform * overlay.bounds()).min - pos2(0.0, 0.0)).length() < 0.01);
        assert!(((transform * overlay.bounds()).size() - vec2(800.0, 600.0)).length() < 0.01);
    }

    let mut exit_shapes = Vec::new();
    all_shapes(content_group(groups.last().unwrap()), &mut exit_shapes);
    assert!(exit_shapes.iter().any(|shape| {
        matches!(&shape.shape, Shape::Rect(rect) if rect.fill == Color32::YELLOW)
    }));
}

#[test]
fn unregister_revokes_barrier_and_cleans_opener_state() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let id = Id::unique("dialog");
    let opener = Id::unique("opener");
    let body = Id::unique("body");

    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        background(context, opener, true).request_focus();
        let registration =
            Dialog::new(&theme, id, "Title").register(context, true, |ui| child(ui, body, 32.0));
        registration.unregister();
        assert!(context.memory(|memory| memory.top_modal_layer().is_none()));
        assert_ne!(context.memory(|memory| memory.focused()), Some(opener));

        // A new same-ID registration must capture this new focus, not stale state.
        let second = Id::unique("second-opener");
        background(context, second, true).request_focus();
        let mut registration =
            Dialog::new(&theme, id, "Title").register(context, true, |ui| child(ui, body, 32.0));
        registration.refresh(false, |ui| child(ui, body, 32.0));
        assert!(context.memory(|memory| memory.top_modal_layer().is_some()));
        assert_ne!(context.memory(|memory| memory.focused()), Some(second));
        while let Some(event) = context.next_input_event() {
            registration.handle(&event);
        }
        registration.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
    frame(&context, 0.201, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let second = Id::unique("second-opener");
        background(context, second, true);
        let mut registration =
            Dialog::new(&theme, id, "Title").register(context, false, |ui| child(ui, body, 32.0));
        assert_eq!(context.memory(|memory| memory.focused()), Some(second));
        assert!(context.memory(|memory| memory.top_modal_layer().is_none()));
        drain(context, &mut registration);
        registration.paint(|ui| child(ui, body, 32.0)).unwrap();
    });
}

#[test]
fn restoration_rejects_disabled_and_removed_targets() {
    for remove in [false, true] {
        let (context, _, _installation) = setup();
        let theme = Theme::light();
        let opener = Id::unique("opener");
        let body = Id::unique("body");

        frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
            context.begin_input();
            background(context, opener, true).request_focus();
            let mut registration = Dialog::new(&theme, Id::unique("dialog"), "Title")
                .restore_focus(opener)
                .register(context, true, |ui| child(ui, body, 32.0));

            if remove {
                context.remove_input_widget(opener);
            } else {
                background(context, opener, false);
            }
            registration.refresh(false, |ui| child(ui, body, 32.0));
            assert_ne!(context.memory(|memory| memory.focused()), Some(opener));
            while let Some(event) = context.next_input_event() {
                registration.handle(&event);
            }
            registration.paint(|ui| child(ui, body, 32.0)).unwrap();
        });
    }
}

#[test]
fn missing_compositor_and_paint_time_geometry_changes_are_errors() {
    let context = Context::default();
    context.options_mut(|options| {
        options.ordered_input = true;
        options.max_passes = 1.try_into().unwrap();
    });
    let theme = Theme::light();

    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title")
                .register(context, true, |ui| child(ui, Id::unique("body"), 32.0));
        drain(context, &mut registration);
        assert_eq!(
            registration.paint(|ui| child(ui, Id::unique("body"), 32.0)),
            Err(Error::MissingHost)
        );
    });

    let (context, _, _installation) = setup();
    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut registration =
            Dialog::new(&theme, Id::unique("dialog"), "Title")
                .register(context, true, |ui| child(ui, Id::unique("body"), 32.0));
        drain(context, &mut registration);
        assert!(matches!(
            registration.paint(|ui| child(ui, Id::unique("body"), 80.0)),
            Err(Error::InvalidGroup(_))
        ));
    });
}

#[test]
fn default_font_weight_limitation_is_explicit() {
    let (context, collector, _installation) = setup();
    let theme = Theme::light();

    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut registration = Dialog::new(&theme, Id::unique("dialog"), "Title")
            .description("Description")
            .register(context, true, |_| ());
        while let Some(event) = context.next_input_event() {
            registration.handle(&event);
        }
        registration.paint(|_| ()).unwrap();
    });

    let groups = collector.0.lock().unwrap();
    let mut shapes = Vec::new();
    all_shapes(groups.last().unwrap(), &mut shapes);
    let title = shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Text(text) if text.galley.job.text == "Title" => Some(text),
            _ => None,
        })
        .unwrap();
    let format = &title.galley.job.sections[0].format;
    assert_eq!(format.font_id.size, 18.0);
    assert_eq!(format.line_height, Some(18.0));
    // This is not a claim that egui's default proportional font is semibold.
    assert_eq!(format.font_id.family, FontFamily::Proportional);
}

#[test]
fn resizing_below_header_height_does_not_panic_or_drop_the_barrier() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    for (index, size) in [vec2(800.0, 600.0), vec2(320.0, 80.0), vec2(64.0, 64.0)]
        .into_iter()
        .enumerate()
    {
        frame(&context, index as f64, size, vec![], |context| {
            context.begin_input();
            let mut registration = Dialog::new(&theme, Id::unique("dialog"), "Title")
                .description("An ordinary native window resize must remain safe.")
                .register(context, true, |_| ());
            assert!(context.memory(|memory| memory.top_modal_layer().is_some()));
            while let Some(event) = context.next_input_event() {
                registration.handle(&event);
            }
            registration.paint(|_| ()).unwrap();
        });
    }
}

#[test]
fn outside_pointer_down_dismisses_without_waiting_for_release_except_context_clicks() {
    for (button, ctrl, expected) in [
        (PointerButton::Primary, false, Some(DialogClose::Backdrop)),
        (PointerButton::Secondary, false, None),
        (PointerButton::Primary, true, None),
    ] {
        let (context, _, _installation) = setup();
        let theme = Theme::light();
        let body = Id::unique("body");
        let point = pos2(2.0, 2.0);
        let mut dismissals = Vec::new();
        frame(
            &context,
            0.0,
            vec2(800.0, 600.0),
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button,
                    pressed: true,
                    modifiers: Modifiers {
                        ctrl,
                        ..Modifiers::NONE
                    },
                },
            ],
            |context| {
                context.begin_input();
                let mut registration = Dialog::new(&theme, Id::unique("dialog"), "Title").register(
                    context,
                    true,
                    |ui| child(ui, body, 32.0),
                );
                while let Some(event) = context.next_input_event() {
                    if let Some(close) = registration.handle(&event).close_requested {
                        dismissals.push(close);
                    }
                }
                assert_eq!(context.memory(|memory| memory.focused()), Some(body));
                registration.paint(|ui| child(ui, body, 32.0)).unwrap();
            },
        );
        assert_eq!(dismissals, expected.into_iter().collect::<Vec<_>>());
    }
}

#[test]
fn refreshing_layout_preserves_focused_child_and_removes_vanished_targets() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let first = Id::unique("first");
    let second = Id::unique("second");
    let content = |ui: &mut DialogUi<'_>, both| {
        let response = child(ui, first, 32.0);
        if both {
            child(ui, second, 32.0);
        }
        response
    };
    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut registration = Dialog::new(&theme, Id::unique("dialog"), "Title")
            .initial_focus(second)
            .register(context, true, |ui| content(ui, true));
        assert_eq!(context.memory(|m| m.focused()), Some(second));
        registration.refresh(true, |ui| content(ui, true));
        assert_eq!(context.memory(|m| m.focused()), Some(second));
        registration.refresh(true, |ui| content(ui, false));
        assert_ne!(context.memory(|m| m.focused()), Some(second));
        assert!(context.read_response(second).is_none());
        drain(context, &mut registration);
        registration.paint(|ui| content(ui, false)).unwrap();
    });
}

#[test]
fn installed_title_font_and_narrow_header_alignment_match_reference_metrics() {
    let (context, collector, _installation) = setup();
    theme::install_fonts(&context, egui::FontDefinitions::default());
    let theme = Theme::light();
    frame(&context, 0.0, vec2(320.0, 240.0), vec![], |context| {
        context.begin_input();
        let mut dialog = Dialog::new(&theme, Id::unique("header"), "Title")
            .description("Description")
            .register(context, true, |_| ());
        while let Some(event) = context.next_input_event() {
            dialog.handle(&event);
        }
        dialog.paint(|_| ()).unwrap();
    });
    let groups = collector.0.lock().unwrap();
    let mut shapes = Vec::new();
    all_shapes(groups.last().unwrap(), &mut shapes);
    for text in shapes.iter().filter_map(|shape| match &shape.shape {
        Shape::Text(text) => Some(text),
        _ => None,
    }) {
        let center = text.pos.x + text.galley.rect.center().x;
        assert!(
            (center - 160.0).abs() < 0.1,
            "{} center={center}, pos={:?}, rect={:?}",
            text.galley.job.text,
            text.pos,
            text.galley.rect
        );
        if text.galley.job.text == "Title" {
            assert_eq!(
                text.galley.job.sections[0].format.font_id.family,
                FontFamily::Name("prism-geist-semibold".into())
            );
        }
    }
    assert!(shapes.iter().any(|shape| matches!(&shape.shape,
        Shape::Rect(rect) if rect.fill == theme.colors.background.background && rect.corner_radius == egui::CornerRadius::same(10))));
}

#[test]
fn reopening_long_body_resets_its_native_scroll_state() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let content = |ui: &mut DialogUi<'_>| {
        (0..20)
            .map(|i| child(ui, Id::unique(("row", i)), 32.0))
            .collect::<Vec<_>>()
    };
    frame(&context, 0.0, vec2(320.0, 240.0), vec![], |context| {
        context.begin_input();
        let mut dialog = Dialog::new(&theme, Id::unique("scroll-reopen"), "Title")
            .register(context, true, content);
        while let Some(event) = context.next_input_event() {
            dialog.handle(&event);
        }
        dialog.paint(content).unwrap();
    });
    frame(
        &context,
        0.3,
        vec2(320.0, 240.0),
        vec![
            Event::PointerMoved(pos2(160.0, 140.0)),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: vec2(0.0, -1000.0),
                phase: egui::TouchPhase::Move,
                source: egui::MouseWheelSource::Wheel,
                modifiers: Modifiers::NONE,
            },
        ],
        |context| {
            context.begin_input();
            let mut dialog = Dialog::new(&theme, Id::unique("scroll-reopen"), "Title")
                .register(context, true, content);
            let top = dialog.inner.as_ref().unwrap()[0].rect.top();
            while let Some(event) = context.next_input_event() {
                if dialog.handle(&event).layout_changed {
                    dialog.refresh(true, content);
                    assert!(dialog.inner.as_ref().unwrap()[0].rect.top() < top - 100.0);
                    dialog.refresh(false, content);
                    dialog.refresh(true, content);
                    assert!((dialog.inner.as_ref().unwrap()[0].rect.top() - top).abs() < 0.1);
                }
            }
            dialog.paint(content).unwrap();
        },
    );
}

#[test]
fn ordinary_body_paint_is_visible_above_shell_without_becoming_a_group_member() {
    let (context, collector, _installation) = setup();
    let theme = Theme::light();
    let content = |slot: &mut DialogUi<'_>| {
        let (rect, _) = slot
            .ui()
            .allocate_exact_size(vec2(40.0, 30.0), Sense::hover());
        slot.ui().painter().rect_filled(rect, 0.0, Color32::RED);
    };
    let shapes = frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut dialog =
            Dialog::new(&theme, Id::unique("ordinary"), "Title").register(context, true, content);
        while let Some(event) = context.next_input_event() {
            dialog.handle(&event);
        }
        dialog.paint(content).unwrap();
    });
    let callback = shapes
        .iter()
        .position(|s| matches!(s.shape, Shape::Callback(_)))
        .unwrap();
    let direct = shapes
        .iter()
        .position(|s| matches!(&s.shape, Shape::Rect(rect) if rect.fill == Color32::RED))
        .unwrap();
    assert!(
        callback < direct,
        "opaque shell must be below direct body paint"
    );
    let groups = collector.0.lock().unwrap();
    let mut explicit = Vec::new();
    all_shapes(groups.last().unwrap(), &mut explicit);
    assert!(
        !explicit
            .iter()
            .any(|s| matches!(&s.shape, Shape::Rect(rect) if rect.fill == Color32::RED))
    );
}

#[test]
fn reopening_exiting_lower_modal_does_not_promote_it_over_its_peer() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let a = Id::unique("a");
    let b = Id::unique("b");
    let a_child = Id::unique("a-child");
    let b_child = Id::unique("b-child");
    frame(&context, 0.0, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut lower =
            Dialog::new(&theme, a, "A").register(context, true, |ui| child(ui, a_child, 32.0));
        let mut upper =
            Dialog::new(&theme, b, "B").register(context, true, |ui| child(ui, b_child, 32.0));
        lower.refresh(false, |ui| child(ui, a_child, 32.0));
        let upper_layer = context.memory(|m| m.top_modal_layer());
        lower.refresh(true, |ui| child(ui, a_child, 32.0));
        assert_eq!(context.memory(|m| m.top_modal_layer()), upper_layer);
        assert_eq!(context.memory(|m| m.focused()), Some(b_child));
        while let Some(event) = context.next_input_event() {
            upper.handle(&event);
            lower.handle(&event);
        }
        lower.paint(|ui| child(ui, a_child, 32.0)).unwrap();
        upper.paint(|ui| child(ui, b_child, 32.0)).unwrap();
    });
    frame(&context, 0.1, vec2(800.0, 600.0), vec![], |context| {
        context.begin_input();
        let mut upper =
            Dialog::new(&theme, b, "B").register(context, true, |ui| child(ui, b_child, 32.0));
        let upper_layer = context.memory(|m| m.top_modal_layer());
        let mut lower =
            Dialog::new(&theme, a, "A").register(context, true, |ui| child(ui, a_child, 32.0));
        assert_eq!(context.memory(|m| m.top_modal_layer()), upper_layer);
        while let Some(event) = context.next_input_event() {
            upper.handle(&event);
            lower.handle(&event);
        }
        assert_eq!(context.memory(|m| m.focused()), Some(b_child));
        lower.paint(|ui| child(ui, a_child, 32.0)).unwrap();
        upper.paint(|ui| child(ui, b_child, 32.0)).unwrap();
    });
}

#[test]
fn zero_opacity_exit_still_blocks_input_until_its_deadline() {
    let (context, _, _installation) = setup();
    let theme = Theme::light();
    let id = Id::unique("zero-exit");
    let opener = Id::unique("zero-opener");
    let body = Id::unique("zero-body");
    for (time, present) in [(0.0, true), (0.199, true), (0.201, false)] {
        frame(&context, time, vec2(800.0, 600.0), vec![], |context| {
            context.begin_input();
            let trigger = background(context, opener, true);
            if time == 0.0 {
                trigger.request_focus();
            }
            let mut dialog = Dialog::new(&theme, id, "Zero")
                .register(context, time == 0.0, |ui| child(ui, body, 32.0));
            if time == 0.0 {
                dialog.refresh(false, |ui| child(ui, body, 32.0));
                dialog.refresh(false, |ui| child(ui, body, 32.0));
            }
            assert_eq!(context.memory(|m| m.top_modal_layer().is_some()), present);
            if !present {
                assert_eq!(context.memory(|m| m.focused()), Some(opener));
            }
            drain(context, &mut dialog);
            dialog.paint(|ui| child(ui, body, 32.0)).unwrap();
        });
    }
}
