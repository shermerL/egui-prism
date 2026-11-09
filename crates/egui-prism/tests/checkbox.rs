//! Public ordered-input, drawing, compositing, and lifecycle regressions.

use egui_prism::{
    checkbox::Checkbox,
    compositing::{self, Group, Item},
    egui,
    theme::Theme,
};
use std::sync::{Arc, Mutex};

fn ordered_context() -> egui::Context {
    let context = egui::Context::default();
    context.options_mut(|options| {
        options.ordered_input = true;
        options.max_passes = std::num::NonZeroUsize::MIN;
    });
    context
}

fn phase_ui(ui: &mut egui::Ui) -> egui::Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .scope_id(egui::Id::unique("checkbox-test-layout"))
            .max_rect(ui.max_rect()),
    )
}

fn key_event(key: egui::Key, pressed: bool, repeat: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat,
        modifiers: egui::Modifiers::NONE,
    }
}

fn pointer_event(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

fn control(group: &Group) -> &Group {
    let [Item::Group(control), Item::Shape(label)] = group.items() else {
        panic!("expected one isolated control and an independent label");
    };
    assert!(matches!(&label.shape, egui::Shape::Text(_)));
    control
}

fn label(group: &Group) -> &egui::epaint::TextShape {
    let Item::Shape(shape) = &group.items()[1] else {
        panic!("expected label shape");
    };
    let egui::Shape::Text(text) = &shape.shape else {
        panic!("expected text");
    };
    text
}

fn rects(group: &Group) -> impl Iterator<Item = &egui::epaint::RectShape> {
    control(group).items().iter().filter_map(|item| match item {
        Item::Shape(shape) => match &shape.shape {
            egui::Shape::Rect(rect) => Some(rect),
            _ => None,
        },
        _ => None,
    })
}

fn background(group: &Group) -> &egui::epaint::RectShape {
    rects(group)
        .find(|rect| rect.blur_width == 0.0 && rect.stroke.width == 0.0)
        .expect("square background, including transparent backgrounds")
}

fn border(group: &Group) -> &egui::epaint::RectShape {
    let square = background(group).rect;
    rects(group)
        .find(|rect| rect.rect == square && rect.stroke.width == 1.0)
        .expect("one point square border")
}

fn ring(group: &Group) -> Option<&egui::epaint::RectShape> {
    let square = background(group).rect;
    rects(group).find(|rect| rect.stroke.width > 0.0 && rect.rect != square)
}

fn mark(group: &Group) -> Option<&[egui::Shape]> {
    control(group).items().iter().find_map(|item| match item {
        Item::Shape(shape) => match &shape.shape {
            egui::Shape::Vec(shapes) => Some(shapes.as_slice()),
            _ => None,
        },
        _ => None,
    })
}

fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.001, "{a} != {b}");
}

struct Harness {
    context: egui::Context,
    theme: Theme,
    values: [bool; 2],
    changes: [usize; 2],
    enabled: bool,
    invalid: bool,
    reversed: bool,
    automatic: bool,
    label: &'static str,
    width: f32,
    time: f64,
    drawings: Vec<Group>,
    tree: Option<egui::accesskit::TreeUpdate>,
    cover: Option<egui::Rect>,
}

impl Harness {
    fn new() -> Self {
        let context = ordered_context();
        context.enable_accesskit();
        Self {
            context,
            theme: Theme::light(),
            values: [false; 2],
            changes: [0; 2],
            enabled: true,
            invalid: false,
            reversed: false,
            automatic: false,
            label: "Enable this option",
            width: 320.0,
            time: 0.0,
            drawings: Vec::new(),
            tree: None,
            cover: None,
        }
    }

