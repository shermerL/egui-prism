//! Public slider input, value boundaries, geometry, and identity.

use egui_prism::{egui, slider::Slider, theme::Theme};

struct Harness {
    context: egui::Context,
    theme: Theme,
    values: [f64; 2],
    changes: [usize; 2],
    enabled: bool,
    reversed: bool,
    width: f32,
    rail_width: f32,
    zoom: f32,
    step: f64,
    range: std::ops::RangeInclusive<f64>,
    time: f64,
    handles: Vec<egui::Pos2>,
    tree: Option<egui::accesskit::TreeUpdate>,
    shapes: Vec<egui::epaint::ClippedShape>,
    transform: egui::emath::TSTransform,
    modal: bool,
    navigation_neighbor: bool,
    release_focus_before_widgets: Option<egui::Id>,
}

impl Harness {
    fn new() -> Self {
        let context = egui::Context::default();
        context.enable_accesskit();
        context.options_mut(|o| o.ordered_input = true);
        Self {
            context,
            theme: Theme::light(),
            values: [0.0; 2],
            changes: [0; 2],
            enabled: true,
            reversed: false,
            width: 320.0,
            rail_width: 200.0,
            zoom: 1.0,
            step: 5.0,
            range: 0.0..=100.0,
            time: 0.0,
            handles: Vec::new(),
            tree: None,
            shapes: Vec::new(),
            transform: egui::emath::TSTransform::IDENTITY,
            modal: false,
            navigation_neighbor: false,
            release_focus_before_widgets: None,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        self.context.set_zoom_factor(self.zoom);
        let mut responses = None;
        let mut shapes = Vec::new();
        let mut output = self.context.run_ui(
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
                self.context.begin_input();
                if let Some(id) = self.release_focus_before_widgets {
                    ui.memory_mut(|memory| memory.surrender_focus(id));
                }
                ui.ctx().set_transform_layer(ui.layer_id(), self.transform);
                if self.modal {
                    ui.memory_mut(|memory| {
                        memory.set_modal_layer(egui::LayerId::new(
                            egui::Order::Foreground,
                            egui::Id::unique("slider-test-modal"),
                        ))
                    });
                }
                ui.spacing_mut().slider_width = self.rail_width;
                if self.navigation_neighbor {
                    let _ = ui.button("Neighbor");
                }
                let order = if self.reversed { [1, 0] } else { [0, 1] };
                let registered = order.map(|index| {
                    let id = ui.make_persistent_id(index);
                    ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                        ui.add_enabled_ui(self.enabled, |ui| {
                            Slider::new(
                                &self.theme,
                                "A labeled slider with enough words to wrap in a narrow host",
                            )
                            .range(self.range.clone())
                            .step_by(self.step)
                            .register(ui, self.values[index])
                        })
                        .inner
                    })
                    .inner
                });
                while let Some(event) = self.context.next_input_event() {
                    for (index, slider) in order.iter().zip(&registered) {
                        if event.target == Some(slider.id())
                            && slider
                                .handle(ui, &mut self.values[*index], &event)
                                .changed()
                        {
                            self.changes[*index] += 1;
                        }
                    }
                }
                responses = Some(
                    registered
                        .into_iter()
                        .zip(order)
                        .map(|(slider, index)| {
                            let output = slider.finish(ui, self.values[index]);
                            if let Some(group) = output.drawing {
                                shapes.extend(group.items().iter().filter_map(|item| match item {
                                    egui_prism::compositing::Item::Shape(shape) => {
                                        Some(shape.clone())
                                    }
                                    _ => None,
                                }));
                            }
                            output.response
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                );
            },
        );
        self.handles = shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Circle(c) if c.radius == 7.5 => Some(c.center),
                _ => None,
            })
            .collect();
        self.shapes = shapes;
        self.tree = output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn pointer(&mut self, pos: egui::Pos2, pressed: bool) {
        self.frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
    }
    fn click(&mut self, pos: egui::Pos2) {
        self.pointer(pos, true);
        self.pointer(pos, false);
    }
    fn key(&mut self, key: egui::Key) {
        self.modified_key(key, egui::Modifiers::NONE);
    }
    fn modified_key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            self.frame(vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }]);
        }
    }
    fn set_accessible(&mut self, id: egui::Id, value: f64) {
        self.frame(vec![egui::Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::SetValue,
                target_tree: egui::accesskit::TreeId::ROOT,
                target_node: id.accesskit_id(),
                data: Some(egui::accesskit::ActionData::NumericValue(value)),
            },
        )]);
    }
}

