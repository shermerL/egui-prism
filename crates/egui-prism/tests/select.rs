//! Public-API behavioral tests using real ordered Context phases.
//!
//! The collector verifies immutable drawings, not GPU rendering or browser
//! parity. Native focus, accessibility, and visual acceptance remain separate.

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use egui_prism::{
    compositing::{self, Group, Item},
    egui,
    select::{Select, SelectItem, SelectSize},
    theme::Theme,
};

#[derive(Default)]
struct Collector(Mutex<Vec<Group>>);

impl compositing::Backend for Collector {
    fn callback(
        &self,
        _context: &egui::Context,
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

fn context() -> egui::Context {
    let context = egui::Context::default();
    context.options_mut(|options| {
        options.ordered_input = true;
        options.max_passes = std::num::NonZeroUsize::MIN;
    });
    context
}

fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
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

fn wheel(delta: f32) -> egui::Event {
    egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, delta),
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Wheel,
        modifiers: egui::Modifiers::NONE,
    }
}

fn visit(group: &Group, visitor: &mut impl FnMut(&egui::epaint::ClippedShape)) {
    for item in group.items() {
        match item {
            Item::Group(group) => visit(group, visitor),
            Item::Shape(shape) => visitor(shape),
        }
    }
}

fn text_rect(group: &Group, label: &str) -> Option<egui::Rect> {
    let mut result = None;
    visit(group, &mut |shape| {
        if let egui::Shape::Text(text) = &shape.shape
            && text.galley.job.text == label
        {
            result = Some(
                shape.transform_after_tessellation * text.galley.rect.translate(text.pos.to_vec2()),
            );
        }
    });
    result
}

fn near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {expected}, got {actual}"
    );
}

struct Harness {
    ctx: egui::Context,
    theme: Theme,
    collector: Arc<Collector>,
    _installation: compositing::Installation,
    items: Vec<SelectItem<u32>>,
    selected: Option<u32>,
    changes: usize,
    enabled: bool,
    present: bool,
    time: f64,
    screen: egui::Vec2,
    origin: egui::Pos2,
    clip: Option<egui::Rect>,
    transform: Option<egui::emath::TSTransform>,
    cover: Option<egui::Rect>,
    size: SelectSize,
    width: Option<f32>,
    invalid: bool,
}

impl Harness {
    fn new() -> Self {
        let ctx = context();
        let collector = Arc::new(Collector::default());
        let installation = compositing::install(&ctx, collector.clone()).unwrap();
        Self {
            ctx,
            theme: Theme::light(),
            collector,
            _installation: installation,
            items: vec![
                SelectItem::new(0, "Apple"),
                SelectItem::new(1, "Apricot"),
                SelectItem::new(2, "Banana"),
                SelectItem::new(3, "Blueberry"),
                SelectItem::new(4, "Cherry"),
            ],
            selected: None,
            changes: 0,
            enabled: true,
            present: true,
            time: 0.0,
            screen: egui::vec2(500.0, 500.0),
            origin: egui::pos2(80.0, 100.0),
            clip: None,
            transform: None,
            cover: None,
            size: SelectSize::Default,
            width: Some(192.0),
            invalid: false,
        }
    }

    fn id(&self) -> egui::Id {
        egui::Id::unique("select-under-test")
    }

    fn drawings(&self) -> Vec<Group> {
        self.collector.0.lock().unwrap().clone()
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> Option<egui::Response> {
        self.frame_after(1.0 / 60.0, events)
    }

    fn frame_after(&mut self, elapsed: f64, events: Vec<egui::Event>) -> Option<egui::Response> {
        self.time += elapsed;
        self.collector.0.lock().unwrap().clear();
        let ctx = self.ctx.clone();
        let id = self.id();
        let mut result = None;

        ctx.run_ui(
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.screen)),
                events,
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                let clean_painter = ui.painter().clone();
                if let Some(transform) = self.transform {
                    ctx.set_transform_layer(ui.layer_id(), transform);
                }
                if !self.present {
                    while ctx.next_input_event().is_some() {}
                    return;
                }

                let mut layout = ui.new_child(
                    egui::UiBuilder::new()
                        .scope_id(egui::Id::unique("select-layout"))
                        .max_rect(egui::Rect::from_min_size(
                            self.origin,
                            egui::vec2(300.0, 300.0),
                        )),
                );
                if let Some(clip) = self.clip {
                    layout.set_clip_rect(clip);
                }
                if !self.enabled {
                    layout.disable();
                }
                let mut builder = Select::new(&self.theme, id, &self.items)
                    .placeholder("Choose an option")
                    .size(self.size)
                    .invalid(self.invalid);
                if let Some(width) = self.width {
                    builder = builder.width(width);
                }
                let mut select = builder.register(&mut layout, &self.selected);

                if let Some(cover) = self.cover {
                    let overlay = ui.new_child(
                        egui::UiBuilder::new()
                            .scope_id(egui::Id::unique("cover-ui"))
                            .layer_id(egui::LayerId::new(
                                egui::Order::Tooltip,
                                egui::Id::unique("cover-layer"),
                            ))
                            .max_rect(cover),
                    );
                    overlay.interact(
                        cover,
                        egui::Id::unique("cover"),
                        egui::Sense::click_and_drag(),
                    );
                }

                while let Some(event) = ctx.next_input_event() {
                    let modal_before = ctx.memory(|memory| memory.top_modal_layer());
                    if select.handle(&event, &mut self.selected).changed() {
                        self.changes += 1;
                    }
                    if modal_before.is_some()
                        && matches!(
                            &event.trigger,
                            egui::InputTrigger::Event {
                                event: egui::Event::MouseWheel { .. },
                                ..
                            }
                        )
                    {
                        // This is where the host background scroll handler
                        // runs, after the owner has handled the current event.
                        assert!(!ctx.input(|input| {
                            input
                                .events
                                .iter()
                                .any(|event| matches!(event, egui::Event::MouseWheel { .. }))
                        }));
                    }
                }
                result = Some(select.paint(&self.selected, &clean_painter).unwrap());
            },
        )
        .drop_without_applying_deltas();
        result
    }

    fn focus(&mut self) -> egui::Response {
        let response = self.frame(vec![]).unwrap();
        response.request_focus();
        response
    }

    fn open(&mut self) {
        self.focus();
        self.frame(vec![key(egui::Key::ArrowDown)]);
        self.time += 0.2;
        self.frame(vec![]);
    }

    fn popup_text(&self, text: &str) -> egui::Rect {
        let drawings = self.drawings();
        assert!(drawings.len() >= 2, "expected a popup drawing");
        text_rect(drawings.last().unwrap(), text).expect("visible popup text")
    }

    fn popup_row(&self, label: &str) -> egui::Rect {
        let drawings = self.drawings();
        drawings
            .last()
            .unwrap()
            .items()
            .iter()
            .find_map(|item| match item {
                Item::Group(group) if text_rect(group, label).is_some() => Some(group.bounds()),
                _ => None,
            })
            .expect("visible row group")
    }

    fn menu(&self) -> egui::Rect {
        let drawings = self.drawings();
        let mut result = None;
        visit(drawings.last().unwrap(), &mut |shape| {
            if let egui::Shape::Rect(rect) = &shape.shape
                && rect.fill == self.theme.colors.surface.background
                && rect.blur_width == 0.0
            {
                result = Some(shape.transform_after_tessellation * rect.rect);
            }
        });
        result.expect("popup surface")
    }

    fn accents(&self) -> Vec<egui::Rect> {
        let mut result = Vec::new();
        let drawings = self.drawings();
        visit(drawings.last().unwrap(), &mut |shape| {
            if let egui::Shape::Rect(rect) = &shape.shape
                && rect.fill == self.theme.colors.accent.background
                && rect.blur_width == 0.0
            {
                result.push(shape.transform_after_tessellation * rect.rect);
            }
        });
        result
    }
}