    fn checkbox(&self, index: usize) -> Checkbox<'_, '_> {
        let checkbox =
            Checkbox::new(&self.theme, &self.values[index], self.label).invalid(self.invalid);
        if self.automatic {
            checkbox
        } else {
            checkbox.id(egui::Id::unique(("checkbox-test", index)))
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let context = self.context.clone();
        let order = if self.reversed { [1, 0] } else { [0, 1] };
        let mut responses = None;
        self.drawings.clear();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(self.width, 800.0),
                )),
                events,
                time: Some(self.time),
                ..Default::default()
            },
            |ui| {
                context.begin_input();
                let mut register = phase_ui(ui);
                let registrations = order.map(|index| {
                    let enabled = self.enabled;
                    let checkbox = self.checkbox(index);
                    register
                        .add_enabled_ui(enabled, |ui| checkbox.register(ui))
                        .inner
                });
                if let Some(cover) = self.cover {
                    register.interact(cover, egui::Id::unique("cover"), egui::Sense::click());
                }
                while let Some(event) = context.next_input_event() {
                    if let Some(slot) = registrations
                        .iter()
                        .position(|registration| Some(registration.response.id) == event.target)
                    {
                        let index = order[slot];
                        if registrations[slot]
                            .handle(&event, &mut self.values[index])
                            .changed()
                        {
                            self.changes[index] += 1;
                        }
                    }
                    for registration in &registrations {
                        registration.cancel_unfocused();
                    }
                }
                let mut draw = phase_ui(ui);
                responses = Some(order.map(|index| {
                    let enabled = self.enabled;
                    let checkbox = self.checkbox(index);
                    let output = draw.add_enabled_ui(enabled, |ui| checkbox.show(ui)).inner;
                    self.drawings.push(output.drawing.unwrap());
                    output.response
                }));
            },
        );
        self.tree = output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn click(&mut self, pos: egui::Pos2) {
        self.frame(vec![
            egui::Event::PointerMoved(pos),
            pointer_event(pos, true),
            pointer_event(pos, false),
        ]);
    }

    fn key(&mut self, key: egui::Key) {
        self.frame(vec![
            key_event(key, true, false),
            key_event(key, false, false),
        ]);
    }

    fn settle(&mut self) {
        self.time += 0.2;
        self.frame(vec![]);
    }

    fn absent_frame(&mut self) {
        self.time += 1.0 / 60.0;
        self.context
            .run_ui(
                egui::RawInput {
                    time: Some(self.time),
                    ..Default::default()
                },
                |ui| {
                    ui.ctx().begin_input();
                    while ui.ctx().next_input_event().is_some() {}
                },
            )
            .drop_without_applying_deltas();
    }

    fn node(&self, id: egui::Id) -> &egui::accesskit::Node {
        &self
            .tree
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == id.accesskit_id())
            .unwrap()
            .1
    }
}

#[test]
fn ordered_square_gap_and_label_clicks_share_one_owner() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    let square = background(&h.drawings[0]).rect;
    for pos in [
        square.center(),
        square.right_center() + egui::vec2(4.0, 0.0),
        r[0].rect.right_center() - egui::vec2(3.0, 0.0),
    ] {
        let before = h.changes[0];
        h.click(pos);
        assert_eq!(h.changes, [before + 1, 0]);
        assert_eq!(h.context.memory(|memory| memory.focused()), Some(r[0].id));
    }
    assert_eq!(h.values, [true, false]);
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(h.changes, [3, 0]);
    assert!(ring(&h.drawings[0]).is_none());
}

#[test]
fn two_clicks_in_one_batch_report_two_changes_even_when_value_is_restored() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
        pointer_event(pos, false),
        pointer_event(pos, true),
        pointer_event(pos, false),
        egui::Event::PointerMoved(egui::pos2(300.0, 700.0)),
    ]);
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [2, 0]);
    assert!(mark(&h.drawings[0]).is_none());
}

#[test]
fn separate_labels_in_one_batch_use_each_events_position() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    let a = r[0].rect.center();
    let b = r[1].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(a),
        pointer_event(a, true),
        pointer_event(a, false),
        egui::Event::PointerMoved(b),
        pointer_event(b, true),
        pointer_event(b, false),
    ]);
    assert_eq!(h.values, [true, true]);
    assert_eq!(h.changes, [1, 1]);
    assert_eq!(h.context.memory(|memory| memory.focused()), Some(r[1].id));
    assert!(mark(&h.drawings[0]).is_some());
    assert!(mark(&h.drawings[1]).is_some());
}

#[test]
fn space_activates_only_on_armed_release_and_enter_is_always_a_noop() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    h.key(egui::Key::Tab);
    assert_eq!(h.context.memory(|memory| memory.focused()), Some(r[0].id));
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [0, 0]);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.frame(vec![key_event(egui::Key::Space, true, true)]);
    h.frame(vec![key_event(egui::Key::Space, true, true)]);
    assert_eq!(h.values, [false, false]);
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [1, 0]);
    assert_eq!(h.values, [true, false]);
    h.frame(vec![
        key_event(egui::Key::Enter, true, false),
        key_event(egui::Key::Enter, true, true),
        key_event(egui::Key::Enter, false, false),
        key_event(egui::Key::Space, false, false),
    ]);
    for key in [
        egui::Key::ArrowDown,
        egui::Key::ArrowRight,
        egui::Key::Escape,
    ] {
        h.key(key);
    }
    assert_eq!(h.changes, [1, 0]);
    assert_eq!(h.context.memory(|memory| memory.focused()), Some(r[0].id));
}

