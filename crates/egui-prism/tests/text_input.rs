//! Public TextInput drawing, editing and ordered-host contracts.

use std::sync::{Arc, Mutex};

use egui_prism::{
    compositing::{self, Group, Item},
    egui,
    text_input::{TextInput, TextInputOutput},
    theme::Theme,
};

fn editor_id(index: usize) -> egui::Id {
    egui::Id::unique(("text-input-test", index))
}

fn key(key: egui::Key, command: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            command,
            ..Default::default()
        },
    }
}

fn shapes(group: &Group) -> impl Iterator<Item = &egui::epaint::ClippedShape> {
    group.items().iter().filter_map(|item| match item {
        Item::Shape(shape) => Some(shape),
        Item::Group(_) => None,
    })
}

fn text_shape<'a>(group: &'a Group, text: &str) -> &'a egui::epaint::TextShape {
    shapes(group)
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing text shape: {text:?}"))
}

fn border(group: &Group) -> &egui::epaint::RectShape {
    shapes(group)
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.stroke_kind == egui::StrokeKind::Inside && rect.stroke.width == 1.0 =>
            {
                Some(rect)
            }
            _ => None,
        })
        .expect("missing custom border")
}

fn ring(group: &Group) -> egui::epaint::RectShape {
    shapes(group)
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.stroke_kind == egui::StrokeKind::Outside && rect.stroke.width > 0.0 =>
            {
                Some(rect.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            egui::epaint::RectShape::stroke(
                egui::Rect::ZERO,
                0,
                egui::Stroke::NONE,
                egui::StrokeKind::Outside,
            )
        })
}

fn background(group: &Group, editor_rect: egui::Rect) -> &egui::epaint::RectShape {
    shapes(group)
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.rect == editor_rect && rect.stroke.width == 0.0 => {
                Some(rect)
            }
            _ => None,
        })
        .expect("missing custom background")
}

struct Harness {
    context: egui::Context,
    theme: Theme,
    values: [String; 2],
    enabled: bool,
    error: bool,
    reversed: bool,
    viewport_width: f32,
    column_width: f32,
    time: f64,
    changes: [usize; 2],
    frame_shapes: Vec<egui::epaint::ClippedShape>,
    tree: Option<egui::accesskit::TreeUpdate>,
}

impl Harness {
    fn new(theme: Theme) -> Self {
        let context = egui::Context::default();
        context.enable_accesskit();
        Self {
            context,
            theme,
            values: [String::new(), String::new()],
            enabled: true,
            error: false,
            reversed: false,
            viewport_width: 1000.0,
            column_width: 320.0,
            time: 0.0,
            changes: [0; 2],
            frame_shapes: Vec::new(),
            tree: None,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> Vec<TextInputOutput> {
        self.frame_after(0.6, events)
    }

    fn frame_after(&mut self, elapsed: f64, events: Vec<egui::Event>) -> Vec<TextInputOutput> {
        self.time += elapsed;
        let context = self.context.clone();
        let mut inputs = Vec::new();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(self.viewport_width, 1000.0),
                )),
                time: Some(self.time),
                events,
                ..Default::default()
            },
            |ui| {
                ui.set_width(self.column_width);
                let before_spacing = ui.spacing().interact_size;
                let before_extra_spacing = ui.spacing().extra_text_line_spacing;
                let before_selection = ui.visuals().selection;
                let before_opacity = ui.painter().opacity();

                for index in if self.reversed { [1, 0] } else { [0, 1] } {
                    let input = ui
                        .scope(|ui| {
                            ui.set_width(self.column_width);
                            if !self.enabled {
                                ui.disable();
                            }
                            let mut input = TextInput::new(
                                &self.theme,
                                &mut self.values[index],
                                format!("Value {index}"),
                            )
                            .id(editor_id(index))
                            .hint("A deliberately long placeholder that must never be ellipsized");
                            if self.error {
                                input = input.error("Caller error");
                            }
                            input.show(ui)
                        })
                        .inner;
                    if input.response.changed() {
                        self.changes[index] += 1;
                    }
                    inputs.push(input);
                }

                assert_eq!(ui.spacing().interact_size, before_spacing);
                assert_eq!(ui.spacing().extra_text_line_spacing, before_extra_spacing);
                assert_eq!(ui.visuals().selection, before_selection);
                assert_eq!(ui.painter().opacity(), before_opacity);
            },
        );
        self.frame_shapes = output.shapes.clone();
        self.tree = output.platform_output.accesskit_update.take();
        output.drop_without_applying_deltas();
        inputs
    }

    fn focus_first(&mut self) {
        self.frame(vec![]);
        self.context
            .memory_mut(|memory| memory.request_focus(editor_id(0)));
        self.frame(vec![]);
    }

    fn press(&mut self, k: egui::Key, command: bool) {
        self.frame(vec![key(k, command)]);
        self.frame(vec![egui::Event::Key {
            key: k,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers {
                command,
                ..Default::default()
            },
        }]);
    }
}