#[test]
fn same_batch_open_navigation_commit_and_no_initial_step() {
    let mut h = Harness::new();
    h.items[1].disabled = true;
    h.focus();
    h.frame(vec![key(egui::Key::ArrowDown), key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(0));
    assert_eq!(h.changes, 1);

    h.frame(vec![
        key(egui::Key::Enter),
        key(egui::Key::ArrowDown),
        key(egui::Key::Enter),
    ]);
    assert_eq!(h.selected, Some(2));
    assert_eq!(h.changes, 2);
}

#[test]
fn navigation_is_nonwrapping_and_home_end_skip_disabled_items() {
    let mut h = Harness::new();
    h.items[0].disabled = true;
    h.items[4].disabled = true;
    h.focus();
    h.frame(vec![
        key(egui::Key::ArrowUp),
        key(egui::Key::ArrowUp),
        key(egui::Key::Enter),
    ]);
    assert_eq!(h.selected, Some(1));

    h.frame(vec![
        key(egui::Key::Enter),
        key(egui::Key::End),
        key(egui::Key::ArrowDown),
        key(egui::Key::Enter),
    ]);
    assert_eq!(h.selected, Some(3));

    h.frame(vec![
        key(egui::Key::Enter),
        key(egui::Key::Home),
        key(egui::Key::Enter),
    ]);
    assert_eq!(h.selected, Some(1));
}

#[test]
fn opening_mouse_release_is_suppressed_across_frames() {
    let mut h = Harness::new();
    let trigger = h.frame(vec![]).unwrap();
    let pos = trigger.rect.center();
    h.frame(vec![egui::Event::PointerMoved(pos), pointer(pos, true)]);
    h.time += 0.3;
    h.frame(vec![pointer(pos + egui::vec2(10.0, 10.0), false)]);
    assert_eq!(h.selected, None);
    assert_eq!(h.changes, 0);
    assert_ne!(h.ctx.memory(|m| m.focused()), Some(trigger.id));

    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(0));
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger.id));
}

#[test]
fn drag_through_release_commits_enabled_row_without_target_assumptions() {
    let mut h = Harness::new();
    let trigger = h.frame(vec![]).unwrap();
    let start = trigger.rect.center();
    h.frame(vec![egui::Event::PointerMoved(start), pointer(start, true)]);
    h.time += 0.2;
    h.frame(vec![]);
    let destination = h.popup_text("Banana").center();
    h.frame(vec![
        egui::Event::PointerMoved(destination),
        pointer(destination, false),
    ]);
    assert_eq!(h.selected, Some(2));
    assert_eq!(h.changes, 1);
}

#[test]
fn disabled_row_and_cover_never_accept_drag_release() {
    for covered in [false, true] {
        let mut h = Harness::new();
        h.items[2].disabled = !covered;
        let trigger = h.frame(vec![]).unwrap();
        let start = trigger.rect.center();
        h.frame(vec![pointer(start, true)]);
        h.time += 0.2;
        h.frame(vec![]);
        let destination = h.popup_text("Banana").center();
        if covered {
            h.cover = Some(egui::Rect::from_center_size(
                destination,
                egui::vec2(100.0, 32.0),
            ));
        }
        h.frame(vec![
            egui::Event::PointerMoved(destination),
            pointer(destination, false),
        ]);
        assert_eq!(h.selected, None);
        assert_eq!(h.changes, 0);
    }
}

