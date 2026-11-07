//! Native defaults and built-in consumers retain their input ownership.
use egui::{Context, Event, Id, Key, Modifiers, RawInput, Rect, Ui, pos2, vec2};
use egui_prism::egui;

fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn raw(events: Vec<Event>, time: f64) -> RawInput {
    RawInput {
        events,
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
        ..Default::default()
    }
}
fn context(ordered: bool) -> Context {
    let ctx = Context::default();
    ctx.options_mut(|o| {
        o.ordered_input = ordered;
        o.max_passes = 1.try_into().unwrap();
    });
    ctx
}
fn defaults(ctx: &Context, events: Vec<Event>, consume: bool) -> egui::FullOutput {
    ctx.run_ui(raw(events, 0.0), |_| {
        if ctx.options(|o| o.ordered_input) {
            ctx.begin_input();
            while ctx.next_input_event().is_some() {
                if consume {
                    ctx.input_mut(|i| i.events.clear());
                }
            }
        } else if consume {
            ctx.input_mut(|i| i.events.clear());
        }
    })
}
#[test]
fn ordered_default_shortcuts_match_stock_frame_semantics() {
    let cmd = Modifiers::COMMAND;
    for keys in [
        vec![Key::Plus],
        vec![Key::Plus, Key::Plus],
        vec![Key::Minus, Key::Plus],
        vec![Key::Plus, Key::Num0],
        vec![Key::Num0, Key::Plus],
    ] {
        let mut zooms = Vec::new();
        for ordered in [false, true] {
            let ctx = context(ordered);
            defaults(&ctx, keys.iter().map(|k| key(*k, cmd)).collect(), false)
                .drop_without_applying_deltas();
            assert_eq!(ctx.zoom_factor(), 1.0);
            defaults(&ctx, vec![], false).drop_without_applying_deltas();
            zooms.push(ctx.zoom_factor());
        }
        assert_eq!(zooms[0], zooms[1], "keys={keys:?}");
    }
    for consume in [false, true] {
        let ctx = context(true);
        let out = defaults(&ctx, vec![key(Key::Q, cmd)], consume);
        let closes = out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .filter(|c| matches!(c, egui::ViewportCommand::Close))
            .count();
        assert_eq!(closes, usize::from(!consume));
        out.drop_without_applying_deltas();
        defaults(&ctx, vec![], false).drop_without_applying_deltas();
    }
}

#[test]
fn middle_click_paste_uses_event_position_and_stock_editor() {
    for old_focus in [false, true] {
        for standalone in [false, true] {
            let ctx = context(true);
            let ids = [Id::unique("paste-a"), Id::unique("paste-b")];
            let rects = [
                Rect::from_min_size(pos2(20.0, 30.0), vec2(120.0, 30.0)),
                Rect::from_min_size(pos2(200.0, 30.0), vec2(120.0, 30.0)),
            ];
            let mut texts = [String::new(), String::new()];
            let pos = rects[1].center();
            let mut events = vec![Event::PointerMoved(rects[0].center())];
            if !standalone {
                events.push(Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Middle,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                });
            }
            events.push(Event::MiddleClickPaste {
                pos,
                text: "primary".into(),
            });
            events.push(Event::Text("!".into()));
            ctx.run_ui(raw(events, 0.0), |ui| {
                ctx.begin_input();
                for n in 0..2 {
                    ui.put(
                        rects[n],
                        egui::TextEdit::singleline(&mut texts[n]).id(ids[n]),
                    );
                }
                if old_focus {
                    ctx.memory_mut(|m| m.request_focus(ids[0]));
                }
                while let Some(input) = ctx.next_input_event() {
                    for n in 0..2 {
                        if input.target == Some(ids[n]) {
                            ui.put(
                                rects[n],
                                egui::TextEdit::singleline(&mut texts[n]).id(ids[n]),
                            );
                        }
                    }
                }
                for n in 0..2 {
                    ui.put(
                        rects[n],
                        egui::TextEdit::singleline(&mut texts[n]).id(ids[n]),
                    );
                }
            })
            .drop_without_applying_deltas();
            assert_eq!(
                texts,
                ["", "primary!"],
                "focus={old_focus}, standalone={standalone}"
            );
            assert_eq!(ctx.memory(|m| m.focused()), Some(ids[1]));
        }
    }
}

