//! Ordered input across platform termination, layer placement, timers and host policy.
use egui::{Context, Event, Id, Key, Modifiers, RawInput, Rect, Sense, Ui, pos2, vec2};
use egui_prism::egui;

fn id(n: usize) -> Id {
    Id::unique(("input-review", n))
}
fn rect(n: usize) -> Rect {
    Rect::from_min_size(pos2(20.0 + n as f32 * 140.0, 30.0), vec2(100.0, 30.0))
}
fn context() -> Context {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    ctx
}
fn raw(events: Vec<Event>, time: f64) -> RawInput {
    RawInput {
        events,
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
        ..Default::default()
    }
}
fn pointer(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn preedit(text: &str) -> Event {
    Event::Ime(egui::ImeEvent::Preedit {
        text: text.into(),
        active_range_chars: None,
    })
}
fn editors(ui: &mut Ui, texts: &mut [String; 2], only: Option<Id>) {
    for (n, text) in texts.iter_mut().enumerate() {
        if only.is_none_or(|target| target == id(n)) {
            ui.put(rect(n), egui::TextEdit::singleline(text).id(id(n)));
        }
    }
}

#[test]
fn interrupted_ime_clear_then_commit_keeps_original_owner() {
    for split_frames in [false, true] {
        let ctx = context();
        let mut texts = [String::new(), String::new()];
        let start = vec![key(Key::Tab), preedit("中"), key(Key::Tab), preedit("")];
        let finish = vec![
            Event::Ime(egui::ImeEvent::Commit("中".into())),
            Event::Text("B".into()),
        ];
        let batches = if split_frames {
            vec![start, finish]
        } else {
            vec![start.into_iter().chain(finish).collect()]
        };
        for (n, events) in batches.into_iter().enumerate() {
            ctx.run_ui(raw(events, n as f64), |ui| {
                ctx.begin_input();
                editors(ui, &mut texts, None);
                while let Some(event) = ctx.next_input_event() {
                    if let Some(target) = event.target {
                        editors(ui, &mut texts, Some(target));
                    }
                }
                editors(ui, &mut texts, None);
            })
            .drop_without_applying_deltas();
        }
        assert_eq!(ctx.memory(|m| m.focused()), Some(id(1)));
        assert_eq!(texts[1], "B", "split_frames={split_frames}");
    }
}

#[test]
fn reopened_modal_becomes_current_barrier_after_placement() {
    let ctx = context();
    let a = egui::Modal::new(id(10));
    let b = egui::Modal::new(id(11));
    let mut reopened = None;
    for pass in 0..3 {
        let events = if pass == 2 {
            vec![key(Key::Tab), key(Key::Tab)]
        } else {
            vec![]
        };
        ctx.run_ui(raw(events, pass as f64), |_| {
            ctx.begin_input();
            if pass == 0 {
                a.clone().register(&ctx, |ui| ui.button("A"));
            }
            b.clone().register(&ctx, |ui| ui.button("B"));
            if pass == 2 {
                reopened = Some(a.clone().register(&ctx, |ui| ui.button("A")).inner.id);
            }
            while ctx.next_input_event().is_some() {}
            if pass == 2 {
                assert_eq!(
                    ctx.memory(|m| m.top_modal_layer()),
                    Some(egui::LayerId::new(egui::Order::Foreground, id(10)))
                );
            }
        })
        .drop_without_applying_deltas();
    }
    assert_eq!(ctx.memory(|m| m.focused()), reopened);
}

fn scene(ui: &mut Ui) -> egui::Response {
    let ctx = ui.ctx().clone();
    let parent = egui::LayerId::new(egui::Order::Middle, id(20));
    ctx.move_to_top(parent);
    let bounds = Rect::from_min_size(pos2(20.0, 20.0), vec2(200.0, 160.0));
    let mut parent_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("scene-parent")
            .layer_id(parent)
            .max_rect(bounds),
    );
    parent_ui.interact(bounds, id(21), Sense::click_and_drag());
    let mut view = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 160.0));
    egui::Scene::new()
        .show(&mut parent_ui, &mut view, |ui| {
            ui.interact(
                Rect::from_min_size(pos2(50.0, 50.0), vec2(60.0, 30.0)),
                id(22),
                Sense::click(),
            )
        })
        .inner
}
#[test]
fn raised_parent_keeps_stock_scene_sublayer_above_it_for_hits() {
    for first_display in [true, false] {
        let measure = context();
        let mut position = None;
        measure
            .run_ui(raw(vec![], 0.0), |ui| {
                measure.begin_input();
                let button = scene(ui);
                position = Some(
                    measure.layer_transform_to_global(button.layer_id).unwrap()
                        * button.rect.center(),
                );
                while measure.next_input_event().is_some() {}
            })
            .drop_without_applying_deltas();
        let ctx = if first_display { context() } else { measure };
        let position = position.unwrap();
        let mut targets = Vec::new();
        ctx.run_ui(
            raw(vec![pointer(position, true), pointer(position, false)], 1.0),
            |ui| {
                ctx.begin_input();
                scene(ui);
                while let Some(event) = ctx.next_input_event() {
                    targets.push(event.target);
                }
            },
        )
        .drop_without_applying_deltas();
        assert_eq!(
            targets,
            vec![Some(id(22)); 2],
            "first_display={first_display}"
        );
    }
}