#[test]
fn light_and_dark_geometry_use_viewport_breakpoint_and_body_family() {
    for theme in [Theme::light(), Theme::dark()] {
        for (viewport_width, font_size, line_height) in [(767.0, 16.0, 24.0), (768.0, 14.0, 20.0)] {
            let mut h = Harness::new(theme.clone());
            h.viewport_width = viewport_width;
            h.column_width = 160.0;
            h.values[0] = "Text".into();
            h.context.all_styles_mut(|style| {
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Body)
                    .unwrap()
                    .family = egui::FontFamily::Monospace;
            });

            let output = h.frame(vec![]);
            let input = &output[0];
            let group = input.drawing.as_ref().unwrap();
            let text = text_shape(group, "Text");
            let format = &text.galley.job.sections[0].format;

            assert_eq!(input.response.rect.height(), 36.0);
            assert!(input.response.rect.width() <= 160.0);
            assert!((text.pos.x - input.response.rect.left() - 13.0).abs() < 0.01);
            assert!(
                (text.pos.y + text.galley.rect.center().y - input.response.rect.center().y).abs()
                    < 0.01
            );
            assert_eq!(format.font_id.size, font_size);
            assert_eq!(format.font_id.family, egui::FontFamily::Monospace);
            assert_eq!(format.line_height, Some(line_height));

            assert_eq!(border(group).corner_radius, theme.metrics.corner_radius);
            assert_eq!(border(group).stroke.color, theme.colors.input);
            assert_eq!(border(group).stroke.width, 1.0);

            let expected_fill = if theme == Theme::dark() {
                theme.colors.input.gamma_multiply(0.30)
            } else {
                egui::Color32::TRANSPARENT
            };
            assert_eq!(background(group, input.response.rect).fill, expected_fill);
            assert!(
                group
                    .bounds()
                    .contains_rect(input.response.rect.expand(4.0))
            );
        }
    }
}

#[test]
fn caller_radius_is_preserved() {
    let mut theme = Theme::light();
    assert_eq!(theme.metrics.corner_radius, egui::CornerRadius::same(8));
    theme.metrics.corner_radius = egui::CornerRadius::same(6);
    let mut h = Harness::new(theme);
    let output = h.frame(vec![]);
    assert_eq!(
        border(output[0].drawing.as_ref().unwrap()).corner_radius,
        egui::CornerRadius::same(6)
    );
}

