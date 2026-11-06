//! Public button input, layout, identity, and accessibility metadata checks.

use egui_prism::{
    button::{Button, ButtonRole, ButtonSize, IconButton},
    egui,
    theme::Theme,
};

struct Harness {
    context: egui::Context,
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
}

impl Harness {
    fn new() -> Self {
        Self {
            context: egui::Context::default(),
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
        }
    }

    fn frame(&mut self, events: Vec<egui::Event>) -> [egui::Response; 2] {
        self.time += 1.0 / 60.0;
        let mut responses = None;
        let output = self.context.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(self.width, 800.0))),
            time: Some(self.time), events, ..Default::default()
        }, |ui| {
            let order = if self.reversed { [1,0] } else { [0,1] };
            responses = Some(order.map(|index| {
                let id = ui.make_persistent_id(index);
                ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
                    let response = if self.icon {
                        let image = egui::Image::new((egui::TextureId::User(1), egui::vec2(16.0,16.0)));
                        ui.add_enabled(self.enabled, IconButton::new(&self.theme, image, format!("Action {index}")).size(self.size))
                    } else {
                        let label = if self.long { "A deliberately long action label that must wrap inside the available width" } else { "Activate" };
                        ui.add_enabled(self.enabled, Button::new(&self.theme, label).size(self.size))
                    };
                    if response.clicked() { self.counts[index] += 1; }
                    response
                }).inner
            }));
        });
        self.names.extend(
            output
                .platform_output
                .events
                .iter()
                .filter_map(|event| event.widget_info().label.clone()),
        );
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
    assert!(r[0].rect.width() >= harness.theme.metrics.control_height);
    assert!(r[0].rect.height() >= harness.theme.metrics.control_height);
}

#[test]
fn size_changes_minimum_height_and_long_text_wraps() {
    let mut harness = Harness::new();
    let mut heights = Vec::new();
    for size in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
        harness.size = size;
        harness.frame(vec![]);
        heights.push(harness.frame(vec![])[0].rect.height());
    }
    assert!(heights[0] < heights[1] && heights[1] < heights[2]);
    harness.long = true;
    harness.width = 180.0;
    harness.frame(vec![]);
    let r = harness.frame(vec![]);
    assert!(r[0].rect.height() > heights[2]);
    assert!(r[0].rect.right() <= harness.width);
}

#[test]
fn button_roles_do_not_leak_style_into_the_host() {
    for theme in [Theme::light(), Theme::dark()] {
        egui::__run_test_ui(|ui| {
            let original = ui.style().clone();
            for role in [
                ButtonRole::Primary,
                ButtonRole::Secondary,
                ButtonRole::Outline,
                ButtonRole::Ghost,
                ButtonRole::Destructive,
            ] {
                ui.add(Button::new(&theme, "Action").role(role));
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
    let context = egui::Context::default();
    let theme = Theme::dark();
    for _ in 0..2 {
        let output = context.run_ui(egui::RawInput::default(), |ui| {
            let image = egui::Image::new((egui::TextureId::User(42), egui::vec2(16.0, 16.0)))
                .tint(egui::Color32::RED);
            ui.add(IconButton::new(&theme, image, "Red action"));
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
    let context = egui::Context::default();
    let theme = Theme::light();
    let output = context.run_ui(egui::RawInput::default(), |ui| {
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(24.0));
        ui.add(Button::new(&theme, "Host button font"));
        ui.add(Button::new(
            &theme,
            egui::RichText::new("Explicit font").size(30.0),
        ));
    });
    let mut sizes = Vec::new();
    for clipped in &output.shapes {
        text_sizes(&clipped.shape, &mut sizes);
    }
    output.drop_without_applying_deltas();
    assert_eq!(sizes, vec![24.0, 30.0]);
}
