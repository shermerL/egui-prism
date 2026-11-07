//! Ordered native scrolling consumes input and frame time once per local shell.
use egui::scroll_area::{ScrollSource, State};
use egui::{
    Context, Event, Id, InputTrigger, Modifiers, Pos2, RawInput, Rect, Sense, Ui, UiBuilder,
};
use egui_prism::egui;
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Ordinary,
    OrderedShell,
    OrderedMuted,
}
#[derive(Clone, Copy)]
struct Geometry {
    origin: Pos2,
    clip: Rect,
}
#[derive(Clone, Copy)]
struct Sample {
    id: Id,
    offset: f32,
    velocity: f32,
    geometry: Geometry,
    updated: Geometry,
}
#[derive(Default)]
struct Counts {
    force_drag: bool,
    ui: usize,
    content: usize,
    actions: [usize; 2],
}
fn bid(i: usize) -> Id {
    Id::unique(("button", i))
}
fn rect(g: Geometry, i: usize) -> Rect {
    Rect::from_min_size(
        g.origin + egui::vec2(20., 20. + i as f32 * 40.),
        egui::vec2(100., 20.),
    )
}
fn controls(ui: &mut Ui, g: Geometry, only: Option<usize>, counts: &mut Counts) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .scope_id(Id::unique("content"))
            .max_rect(Rect::from_min_size(g.origin, egui::vec2(180., 600.))),
    );
    child.set_clip_rect(g.clip);
    for i in 0..2 {
        if only.is_none_or(|n| n == i) {
            let response = child.interact(rect(g, i), bid(i), Sense::click());
            if response.clicked() {
                counts.actions[i] += 1;
            }
        }
    }
}
fn shell(ui: &mut Ui, enabled: bool, content: bool, counts: &mut Counts) -> Sample {
    let mut area_ui = ui.new_child(
        UiBuilder::new()
            .scope_id(Id::unique("root-shell"))
            .max_rect(Rect::from_min_size(Pos2::ZERO, egui::vec2(200., 120.))),
    );
    let output = egui::ScrollArea::vertical()
        .id_salt("scroll")
        .max_height(120.)
        .auto_shrink([false, false])
        .content_margin(0.)
        .scroll_source(if enabled {
            ScrollSource {
                drag: if counts.force_drag {
                    egui::scroll_area::DragScroll::Always
                } else {
                    egui::scroll_area::DragScroll::OnTouch
                },
                ..Default::default()
            }
        } else {
            ScrollSource::NONE
        })
        .show_viewport(&mut area_ui, |ui, _viewport| {
            let g = Geometry {
                origin: ui.max_rect().min,
                clip: ui.clip_rect(),
            };
            ui.expand_to_include_rect(Rect::from_min_size(g.origin, egui::vec2(180., 600.)));
            if content {
                counts.content += 1;
                controls(ui, g, None, counts);
            }
            g
        });
    Sample {
        id: output.id,
        offset: output.state.unclamped_offset().y,
        velocity: output.state.velocity().y,
        geometry: output.inner,
        updated: Geometry {
            origin: Pos2::new(
                output.inner.origin.x,
                output.inner_rect.min.y - output.state.unclamped_offset().y,
            ),
            clip: output.inner.clip,
        },
    }
}
struct Harness {
    ctx: Context,
    frame: usize,
    last: Option<Sample>,
    repeats: usize,
}
impl Harness {
    fn new() -> Self {
        let ctx = Context::default();
        ctx.options_mut(|o| o.max_passes = 1.try_into().unwrap());
        Self {
            ctx,
            frame: 0,
            last: None,
            repeats: 1,
        }
    }
    fn frame(&mut self, mode: Mode, events: Vec<Event>, force_drag: bool) -> (Sample, Counts) {
        self.ctx
            .options_mut(|o| o.ordered_input = mode != Mode::Ordinary);
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(200., 120.))),
            time: Some(self.frame as f64 / 60.),
            predicted_dt: 1. / 60.,
            events,
            ..Default::default()
        };
        let mut counts = Counts {
            force_drag,
            ..Default::default()
        };
        let mut result = None;
        let output = self.ctx.run_ui(raw, |ui| {
            counts.ui += 1;
            if mode == Mode::Ordinary {
                let s = shell(ui, true, true, &mut counts);

                result = Some(s);
                return;
            }
            self.ctx.begin_input();
            let mut s = shell(ui, mode != Mode::OrderedMuted, true, &mut counts);

            while let Some(e) = self.ctx.next_input_event() {
                if (force_drag
                    || e.target == Some(s.id.with(1))
                    || matches!(
                        e.trigger,
                        InputTrigger::Time
                            | InputTrigger::Event {
                                event: Event::MouseWheel { .. },
                                ..
                            }
                    ))
                    && matches!(mode, Mode::OrderedShell | Mode::OrderedMuted)
                {
                    for _ in 0..self.repeats {
                        s = shell(ui, true, false, &mut counts);
                    }

                    s.geometry = s.updated;
                    self.ctx
                        .input_layout(|| controls(ui, s.geometry, None, &mut counts));
                }
                for i in 0..2 {
                    if e.target == Some(bid(i)) {
                        controls(ui, s.geometry, Some(i), &mut counts);
                    }
                }
            }
            s = shell(ui, mode != Mode::OrderedMuted, true, &mut counts);

            result = Some(s);
        });
        output.drop_without_applying_deltas();
        let s = result.unwrap();
        self.last = Some(s);

        assert_eq!(counts.ui, 1, "no extra run_ui pass");
        self.frame += 1;
        (s, counts)
    }
    fn warm(&mut self, frames: usize) {
        for _ in 0..frames {
            self.frame(
                Mode::Ordinary,
                vec![Event::PointerMoved(egui::pos2(70., 30.))],
                false,
            );
        }
    }
}
fn wheel(delta: f32, source: egui::MouseWheelSource) -> Event {
    Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0., delta),
        phase: egui::TouchPhase::Move,
        source,
        modifiers: Modifiers::NONE,
    }
}
fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn touch_start() -> Event {
    Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::Vec2::ZERO,
        phase: egui::TouchPhase::Start,
        source: egui::MouseWheelSource::Trackpad,
        modifiers: Modifiers::NONE,
    }
}