#[test]
fn long_content_and_placeholder_do_not_impose_a_default_width_floor() {
    for width in [160.0, 40.0, 26.0, 20.0, 8.0, 0.0] {
        let mut h = Harness::new(Theme::light());
        h.column_width = width;
        h.values[0] = "long content ".repeat(100);
        let original = h.values[0].clone();

        let output = h.frame(vec![]);
        for input in &output {
            assert!(
                (input.response.rect.width() - width.max(26.0)).abs() <= 0.01,
                "requested {width}, actual {}",
                input.response.rect.width()
            );
            assert_eq!(input.response.rect.height(), 36.0);
        }
        assert_eq!(h.values[0], original);

        if width <= 26.0 {
            assert!(!shapes(output[1].drawing.as_ref().unwrap()).any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().starts_with("A deliberately"))
            }));
        } else {
            let group = output[1].drawing.as_ref().unwrap();
            let hint = text_shape(
                group,
                "A deliberately long placeholder that must never be ellipsized",
            );
            assert_eq!(
                hint.galley.job.text,
                "A deliberately long placeholder that must never be ellipsized"
            );
            assert!(!hint.galley.elided);
            let clipped_hint = shapes(group)
                .find(|shape| {
                    matches!(&shape.shape, egui::Shape::Text(text)
                    if text.galley.job.text.starts_with("A deliberately"))
                })
                .unwrap();
            assert!(clipped_hint.clip_rect.width() <= width - 26.0 + 0.01);
        }
    }
}

#[test]
fn capture_removes_native_text_but_not_label_or_error_sources() {
    let mut h = Harness::new(Theme::light());
    h.values[0] = "Native editor text".into();
    h.error = true;
    let output = h.frame(vec![]);

    assert!(shapes(output[0].drawing.as_ref().unwrap()).any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text)
            if text.galley.job.text == "Native editor text")
    }));
    assert!(!h.frame_shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text)
            if text.galley.job.text == "Native editor text")
    }));

    for text in ["Value 0", "Caller error"] {
        assert!(h.frame_shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(shape)
                if shape.galley.job.text == text)
        }));
        assert!(!shapes(output[0].drawing.as_ref().unwrap()).any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(shape)
                if shape.galley.job.text == text)
        }));
    }
}

#[test]
fn first_disabled_frame_is_safe_and_only_editor_group_is_half_opaque() {
    let mut h = Harness::new(Theme::light());
    h.focus_first();
    h.values[0] = "retained".into();
    h.error = true;
    h.enabled = false;
    let changes = h.changes;

    let output = h.frame(vec![
        egui::Event::Text("ignored".into()),
        egui::Event::Paste("ignored".into()),
        egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "ignored".into(),
            active_range_chars: Some(0..7),
        }),
    ]);

    assert_eq!(h.values, ["retained", ""]);
    assert_eq!(h.changes, changes);
    assert!(!output[0].response.changed());
    assert!(!output[0].response.enabled());
    assert_eq!(output[0].drawing.as_ref().unwrap().opacity(), 0.5);

    // The retained native text is not itself faded before compositing.
    let text = text_shape(output[0].drawing.as_ref().unwrap(), "retained");
    assert_eq!(
        text.galley.job.sections[0].format.color,
        h.theme.colors.background.foreground
    );

    // The editor's colors remain opaque until its whole group is composited.
    let text_primitives = h.context.tessellate(
        shapes(output[0].drawing.as_ref().unwrap())
            .filter(|shape| matches!(&shape.shape, egui::Shape::Text(_)))
            .cloned()
            .collect(),
        h.context.pixels_per_point(),
    );
    assert!(text_primitives.iter().any(|primitive| {
        matches!(&primitive.primitive, egui::epaint::Primitive::Mesh(mesh)
            if mesh.vertices.iter().any(|vertex| vertex.color == h.theme.colors.background.foreground))
    }));

    // Label/error styles stay outside the editor group; their painter keeps
    // the caller's normal disabled attenuation.
    for (expected_text, expected_color) in [
        ("Value 0", h.theme.colors.background.foreground),
        ("Caller error", h.theme.colors.destructive.background),
    ] {
        let shape = h
            .frame_shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == expected_text => Some(text),
                _ => None,
            })
            .unwrap();
        assert!(
            shape
                .galley
                .job
                .sections
                .iter()
                .any(|section| { section.format.color == expected_color })
        );
    }

    h.frame(vec![key(egui::Key::Backspace, false)]);
    assert_eq!(h.values[0], "retained");
}