fn labels(ui: &mut Ui, only: Option<Id>) -> Vec<egui::Response> {
    ["alpha", "bravo", "charlie"]
        .into_iter()
        .enumerate()
        .filter_map(|(n, text)| {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .scope_id(Id::unique(("label-scope", n)))
                    .max_rect(Rect::from_min_size(
                        pos2(20.0, 30.0 + n as f32 * 22.0),
                        vec2(150.0, 20.0),
                    )),
            );
            if only.is_none_or(|id| id == child.next_auto_id()) {
                Some(child.add(egui::Label::new(text).selectable(true)))
            } else {
                None
            }
        })
        .collect()
}
fn label_frame(ctx: &Context, events: Vec<Event>, time: f64) -> (egui::FullOutput, Vec<Rect>) {
    label_frame_with_policy(ctx, raw(events, time), None)
}
fn label_frame_with_policy(
    ctx: &Context,
    input: RawInput,
    policy: Option<bool>,
) -> (egui::FullOutput, Vec<Rect>) {
    let mut rects = Vec::new();
    let out = ctx.run_ui(input, |ui| {
        if let Some(policy) = policy {
            ui.style_mut().interaction.multi_widget_text_select = policy;
        }
        if ctx.options(|o| o.ordered_input) {
            ctx.begin_input();
            labels(ui, None);
            while let Some(input) = ctx.next_input_event() {
                if let Some(target) = input.target {
                    labels(ui, Some(target));
                }
            }
        }
        rects = labels(ui, None).iter().map(|r| r.rect).collect();
    });
    (out, rects)
}
fn press(pos: egui::Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        pressed,
        button: egui::PointerButton::Primary,
        modifiers: Modifiers::NONE,
    }
}
fn copied(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(s) => Some(s.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn stock_multilabel_copy_snapshot_precedes_escape_or_selection_change() {
    for change_selection in [false, true] {
        let ctx = context(false);
        let (out, rects) = label_frame(&ctx, vec![], 0.0);
        out.drop_without_applying_deltas();
        let start = rects[0].left_center();
        let end = rects[1].right_center();
        for (n, events) in [
            vec![Event::PointerMoved(start), press(start, true)],
            vec![Event::PointerMoved(end)],
            vec![press(end, false)],
            vec![],
        ]
        .into_iter()
        .enumerate()
        {
            label_frame(&ctx, events, 0.1 + n as f64 * 0.1)
                .0
                .drop_without_applying_deltas();
        }
        let baseline = label_frame(&ctx, vec![Event::Copy], 0.6).0;
        let expected = copied(&baseline);
        assert_eq!(expected.len(), 1);
        assert!(
            expected[0].contains("alpha") && expected[0].contains("bravo"),
            "{expected:?}"
        );
        baseline.drop_without_applying_deltas();
        ctx.options_mut(|o| o.ordered_input = true);
        let mut events = vec![Event::Copy];
        if change_selection {
            events.extend([
                Event::PointerMoved(rects[2].center()),
                press(rects[2].center(), true),
                press(rects[2].center(), false),
            ]);
        }
        events.push(key(Key::Escape, Modifiers::NONE));
        let out = label_frame(&ctx, events, 0.7).0;
        assert_eq!(
            copied(&out),
            expected,
            "change_selection={change_selection}"
        );
        out.drop_without_applying_deltas();
        assert!(
            !ctx.plugin::<egui::text_selection::LabelSelectionState>()
                .lock()
                .has_selection()
        );
        let out = label_frame(&ctx, vec![], 0.8).0;
        assert!(copied(&out).is_empty());
        out.drop_without_applying_deltas();
    }
}

#[test]
fn ordered_labels_copy_each_event_selection_without_paint_replay() {
    let ctx = context(true);
    let (out, rects) = label_frame(&ctx, vec![], 0.0);
    out.drop_without_applying_deltas();
    let start = rects[0].left_center();
    let end = rects[1].right_center();
    label_frame(
        &ctx,
        vec![
            press(start, true),
            Event::PointerMoved(end),
            press(end, false),
        ],
        0.1,
    )
    .0
    .drop_without_applying_deltas();
    let out = label_frame(
        &ctx,
        vec![
            Event::Copy,
            press(rects[2].center(), true),
            press(rects[2].center(), false),
            Event::Cut,
            key(Key::Escape, Modifiers::NONE),
            Event::Copy,
        ],
        0.5,
    )
    .0;
    assert_eq!(copied(&out), vec!["alpha\nbravo", "charlie"]);
    out.drop_without_applying_deltas();
    let out = label_frame(&ctx, vec![Event::Copy], 0.6).0;
    assert!(copied(&out).is_empty());
    out.drop_without_applying_deltas();
}

#[test]
fn configured_shortcuts_consumption_and_discard_do_not_replay() {
    for consume in [false, true] {
        let ctx = context(true);
        ctx.options_mut(|o| {
            o.zoom_with_keyboard = false;
            o.quit_shortcuts = vec![egui::KeyboardShortcut::new(Modifiers::NONE, Key::F12)];
            o.max_passes = 2.try_into().unwrap();
        });
        let mut passes = 0;
        let out = ctx.run_ui(
            raw(
                vec![
                    key(Key::Plus, Modifiers::COMMAND),
                    key(Key::Q, Modifiers::COMMAND),
                    key(Key::F12, Modifiers::NONE),
                ],
                0.0,
            ),
            |_| {
                ctx.begin_input();
                while ctx.next_input_event().is_some() {
                    if consume {
                        ctx.input_mut(|i| i.events.clear());
                    }
                }
                assert!(ctx.next_input_event().is_none());
                passes += 1;
                if passes == 1 {
                    ctx.request_discard("exercise shortcut finalization once");
                }
            },
        );
        assert_eq!(passes, 2);
        assert_eq!(
            out.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .filter(|c| matches!(c, egui::ViewportCommand::Close))
                .count(),
            usize::from(!consume)
        );
        out.drop_without_applying_deltas();
        defaults(&ctx, vec![], false).drop_without_applying_deltas();
        assert_eq!(ctx.zoom_factor(), 1.0);
    }
    let ctx = context(true);
    ctx.options_mut(|o| {
        o.quit_shortcuts = vec![egui::KeyboardShortcut::new(Modifiers::NONE, Key::Escape)]
    });
    egui::DragAndDrop::set_payload(&ctx, "payload");
    let out = defaults(&ctx, vec![key(Key::Escape, Modifiers::NONE)], false);
    assert!(!egui::DragAndDrop::has_any_payload(&ctx));
    assert!(
        !out.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Close))
    );
    out.drop_without_applying_deltas();
}