fn close(actual: Sample, expected: Sample) {
    assert!(
        (actual.offset - expected.offset).abs() < 0.002,
        "offset {} != {}",
        actual.offset,
        expected.offset
    );
    assert!(
        (actual.velocity - expected.velocity).abs() < 0.002,
        "velocity {} != {}",
        actual.velocity,
        expected.velocity
    );
}

#[test]
fn wheel_units_and_empty_frames_match_native_consumption() {
    for unit in [egui::MouseWheelUnit::Point, egui::MouseWheelUnit::Line] {
        for delta in [-0.1, -4., -40.] {
            for mode in [Mode::OrderedShell, Mode::OrderedMuted] {
                let mut native = Harness::new();
                let mut ordered = Harness::new();
                native.warm(3);
                ordered.warm(3);
                ordered.repeats = 3;
                for frame in 0..16 {
                    let events = if frame == 0 {
                        vec![Event::MouseWheel {
                            unit,
                            delta: egui::vec2(0., delta),
                            phase: egui::TouchPhase::Move,
                            source: egui::MouseWheelSource::Wheel,
                            modifiers: Modifiers::NONE,
                        }]
                    } else {
                        vec![]
                    };
                    let (expected, _) = native.frame(Mode::Ordinary, events.clone(), false);
                    let (actual, counts) = ordered.frame(mode, events, false);
                    close(actual, expected);
                    assert_eq!(
                        counts.content, 2,
                        "only registration and paint traverse content"
                    );
                }
            }
        }
    }
}

#[test]
fn trackpad_scroll_click_scroll_click_refreshes_local_geometry() {
    for mode in [Mode::OrderedShell, Mode::OrderedMuted] {
        let mut h = Harness::new();
        h.warm(3);
        h.repeats = 3;
        let p = egui::pos2(70., 30.);
        let (s, counts) = h.frame(
            mode,
            vec![
                touch_start(),
                wheel(-40., egui::MouseWheelSource::Trackpad),
                pointer(p, true),
                pointer(p, false),
                wheel(40., egui::MouseWheelSource::Trackpad),
                pointer(p, true),
                pointer(p, false),
            ],
            false,
        );
        assert_eq!(counts.actions, [1, 1]);
        assert_eq!(s.offset, 0.);
        assert_eq!(counts.content, 2);
    }
}