#[test]
fn disabled_trigger_and_ctrl_click_do_not_open() {
    let mut h = Harness::new();
    let response = h.frame(vec![]).unwrap();
    let pos = response.rect.center();
    let mut ctrl = pointer(pos, true);
    if let egui::Event::PointerButton { modifiers, .. } = &mut ctrl {
        modifiers.ctrl = true;
    }
    h.frame(vec![ctrl, pointer(pos, false)]);
    assert_eq!(h.drawings().len(), 1);

    h.enabled = false;
    h.frame(vec![
        pointer(pos, true),
        pointer(pos, false),
        key(egui::Key::Enter),
    ]);
    assert_eq!(h.selected, None);
    assert_eq!(h.drawings().len(), 1);
    assert_eq!(h.drawings()[0].opacity(), 0.5);
}

#[test]
fn unknown_some_is_blank_and_none_uses_placeholder() {
    let mut h = Harness::new();
    h.frame(vec![]);
    assert!(text_rect(&h.drawings()[0], "Choose an option").is_some());

    h.selected = Some(999);
    h.frame(vec![]);
    let mut text_count = 0;
    visit(&h.drawings()[0], &mut |shape| {
        if matches!(shape.shape, egui::Shape::Text(_)) {
            text_count += 1;
        }
    });
    assert_eq!(text_count, 0);
    assert_eq!(h.selected, Some(999));
    assert_eq!(h.changes, 0);
}

#[test]
fn closed_typeahead_cycles_expires_and_open_typeahead_does_not_commit() {
    let mut h = Harness::new();
    h.focus();
    h.frame(vec![egui::Event::Text("a".into())]);
    assert_eq!(h.selected, Some(0));
    h.frame(vec![egui::Event::Text("a".into())]);
    assert_eq!(h.selected, Some(1));
    h.frame(vec![egui::Event::Text("a".into())]);
    assert_eq!(h.selected, Some(0));

    h.time += 1.1;
    h.frame(vec![egui::Event::Text("b".into())]);
    assert_eq!(h.selected, Some(2));

    h.frame(vec![key(egui::Key::Enter), egui::Event::Text("c".into())]);
    assert_eq!(h.selected, Some(2));
    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(4));
}

#[test]
fn space_extends_an_active_search_instead_of_committing() {
    let mut h = Harness::new();
    h.items = vec![
        SelectItem::new(10, "New Hampshire"),
        SelectItem::new(11, "New York"),
    ];
    h.focus();
    h.frame(vec![
        key(egui::Key::Enter),
        egui::Event::Text("new".into()),
        key(egui::Key::Space),
        egui::Event::Text(" ".into()),
        egui::Event::Text("y".into()),
    ]);
    assert_eq!(h.selected, None);
    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(11));
}

#[test]
fn reorder_preserves_active_value_identity() {
    let mut h = Harness::new();
    h.selected = Some(1);
    h.open();
    h.frame(vec![key(egui::Key::ArrowDown)]);
    h.items.reverse();
    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(2));
}

#[test]
fn tab_and_shift_tab_are_blocked_until_escape_returns_focus() {
    let mut h = Harness::new();
    let trigger = h.focus();
    h.frame(vec![key(egui::Key::Enter)]);
    let popup_focus = h.ctx.memory(|memory| memory.focused());
    assert_ne!(popup_focus, Some(trigger.id));

    let mut shift_tab = key(egui::Key::Tab);
    if let egui::Event::Key { modifiers, .. } = &mut shift_tab {
        modifiers.shift = true;
    }
    h.frame(vec![key(egui::Key::Tab), shift_tab]);
    assert_eq!(h.ctx.memory(|memory| memory.focused()), popup_focus);
    h.frame(vec![key(egui::Key::Escape)]);
    assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(trigger.id));
    assert_eq!(h.changes, 0);
}

#[test]
fn explicit_host_focus_surrender_is_not_undone() {
    let mut h = Harness::new();
    h.open();
    let focused = h.ctx.memory(|memory| memory.focused()).unwrap();
    h.ctx.memory_mut(|memory| memory.surrender_focus(focused));
    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.ctx.memory(|memory| memory.focused()), None);
    assert_eq!(h.selected, None);
}

#[test]
fn outside_press_blocks_underlying_click_and_disposes_before_next_event() {
    let ctx = context();
    let theme = Theme::light();
    let items = [SelectItem::new(1, "One"), SelectItem::new(2, "Two")];
    let id = egui::Id::unique("outside-test-select");
    let underneath_id = egui::Id::unique("underneath");
    let outside = egui::pos2(350.0, 350.0);
    let mut selected = None;
    let mut clicked = 0;
    let mut trigger = None;
    let mut dismissed_on_press = false;

    for events in [
        vec![],
        vec![
            key(egui::Key::Enter),
            egui::Event::PointerMoved(outside),
            pointer(outside, true),
            pointer(outside, false),
            pointer(outside, true),
            pointer(outside, false),
        ],
    ] {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500.0, 500.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                let mut select = Select::new(&theme, id, &items).register(ui, &selected);
                let under = ui.interact(
                    egui::Rect::from_center_size(outside, egui::vec2(100.0, 50.0)),
                    underneath_id,
                    egui::Sense::click(),
                );
                if trigger.is_none() {
                    select.response.request_focus();
                }
                while let Some(event) = ctx.next_input_event() {
                    let had_modal = ctx.memory(|memory| memory.top_modal_layer()).is_some();
                    select.handle(&event, &mut selected);
                    if had_modal
                        && matches!(
                            &event.trigger,
                            egui::InputTrigger::Event {
                                event: egui::Event::PointerButton { pressed: true, .. },
                                ..
                            }
                        )
                    {
                        assert_eq!(ctx.memory(|memory| memory.top_modal_layer()), None);
                        assert_ne!(event.target, Some(underneath_id));
                        dismissed_on_press = true;
                    }
                    if event.target == Some(underneath_id)
                        && ctx.read_response(under.id).unwrap().clicked()
                    {
                        clicked += 1;
                    }
                }
                trigger = Some(select.response);
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(dismissed_on_press);
    assert_eq!(
        clicked, 1,
        "only the fresh click after dismissal may pass through"
    );
    assert_eq!(selected, None);
}

#[test]
fn empty_and_all_disabled_lists_are_safe_and_dismissible() {
    for empty in [false, true] {
        let mut h = Harness::new();
        if empty {
            h.items.clear();
        } else {
            for item in &mut h.items {
                item.disabled = true;
            }
        }
        let trigger = h.focus();
        h.frame(vec![
            key(egui::Key::ArrowDown),
            key(egui::Key::Home),
            key(egui::Key::End),
            key(egui::Key::Enter),
            key(egui::Key::Escape),
        ]);
        assert_eq!(h.selected, None);
        assert_eq!(h.changes, 0);
        assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(trigger.id));
    }
}

