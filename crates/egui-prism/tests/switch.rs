//! Public switch interaction, identity, layout, and host-style boundaries.

use egui_prism::{
    egui,
    switch::{Switch, SwitchSize},
    theme::Theme,
};

struct Harness {
    context: egui::Context,
    theme: Theme,
    values: [bool; 2],
    changes: [usize; 2],
    enabled: bool,
    reversed: bool,
    long: bool,
    width: f32,
    time: f64,
    circles: Vec<egui::epaint::CircleShape>,
    tracks: Vec<egui::epaint::RectShape>,
    outlines: Vec<egui::epaint::RectShape>,
    size: SwitchSize,
    invalid: bool,
    tree: Option<egui::accesskit::TreeUpdate>,
}

impl Harness {
    fn new() -> Self {
        let context = egui::Context::default();
        context.enable_accesskit();
        Self {
            context,
            theme: Theme::light(),
            values: [false; 2],
            changes: [0; 2],
            enabled: true,
            reversed: false,
            long: false,
            width: 320.0,
            time: 0.0,
            circles: Vec::new(),
            tracks: Vec::new(),
            outlines: Vec::new(),
            size: SwitchSize::Default,
            invalid: false,
            tree: None,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let mut responses = None;
        let mut output = self.context.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(self.width, 800.0))),
            events, time: Some(self.time), ..Default::default()
        }, |ui| {
            responses = Some((if self.reversed { [1, 0] } else { [0, 1] }).map(|index| {
                let id = ui.make_persistent_id(index);
                ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                    let label = if self.long { "A deliberately long switch label with enough words to wrap on multiple lines in a narrow host" } else { "Enable this option" };
                    let response = ui.add_enabled(self.enabled, Switch::new(&self.theme, &mut self.values[index], label).size(self.size).invalid(self.invalid));
                    if response.changed() { self.changes[index] += 1; }
                    response
                }).inner
            }));
        });
        self.circles = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Circle(circle) = &s.shape {
                    Some(*circle)
                } else {
                    None
                }
            })
            .collect();
        self.tracks = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Rect(rect) = &s.shape {
                    (rect.blur_width == 0.0 && rect.fill != egui::Color32::TRANSPARENT)
                        .then(|| rect.clone())
                } else {
                    None
                }
            })
            .collect();
        self.outlines = output
            .shapes
            .iter()
            .filter_map(|s| {
                if let egui::Shape::Rect(rect) = &s.shape {
                    Some(rect.clone())
                } else {
                    None
                }
            })
            .collect();
        self.tree = output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn click(&mut self, pos: egui::Pos2) {
        self.frame(vec![egui::Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            self.frame(vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }]);
        }
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
}

#[test]
fn track_and_label_toggle_once_and_idle_frames_do_not_repeat() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.click(egui::pos2(r[0].rect.left() + 6.0, r[0].rect.center().y));
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
    let r = h.frame(vec![]);
    h.click(egui::pos2(r[0].rect.right() - 3.0, r[0].rect.center().y));
    assert_eq!(h.values, [false, false]);
    for _ in 0..4 {
        h.frame(vec![]);
    }
    assert_eq!(h.changes, [2, 0]);
}

#[test]
fn keyboard_activation_and_first_disabled_frame_preserve_contract() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.key(egui::Key::Tab);
    assert_eq!(h.context.memory(|m| m.focused()), Some(r[0].id));
    h.key(egui::Key::Space);
    assert_eq!(h.values, [true, false]);
    h.key(egui::Key::Enter);
    assert_eq!(h.values, [false, false]);
    h.enabled = false;
    h.key(egui::Key::Space);
    h.key(egui::Key::Enter);
    let r = h.frame(vec![]);
    h.click(r[0].rect.center());
    h.click(r[1].rect.center());
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [2, 0]);
}

#[test]
fn identity_and_value_follow_instances_through_reorder_and_theme_change() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let before = h.frame(vec![]);
    h.click(before[0].rect.center());
    before[0].request_focus();
    assert!(h.frame(vec![])[0].has_focus());
    h.reversed = true;
    h.theme = Theme::dark();
    let after = h.frame(vec![]);
    assert_eq!(before[0].id, after[1].id);
    assert_eq!(before[1].id, after[0].id);
    assert!(after[1].has_focus());
    h.key(egui::Key::Space);
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [2, 0]);
    h.click(after[0].rect.center());
    assert_eq!(h.values, [false, true]);
    assert_eq!(h.changes, [2, 1]);
}