fn kinetic_seed(drag: bool) -> Harness {
    let mut seed = Harness::new();
    seed.warm(3);
    if drag {
        let p = egui::pos2(150., 100.);
        seed.frame(Mode::Ordinary, vec![Event::PointerMoved(p)], true);
        seed.frame(Mode::Ordinary, vec![pointer(p, true)], true);
        for y in [80., 60., 40.] {
            seed.frame(
                Mode::Ordinary,
                vec![Event::PointerMoved(egui::pos2(150., y))],
                true,
            );
        }
        seed.frame(
            Mode::Ordinary,
            vec![pointer(egui::pos2(150., 40.), false)],
            true,
        );
        assert!(seed.last.unwrap().velocity > 100.);
    } else {
        seed.frame(
            Mode::Ordinary,
            vec![wheel(80., egui::MouseWheelSource::Trackpad)],
            false,
        );
        assert!(seed.last.unwrap().offset < -1.);
    }
    seed
}

#[test]
fn inertia_and_spring_spend_one_frame_budget_across_repeated_shells() {
    for drag in [false, true] {
        let seed = kinetic_seed(drag);
        let state = State::load(&seed.ctx, seed.last.unwrap().id).unwrap();
        for mode in [Mode::OrderedShell, Mode::OrderedMuted] {
            let mut native = Harness::new();
            let mut ordered = Harness::new();
            native.warm(seed.frame);
            ordered.warm(seed.frame);
            ordered.repeats = 3;
            state.store(&native.ctx, native.last.unwrap().id);
            state.store(&ordered.ctx, ordered.last.unwrap().id);
            for _ in 0..6 {
                let (expected, _) = native.frame(Mode::Ordinary, vec![], false);
                let (actual, _) = ordered.frame(mode, vec![], false);
                close(actual, expected);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Scene {
    Siblings,
    Nested { at_end: bool },
    Clipped,
    Occluded,
}

fn scene(ui: &mut Ui, layout: Scene) -> Vec<(Id, f32)> {
    let mut results = Vec::new();
    for n in 0..if matches!(layout, Scene::Siblings) {
        2
    } else {
        1
    } {
        let mut child = ui.new_child(
            UiBuilder::new()
                .scope_id(Id::unique(("shell", n)))
                .max_rect(Rect::from_min_size(
                    egui::pos2(n as f32 * 220., 0.),
                    egui::vec2(200., 120.),
                )),
        );
        if matches!(layout, Scene::Clipped) {
            child.set_clip_rect(Rect::from_min_size(Pos2::ZERO, egui::vec2(30., 120.)));
        }
        let out = egui::ScrollArea::vertical()
            .id_salt("outer")
            .max_height(120.)
            .auto_shrink([false, false])
            .content_margin(0.)
            .show_viewport(&mut child, |ui, _| {
                let origin = ui.max_rect().min;
                if let Scene::Nested { at_end } = layout {
                    let mut inner_ui = ui.new_child(
                        UiBuilder::new()
                            .scope_id(Id::unique("inner-shell"))
                            .max_rect(Rect::from_min_size(origin, egui::vec2(180., 100.))),
                    );
                    let area = egui::ScrollArea::vertical()
                        .id_salt("inner")
                        .max_height(100.)
                        .auto_shrink([false, false])
                        .content_margin(0.);
                    let area = if at_end {
                        area.vertical_scroll_offset(500.)
                    } else {
                        area
                    };
                    let inner = area.show_viewport(&mut inner_ui, |ui, _| {
                        ui.set_min_size(egui::vec2(160., 600.));
                    });
                    results.push((inner.id, inner.state.unclamped_offset().y));
                }
                ui.expand_to_include_rect(Rect::from_min_size(origin, egui::vec2(180., 1000.)));
            });
        results.push((out.id, out.state.unclamped_offset().y));
    }
    results
}

fn scene_frame(
    ctx: &Context,
    frame: usize,
    ordered: bool,
    layout: Scene,
    events: Vec<Event>,
) -> Vec<(Id, f32)> {
    ctx.options_mut(|o| {
        o.ordered_input = ordered;
        o.max_passes = 1.try_into().unwrap();
    });
    let mut result = Vec::new();
    let output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(440., 150.))),
            time: Some(frame as f64 / 60.),
            events,
            ..Default::default()
        },
        |ui| {
            if ordered {
                ctx.begin_input();
            }
            // Native area layers, rather than a second scroll-specific hit tree.
            if matches!(layout, Scene::Occluded) {
                egui::Area::new(Id::unique("cover"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(Pos2::ZERO)
                    .show(ctx, |ui| {
                        ui.set_min_size(egui::vec2(200., 120.));
                    });
            }
            result = scene(ui, layout);
            if ordered {
                while ctx.next_input_event().is_some() {
                    for _ in 0..3 {
                        result = scene(ui, layout);
                    }
                }
                result = scene(ui, layout);
            }
        },
    );
    output.drop_without_applying_deltas();
    result
}

#[test]
fn native_nested_chain_siblings_clip_and_top_area_remain_authoritative() {
    for layout in [
        Scene::Siblings,
        Scene::Nested { at_end: false },
        Scene::Nested { at_end: true },
        Scene::Clipped,
        Scene::Occluded,
    ] {
        for source in [
            egui::MouseWheelSource::Wheel,
            egui::MouseWheelSource::Trackpad,
        ] {
            let native = Context::default();
            let ordered = Context::default();
            for frame in 0..8 {
                let mut events = vec![Event::PointerMoved(egui::pos2(70., 30.))];
                if frame == 3 {
                    if source == egui::MouseWheelSource::Trackpad {
                        events.push(touch_start());
                    }
                    events.push(wheel(-40., source));
                }
                let expected = scene_frame(&native, frame, false, layout, events.clone());
                let actual = scene_frame(&ordered, frame, true, layout, events);
                for (a, e) in actual.iter().zip(&expected) {
                    assert_eq!(a.0, e.0);
                    assert!(
                        (a.1 - e.1).abs() < 0.002,
                        "{layout:?}, {source:?}, frame {frame}: {} != {}",
                        a.1,
                        e.1
                    );
                }
                if frame == 3 {
                    match layout {
                        Scene::Siblings => {
                            assert!(actual[0].1 > 0.);
                            assert_eq!(actual[1].1, 0.);
                        }
                        Scene::Nested { at_end: false } => {
                            assert!(actual[0].1 > 0.);
                            assert_eq!(actual[1].1, 0.);
                        }
                        Scene::Nested { at_end: true } => {
                            assert_eq!(actual[0].1, 500.);
                            assert!(actual[1].1 > 0.);
                        }
                        Scene::Clipped | Scene::Occluded => assert_eq!(actual[0].1, 0.),
                    }
                }
            }
        }
    }
}

#[test]
fn removing_all_containers_removes_the_time_opportunity() {
    let mut h = kinetic_seed(true);
    let before = h.last.unwrap();
    h.ctx.options_mut(|o| o.ordered_input = true);
    let mut time_count = 0;
    let output = h.ctx.run_ui(
        RawInput {
            time: Some(4.),
            ..Default::default()
        },
        |_| {
            h.ctx.begin_input();
            while let Some(event) = h.ctx.next_input_event() {
                if matches!(event.trigger, InputTrigger::Time) {
                    time_count += 1;
                }
            }
        },
    );
    output.drop_without_applying_deltas();
    assert_eq!(time_count, 0);
    let after = State::load(&h.ctx, before.id).unwrap();
    assert_eq!(after.unclamped_offset().y, before.offset);
    assert_eq!(after.velocity().y, before.velocity);
    // Reappearing instances use the current frame budget, not elapsed absence.
    h.frame = 241;
    let native = Harness::new();
    native.ctx.options_mut(|o| o.ordered_input = false);
    let mut reference = Harness {
        ctx: native.ctx,
        frame: h.frame,
        last: None,
        repeats: 1,
    };
    after.store(&reference.ctx, before.id);
    // Warm timing separately without consuming the stored state.
    let output = reference.ctx.run_ui(
        RawInput {
            time: Some(4.),
            ..Default::default()
        },
        |_| {},
    );
    output.drop_without_applying_deltas();
    let (expected, _) = reference.frame(Mode::Ordinary, vec![], false);
    let (actual, _) = h.frame(Mode::OrderedShell, vec![], false);
    close(actual, expected);
}

fn retry_frame(
    ctx: &Context,
    retry: bool,
    time: Option<f64>,
    events: Vec<Event>,
) -> (Sample, usize) {
    ctx.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 2.try_into().unwrap();
    });
    let mut result = None;
    let mut clocks = 0;
    let output = ctx.run_ui(
        RawInput {
            time,
            events,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(200., 120.))),
            ..Default::default()
        },
        |ui| {
            let mut counts = Counts::default();
            ctx.begin_input();
            shell(ui, true, true, &mut counts);
            while let Some(event) = ctx.next_input_event() {
                if matches!(event.trigger, InputTrigger::Time) && event.target.is_none() {
                    clocks += 1;
                }
                shell(ui, true, false, &mut counts);
            }
            result = Some(shell(ui, true, true, &mut counts));
            if retry && ctx.current_pass_index() == 0 {
                ctx.request_discard("remeasure scroll content");
            }
        },
    );
    assert_eq!(
        output.platform_output.num_completed_passes,
        if retry { 2 } else { 1 }
    );
    output.drop_without_applying_deltas();
    (result.unwrap(), clocks)
}