#[test]
fn stationary_touch_dispatches_long_touch_once_without_raw_events() {
    let ctx = context();
    let pos = rect(0).center();
    let touch = Event::Touch {
        device_id: egui::TouchDeviceId(1),
        id: egui::TouchId(1),
        phase: egui::TouchPhase::Start,
        pos,
        force: None,
    };
    let mut delivered = 0;
    for (time, events) in [
        (0.0, vec![touch, pointer(pos, true)]),
        (1.0, vec![]),
        (2.0, vec![]),
    ] {
        ctx.run_ui(raw(events.clone(), time), |ui| {
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click());
            while let Some(event) = ctx.next_input_event() {
                assert_eq!(ui.input(|i| i.raw.events.clone()), events);
                if matches!(event.trigger, egui::InputTrigger::Time) {
                    assert!(ui.input(|i| i.events.is_empty()));
                    assert_eq!(time, 1.0);
                }
                if event.target == Some(id(0)) {
                    let response = ui.interact(rect(0), id(0), Sense::click());
                    delivered += usize::from(response.long_touched());
                }
            }
            assert!(!ui.interact(rect(0), id(0), Sense::click()).long_touched());
        })
        .drop_without_applying_deltas();
        assert_eq!(delivered, usize::from(time >= 1.0), "time={time}");
    }
}

#[test]
fn focus_surrender_policy_distinguishes_press_click_and_never() {
    use egui::SurrenderFocusOn::{Clicks, Never, Presses};
    for policy in [Clicks, Never, Presses] {
        for drag in [false, true] {
            let ctx = context();
            ctx.options_mut(|o| o.input_options.surrender_focus_on = policy);
            ctx.memory_mut(|m| m.request_focus(id(0)));
            let outside = pos2(450.0, 250.0);
            let release = if drag { pos2(350.0, 200.0) } else { outside };
            let mut states = Vec::new();
            ctx.run_ui(
                raw(
                    vec![
                        pointer(outside, true),
                        Event::PointerMoved(release),
                        pointer(release, false),
                    ],
                    0.0,
                ),
                |ui| {
                    ctx.begin_input();
                    ui.interact(rect(0), id(0), Sense::click());
                    while ctx.next_input_event().is_some() {
                        states.push(ctx.memory(|m| m.focused()));
                    }
                },
            )
            .drop_without_applying_deltas();
            let after_press = if policy == Presses { None } else { Some(id(0)) };
            let after_release = if policy == Never || (policy == Clicks && drag) {
                Some(id(0))
            } else {
                None
            };
            assert_eq!(
                states,
                vec![after_press, after_press, after_release],
                "{policy:?}, drag={drag}"
            );
        }
    }
}

