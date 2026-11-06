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
}

impl Harness {
    fn new() -> Self {
        let context = egui::Context::default();
        context.enable_accesskit();
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
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        self.context.set_zoom_factor(self.zoom);
        let mut responses = None;
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
                ui.spacing_mut().slider_width = self.rail_width;
                responses = Some((if self.reversed { [1, 0] } else { [0, 1] }).map(|index| {
                    let id = ui.make_persistent_id(index);
                    ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                        let response = ui.add_enabled(
                            self.enabled,
                            Slider::new(
                                &self.theme,
                                &mut self.values[index],
                                "A labeled slider with enough words to wrap in a narrow host",
                            )
                            .range(self.range.clone())
                            .step_by(self.step),
                        );
                        if response.changed() {
                            self.changes[index] += 1;
                        }
                        response
                    })
                    .inner
                }));
            },
        );
        self.handles = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Circle(c) if c.radius == 7.5 => Some(c.center),
                _ => None,
            })
            .collect();
        self.shapes = output.shapes.clone();
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
    let mut value = 0.5;
    let output = context.run_ui(Default::default(), |ui| {
        ui.spacing_mut().slider_width = 277.0;
        let before = ui.style().clone();
        ui.add(Slider::new(&theme, &mut value, "Amount"));
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
                let mut value = 0.0;
                let _ = Slider::new(&Theme::light(), &mut value, "Amount")
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