#[test]
fn long_list_end_is_visible_near_viewport_edge_and_wheel_changes_viewport() {
    let mut h = Harness::new();
    h.screen = egui::vec2(280.0, 240.0);
    h.origin = egui::pos2(180.0, 195.0);
    h.items = (0..80)
        .map(|index| SelectItem::new(index, format!("Option {index:02}")))
        .collect();
    h.open();
    h.frame(vec![key(egui::Key::End)]);
    let last = h.popup_text("Option 79");
    assert!(last.top() >= 10.0);
    assert!(last.bottom() <= h.screen.y - 10.0);
    assert!(last.left() >= 10.0);

    h.frame(vec![egui::Event::PointerMoved(last.center()), wheel(96.0)]);
    let drawings = h.drawings();
    let popup = drawings.last().unwrap();
    assert!(text_rect(popup, "Option 76").is_some());
    h.frame(vec![key(egui::Key::End), key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(79));
}

#[test]
fn blur_resize_disappearance_and_disable_cancel_the_opening_gesture() {
    for case in 0..4 {
        let mut h = Harness::new();
        let trigger = h.frame(vec![]).unwrap();
        let pos = trigger.rect.center();
        h.frame(vec![pointer(pos, true)]);

        match case {
            0 => {
                h.frame(vec![egui::Event::WindowFocused(false)]);
                h.frame(vec![egui::Event::WindowFocused(true)]);
            }
            1 => {
                h.screen.x += 20.0;
                h.frame(vec![]);
            }
            2 => {
                h.present = false;
                h.frame(vec![]);
                h.present = true;
                h.frame(vec![]);
            }
            3 => {
                h.enabled = false;
                h.frame(vec![]);
                h.enabled = true;
                h.frame(vec![]);
            }
            _ => unreachable!(),
        }

        h.frame(vec![pointer(pos + egui::vec2(0.0, 64.0), false)]);
        assert_eq!(h.selected, None, "case {case}");
        assert_eq!(h.changes, 0, "case {case}");
    }
}

#[test]
fn transformed_trigger_opens_a_global_unclipped_portal() {
    let mut h = Harness::new();
    h.origin = egui::pos2(30.0, 40.0);
    h.transform = Some(egui::emath::TSTransform {
        scaling: 1.25,
        translation: egui::vec2(35.0, 20.0),
    });
    h.clip = Some(egui::Rect::from_min_size(h.origin, egui::vec2(192.0, 36.0)));
    let response = h.frame(vec![]).unwrap();
    let transform = h.transform.unwrap();
    let pos = transform * response.rect.center();
    h.frame(vec![egui::Event::PointerMoved(pos), pointer(pos, true)]);
    h.time += 0.2;
    h.frame(vec![pointer(pos, false)]);
    let row = h.popup_text("Cherry");
    assert!(row.bottom() > (transform * h.clip.unwrap()).bottom());
    h.frame(vec![
        egui::Event::PointerMoved(row.center()),
        pointer(row.center(), true),
        pointer(row.center(), false),
    ]);
    assert_eq!(h.selected, Some(4));
}

#[test]
fn disabled_trigger_is_one_group_and_reference_sizes_do_not_follow_theme_height() {
    for (size, expected) in [(SelectSize::Default, 36.0), (SelectSize::Small, 32.0)] {
        let mut h = Harness::new();
        h.size = size;
        h.enabled = false;
        h.theme.metrics.control_height = 90.0;
        let response = h.frame(vec![]).unwrap();
        assert_eq!(response.rect.height(), expected);
        assert_eq!(response.rect.width(), 192.0);
        let drawings = h.drawings();
        assert_eq!(drawings.len(), 1);
        assert_eq!(drawings[0].opacity(), 0.5);
        assert!(
            drawings[0]
                .items()
                .iter()
                .all(|item| matches!(item, Item::Shape(_)))
        );
    }
}

#[test]
fn selected_row_is_not_permanently_accented_after_navigation() {
    let mut h = Harness::new();
    h.selected = Some(0);
    h.open();
    h.frame(vec![key(egui::Key::ArrowDown)]);
    let accent_rects = h.accents();
    assert_eq!(accent_rects.len(), 1);
    assert!(accent_rects[0].contains(h.popup_text("Apricot").center()));
    assert!(!accent_rects[0].contains(h.popup_text("Apple").center()));
}

#[test]
fn independent_ids_and_non_send_values_compile_and_do_not_share_state() {
    let ctx = context();
    let theme = Theme::light();
    let items = [
        SelectItem::new(Rc::new(1), "One"),
        SelectItem::new(Rc::new(2), "Two"),
    ];
    let mut a = None;
    let mut b = None;
    let id_a = egui::Id::unique("a");
    let id_b = egui::Id::unique("b");

    for pass in 0..2 {
        ctx.run_ui(
            egui::RawInput {
                events: if pass == 0 {
                    vec![]
                } else {
                    vec![
                        key(egui::Key::Enter),
                        key(egui::Key::End),
                        key(egui::Key::Enter),
                    ]
                },
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                let mut first = Select::new(&theme, id_a, &items).register(ui, &a);
                let mut second = Select::new(&theme, id_b, &items).register(ui, &b);
                if pass == 0 {
                    second.response.request_focus();
                }
                while let Some(event) = ctx.next_input_event() {
                    first.handle(&event, &mut a);
                    second.handle(&event, &mut b);
                }
            },
        )
        .drop_without_applying_deltas();
    }
    assert_eq!(a, None);
    assert_eq!(b, Some(Rc::new(2)));
}

#[test]
fn same_batch_unregister_revokes_popup_before_following_keyboard_event() {
    let ctx = context();
    let theme = Theme::light();
    let items = [SelectItem::new(1, "One")];
    let mut selected = None;
    let id = egui::Id::unique("removed-select");

    ctx.run_ui(
        egui::RawInput {
            events: vec![key(egui::Key::Enter), key(egui::Key::Enter)],
            ..Default::default()
        },
        |ui| {
            ctx.begin_input();
            let mut registration = Select::new(&theme, id, &items).register(ui, &selected);
            registration.response.request_focus();
            let mut count = 0;
            while let Some(event) = ctx.next_input_event() {
                registration.handle(&event, &mut selected);
                if matches!(
                    event.trigger,
                    egui::InputTrigger::Event {
                        event: egui::Event::Key { pressed: true, .. },
                        ..
                    }
                ) {
                    count += 1;
                    if count == 1 {
                        registration.unregister();
                    }
                }
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(selected, None);
    assert_eq!(ctx.memory(|memory| memory.top_modal_layer()), None);
}

#[test]
#[should_panic(expected = "Select option values must be unique")]
fn duplicate_values_are_programmer_errors() {
    let ctx = context();
    let theme = Theme::light();
    let items = [SelectItem::new(1, "One"), SelectItem::new(1, "Duplicate")];
    ctx.run_ui(Default::default(), |ui| {
        ctx.begin_input();
        Select::new(&theme, egui::Id::unique("duplicates"), &items).register(ui, &None);
        while ctx.next_input_event().is_some() {}
    })
    .drop_without_applying_deltas();
}

#[test]
fn default_width_fits_current_display_and_explicit_width_clips_one_line() {
    let mut h = Harness::new();
    h.width = None;

    for (value, label) in [
        (None, Some("Choose an option")),
        (Some(0), Some("Apple")),
        (Some(999), None),
    ] {
        h.selected = value;
        let response = h.frame(vec![]).unwrap();
        let drawings = h.drawings();
        let mut measured = 0.0;
        let mut count = 0;
        visit(&drawings[0], &mut |shape| {
            if let egui::Shape::Text(text) = &shape.shape {
                assert_eq!(Some(text.galley.job.text.as_str()), label);
                measured = text.galley.size().x;
                count += 1;
            }
        });
        assert_eq!(count, usize::from(label.is_some()));
        near(response.rect.width(), measured + 50.0);
    }

    h.items[0].label = "A very long option that must remain a single clipped line".into();
    h.selected = Some(0);
    h.width = Some(192.0);
    let response = h.frame(vec![]).unwrap();
    near(response.rect.width(), 192.0);
    let mut found = false;
    visit(&h.drawings()[0], &mut |shape| {
        if let egui::Shape::Text(text) = &shape.shape {
            assert_eq!(text.galley.rows.len(), 1);
            assert!(text.galley.size().x > shape.clip_rect.width());
            near(shape.clip_rect.left(), response.rect.left() + 13.0);
            near(shape.clip_rect.right(), response.rect.right() - 37.0);
            found = true;
        }
    });
    assert!(found);
}

#[test]
fn pointer_leave_disabled_row_and_backdrop_clear_highlight_without_revival() {
    for disabled_destination in [false, true] {
        let mut h = Harness::new();
        h.items[2].disabled = true;
        h.open();

        let enabled = h.popup_text("Apricot").center();
        h.frame(vec![egui::Event::PointerMoved(enabled)]);
        assert_eq!(h.accents().len(), 1);
        assert!(h.accents()[0].contains(enabled));

        let destination = if disabled_destination {
            h.popup_text("Banana").center()
        } else {
            egui::pos2(450.0, 450.0)
        };
        h.frame(vec![egui::Event::PointerMoved(destination)]);
        assert!(h.accents().is_empty());
        h.frame(vec![]);
        assert!(
            h.accents().is_empty(),
            "registration must not revive an active row"
        );
        h.frame(vec![key(egui::Key::Enter)]);
        assert_eq!(h.selected, None);
        assert_eq!(h.changes, 0);
        h.frame(vec![key(egui::Key::Escape)]);
        assert_eq!(h.ctx.memory(|memory| memory.focused()), Some(h.id()));
    }
}

#[test]
fn rounded_row_corner_is_not_a_row_hit() {
    let mut h = Harness::new();
    let start = h.frame(vec![]).unwrap().rect.center();
    h.frame(vec![pointer(start, true)]);
    h.frame_after(0.2, vec![]);
    let row = h.popup_row("Banana");
    let corner = row.min + egui::vec2(0.25, 0.25);
    assert!(row.contains(corner));
    h.frame(vec![
        egui::Event::PointerMoved(corner),
        pointer(corner, false),
    ]);
    assert_eq!(h.selected, None);
    assert_eq!(h.changes, 0);
}

#[test]
fn clipped_row_under_scroll_indicator_is_not_a_row_hit() {
    let mut h = Harness::new();
    h.screen = egui::vec2(1280.0, 720.0);
    h.origin = egui::pos2(48.0, 436.0);
    h.items = (0..30)
        .map(|index| SelectItem::new(index, format!("Option {index:02}")))
        .collect();
    h.selected = Some(29);
    let start = h.frame(vec![]).unwrap().rect.center();
    h.frame(vec![pointer(start, true)]);
    h.frame_after(0.2, vec![]);

    let clipped_row = h.popup_row("Option 16");
    let position = egui::pos2(clipped_row.center().x, 30.0);
    assert!(clipped_row.contains(position));
    h.frame(vec![
        egui::Event::PointerMoved(position),
        pointer(position, false),
    ]);
    assert_eq!(h.selected, Some(29));
    assert_eq!(h.changes, 0);
}

#[test]
fn pinned_item_alignment_and_home_expansion_match_logical_geometry() {
    let mut h = Harness::new();
    h.screen = egui::vec2(1280.0, 720.0);
    h.origin = egui::pos2(48.0, 436.0);
    h.items = (0..30)
        .map(|index| SelectItem::new(index, format!("Option {index:02}")))
        .collect();
    h.selected = Some(29);
    h.open();

    near(h.menu().top(), 10.0);
    near(h.menu().height(), 465.0);
    near(h.popup_row("Option 29").center().y, 454.0);
    near(h.popup_row("Option 16").top(), 22.0);

    let mut first_clip = None;
    visit(h.drawings().last().unwrap(), &mut |shape| {
        if let egui::Shape::Text(text) = &shape.shape
            && text.galley.job.text == "Option 16"
        {
            first_clip = Some(shape.clip_rect);
        }
    });
    near(first_clip.unwrap().top(), 35.0);
    // viewport: 35..474, scrollHeight: 30*32+8, scrollTop: 529.
    near(h.menu().bottom() - 1.0 - 35.0, 439.0);
    near(
        35.0 + 4.0 + 16.0 * 32.0 - h.popup_row("Option 16").top(),
        529.0,
    );

    h.frame(vec![key(egui::Key::Home)]);
    near(h.menu().top(), 10.0);
    near(h.menu().bottom(), 710.0);
    near(h.popup_row("Option 00").top(), 15.0);
    // No up button; the 24-point down button leaves a 674-point viewport.
    near(h.menu().height() - 2.0 - 24.0, 674.0);
    assert_eq!(h.selected, Some(29));
    h.frame(vec![key(egui::Key::Enter)]);
    assert_eq!(h.selected, Some(0));
}

#[test]
fn short_item_aligned_content_does_not_fill_its_bottom_anchored_wrapper() {
    let mut h = Harness::new();
    h.screen = egui::vec2(1280.0, 720.0);
    h.origin = egui::pos2(48.0, 320.0);
    h.selected = Some(0);
    h.open();
    near(h.menu().top(), 317.0);
    near(h.menu().height(), 170.0);
    near(h.popup_row("Apple").center().y, 338.0);
}

#[test]
fn wheel_expands_top_and_bottom_anchored_wrappers_and_is_consumed() {
    for selected in [0, 29] {
        let mut h = Harness::new();
        h.screen = egui::vec2(1280.0, 720.0);
        h.origin = egui::pos2(48.0, 320.0);
        h.items = (0..30)
            .map(|index| SelectItem::new(index, format!("Option {index:02}")))
            .collect();
        h.selected = Some(selected);
        h.open();
        let before = h.menu();
        let point = h.popup_text(&format!("Option {selected:02}")).center();
        h.frame(vec![
            egui::Event::PointerMoved(point),
            wheel(if selected == 0 { -32.0 } else { 32.0 }),
        ]);
        let after = h.menu();
        near(after.height(), before.height() + 32.0);
        if selected == 0 {
            near(after.bottom(), before.bottom());
            near(after.top(), before.top() - 32.0);
        } else {
            near(after.top(), before.top());
        }
        assert_eq!(h.selected, Some(selected));
        assert_eq!(h.changes, 0);
    }
}

#[test]
fn scroll_indicator_repeats_after_50ms_without_raw_events_and_cancels() {
    let mut h = Harness::new();
    h.screen = egui::vec2(1280.0, 720.0);
    h.origin = egui::pos2(48.0, 436.0);
    h.items = (0..30)
        .map(|index| SelectItem::new(index, format!("Option {index:02}")))
        .collect();
    h.selected = Some(29);
    h.open();

    let menu = h.menu();
    let up = egui::pos2(menu.center().x, menu.top() + 1.0 + 12.0);
    h.frame(vec![egui::Event::PointerMoved(up)]);
    near(h.menu().height(), 465.0);
    h.frame_after(0.049, vec![]);
    near(h.menu().height(), 465.0);
    h.frame_after(0.002, vec![]);
    near(h.menu().height(), 497.0);
    h.frame_after(0.050, vec![]);
    near(h.menu().height(), 529.0);

    h.frame(vec![egui::Event::PointerMoved(egui::pos2(1100.0, 600.0))]);
    let stopped = h.menu();
    h.frame_after(0.3, vec![]);
    assert_eq!(h.menu(), stopped);
    assert_eq!(h.selected, Some(29));
    assert_eq!(h.changes, 0);

    h.frame(vec![egui::Event::PointerMoved(up)]);
    h.frame(vec![key(egui::Key::Escape)]);
    h.frame_after(0.3, vec![]);
    assert_eq!(h.drawings().len(), 1);
    h.frame_after(1.0, vec![]);
    assert_eq!(h.drawings().len(), 1);
    assert_eq!(h.ctx.memory(|memory| memory.top_modal_layer()), None);
}

#[test]
fn popup_uses_css_ease_and_scales_about_menu_center() {
    let mut h = Harness::new();
    h.focus();
    h.frame(vec![key(egui::Key::Enter)]);
    h.frame_after(0.075, vec![]);
    let drawings = h.drawings();
    let popup = drawings.last().unwrap();
    let progress = popup.opacity();
    assert!((progress - 0.8024034).abs() < 0.0001);

    let mut checked = false;
    visit(popup, &mut |shape| {
        if let egui::Shape::Rect(rect) = &shape.shape
            && rect.fill == h.theme.colors.surface.background
            && rect.blur_width == 0.0
        {
            let transform = shape.transform_after_tessellation;
            near(transform.scaling, 0.95 + 0.05 * progress);
            let center = rect.rect.center();
            let transformed_center = transform * center;
            near(transformed_center.x, center.x);
            near(transformed_center.y, center.y);
            checked = true;
        }
    });
    assert!(checked);

    h.frame_after(0.075, vec![]);
    near(h.drawings().last().unwrap().opacity(), 1.0);
    h.frame(vec![key(egui::Key::Escape)]);
    h.frame_after(0.075, vec![]);
    assert!((h.drawings().last().unwrap().opacity() - (1.0 - 0.8024034)).abs() < 0.0001);
    h.frame_after(0.076, vec![]);
    assert_eq!(h.drawings().len(), 1);
}

#[test]
fn nested_modal_is_restored_only_when_renewed_in_the_current_pass() {
    for renew_parent in [false, true] {
        let ctx = context();
        let theme = Theme::light();
        let items = [SelectItem::new(1, "One"), SelectItem::new(2, "Two")];
        let select_id = egui::Id::unique("nested-select");
        let parent_id = egui::Id::unique("parent-modal");
        let parent_layer = egui::LayerId::new(egui::Order::Foreground, parent_id);
        let mut selected = None;

        for pass in 0..3 {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(500.0, 500.0),
                    )),
                    events: match pass {
                        0 => vec![],
                        1 => vec![key(egui::Key::Enter)],
                        _ => vec![key(egui::Key::Escape)],
                    },
                    ..Default::default()
                },
                |ui| {
                    ctx.begin_input();
                    let mut select = if pass < 2 || renew_parent {
                        egui::Modal::new(parent_id)
                            .area(
                                egui::Area::new(parent_id)
                                    .order(egui::Order::Foreground)
                                    .fixed_pos(egui::pos2(50.0, 50.0))
                                    .fade_in(false),
                            )
                            .register(&ctx, |ui| {
                                Select::new(&theme, select_id, &items).register(ui, &selected)
                            })
                            .inner
                    } else {
                        Select::new(&theme, select_id, &items).register(ui, &selected)
                    };
                    if pass == 0 {
                        select.response.request_focus();
                    }
                    while let Some(event) = ctx.next_input_event() {
                        select.handle(&event, &mut selected);
                    }
                    if pass == 1 {
                        let top = ctx.memory(|memory| memory.top_modal_layer());
                        assert!(top.is_some());
                        assert_ne!(top, Some(parent_layer));
                    }
                    if pass == 2 {
                        assert_eq!(
                            ctx.memory(|memory| memory.top_modal_layer()),
                            renew_parent.then_some(parent_layer)
                        );
                        assert_eq!(ctx.memory(|memory| memory.focused()), Some(select_id));
                    }
                },
            )
            .drop_without_applying_deltas();
        }
        assert_eq!(selected, None);
    }
}

