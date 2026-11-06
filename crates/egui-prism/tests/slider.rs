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
                egui::Shape::Circle(c) => Some(c.center),
                egui::Shape::Rect(r)
                    if r.rect.height() > self.theme.metrics.control_height * 0.5 =>
                {
                    Some(r.rect.center())
                }
                _ => None,
            })
            .collect();
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
        for pressed in [true, false] {
            self.frame(vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
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
fn rail_clicks_and_drag_match_painted_positions_at_sizes_and_scales() {
    for (width, zoom) in [(120.0, 1.0), (240.0, 1.0), (180.0, 1.5)] {
        let mut h = Harness::new();
        h.rail_width = width;
        h.zoom = zoom;
        h.frame(vec![]);
        let r = h.frame(vec![]);
        assert_eq!(h.handles.len(), 2);
        let left = h.handles[0];
        h.values[0] = 100.0;
        h.frame(vec![]);
        let right = h.handles[0];
        assert!(right.x > left.x);
        assert!(r[0].rect.contains(left) && r[0].rect.contains(right));
        h.click(left);
        assert_eq!(h.values[0], 0.0);
        h.click(right);
        assert_eq!(h.values[0], 100.0);
        h.click(left.lerp(right, 0.5));
        assert_eq!(h.values[0], 50.0);
        assert!((h.handles[0].x - left.lerp(right, 0.5).x).abs() < 0.5);
        h.pointer(left.lerp(right, 0.5), true);
        h.frame(vec![egui::Event::PointerMoved(left.lerp(right, 0.75))]);
        h.pointer(left.lerp(right, 0.75), false);
        assert_eq!(h.values, [75.0, 0.0]);
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