#[test]
fn removed_label_endpoint_cannot_supply_clipboard_text() {
    for whole_layer in [false, true] {
        let ctx = context(true);
        let (out, rects) = label_frame(&ctx, vec![], 0.0);
        out.drop_without_applying_deltas();
        label_frame(
            &ctx,
            vec![
                press(rects[0].left_center(), true),
                Event::PointerMoved(rects[1].right_center()),
                press(rects[1].right_center(), false),
            ],
            0.1,
        )
        .0
        .drop_without_applying_deltas();
        let out = ctx.run_ui(raw(vec![Event::Copy], 0.2), |ui| {
            ctx.begin_input();
            let registered = labels(ui, None);
            if whole_layer {
                ctx.remove_input_layer(registered[0].layer_id);
            } else {
                ctx.remove_input_widget(registered[0].id);
            }
            while ctx.next_input_event().is_some() {}
        });
        assert!(copied(&out).is_empty());
        out.drop_without_applying_deltas();
        assert!(
            !ctx.plugin::<egui::text_selection::LabelSelectionState>()
                .lock()
                .has_selection()
        );
    }
}

#[test]
fn unfocused_label_selection_keys_precede_copy() {
    let mut results = Vec::new();
    for ordered in [false, true] {
        let ctx = context(ordered);
        let (out, rects) = label_frame(&ctx, vec![], 0.0);
        out.drop_without_applying_deltas();
        label_frame(
            &ctx,
            vec![
                press(rects[2].center(), true),
                press(rects[2].center(), false),
            ],
            0.1,
        )
        .0
        .drop_without_applying_deltas();
        let out = label_frame(
            &ctx,
            vec![key(Key::ArrowLeft, Modifiers::SHIFT), Event::Copy],
            0.2,
        )
        .0;
        results.push(copied(&out));
        out.drop_without_applying_deltas();
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0].len(), 1);
    assert_eq!(results[0][0].chars().count(), 1);
}