#[test]
fn newer_host_modal_and_explicit_focus_transfer_win_in_the_same_batch() {
    for modal_transfer in [false, true] {
        let ctx = context();
        let theme = Theme::light();
        let items = [SelectItem::new(1, "One"), SelectItem::new(2, "Two")];
        let select_id = egui::Id::unique("transfer-select");
        let host_id = egui::Id::unique("host-focus");
        let host_layer =
            egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("new-host-modal"));
        let mut selected = None;
        let mut transferred = false;

        ctx.run_ui(
            egui::RawInput {
                events: vec![
                    key(egui::Key::Enter),
                    key(egui::Key::End),
                    key(egui::Key::Enter),
                ],
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                let mut select = Select::new(&theme, select_id, &items).register(ui, &selected);
                // Measure the nonmodal Area before dispatch, then renew it in layout.
                let host_area = egui::LayerId::new(egui::Order::Tooltip, host_id.with("layer"));
                let register_host = || {
                    egui::Area::new(host_area.id)
                        .order(host_area.order)
                        .fixed_pos(egui::pos2(300.0, 300.0))
                        .fade_in(false)
                        .show(&ctx, |ui| {
                            ui.set_min_size(egui::vec2(100.0, 30.0));
                            let response =
                                ui.interact(ui.min_rect(), host_id, egui::Sense::click());
                            ui.memory_mut(|memory| {
                                memory.register_focus_policy(
                                    response.id,
                                    response.layer_id,
                                    egui::EventFilter {
                                        tab: true,
                                        horizontal_arrows: true,
                                        vertical_arrows: true,
                                        escape: true,
                                    },
                                );
                            });
                            response
                        })
                        .inner
                };
                if !modal_transfer {
                    let _ = register_host();
                }
                select.response.request_focus();

                while let Some(event) = ctx.next_input_event() {
                    select.handle(&event, &mut selected);
                    if !transferred
                        && matches!(
                            &event.trigger,
                            egui::InputTrigger::Event {
                                event: egui::Event::Key {
                                    key: egui::Key::Enter,
                                    ..
                                },
                                ..
                            }
                        )
                    {
                        if modal_transfer {
                            egui::Modal::new(host_layer.id)
                                .area(
                                    egui::Area::new(host_layer.id)
                                        .order(host_layer.order)
                                        .fade_in(false),
                                )
                                .register(&ctx, |ui| {
                                    let response = ui.interact(
                                        egui::Rect::from_min_size(
                                            egui::pos2(300.0, 300.0),
                                            egui::vec2(100.0, 30.0),
                                        ),
                                        host_id.with("modal"),
                                        egui::Sense::click(),
                                    );
                                    ui.memory_mut(|memory| {
                                        memory.register_focus_policy(
                                            response.id,
                                            response.layer_id,
                                            egui::EventFilter {
                                                tab: true,
                                                horizontal_arrows: true,
                                                vertical_arrows: true,
                                                escape: true,
                                            },
                                        );
                                    });
                                    response.request_focus();
                                });
                        } else {
                            let host = ctx.input_layout(register_host);
                            assert!(host.enabled() && host.visible());
                            assert!(ctx.memory(|memory| memory.allows_interaction(host_area)));
                            host.request_focus();
                        }
                        transferred = true;
                    }
                }

                assert_eq!(selected, None);
                assert_eq!(
                    ctx.memory(|memory| memory.focused()),
                    Some(if modal_transfer {
                        host_id.with("modal")
                    } else {
                        host_id
                    })
                );
                assert_eq!(
                    ctx.memory(|memory| memory.top_modal_layer()),
                    modal_transfer.then_some(host_layer)
                );
            },
        )
        .drop_without_applying_deltas();
        assert!(transferred);
    }
}

