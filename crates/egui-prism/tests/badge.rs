//! Public contract coverage: layout, semantic colors, accessibility and submission.

use std::sync::{Arc, Mutex};

use egui_prism::{
    badge::{Badge, BadgeOutput, BadgeVariant},
    compositing::{self, Backend, Error, Group, Item},
    egui,
    theme::{self, Theme},
};

fn context() -> egui::Context {
    let context = egui::Context::default();
    theme::install_fonts(&context, egui::FontDefinitions::default());
    context
}

fn background(group: &Group) -> &egui::epaint::RectShape {
    group
        .items()
        .iter()
        .find_map(|item| match item {
            Item::Shape(shape) => match &shape.shape {
                egui::Shape::Rect(rect) => Some(rect),
                _ => None,
            },
            _ => None,
        })
        .expect("badge background")
}

fn text(output: &BadgeOutput) -> &egui::epaint::TextShape {
    output
        .drawing
        .as_ref()
        .unwrap()
        .items()
        .iter()
        .find_map(|item| match item {
            Item::Shape(shape) => match &shape.shape {
                egui::Shape::Text(text) => Some(text),
                _ => None,
            },
            _ => None,
        })
        .expect("nonempty badge text")
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}

#[test]
fn all_six_variants_have_pinned_geometry_and_light_dark_colors() {
    use BadgeVariant::*;
    use egui::Color32 as C;

    for (theme, dark) in [(Theme::light(), false), (Theme::dark(), true)] {
        let (primary, primary_text, secondary, secondary_text, foreground, border, destructive) =
            if dark {
                (
                    C::from_gray(229),
                    C::from_gray(23),
                    C::from_gray(38),
                    C::from_gray(250),
                    C::from_gray(250),
                    C::from_white_alpha(26),
                    C::from_rgba_unmultiplied(255, 100, 103, 153),
                )
            } else {
                (
                    C::BLACK,
                    C::from_gray(250),
                    C::from_gray(245),
                    C::from_gray(23),
                    C::BLACK,
                    C::from_gray(229),
                    C::from_rgb(231, 0, 11),
                )
            };
        let cases = [
            (Primary, primary, primary_text, C::TRANSPARENT),
            (Secondary, secondary, secondary_text, C::TRANSPARENT),
            (Destructive, destructive, C::WHITE, C::TRANSPARENT),
            (Outline, C::TRANSPARENT, foreground, border),
            (Ghost, C::TRANSPARENT, foreground, C::TRANSPARENT),
            (Link, C::TRANSPARENT, primary, C::TRANSPARENT),
        ];

        context()
            .run_ui(Default::default(), |ui| {
                theme.apply(ui.style_mut());
                for (variant, fill, foreground, border) in cases {
                    let output = Badge::new(&theme, "Status").variant(variant).show(ui);
                    let group = output.drawing.as_ref().unwrap();
                    let rect = background(group);
                    let text = text(&output);
                    let format = &text.galley.job.sections[0].format;

                    close(output.response.rect.height(), 22.0);
                    close(rect.rect.height(), 22.0);
                    close(rect.rect.width(), text.galley.size().x + 18.0);
                    close(text.pos.x - rect.rect.left(), 9.0);
                    close(text.pos.y - rect.rect.top(), 3.0);
                    close(text.galley.size().y, 16.0);
                    assert_eq!(text.galley.rows.len(), 1);
                    assert_eq!(format.font_id.size, 12.0);
                    assert_eq!(
                        format.font_id.family,
                        egui::FontFamily::Name("prism-geist-medium".into())
                    );
                    assert_eq!(format.line_height, Some(16.0));
                    assert_eq!(format.color, foreground);
                    assert_eq!(format.underline, egui::Stroke::NONE);
                    assert_eq!(rect.fill, fill);
                    assert_eq!(rect.stroke, egui::Stroke::new(1.0, border));
                    assert_eq!(rect.stroke_kind, egui::StrokeKind::Inside);
                    assert_eq!(rect.corner_radius, egui::CornerRadius::same(11));
                    assert_eq!(group.opacity(), 1.0);
                }
            })
            .drop_without_applying_deltas();
    }
}

#[test]
fn default_uses_theme_colors_and_the_host_button_family_but_not_control_metrics() {
    let mut theme = Theme::light();
    theme.typography.body = 40.0;
    theme.metrics.control_height = 80.0;
    theme.metrics.control_padding = egui::Vec2::splat(30.0);
    theme.colors.primary.background = egui::Color32::from_rgb(10, 80, 120);
    theme.colors.primary.foreground = egui::Color32::YELLOW;
    theme.colors.destructive.foreground = egui::Color32::GREEN;

    context()
        .run_ui(Default::default(), |ui| {
            theme.apply(ui.style_mut());
            ui.style_mut()
                .text_styles
                .get_mut(&egui::TextStyle::Button)
                .unwrap()
                .family = egui::FontFamily::Monospace;

            assert_eq!(BadgeVariant::default(), BadgeVariant::Primary);
            let output = Badge::new(&theme, "Default").show(ui);
            assert_eq!(
                background(output.drawing.as_ref().unwrap()).fill,
                theme.colors.primary.background
            );
            let format = &text(&output).galley.job.sections[0].format;
            assert_eq!(format.font_id, egui::FontId::monospace(12.0));
            assert_eq!(format.color, theme.colors.primary.foreground);
            close(output.response.rect.height(), 22.0);

            let link = Badge::new(&theme, "Link")
                .variant(BadgeVariant::Link)
                .show(ui);
            assert_eq!(
                text(&link).galley.job.sections[0].format.color,
                theme.colors.primary.background
            );

            let destructive = Badge::new(&theme, "Destructive")
                .variant(BadgeVariant::Destructive)
                .show(ui);
            assert_eq!(
                text(&destructive).galley.job.sections[0].format.color,
                egui::Color32::WHITE
            );
        })
        .drop_without_applying_deltas();
}