#[test]
fn middle_paste_does_not_fall_back_through_blocked_or_blank_targets() {
    for mode in ["disabled", "readonly", "modal", "blank"] {
        let ctx = context(true);
        let ids = [Id::unique("blocked-a"), Id::unique("blocked-b")];
        let rects = [
            Rect::from_min_size(pos2(20.0, 30.0), vec2(120.0, 30.0)),
            Rect::from_min_size(pos2(200.0, 30.0), vec2(120.0, 30.0)),
        ];
        let pos = if mode == "blank" {
            pos2(450.0, 250.0)
        } else {
            rects[1].center()
        };
        let mut texts = [String::new(), String::new()];
        ctx.run_ui(
            raw(
                vec![Event::MiddleClickPaste {
                    pos,
                    text: "primary".into(),
                }],
                0.0,
            ),
            |ui| {
                ctx.begin_input();
                if mode == "modal" {
                    egui::Modal::new(Id::unique("blocking-modal"))
                        .register(&ctx, |ui| ui.label("Modal"));
                }
                ui.put(
                    rects[0],
                    egui::TextEdit::singleline(&mut texts[0]).id(ids[0]),
                );
                ui.add_enabled_ui(mode != "disabled", |ui| {
                    ui.put(
                        rects[1],
                        egui::TextEdit::singleline(&mut texts[1])
                            .id(ids[1])
                            .interactive(mode != "readonly"),
                    );
                });
                ctx.memory_mut(|m| m.request_focus(ids[0]));
                while let Some(input) = ctx.next_input_event() {
                    if input.target == Some(ids[0]) {
                        ui.put(
                            rects[0],
                            egui::TextEdit::singleline(&mut texts[0]).id(ids[0]),
                        );
                    }
                    if input.target == Some(ids[1]) {
                        ui.add_enabled_ui(mode != "disabled", |ui| {
                            ui.put(
                                rects[1],
                                egui::TextEdit::singleline(&mut texts[1])
                                    .id(ids[1])
                                    .interactive(mode != "readonly"),
                            );
                        });
                    }
                }
            },
        )
        .drop_without_applying_deltas();
        assert_eq!(texts, ["", ""], "mode={mode}");
    }
}

#[test]
fn single_label_policy_does_not_expand_across_labels() {
    for local_override in [false, true] {
        let mut copies = Vec::new();
        for ordered in [false, true] {
            let ctx = context(ordered);
            if !local_override {
                ctx.global_style_mut(|style| style.interaction.multi_widget_text_select = false);
            }
            let policy = local_override.then_some(false);
            let (out, rects) = label_frame_with_policy(&ctx, raw(vec![], 0.0), policy);
            out.drop_without_applying_deltas();
            for (n, events) in [
                vec![press(rects[0].left_center(), true)],
                vec![Event::PointerMoved(rects[1].right_center())],
                vec![press(rects[1].right_center(), false)],
            ]
            .into_iter()
            .enumerate()
            {
                label_frame_with_policy(&ctx, raw(events, 0.1 + n as f64 * 0.1), policy)
                    .0
                    .drop_without_applying_deltas();
            }
            let out = label_frame_with_policy(&ctx, raw(vec![Event::Copy], 0.5), policy).0;
            copies.push(copied(&out));
            out.drop_without_applying_deltas();
        }
        assert_eq!(copies[0], vec!["alpha"]);
        assert_eq!(copies[0], copies[1], "local_override={local_override}");
    }
}