#[test]
fn indicator_hover_repeats_without_new_raw_events() {
    let mut h = Harness::new();
    h.screen = egui::vec2(400.0, 300.0);
    h.items = (0..30)
        .map(|index| SelectItem::new(index, format!("Option {index:02}")))
        .collect();
    h.open();
    let popup = h.drawings().last().unwrap().clone();
    let mut menu = None;
    visit(&popup, &mut |shape| {
        if let egui::Shape::Rect(rect) = &shape.shape
            && rect.fill == h.theme.colors.surface.background
        {
            menu = Some(rect.rect);
        }
    });
    let menu = menu.expect("popup background");
    let first_before = h.popup_text("Option 00").top();
    let down = egui::pos2(menu.center().x, menu.bottom() - 17.0);
    h.frame(vec![egui::Event::PointerMoved(down)]);
    h.time += 0.101;
    h.frame(vec![]);
    let popup = h.drawings().last().unwrap().clone();
    let first_after = text_rect(&popup, "Option 00");
    assert!(
        first_after.is_none_or(|rect| rect.top() < first_before),
        "an indicator timer must advance scrolling with no new raw input"
    );
}

#[test]
fn discarded_parent_modal_is_not_restored_in_the_next_layout_pass() {
    let ctx = context();
    ctx.options_mut(|options| options.max_passes = 2.try_into().unwrap());
    let theme = Theme::light();
    let items = [SelectItem::new(1, "One"), SelectItem::new(2, "Two")];
    let select_id = egui::Id::unique("discard-parent-select");
    let parent_id = egui::Id::unique("discard-parent-modal");
    let mut selected = None;
    let mut calls = 0;
    let mut restored = None;
    ctx.run_ui(
        egui::RawInput {
            events: vec![key(egui::Key::Enter)],
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500.0, 500.0),
            )),
            ..Default::default()
        },
        |ui| {
            calls += 1;
            ctx.begin_input();
            let mut select = if ctx.current_pass_index() == 0 {
                egui::Modal::new(parent_id)
                    .area(
                        egui::Area::new(parent_id)
                            .fixed_pos(egui::pos2(50.0, 50.0))
                            .fade_in(false),
                    )
                    .register(&ctx, |ui| {
                        Select::new(&theme, select_id, &items).register(ui, &selected)
                    })
                    .inner
            } else {
                Select::new(&theme, select_id, &items).register(ui, &selected)
            };
            if ctx.current_pass_index() == 0 {
                select.response.request_focus();
            } else {
                select.unregister();
                restored = ctx.memory(|memory| memory.top_modal_layer());
            }
            while let Some(event) = ctx.next_input_event() {
                select.handle(&event, &mut selected);
            }
            if ctx.current_pass_index() == 0 {
                assert!(ctx.memory(|memory| memory.top_modal_layer()).is_some());
                ctx.request_discard("parent modal is absent in the next layout");
            }
        },
    )
    .drop_without_applying_deltas();
    assert_eq!(calls, 2);
    assert_eq!(selected, None);
    assert_eq!(restored, None);
}