#[test]
fn discard_remeasurement_does_not_release_wheel_carry_or_advance_physics() {
    for explicit_time in [false, true] {
        let native = Context::default();
        let retry = Context::default();
        for frame in 0..16 {
            let events = if frame == 3 {
                vec![wheel(-40., egui::MouseWheelSource::Wheel)]
            } else {
                vec![Event::PointerMoved(egui::pos2(70., 30.))]
            };
            let time = explicit_time.then_some(frame as f64 / 60.);
            let (expected, _) = retry_frame(&native, false, time, events.clone());
            let (actual, clocks) = retry_frame(&retry, true, time, events);
            assert_eq!(clocks, 1, "discard is layout, not another time budget");
            close(actual, expected);
        }
    }
}

#[test]
fn raw_events_do_not_spend_inertia_time_again() {
    let seed = kinetic_seed(true);
    let state = State::load(&seed.ctx, seed.last.unwrap().id).unwrap();
    let ordinary = Context::default();
    let burst = Context::default();
    for frame in 0..3 {
        retry_frame(&ordinary, false, Some(frame as f64 / 60.), vec![]);
        retry_frame(&burst, false, Some(frame as f64 / 60.), vec![]);
    }
    state.store(&ordinary, seed.last.unwrap().id);
    state.store(&burst, seed.last.unwrap().id);
    let (expected, _) = retry_frame(&ordinary, false, Some(3. / 60.), vec![]);
    let (actual, _) = retry_frame(
        &burst,
        false,
        Some(3. / 60.),
        vec![Event::Text("unused".into()); 20],
    );
    close(actual, expected);
}