#[test]
fn space_cancels_on_blur_disable_and_window_focus_loss() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    r[0].request_focus();
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    r[1].request_focus();
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [0, 0]);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.enabled = false;
    h.frame(vec![]);
    h.enabled = true;
    r[1].request_focus();
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [0, 0]);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.frame(vec![egui::Event::WindowFocused(false)]);
    h.frame(vec![
        egui::Event::WindowFocused(true),
        key_event(egui::Key::Space, false, false),
    ]);
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn same_batch_focus_loss_and_refocus_do_not_revive_space() {
    let mut h = Harness::new();
    let response = h.frame(vec![])[0].clone();
    response.request_focus();
    let tab = |pressed, shift| egui::Event::Key {
        key: egui::Key::Tab,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            ..egui::Modifiers::NONE
        },
    };
    h.frame(vec![
        key_event(egui::Key::Space, true, false),
        tab(true, false),
        tab(false, false),
        tab(true, true),
        key_event(egui::Key::Space, false, false),
    ]);
    assert_eq!(
        h.context.memory(|memory| memory.focused()),
        Some(response.id)
    );
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn absence_cancels_both_space_and_pointer_gestures() {
    let mut h = Harness::new();
    let response = h.frame(vec![])[0].clone();
    response.request_focus();
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.absent_frame();
    response.request_focus();
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [0, 0]);
    let pos = response.rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
    h.absent_frame();
    h.frame(vec![]);
    h.frame(vec![pointer_event(pos, false)]);
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn long_pointer_hold_activates_but_outside_and_disabled_releases_cancel() {
    let mut h = Harness::new();
    let response = h.frame(vec![])[0].clone();
    let pos = response.rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
    h.time += 1.0;
    h.frame(vec![pointer_event(pos, false)]);
    assert_eq!(h.changes, [1, 0]);
    h.frame(vec![pointer_event(pos, true)]);
    let outside = egui::pos2(300.0, 700.0);
    h.frame(vec![
        egui::Event::PointerMoved(outside),
        pointer_event(outside, false),
    ]);
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
    h.enabled = false;
    h.frame(vec![]);
    h.enabled = true;
    h.frame(vec![pointer_event(pos, false)]);
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn click_only_cover_blocks_short_and_long_pointer_releases() {
    for long in [false, true] {
        let mut h = Harness::new();
        let response = h.frame(vec![])[0].clone();
        let pos = background(&h.drawings[0]).rect.center();
        h.frame(vec![
            egui::Event::PointerMoved(pos),
            pointer_event(pos, true),
        ]);
        if long {
            h.time += 1.0;
        }
        h.cover = Some(response.rect);
        h.frame(vec![pointer_event(pos, false)]);
        assert_eq!(h.changes, [0, 0], "long={long}");
    }
}

#[test]
fn rounded_square_hit_geometry_excludes_corners_and_visual_overflow() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let square = background(&h.drawings[0]).rect;
    h.click(square.min + egui::vec2(0.1, 0.1));
    h.click(square.left_center() - egui::vec2(2.0, 0.0));
    assert_eq!(h.changes, [0, 0]);
    h.click(square.left_center() + egui::vec2(0.5, 0.0));
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn pointer_release_does_not_reclaim_focus_surrendered_by_host() {
    let mut h = Harness::new();
    let response = h.frame(vec![])[0].clone();
    let pos = background(&h.drawings[0]).rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
    assert_eq!(
        h.context.memory(|memory| memory.focused()),
        Some(response.id)
    );
    response.surrender_focus();
    h.frame(vec![pointer_event(pos, false)]);
    assert_eq!(h.changes, [1, 0]);
    assert_eq!(h.context.memory(|memory| memory.focused()), None);
}

#[test]
fn first_frame_disabled_has_final_opacity_and_cannot_activate() {
    let mut h = Harness::new();
    h.enabled = false;
    h.values[0] = true;
    let r = h.frame(vec![]);
    for drawing in &h.drawings {
        assert_eq!(drawing.opacity(), 1.0);
        assert_eq!(control(drawing).opacity(), 0.5);
    }
    assert!(mark(&h.drawings[0]).is_some());
    assert!(!r[0].enabled());
    r[0].request_focus();
    h.key(egui::Key::Space);
    h.key(egui::Key::Enter);
    h.click(r[0].rect.center());
    h.click(r[1].rect.center());
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [0, 0]);
    h.enabled = true;
    h.frame(vec![]);
    assert_eq!(control(&h.drawings[0]).opacity(), 1.0);
    h.enabled = false;
    h.frame(vec![]);
    assert_eq!(control(&h.drawings[0]).opacity(), 0.5);
}

