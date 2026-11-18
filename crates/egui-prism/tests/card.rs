//! Card geometry, drawing boundaries and ordinary nested input through the public API.
use egui_prism::{
    button::Button,
    card::Card,
    compositing::{self, Group, Item},
    egui,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    ctx
}
fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.02, "{a} != {b}");
}
fn texts(group: &Group) -> Vec<&egui::epaint::TextShape> {
    group
        .items()
        .iter()
        .filter_map(|item| match item {
            Item::Shape(s) => match &s.shape {
                egui::Shape::Text(t) => Some(t),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn pinned_section_geometry_and_typography_at_both_scales_and_themes() {
    for theme in [Theme::light(), Theme::dark()] {
        for scale in [1.0, 1.5] {
            let ctx = context();
            ctx.set_pixels_per_point(scale);
            ctx.run_ui(Default::default(), |ui| {
                theme.apply(ui.style_mut());
                ui.set_max_width(320.0);
                let mut content = egui::Rect::NOTHING;
                let mut footer = egui::Rect::NOTHING;
                let output = Card::new(&theme).show(ui, |card| {
                    card.header("Project", Some("Description"));
                    card.content(|body| {
                        content = body
                            .ui()
                            .allocate_exact_size(egui::vec2(200., 40.), egui::Sense::hover())
                            .0;
                    });
                    card.footer(36.0, |body| {
                        footer = body
                            .ui()
                            .allocate_exact_size(egui::vec2(80., 36.), egui::Sense::hover())
                            .0;
                    });
                    42
                });
                let rect = output.response.rect;
                let group = output.drawing.as_ref().unwrap();
                let labels = texts(group);
                near(rect.width(), 320.);
                near(rect.height(), 218.);
                near(labels[0].pos.x - rect.left(), 25.);
                near(labels[0].pos.y - rect.top(), 25.);
                near(labels[1].pos.y - labels[0].pos.y, 24.);
                near(content.top() - rect.top(), 93.);
                near(content.left() - rect.left(), 25.);
                near(footer.top() - content.bottom(), 24.);
                near(rect.bottom() - footer.bottom(), 25.);
                assert_eq!(output.inner, 42);
                assert_eq!(labels[0].galley.job.sections[0].format.font_id.size, 16.);
                assert_eq!(
                    labels[0].galley.job.sections[0].format.line_height,
                    Some(16.)
                );
                assert_eq!(
                    labels[0].galley.job.sections[0].format.font_id.family,
                    egui::FontFamily::Name("prism-geist-semibold".into())
                );
                assert_eq!(labels[1].galley.job.sections[0].format.font_id.size, 14.);
                assert_eq!(
                    labels[1].galley.job.sections[0].format.line_height,
                    Some(20.)
                );
                assert_eq!(labels[1].fallback_color, theme.colors.muted.foreground);
                let Item::Shape(shell) = &group.items()[1] else {
                    panic!()
                };
                let egui::Shape::Rect(shell) = &shell.shape else {
                    panic!()
                };
                assert_eq!(shell.fill, theme.colors.surface.background);
                assert_eq!(shell.corner_radius, egui::CornerRadius::same(14));
                let Item::Shape(border) = &group.items()[2] else {
                    panic!()
                };
                let egui::Shape::Rect(border) = &border.shape else {
                    panic!()
                };
                assert_eq!(border.stroke, egui::Stroke::new(1., theme.colors.border));
                assert_eq!(border.stroke_kind, egui::StrokeKind::Inside);
                assert_eq!(border.corner_radius, egui::CornerRadius::same(14));
            })
            .drop_without_applying_deltas();
        }
    }
}

#[test]
fn empty_sections_and_header_rows_have_no_implicit_control_height() {
    context()
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            ui.set_max_width(320.);
            near(
                Card::new(&theme).show(ui, |_| {}).response.rect.height(),
                50.,
            );
            near(
                Card::new(&theme)
                    .show(ui, |card| {
                        card.header("", None);
                    })
                    .response
                    .rect
                    .height(),
                58.,
            );
            near(
                Card::new(&theme)
                    .show(ui, |card| {
                        card.header("Title", None);
                    })
                    .response
                    .rect
                    .height(),
                74.,
            );
            near(
                Card::new(&theme)
                    .show(ui, |card| {
                        card.content(|_| {});
                        card.footer(0.0, |_| {});
                    })
                    .response
                    .rect
                    .height(),
                74.,
            );
        })
        .drop_without_applying_deltas();
}

#[test]
fn wrapping_grows_before_content_and_does_not_add_container_padding_twice() {
    context()
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            theme.apply(ui.style_mut());
            ui.set_max_width(140.);
            let output = Card::new(&theme).show(ui, |card| {
                card.header(
                    "A longer title with wrapping",
                    Some("A longer description with several words."),
                );
                card.content(|body| {
                    assert_eq!(body.ui().spacing().item_spacing.y, 8.);
                    body.ui()
                        .allocate_exact_size(egui::vec2(40., 20.), egui::Sense::hover())
                        .0
                })
            });
            let labels = texts(output.drawing.as_ref().unwrap());
            assert!(labels[0].galley.rows.len() > 1);
            assert!(labels[1].galley.rows.len() > 1);
            near(
                output.inner.top(),
                labels[1].pos.y + labels[1].galley.size().y + 24.,
            );
            near(output.inner.left(), output.response.rect.left() + 25.);
            near(output.response.rect.bottom(), output.inner.bottom() + 25.);
        })
        .drop_without_applying_deltas();
}