#[test]
fn idle_wheel_uses_the_existing_predicted_frame_budget() {
    let mut native = Harness::new();
    let mut ordered = Harness::new();
    native.warm(3);
    ordered.warm(3);
    for _ in 0..90 {
        native.frame(Mode::Ordinary, vec![], false);
        ordered.frame(Mode::Ordinary, vec![], false);
    }
    native.frame = 600;
    ordered.frame = 600;
    let events = vec![wheel(-40., egui::MouseWheelSource::Wheel)];
    let (expected, _) = native.frame(Mode::Ordinary, events.clone(), false);
    let (actual, _) = ordered.frame(Mode::OrderedShell, events, false);
    close(actual, expected);
    assert!(
        actual.offset > 10. && actual.offset < 20.,
        "idle uses predicted dt, not the ten-second wall gap"
    );
}

#[test]
fn every_live_sibling_advances_its_own_inertia_once() {
    let seed = kinetic_seed(true);
    let state = State::load(&seed.ctx, seed.last.unwrap().id).unwrap();
    let native = Context::default();
    let ordered = Context::default();
    for frame in 0..seed.frame {
        for (ctx, mode) in [(&native, false), (&ordered, true)] {
            let ids = scene_frame(ctx, frame, mode, Scene::Siblings, vec![]);
            if frame == seed.frame - 1 {
                for (id, _) in ids {
                    state.store(ctx, id);
                }
            }
        }
    }
    for frame in seed.frame..seed.frame + 6 {
        let expected = scene_frame(&native, frame, false, Scene::Siblings, vec![]);
        let actual = scene_frame(&ordered, frame, true, Scene::Siblings, vec![]);
        for (a, e) in actual.iter().zip(&expected) {
            assert!((a.1 - e.1).abs() < 0.002);
        }
        assert!(
            actual
                .iter()
                .all(|(_, offset)| *offset > seed.last.unwrap().offset)
        );
    }
}

