use super::*;
use egui_prism::theme::Theme;

struct Harness {
    context: egui::Context,
    case: InteractionCase,
    time: f64,
    width: f32,
}

impl Harness {
    fn new() -> Self {
        let context = egui::Context::default();
        context.all_styles_mut(|style| Theme::light().apply(style));
        Self {
            context,
            case: InteractionCase::default(),
            time: 0.0,
            width: 640.0,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>, long_labels: bool) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(self.width, 1000.0),
            )),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut responses = None;
        let output = self.context.run_ui(input, |ui| {
            responses = Some(self.case.show(ui, long_labels));
        });
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn click(&mut self, position: egui::Pos2) {
        self.frame(vec![egui::Event::PointerMoved(position)], false);
        for pressed in [true, false] {
            self.frame(
                vec![egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
                false,
            );
        }
    }

    fn key(&mut self, key: egui::Key, pressed: bool) {
        self.frame(
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            false,
        );
    }
}

#[test]
fn pointer_activation_is_once_and_disabled_actions_are_inert() {
    let mut harness = Harness::new();
    harness.frame(vec![], false);
    let responses = harness.frame(vec![], false);
    harness.click(responses[0].rect.center());
    assert_eq!(harness.case.instances[0].activations, 1);
    assert_eq!(harness.case.instances[1].activations, 0);
    for _ in 0..4 {
        harness.frame(vec![], false);
    }
    assert_eq!(harness.case.instances[0].activations, 1);
    harness.case.disabled = true;
    let responses = harness.frame(vec![], false);
    harness.click(responses[0].rect.center());
    harness.click(responses[1].rect.center());
    assert_eq!(harness.case.instances[0].activations, 1);
    assert_eq!(harness.case.instances[1].activations, 0);
}

#[test]
fn keyboard_activation_and_tab_navigation_use_real_input_events() {
    let mut harness = Harness::new();
    harness.frame(vec![], false);
    let responses = harness.frame(vec![], false);
    // The first instance has text, checkbox, then action in tab order.
    for _ in 0..3 {
        harness.key(egui::Key::Tab, true);
        harness.key(egui::Key::Tab, false);
    }
    assert_eq!(
        harness.context.memory(|memory| memory.focused()),
        Some(responses[0].id)
    );
    harness.key(egui::Key::Enter, true);
    harness.key(egui::Key::Enter, false);
    for _ in 0..4 {
        harness.frame(vec![], false);
    }
    assert_eq!(harness.case.instances[0].activations, 1);
    assert_eq!(harness.case.instances[1].activations, 0);
    harness.case.disabled = true;
    harness.frame(vec![], false);
    harness.key(egui::Key::Enter, true);
    harness.key(egui::Key::Enter, false);
    assert_eq!(harness.case.instances[0].activations, 1);
    assert_eq!(harness.case.instances[1].activations, 0);
}

#[test]
fn ids_and_values_survive_reordering_and_theme_switches() {
    let mut harness = Harness::new();
    harness.frame(vec![], false);
    let before = harness.frame(vec![], false);
    harness.case.instances[0].text = "first".into();
    harness.case.instances[1].text = "second".into();
    harness.case.instances[1].checked = true;
    harness.case.reversed = true;
    harness
        .context
        .all_styles_mut(|style| Theme::dark().apply(style));
    let after = harness.frame(vec![], false);
    assert_eq!(before[0].id, after[1].id);
    assert_eq!(before[1].id, after[0].id);
    harness.click(after[0].rect.center());
    assert_eq!(harness.case.instances[0].activations, 0);
    assert_eq!(harness.case.instances[1].activations, 1);
    assert_eq!(harness.case.instances[0].text, "first");
    assert_eq!(harness.case.instances[1].text, "second");
    assert!(harness.case.instances[1].checked);
}

#[test]
fn long_actions_wrap_within_restricted_width() {
    let mut harness = Harness::new();
    harness.width = 280.0;
    harness.frame(vec![], true);
    for response in harness.frame(vec![], true) {
        assert!(response.rect.right() <= harness.width);
        assert!(response.rect.height() > Theme::light().metrics.control_height);
    }
}
