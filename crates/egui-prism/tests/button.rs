//! Public button input, layout, identity, and accessibility metadata checks.

use egui_prism::{
    button::{Button, ButtonRole, ButtonSize, IconButton},
    egui,
    theme::Theme,
};

struct Harness {
    context: TestContext,
    theme: Theme,
    time: f64,
    enabled: bool,
    reversed: bool,
    icon: bool,
    long: bool,
    width: f32,
    size: ButtonSize,
    counts: [usize; 2],
    names: Vec<String>,
    shapes: Vec<egui::epaint::ClippedShape>,
    role: ButtonRole,
    invalid: bool,
}

impl Harness {
    fn new() -> Self {
        Self {
            context: TestContext::default(),
            theme: Theme::light(),
            time: 0.0,
            enabled: true,
            reversed: false,
            icon: false,
            long: false,
            width: 400.0,
            size: ButtonSize::Medium,
            counts: [0; 2],
            names: Vec::new(),
            shapes: Vec::new(),
            role: ButtonRole::Primary,
            invalid: false,
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let mut responses = None;
        let output = self.context.run_ui(
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
                        let response = if self.icon {
                            let image = egui::Image::new((
                                egui::TextureId::User(1),
                                egui::vec2(16.0, 16.0),
                            ));
                            show_enabled(ui, self.enabled, |ui| {
                                IconButton::new(&self.theme, image, format!("Action {index}"))
                                    .size(self.size)
                                    .show(ui)
                            })
                        } else {
                            let label = if self.long {
                                "A deliberately long action label that keeps its intrinsic width"
                            } else {
                                "Activate"
                            };
                            show_enabled(ui, self.enabled, |ui| {
                                Button::new(&self.theme, label)
                                    .size(self.size)
                                    .role(self.role)
                                    .invalid(self.invalid)
                                    .show(ui)
                            })
                        };
                        if response.clicked() {
                            self.counts[index] += 1;
                        }
                        response
                    })
                    .inner
                }));
            },
        );
        self.names.extend(
            output
                .platform_output
                .events
                .iter()
                .filter_map(|event| event.widget_info().label.clone()),
        );
        self.shapes = output.shapes.clone();
        output.drop_without_applying_deltas();
        responses.unwrap()
    }

    fn click(&mut self, position: egui::Pos2) {
        self.frame(vec![egui::Event::PointerMoved(position)]);
        for pressed in [true, false] {
            self.frame(vec![egui::Event::PointerButton {
                pos: position,
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
fn pointer_actions_are_independent_disabled_and_idle_safe() {
    for icon in [false, true] {
        let mut harness = Harness::new();
        harness.icon = icon;
        harness.frame(vec![]);
        let r = harness.frame(vec![]);
        harness.click(r[0].rect.center());
        assert_eq!(harness.counts, [1, 0]);
        for _ in 0..4 {
            harness.frame(vec![]);
        }
        assert_eq!(harness.counts, [1, 0]);
        harness.enabled = false;
        let r = harness.frame(vec![]);
        for response in r {
            harness.click(response.rect.center());
        }
        assert_eq!(harness.counts, [1, 0]);
    }
}

#[test]
fn keyboard_activation_and_disabled_focus_use_public_responses() {
    for icon in [false, true] {
        let mut harness = Harness::new();
        harness.icon = icon;
        harness.frame(vec![]);
        let r = harness.frame(vec![]);
        harness.key(egui::Key::Tab);
        assert_eq!(
            harness.context.memory(|memory| memory.focused()),
            Some(r[0].id)
        );
        harness.key(egui::Key::Enter);
        for _ in 0..4 {
            harness.frame(vec![]);
        }
        assert_eq!(harness.counts, [1, 0]);
        harness.key(egui::Key::Space);
        assert_eq!(harness.counts, [2, 0]);
        harness.enabled = false;
        harness.frame(vec![]);
        harness.key(egui::Key::Enter);
        harness.key(egui::Key::Space);
        assert_eq!(harness.counts, [2, 0]);
    }
}

#[test]
fn caller_scopes_keep_identity_across_reorder_theme_and_size() {
    let mut harness = Harness::new();
    harness.frame(vec![]);
    let before = harness.frame(vec![]);
    harness.reversed = true;
    harness.theme = Theme::dark();
    harness.size = ButtonSize::Large;
    let after = harness.frame(vec![]);
    assert_eq!(before[0].id, after[1].id);
    assert_eq!(before[1].id, after[0].id);
    harness.click(after[0].rect.center());
    assert_eq!(harness.counts, [0, 1]);
}

#[test]
fn icon_accessible_name_is_part_of_native_button_events() {
    let mut harness = Harness::new();
    harness.icon = true;
    harness.frame(vec![]);
    let r = harness.frame(vec![]);
    harness.click(r[0].rect.center());
    assert!(harness.names.iter().any(|name| name == "Action 0"));
    assert!(harness.names.iter().all(|name| !name.is_empty()));
    assert!(r[0].rect.width() >= harness.theme.metrics.control_height);
    assert!(r[0].rect.height() >= harness.theme.metrics.control_height);
}

#[test]
fn reference_sizes_are_fixed_and_long_text_does_not_wrap() {
    let mut harness = Harness::new();
    for (size, height) in [
        (ButtonSize::ExtraSmall, 24.0),
        (ButtonSize::Small, 32.0),
        (ButtonSize::Medium, 36.0),
        (ButtonSize::Large, 40.0),
    ] {
        harness.size = size;
        for icon in [false, true] {
            harness.icon = icon;
            harness.frame(vec![]);
            let response = harness.frame(vec![]);
            assert_eq!(response[0].rect.height(), height);
            if icon {
                assert_eq!(response[0].rect.width(), height);
            }
        }
    }
    harness.icon = false;
    harness.long = true;
    harness.width = 180.0;
    harness.frame(vec![]);
    let r = harness.frame(vec![]);
    assert_eq!(r[0].rect.height(), 40.0);
    assert!(r[0].rect.width() > harness.width);
}

#[test]
fn button_roles_do_not_leak_style_into_the_host() {
    for theme in [Theme::light(), Theme::dark()] {
        test_ui(|ui| {
            let original = ui.style().clone();
            for role in [
                ButtonRole::Primary,
                ButtonRole::Secondary,
                ButtonRole::Outline,
                ButtonRole::Ghost,
                ButtonRole::Destructive,
                ButtonRole::Link,
            ] {
                Button::new(&theme, "Action")
                    .role(role)
                    .show(ui)
                    .paint(ui.painter())
                    .expect("button compositor");
                assert_eq!(ui.style().as_ref(), original.as_ref());
            }
        });
    }
}

#[test]
#[should_panic(expected = "nonempty accessible name")]
fn icon_rejects_an_empty_accessible_name() {
    let theme = Theme::light();
    let image = egui::Image::new((egui::TextureId::User(1), egui::vec2(16.0, 16.0)));
    let _button = IconButton::new(&theme, image, "  ");
}

#[test]
fn explicitly_tinted_images_preserve_their_rendered_color() {
    let context = TestContext::default();
    let theme = Theme::dark();
    for _ in 0..2 {
        let output = context.run_ui(egui::RawInput::default(), |ui| {
            let image = egui::Image::new((egui::TextureId::User(42), egui::vec2(16.0, 16.0)))
                .tint(egui::Color32::RED);
            IconButton::new(&theme, image, "Red action")
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
        });
        let primitives = context.tessellate(output.shapes.clone(), output.pixels_per_point);
        let colors: Vec<_> = primitives
            .iter()
            .filter_map(|primitive| match &primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh)
                    if mesh.texture_id == egui::TextureId::User(42) =>
                {
                    Some(mesh.vertices.iter().map(|vertex| vertex.color))
                }
                _ => None,
            })
            .flatten()
            .collect();
        output.drop_without_applying_deltas();
        let colors: Vec<_> = colors.into_iter().filter(|color| color.a() != 0).collect();
        assert!(!colors.is_empty());
        assert!(
            colors.iter().all(|color| *color == egui::Color32::RED),
            "rendered colors: {colors:?}"
        );
    }
}

#[test]
fn labels_use_the_host_button_font_and_keep_explicit_formatting() {
    fn text_sizes(shape: &egui::Shape, sizes: &mut Vec<f32>) {
        match shape {
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    text_sizes(shape, sizes);
                }
            }
            egui::Shape::Text(text) => sizes.extend(
                text.galley
                    .job
                    .sections
                    .iter()
                    .map(|section| section.format.font_id.size),
            ),
            _ => {}
        }
    }
    for (override_style, expected) in [(None, 24.0), (Some(egui::TextStyle::Heading), 28.0)] {
        let context = TestContext::default();
        let theme = Theme::light();
        let output = context.run_ui(egui::RawInput::default(), |ui| {
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(24.0));
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Heading, egui::FontId::proportional(28.0));
            ui.style_mut().override_text_style = override_style.clone();
            Button::new(&theme, "Host button font")
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
            Button::new(&theme, egui::RichText::new("Explicit font").size(30.0))
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
        });
        let mut sizes = Vec::new();
        for clipped in &output.shapes {
            text_sizes(&clipped.shape, &mut sizes);
        }
        output.drop_without_applying_deltas();
        assert_eq!(sizes, vec![expected, 30.0]);
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
fn space_activates_on_release_and_cancels_on_focus_loss_or_disable() {
    for icon in [false, true] {
        let mut h = Harness::new();
        h.icon = icon;
        h.frame(vec![]);
        let r = h.frame(vec![]);
        h.key(egui::Key::Tab);
        h.frame(vec![key_event(egui::Key::Space, true, false)]);
        h.frame(vec![key_event(egui::Key::Space, true, true)]);
        assert_eq!(h.counts, [0, 0]);
        h.frame(vec![key_event(egui::Key::Space, false, false)]);
        assert_eq!(h.counts, [1, 0]);
        h.frame(vec![key_event(egui::Key::Enter, true, false)]);
        assert_eq!(h.counts, [2, 0]);
        h.frame(vec![key_event(egui::Key::Enter, false, false)]);
        h.key(egui::Key::ArrowRight);
        h.key(egui::Key::Escape);
        assert_eq!(h.context.memory(|m| m.focused()), Some(r[0].id));
        h.frame(vec![key_event(egui::Key::Space, true, false)]);
        h.key(egui::Key::Tab);
        h.frame(vec![key_event(egui::Key::Space, false, false)]);
        assert_eq!(h.counts, [2, 0]);
        h.frame(vec![key_event(egui::Key::Space, true, false)]);
        h.enabled = false;
        h.frame(vec![key_event(egui::Key::Space, false, false)]);
        h.enabled = true;
        h.frame(vec![key_event(egui::Key::Space, false, false)]);
        assert_eq!(h.counts, [2, 0]);
    }
}

fn rect_shapes(shapes: &[egui::epaint::ClippedShape]) -> Vec<&egui::epaint::RectShape> {
    fn collect<'a>(s: &'a egui::Shape, out: &mut Vec<&'a egui::epaint::RectShape>) {
        match s {
            egui::Shape::Rect(rect) => out.push(rect),
            egui::Shape::Vec(children) => children.iter().for_each(|s| collect(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for s in shapes {
        collect(&s.shape, &mut out);
    }
    out
}

#[test]
fn pointer_and_keyboard_focus_have_distinct_reference_paint() {
    for dark in [false, true] {
        let mut h = Harness::new();
        if dark {
            h.theme = Theme::dark();
        }
        h.frame(vec![]);
        let r = h.frame(vec![]);
        h.click(r[0].rect.center());
        h.time += 1.0;
        h.frame(vec![]);
        assert!(
            !rect_shapes(&h.shapes)
                .iter()
                .any(|s| s.stroke_kind == egui::StrokeKind::Outside && s.stroke.width > 0.0)
        );
        h.key(egui::Key::Tab);
        h.time += 1.0;
        h.frame(vec![]);
        let rings: Vec<_> = rect_shapes(&h.shapes)
            .into_iter()
            .filter(|s| s.stroke_kind == egui::StrokeKind::Outside && s.stroke.width > 0.0)
            .collect();
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].rect, r[1].rect);
        assert_eq!(rings[0].stroke.width, 3.0);
        assert_eq!(
            rings[0].stroke.color,
            h.theme.colors.ring.gamma_multiply(0.5)
        );
        h.invalid = true;
        h.frame(vec![]);
        h.time += 1.0;
        h.frame(vec![]);
        let ring = rect_shapes(&h.shapes)
            .into_iter()
            .find(|s| s.stroke_kind == egui::StrokeKind::Outside && s.stroke.width > 0.0)
            .unwrap();
        assert_eq!(
            ring.stroke.color,
            h.theme
                .colors
                .destructive
                .background
                .gamma_multiply(if dark { 0.4 } else { 0.2 })
        );
    }
}

#[test]
fn disabled_buttons_use_half_opacity_and_do_not_change_host_opacity() {
    for dark in [false, true] {
        let mut h = Harness::new();
        if dark {
            h.theme = Theme::dark();
        }
        h.enabled = false;
        h.frame(vec![]);
        let r = h.frame(vec![]);
        assert_eq!(h.context.groups.lock().unwrap()[0].opacity(), 0.5);
        let shape = rect_shapes(&h.shapes)
            .into_iter()
            .find(|s| s.rect == r[0].rect)
            .unwrap();
        assert_eq!(shape.fill, h.theme.colors.primary.background);
    }
    test_ui(|ui| {
        ui.set_opacity(0.7);
        let before = ui.opacity();
        Button::new(&Theme::light(), "Action")
            .show(ui)
            .paint(ui.painter())
            .expect("button compositor");
        assert_eq!(ui.opacity(), before);
    });
}

#[test]
fn reference_padding_gap_and_icon_geometry_do_not_follow_source_texture_size() {
    let context = TestContext::default();
    let theme = Theme::light();
    egui_prism::theme::install_fonts(&context, egui::FontDefinitions::default());
    context.all_styles_mut(|style| theme.apply(style));
    let mut observed = Vec::new();
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        for (size, padding, icon_padding, gap, icon_size) in [
            (ButtonSize::ExtraSmall, 8.0, 6.0, 4.0, 12.0),
            (ButtonSize::Small, 12.0, 10.0, 6.0, 16.0),
            (ButtonSize::Medium, 16.0, 12.0, 8.0, 16.0),
            (ButtonSize::Large, 24.0, 16.0, 8.0, 16.0),
        ] {
            let text = Button::new(&theme, "Action")
                .size(size)
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
            let outline = Button::new(&theme, "Action")
                .size(size)
                .role(ButtonRole::Outline)
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
            let image = (egui::TextureId::User(9), egui::vec2(128.0, 64.0));
            let icon = Button::new(&theme, "Action")
                .size(size)
                .icon(image)
                .show(ui)
                .paint(ui.painter())
                .expect("button compositor");
            observed.push((
                outline.rect.width() - text.rect.width(),
                icon.rect.width() - text.rect.width(),
                icon_size + gap + 2.0 * (icon_padding - padding),
            ));
        }
    });
    let image_bounds: Vec<_> = rect_shapes(&output.shapes)
        .into_iter()
        .filter(|shape| shape.fill_texture_id() == egui::TextureId::User(9))
        .map(|shape| shape.rect.size())
        .collect();
    output.drop_without_applying_deltas();
    for (border, difference, expected) in observed {
        assert!((border - 2.0).abs() < 0.01);
        assert!((difference - expected).abs() < 0.01);
    }
    assert_eq!(
        image_bounds,
        vec![
            egui::Vec2::splat(12.0),
            egui::Vec2::splat(16.0),
            egui::Vec2::splat(16.0),
            egui::Vec2::splat(16.0)
        ]
    );
}