#[test]
fn identities_and_label_ownership_follow_instances_through_reordering() {
    let mut h = Harness::new();
    let before = h.frame(vec![]);
    h.click(before[0].rect.center());
    h.reversed = true;
    h.theme = Theme::dark();
    h.label = "A renamed option";
    let after = h.frame(vec![]);
    assert_eq!(before[0].id, after[1].id);
    assert_eq!(before[1].id, after[0].id);
    assert!(after[1].has_focus());
    h.key(egui::Key::Space);
    assert_eq!(h.values, [false, false]);
    h.click(after[0].rect.center());
    assert_eq!(h.values, [false, true]);
    assert_eq!(h.changes, [2, 1]);
    assert_eq!(h.node(before[1].id).label(), Some("A renamed option"));
}

#[test]
fn automatic_identity_does_not_depend_on_label_text() {
    let mut h = Harness::new();
    h.automatic = true;
    let before = h.frame(vec![]);
    h.click(before[0].rect.center());
    h.label = "Different text, same allocation order";
    let after = h.frame(vec![]);
    assert_eq!(before[0].id, after[0].id);
    assert_eq!(before[1].id, after[1].id);
    assert_ne!(after[0].id, after[1].id);
    assert_eq!(h.values, [true, false]);
    h.click(after[1].rect.center());
    assert_eq!(h.values, [true, true]);
}