#[test]
fn label_drag_extends_past_text_in_the_same_column() {
    for upward in [false, true] {
        let mut results = Vec::new();
        for ordered in [false, true] {
            let ctx = context(ordered);
            let (out, rects) = label_frame(&ctx, vec![], 0.0);
            out.drop_without_applying_deltas();
            let start = if upward {
                rects[2].right_center()
            } else {
                rects[0].left_center()
            };
            let end = if upward {
                pos2(rects[0].center().x, rects[0].top() - 10.0)
            } else {
                pos2(rects[2].center().x, rects[2].bottom() + 10.0)
            };
            for (n, events) in [
                vec![press(start, true)],
                vec![Event::PointerMoved(end)],
                vec![press(end, false)],
            ]
            .into_iter()
            .enumerate()
            {
                label_frame(&ctx, events, 0.1 + n as f64 * 0.1)
                    .0
                    .drop_without_applying_deltas();
            }
            let out = label_frame(&ctx, vec![Event::Copy], 0.5).0;
            results.push(copied(&out));
            out.drop_without_applying_deltas();
        }
        assert_eq!(results[0], results[1], "upward={upward}");
        assert_eq!(results[0], vec!["alpha\nbravo\ncharlie"]);
    }
}

#[test]
fn label_selection_is_removed_with_its_viewport() {
    let ctx = context(true);
    ctx.set_embed_viewports(false);
    let child = egui::ViewportId::from_hash_of("label-child");
    let (out, rects) = label_frame(&ctx, vec![], 0.0);
    out.drop_without_applying_deltas();
    let parent = |keep| {
        ctx.run_ui(raw(vec![], 0.0), |_| {
            ctx.begin_input();
            if keep {
                ctx.show_viewport_deferred(child, egui::ViewportBuilder::default(), |ui, _| {
                    let ctx = ui.ctx().clone();
                    ctx.begin_input();
                    labels(ui, None);
                    while let Some(input) = ctx.next_input_event() {
                        if let Some(target) = input.target {
                            labels(ui, Some(target));
                        }
                    }
                    labels(ui, None);
                });
            }
            while ctx.next_input_event().is_some() {}
        })
    };
    let child_pass = |parent: egui::FullOutput, events| {
        let callback = parent.viewport_output[&child]
            .viewport_ui_cb
            .clone()
            .unwrap();
        parent.drop_without_applying_deltas();
        let mut input = raw(events, 0.1);
        input.viewport_id = child;
        input.viewports.insert(
            child,
            egui::ViewportInfo {
                parent: Some(egui::ViewportId::ROOT),
                ..Default::default()
            },
        );
        ctx.run_ui(input, |ui| callback(ui))
    };
    child_pass(
        parent(true),
        vec![
            press(rects[0].left_center(), true),
            Event::PointerMoved(rects[1].right_center()),
            press(rects[1].right_center(), false),
        ],
    )
    .drop_without_applying_deltas();
    assert!(
        ctx.plugin::<egui::text_selection::LabelSelectionState>()
            .lock()
            .has_selection()
    );
    let out = child_pass(parent(true), vec![Event::Copy]);
    assert_eq!(copied(&out), vec!["alpha\nbravo"]);
    out.drop_without_applying_deltas();
    parent(false).drop_without_applying_deltas();
    assert!(
        !ctx.plugin::<egui::text_selection::LabelSelectionState>()
            .lock()
            .has_selection()
    );
    let out = child_pass(parent(true), vec![Event::Copy]);
    assert!(copied(&out).is_empty());
    out.drop_without_applying_deltas();
}