#[test]
fn focus_ring_and_invalid_border_have_independent_transition_rules() {
    for theme in [Theme::light(), Theme::dark()] {
        let mut h = Harness::new(theme.clone());
        h.frame(vec![]);

        h.context
            .memory_mut(|memory| memory.request_focus(editor_id(0)));
        let start = h.frame_after(0.001, vec![]);
        let group = start[0].drawing.as_ref().unwrap();
        assert_eq!(border(group).stroke.color, theme.colors.ring);
        assert_eq!(ring(group).stroke.width, 0.0);

        let middle = h.frame_after(0.075, vec![]);
        let width = ring(middle[0].drawing.as_ref().unwrap()).stroke.width;
        assert!(width > 0.0 && width < 3.0);

        let settled = h.frame_after(0.2, vec![]);
        let group = settled[0].drawing.as_ref().unwrap();
        assert_eq!(ring(group).stroke.width, 3.0);
        assert_eq!(
            ring(group).stroke.color,
            theme.colors.ring.gamma_multiply(0.5)
        );

        h.error = true;
        let invalid_start = h.frame_after(0.001, vec![]);
        let group = invalid_start[0].drawing.as_ref().unwrap();
        assert_eq!(
            border(group).stroke.color,
            theme.colors.destructive.background
        );
        // The border changed immediately; the ring has not jumped to its target.
        assert_eq!(
            ring(group).stroke.color,
            theme.colors.ring.gamma_multiply(0.5)
        );

        let invalid_settled = h.frame_after(0.2, vec![]);
        let expected = theme
            .colors
            .destructive
            .background
            .gamma_multiply(if theme == Theme::dark() { 0.4 } else { 0.2 });
        assert_eq!(
            ring(invalid_settled[0].drawing.as_ref().unwrap())
                .stroke
                .color,
            expected
        );

        h.context
            .memory_mut(|memory| memory.surrender_focus(editor_id(0)));
        h.frame_after(0.001, vec![]);
        let unfocused = h.frame_after(0.2, vec![]);
        let group = unfocused[0].drawing.as_ref().unwrap();
        assert_eq!(
            border(group).stroke.color,
            theme.colors.destructive.background
        );
        assert_eq!(ring(group).stroke.width, 0.0);
        assert_eq!(ring(group).stroke.color, egui::Color32::TRANSPARENT);
    }
}