#[test]
fn wrapped_labels_keep_gap_line_height_and_clickable_last_line() {
    let mut h = Harness::new();
    h.width = 180.0;
    h.label = "A deliberately long checkbox label with enough words to wrap onto several lines";
    let r = h.frame(vec![]);
    let square = background(&h.drawings[0]).rect;
    let text = label(&h.drawings[0]);
    let text_bounds = text.galley.rect.translate(text.pos.to_vec2());
    assert!(r[0].rect.height() > 20.0);
    assert!(r[0].rect.right() <= h.width);
    close(square.center().y, r[0].rect.center().y);
    close(text_bounds.left(), square.right() + 8.0);
    for section in &text.galley.job.sections {
        assert_eq!(section.format.font_id.size, 14.0);
        assert_eq!(section.format.line_height, Some(20.0));
    }
    h.click(egui::pos2(
        r[0].rect.left() + 30.0,
        r[0].rect.bottom() - 3.0,
    ));
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn fixed_geometry_shadow_and_lucide_mark_are_in_the_actual_drawing() {
    for dark in [false, true] {
        let mut h = Harness::new();
        h.theme = if dark { Theme::dark() } else { Theme::light() };
        h.theme.metrics.control_height = 72.0;
        h.theme.metrics.corner_radius = egui::CornerRadius::same(20);
        h.theme.typography.body = 29.0;
        let r = h.frame(vec![]);
        let drawing = &h.drawings[0];
        let square = background(drawing).rect;
        assert_eq!(square.size(), egui::vec2(16.0, 16.0));
        assert_eq!(
            background(drawing).corner_radius,
            egui::CornerRadius::same(4)
        );
        assert_eq!(border(drawing).corner_radius, egui::CornerRadius::same(4));
        assert_eq!(border(drawing).stroke_kind, egui::StrokeKind::Inside);
        assert_eq!(border(drawing).stroke.width, 1.0);
        assert_eq!(border(drawing).stroke.color, h.theme.colors.input);
        assert_eq!(r[0].rect.height(), 20.0);
        assert_eq!(control(drawing).bounds(), square.expand(4.0));
        assert_eq!(drawing.bounds(), r[0].rect.union(square.expand(4.0)));
        assert_eq!(
            background(drawing).fill,
            if dark {
                h.theme.colors.input.gamma_multiply(0.3)
            } else {
                egui::Color32::TRANSPARENT
            }
        );
        let Item::Shape(shadow) = &control(drawing).items()[0] else {
            panic!("shadow must be part of the control");
        };
        assert!(matches!(shadow.shape, egui::Shape::Mesh(_)));
        assert!(mark(drawing).is_none());
        for section in &label(drawing).galley.job.sections {
            assert_eq!(section.format.font_id.size, 14.0);
        }
        h.values[0] = true;
        h.frame(vec![]);
        let drawing = &h.drawings[0];
        assert_eq!(background(drawing).fill, h.theme.colors.primary.background);
        assert_eq!(
            border(drawing).stroke.color,
            h.theme.colors.primary.background
        );
        let shapes = mark(drawing).expect("checked mark appears on the first changed frame");
        assert_eq!(shapes.len(), 5);
        let scale = 14.0 / 24.0;
        let origin = square.center() - egui::vec2(7.0, 7.0);
        let points =
            [(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)].map(|(x, y)| origin + egui::vec2(x, y) * scale);
        for index in 0..2 {
            let egui::Shape::LineSegment {
                points: actual,
                stroke,
            } = &shapes[index]
            else {
                panic!("expected Lucide path segment");
            };
            assert_eq!(*actual, [points[index], points[index + 1]]);
            close(stroke.width, 2.0 * scale);
            assert_eq!(stroke.color, h.theme.colors.primary.foreground);
        }
        for index in 0..3 {
            let egui::Shape::Circle(circle) = &shapes[index + 2] else {
                panic!("expected round cap or join");
            };
            assert_eq!(circle.center, points[index]);
            close(circle.radius, scale);
            assert_eq!(circle.fill, h.theme.colors.primary.foreground);
        }
        assert_eq!(h.changes, [0, 0]);
        h.values[0] = false;
        h.frame(vec![]);
        assert!(mark(&h.drawings[0]).is_none());
    }
}

#[test]
fn border_variant_order_and_invalid_ring_colors_match_all_states() {
    for dark in [false, true] {
        for checked in [false, true] {
            for invalid in [false, true] {
                for focus_visible in [false, true] {
                    let mut h = Harness::new();
                    h.theme = if dark { Theme::dark() } else { Theme::light() };
                    h.values[0] = checked;
                    h.invalid = invalid;
                    let r = h.frame(vec![]);
                    if focus_visible {
                        r[0].request_focus();
                        h.frame(vec![]);
                        h.settle();
                    }
                    let colors = h.theme.colors;
                    let expected_border = if checked {
                        colors.primary.background
                    } else if invalid {
                        colors.destructive.background
                    } else if focus_visible {
                        colors.ring
                    } else {
                        colors.input
                    };
                    assert_eq!(border(&h.drawings[0]).stroke.color, expected_border);
                    if focus_visible {
                        let ring = ring(&h.drawings[0]).unwrap();
                        assert_eq!(ring.stroke.width, 3.0);
                        assert_eq!(ring.rect, background(&h.drawings[0]).rect.expand(3.0));
                        assert_eq!(ring.corner_radius, egui::CornerRadius::same(7));
                        assert_eq!(ring.stroke_kind, egui::StrokeKind::Inside);
                        assert_eq!(
                            ring.stroke.color,
                            if invalid {
                                colors.destructive.background.gamma_multiply(if dark {
                                    0.4
                                } else {
                                    0.2
                                })
                            } else {
                                colors.ring.gamma_multiply(0.5)
                            }
                        );
                    } else {
                        assert!(ring(&h.drawings[0]).is_none());
                    }
                    let node = h.node(r[0].id);
                    assert_eq!(node.role(), egui::Role::CheckBox);
                    assert_eq!(node.label(), Some("Enable this option"));
                    assert_eq!(
                        node.toggled(),
                        Some(if checked {
                            egui::accesskit::Toggled::True
                        } else {
                            egui::accesskit::Toggled::False
                        })
                    );
                    assert_eq!(
                        node.invalid(),
                        invalid.then_some(egui::accesskit::Invalid::True)
                    );
                }
            }
        }
    }
}

#[test]
fn only_the_focus_shadow_transitions_not_border_color_mark_or_opacity() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    r[0].request_focus();
    h.frame(vec![]);
    assert_eq!(border(&h.drawings[0]).stroke.color, h.theme.colors.ring);
    assert!(ring(&h.drawings[0]).is_none());
    h.frame(vec![]);
    let intermediate = ring(&h.drawings[0]).unwrap().stroke.width;
    assert!(intermediate > 0.0 && intermediate < 3.0);
    h.settle();
    let old_ring = ring(&h.drawings[0]).unwrap().stroke.color;
    h.invalid = true;
    h.values[0] = true;
    h.frame(vec![]);
    assert_eq!(ring(&h.drawings[0]).unwrap().stroke.color, old_ring);
    assert_eq!(
        border(&h.drawings[0]).stroke.color,
        h.theme.colors.primary.background
    );
    assert_eq!(
        background(&h.drawings[0]).fill,
        h.theme.colors.primary.background
    );
    assert!(mark(&h.drawings[0]).is_some());
    h.settle();
    assert_eq!(
        ring(&h.drawings[0]).unwrap().stroke.color,
        h.theme.colors.destructive.background.gamma_multiply(0.2)
    );
    h.theme = Theme::dark();
    h.frame(vec![]);
    assert_eq!(
        background(&h.drawings[0]).fill,
        h.theme.colors.primary.background
    );
    assert_eq!(
        border(&h.drawings[0]).stroke.color,
        h.theme.colors.primary.background
    );
    let old_label = label(&h.drawings[0]).clone();
    h.enabled = false;
    h.values[0] = false;
    h.frame(vec![]);
    assert_eq!(control(&h.drawings[0]).opacity(), 0.5);
    assert_eq!(h.drawings[0].opacity(), 1.0);
    assert_eq!(label(&h.drawings[0]), &old_label);
    assert!(mark(&h.drawings[0]).is_none());
    assert_eq!(
        background(&h.drawings[0]).fill,
        h.theme.colors.input.gamma_multiply(0.3)
    );
    assert_eq!(
        border(&h.drawings[0]).stroke.color,
        h.theme.colors.destructive.background
    );
    h.settle();
    assert!(ring(&h.drawings[0]).is_none());
    h.invalid = false;
    let r = h.frame(vec![]);
    assert_eq!(h.node(r[0].id).invalid(), None);
    assert_eq!(border(&h.drawings[0]).stroke.color, h.theme.colors.input);
}