#[test]
fn justified_layout_does_not_expand_the_button_hit_target() {
    let context = TestContext::default();
    let theme = Theme::light();
    let mut clicked = false;
    let mut button_rect = egui::Rect::NOTHING;
    let mut painted_rect = egui::Rect::NOTHING;
    for events in [
        vec![],
        vec![],
        vec![egui::Event::PointerMoved(egui::pos2(380.0, 18.0))],
        vec![egui::Event::PointerButton {
            pos: egui::pos2(380.0, 18.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        vec![egui::Event::PointerButton {
            pos: egui::pos2(380.0, 18.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    ] {
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 200.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                ui.vertical_centered_justified(|ui| {
                    let response = Button::new(&theme, "Action")
                        .show(ui)
                        .paint(ui.painter())
                        .expect("button compositor");
                    button_rect = response.rect;
                    clicked |= response.clicked();
                });
            },
        );
        painted_rect = rect_shapes(&output.shapes).first().unwrap().rect;
        output.drop_without_applying_deltas();
    }
    assert_eq!(button_rect, painted_rect);
    assert!(
        !clicked,
        "empty justified space must not activate the button"
    );
}

#[test]
fn restored_instance_cancels_an_old_space_press() {
    let mut h = Harness::new();
    h.frame(vec![]);
    let r = h.frame(vec![]);
    h.key(egui::Key::Tab);
    h.frame(vec![key_event(egui::Key::Space, true, false)]);
    h.context
        .run_ui(egui::RawInput::default(), |_| {})
        .drop_without_applying_deltas();
    h.context.memory_mut(|memory| memory.request_focus(r[0].id));
    h.frame(vec![key_event(egui::Key::Space, false, false)]);
    assert_eq!(h.counts, [0, 0]);
}

#[test]
fn inherited_text_color_does_not_replace_semantics_or_explicit_rich_text() {
    let context = TestContext::default();
    let theme = Theme::light();
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        ui.visuals_mut().override_text_color = Some(egui::Color32::BLACK);
        Button::new(&theme, "Semantic")
            .show(ui)
            .paint(ui.painter())
            .expect("button compositor");
        Button::new(
            &theme,
            egui::RichText::new("Explicit").color(egui::Color32::RED),
        )
        .show(ui)
        .paint(ui.painter())
        .expect("button compositor");
        assert_eq!(ui.visuals().override_text_color, Some(egui::Color32::BLACK));
    });
    let colors: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some((t.galley.job.sections[0].format.color, t.fallback_color)),
            _ => None,
        })
        .collect();
    output.drop_without_applying_deltas();
    assert_eq!(
        colors[0],
        (egui::Color32::PLACEHOLDER, theme.colors.primary.foreground)
    );
    assert_eq!(colors[1].0, egui::Color32::RED);
}