#[test]
fn stock_editor_release_preserves_host_transfer_or_surrender() {
    for transfer in [false, true] {
        for split_frames in [false, true] {
            let ctx = context();
            let mut texts = [String::new(), String::new()];
            let press = vec![pointer(rect(0).center(), true)];
            let release = vec![pointer(rect(0).center(), false), Event::Text("B".into())];
            let batches = if split_frames {
                vec![press, release]
            } else {
                vec![press.into_iter().chain(release).collect()]
            };
            for (n, events) in batches.into_iter().enumerate() {
                ctx.run_ui(raw(events, n as f64 * 0.1), |ui| {
                    ctx.begin_input();
                    editors(ui, &mut texts, None);
                    while let Some(input) = ctx.next_input_event() {
                        if let Some(target) = input.target {
                            editors(ui, &mut texts, Some(target));
                        }
                        if matches!(
                            input.trigger,
                            egui::InputTrigger::Event {
                                event: Event::PointerButton { pressed: true, .. },
                                ..
                            }
                        ) {
                            if transfer {
                                ctx.memory_mut(|m| m.request_focus(id(1)));
                            } else {
                                ctx.memory_mut(|m| m.surrender_focus(id(0)));
                            }
                        }
                    }
                    editors(ui, &mut texts, None);
                })
                .drop_without_applying_deltas();
            }
            assert_eq!(ctx.memory(|m| m.focused()), transfer.then_some(id(1)));
            assert_eq!(texts[0], "");
            assert_eq!(texts[1], if transfer { "B" } else { "" });
        }
    }
}