#[test]
fn pointer_uses_full_rail_while_thumb_uses_inset_travel() {
    for (width, zoom) in [(120.0, 1.0), (240.0, 1.0), (180.0, 1.5), (20.0, 1.0)] {
        let mut h = Harness::new();
        h.rail_width = width;
        h.zoom = zoom;
        h.step = 1.0;
        h.frame(vec![]);
        let r = h.frame(vec![]);
        let rail = r[0].rect;
        assert_eq!(h.handles.len(), 2);
        assert!((h.handles[0].x - (rail.left() + 8.0)).abs() < 0.01);
        h.values[0] = 100.0;
        h.frame(vec![]);
        assert!((h.handles[0].x - (rail.right() - 8.0)).abs() < 0.01);
        h.click(rail.left_center());
        assert_eq!(h.values[0], 0.0);
        h.click(rail.right_center());
        assert_eq!(h.values[0], 100.0);
        // Thumb presses focus without jumping; subsequent motion uses rail mapping.
        let thumb = h.handles[0];
        h.pointer(thumb, true);
        assert_eq!(h.values[0], 100.0);
        let quarter = egui::pos2(rail.left() + rail.width() * 0.25, rail.center().y);
        h.frame(vec![egui::Event::PointerMoved(quarter)]);
        h.pointer(quarter, false);
        assert_eq!(h.values, [25.0, 0.0]);
        assert!((h.handles[0].x - (rail.left() + 8.0 + (rail.width() - 16.0) * 0.25)).abs() < 0.01);
        let changes = h.changes;
        for _ in 0..3 {
            h.frame(vec![]);
        }
        assert_eq!(h.changes, changes);
    }
}

#[test]
fn keyboard_focus_step_bounds_and_disabled_writes_are_consistent() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.key(egui::Key::Tab);
    assert!(h.frame(vec![])[0].has_focus());
    h.key(egui::Key::ArrowRight);
    assert_eq!(h.values[0], 5.0);
    h.key(egui::Key::ArrowLeft);
    assert_eq!(h.values[0], 0.0);
    h.key(egui::Key::ArrowLeft);
    assert_eq!(h.values[0], 0.0);
    h.enabled = false;
    let changes = h.changes;
    h.key(egui::Key::ArrowRight);
    h.click(r[0].rect.right_center());
    h.set_accessible(r[0].id, 50.0);
    assert_eq!(h.values, [0.0; 2]);
    assert_eq!(h.changes, changes);
}

#[test]
fn nondividing_step_stays_in_range_and_idle_preserves_external_values() {
    let mut h = Harness::new();
    h.range = 0.2..=1.0;
    h.step = 0.6;
    h.values = [2.0, -2.0];
    h.frame(vec![]);
    let r = h.frame(vec![]);
    assert_eq!(h.values, [2.0, -2.0]);
    assert_eq!(h.changes, [0; 2]);
    h.set_accessible(r[0].id, 0.9);
    assert!((h.values[0] - 0.8).abs() < 1e-9);
    h.step = 0.5;
    h.set_accessible(r[0].id, 1.0);
    assert_eq!(h.values[0], 1.0);
    h.set_accessible(r[0].id, f64::NAN);
    assert!(h.values[0].is_finite());
}