#[test]
fn pointer_focus_has_no_ring_until_keyboard_modality_is_observed() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    h.click(r[0].rect.center());
    h.settle();
    assert!(h.context.memory(|memory| memory.has_focus(r[0].id)));
    assert!(ring(&h.drawings[0]).is_none());
    h.key(egui::Key::ArrowRight);
    h.settle();
    assert_eq!(ring(&h.drawings[0]).unwrap().stroke.width, 3.0);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn same_batch_value_and_invalid_changes_are_used_by_final_drawing_and_metadata() {
    let context = ordered_context();
    context.enable_accesskit();
    let theme = Theme::light();
    let id = egui::Id::unique("same-batch-checkbox");
    let mut value = false;
    let mut invalid = false;
    let mut changes = Vec::new();
    let mut drawing = None;
    context.memory_mut(|memory| memory.request_focus(id));
    let mut output = context.run_ui(
        egui::RawInput {
            events: vec![
                key_event(egui::Key::Space, true, false),
                key_event(egui::Key::Space, false, false),
                key_event(egui::Key::Enter, true, false),
                key_event(egui::Key::Enter, false, false),
                key_event(egui::Key::Space, true, false),
                key_event(egui::Key::Space, false, false),
            ],
            ..Default::default()
        },
        |ui| {
            context.begin_input();
            let registration = Checkbox::new(&theme, &value, "Current state")
                .id(id)
                .invalid(invalid)
                .register(&mut phase_ui(ui));
            assert!(!value);
            while let Some(event) = context.next_input_event() {
                if event.target == Some(id) && registration.handle(&event, &mut value).changed() {
                    changes.push(value);
                    invalid = true;
                }
                registration.cancel_unfocused();
            }
            drawing = Checkbox::new(&theme, &value, "Current state")
                .id(id)
                .invalid(invalid)
                .show(&mut phase_ui(ui))
                .drawing;
        },
    );
    assert_eq!(changes, [true, false]);
    assert!(!value);
    let drawing = drawing.unwrap();
    assert!(mark(&drawing).is_none());
    assert_eq!(
        border(&drawing).stroke.color,
        theme.colors.destructive.background
    );
    let tree = output.platform_output.accesskit_update.take().unwrap();
    let node = &tree
        .nodes
        .iter()
        .find(|(node_id, _)| *node_id == id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::False));
    assert_eq!(node.invalid(), Some(egui::accesskit::Invalid::True));
    output.drop_without_applying_deltas();
}

