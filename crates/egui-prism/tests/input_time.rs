//! Public ordered-time opt-in uses the existing frame budget, not a timer.

use egui_prism::egui::{self, Context, Event, InputTrigger, RawInput, RoutedInput, Ui};

fn context() -> Context {
    let ctx = Context::default();
    ctx.options_mut(|options| {
        options.ordered_input = true;
        options.max_passes = std::num::NonZeroUsize::MIN;
    });
    ctx
}

fn raw(events: Vec<Event>) -> RawInput {
    RawInput {
        events,
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 300.0),
        )),
        ..Default::default()
    }
}

fn assert_time(ctx: &Context, routed: &RoutedInput) {
    assert_eq!(routed.target, None);
    assert!(matches!(routed.trigger, InputTrigger::Time));
    assert!(ctx.input(|input| input.events.is_empty()));
}

fn scroll(ui: &mut Ui) {
    let mut shell = ui.new_child(
        egui::UiBuilder::new()
            .scope_id(egui::Id::unique("time-scroll-shell"))
            .max_rect(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
    );
    egui::ScrollArea::vertical()
        .id_salt("time-scroll")
        .max_height(120.0)
        .auto_shrink([false, false])
        .show(&mut shell, |ui| {
            ui.set_min_size(egui::vec2(180.0, 600.0));
        });
}

#[test]
fn empty_frame_time_is_opt_in_and_must_be_renewed() {
    let ctx = context();
    for requested in [false, true, false] {
        let mut times = 0;
        ctx.run_ui(raw(vec![]), |_| {
            ctx.begin_input();
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            if requested {
                ctx.request_input_time();
            }
            while let Some(routed) = ctx.next_input_event() {
                assert_time(&ctx, &routed);
                times += 1;
            }
        })
        .drop_without_applying_deltas();
        assert_eq!(times, usize::from(requested));
    }
}

#[test]
fn repeated_requests_coalesce_without_consuming_raw_indices() {
    let ctx = context();
    let events = vec![Event::Text("first".into()), Event::Text("second".into())];
    ctx.run_ui(raw(events.clone()), |_| {
        ctx.begin_input();
        for _ in 0..3 {
            ctx.request_input_time();
        }
        assert_time(&ctx, &ctx.next_input_event().unwrap());
        assert_eq!(ctx.input(|input| input.raw.events.clone()), events);
        for (expected_index, expected) in events.iter().enumerate() {
            let routed = ctx.next_input_event().unwrap();
            let InputTrigger::Event { index, event } = routed.trigger else {
                panic!("requests must share one untargeted opportunity");
            };
            assert_eq!(index, expected_index);
            assert_eq!(&event, expected);
            assert_eq!(
                ctx.input(|input| input.events.clone()),
                vec![expected.clone()]
            );
        }
        assert!(ctx.next_input_event().is_none());
    })
    .drop_without_applying_deltas();
}

#[test]
fn custom_consumers_and_scroll_area_share_one_opportunity() {
    for scroll_first in [false, true] {
        let ctx = context();
        ctx.run_ui(raw(vec![]), |ui| {
            ctx.begin_input();
            if scroll_first {
                scroll(ui);
            }
            // Two independent consumers opt in without a shared registry.
            ctx.request_input_time();
            ctx.request_input_time();
            if !scroll_first {
                scroll(ui);
            }
            let mut ticks = [0; 2];
            let mut scroll_ticks = 0;
            while let Some(routed) = ctx.next_input_event() {
                assert_time(&ctx, &routed);
                for tick in &mut ticks {
                    *tick += 1;
                }
                scroll(ui);
                scroll_ticks += 1;
            }
            assert_eq!(ticks, [1, 1]);
            assert_eq!(scroll_ticks, 1);
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn discard_continuation_cannot_request_a_second_time_budget() {
    for with_scroll in [false, true] {
        let ctx = context();
        ctx.options_mut(|options| options.max_passes = 2.try_into().unwrap());
        let mut counts = Vec::new();
        let mut raw_events = 0;
        let output = ctx.run_ui(raw(vec![Event::Text("once".into())]), |ui| {
            ctx.begin_input();
            ctx.request_input_time();
            ctx.request_input_time();
            if with_scroll {
                scroll(ui);
            }
            let mut times = 0;
            while let Some(routed) = ctx.next_input_event() {
                match &routed.trigger {
                    InputTrigger::Time => {
                        assert_time(&ctx, &routed);
                        times += 1;
                        if with_scroll {
                            scroll(ui);
                        }
                    }
                    InputTrigger::Event { index, .. } => {
                        assert_eq!(ctx.current_pass_index(), 0);
                        assert_eq!(*index, 0);
                        raw_events += 1;
                    }
                }
            }
            counts.push(times);
            if ctx.current_pass_index() == 0 {
                ctx.request_discard("time requests on a layout-only continuation");
            }
        });
        assert_eq!(output.platform_output.num_completed_passes, 2);
        output.drop_without_applying_deltas();
        assert_eq!(counts, vec![1, 0]);
        assert_eq!(raw_events, 1);
    }
}

#[test]
fn layout_refresh_after_dispatch_cannot_arm_or_rearm_time() {
    for initially_requested in [false, true] {
        let ctx = context();
        ctx.run_ui(
            raw(vec![
                Event::Text("first".into()),
                Event::Text("second".into()),
            ]),
            |ui| {
                ctx.begin_input();
                if initially_requested {
                    ctx.request_input_time();
                }
                let first = ctx.next_input_event().unwrap();
                let mut next_index = if initially_requested {
                    assert_time(&ctx, &first);
                    0
                } else {
                    assert!(matches!(
                        first.trigger,
                        InputTrigger::Event { index: 0, .. }
                    ));
                    1
                };
                let active_events = ctx.input(|input| input.events.clone());
                ctx.input_layout(|| {
                    assert!(ctx.input(|input| input.events.is_empty()));
                    ctx.request_input_time();
                    ctx.request_input_time();
                    scroll(ui);
                });
                assert_eq!(ctx.input(|input| input.events.clone()), active_events);
                while let Some(routed) = ctx.next_input_event() {
                    let InputTrigger::Event { index, .. } = routed.trigger else {
                        panic!("layout refresh must not create another time opportunity");
                    };
                    assert_eq!(index, next_index);
                    next_index += 1;
                }
                assert_eq!(next_index, 2);
            },
        )
        .drop_without_applying_deltas();
    }
}

#[test]
#[should_panic(expected = "enable ordered input before using its phases")]
fn request_requires_ordered_mode() {
    Context::default().request_input_time();
}

#[test]
#[should_panic(expected = "request input time between begin_input and the end of dispatch")]
fn request_requires_begin_input() {
    let ctx = context();
    ctx.run_ui(raw(vec![]), |_| ctx.request_input_time())
        .drop_without_applying_deltas();
}

#[test]
#[should_panic(expected = "request input time during registration, not event handling")]
fn request_is_not_an_event_phase_action() {
    let ctx = context();
    ctx.run_ui(raw(vec![]), |_| {
        ctx.begin_input();
        ctx.request_input_time();
        assert_time(&ctx, &ctx.next_input_event().unwrap());
        ctx.request_input_time();
    })
    .drop_without_applying_deltas();
}

#[test]
#[should_panic(expected = "request input time between begin_input and the end of dispatch")]
fn request_is_not_a_paint_phase_action() {
    let ctx = context();
    ctx.run_ui(raw(vec![]), |_| {
        ctx.begin_input();
        assert!(ctx.next_input_event().is_none());
        ctx.request_input_time();
    })
    .drop_without_applying_deltas();
}