#[test]
fn identity_label_and_value_survive_reorder_and_theme_change() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let before = h.frame(vec![]);
    before[0].request_focus();
    h.frame(vec![]);
    h.reversed = true;
    h.theme = Theme::dark();
    let after = h.frame(vec![]);
    assert_eq!(before[0].id, after[1].id);
    assert!(after[1].has_focus());
    h.key(egui::Key::ArrowRight);
    assert_eq!(h.values, [5.0, 0.0]);
    let node = &h
        .tree
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(id, _)| *id == before[0].id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.role(), egui::Role::Slider);
    assert_eq!(node.numeric_value(), Some(5.0));
    assert!(!node.labelled_by().is_empty());
}

#[test]
fn narrow_rail_and_wrapped_label_preserve_host_style() {
    let mut h = Harness::new();
    h.width = 100.0;
    h.frame(vec![]);
    let r = h.frame(vec![]);
    assert!(r[0].rect.right() <= h.width);
    let context = egui::Context::default();
    let theme = Theme::dark();
    let value = 0.5;
    context.options_mut(|o| o.ordered_input = true);
    let output = context.run_ui(Default::default(), |ui| {
        context.begin_input();
        ui.spacing_mut().slider_width = 277.0;
        let before = ui.style().clone();
        let slider = Slider::new(&theme, "Amount").register(ui, value);
        while context.next_input_event().is_some() {}
        let _ = slider.finish(ui, value);
        assert_eq!(ui.style().as_ref(), before.as_ref());
    });
    output.drop_without_applying_deltas();
}

#[test]
fn invalid_configuration_is_rejected() {
    for (range, step) in [
        (1.0..=0.0, 1.0),
        (0.0..=f64::INFINITY, 1.0),
        (0.0..=1.0, 0.0),
        (0.0..=1.0, f64::NAN),
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                let _ = Slider::new(&Theme::light(), "Amount")
                    .range(range)
                    .step_by(step);
            })
            .is_err()
        );
    }
}

#[test]
fn thumb_travel_never_reverses_on_narrow_rails() {
    for width in [0.0, 8.0, 16.0, 20.0, 25.0, 32.0] {
        let mut h = Harness::new();
        h.rail_width = width;
        h.frame(vec![]);
        h.frame(vec![]);
        let left = h.handles[0];
        h.values[0] = 100.0;
        h.frame(vec![]);
        let right = h.handles[0];
        assert!(right.x >= left.x, "width {width}: {left:?} -> {right:?}");
    }
}

#[test]
fn radix_keyboard_endpoints_and_modifiers_share_the_step_grid() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    r[0].request_focus();
    h.key(egui::Key::End);
    assert_eq!(h.values[0], 100.0);
    h.key(egui::Key::PageDown);
    assert_eq!(h.values[0], 50.0);
    h.modified_key(egui::Key::ArrowLeft, egui::Modifiers::SHIFT);
    assert_eq!(h.values[0], 0.0);
    h.modified_key(egui::Key::ArrowUp, egui::Modifiers::CTRL);
    assert_eq!(h.values[0], 5.0);
    h.modified_key(egui::Key::ArrowUp, egui::Modifiers::ALT);
    assert_eq!(h.values[0], 10.0);
    h.key(egui::Key::PageUp);
    assert_eq!(h.values[0], 60.0);
    h.key(egui::Key::ArrowDown);
    assert_eq!(h.values[0], 55.0);
    h.key(egui::Key::Home);
    assert_eq!(h.values[0], 0.0);
    h.range = 0.0..=1.0;
    h.step = 0.3;
    h.key(egui::Key::End);
    assert_eq!(
        h.values[0], 0.9,
        "Radix snaps End to its step grid before clamping"
    );
    h.key(egui::Key::ArrowUp);
    assert_eq!(h.values[0], 1.0);
}

#[test]
fn disable_during_drag_and_release_outside_do_not_leak_edits() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.pointer(r[0].rect.center(), true);
    assert_eq!(h.values[0], 50.0);
    h.enabled = false;
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.right_center())]);
    h.pointer(r[0].rect.right_center(), false);
    h.enabled = true;
    h.frame(vec![]);
    assert_eq!(h.values[0], 50.0);
    h.pointer(h.handles[0], true);
    let outside = r[0].rect.right_center() + egui::vec2(100.0, 100.0);
    h.frame(vec![egui::Event::PointerMoved(outside)]);
    h.pointer(outside, false);
    assert_eq!(h.values[0], 100.0);
    let changes = h.changes;
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.center())]);
    assert_eq!(h.changes, changes);
}

