//! Public checkbox interaction, identity, layout, and host-style boundaries.

use egui_prism::{checkbox::Checkbox, egui, theme::Theme};

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
}

impl Harness {
    fn new() -> Self {
        Self {
            context: egui::Context::default(),
            theme: Theme::light(),
            values: [false; 2],
            changes: [0; 2],
            enabled: true,
            reversed: false,
            long: false,
            width: 320.0,
            time: 0.0,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let mut responses = None;
        let output = self.context.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(self.width, 800.0))),
            events, time: Some(self.time), ..Default::default()
        }, |ui| {
            responses = Some((if self.reversed { [1, 0] } else { [0, 1] }).map(|index| {
                let id = ui.make_persistent_id(index);
                ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                    let label = if self.long { "A deliberately long checkbox label with enough words to wrap on multiple lines in a narrow host" } else { "Enable this option" };
                    let response = ui.add_enabled(self.enabled, Checkbox::new(&self.theme, &mut self.values[index], label));
                    if response.changed() { self.changes[index] += 1; }
                    response
                }).inner
            }));
        });
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
fn box_and_label_toggle_once_and_idle_frames_do_not_repeat() {
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
        ui.add(Checkbox::new(&theme, &mut value, "Keep the host style"));
        unchanged = ui.style().as_ref() == before.as_ref();
    });
    output.drop_without_applying_deltas();
    assert!(unchanged);
}

#[test]
#[should_panic(expected = "Checkbox requires a nonempty label")]
fn empty_label_is_rejected() {
    let mut checked = false;
    let _ = Checkbox::new(&Theme::light(), &mut checked, "  ");
}