#[test]
fn parent_clip_bounds_drawings_without_clipping_overflow_to_the_card_box() {
    context()
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            ui.set_max_width(100.);
            let clip = egui::Rect::from_min_size(
                ui.available_rect_before_wrap().min,
                egui::vec2(200., 100.),
            );
            ui.set_clip_rect(clip);
            let output = Card::new(&theme).show(ui, |card| {
                card.content(|body| {
                    let child = Button::new(&theme, "A long action that may overflow the card")
                        .register(body.ui());
                    let rect = child.response.rect;
                    if let Some(group) = child.drawing {
                        body.draw(group);
                    }
                    rect
                })
            });
            near(output.response.rect.width(), 100.);
            assert!(output.inner.right() > output.response.rect.right());
            let group = output.drawing.unwrap();
            assert!(clip.contains_rect(group.bounds()));
            assert!(group.bounds().right() > output.response.rect.right());
            assert!(!output.response.sense.is_focusable());
            assert!(!output.response.sense.senses_click());
        })
        .drop_without_applying_deltas();
}

#[derive(Default)]
struct Host(Mutex<Vec<Group>>);
impl compositing::Backend for Host {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}

#[test]
fn explicit_groups_are_isolated_once_and_direct_shapes_stay_above_the_shell() {
    let ctx = context();
    let host = Arc::new(Host::default());
    let _installation = compositing::install(&ctx, host.clone()).unwrap();
    let output = ctx.run_ui(Default::default(), |ui| {
        ui.multiply_opacity(0.6);
        let theme = Theme::light();
        Card::new(&theme)
            .show(ui, |card| {
                card.header("Header", Some("Description"));
                card.content(|body| {
                    let output = Button::new(&theme, "Grouped child").register(body.ui());
                    if let Some(group) = output.drawing {
                        body.draw(group);
                    }
                    body.ui().label("Direct egui content");
                });
            })
            .paint(ui.painter())
            .unwrap();
    });
    let groups = host.0.lock().unwrap();
    assert_eq!(groups.len(), 1);
    near(groups[0].opacity(), 0.6);
    assert!(
        groups[0]
            .items()
            .iter()
            .any(|item| matches!(item,Item::Group(g) if g.opacity()==1.))
    );
    let callback = output
        .shapes
        .iter()
        .position(|s| matches!(s.shape, egui::Shape::Callback(_)))
        .unwrap();
    let text = output
        .shapes
        .iter()
        .position(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Direct egui content"),
        )
        .unwrap();
    assert!(callback < text);
    output.drop_without_applying_deltas();
}

