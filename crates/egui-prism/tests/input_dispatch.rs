//! Framework input routing regressions using ordinary egui consumers.

use egui::{Context, Event, EventFilter, Id, Key, Modifiers, RawInput, Rect, Sense, pos2, vec2};
use egui_prism::egui;
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn rect(n: usize) -> Rect {
    Rect::from_min_size(pos2(20. + n as f32 * 140., 20.), vec2(100., 30.))
}
fn id(n: usize) -> Id {
    Id::unique(("current", n))
}
fn raw(events: Vec<Event>) -> RawInput {
    RawInput {
        events,
        time: Some(1.),
        screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(500., 300.))),
        ..Default::default()
    }
}
#[test]
fn one_pass_arrow_tab_arrow() {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut edits = [0; 2];
    let mut calls = 0;
    ctx.run_ui(
        raw(vec![
            key(Key::Tab),
            key(Key::ArrowRight),
            key(Key::Tab),
            key(Key::ArrowRight),
        ]),
        |ui| {
            calls += 1;
            ctx.begin_input();
            for n in 0..2 {
                ui.interact(rect(n), id(n), Sense::click());
                ui.memory_mut(|m| {
                    m.register_focus_policy(
                        id(n),
                        ui.layer_id(),
                        EventFilter {
                            horizontal_arrows: true,
                            escape: true,
                            ..Default::default()
                        },
                    )
                });
            }
            while let Some(e) = ctx.next_input_event() {
                if matches!(
                    e.event,
                    Event::Key {
                        key: Key::ArrowRight,
                        ..
                    }
                ) && let Some(n) = (0..2).find(|&n| e.target == Some(id(n)))
                {
                    edits[n] += 1;
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(edits, [1, 1]);
    assert_eq!(calls, 1);
}
#[test]
fn ordinary_text_edit_in_event_phase() {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut text = String::new();
    ctx.run_ui(
        raw(vec![
            key(Key::Tab),
            Event::Ime(egui::ImeEvent::Preedit {
                text: "中".into(),
                active_range_chars: None,
            }),
            Event::Ime(egui::ImeEvent::Commit("中".into())),
            key(Key::ArrowLeft),
            Event::Text("A".into()),
        ]),
        |ui| {
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click_and_drag());
            ui.memory_mut(|m| {
                m.register_focus_policy(
                    id(0),
                    ui.layer_id(),
                    EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                )
            });
            while let Some(e) = ctx.next_input_event() {
                if e.target == Some(id(0)) {
                    ui.scope(|ui| {
                        ui.put(rect(0), egui::TextEdit::singleline(&mut text).id(id(0)));
                    });
                }
            }
            ui.put(rect(0), egui::TextEdit::singleline(&mut text).id(id(0)));
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(text, "A中");
}
fn pointer(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn register_modal(ctx: &Context, n: usize) -> egui::LayerId {
    let layer = egui::LayerId::new(egui::Order::Foreground, id(10 + n));
    let bounds = Rect::from_min_size(pos2(0., 0.), vec2(400., 180.));
    // This fixture has a fixed, known layout; use the ordinary egui area state.
    ctx.memory_mut(|m| {
        m.areas_mut().set_state(
            layer,
            egui::AreaState {
                pivot_pos: Some(bounds.min),
                size: Some(bounds.size()),
                interactable: true,
                ..Default::default()
            },
        );
        m.areas_mut().move_to_top(layer);
        m.set_modal_layer(layer);
    });
    let ui = egui::Ui::new(
        ctx.clone(),
        layer.id,
        egui::UiBuilder::new().layer_id(layer).max_rect(bounds),
    );
    ui.interact(bounds, id(20 + n), Sense::click());
    ui.interact(rect(1), id(30 + n), Sense::click());
    layer
}
#[test]
fn modal_open_replace_close_then_background_in_one_pass() {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut events = Vec::new();
    for pos in [
        rect(0).center(),
        rect(1).center(),
        rect(1).center(),
        rect(1).center(),
    ] {
        events.extend([pointer(pos, true), pointer(pos, false)]);
    }
    let mut state = 0;
    let mut modal = None;
    let mut targets = Vec::new();
    let mut calls = 0;
    ctx.run_ui(raw(events), |ui| {
        calls += 1;
        ctx.begin_input();
        ui.interact(rect(0), id(0), Sense::click());
        ui.interact(rect(1), id(1), Sense::click());
        while let Some(e) = ctx.next_input_event() {
            if matches!(e.event, Event::PointerButton { pressed: false, .. }) {
                targets.push(e.target);
                match state {
                    0 => {
                        assert_eq!(e.target, Some(id(0)));
                        modal = Some(register_modal(&ctx, 0));
                    }
                    1 => {
                        assert_eq!(e.target, Some(id(30)));
                        ctx.remove_input_layer(modal.take().unwrap());
                        modal = Some(register_modal(&ctx, 1));
                    }
                    2 => {
                        assert_eq!(e.target, Some(id(31)));
                        ctx.remove_input_layer(modal.take().unwrap());
                    }
                    3 => assert_eq!(e.target, Some(id(1))),
                    _ => unreachable!(),
                }
                state += 1;
            }
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(state, 4);
    assert_eq!(calls, 1);
    assert_eq!(
        targets,
        vec![Some(id(0)), Some(id(30)), Some(id(31)), Some(id(1))]
    );
}
fn native_modal(ctx: &Context, register: bool, counter: &mut usize, salt: usize) {
    let modal = egui::Modal::new(id(100 + salt)).area(
        egui::Area::new(id(100 + salt))
            .fixed_pos(pos2(0., 0.))
            .order(egui::Order::Foreground),
    );
    let content = |ui: &mut egui::Ui| {
        ui.set_min_size(vec2(300., 150.));
        *counter += usize::from(
            ui.interact(rect(1), id(200 + salt), Sense::click())
                .clicked(),
        );
    };
    if register {
        modal.register(ctx, content);
    } else {
        modal.show(ctx, content);
    }
}
#[test]
fn actual_modal_open_replace_close_then_background() {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let opener = Rect::from_min_size(pos2(20., 200.), vec2(80., 30.));
    let mut events = Vec::new();
    for pos in [
        opener.center(),
        rect(1).center(),
        rect(1).center(),
        rect(1).center(),
    ] {
        events.extend([pointer(pos, true), pointer(pos, false)]);
    }
    let mut modal = None;
    let mut modal_clicks = 0;
    let mut background_clicks = 0;
    let mut calls = 0;
    ctx.run_ui(raw(events), |ui| {
        calls += 1;
        ctx.begin_input();
        ui.interact(opener, id(0), Sense::click());
        ui.interact(rect(1), id(1), Sense::click());
        while let Some(event) = ctx.next_input_event() {
            match event.target {
                Some(target) if target == id(0) => {
                    if ui.interact(opener, id(0), Sense::click()).clicked() {
                        modal = Some(0);
                        native_modal(&ctx, true, &mut modal_clicks, 0);
                    }
                }
                Some(target) if modal.is_some_and(|n| target == id(200 + n)) => {
                    let n = modal.unwrap();
                    let before = modal_clicks;
                    native_modal(&ctx, false, &mut modal_clicks, n);
                    if modal_clicks > before {
                        ctx.remove_input_layer(egui::LayerId::new(
                            egui::Order::Foreground,
                            id(100 + n),
                        ));
                        modal = if n == 0 {
                            native_modal(&ctx, true, &mut modal_clicks, 1);
                            Some(1)
                        } else {
                            None
                        };
                    }
                }
                Some(target) if target == id(1) => {
                    background_clicks +=
                        usize::from(ui.interact(rect(1), id(1), Sense::click()).clicked());
                }
                _ => {}
            }
        }
        ui.interact(opener, id(0), Sense::click());
        ui.interact(rect(1), id(1), Sense::click());
    })
    .drop_without_applying_deltas();
    assert_eq!(calls, 1);
    assert_eq!(modal_clicks, 2);
    assert_eq!(background_clicks, 1);
    assert!(modal.is_none());
}
// Use the stock modal's automatic centering and the button's natural layout.
// The returned response is the only source of target geometry and identity.
fn natural_modal(ctx: &Context, register: bool) -> egui::Response {
    let modal = egui::Modal::new(id(400));
    let content = |ui: &mut egui::Ui| {
        ui.label("An automatically sized modal");
        ui.add_space(13.0);
        ui.button("Keep this modal open")
    };
    if register {
        modal.register(ctx, content).inner
    } else {
        modal.show(ctx, content).inner
    }
}

fn persistent_natural_modal_case(modal_before: bool) -> Vec<Id> {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let opener = Rect::from_min_size(pos2(20.0, 250.0), vec2(80.0, 30.0));
    let mut open = false;
    let mut button = None;
    let mut modal_clicks = 0;
    let mut background_clicks = 0;
    let mut targets = Vec::new();
    let mut calls = 0;
    for frame in 0..3 {
        let position = button
            .as_ref()
            .map_or(opener.center(), |r: &egui::Response| r.rect.center());
        let mut input = raw(vec![pointer(position, true), pointer(position, false)]);
        input.time = Some(1.0 + frame as f64 / 60.0);
        let mut registration = Vec::new();
        let mut paint = Vec::new();
        ctx.run_ui(input, |ui| {
            calls += 1;
            ctx.begin_input();
            if open && modal_before {
                button = Some(natural_modal(&ctx, true));
                registration.push("modal");
            }
            ui.interact(opener, id(0), Sense::click());
            // A real competing target directly underneath the naturally laid-out button.
            if let Some(button) = &button {
                ui.interact(button.rect, id(1), Sense::click());
            }
            registration.push("background");
            if open && !modal_before {
                button = Some(natural_modal(&ctx, true));
                registration.push("modal");
            }
            while let Some(event) = ctx.next_input_event() {
                if event.target == Some(id(0)) {
                    if ui.interact(opener, id(0), Sense::click()).clicked() {
                        open = true;
                        button = Some(natural_modal(&ctx, true));
                        assert!(button.as_ref().unwrap().enabled());
                    }
                } else if button.as_ref().is_some_and(|r| event.target == Some(r.id)) {
                    let response = natural_modal(&ctx, false);
                    if response.clicked() {
                        modal_clicks += 1;
                        targets.push(response.id);
                    }
                    assert_eq!(response.rect, button.as_ref().unwrap().rect);
                } else if event.target == Some(id(1)) {
                    background_clicks += usize::from(
                        ui.interact(button.as_ref().unwrap().rect, id(1), Sense::click())
                            .clicked(),
                    );
                }
            }
            assert!(open, "the modal must reach both paint order branches");
            if modal_before {
                let response = natural_modal(&ctx, false);
                assert!(!response.clicked(), "painting must not replay the click");
                assert_eq!(response.rect, button.as_ref().unwrap().rect);
                paint.push("modal");
            }
            ui.interact(opener, id(0), Sense::click());
            ui.interact(button.as_ref().unwrap().rect, id(1), Sense::click());
            paint.push("background");
            if !modal_before {
                let response = natural_modal(&ctx, false);
                assert!(!response.clicked(), "painting must not replay the click");
                assert_eq!(response.rect, button.as_ref().unwrap().rect);
                paint.push("modal");
            }
        })
        .drop_without_applying_deltas();
        let expected = if modal_before {
            vec!["modal", "background"]
        } else {
            vec!["background", "modal"]
        };
        assert_eq!(paint, expected);
        if frame > 0 {
            assert_eq!(registration, expected);
            assert_eq!(modal_clicks, frame as usize);
        }
    }
    assert!(open);
    assert_eq!(calls, 3);
    assert_eq!(modal_clicks, 2);
    assert_eq!(background_clicks, 0);
    targets
}

#[test]
fn actual_modal_remains_open_across_both_layout_orders() {
    let before = persistent_natural_modal_case(true);
    let after = persistent_natural_modal_case(false);
    assert_eq!(
        before, after,
        "layout order must not change the routed targets"
    );
}

#[test]
fn actual_modal_first_sizing_routes_a_natural_button_in_the_opening_batch() {
    // Measure with a separate context, then send the same physical position to a
    // fresh context with no cached AreaState. No geometry is seeded into it.
    let reference = Context::default();
    reference.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut measured = None;
    reference
        .run_ui(raw(vec![]), |_| {
            reference.begin_input();
            measured = Some(natural_modal(&reference, true));
            assert!(reference.next_input_event().is_none());
            natural_modal(&reference, false);
        })
        .drop_without_applying_deltas();
    let measured = measured.unwrap();
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let opener = Rect::from_min_size(pos2(20.0, 250.0), vec2(80.0, 30.0));
    let events = [opener.center(), measured.rect.center()]
        .into_iter()
        .flat_map(|p| [pointer(p, true), pointer(p, false)])
        .collect();
    let mut button = None;
    let mut clicks = 0;
    let mut background_clicks = 0;
    let mut calls = 0;
    ctx.run_ui(raw(events), |ui| {
        calls += 1;
        ctx.begin_input();
        ui.interact(opener, id(0), Sense::click());
        ui.interact(measured.rect, id(1), Sense::click());
        while let Some(event) = ctx.next_input_event() {
            if event.target == Some(id(0)) && ui.interact(opener, id(0), Sense::click()).clicked() {
                button = Some(natural_modal(&ctx, true));
                let button = button.as_ref().unwrap();
                assert!(button.enabled());
                assert_eq!(button.rect, measured.rect);
                assert_eq!(button.id, measured.id);
            } else if event.target == Some(measured.id) {
                clicks += usize::from(natural_modal(&ctx, false).clicked());
            } else if event.target == Some(id(1)) {
                background_clicks +=
                    usize::from(ui.interact(measured.rect, id(1), Sense::click()).clicked());
            }
        }
        assert!(button.is_some());
        assert!(!natural_modal(&ctx, false).clicked());
    })
    .drop_without_applying_deltas();
    assert_eq!(calls, 1);
    assert_eq!(clicks, 1);
    assert_eq!(background_clicks, 0);
}

#[test]
fn newly_registered_modal_excludes_lower_modal_from_tab_navigation() {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut upper = None;
    ctx.run_ui(raw(vec![key(Key::Tab)]), |_| {
        ctx.begin_input();
        egui::Modal::new(id(500)).register(&ctx, |ui| ui.button("Lower modal"));
        upper = Some(
            egui::Modal::new(id(501))
                .register(&ctx, |ui| ui.button("Upper modal"))
                .inner
                .id,
        );
        while ctx.next_input_event().is_some() {}
    })
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), upper);
}
