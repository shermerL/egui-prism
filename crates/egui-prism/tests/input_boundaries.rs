//! Original boundary contracts exercised through ordered input.

use egui::{
    Context, Event, EventFilter, Id, Key, Modifiers, PointerButton, RawInput, Rect, Sense, pos2,
    vec2,
};
use egui_prism::egui;

fn id(n: usize) -> Id {
    Id::unique(("input-intake", n))
}
fn rect(n: usize) -> Rect {
    Rect::from_min_size(pos2(20. + 140. * n as f32, 30.), vec2(100., 30.))
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
fn click(n: usize) -> Vec<Event> {
    let pos = rect(n).center();
    vec![
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ]
}
struct Harness {
    ctx: Context,
    clicks: [usize; 2],
    arrows: [usize; 2],
    block_first: bool,
}
impl Harness {
    fn new() -> Self {
        let mut h = Self {
            ctx: Context::default(),
            clicks: [0; 2],
            arrows: [0; 2],
            block_first: false,
        };
        h.ctx.options_mut(|o| {
            o.ordered_input = true;
            o.max_passes = 1.try_into().unwrap();
        });
        h.frame(vec![], false);
        h.frame(vec![], false);
        h
    }
    fn frame(&mut self, events: Vec<Event>, surrender: bool) {
        let mut calls = 0;
        self.ctx
            .run_ui(
                RawInput {
                    events,
                    screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(500., 300.))),
                    ..Default::default()
                },
                |ui| {
                    calls += 1;
                    if surrender {
                        ui.memory_mut(|m| m.surrender_focus(id(0)));
                    }
                    ui.ctx().begin_input();
                    for n in 0..2 {
                        ui.interact(rect(n), id(n), Sense::click());
                        ui.memory_mut(|m| {
                            m.register_focus_policy(
                                id(n),
                                ui.layer_id(),
                                EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    escape: true,
                                    ..Default::default()
                                },
                            )
                        });
                    }
                    if self.block_first {
                        ui.interact(rect(0), id(2), Sense::click());
                    }
                    while let Some(event) = ui.ctx().next_input_event() {
                        if let Some(n) = (0..2).find(|&n| event.target == Some(id(n))) {
                            self.clicks[n] +=
                                usize::from(ui.interact(rect(n), id(n), Sense::click()).clicked());
                            self.arrows[n] += usize::from(matches!(
                                event.event,
                                Event::Key {
                                    key: Key::ArrowRight,
                                    pressed: true,
                                    ..
                                }
                            ));
                        }
                    }
                },
            )
            .drop_without_applying_deltas();
        assert_eq!(calls, 1);
    }
    fn focus(&self) -> Option<Id> {
        self.ctx.memory(|m| m.focused())
    }
}

#[test]
fn first_tab_then_immediate_escape_keeps_focus() {
    let mut h = Harness::new();
    h.frame(vec![key(Key::Tab, Modifiers::NONE)], false);
    assert_eq!(h.focus(), Some(id(0)));
    h.frame(vec![key(Key::Escape, Modifiers::NONE)], false);
    assert_eq!(h.focus(), Some(id(0)));
}
#[test]
fn same_batch_tab_escape_uses_new_target_policy() {
    let mut h = Harness::new();
    h.frame(
        vec![
            key(Key::Tab, Modifiers::NONE),
            key(Key::Escape, Modifiers::NONE),
        ],
        false,
    );
    assert_eq!(h.focus(), Some(id(0)));
}
#[test]
fn arrows_before_and_after_tab_reach_different_targets() {
    let mut h = Harness::new();
    h.ctx.memory_mut(|m| m.request_focus(id(0)));
    h.frame(vec![], false);
    h.frame(vec![], false);
    h.frame(
        vec![
            key(Key::ArrowRight, Modifiers::NONE),
            key(Key::Tab, Modifiers::NONE),
            key(Key::ArrowRight, Modifiers::NONE),
        ],
        false,
    );
    assert_eq!(h.arrows, [1, 1]);
}
#[test]
fn reverse_tab_then_arrow_does_not_edit_old_target() {
    let mut h = Harness::new();
    h.ctx.memory_mut(|m| m.request_focus(id(1)));
    h.frame(vec![], false);
    h.frame(vec![], false);
    h.frame(
        vec![
            key(Key::Tab, Modifiers::SHIFT),
            key(Key::ArrowRight, Modifiers::NONE),
        ],
        false,
    );
    assert_eq!(h.arrows, [1, 0]);
}
#[test]
fn two_clicks_in_one_batch_reach_both_targets() {
    let mut h = Harness::new();
    let mut events = click(0);
    events.extend(click(1));
    h.frame(events, false);
    assert_eq!(h.clicks, [1, 1]);
}
#[test]
fn click_then_move_does_not_retarget_click() {
    let mut h = Harness::new();
    let mut events = click(0);
    events.push(Event::PointerMoved(rect(1).center()));
    h.frame(events, false);
    assert_eq!(h.clicks, [1, 0]);
}
#[test]
fn same_layer_blocker_control() {
    let mut h = Harness::new();
    h.block_first = true;
    h.frame(vec![], false);
    h.frame(click(0), false);
    assert_eq!(h.clicks, [0, 0]);
    assert_eq!(h.ctx.interaction_snapshot(|s| s.clicked), Some(id(2)));
}
#[test]
fn explicit_host_surrender_control() {
    let mut h = Harness::new();
    h.ctx.memory_mut(|m| m.request_focus(id(0)));
    h.frame(vec![], false);
    h.frame(vec![key(Key::Escape, Modifiers::NONE)], true);
    assert_eq!(h.focus(), None);
}