#[test]
fn pointer_focus_also_gets_the_focus_ring() {
    let mut h = Harness::new(Theme::light());
    let initial = h.frame(vec![]);
    let position = initial[0].response.rect.center();
    h.frame(vec![
        egui::Event::PointerMoved(position),
        egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    h.frame(vec![egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    let output = h.frame(vec![]);
    assert!(output[0].response.has_focus());
    assert_eq!(ring(output[0].drawing.as_ref().unwrap()).stroke.width, 3.0);
}

#[test]
fn selection_paste_undo_and_reordering_keep_native_state() {
    let mut h = Harness::new(Theme::light());
    h.focus_first();
    h.frame(vec![egui::Event::Text("original".into())]);
    h.frame(vec![]);
    h.press(egui::Key::A, true);

    let selected = h.frame(vec![]);
    let group = selected[0].drawing.as_ref().unwrap();
    let primitives = h.context.tessellate(
        shapes(group).cloned().collect(),
        h.context.pixels_per_point(),
    );
    for expected in [
        h.theme.colors.primary.background,
        h.theme.colors.primary.foreground,
    ] {
        assert!(primitives.iter().any(|primitive| {
            matches!(&primitive.primitive, egui::epaint::Primitive::Mesh(mesh)
                if mesh.vertices.iter().any(|vertex| vertex.color == expected))
        }));
    }

    h.frame(vec![egui::Event::Paste("replacement".into())]);
    assert_eq!(h.values, ["replacement", ""]);
    h.frame(vec![]);
    h.press(egui::Key::Z, true);
    assert_eq!(h.values, ["original", ""]);

    h.reversed = true;
    h.theme = Theme::dark();
    h.error = true;
    let moved = h.frame(vec![]);
    assert_eq!(moved[1].response.id, editor_id(0));
    assert!(moved[1].response.has_focus());

    h.press(egui::Key::End, true);
    h.frame(vec![egui::Event::Text("!".into())]);
    assert_eq!(h.values, ["original!", ""]);
}

#[test]
fn ime_preedit_and_commit_retain_native_visuals_and_value_semantics() {
    let mut h = Harness::new(Theme::light());
    h.focus_first();
    let preedit = h.frame(vec![egui::Event::Ime(egui::ImeEvent::Preedit {
        text: "ni".into(),
        active_range_chars: Some(0..2),
    })]);

    assert_eq!(h.values, ["ni", ""]);
    assert_eq!(
        text_shape(preedit[0].drawing.as_ref().unwrap(), "ni")
            .galley
            .job
            .text,
        "ni"
    );
    assert!(
        shapes(preedit[0].drawing.as_ref().unwrap())
            .any(|shape| { matches!(shape.shape, egui::Shape::LineSegment { .. }) }),
        "native IME underline must be captured"
    );

    h.frame(vec![egui::Event::Ime(egui::ImeEvent::Commit("你".into()))]);
    assert_eq!(h.values, ["你", ""]);
    let committed = h.frame(vec![]);
    assert_eq!(
        text_shape(committed[0].drawing.as_ref().unwrap(), "你")
            .galley
            .job
            .text,
        "你"
    );
}

#[test]
fn error_metadata_is_linked_and_cleared_without_editing() {
    let mut h = Harness::new(Theme::light());
    h.values[0] = "retained".into();
    h.error = true;
    h.frame(vec![]);
    {
        let node = h
            .tree
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(id, _)| *id == editor_id(0).accesskit_id())
            .unwrap();
        assert_eq!(node.1.description(), Some("Caller error"));
        assert_eq!(
            node.1.placeholder(),
            Some("A deliberately long placeholder that must never be ellipsized")
        );
        assert_eq!(node.1.invalid(), Some(egui::accesskit::Invalid::True));
        assert!(!node.1.labelled_by().is_empty());
    }

    h.error = false;
    h.frame(vec![]);
    let node = h
        .tree
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(id, _)| *id == editor_id(0).accesskit_id())
        .unwrap();
    assert_eq!(node.1.description(), None);
    assert_eq!(node.1.invalid(), None);
    assert_eq!(h.values[0], "retained");
    assert_eq!(h.changes, [0, 0]);
}

#[derive(Default)]
struct RecordingBackend {
    groups: Mutex<Vec<Group>>,
}

impl compositing::Backend for RecordingBackend {
    fn callback(
        &self,
        _context: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        let rect = group.bounds();
        self.groups.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}

#[test]
fn submission_is_same_context_same_layer_and_applies_opacity_once() {
    let context = egui::Context::default();
    let backend = Arc::new(RecordingBackend::default());
    let _installation = compositing::install(&context, backend.clone()).unwrap();
    let theme = Theme::light();
    let mut value = "value".to_owned();

    let output = context.run_ui(Default::default(), |ui| {
        let input = ui
            .scope(|ui| {
                ui.disable();
                TextInput::new(&theme, &mut value, "Value").show(ui)
            })
            .inner;

        let mut parent = ui.painter().clone();
        parent.set_opacity(0.8);
        input.paint(&parent).unwrap();
    });
    output.drop_without_applying_deltas();

    let groups = backend.groups.lock().unwrap();
    assert_eq!(groups.len(), 1);
    assert!((groups[0].opacity() - 0.4).abs() < 0.0001);
}

#[test]
fn ordered_batch_routes_focus_text_paste_and_composition_without_replay() {
    let context = egui::Context::default();
    context.options_mut(|options| options.ordered_input = true);
    let theme = Theme::light();
    let mut values = [String::new(), String::new()];
    let mut changes = [0; 2];
    context.memory_mut(|memory| memory.request_focus(editor_id(0)));
    let events = vec![
        egui::Event::Text("before".into()),
        key(egui::Key::Tab, false),
        egui::Event::Text("after".into()),
        egui::Event::Paste(" paste".into()),
        egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "ni".into(),
            active_range_chars: Some(0..2),
        }),
        egui::Event::Ime(egui::ImeEvent::Commit("你".into())),
    ];
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 600.0),
            )),
            events,
            time: Some(1.0),
            ..Default::default()
        },
        |root| {
            let local = |index: usize| {
                egui::Ui::new(
                    context.clone(),
                    editor_id(index).with("local"),
                    egui::UiBuilder::new().layer_id(root.layer_id()).max_rect(
                        egui::Rect::from_min_size(
                            egui::pos2(0.0, index as f32 * 100.0),
                            egui::vec2(320.0, 90.0),
                        ),
                    ),
                )
            };
            context.begin_input();
            for (index, value) in values.iter_mut().enumerate() {
                let output = TextInput::new(&theme, value, "Value")
                    .id(editor_id(index))
                    .show(&mut local(index));
                assert!(!output.response.changed());
            }
            while let Some(event) = context.next_input_event() {
                if let Some(index) = (0..2).find(|index| event.target == Some(editor_id(*index))) {
                    let output = TextInput::new(&theme, &mut values[index], "Value")
                        .id(editor_id(index))
                        .show(&mut local(index));
                    if output.response.changed() {
                        changes[index] += 1;
                    }
                }
            }
            for (index, value) in values.iter_mut().enumerate() {
                let output = TextInput::new(&theme, value, "Value")
                    .id(editor_id(index))
                    .show(&mut local(index));
                assert!(!output.response.changed());
                assert!(
                    shapes(output.drawing.as_ref().unwrap())
                        .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()==value))
                );
            }
        },
    );
    output.drop_without_applying_deltas();
    assert_eq!(values, ["before", "after paste你"]);
    assert_eq!(changes, [1, 4]);
}

