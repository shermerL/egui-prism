//! Public geometry, semantic, clipping, and explicit-submission contracts.

use std::sync::{Arc, Mutex};

use egui_prism::{
    compositing::{self, Backend, Error, Group, Item},
    egui,
    separator::{Orientation, Separator},
    theme::Theme,
};

fn shape(group: &Group) -> &egui::epaint::PathShape {
    let [Item::Shape(clipped)] = group.items() else {
        panic!("one separator shape");
    };
    let egui::Shape::Path(shape) = &clipped.shape else {
        panic!("filled separator rectangle");
    };
    shape
}

fn shape_rect(group: &Group) -> egui::Rect {
    egui::Rect::from_points(&shape(group).points)
}

#[test]
fn parent_extent_one_point_thickness_theme_and_fractional_scale() {
    for theme in [Theme::light(), Theme::dark()] {
        for scale in [1.0, 1.5, 2.0] {
            let context = egui::Context::default();
            context.set_pixels_per_point(scale);
            context
                .run_ui(Default::default(), |ui| {
                    ui.set_max_width(320.0);
                    let output = Separator::new(&theme).show(ui);
                    assert_eq!(output.response.rect.size(), egui::vec2(320.0, 1.0));
                    let group = output.drawing.as_ref().unwrap();
                    assert_eq!(shape_rect(group), output.response.rect);
                    assert_eq!(shape(group).fill, theme.colors.border);
                    assert!(shape(group).stroke.is_empty());
                    assert_eq!(group.opacity(), 1.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(240.0, 48.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let output = Separator::new(&theme)
                                .orientation(Orientation::Vertical)
                                .show(ui);
                            assert_eq!(output.response.rect.size(), egui::vec2(1.0, 48.0));
                            let group = output.drawing.as_ref().unwrap();
                            assert_eq!(shape_rect(group), output.response.rect);
                            assert_eq!(shape(group).fill, theme.colors.border);
                        },
                    );
                })
                .drop_without_applying_deltas();
        }
    }
}

#[test]
fn semantics_use_real_accesskit_role_orientation_bounds_and_no_actions() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut ids = Vec::new();
    let output = context.run_ui(Default::default(), |ui| {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            for decorative in [true, false] {
                ui.allocate_ui(egui::vec2(120.0, 48.0), |ui| {
                    let output = Separator::new(&Theme::light())
                        .orientation(orientation)
                        .decorative(decorative)
                        .show(ui);
                    ids.push((
                        output.response.id,
                        output.response.rect,
                        decorative,
                        orientation,
                    ));
                });
            }
        }
    });
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    for (id, rect, decorative, orientation) in ids {
        let node = tree
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == id.accesskit_id());
        if decorative {
            assert!(
                node.is_none(),
                "decorative separators do not create semantic nodes"
            );
        } else {
            let (_, node) = node.expect("semantic separator node");
            assert_eq!(node.role(), egui::Role::Splitter);
            assert_eq!(
                node.orientation(),
                Some(match orientation {
                    Orientation::Horizontal => egui::accesskit::Orientation::Horizontal,
                    Orientation::Vertical => egui::accesskit::Orientation::Vertical,
                }),
            );
            let bounds = node.bounds().unwrap();
            assert_eq!(bounds.width(), f64::from(rect.width()));
            assert_eq!(bounds.height(), f64::from(rect.height()));
            assert!(!node.supports_action(egui::accesskit::Action::Focus));
            assert!(!node.supports_action(egui::accesskit::Action::Click));
            assert!(!node.supports_action(egui::accesskit::Action::SetValue));
        }
    }
    output.drop_without_applying_deltas();
}

#[test]
fn clip_preserves_layout_and_zero_or_offscreen_extents_do_not_draw() {
    egui::Context::default()
        .run_ui(Default::default(), |ui| {
            ui.set_max_width(320.0);
            let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(160.0, 1.0));
            ui.set_clip_rect(clip);
            let output = Separator::new(&Theme::light()).show(ui);
            assert_eq!(output.response.rect.size(), egui::vec2(320.0, 1.0));
            let group = output.drawing.as_ref().unwrap();
            assert_eq!(shape_rect(group).width(), 320.0);
            assert_eq!(group.bounds(), clip);
            let [Item::Shape(clipped)] = group.items() else {
                unreachable!()
            };
            assert_eq!(clipped.clip_rect, clip);
            assert!(Separator::new(&Theme::light()).show(ui).drawing.is_none());
            ui.set_clip_rect(egui::Rect::EVERYTHING);
            let mut empty = ui.new_child(egui::UiBuilder::new().max_rect(
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(0.0, 8.0)),
            ));
            let output = Separator::new(&Theme::light()).show(&mut empty);
            assert_eq!(output.response.rect.width(), 0.0);
            assert!(output.drawing.is_none());
        })
        .drop_without_applying_deltas();
}