#[test]
fn painted_geometry_and_roles_do_not_depend_on_egui_control_height() {
    for theme in [Theme::light(), Theme::dark()] {
        let mut h = Harness::new();
        h.theme = theme;
        h.context
            .all_styles_mut(|style| style.spacing.interact_size.y = 64.0);
        h.frame(vec![]);
        h.frame(vec![]);
        let thumb = h
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Circle(circle) if circle.fill == egui::Color32::WHITE => Some(circle),
                _ => None,
            })
            .unwrap();
        assert_eq!(thumb.radius * 2.0 + thumb.stroke.width, 16.0);
        assert_eq!(
            thumb.stroke,
            egui::Stroke::new(1.0, h.theme.colors.primary.background)
        );
        let rail = h
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Rect(rect) if rect.fill == h.theme.colors.muted.background => {
                    Some(rect)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(rail.rect.height(), 6.0);
        assert_eq!(rail.corner_radius, egui::CornerRadius::same(3));
        assert!(h.shapes.iter().any(
            |shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.blur_width == 3.0)
        ));
    }
}

#[test]
fn disabling_cancels_capture_even_if_reenabled_before_release() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.pointer(r[0].rect.center(), true);
    h.enabled = false;
    h.frame(vec![]);
    h.enabled = true;
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.right_center())]);
    assert_eq!(h.values[0], 50.0);
    h.pointer(r[0].rect.right_center(), false);
}

#[test]
fn fast_track_click_in_one_input_frame_updates_once() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    assert_eq!(h.values[0], 50.0);
    assert_eq!(h.changes[0], 1);
}

#[test]
fn modal_layer_cancels_capture_and_blocks_accessibility_writes() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.pointer(r[0].rect.center(), true);
    h.modal = true;
    h.frame(vec![]);
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.right_center())]);
    assert_eq!(h.values[0], 50.0);
    h.set_accessible(r[0].id, 100.0);
    assert_eq!(h.values[0], 50.0);
    h.modal = false;
    h.frame(vec![]);
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.left_center())]);
    assert_eq!(h.values[0], 50.0);
    h.pointer(r[0].rect.left_center(), false);
}

#[test]
fn transformed_layer_pointer_mapping_uses_local_coordinates() {
    for scale in [1.0, 1.5] {
        let mut h = Harness::new();
        h.width = 800.0;
        h.transform = egui::emath::TSTransform {
            scaling: scale,
            translation: egui::vec2(100.0, 20.0),
        };
        h.frame(vec![]);
        let r = h.frame(vec![]);
        let middle = h.transform * r[0].rect.center();
        h.click(middle);
        assert_eq!(h.values[0], 50.0);
        let three_quarters = h.transform
            * egui::pos2(
                r[0].rect.left() + 0.75 * r[0].rect.width(),
                r[0].rect.center().y,
            );
        h.pointer(middle, true);
        h.frame(vec![egui::Event::PointerMoved(three_quarters)]);
        h.pointer(three_quarters, false);
        assert_eq!(h.values[0], 75.0);
    }
}