#[test]
fn accesskit_click_changes_only_an_enabled_checkbox() {
    let mut h = Harness::new();
    let owner = h.frame(vec![])[0].id;
    let click = || {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            action: egui::accesskit::Action::Click,
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: owner.accesskit_id(),
            data: None,
        })
    };
    h.frame(vec![click()]);
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
    h.enabled = false;
    h.frame(vec![click()]);
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn aligned_rich_labels_remain_beside_the_square_and_inside_the_row() {
    for align in [egui::Align::Center, egui::Align::Max] {
        let context = ordered_context();
        let theme = Theme::light();
        let value = false;
        context
            .run_ui(Default::default(), |ui| {
                let mut job = egui::text::LayoutJob::simple(
                    "Aligned checkbox label".into(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                    240.0,
                );
                job.halign = align;
                let id = egui::Id::unique("aligned-checkbox");
                context.begin_input();
                let _registration = Checkbox::new(&theme, &value, job.clone())
                    .id(id)
                    .register(&mut phase_ui(ui));
                while context.next_input_event().is_some() {}
                let output = Checkbox::new(&theme, &value, job)
                    .id(id)
                    .show(&mut phase_ui(ui));
                let drawing = output.drawing.unwrap();
                let text = label(&drawing);
                let bounds = text.galley.rect.translate(text.pos.to_vec2());
                close(bounds.left(), background(&drawing).rect.right() + 8.0);
                assert!(bounds.right() <= output.response.rect.right() + 0.1);
                assert!(bounds.top() >= output.response.rect.top() - 0.1);
                assert!(bounds.bottom() <= output.response.rect.bottom() + 0.1);
            })
            .drop_without_applying_deltas();
    }
}

#[test]
fn component_style_and_parent_style_are_preserved_in_both_phases() {
    let context = ordered_context();
    let theme = Theme::dark();
    let value = false;
    context
        .run_ui(Default::default(), |ui| {
            ui.style_mut().visuals.override_text_color = Some(egui::Color32::RED);
            ui.spacing_mut().interact_size.y = 47.0;
            ui.style_mut()
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .family = egui::FontFamily::Monospace;
            let before = ui.style().clone();
            context.begin_input();
            let mut register = phase_ui(ui);
            let _registration =
                Checkbox::new(&theme, &value, "Preserve host style").register(&mut register);
            assert_eq!(register.style().as_ref(), before.as_ref());
            while context.next_input_event().is_some() {}
            let mut draw = phase_ui(ui);
            let output = Checkbox::new(&theme, &value, "Preserve host style").show(&mut draw);
            assert_eq!(draw.style().as_ref(), before.as_ref());
            assert_eq!(ui.style().as_ref(), before.as_ref());
            for section in &label(&output.drawing.unwrap()).galley.job.sections {
                assert_eq!(section.format.font_id.family, egui::FontFamily::Monospace);
                assert_eq!(section.format.font_id.size, 14.0);
            }
        })
        .drop_without_applying_deltas();
}

struct Recorder(Arc<Mutex<Vec<Group>>>);

impl compositing::Backend for Recorder {
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

#[test]
fn painting_applies_parent_opacity_once_outside_the_completed_disabled_control() {
    let context = ordered_context();
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let _installation =
        compositing::install(&context, Arc::new(Recorder(recorded.clone()))).unwrap();
    let theme = Theme::light();
    let checked = true;
    context
        .run_ui(Default::default(), |ui| {
            let mut painter = ui.painter().clone();
            painter.multiply_opacity(0.4);
            context.begin_input();
            let mut register = phase_ui(ui);
            let _registration = register
                .add_enabled_ui(false, |ui| {
                    Checkbox::new(&theme, &checked, "Independent label").register(ui)
                })
                .inner;
            while context.next_input_event().is_some() {}
            let mut draw = phase_ui(ui);
            let output = draw
                .add_enabled_ui(false, |ui| {
                    Checkbox::new(&theme, &checked, "Independent label").show(ui)
                })
                .inner;
            assert_eq!(output.drawing.as_ref().unwrap().opacity(), 1.0);
            let response = output.paint(&painter).unwrap();
            assert!(!response.enabled());
        })
        .drop_without_applying_deltas();
    let groups = recorded.lock().unwrap();
    assert_eq!(groups.len(), 1);
    let drawing = &groups[0];
    close(drawing.opacity(), 0.4);
    assert_eq!(control(drawing).opacity(), 0.5);
    assert_eq!(background(drawing).fill, theme.colors.primary.background);
    assert_eq!(
        border(drawing).stroke.color,
        theme.colors.primary.background
    );
    assert!(mark(drawing).is_some());
    assert_eq!(
        label(drawing).fallback_color,
        theme.colors.background.foreground
    );
}

#[test]
fn painting_without_a_compositor_returns_an_explicit_error() {
    let context = ordered_context();
    let theme = Theme::light();
    let checked = false;
    context
        .run_ui(Default::default(), |ui| {
            context.begin_input();
            let _registration =
                Checkbox::new(&theme, &checked, "Needs a compositor").register(&mut phase_ui(ui));
            while context.next_input_event().is_some() {}
            let output =
                Checkbox::new(&theme, &checked, "Needs a compositor").show(&mut phase_ui(ui));
            assert!(matches!(
                output.paint(ui.painter()),
                Err(compositing::Error::MissingHost)
            ));
        })
        .drop_without_applying_deltas();
}

#[test]
#[should_panic(expected = "Checkbox requires a nonempty label")]
fn empty_label_is_rejected() {
    let checked = false;
    let _ = Checkbox::new(&Theme::light(), &checked, "  ");
}

fn untextured_mesh(ctx: &egui::Context, shape: egui::Shape) -> egui::epaint::Mesh {
    let mut mesh = egui::epaint::Mesh::default();
    for primitive in ctx.tessellate(
        vec![egui::epaint::ClippedShape::new(ctx.content_rect(), shape)],
        ctx.pixels_per_point(),
    ) {
        let egui::epaint::Primitive::Mesh(part) = primitive.primitive else {
            panic!("drawing test expects mesh primitives");
        };
        mesh.append(part);
    }
    mesh
}

fn over(source: [f32; 4], destination: [f32; 4]) -> [f32; 4] {
    std::array::from_fn(|channel| source[channel] + destination[channel] * (1.0 - source[3]))
}

fn mesh_sample(mesh: &egui::epaint::Mesh, point: egui::Pos2) -> [f32; 4] {
    let cross = |a: egui::Vec2, b: egui::Vec2| a.x * b.y - a.y * b.x;
    let mut sample = [0.0; 4];
    for triangle in mesh.indices.chunks_exact(3) {
        let mut vertices =
            [triangle[0], triangle[1], triangle[2]].map(|index| mesh.vertices[index as usize]);
        let area = cross(
            vertices[1].pos - vertices[0].pos,
            vertices[2].pos - vertices[0].pos,
        );
        if area == 0.0 {
            continue;
        }
        if area < 0.0 {
            vertices.swap(1, 2);
        }
        let [a, b, c] = vertices;
        let edges = [(b.pos, c.pos), (c.pos, a.pos), (a.pos, b.pos)];
        let weights = edges.map(|(a, b)| cross(b - a, point - a));
        // Half-open edges sample a shared triangle boundary only once.
        if edges.iter().zip(weights).any(|(&(a, b), weight)| {
            weight < 0.0 || (weight == 0.0 && !(b.y < a.y || (b.y == a.y && b.x > a.x)))
        }) {
            continue;
        }
        let colors = vertices.map(|vertex| vertex.color.to_array());
        let color = std::array::from_fn(|channel| {
            (0..3)
                .map(|i| weights[i] / area.abs() * f32::from(colors[i][channel]) / 255.0)
                .sum::<f32>()
        });
        sample = over(color, sample);
    }
    sample
}

#[test]
fn outer_shadow_excludes_the_transparent_square_interior() {
    for pixels_per_point in [1.0, 1.5, 2.0] {
        let mut h = Harness::new();
        h.context.set_pixels_per_point(pixels_per_point);
        h.frame(vec![]);
        let drawing = &h.drawings[0];
        let square = background(drawing).rect;
        let Item::Shape(shadow) = &control(drawing).items()[0] else {
            panic!("expected shadow");
        };
        let mesh = untextured_mesh(&h.context, shadow.shape.clone());
        for point in [
            square.center(),
            square.left_center() + egui::vec2(0.25, 0.3),
            square.center_bottom() - egui::vec2(0.3, 0.25),
        ] {
            assert_eq!(
                mesh_sample(&mesh, point),
                [0.0; 4],
                "shadow leaked at {point:?}"
            );
        }
        let outside = square.center_bottom() + egui::vec2(0.3, 0.75);
        assert!(
            mesh_sample(&mesh, outside)[3] > 0.0,
            "exterior shadow is missing"
        );
    }
}

#[test]
fn rich_label_ink_is_not_clipped_to_the_twenty_point_row() {
    for enabled in [true, false] {
        let context = ordered_context();
        let theme = Theme::light();
        let value = false;
        context
            .run_ui(Default::default(), |ui| {
                context.begin_input();
                let text = egui::RichText::new("Option gj").size(40.0).italics();
                let mut register = phase_ui(ui);
                register.add_space(48.0);
                let registration = register
                    .add_enabled_ui(enabled, |ui| {
                        Checkbox::new(&theme, &value, text.clone()).register(ui)
                    })
                    .inner;
                while context.next_input_event().is_some() {}
                let mut draw = phase_ui(ui);
                draw.add_space(48.0);
                let output = draw
                    .add_enabled_ui(enabled, |ui| {
                        Checkbox::new(&theme, &value, text.clone()).show(ui)
                    })
                    .inner;
                assert_eq!(registration.response.rect, output.response.rect);
                assert_eq!(output.response.rect.height(), 20.0);
                let drawing = output.drawing.unwrap();
                let ink = label(&drawing).visual_bounding_rect();
                assert!(
                    !output.response.rect.contains_rect(ink),
                    "test requires overflowing glyph ink"
                );
                assert!(
                    drawing.bounds().contains_rect(ink),
                    "group clips rich-label ink: {:?} versus {ink:?}",
                    drawing.bounds()
                );
                let Item::Shape(shape) = &drawing.items()[1] else {
                    panic!("label shape");
                };
                assert!(
                    shape.clip_rect.contains_rect(ink),
                    "label clip excludes glyph ink"
                );
                assert_eq!(
                    control(&drawing).bounds(),
                    background(&drawing).rect.expand(4.0)
                );
            })
            .drop_without_applying_deltas();
    }
}