#[test]
fn tab_traversal_survives_a_later_filtered_key_in_the_same_frame() {
    for key in [egui::Key::ArrowDown, egui::Key::Escape] {
        let mut h = Harness::new();
        h.frame(vec![]);
        let r = h.frame(vec![]);
        h.key(egui::Key::Tab);
        h.frame(vec![
            key_event(egui::Key::Tab, true, false),
            key_event(key, true, false),
        ]);
        assert_eq!(h.context.memory(|memory| memory.focused()), Some(r[1].id));
    }
}

#[test]
fn caller_can_release_button_focus_while_handling_escape() {
    let context = TestContext::default();
    let theme = Theme::light();
    let mut id = None;
    for events in [
        vec![],
        vec![key_event(egui::Key::Tab, true, false)],
        vec![key_event(egui::Key::Tab, false, false)],
        vec![key_event(egui::Key::Escape, true, false)],
    ] {
        context
            .run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                        ui.memory_mut(|memory| memory.surrender_focus(id.unwrap()));
                    }
                    id = Some(
                        Button::new(&theme, "Action")
                            .show(ui)
                            .paint(ui.painter())
                            .expect("button compositor")
                            .id,
                    );
                },
            )
            .drop_without_applying_deltas();
    }
    assert_eq!(context.memory(|memory| memory.focused()), None);
}