#[test]
fn modal_close_enables_current_background_directional_navigation() {
    let ctx = context();
    ctx.run_ui(
        raw(vec![key(Key::Escape), key(Key::ArrowRight)], 0.0),
        |ui| {
            ctx.begin_input();
            let modal = egui::Modal::new(id(30)).register(&ctx, |ui| ui.button("Close"));
            for n in 0..2 {
                ui.interact(rect(n), id(n), Sense::click());
            }
            ctx.memory_mut(|m| m.request_focus(modal.inner.id));
            while let Some(event) = ctx.next_input_event() {
                if matches!(
                    event.trigger,
                    egui::InputTrigger::Event {
                        event: Event::Key {
                            key: Key::Escape,
                            ..
                        },
                        ..
                    }
                ) {
                    ctx.remove_input_layer(modal.response.layer_id);
                    ctx.memory_mut(|m| m.request_focus(id(0)));
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(1)));
}

#[test]
fn time_transition_precedes_raw_events_without_replay() {
    let ctx = context();
    let pos = rect(0).center();
    let mut triggers = Vec::new();
    for (time, events) in [
        (0.0, vec![pointer(pos, true)]),
        (1.0, vec![key(Key::A)]),
        (2.0, vec![]),
    ] {
        ctx.run_ui(raw(events.clone(), time), |ui| {
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click_and_drag());
            while let Some(input) = ctx.next_input_event() {
                assert_eq!(ui.input(|i| i.raw.events.clone()), events);
                match input.trigger {
                    egui::InputTrigger::Time => {
                        assert!(ui.input(|i| i.events.is_empty()));
                        assert_eq!(input.target, Some(id(0)));
                        assert!(
                            ui.interact(rect(0), id(0), Sense::click_and_drag())
                                .drag_started()
                        );
                        triggers.push("time");
                    }
                    egui::InputTrigger::Event { index, event } => {
                        assert_eq!(index, 0);
                        assert_eq!(ui.input(|i| i.events.clone()), vec![event]);
                        triggers.push("raw");
                    }
                }
            }
        })
        .drop_without_applying_deltas();
    }
    assert_eq!(triggers, vec!["raw", "time", "raw"]);
}

#[test]
fn new_composition_after_interrupted_clear_binds_to_new_focus() {
    let ctx = context();
    let mut texts = [String::new(), String::new()];
    ctx.run_ui(
        raw(
            vec![
                key(Key::Tab),
                preedit("中"),
                key(Key::Tab),
                preedit(""),
                preedit("文"),
                Event::Ime(egui::ImeEvent::Commit("文".into())),
            ],
            0.0,
        ),
        |ui| {
            ctx.begin_input();
            editors(ui, &mut texts, None);
            while let Some(input) = ctx.next_input_event() {
                if let Some(target) = input.target {
                    editors(ui, &mut texts, Some(target));
                }
            }
            editors(ui, &mut texts, None);
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(texts[1], "文");
}

#[test]
fn modal_close_uses_registered_background_focus_policy() {
    let ctx = context();
    let mut received = None;
    ctx.run_ui(
        raw(vec![key(Key::Escape), key(Key::ArrowRight)], 0.0),
        |ui| {
            ctx.begin_input();
            let modal = egui::Modal::new(id(30)).register(&ctx, |ui| ui.button("Close"));
            for n in 0..2 {
                ui.interact(rect(n), id(n), Sense::click());
                ui.memory_mut(|m| {
                    m.register_focus_policy(
                        id(n),
                        ui.layer_id(),
                        egui::EventFilter {
                            horizontal_arrows: true,
                            ..Default::default()
                        },
                    )
                });
            }
            ctx.memory_mut(|m| m.request_focus(modal.inner.id));
            while let Some(input) = ctx.next_input_event() {
                match input.trigger {
                    egui::InputTrigger::Event {
                        event:
                            Event::Key {
                                key: Key::Escape, ..
                            },
                        ..
                    } => {
                        ctx.remove_input_layer(modal.response.layer_id);
                        ctx.memory_mut(|m| m.request_focus(id(0)));
                    }
                    egui::InputTrigger::Event {
                        event:
                            Event::Key {
                                key: Key::ArrowRight,
                                ..
                            },
                        ..
                    } => received = input.target,
                    _ => {}
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(0)));
    assert_eq!(received, Some(id(0)));
}

#[test]
fn elapsed_touch_does_not_deliver_to_cancelled_or_ineligible_owner() {
    for mode in 0..4 {
        let ctx = context();
        let pos = rect(0).center();
        let touch = Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase: egui::TouchPhase::Start,
            pos,
            force: None,
        };
        ctx.run_ui(raw(vec![touch, pointer(pos, true)], 0.0), |ui| {
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click());
            while ctx.next_input_event().is_some() {}
        })
        .drop_without_applying_deltas();
        let events = match mode {
            0 => vec![Event::PointerGone],
            1 => vec![Event::WindowFocused(false)],
            _ => vec![],
        };
        ctx.run_ui(raw(events, 1.0), |ui| {
            ctx.begin_input();
            if mode == 2 {
                ui.add_enabled_ui(false, |ui| ui.interact(rect(0), id(0), Sense::click()));
            } else {
                ui.interact(rect(0), id(0), Sense::click());
            }
            if mode == 3 {
                egui::Modal::new(id(40)).register(&ctx, |ui| ui.button("Blocked"));
            }
            while let Some(input) = ctx.next_input_event() {
                assert!(!matches!(input.trigger, egui::InputTrigger::Time));
            }
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn ime_interruption_alone_does_not_override_click_surrender_policy() {
    let ctx = context();
    ctx.memory_mut(|m| m.request_focus(id(0)));
    let outside = pos2(450.0, 250.0);
    ctx.run_ui(
        raw(vec![pointer(outside, true), pointer(outside, false)], 0.0),
        |ui| {
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click());
            while let Some(input) = ctx.next_input_event() {
                if matches!(
                    input.trigger,
                    egui::InputTrigger::Event {
                        event: Event::PointerButton { pressed: true, .. },
                        ..
                    }
                ) {
                    ctx.memory_mut(|m| m.interrupt_ime());
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), None);
}