#[test]
fn wheel_terminal_events_cancel_future_carry_without_undoing_prior_actions() {
    for phase in [egui::TouchPhase::End, egui::TouchPhase::Cancel] {
        let mut h = Harness::new();
        h.warm(3);
        let p = egui::pos2(70., 55.);
        let terminal = Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::Vec2::ZERO,
            phase,
            source: egui::MouseWheelSource::Wheel,
            modifiers: Modifiers::NONE,
        };
        // The first released slice moves the second control under this pointer.
        let (first, counts) = h.frame(
            Mode::OrderedShell,
            vec![
                Event::PointerMoved(p),
                wheel(-40., egui::MouseWheelSource::Wheel),
                pointer(p, true),
                pointer(p, false),
                terminal,
            ],
            false,
        );
        assert_eq!(counts.actions, [0, 1]);
        assert!(first.offset > 10. && first.offset < 20.);
        assert!(!h.ctx.input(|i| i.is_scrolling()));
        for _ in 0..8 {
            let (after, _) = h.frame(Mode::OrderedShell, vec![], false);
            close(after, first);
        }
    }
}

#[test]
fn reversing_wheel_impulses_preserves_interleaved_clicks() {
    let mut h = Harness::new();
    h.warm(3);
    let b = egui::pos2(70., 55.);
    let a = egui::pos2(70., 30.);
    let (last, counts) = h.frame(
        Mode::OrderedShell,
        vec![
            Event::PointerMoved(b),
            wheel(-40., egui::MouseWheelSource::Wheel),
            pointer(b, true),
            pointer(b, false),
            Event::PointerMoved(a),
            wheel(40., egui::MouseWheelSource::Wheel),
            pointer(a, true),
            pointer(a, false),
        ],
        false,
    );
    assert_eq!(counts.actions, [1, 1]);
    assert!(last.offset.abs() < 0.002);
    let (empty, _) = h.frame(Mode::OrderedShell, vec![], false);
    close(empty, last);
}

#[test]
fn non_wheel_events_do_not_flush_small_remaining_carry() {
    let mut reference = Harness::new();
    let mut burst = Harness::new();
    reference.warm(3);
    burst.warm(3);
    let input = Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0., -0.1),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: Modifiers::NONE,
    };
    reference.frame(Mode::OrderedShell, vec![input.clone()], false);
    burst.frame(Mode::OrderedShell, vec![input], false);
    for _ in 0..12 {
        let (expected, _) = reference.frame(Mode::OrderedShell, vec![], false);
        let (actual, _) = burst.frame(
            Mode::OrderedShell,
            vec![Event::Text("unused".into()); 20],
            false,
        );
        close(actual, expected);
    }
    assert!((burst.last.unwrap().offset - 4.).abs() < 0.002);
}

#[test]
fn momentum_bounce_uses_frame_dt_without_reintegrating_spring() {
    let seed = kinetic_seed(false);
    let state = State::load(&seed.ctx, seed.last.unwrap().id).unwrap();
    let mut native = Harness::new();
    let mut ordered = Harness::new();
    native.warm(seed.frame);
    ordered.warm(seed.frame);
    ordered.repeats = 3;
    state.store(&native.ctx, native.last.unwrap().id);
    state.store(&ordered.ctx, ordered.last.unwrap().id);
    for n in 0..8 {
        let events = if n == 0 {
            vec![wheel(5., egui::MouseWheelSource::Momentum)]
        } else {
            vec![]
        };
        let (expected, _) = native.frame(Mode::Ordinary, events.clone(), false);
        let (actual, _) = ordered.frame(Mode::OrderedShell, events, false);
        close(actual, expected);
        if n == 0 {
            assert!(
                actual.velocity.abs() > 1.,
                "momentum keeps its native velocity denominator"
            );
        }
    }
}

