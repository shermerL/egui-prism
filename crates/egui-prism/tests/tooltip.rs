//! Public API regressions with a host-owned cursor and deterministic RawInput time.

use egui_prism::{
    compositing::{self, Group, Item},
    egui,
    theme::Theme,
    tooltip::{Tooltip, TooltipOutput, TooltipRegistration},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Default)]
struct Collector(Mutex<Vec<Group>>);
impl compositing::Backend for Collector {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}
fn id(index: usize) -> egui::Id {
    egui::Id::unique(("tooltip-test", index))
}
fn key(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}
fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn focus(index: usize) -> egui::Event {
    egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
        action: egui::accesskit::Action::Focus,
        target_tree: egui::accesskit::TreeId::ROOT,
        target_node: id(index).accesskit_id(),
        data: None,
    })
}
fn wheel() -> egui::Event {
    egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -40.0),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: egui::Modifiers::NONE,
    }
}

struct Harness {
    ctx: egui::Context,
    collector: Arc<Collector>,
    _installation: compositing::Installation,
    theme: Theme,
    rects: [egui::Rect; 2],
    enabled: [bool; 2],
    shown: [bool; 2],
    reverse: bool,
    delay: Duration,
    clip: egui::Rect,
    screen: egui::Rect,
    text: String,
    parent_opacity: f32,
    outputs: [Option<TooltipOutput>; 2],
    transitions: Vec<[bool; 2]>,
    groups: Vec<Group>,
    descriptions: [Option<String>; 2],
    labels: [Option<String>; 2],
    clicks: [usize; 2],
}
impl Harness {
    fn new() -> Self {
        let ctx = egui::Context::default();
        ctx.options_mut(|options| {
            options.ordered_input = true;
            options.max_passes = std::num::NonZeroUsize::MIN;
        });
        ctx.enable_accesskit();
        let collector = Arc::new(Collector::default());
        let installation = compositing::install(&ctx, collector.clone()).unwrap();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500.0, 400.0));
        Self {
            ctx,
            collector,
            _installation: installation,
            theme: Theme::light(),
            rects: [
                egui::Rect::from_min_size(egui::pos2(60.0, 180.0), egui::vec2(100.0, 36.0)),
                egui::Rect::from_min_size(egui::pos2(320.0, 180.0), egui::vec2(100.0, 36.0)),
            ],
            enabled: [true; 2],
            shown: [true; 2],
            reverse: false,
            delay: Duration::ZERO,
            clip: screen,
            screen,
            text: "A plain hint".into(),
            parent_opacity: 1.0,
            outputs: [None; 2],
            transitions: Vec::new(),
            groups: Vec::new(),
            descriptions: [None, None],
            labels: [None, None],
            clicks: [0; 2],
        }
    }
    fn frame(&mut self, time: f64, events: Vec<egui::Event>) {
        self.collector.0.lock().unwrap().clear();
        self.outputs = [None; 2];
        self.transitions.clear();
        let ctx = self.ctx.clone();
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(self.screen),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                ui.set_clip_rect(self.clip);
                let mut painter = ui.painter().clone();
                painter.set_opacity(self.parent_opacity);
                let mut tips = Vec::new();
                let mut indices = Vec::new();
                for index in if self.reverse { [1, 0] } else { [0, 1] } {
                    if !self.shown[index] {
                        continue;
                    }
                    let response = ui.interact(self.rects[index], id(index), egui::Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::Button,
                            true,
                            format!("Action {index}"),
                        )
                    });
                    ctx.accesskit_node_builder(id(index), |node| {
                        node.set_description("Existing description")
                    });
                    ctx.memory_mut(|memory| {
                        memory.register_focus_policy(
                            id(index),
                            response.layer_id,
                            egui::EventFilter {
                                escape: true,
                                ..Default::default()
                            },
                        )
                    });
                    tips.push(
                        Tooltip::new(&self.theme, &response, &self.text)
                            .enabled(self.enabled[index])
                            .delay(self.delay)
                            .register(ui),
                    );
                    indices.push(index);
                }
                while let Some(event) = ctx.next_input_event() {
                    let focused = ctx.memory(|memory| memory.focused());
                    let modal = ctx.memory(|memory| memory.top_modal_layer());
                    let untouched = ctx.input(|input| input.events.clone());
                    TooltipRegistration::handle_all(&mut tips, &event);
                    assert_eq!(
                        focused,
                        ctx.memory(|memory| memory.focused()),
                        "tooltip must not change focus"
                    );
                    assert_eq!(
                        modal,
                        ctx.memory(|memory| memory.top_modal_layer()),
                        "tooltip must not become modal"
                    );
                    assert_eq!(
                        untouched,
                        ctx.input(|input| input.events.clone()),
                        "tooltip must not consume input"
                    );
                    let mut opened = [false; 2];
                    for (tip, &index) in tips.iter().zip(&indices) {
                        opened[index] = tip.output().open;
                        if event.target == Some(id(index))
                            && ctx.read_response(id(index)).unwrap().clicked()
                        {
                            self.clicks[index] += 1;
                        }
                    }
                    self.transitions.push(opened);
                }
                for (tip, &index) in tips.iter().zip(&indices) {
                    self.outputs[index] = Some(tip.paint(&painter).unwrap());
                }
            },
        );
        if let Some(update) = &output.platform_output.accesskit_update {
            for index in 0..2 {
                if let Some((_, node)) = update
                    .nodes
                    .iter()
                    .find(|(node, _)| *node == id(index).accesskit_id())
                {
                    self.descriptions[index] = node.description().map(str::to_owned);
                    self.labels[index] = node.label().map(str::to_owned);
                }
            }
        }
        output.drop_without_applying_deltas();
        self.groups = self.collector.0.lock().unwrap().clone();
    }
    fn open(&self, index: usize) -> bool {
        self.outputs[index].is_some_and(|output| output.open)
    }
    fn hover(&mut self, time: f64, index: usize) {
        self.frame(
            time,
            vec![egui::Event::PointerMoved(self.rects[index].center())],
        );
    }
}

