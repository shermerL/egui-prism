//! Public text editing, identity, validation metadata, and layout checks.

use egui_prism::{egui, text_input::TextInput, theme::Theme};

struct Harness {
    context: egui::Context,
    theme: Theme,
    values: [String; 2],
    changes: [usize; 2],
    time: f64,
    enabled: bool,
    reversed: bool,
    error: bool,
    width: f32,
    tree: Option<egui::accesskit::TreeUpdate>,
}

impl Harness {
    fn new() -> Self {
        let context = egui::Context::default();
        context.enable_accesskit();
        Self {
            context,
            theme: Theme::light(),
            values: [String::new(), String::new()],
            changes: [0; 2],
            time: 0.0,
            enabled: true,
            reversed: false,
            error: false,
            width: 320.0,
            tree: None,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 0.6;
        let mut responses = None;
        let mut output = self.context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(self.width, 800.0),
                )),
                time: Some(self.time),
                events,
                ..Default::default()
            },
            |ui| {
                let order = if self.reversed { [1, 0] } else { [0, 1] };
                responses = Some(order.map(|index| {
                    let id = ui.make_persistent_id(index);
                    ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                        let mut input = TextInput::new(
                            &self.theme,
                            &mut self.values[index],
                            format!("Value {index}"),
                        );
                        if self.error {
                            input = input.error("A caller-provided error");
                        }
                        let response = ui.add_enabled(self.enabled, input);
                        if response.changed() {
                            self.changes[index] += 1;
                        }
                        response
                    })
                    .inner
                }));
            },
        );
        self.tree = output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn key(&mut self, key: egui::Key, command: bool) {
        for pressed in [true, false] {
            self.frame(vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    command,
                    ..Default::default()
                },
            }]);
        }
    }

    fn focus_first(&mut self) {
        self.frame(vec![]);
        self.frame(vec![])[0].request_focus();
        self.frame(vec![]);
    }
}

#[test]
fn text_selection_paste_and_undo_update_only_the_focused_value() {
    let mut h = Harness::new();
    h.focus_first();
    h.frame(vec![egui::Event::Text("original".into())]);
    h.frame(vec![]);
    h.key(egui::Key::A, true);
    h.frame(vec![egui::Event::Paste("replacement".into())]);
    assert_eq!(h.values, ["replacement", ""]);
    h.frame(vec![]);
    h.key(egui::Key::Z, true);
    assert_eq!(h.values, ["original", ""]);
    let changes = h.changes;
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(h.changes, changes);
}

#[test]
fn disabled_fields_ignore_editing_and_keyboard_input() {
    let mut h = Harness::new();
    h.focus_first();
    h.frame(vec![egui::Event::Text("retained".into())]);
    h.enabled = false;
    h.frame(vec![]);
    let changes = h.changes;
    h.frame(vec![
        egui::Event::Text("ignored".into()),
        egui::Event::Paste("ignored".into()),
    ]);
    h.key(egui::Key::Backspace, false);
    assert_eq!(h.values, ["retained", ""]);
    assert_eq!(h.changes, changes);
}

#[test]
fn stable_scope_preserves_focus_and_values_after_reorder_and_theme_change() {
    let mut h = Harness::new();
    h.focus_first();
    let before = h.frame(vec![egui::Event::Text("first".into())]);
    h.reversed = true;
    h.theme = Theme::dark();
    h.error = true;
    let after = h.frame(vec![]);
    assert_eq!(before[0].id, after[1].id);
    assert_eq!(before[1].id, after[0].id);
    assert!(after[1].has_focus());
    h.frame(vec![egui::Event::Text("!".into())]);
    assert_eq!(h.values, ["first!", ""]);
    h.key(egui::Key::Tab, false);
    let after_tab = h.frame(vec![]);
    assert!(after_tab.iter().any(egui::Response::has_focus));
}

#[test]
fn ime_preedit_and_commit_use_the_text_edit_path() {
    let mut h = Harness::new();
    h.focus_first();
    h.frame(vec![egui::Event::Ime(egui::ImeEvent::Preedit {
        text: "ni".into(),
        active_range_chars: Some(0..2),
    })]);
    h.frame(vec![egui::Event::Ime(egui::ImeEvent::Commit("你".into()))]);
    assert_eq!(h.values, ["你", ""]);
}

#[test]
fn error_description_is_linked_and_cleared_without_mutating_the_value() {
    let mut h = Harness::new();
    h.values[0] = "retained".into();
    h.error = true;
    let r = h.frame(vec![]);
    let node = h
        .tree
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(id, _)| *id == r[0].id.accesskit_id())
        .unwrap();
    assert_eq!(node.1.description(), Some("A caller-provided error"));
    assert_eq!(node.1.invalid(), Some(egui::accesskit::Invalid::True));
    assert!(!node.1.labelled_by().is_empty());
    h.error = false;
    let r = h.frame(vec![]);
    let node = h
        .tree
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(id, _)| *id == r[0].id.accesskit_id())
        .unwrap();
    assert_eq!(node.1.description(), None);
    assert_eq!(node.1.invalid(), None);
    assert_eq!(h.values[0], "retained");
    assert_eq!(h.changes, [0, 0]);
}

#[test]
fn long_content_stays_inside_a_narrow_field() {
    let mut h = Harness::new();
    h.width = 180.0;
    h.values[0] = "long content ".repeat(100);
    h.frame(vec![]);
    let r = h.frame(vec![]);
    assert!(r[0].rect.right() <= h.width);
    assert_eq!(h.values[0], "long content ".repeat(100));
}