#[test]
fn restored_state_does_not_inherit_another_contexts_consumption_token() {
    let seed = kinetic_seed(true);
    let state = State::load(&seed.ctx, seed.last.unwrap().id).unwrap();
    let mut owner = Harness::new();
    let mut restored = Harness::new();
    owner.warm(seed.frame);
    restored.warm(seed.frame);
    state.store(&owner.ctx, owner.last.unwrap().id);
    owner.frame(Mode::OrderedShell, vec![], false);
    let state = State::load(&owner.ctx, owner.last.unwrap().id).unwrap();
    state.store(&restored.ctx, restored.last.unwrap().id);
    let before = state.unclamped_offset().y;
    let (after, _) = restored.frame(Mode::OrderedShell, vec![], false);
    assert!(
        after.offset > before,
        "matching pass numbers in distinct contexts cannot suppress time"
    );
}

#[test]
fn native_bar_capture_and_ordered_background_drag_survive_paint() {
    let mut h = Harness::new();
    h.warm(4);
    h.repeats = 3;
    let bar = h
        .ctx
        .read_response(h.last.unwrap().id.with(1))
        .unwrap()
        .rect;
    let p = bar.center();
    let (pressed, _) = h.frame(
        Mode::OrderedShell,
        vec![Event::PointerMoved(p), pointer(p, true)],
        false,
    );
    assert!(
        pressed.offset > 100.,
        "bar click reaches its native local handler"
    );
    let (moved, _) = h.frame(
        Mode::OrderedShell,
        vec![Event::PointerMoved(p + egui::vec2(0., 10.))],
        false,
    );
    assert!(
        moved.offset > pressed.offset,
        "paint must preserve the bar grab offset"
    );
    h.frame(
        Mode::OrderedShell,
        vec![pointer(p + egui::vec2(0., 10.), false)],
        false,
    );

    let mut h = Harness::new();
    h.warm(3);
    h.repeats = 3;
    let p = egui::pos2(150., 100.);
    h.frame(Mode::OrderedShell, vec![Event::PointerMoved(p)], true);
    h.frame(Mode::OrderedShell, vec![pointer(p, true)], true);
    for y in [80., 60., 40.] {
        h.frame(
            Mode::OrderedShell,
            vec![Event::PointerMoved(egui::pos2(150., y))],
            true,
        );
    }
    let before = h.last.unwrap();
    assert!(before.offset > 40.);
    let (released, _) = h.frame(
        Mode::OrderedShell,
        vec![pointer(egui::pos2(150., 40.), false)],
        true,
    );
    assert_eq!(
        released.offset, before.offset,
        "release installs velocity after this frame's time opportunity"
    );
    assert!(released.velocity > 100.);
    let (next, _) = h.frame(Mode::OrderedShell, vec![], true);
    assert!(next.offset > released.offset);
    assert!(next.velocity < released.velocity);
}

#[test]
fn mixed_gesture_modifiers_and_sources_take_effect_in_event_order() {
    let mut h = Harness::new();
    h.warm(3);
    h.ctx.options_mut(|o| o.ordered_input = true);
    let mut zoom = wheel(-40., egui::MouseWheelSource::Trackpad);
    if let Event::MouseWheel { modifiers, .. } = &mut zoom {
        *modifiers = Modifiers::COMMAND;
    }
    let mut end = touch_start();
    if let Event::MouseWheel { phase, .. } = &mut end {
        *phase = egui::TouchPhase::End;
    }
    let mut deltas = Vec::new();
    let mut result = None;
    let output = h.ctx.run_ui(
        RawInput {
            time: Some(3. / 60.),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(200., 120.))),
            events: vec![
                touch_start(),
                wheel(-40., egui::MouseWheelSource::Trackpad),
                zoom,
                end,
                wheel(-4., egui::MouseWheelSource::Wheel),
            ],
            ..Default::default()
        },
        |ui| {
            let mut counts = Counts::default();
            h.ctx.begin_input();
            shell(ui, true, true, &mut counts);
            while let Some(event) = h.ctx.next_input_event() {
                if matches!(
                    event.trigger,
                    InputTrigger::Event {
                        event: Event::MouseWheel { .. },
                        ..
                    }
                ) {
                    deltas.push(h.ctx.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta())));
                }
                shell(ui, true, false, &mut counts);
            }
            result = Some(shell(ui, true, true, &mut counts));
        },
    );
    output.drop_without_applying_deltas();
    assert_eq!(
        deltas.iter().map(|d| d.0).collect::<Vec<_>>(),
        [0., -40., 0., 0., -4.]
    );
    assert_ne!(deltas[2].1, 1.);
    assert_eq!(deltas[4].1, 1.);
    assert_eq!(result.unwrap().offset, 44.);
}