#[test]
fn default_zero_delay_and_isolated_reference_animation() {
    let mut h = Harness::new();
    h.parent_opacity = 0.5;
    h.theme.colors.background.foreground = egui::Color32::from_rgb(12, 34, 56);
    h.theme.colors.background.background = egui::Color32::from_rgb(210, 220, 230);
    h.hover(0.0, 0);
    assert!(h.open(0));
    assert!(
        h.groups.is_empty(),
        "initial zero-opacity group need not be submitted"
    );
    h.frame(0.075, vec![]);
    assert_eq!(h.groups.len(), 1);
    let middle = &h.groups[0];
    assert!(
        middle.opacity() > 0.25 && middle.opacity() < 0.5,
        "CSS ease with inherited opacity"
    );
    let mut rectangle = false;
    let mut transformed = false;
    for item in middle.items() {
        if let Item::Shape(shape) = item {
            transformed |= shape.transform_after_tessellation.scaling < 1.0;
            if let egui::Shape::Rect(rect) = &shape.shape {
                rectangle = true;
                assert_eq!(
                    rect.fill, h.theme.colors.background.foreground,
                    "unfaded foreground background"
                );
                assert_eq!(rect.fill.a(), 255);
            }
        }
    }
    assert!(rectangle && transformed);
    h.frame(0.15, vec![]);
    assert_eq!(h.groups[0].opacity(), 0.5);
    assert!(h.groups[0].items().iter().all(|item| match item {
        Item::Shape(shape) => shape.transform_after_tessellation.scaling == 1.0,
        _ => false,
    }));
    h.frame(0.2, vec![key(egui::Key::Escape, true)]);
    assert!(!h.open(0));
    assert!(
        h.outputs[0].unwrap().content_rect.is_some(),
        "exit presence"
    );
    h.frame(0.351, vec![]);
    assert!(h.outputs[0].unwrap().content_rect.is_none());
    assert!(h.groups.is_empty());
}

#[test]
fn controlled_delay_cancel_and_provider_skip_window() {
    let mut h = Harness::new();
    h.delay = Duration::from_millis(500);
    h.hover(0.0, 0);
    h.frame(0.499, vec![]);
    assert!(!h.open(0));
    h.frame(0.5, vec![]);
    assert!(h.open(0));
    h.frame(0.6, vec![egui::Event::PointerGone]);
    h.hover(0.7, 1);
    assert!(h.open(1), "peer inherits the provider skip window");
    h.frame(0.8, vec![egui::Event::PointerGone]);
    h.hover(1.2, 0);
    assert!(!h.open(0));
    h.frame(1.3, vec![egui::Event::PointerGone]);
    h.frame(2.0, vec![]);
    assert!(!h.open(0), "leaving cancels the pending deadline");
    h.hover(2.1, 0);
    h.frame(2.601, vec![]);
    assert!(h.open(0));
}

#[test]
fn keyboard_focus_opens_immediately_blur_closes_and_escape_does_not_steal_focus() {
    let mut h = Harness::new();
    h.delay = Duration::from_secs(5);
    h.frame(0.0, vec![focus(0)]);
    assert!(h.open(0));
    assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(id(0)));
    h.frame(0.01, vec![key(egui::Key::Escape, true)]);
    assert!(!h.open(0));
    assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(id(0)));
    h.frame(0.02, vec![]);
    assert!(!h.open(0), "unchanged focus must not reopen after Escape");
    h.frame(0.03, vec![focus(1)]);
    assert!(!h.open(0));
    assert!(h.open(1));
    h.frame(0.04, vec![egui::Event::WindowFocused(false)]);
    assert!(!h.open(1));
}