#[test]
fn card_shadow_has_two_layers_and_excludes_the_rounded_border_box() {
    context()
        .run_ui(Default::default(), |ui| {
            let output = Card::new(&Theme::light()).show(ui, |_| {});
            let rect = output.response.rect;
            let Item::Shape(shadow) = &output.drawing.as_ref().unwrap().items()[0] else {
                panic!()
            };
            let egui::Shape::Mesh(mesh) = &shadow.shape else {
                panic!()
            };
            assert!(!mesh.indices.is_empty());
            assert!(mesh.vertices.iter().any(|v| v.pos.y > rect.bottom()));
            assert!(
                mesh.vertices
                    .iter()
                    .all(|v| !rect.shrink(14.).contains(v.pos))
            );
            // Both layers emit independently clipped triangles, rather than shadow-xs.
            assert!(mesh.vertices.iter().all(|v| v.color.a() <= 26));
            let point = egui::pos2(rect.center().x + 0.17, rect.bottom() + 0.2);
            let covering = mesh
                .indices
                .chunks_exact(3)
                .filter(|triangle| {
                    let [a, b, c] = [
                        mesh.vertices[triangle[0] as usize].pos,
                        mesh.vertices[triangle[1] as usize].pos,
                        mesh.vertices[triangle[2] as usize].pos,
                    ];
                    let cross = |a: egui::Vec2, b: egui::Vec2| a.x * b.y - a.y * b.x;
                    let area = cross(b - a, c - a);
                    if area.abs() < 1e-6 {
                        return false;
                    }
                    let u = cross(b - point, c - point) / area;
                    let v = cross(c - point, a - point) / area;
                    let w = 1. - u - v;
                    u > 0. && v > 0. && w > 0.
                })
                .count();
            assert_eq!(covering, 2, "both CSS shadow layers cover this point");
        })
        .drop_without_applying_deltas();
}

fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn nested_button_keeps_ordered_pointer_keyboard_and_disabled_behavior() {
    let ctx = context();
    ctx.options_mut(|o| o.ordered_input = true);
    let mut count = 0;
    let mut frame = |events: Vec<egui::Event>, disabled: bool| {
        let mut rect = egui::Rect::NOTHING;
        ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                ctx.begin_input();
                let output = Card::new(&Theme::light()).show(ui, |card| {
                    card.footer(36.0, |footer| {
                        if disabled {
                            footer.ui().disable();
                        }
                        Button::new(&Theme::light(), "Save").register(footer.ui())
                    })
                });
                let button = output.inner;
                rect = button.response.rect;
                while let Some(event) = ctx.next_input_event() {
                    if event.target == Some(button.response.id) && button.handle(&event).clicked() {
                        count += 1;
                    }
                }
            },
        )
        .drop_without_applying_deltas();
        rect
    };
    let rect = frame(vec![], false);
    frame(vec![], false);
    frame(
        vec![
            egui::Event::PointerMoved(rect.center()),
            pointer(rect.center(), true),
            pointer(rect.center(), false),
        ],
        false,
    );
    frame(vec![key(egui::Key::Tab), key(egui::Key::Enter)], false);
    frame(
        vec![
            pointer(rect.center(), true),
            pointer(rect.center(), false),
            key(egui::Key::Enter),
        ],
        true,
    );
    frame(vec![], false);
    assert_eq!(count, 2);
}

#[test]
fn missing_host_is_explicit_and_unbounded_clip_is_supported() {
    context()
        .run_ui(Default::default(), |ui| {
            ui.set_clip_rect(egui::Rect::EVERYTHING);
            ui.set_max_width(320.);
            assert!(matches!(
                Card::new(&Theme::light())
                    .show(ui, |card| card.header("Header", None))
                    .paint(ui.painter()),
                Err(compositing::Error::MissingHost)
            ));
        })
        .drop_without_applying_deltas();
}

#[test]
fn overflowing_content_cannot_resize_later_sections() {
    context()
        .run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            theme.apply(ui.style_mut());
            ui.set_max_width(140.);
            let output = Card::new(&theme).show(ui, |card| {
                card.content(|body| {
                    body.ui()
                        .allocate_exact_size(egui::vec2(400., 10.), egui::Sense::hover());
                });
                card.header("A longer title with wrapping", None);
                card.content(|body| near(body.ui().available_width(), 90.));
                card.footer(20., |body| near(body.ui().available_width(), 90.));
            });
            near(output.response.rect.width(), 140.);
            assert!(texts(output.drawing.as_ref().unwrap())[0].galley.rows.len() > 1);
        })
        .drop_without_applying_deltas();
}