#[test]
fn missing_backend_has_no_editor_fallback_and_parent_opacity_stays_local() {
    let context = egui::Context::default();
    let theme = Theme::light();
    let mut value = String::from("Editor");
    let mut output = context.run_ui(Default::default(), |ui| {
        ui.set_opacity(0.4);
        let painter = ui.painter().clone();
        let input = TextInput::new(&theme, &mut value, "Label")
            .error("Error")
            .show(ui);
        assert_eq!(
            text_shape(input.drawing.as_ref().unwrap(), "Editor").opacity_factor,
            1.0
        );
        assert!(matches!(
            input.paint(&painter),
            Err(compositing::Error::MissingHost)
        ));
        assert_eq!(ui.opacity(), 0.4);
    });
    // Release the frame's texture obligation before assertions can unwind.
    let frame_shapes = std::mem::take(&mut output.shapes);
    let pixels_per_point = output.pixels_per_point;
    output.drop_without_applying_deltas();
    for (text, color) in [
        ("Label", theme.colors.background.foreground),
        ("Error", theme.colors.destructive.background),
    ] {
        let shape = frame_shapes
            .iter()
            .find(|s| {
                matches!(&s.shape,
            egui::Shape::Text(t) if t.galley.text() == text)
            })
            .unwrap();
        let primitives = context.tessellate(vec![shape.clone()], pixels_per_point);
        let colors: Vec<_> = primitives
            .iter()
            .flat_map(|primitive| {
                let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive else {
                    panic!("text must tessellate to a mesh");
                };
                mesh.vertices.iter().map(|vertex| vertex.color)
            })
            .collect();
        assert!(!colors.is_empty());
        assert!(
            colors
                .iter()
                .all(|actual| *actual == color.gamma_multiply(0.4))
        );
    }
    assert!(
        !frame_shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Editor"))
    );
}