#[test]
fn complete_thumb_drag_in_one_frame_is_not_discarded() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let from = h.handles[0];
    let to = egui::pos2(
        r[0].rect.left() + 0.75 * r[0].rect.width(),
        r[0].rect.center().y,
    );
    h.frame(vec![
        egui::Event::PointerMoved(from),
        egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerMoved(to),
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    assert_eq!(h.values[0], 75.0);
    assert_eq!(h.changes[0], 1);
}

#[test]
fn track_click_retains_thumb_focus_for_following_keys() {
    let mut h = Harness::new();
    h.navigation_neighbor = true;
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.click(r[0].rect.right_center());
    assert_eq!(h.values[0], 100.0);
    h.key(egui::Key::ArrowLeft);
    assert_eq!(h.values[0], 95.0);
    assert!(h.frame(vec![])[0].has_focus());
}

#[test]
fn capture_release_preserves_host_surrender_until_a_new_press() {
    for release_before_widgets in [false, true] {
        let mut h = Harness::new();
        h.frame(vec![]);
        let responses = h.frame(vec![]);
        let id = responses[0].id;
        let pos = responses[0].rect.right_center();
        h.pointer(pos, true);
        assert!(h.context.memory(|memory| memory.has_focus(id)));
        if release_before_widgets {
            h.release_focus_before_widgets = Some(id);
        } else {
            responses[0].surrender_focus();
        }
        h.pointer(pos, false);
        assert!(!h.context.memory(|memory| memory.has_focus(id)));
        assert_eq!(h.values[0], 100.0);
        h.release_focus_before_widgets = None;
        h.key(egui::Key::ArrowLeft);
        assert_eq!(h.values[0], 100.0);
        h.pointer(pos, true);
        assert!(h.context.memory(|memory| memory.has_focus(id)));
    }
}

fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
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

#[test]
fn navigation_and_arrows_edit_only_the_current_owner_in_one_pass() {
    use egui::{Key, Modifiers};
    for passes in [1, 2] {
        let mut h = Harness::new();
        h.context
            .options_mut(|o| o.max_passes = passes.try_into().unwrap());
        h.frame(vec![
            key_event(Key::Tab, Modifiers::NONE),
            key_event(Key::ArrowRight, Modifiers::NONE),
            key_event(Key::Tab, Modifiers::NONE),
            key_event(Key::ArrowRight, Modifiers::NONE),
        ]);
        assert_eq!(h.values, [5.0, 5.0]);
        h.frame(vec![
            key_event(Key::Tab, Modifiers::SHIFT),
            key_event(Key::ArrowRight, Modifiers::NONE),
        ]);
        assert_eq!(h.values, [10.0, 5.0]);
        let responses = h.frame(vec![]);
        responses[1].request_focus();
        h.frame(vec![key_event(Key::ArrowRight, Modifiers::NONE)]);
        assert_eq!(h.values, [10.0, 10.0]);
        assert_eq!(h.changes, [2, 2]);
    }
}

#[test]
fn moved_thumb_is_registered_before_the_next_gesture() {
    let mut h = Harness::new();
    let rail = h.frame(vec![])[0].rect;
    let at_end = rail.right_center();
    let inset_end = at_end - egui::vec2(8.0, 0.0);
    h.frame(vec![
        pointer_event(at_end, true),
        pointer_event(at_end, false),
        pointer_event(inset_end, true),
        pointer_event(inset_end, false),
        key_event(egui::Key::ArrowLeft, egui::Modifiers::NONE),
    ]);
    assert_eq!(h.values[0], 95.0);
    assert_eq!(
        h.changes[0], 2,
        "a thumb press must not jump to its inset coordinate"
    );
}

#[test]
fn tab_restores_keyboard_ring_after_pointer_focus() {
    let mut h = Harness::new();
    let r = h.frame(vec![]);
    h.click(r[0].rect.center());
    h.key(egui::Key::Tab);
    h.modified_key(egui::Key::Tab, egui::Modifiers::SHIFT);
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert!(
        h.shapes
            .iter()
            .any(|s| matches!(&s.shape, egui::Shape::Circle(c) if c.stroke.width == 4.0))
    );
}

#[test]
fn lost_pointer_and_window_focus_cancel_mapping_until_a_new_press() {
    for cancellation in [egui::Event::PointerGone, egui::Event::WindowFocused(false)] {
        let mut h = Harness::new();
        let r = h.frame(vec![]);
        h.pointer(r[0].rect.center(), true);
        h.frame(vec![cancellation]);
        h.frame(vec![
            egui::Event::WindowFocused(true),
            egui::Event::PointerMoved(r[0].rect.right_center()),
        ]);
        h.pointer(r[0].rect.right_center(), false);
        assert_eq!(h.values[0], 50.0);
        h.click(r[0].rect.right_center());
        assert_eq!(h.values[0], 100.0);
    }
}

#[test]
fn authoritative_rounded_parts_leave_empty_corners_to_the_underlying_widget() {
    let context = egui::Context::default();
    context.options_mut(|o| {
        o.ordered_input = true;
        o.max_passes = 1.try_into().unwrap();
    });
    let theme = Theme::light();
    let mut value = 50.0;
    let mut rail = egui::Rect::NOTHING;
    let under = egui::Id::unique("under-slider");
    let mut targets = Vec::new();
    let mut run = |events| {
        context
            .run_ui(
                egui::RawInput {
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 200.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    context.begin_input();
                    ui.interact(ui.max_rect(), under, egui::Sense::click());
                    let slider = Slider::new(&theme, "Amount")
                        .range(0.0..=100.0)
                        .register(ui, value);
                    while let Some(event) = context.next_input_event() {
                        if matches!(
                            event.trigger,
                            egui::InputTrigger::Event {
                                event: egui::Event::PointerButton { pressed: true, .. },
                                ..
                            }
                        ) {
                            targets.push(event.target);
                        }
                        if event.target == Some(slider.id()) {
                            slider.handle(ui, &mut value, &event);
                        }
                    }
                    rail = slider.finish(ui, value).response.rect;
                },
            )
            .drop_without_applying_deltas();
        rail
    };
    let bounds = run(vec![]);
    let center = bounds.center();
    let empty = egui::pos2(bounds.left() + 1.0, center.y - 7.0);
    let round_corner = center - egui::vec2(7.5, 7.5);
    run(vec![
        pointer_event(empty, true),
        pointer_event(empty, false),
        pointer_event(round_corner, true),
        pointer_event(round_corner, false),
    ]);
    assert_eq!(targets, vec![Some(under), Some(under)]);
    assert_eq!(value, 50.0);
}

#[test]
fn releasing_at_a_different_coordinate_does_not_edit_without_motion() {
    let mut h = Harness::new();
    let rail = h.frame(vec![])[0].rect;
    h.frame(vec![
        pointer_event(rail.center(), true),
        pointer_event(rail.right_center(), false),
    ]);
    assert_eq!(h.values[0], 50.0);
    assert_eq!(h.changes[0], 1);
}

#[test]
fn disabled_output_has_one_group_opacity_and_unfaded_thumb() {
    use egui_prism::compositing::{self, Backend, Error, Group, Item};
    use std::sync::{Arc, Mutex};
    #[derive(Default)]
    struct Capture(Mutex<Vec<Group>>);
    impl Backend for Capture {
        fn callback(&self, _: &egui::Context, group: Group) -> Result<egui::PaintCallback, Error> {
            let rect = group.bounds();
            self.0.lock().unwrap().push(group);
            Ok(egui::PaintCallback {
                rect,
                callback: Arc::new(()),
            })
        }
    }

    let context = egui::Context::default();
    context.options_mut(|o| o.ordered_input = true);
    let backend = Arc::new(Capture::default());
    let _installation = compositing::install(&context, backend.clone()).unwrap();
    let theme = Theme::light();
    context
        .run_ui(Default::default(), |ui| {
            context.begin_input();
            ui.set_opacity(0.8);
            let painter = ui.painter().clone();
            // A zero egui disabled alpha cannot be recovered by division.
            ui.visuals_mut().disabled_alpha = 0.0;
            let slider = ui
                .add_enabled_ui(false, |ui| {
                    Slider::new(&theme, "Disabled").register(ui, 0.5)
                })
                .inner;
            while context.next_input_event().is_some() {}
            slider.finish(ui, 0.5).paint(&painter).unwrap();
        })
        .drop_without_applying_deltas();
    let groups = backend.0.lock().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].opacity(), 0.4);
    assert!(groups[0].items().iter().any(|item| matches!(item,
        Item::Shape(shape) if matches!(&shape.shape, egui::Shape::Circle(c) if c.fill == egui::Color32::WHITE))));
}