#[test]
fn wrapped_label_remains_in_the_same_hit_target() {
    let mut h = Harness::new();
    h.long = true;
    h.width = 180.0;
    h.frame(vec![]);
    let r = h.frame(vec![]);
    assert!(r[0].rect.right() <= h.width);
    assert!(r[0].rect.height() > h.theme.metrics.control_height);
    h.click(egui::pos2(
        r[0].rect.left() + 40.0,
        r[0].rect.bottom() - 3.0,
    ));
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn component_style_is_local_to_the_call() {
    let context = egui::Context::default();
    let theme = Theme::dark();
    let mut value = false;
    let mut unchanged = false;
    let output = context.run_ui(Default::default(), |ui| {
        ui.style_mut().visuals.override_text_color = Some(egui::Color32::RED);
        ui.spacing_mut().interact_size.y = 47.0;
        let before = ui.style().clone();
        ui.add(Switch::new(&theme, &mut value, "Keep the host style"));
        unchanged = ui.style().as_ref() == before.as_ref();
    });
    output.drop_without_applying_deltas();
    assert!(unchanged);
}

#[test]
#[should_panic(expected = "Switch requires a nonempty label")]
fn empty_label_is_rejected() {
    let mut checked = false;
    let _ = Switch::new(&Theme::light(), &mut checked, "  ");
}

#[test]
fn painted_thumb_and_accessibility_follow_the_value_at_different_sizes() {
    for (size, width, height, radius) in [
        (SwitchSize::Default, 32.0, 18.4, 8.0),
        (SwitchSize::Small, 24.0, 14.0, 6.0),
    ] {
        let mut h = Harness::new();
        h.size = size;
        h.theme.metrics.control_height = 72.0;
        h.frame(vec![]);
        let r = h.frame(vec![]);
        assert_eq!(h.circles.len(), 2);
        let off = h.circles[0].center;
        let bounds = h.tracks[0].rect;
        assert!((bounds.width() - width).abs() < 0.05);
        assert!((bounds.height() - height).abs() < 0.05);
        assert!((off.x - r[0].rect.left() - 1.0 - radius).abs() < 0.01);
        assert_eq!(r[0].rect.height(), 20.0);
        h.click(r[0].rect.center());
        for _ in 0..12 {
            h.frame(vec![]);
        }
        let on = h.circles[0].center;
        assert!((on.x - off.x - (radius * 2.0 - 2.0)).abs() < 0.01);
        let node = &h
            .tree
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(id, _)| *id == r[0].id.accesskit_id())
            .unwrap()
            .1;
        assert_eq!(node.role(), egui::Role::Switch);
        assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
        assert_eq!(node.label(), Some("Enable this option"));
    }
}

#[test]
fn aligned_labels_stay_beside_the_track_and_inside_the_hit_target() {
    for align in [egui::Align::Center, egui::Align::Max] {
        let context = egui::Context::default();
        let theme = Theme::light();
        let mut value = false;
        let mut response = None;
        let mut text_bounds = None;
        for _ in 0..2 {
            let output = context.run_ui(Default::default(), |ui| {
                let mut job = egui::text::LayoutJob::simple(
                    "Aligned switch label".into(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                    240.0,
                );
                job.halign = align;
                response = Some(ui.add(Switch::new(&theme, &mut value, job)));
            });
            for shape in &output.shapes {
                if let egui::Shape::Text(text) = &shape.shape {
                    text_bounds = Some(text.galley.rect.translate(text.pos.to_vec2()));
                }
            }
            output.drop_without_applying_deltas();
        }
        let response = response.unwrap();
        let text = text_bounds.unwrap();
        let label_left = response.rect.left() + 32.0 + 8.0;
        assert!(text.left() >= label_left - 0.1, "{align:?}: {text:?}");
        assert!(text.right() <= response.rect.right() + 0.1);
        assert!(text.top() >= response.rect.top() - 0.1);
        assert!(text.bottom() <= response.rect.bottom() + 0.1);
    }
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

#[test]
fn space_waits_for_release_and_cancels_on_blur_or_disabling() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    r[0].request_focus();
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.frame(vec![key_event(egui::Key::Space, true, true)]);
    assert_eq!(h.changes, [0, 0]);
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [1, 0]);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    r[1].request_focus();
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [1, 0]);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.enabled = false;
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    h.enabled = true;
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn pointer_focus_has_no_ring_keyboard_focus_has_reference_ring() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.click(r[0].rect.center());
    assert!(h.frame(vec![])[0].has_focus());
    assert!(!h.outlines.iter().any(|r| r.stroke.width > 0.0));
    h.key(egui::Key::ArrowRight);
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert!(h.frame(vec![])[0].has_focus());
    assert!(h.outlines.iter().any(
        |r| r.stroke.width == 3.0 && r.stroke.color == h.theme.colors.ring.gamma_multiply(0.5)
    ));
    h.enabled = false;
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert!(!h.outlines.iter().any(|r| r.stroke.width > 0.0));
}

