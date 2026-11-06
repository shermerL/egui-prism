//! Authoritative shape, clip, layer and composite-owner routing.
use egui::{
    Context, Event, Id, LayerId, Modifiers, Order, RawInput, Rect, Sense, UiBuilder, pos2, vec2,
};
use egui_prism::egui;
fn id(n: usize) -> Id {
    Id::unique(("geometry", n))
}
fn click(pos: egui::Pos2) -> Vec<Event> {
    [true, false]
        .map(|pressed| Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        })
        .to_vec()
}
fn context() -> Context {
    let c = Context::default();
    c.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    c
}
fn input(events: Vec<Event>) -> RawInput {
    RawInput {
        events,
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(600.0, 400.0))),
        ..Default::default()
    }
}
#[test]
fn composite_rail_and_round_thumb_do_not_swallow_empty_corners() {
    let ctx = context();
    let rail = Rect::from_min_size(pos2(20.0, 40.0), vec2(200.0, 6.0));
    let thumb = Rect::from_center_size(pos2(120.0, 43.0), vec2(16.0, 16.0));
    let positions = [
        pos2(21.0, 36.0),
        thumb.min + vec2(0.5, 0.5),
        rail.left_center() + vec2(1.0, 0.0),
        thumb.center(),
    ];
    let mut targets = Vec::new();
    ctx.run_ui(
        input(positions.into_iter().flat_map(click).collect()),
        |ui| {
            ctx.begin_input();
            ui.interact(rail.union(thumb), id(9), Sense::click());
            ui.interact(rail, id(1), Sense::click_and_drag());
            ui.interact(thumb, id(0), Sense::click_and_drag());
            ctx.set_input_owner(id(1), id(0));
            ctx.set_input_rounding(id(1), 3.0);
            ctx.set_input_rounding(id(0), 8.0);
            while let Some(e) = ctx.next_input_event() {
                if matches!(e.event, Event::PointerButton { pressed: false, .. }) {
                    targets.push(e.target);
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(
        targets,
        vec![Some(id(9)), Some(id(9)), Some(id(0)), Some(id(0))]
    );
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(0)));
}
#[test]
fn transformed_clipped_hit_and_same_layer_blocker() {
    let ctx = context();
    let layer = LayerId::new(Order::Foreground, id(30));
    let bounds = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 100.0));
    let transform = egui::emath::TSTransform::new(vec2(100.0, 80.0), 2.0);
    let positions = [pos2(10.0, 10.0), pos2(50.0, 50.0), pos2(90.0, 50.0)];
    let mut targets = Vec::new();
    ctx.run_ui(
        input(
            positions
                .into_iter()
                .flat_map(|p| click(transform * p))
                .collect(),
        ),
        |ui| {
            ctx.begin_input();
            ui.interact(Rect::EVERYTHING, id(9), Sense::click());
            ctx.set_transform_layer(layer, transform);
            let mut child = egui::Ui::new(
                ctx.clone(),
                id(30),
                UiBuilder::new().layer_id(layer).max_rect(bounds),
            );
            child.set_clip_rect(Rect::from_min_max(pos2(0.0, 0.0), pos2(60.0, 100.0)));
            child.interact(bounds, id(0), Sense::click());
            child.interact(
                Rect::from_min_max(pos2(30.0, 30.0), pos2(60.0, 70.0)),
                id(1),
                Sense::click(),
            );
            while let Some(e) = ctx.next_input_event() {
                if matches!(e.event, Event::PointerButton { pressed: false, .. }) {
                    targets.push(e.target);
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(targets, vec![Some(id(0)), Some(id(1)), Some(id(9))]);
}
#[test]
fn composite_capture_release_does_not_reclaim_host_focus() {
    let ctx = context();
    let rail = Rect::from_min_size(pos2(20.0, 40.0), vec2(200.0, 6.0));
    let thumb = Rect::from_center_size(pos2(120.0, 43.0), vec2(16.0, 16.0));
    let events = click(rail.left_center() + vec2(2.0, 0.0));
    let mut stopped = 0;
    ctx.run_ui(input(events), |ui| {
        ctx.begin_input();
        ui.interact(rail, id(1), Sense::drag());
        ui.interact(thumb, id(0), Sense::drag());
        ui.interact(
            Rect::from_min_size(pos2(20.0, 100.0), vec2(100.0, 30.0)),
            id(2),
            Sense::click(),
        );
        ctx.set_input_owner(id(1), id(0));
        while let Some(e) = ctx.next_input_event() {
            assert_eq!(e.target, Some(id(0)));
            if matches!(e.event, Event::PointerButton { pressed: true, .. }) {
                assert_eq!(ctx.memory(|m| m.focused()), Some(id(0)));
                ctx.memory_mut(|m| m.request_focus(id(2)));
            } else {
                stopped += usize::from(ui.interact(thumb, id(0), Sense::drag()).drag_stopped());
            }
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(stopped, 1);
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(2)));
}

#[test]
fn release_in_a_rounded_corner_cancels_click_but_reaches_capture_owner() {
    let ctx = context();
    let bounds = Rect::from_min_size(pos2(20.0, 20.0), vec2(16.0, 16.0));
    let events = [(pos2(23.0, 23.0), true), (pos2(21.0, 21.0), false)]
        .map(|(pos, pressed)| Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: Modifiers::NONE,
        })
        .to_vec();
    let mut clicks = 0;
    let mut releases = 0;
    ctx.run_ui(input(events), |ui| {
        ctx.begin_input();
        ui.interact(bounds, id(0), Sense::click());
        ctx.set_input_rounding(id(0), 8.0);
        while let Some(event) = ctx.next_input_event() {
            if event.target == Some(id(0)) {
                let response = ui.interact(bounds, id(0), Sense::click());
                clicks += usize::from(response.clicked());
                releases += usize::from(matches!(
                    event.event,
                    Event::PointerButton { pressed: false, .. }
                ));
            }
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(releases, 1);
    assert_eq!(clicks, 0);
}

#[test]
fn removing_composite_owner_removes_its_parts_without_touching_other_focus() {
    let ctx = context();
    let bounds = Rect::from_min_size(pos2(20.0, 20.0), vec2(100.0, 30.0));
    let mut releases = Vec::new();
    ctx.run_ui(input(click(bounds.center())), |ui| {
        ctx.begin_input();
        ui.interact(bounds, id(1), Sense::drag());
        ui.interact(bounds.translate(vec2(120.0, 0.0)), id(0), Sense::drag());
        ui.interact(bounds.translate(vec2(0.0, 100.0)), id(2), Sense::click());
        ctx.set_input_owner(id(1), id(0));
        while let Some(event) = ctx.next_input_event() {
            if matches!(event.event, Event::PointerButton { pressed: true, .. }) {
                ctx.memory_mut(|m| m.request_focus(id(2)));
                ctx.remove_input_widget(id(0));
            } else {
                releases.push(event.target);
            }
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(releases, vec![None]);
    assert_eq!(ctx.memory(|m| m.focused()), Some(id(2)));
}

#[test]
fn button_click_wins_over_a_containing_drag_candidate() {
    let ctx = context();
    let button = Rect::from_min_size(pos2(40.0, 40.0), vec2(100.0, 30.0));
    let container = button.expand(20.0);
    let mut clicks = 0;
    let mut releases = Vec::new();
    ctx.run_ui(input(click(button.center())), |ui| {
        ctx.begin_input();
        ui.interact(container, id(1), Sense::drag());
        ui.interact(button, id(0), Sense::click());
        while let Some(event) = ctx.next_input_event() {
            if matches!(event.event, Event::PointerButton { pressed: false, .. }) {
                releases.push(event.target);
            }
            if event.target == Some(id(0)) {
                clicks += usize::from(ui.interact(button, id(0), Sense::click()).clicked());
            }
        }
    })
    .drop_without_applying_deltas();
    assert_eq!(releases, vec![Some(id(0))]);
    assert_eq!(clicks, 1);
}