#[test]
fn click_dismissal_preserves_trigger_activation_and_does_not_reopen_on_pointer_focus() {
    let mut h = Harness::new();
    h.hover(0.0, 0);
    let pos = h.rects[0].center();
    h.frame(0.2, vec![pointer(pos, true), pointer(pos, false)]);
    assert!(!h.open(0));
    assert_eq!(h.clicks, [1, 0]);
    h.frame(
        0.3,
        vec![egui::Event::PointerMoved(pos + egui::vec2(1.0, 0.0))],
    );
    assert!(!h.open(0), "Radix permits one pointer opening until leave");
    h.frame(0.4, vec![egui::Event::PointerGone]);
    h.hover(0.5, 0);
    assert!(h.open(0));
    let other = h.rects[1].center();
    h.frame(
        0.6,
        vec![
            egui::Event::PointerMoved(other),
            pointer(other, true),
            pointer(other, false),
        ],
    );
    assert_eq!(h.clicks, [1, 1], "no outside-click modal barrier");
    assert!(!h.open(0) && !h.open(1));
}

#[test]
fn both_directions_of_hover_transit_and_stationary_content() {
    let mut h = Harness::new();
    h.hover(0.0, 0);
    h.frame(0.2, vec![]);
    let body = h.outputs[0].unwrap().content_rect.unwrap();
    let gap = egui::pos2(h.rects[0].center().x, h.rects[0].top() - 2.0);
    h.frame(0.21, vec![egui::Event::PointerMoved(gap)]);
    assert!(h.open(0));
    h.frame(0.22, vec![egui::Event::PointerMoved(body.center())]);
    h.frame(10.0, vec![]);
    assert!(
        h.open(0),
        "hoverable, noninteractive content has no arbitrary timeout"
    );
    h.frame(
        10.01,
        vec![egui::Event::PointerMoved(egui::pos2(
            body.center().x,
            body.bottom() + 2.0,
        ))],
    );
    assert!(h.open(0));
    h.hover(10.02, 0);
    assert!(h.open(0));
    h.frame(
        10.03,
        vec![egui::Event::PointerMoved(egui::pos2(490.0, 390.0))],
    );
    assert!(!h.open(0), "leaving the convex corridor closes immediately");
}

#[test]
fn independent_ids_reordering_and_peer_open_dismissal() {
    let mut h = Harness::new();
    h.hover(0.0, 0);
    h.reverse = true;
    h.frame(0.2, vec![]);
    assert!(h.open(0) && !h.open(1));
    h.frame(0.3, vec![focus(1)]);
    assert!(!h.open(0) && h.open(1));
    h.frame(0.4, vec![focus(0)]);
    assert!(h.open(0) && !h.open(1));
}

#[test]
fn disable_and_missing_frame_cancel_pending_open_and_exit_drawing() {
    let mut h = Harness::new();
    h.delay = Duration::from_millis(500);
    h.hover(0.0, 0);
    h.enabled[0] = false;
    h.frame(0.2, vec![]);
    h.enabled[0] = true;
    h.frame(1.0, vec![]);
    assert!(!h.open(0));
    h.hover(1.1, 0);
    h.shown[0] = false;
    h.frame(1.2, vec![]);
    h.shown[0] = true;
    h.frame(2.0, vec![]);
    assert!(!h.open(0), "a missing frame invalidates the old deadline");
    h.hover(2.1, 0);
    h.frame(2.601, vec![]);
    assert!(h.open(0));
    h.enabled[0] = false;
    h.frame(2.7, vec![]);
    assert!(h.outputs[0].unwrap().content_rect.is_none());
}

#[test]
fn clipped_hit_authority_and_fully_clipped_focus_cancellation() {
    let mut h = Harness::new();
    h.clip = egui::Rect::from_min_max(egui::pos2(100.0, 0.0), h.screen.max);
    h.frame(
        0.0,
        vec![egui::Event::PointerMoved(egui::pos2(80.0, 190.0))],
    );
    assert!(!h.open(0), "unclipped trigger rect is not hit authority");
    h.frame(
        0.1,
        vec![egui::Event::PointerMoved(egui::pos2(120.0, 190.0))],
    );
    assert!(h.open(0));
    h.clip = egui::Rect::from_min_max(egui::pos2(200.0, 0.0), h.screen.max);
    h.frame(0.2, vec![focus(0)]);
    assert!(!h.open(0));
    assert!(h.outputs[0].unwrap().content_rect.is_none());
}