#[test]
fn pointer_and_keyboard_events_do_not_activate_or_focus_either_semantics() {
    for decorative in [true, false] {
        let context = egui::Context::default();
        let mut center = egui::Pos2::ZERO;
        for step in 0..6 {
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
                    key: match step {
                        3 => egui::Key::Tab,
                        4 => egui::Key::Enter,
                        _ => egui::Key::Space,
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
                        ui.set_max_width(320.0);
                        let output = Separator::new(&Theme::light())
                            .decorative(decorative)
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
fn drawing_is_explicit_and_parent_opacity_is_applied_once_even_in_disabled_scope() {
    let context = egui::Context::default();
    let backend = Arc::new(RecordingBackend::default());
    let _installation = compositing::install(&context, backend.clone()).unwrap();
    context
        .run_ui(Default::default(), |ui| {
            ui.multiply_opacity(0.6);
            let parent_painter = ui.painter().clone();
            ui.add_enabled_ui(false, |ui| {
                ui.set_max_width(320.0);
                let output = Separator::new(&Theme::dark()).show(ui);
                assert!(backend.0.lock().unwrap().is_empty());
                assert_eq!(
                    shape(output.drawing.as_ref().unwrap()).fill,
                    egui::Color32::from_white_alpha(26)
                );
                output.paint(&parent_painter).unwrap();
            });
        })
        .drop_without_applying_deltas();
    let submitted = backend.0.lock().unwrap();
    assert_eq!(submitted.len(), 1);
    assert_eq!(submitted[0].opacity(), 0.6);
    assert_eq!(
        shape(&submitted[0]).fill,
        egui::Color32::from_white_alpha(26)
    );
}

#[test]
fn justified_layout_keeps_registration_and_semantics_on_the_line() {
    let context = egui::Context::default();
    for _ in 0..2 {
        context
            .run_ui(
                egui::RawInput {
                    events: vec![egui::Event::PointerMoved(egui::pos2(100.0, 10.0))],
                    ..Default::default()
                },
                |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(200.0, 48.0),
                        egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                        |ui| {
                            let output = Separator::new(&Theme::light()).decorative(false).show(ui);
                            assert_eq!(output.response.rect.size(), egui::vec2(200.0, 1.0));
                            assert_eq!(
                                shape_rect(output.drawing.as_ref().unwrap()),
                                output.response.rect
                            );
                            assert!(!output.response.rect.contains(egui::pos2(100.0, 10.0)));
                            assert!(!output.response.contains_pointer());
                            assert!(!output.response.hovered());
                        },
                    );
                },
            )
            .drop_without_applying_deltas();
    }
}

#[test]
fn tessellation_preserves_fractional_geometry_and_antialiasing_bounds() {
    for scale in [1.0, 1.5, 2.0] {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            let context = egui::Context::default();
            context.set_pixels_per_point(scale);
            let mut clipped = None;
            let mut rect = egui::Rect::NOTHING;
            let mut group_bounds = egui::Rect::NOTHING;
            context
                .run_ui(Default::default(), |ui| {
                    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(
                        egui::Rect::from_min_size(egui::pos2(8.25, 23.5), egui::vec2(120.0, 48.0)),
                    ));
                    let output = Separator::new(&Theme::light())
                        .orientation(orientation)
                        .show(&mut child);
                    rect = output.response.rect;
                    let drawing = output.drawing.unwrap();
                    group_bounds = drawing.bounds();
                    let [Item::Shape(shape)] = drawing.items() else {
                        panic!("separator shape");
                    };
                    clipped = Some(shape.clone());
                })
                .drop_without_applying_deltas();
            let clipped = clipped.unwrap();
            assert_eq!(clipped.clip_rect, group_bounds);
            assert_eq!(group_bounds, rect.expand(0.5 / scale));
            let primitives = context.tessellate(vec![clipped], scale);
            let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive else {
                panic!("separator mesh");
            };
            let points: Vec<_> = mesh.vertices.iter().map(|v| v.pos).collect();
            let bounds = egui::Rect::from_points(&points);
            assert!((bounds.center() - rect.center()).length() < 0.001);
            assert!((bounds.min - group_bounds.min).length() < 0.001);
            assert!((bounds.max - group_bounds.max).length() < 0.001);
            let opaque: Vec<_> = mesh
                .vertices
                .iter()
                .filter(|v| v.color.a() > 0)
                .map(|v| v.pos)
                .collect();
            let inner = egui::Rect::from_points(&opaque);
            assert!((inner.center() - rect.center()).length() < 0.001);
        }
    }
}