fn show_enabled(
    ui: &mut egui::Ui,
    enabled: bool,
    show: impl FnOnce(&mut egui::Ui) -> egui_prism::button::ButtonOutput,
) -> egui::Response {
    let painter = ui.painter().clone();
    ui.add_enabled_ui(enabled, show)
        .inner
        .paint(&painter)
        .expect("button compositor")
}

// Inspect the actual unfaded Group contents for layout/input contracts; GPU tests
// separately verify final opacity and overlapping pixels.
struct TestContext {
    context: egui::Context,
    groups: std::sync::Arc<std::sync::Mutex<Vec<egui_prism::compositing::Group>>>,
    _installation: egui_prism::compositing::Installation,
}
impl Default for TestContext {
    fn default() -> Self {
        let context = egui::Context::default();
        let groups = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let installation = egui_prism::compositing::install(
            &context,
            std::sync::Arc::new(GroupRecorder(groups.clone())),
        )
        .unwrap();
        Self {
            context,
            groups,
            _installation: installation,
        }
    }
}
impl std::ops::Deref for TestContext {
    type Target = egui::Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
struct GroupRecorder(std::sync::Arc<std::sync::Mutex<Vec<egui_prism::compositing::Group>>>);
impl egui_prism::compositing::Backend for GroupRecorder {
    fn callback(
        &self,
        _: &egui::Context,
        group: egui_prism::compositing::Group,
    ) -> Result<egui::PaintCallback, egui_prism::compositing::Error> {
        self.0.lock().unwrap().push(group.clone());
        Ok(egui::PaintCallback {
            rect: group.bounds(),
            callback: std::sync::Arc::new(group),
        })
    }
}
impl TestContext {
    fn run_ui(&self, input: egui::RawInput, ui: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
        self.groups.lock().unwrap().clear();
        let mut output = self.context.run_ui(input, ui);
        output.shapes = output
            .shapes
            .into_iter()
            .flat_map(|shape| {
                if let egui::Shape::Callback(callback) = &shape.shape {
                    let group = callback
                        .callback
                        .downcast_ref::<egui_prism::compositing::Group>()
                        .unwrap();
                    group
                        .items()
                        .iter()
                        .map(|item| match item {
                            egui_prism::compositing::Item::Shape(shape) => shape.clone(),
                            egui_prism::compositing::Item::Group(_) => {
                                panic!("button output contains only primitives")
                            }
                        })
                        .collect()
                } else {
                    vec![shape]
                }
            })
            .collect();
        output
    }
}
fn test_ui(ui: impl FnMut(&mut egui::Ui)) {
    TestContext::default()
        .run_ui(egui::RawInput::default(), ui)
        .drop_without_applying_deltas();
}

#[test]
fn disabled_scopes_keep_parent_opacity_and_child_clipping() {
    for parent in [0.0, 0.4, 1.0] {
        for disabled_alpha in [0.0, 0.3, 1.0] {
            let context = TestContext::default();
            context
                .run_ui(egui::RawInput::default(), |ui| {
                    ui.set_opacity(parent);
                    let painter = ui.painter().clone();
                    ui.visuals_mut().disabled_alpha = disabled_alpha;
                    ui.disable();
                    ui.disable();
                    // Changing style after disable must not change the parent opacity.
                    ui.visuals_mut().disabled_alpha = 0.7;
                    let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(30.0, 25.0));
                    ui.set_clip_rect(clip);
                    let output = Button::new(&Theme::light(), "Clipped").show(ui);
                    let group = output.drawing.as_ref().unwrap();
                    assert_eq!(group.opacity(), 0.5);
                    for item in group.items() {
                        let egui_prism::compositing::Item::Shape(shape) = item else {
                            panic!()
                        };
                        assert_eq!(shape.clip_rect, clip);
                    }
                    output.paint(&painter).unwrap();
                })
                .drop_without_applying_deltas();
            let groups = context.groups.lock().unwrap();
            assert!((groups[0].opacity() - parent * 0.5).abs() < 0.0001);
        }
    }
}

#[test]
fn hidden_buttons_cannot_be_restored_by_an_external_painter() {
    let context = TestContext::default();
    context
        .run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter().clone();
            ui.set_invisible();
            let output = Button::new(&Theme::light(), "Hidden").show(ui);
            assert!(output.drawing.is_none());
            output.paint(&painter).unwrap();
        })
        .drop_without_applying_deltas();
    assert!(context.groups.lock().unwrap().is_empty());
}

#[test]
fn visible_buttons_require_a_live_compositor() {
    egui::__run_test_ui(|ui| {
        let output = Button::new(&Theme::light(), "No host").show(ui);
        assert!(output.paint(ui.painter()).is_err());
    });
}