#[test]
fn long_text_preserves_natural_width_with_viewport_bounded_drawing() {
    for (x, y) in [(0.0, 0.0), (400.0, 0.0), (0.0, 360.0), (400.0, 360.0)] {
        let mut h = Harness::new();
        h.text = "A long plain hint with words and an_unbroken_token_that_must_not_escape_the_viewport. ".repeat(100);
        h.rects[0] = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(100.0, 36.0));
        h.hover(0.0, 0);
        let body = h.outputs[0].unwrap().content_rect.unwrap();
        assert!(
            body.width() > h.screen.width(),
            "reference max-content width"
        );
        assert_eq!(body.height(), 28.0, "no added wrapping or truncation");
        assert_eq!(body.left(), h.screen.left());
        if y == 0.0 {
            assert!(body.top() >= h.rects[0].bottom());
        } else {
            assert!(body.bottom() <= h.rects[0].top());
        }
        h.frame(0.2, vec![]);
        assert!(
            h.groups
                .iter()
                .all(|group| h.screen.contains_rect(group.bounds()))
        );
        assert_eq!(
            h.descriptions[0].as_deref(),
            Some(format!("Existing description\n{}", h.text).as_str())
        );
        assert!(
            h.groups
                .iter()
                .flat_map(Group::items)
                .all(|item| match item {
                    Item::Shape(shape) => h.screen.contains_rect(shape.clip_rect),
                    _ => false,
                })
        );
    }
}

#[test]
fn accessibility_augments_description_without_replacing_label() {
    let mut h = Harness::new();
    h.frame(0.0, vec![focus(0)]);
    assert_eq!(h.labels[0].as_deref(), Some("Action 0"));
    assert_eq!(
        h.descriptions[0].as_deref(),
        Some("Existing description\nA plain hint")
    );
    h.frame(0.1, vec![key(egui::Key::Escape, true)]);
    assert_eq!(h.labels[0].as_deref(), Some("Action 0"));
    assert_eq!(h.descriptions[0].as_deref(), Some("Existing description"));
}

#[test]
fn wheel_is_not_consumed_and_actual_anchor_motion_cancels() {
    let mut h = Harness::new();
    h.hover(0.0, 0);
    h.frame(0.2, vec![wheel()]);
    assert!(
        h.open(0),
        "wheel alone is not an ancestor scroll notification"
    );
    h.rects[0] = h.rects[0].translate(egui::vec2(0.0, -20.0));
    h.frame(0.3, vec![]);
    assert!(!h.open(0));
}

#[test]
fn touch_synthetic_pointer_does_not_open_hover_hint() {
    let mut h = Harness::new();
    let pos = h.rects[0].center();
    h.frame(
        0.0,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::Touch {
                device_id: egui::TouchDeviceId(0),
                id: egui::TouchId(0),
                phase: egui::TouchPhase::Start,
                pos,
                force: None,
            },
            pointer(pos, true),
        ],
    );
    assert!(!h.open(0));
}

#[test]
fn same_batch_retirement_does_not_remove_trigger_or_revive_old_deadline() {
    let ctx = egui::Context::default();
    ctx.options_mut(|options| options.ordered_input = true);
    let theme = Theme::light();
    let rect = egui::Rect::from_min_size(egui::pos2(30.0, 120.0), egui::vec2(120.0, 36.0));
    ctx.run_ui(
        egui::RawInput {
            time: Some(0.0),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            events: vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::Text("retire".into()),
                pointer(rect.center(), true),
                pointer(rect.center(), false),
            ],
            ..Default::default()
        },
        |ui| {
            ctx.begin_input();
            let response = ui.interact(rect, id(0), egui::Sense::click());
            let mut tips = vec![
                Tooltip::new(&theme, &response, "Hint")
                    .delay(Duration::from_secs(1))
                    .register(ui),
            ];
            let mut clicks = 0;
            while let Some(event) = ctx.next_input_event() {
                TooltipRegistration::handle_all(&mut tips, &event);
                if matches!(
                    &event.trigger,
                    egui::InputTrigger::Event {
                        event: egui::Event::Text(_),
                        ..
                    }
                ) {
                    tips[0].unregister();
                    assert!(ctx.read_response(id(0)).is_some());
                    ctx.input_layout(|| {
                        tips[0] = Tooltip::new(&theme, &response, "Replacement").register(ui);
                    });
                }
                if event.target == Some(id(0)) && ctx.read_response(id(0)).unwrap().clicked() {
                    clicks += 1;
                }
            }
            assert_eq!(clicks, 1);
            assert!(!tips[0].output().open);
            tips[0].dismiss();
            assert!(!tips[0].output().open);
        },
    )
    .drop_without_applying_deltas();
}