#[test]
fn footer_explicit_height_centers_short_and_tall_children_in_either_order() {
    for heights in [[10.0_f32, 10.0], [20., 60.], [60., 20.]] {
        context()
            .run_ui(Default::default(), |ui| {
                let theme = Theme::light();
                theme.apply(ui.style_mut());
                let height = heights[0].max(heights[1]);
                let output = Card::new(&theme).show(ui, |card| {
                    card.footer(height, |footer| {
                        heights.map(|height| {
                            footer
                                .ui()
                                .allocate_exact_size(egui::vec2(20., height), egui::Sense::hover())
                                .0
                        })
                    })
                });
                near(output.response.rect.height(), 50. + height);
                near(output.inner[0].center().y, output.inner[1].center().y);
                near(
                    output.inner[0].center().y,
                    output.response.rect.top() + 25. + height * 0.5,
                );
            })
            .drop_without_applying_deltas();
    }
}

#[test]
fn header_clip_contains_accented_glyphs_outside_the_line_box() {
    context()
        .run_ui(Default::default(), |ui| {
            let output = Card::new(&Theme::light()).show(ui, |card| {
                card.header("ẤÉǗ", Some("Accents remain visible"))
            });
            for item in output.drawing.unwrap().items() {
                if let Item::Shape(shape) = item
                    && matches!(shape.shape, egui::Shape::Text(_))
                {
                    assert!(shape.clip_rect.contains_rect(
                        shape.shape.visual_bounding_rect().intersect(ui.clip_rect())
                    ));
                }
            }
        })
        .drop_without_applying_deltas();
}

#[test]
fn nested_editor_retains_caller_value_across_unrelated_frames() {
    let ctx = context();
    let mut value = String::new();
    let mut frame = |events: Vec<egui::Event>| {
        let mut rect = egui::Rect::NOTHING;
        ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                let output = Card::new(&Theme::light()).show(ui, |card| {
                    card.header("Editor", None);
                    card.content(|body| body.ui().text_edit_singleline(&mut value))
                });
                rect = output.inner.rect;
            },
        )
        .drop_without_applying_deltas();
        rect
    };
    let rect = frame(vec![]);
    frame(vec![]);
    frame(vec![
        egui::Event::PointerMoved(rect.center()),
        pointer(rect.center(), true),
        pointer(rect.center(), false),
    ]);
    frame(vec![egui::Event::Text("Draft".into())]);
    frame(vec![]);
    frame(vec![]);
    assert_eq!(value, "Draft");
}

#[test]
#[should_panic(expected = "Card footer height must fit its tallest child")]
fn footer_rejects_insufficient_height_instead_of_misaligning_controls() {
    context()
        .run_ui(Default::default(), |ui| {
            let _ = Card::new(&Theme::light()).show(ui, |card| {
                card.footer(10., |footer| {
                    footer
                        .ui()
                        .allocate_exact_size(egui::vec2(30., 40.), egui::Sense::hover());
                })
            });
        })
        .drop_without_applying_deltas();
}

#[test]
#[should_panic(expected = "Card footer height must be finite and nonnegative")]
fn footer_rejects_nonfinite_height() {
    context()
        .run_ui(Default::default(), |ui| {
            let _ = Card::new(&Theme::light()).show(ui, |card| card.footer(f32::NAN, |_| {}));
        })
        .drop_without_applying_deltas();
}

#[test]
fn footer_accepts_fractional_heights_on_eguis_layout_grid() {
    for scale in [1.0, 1.5] {
        for height in [0.01, 20.02, 36.01] {
            let ctx = context();
            ctx.set_pixels_per_point(scale);
            ctx.run_ui(Default::default(), |ui| {
                let output = Card::new(&Theme::light()).show(ui, |card| {
                    card.footer(height, |footer| {
                        footer
                            .ui()
                            .allocate_exact_size(egui::vec2(40., height), egui::Sense::hover())
                            .0
                    })
                });
                near(output.response.rect.height(), 50. + height);
                assert!((output.inner.height() - height).abs() <= egui::emath::GUI_ROUNDING);
            })
            .drop_without_applying_deltas();
        }
    }
}