#[test]
fn registration_cannot_paint_after_a_discard_without_reregistering() {
    let ctx = context();
    ctx.options_mut(|options| options.max_passes = 2.try_into().unwrap());
    let backend = Arc::new(Collector::default());
    let _installation = compositing::install(&ctx, backend).unwrap();
    let theme = Theme::light();
    let items = [SelectItem::new(1, "One")];
    let mut registration = None;
    let mut rejected = false;
    ctx.run_ui(egui::RawInput::default(), |ui| {
        ctx.begin_input();
        if ctx.current_pass_index() == 0 {
            registration = Some(
                Select::new(&theme, egui::Id::unique("stale-registration"), &items)
                    .register(ui, &None),
            );
        }
        while ctx.next_input_event().is_some() {}
        if ctx.current_pass_index() == 0 {
            ctx.request_discard("exercise retained registration rejection");
        } else {
            rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration
                    .as_ref()
                    .unwrap()
                    .paint(&None, ui.painter())
                    .unwrap();
            }))
            .is_err_and(|panic| {
                panic
                    .downcast_ref::<String>()
                    .is_some_and(|message| message.contains("Select registrations are pass-local"))
            });
        }
    })
    .drop_without_applying_deltas();
    assert!(
        rejected,
        "a registration from a previous pass must be rejected"
    );
}
