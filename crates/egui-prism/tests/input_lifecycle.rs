//! Ownership, event visibility, and text editing across ordered input phases.
use egui::{Context, Event, Id, Key, Modifiers, RawInput, Rect, Sense, Ui, pos2, vec2};
use egui_prism::egui;
fn id(n: usize) -> Id {
    Id::unique(("lifecycle", n))
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
fn pointer(n: usize, pressed: bool) -> Event {
    Event::PointerButton {
        pos: rect(n).center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn frame(
    ctx: &Context,
    events: Vec<Event>,
    mut register: impl FnMut(&mut Ui),
    mut handle: impl FnMut(&mut Ui, egui::RoutedInput),
) -> egui::FullOutput {
    ctx.run_ui(
        RawInput {
            events,
            time: Some(1.0 + ctx.cumulative_frame_nr() as f64 / 60.0),
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            ctx.begin_input();
            register(ui);
            while let Some(event) = ctx.next_input_event() {
                handle(ui, event);
            }
        },
    )
}
#[test]
fn capture_survives_frame_boundary_and_release_respects_host_surrender() {
    let ctx = context();
    let register = |ui: &mut Ui| {
        ui.interact(rect(0), id(0), Sense::drag());
    };
    frame(&ctx, vec![pointer(0, true)], register, |_, _| {}).drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(0)));
    ctx.memory_mut(|m| m.surrender_focus(id(0)));
    let mut stopped = 0;
    frame(
        &ctx,
        vec![pointer(1, false), key(Key::A, Modifiers::NONE)],
        register,
        |ui, event| {
            if event.target == Some(id(0)) {
                stopped += usize::from(ui.interact(rect(0), id(0), Sense::drag()).drag_stopped());
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(stopped, 1);
    assert_eq!(ctx.memory(|m| m.focused()), None);
}
#[test]
fn disabled_or_disappeared_capture_is_cancelled() {
    for disabled in [false, true] {
        let ctx = context();
        frame(
            &ctx,
            vec![pointer(0, true)],
            |ui| {
                ui.interact(rect(0), id(0), Sense::drag());
            },
            |_, _| {},
        )
        .drop_without_applying_deltas();
        let mut targets = Vec::new();
        frame(
            &ctx,
            vec![Event::PointerMoved(rect(1).center()), pointer(1, false)],
            |ui| {
                if disabled {
                    ui.add_enabled_ui(false, |ui| {
                        ui.interact(rect(0), id(0), Sense::drag());
                    });
                }
            },
            |_, event| targets.push(event.target),
        )
        .drop_without_applying_deltas();
        assert!(
            !targets.contains(&Some(id(0))),
            "disabled={disabled}: {targets:?}"
        );
        assert_eq!(ctx.memory(|m| m.focused()), None);
    }
}
#[test]
fn request_discard_does_not_replay_actions() {
    let ctx = context();
    ctx.options_mut(|o| o.max_passes = 2.try_into().unwrap());
    let mut calls = 0;
    let mut clicks = 0;
    ctx.run_ui(
        RawInput {
            events: vec![pointer(0, true), pointer(0, false)],
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            calls += 1;
            ctx.begin_input();
            ui.interact(rect(0), id(0), Sense::click());
            while let Some(event) = ctx.next_input_event() {
                if event.target == Some(id(0)) {
                    clicks += usize::from(ui.interact(rect(0), id(0), Sense::click()).clicked());
                }
            }
            if calls == 1 {
                ctx.request_discard("exercise a caller-requested layout pass");
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(calls, 2);
    assert_eq!(clicks, 1);
}
#[derive(Default)]
struct Observer {
    raw: Vec<Vec<Event>>,
    begin: Vec<Vec<Event>>,
    end: Vec<(Vec<Event>, bool, bool)>,
}
impl egui::Plugin for Observer {
    fn debug_name(&self) -> &'static str {
        "ordered input observer"
    }
    fn input_hook(&mut self, _: &Context, raw: &mut RawInput) {
        self.raw.push(raw.events.clone());
    }
    fn on_begin_pass(&mut self, ui: &mut Ui) {
        self.begin.push(ui.input(|i| i.events.clone()));
    }
    fn on_end_pass(&mut self, ui: &mut Ui) {
        self.end.push(ui.input(|i| {
            (
                i.raw.events.clone(),
                i.pointer.any_released(),
                i.key_down(Key::A),
            )
        }));
    }
}
#[test]
fn plugins_observe_original_batch_and_final_aggregate_state() {
    let ctx = context();
    ctx.add_plugin(Observer::default());
    let observer = ctx.plugin::<Observer>();
    let events = vec![
        pointer(0, true),
        pointer(0, false),
        key(Key::A, Modifiers::NONE),
    ];
    let mut routed = Vec::new();
    frame(
        &ctx,
        events.clone(),
        |ui| {
            ui.interact(rect(0), id(0), Sense::click());
        },
        |_, event| routed.push(event.event),
    )
    .drop_without_applying_deltas();
    let observer = observer.lock();
    assert_eq!(observer.raw, vec![events.clone()]);
    assert_eq!(observer.begin, vec![events.clone()]);
    assert_eq!(routed, events);
    assert_eq!(observer.end, vec![(events, true, true)]);
}
fn editors(ui: &mut Ui, texts: &mut [String; 2], only: Option<Id>) {
    for (n, text) in texts.iter_mut().enumerate() {
        if only.is_none_or(|target| target == id(n)) {
            ui.put(rect(n), egui::TextEdit::singleline(text).id(id(n)));
        }
    }
}
fn edit_frame(ctx: &Context, texts: &mut [String; 2], events: Vec<Event>) -> egui::FullOutput {
    ctx.run_ui(
        RawInput {
            events,
            time: Some(1.0 + ctx.cumulative_frame_nr() as f64 * 2.0),
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
            ..Default::default()
        },
        |ui| {
            ctx.begin_input();
            editors(ui, texts, None);
            while let Some(event) = ctx.next_input_event() {
                if let Some(target) = event.target {
                    editors(ui, texts, Some(target));
                }
            }
            editors(ui, texts, None);
        },
    )
}
#[test]
fn text_selection_copy_undo_and_paint_do_not_replay() {
    let ctx = context();
    let mut texts = ["abc".into(), String::new()];
    edit_frame(&ctx, &mut texts, vec![key(Key::Tab, Modifiers::NONE)])
        .drop_without_applying_deltas();
    let mut state = egui::TextEdit::load_state(&ctx, id(0)).unwrap();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(1),
            egui::text::CCursor::new(3),
        )));
    state.store(&ctx, id(0));
    let out = edit_frame(&ctx, &mut texts, vec![Event::Copy, Event::Text("X".into())]);
    assert_eq!(texts, ["aX", ""]);
    assert_eq!(
        out.platform_output
            .commands
            .iter()
            .filter(|c| matches!(c,egui::OutputCommand::CopyText(text) if text=="bc"))
            .count(),
        1
    );
    out.drop_without_applying_deltas();
    edit_frame(&ctx, &mut texts, vec![key(Key::Z, Modifiers::COMMAND)])
        .drop_without_applying_deltas();
    assert_eq!(texts, ["abc", ""]);
    edit_frame(&ctx, &mut texts, vec![]).drop_without_applying_deltas();
    assert_eq!(texts, ["abc", ""]);
}
#[test]
fn interrupted_composition_cannot_commit_to_new_focus() {
    let ctx = context();
    let mut texts = [String::new(), String::new()];
    edit_frame(
        &ctx,
        &mut texts,
        vec![
            key(Key::Tab, Modifiers::NONE),
            Event::Ime(egui::ImeEvent::Preedit {
                text: "中".into(),
                active_range_chars: None,
            }),
            key(Key::Tab, Modifiers::NONE),
            Event::Ime(egui::ImeEvent::Commit("中".into())),
            Event::Text("B".into()),
        ],
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(1)));
    assert_eq!(texts[1], "B");
}

#[test]
fn viewport_focus_capture_and_removed_viewport_do_not_leak() {
    use std::sync::{Arc, Mutex};
    let ctx = context();
    ctx.set_embed_viewports(false);
    let child = egui::ViewportId::from_hash_of("child");
    let received = Arc::new(Mutex::new(Vec::new()));
    let parent_frame = |keep_child: bool, events| {
        let received = received.clone();
        frame(
            &ctx,
            events,
            |ui| {
                ui.interact(rect(0), id(0), Sense::drag());
                if keep_child {
                    let received = received.clone();
                    ctx.show_viewport_deferred(
                        child,
                        egui::ViewportBuilder::default(),
                        move |ui, class| {
                            assert!(class == egui::ViewportClass::Deferred);
                            let ctx = ui.ctx().clone();
                            ctx.begin_input();
                            ui.interact(rect(0), id(0), Sense::drag());
                            while let Some(event) = ctx.next_input_event() {
                                received.lock().unwrap().push(event.target);
                            }
                        },
                    );
                }
            },
            |_, event| assert_eq!(event.target, None),
        )
    };
    let child_frame = |output: egui::FullOutput, events| {
        let callback = output.viewport_output[&child]
            .viewport_ui_cb
            .clone()
            .unwrap();
        output.drop_without_applying_deltas();
        let mut input = RawInput {
            viewport_id: child,
            events,
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 300.0))),
            ..Default::default()
        };
        input.viewports.insert(
            child,
            egui::ViewportInfo {
                parent: Some(egui::ViewportId::ROOT),
                ..Default::default()
            },
        );
        ctx.run_ui(input, |ui| callback(ui))
            .drop_without_applying_deltas();
    };
    child_frame(parent_frame(true, vec![]), vec![pointer(0, true)]);
    child_frame(
        parent_frame(true, vec![key(Key::A, Modifiers::NONE)]),
        vec![pointer(1, false)],
    );
    assert_eq!(*received.lock().unwrap(), vec![Some(id(0)), Some(id(0))]);
    let closed = parent_frame(false, vec![]);
    assert!(!closed.viewport_output.contains_key(&child));
    closed.drop_without_applying_deltas();
    received.lock().unwrap().clear();
    child_frame(
        parent_frame(true, vec![]),
        vec![pointer(1, false), key(Key::A, Modifiers::NONE)],
    );
    assert_eq!(*received.lock().unwrap(), vec![None, None]);
}
#[derive(Default)]
struct Consumer;
impl egui::Plugin for Consumer {
    fn debug_name(&self) -> &'static str {
        "key consumer"
    }
    fn on_begin_pass(&mut self, ui: &mut Ui) {
        ui.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::A);
        });
    }
}
#[test]
fn plugin_consumption_is_respected_without_deleting_raw_events() {
    let ctx = context();
    ctx.add_plugin(Consumer);
    let events = vec![key(Key::A, Modifiers::NONE), key(Key::B, Modifiers::NONE)];
    let mut routed = Vec::new();
    frame(
        &ctx,
        events.clone(),
        |_| {},
        |ui, event| {
            assert_eq!(ui.input(|i| i.raw.events.clone()), events);
            routed.push(event.event);
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(routed, vec![key(Key::B, Modifiers::NONE)]);
}
#[test]
fn first_tab_then_arrow_uses_text_edit_policy_before_any_edit() {
    let ctx = context();
    let mut texts = ["abc".into(), "def".into()];
    edit_frame(
        &ctx,
        &mut texts,
        vec![
            key(Key::Tab, Modifiers::NONE),
            key(Key::ArrowLeft, Modifiers::NONE),
            Event::Text("X".into()),
        ],
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(0)));
    assert_eq!(texts, ["abXc", "def"]);
}
#[test]
fn focus_policy_requires_current_pass_registration() {
    let ctx = context();
    frame(
        &ctx,
        vec![key(Key::Tab, Modifiers::NONE)],
        |ui| {
            ui.interact(rect(0), id(0), Sense::click());
            ui.memory_mut(|m| {
                m.register_focus_policy(
                    id(0),
                    ui.layer_id(),
                    egui::EventFilter {
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        },
        |_, _| {},
    )
    .drop_without_applying_deltas();
    frame(
        &ctx,
        vec![key(Key::Escape, Modifiers::NONE)],
        |ui| {
            ui.interact(rect(0), id(0), Sense::click());
        },
        |_, _| {},
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), None);
}
#[test]
fn empty_frame_keeps_hover_and_cancels_disappeared_capture() {
    let ctx = context();
    frame(
        &ctx,
        vec![Event::PointerMoved(rect(0).center())],
        |ui| {
            ui.interact(rect(0), id(0), Sense::drag());
        },
        |_, _| {},
    )
    .drop_without_applying_deltas();
    frame(
        &ctx,
        vec![],
        |ui| {
            ui.interact(rect(0), id(0), Sense::drag());
        },
        |_, _| {},
    )
    .drop_without_applying_deltas();
    assert!(ctx.read_response(id(0)).unwrap().hovered());
    frame(
        &ctx,
        vec![pointer(0, true)],
        |ui| {
            ui.interact(rect(0), id(0), Sense::drag());
        },
        |_, _| {},
    )
    .drop_without_applying_deltas();
    frame(&ctx, vec![], |_| {}, |_, _| {}).drop_without_applying_deltas();
    assert!(!ctx.egui_is_using_pointer());
}

#[test]
fn disabling_between_press_and_release_cancels_in_the_same_batch() {
    let ctx = context();
    let mut releases = Vec::new();
    frame(
        &ctx,
        vec![pointer(0, true), pointer(0, false)],
        |ui| {
            ui.interact(rect(0), id(0), Sense::drag());
        },
        |ui, event| {
            if matches!(event.event, Event::PointerButton { pressed: true, .. }) {
                ctx.input_layout(|| {
                    ui.add_enabled_ui(false, |ui| {
                        ui.interact(rect(0), id(0), Sense::drag());
                    });
                });
            } else {
                releases.push(event.target);
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(releases, vec![None]);
    assert_eq!(ctx.memory(|m| m.focused()), None);
}

#[test]
fn accesskit_focus_then_directional_navigation_uses_current_geometry() {
    let ctx = context();
    let mut clicks = 0;
    let focus = Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::Focus,
        target_node: id(0).accesskit_id(),
        target_tree: egui::accesskit::TreeId::ROOT,
        data: None,
    });
    frame(
        &ctx,
        vec![
            focus,
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ],
        |ui| {
            for n in 0..2 {
                ui.interact(rect(n), id(n), Sense::click());
            }
        },
        |ui, event| {
            if event.target == Some(id(1)) {
                clicks += usize::from(ui.interact(rect(1), id(1), Sense::click()).clicked());
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(1)));
    assert_eq!(clicks, 1);
}