#[test]
fn hover_and_invalid_do_not_invent_reference_styles() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let before = h.circles[0];
    h.invalid = true;
    h.frame(vec![egui::Event::PointerMoved(r[0].rect.center())]);
    assert_eq!(before, h.circles[0]);
    assert!(!h.outlines.iter().any(|r| r.stroke.width > 0.0));
    let node = &h
        .tree
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(id, _)| *id == r[0].id.accesskit_id())
        .unwrap()
        .1;
    assert_eq!(node.invalid(), Some(egui::accesskit::Invalid::True));
}

#[test]
fn animation_moves_through_intermediate_frames_and_reverses_from_current_position() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let off = h.circles[0].center.x;
    h.values[0] = true;
    h.frame(vec![]);
    assert_eq!(h.changes, [0, 0]);
    assert!((h.circles[0].center.x - off).abs() < 0.01);
    h.frame(vec![]);
    let moving = h.circles[0].center.x;
    assert!(moving > off && moving < off + 14.0);
    h.values[0] = false;
    h.frame(vec![]);
    let reversed = h.circles[0].center.x;
    assert!(reversed >= moving && reversed < off + 14.0);
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert!((h.circles[0].center.x - off).abs() < 0.01);
    assert_eq!(h.changes, [0, 0]);
    assert_eq!(h.frame(vec![])[0].id, r[0].id);
}

#[test]
fn dark_thumb_changes_immediately_and_disabled_shapes_reach_half_opacity() {
    let mut h = Harness::new();
    h.theme = Theme::dark();
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(h.circles[0].fill, h.theme.colors.background.foreground);
    h.values[0] = true;
    h.frame(vec![]);
    assert_eq!(h.circles[0].fill, h.theme.colors.primary.foreground);
    h.enabled = false;
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert_eq!(
        h.circles[0].fill,
        h.theme.colors.primary.foreground.gamma_multiply(0.5)
    );
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn background_uses_css_oklab_midpoint_and_theme_changes_transition() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    h.values[0] = true;
    h.frame(vec![]);
    h.time += 0.075 - 1.0 / 60.0;
    h.frame(vec![]);
    // Reference: L=.922*(1-ease(.5)), ease(.5)=.77556, sRGB about 23.
    let halfway = h.tracks[0].fill;
    assert!((22..=25).contains(&halfway.r()), "{halfway:?}");
    assert_eq!(halfway.r(), halfway.g());
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert_eq!(h.tracks[0].fill, egui::Color32::BLACK);
    h.theme = Theme::dark();
    h.frame(vec![]);
    assert_eq!(h.tracks[0].fill, egui::Color32::BLACK);
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert_eq!(h.tracks[0].fill, h.theme.colors.primary.background);
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
fn completed_pointer_click_in_one_frame_acquires_keyboard_focus() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
        pointer_event(pos, false),
    ]);
    assert_eq!(h.context.memory(|m| m.focused()), Some(r[0].id));
    h.key(egui::Key::Space);
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [2, 0]);
}

#[test]
fn held_pointer_still_activates_after_egui_click_duration_expires() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
    h.time += 1.0;
    h.frame(vec![pointer_event(pos, false)]);
    assert_eq!(h.values, [true, false]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn two_complete_pointer_clicks_in_one_frame_restore_the_value() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
        pointer_event(pos, false),
        pointer_event(pos, true),
        pointer_event(pos, false),
    ]);
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [1, 0]);
}

#[test]
fn arrow_immediately_after_tab_keeps_focus_without_an_idle_frame() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.frame(vec![key_event(egui::Key::Tab, true, false)]);
    h.frame(vec![key_event(egui::Key::ArrowDown, true, false)]);
    assert_eq!(h.context.memory(|m| m.focused()), Some(r[0].id));
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn pointer_release_outside_or_after_disabling_cancels_activation() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    let pos = r[0].rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(pos),
        pointer_event(pos, true),
    ]);
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
    assert_eq!(h.values, [false, false]);
    assert_eq!(h.changes, [0, 0]);
}