#[test]
fn empty_text_and_unbounded_parent_clips_are_valid() {
    context()
        .run_ui(Default::default(), |ui| {
            ui.set_clip_rect(egui::Rect::EVERYTHING);
            let output = Badge::new(&Theme::light(), "").show(ui);
            close(output.response.rect.width(), 18.0);
            close(output.response.rect.height(), 6.0);
            let group = output.drawing.as_ref().unwrap();
            assert!(group.bounds().is_finite());
            assert!(group.bounds().is_positive());
            for item in group.items() {
                let Item::Shape(shape) = item else {
                    panic!("expected a primitive");
                };
                assert!(shape.clip_rect.is_finite());
            }

            assert!(matches!(
                output.paint(ui.painter()),
                Err(Error::MissingHost)
            ));
        })
        .drop_without_applying_deltas();
}

#[test]
fn long_text_keeps_natural_width_full_accessible_name_and_parent_clip() {
    let context = context();
    context.enable_accesskit();
    let label = format!("{}END", "Caller-owned status ".repeat(256));

    let output = context.run_ui(Default::default(), |ui| {
        ui.set_max_width(96.0);
        let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(96.0, 22.0));
        ui.set_clip_rect(clip);
        let output = Badge::new(&Theme::light(), &label).show(ui);

        assert!(output.response.rect.width() > clip.width());
        close(output.response.rect.height(), 22.0);
        let text = text(&output);
        assert_eq!(text.galley.job.text, label);
        assert_eq!(text.galley.rows.len(), 1);
        close(output.response.rect.width(), text.galley.size().x + 18.0);

        let group = output.drawing.as_ref().unwrap();
        assert!(clip.contains_rect(group.bounds()));
        for item in group.items() {
            let Item::Shape(shape) = item else {
                panic!("expected a primitive");
            };
            assert!(clip.contains_rect(shape.clip_rect));
        }
    });

    let update = output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("accessibility update");
    let node = update
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == egui::Role::Label && node.value() == Some(label.as_str()))
        .expect("complete accessible label, despite visual clipping");
    assert!(!node.supports_action(egui::accesskit::Action::Focus));
    assert!(!node.supports_action(egui::accesskit::Action::Click));
    output.drop_without_applying_deltas();
}

#[test]
fn link_variant_is_not_a_pointer_or_keyboard_action() {
    let context = context();
    let mut center = egui::Pos2::ZERO;

    for step in 0..5 {
        let events = match step {
            0 => vec![],
            1 | 2 => vec![
                egui::Event::PointerMoved(center),
                egui::Event::PointerButton {
                    pos: center,
                    button: egui::PointerButton::Primary,
                    pressed: step == 1,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            _ => vec![egui::Event::Key {
                key: if step == 3 {
                    egui::Key::Tab
                } else {
                    egui::Key::Enter
                },
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        };
        context
            .run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let output = Badge::new(&Theme::light(), "Not a link action")
                        .variant(BadgeVariant::Link)
                        .show(ui);
                    center = output.response.rect.center();
                    assert_eq!(output.response.sense, egui::Sense::hover());
                    assert!(!output.response.clicked());
                    assert!(!output.response.changed());
                    assert!(!output.response.has_focus());
                    assert!(!output.response.dragged());
                },
            )
            .drop_without_applying_deltas();
    }
}

#[derive(Default)]
struct RecordingBackend(Mutex<Vec<Group>>);

impl Backend for RecordingBackend {
    fn callback(&self, _: &egui::Context, group: Group) -> Result<egui::PaintCallback, Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}

#[test]
fn disabled_scope_does_not_fade_drawing_and_parent_opacity_is_submitted_once() {
    let context = context();
    let backend = Arc::new(RecordingBackend::default());
    let _installation = compositing::install(&context, backend.clone()).unwrap();
    let fill = egui::Color32::from_rgba_unmultiplied(255, 100, 103, 153);

    context
        .run_ui(Default::default(), |ui| {
            ui.multiply_opacity(0.5);
            let parent_painter = ui.painter().clone();

            ui.add_enabled_ui(false, |ui| {
                let output = Badge::new(&Theme::dark(), "Static")
                    .variant(BadgeVariant::Destructive)
                    .show(ui);
                assert!(!output.response.enabled());
                let group = output.drawing.as_ref().unwrap();
                assert_eq!(group.opacity(), 1.0);
                assert_eq!(background(group).fill, fill);
                assert_eq!(
                    text(&output).galley.job.sections[0].format.color,
                    egui::Color32::WHITE
                );
                assert!(backend.0.lock().unwrap().is_empty());
                output.paint(&parent_painter).unwrap();
            });
        })
        .drop_without_applying_deltas();

    let submitted = backend.0.lock().unwrap();
    assert_eq!(submitted.len(), 1);
    assert_eq!(submitted[0].opacity(), 0.5);
    assert_eq!(background(&submitted[0]).fill, fill);
}

#[test]
fn css_whitespace_collapses_without_losing_nonbreaking_spaces() {
    context()
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            let output = Badge::new(&theme, "  Ready\n\t for\r  review\u{00a0}now  ").show(ui);
            assert_eq!(text(&output).galley.job.text, "Ready for review\u{00a0}now");
            close(output.response.rect.height(), 22.0);
            let empty = Badge::new(&theme, " \n\t ").show(ui);
            assert_eq!(empty.response.rect.size(), egui::vec2(18.0, 6.0));
        })
        .drop_without_applying_deltas();
}
