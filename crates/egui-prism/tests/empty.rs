//! Public Empty contracts; headless metadata and drawing, not browser/native acceptance.
use egui_prism::{
    button::Button,
    compositing::{self, Group, Item},
    egui,
    empty::Empty,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};

fn context() -> egui::Context {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    ctx
}
fn input(width: f32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 2400.),
        )),
        ..Default::default()
    }
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.02, "{actual} != {expected}");
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
fn viewport_breakpoint_is_independent_of_container_and_short_copy_is_natural_height() {
    for scale in [1., 1.5] {
        for (viewport, padding) in [(767., 24.), (768., 48.), (1200., 48.)] {
            for width in [160., 320., 640.] {
                let ctx = context();
                ctx.set_pixels_per_point(scale);
                ctx.run_ui(input(viewport), |_| {})
                    .drop_without_applying_deltas();
                ctx.run_ui(input(viewport), |ui| {
                    let theme = Theme::light();
                    theme.apply(ui.style_mut());
                    ui.set_max_width(width);
                    let out = Empty::new(&theme, "empty")
                        .title("Nothing yet")
                        .description("Start here.")
                        .show(ui, |body| {
                            near(
                                body.ui().available_width(),
                                (width - padding * 2.).clamp(0., 384.),
                            );
                        });
                    let labels = texts(out.drawing.as_ref().unwrap());
                    near(out.response.rect.width(), width);
                    near(
                        out.response.rect.height(),
                        padding * 2. + labels[0].galley.size().y + 8. + labels[1].galley.size().y,
                    );
                    near(labels[0].pos.y, out.response.rect.top() + padding);
                    near(
                        labels[1].pos.y,
                        labels[0].pos.y + labels[0].galley.size().y + 8.,
                    );
                    for text in labels {
                        near(
                            text.pos.x + text.galley.rect.center().x,
                            out.response.rect.center().x,
                        );
                    }
                    assert!(!out.response.sense.is_focusable());
                    assert!(!out.response.sense.senses_click());
                    assert!(!out.response.sense.senses_drag());
                })
                .drop_without_applying_deltas();
            }
        }
    }
}

#[test]
fn pinned_typography_colors_icon_and_content_gaps_survive_theme_metric_changes() {
    for mut theme in [Theme::light(), Theme::dark()] {
        theme.typography.body = 40.;
        theme.metrics.item_spacing = egui::vec2(90., 90.);
        context()
            .run_ui(input(900.), |ui| {
                theme.apply(ui.style_mut());
                ui.set_max_width(600.);
                let out = Empty::new(&theme, "empty")
                    .title("No projects")
                    .description("Add your first project.")
                    .icon((egui::TextureId::User(12), egui::vec2(24., 24.)))
                    .show(ui, |content| {
                        near(content.ui().spacing().item_spacing.y, 16.);
                        let a = content
                            .ui()
                            .allocate_exact_size(egui::vec2(60., 36.), egui::Sense::hover())
                            .0;
                        let b = content
                            .ui()
                            .allocate_exact_size(egui::vec2(90., 10.), egui::Sense::hover())
                            .0;
                        (a, b)
                    });
                let group = out.drawing.as_ref().unwrap();
                let labels = texts(group);
                let title = &labels[0].galley.job.sections[0].format;
                let description = &labels[1].galley.job.sections[0].format;
                assert_eq!(title.font_id.size, 18.);
                assert_eq!(
                    title.font_id.family,
                    egui::FontFamily::Name("prism-geist-medium".into())
                );
                assert_eq!(title.line_height, Some(28.));
                assert_eq!(title.extra_letter_spacing, -0.45);
                assert_eq!(title.color, theme.colors.background.foreground);
                assert_eq!(description.font_id.size, 14.);
                assert_eq!(description.line_height, Some(22.75));
                assert_eq!(description.color, theme.colors.muted.foreground);
                let Item::Shape(tile) = &group.items()[0] else {
                    panic!()
                };
                let egui::Shape::Rect(tile) = &tile.shape else {
                    panic!()
                };
                assert_eq!(tile.rect.size(), egui::vec2(40., 40.));
                assert_eq!(tile.corner_radius, egui::CornerRadius::same(10));
                assert_eq!(tile.fill, theme.colors.muted.background);
                near(labels[0].pos.y, tile.rect.bottom() + 16.);
                near(
                    out.inner.0.top(),
                    labels[1].pos.y + labels[1].galley.size().y + 24.,
                );
                near(out.inner.1.top(), out.inner.0.bottom() + 16.);
                near(out.inner.0.center().x, out.response.rect.center().x);
                near(out.inner.1.center().x, out.response.rect.center().x);
                near(out.response.rect.bottom(), out.inner.1.bottom() + 48.);
            })
            .drop_without_applying_deltas();
    }
}

#[test]
fn absent_empty_and_content_only_sections_have_no_fabricated_gaps_or_shell() {
    context()
        .run_ui(input(600.), |ui| {
            let theme = Theme::light();
            let out = Empty::new(&theme, "none").show(ui, |_| 7);
            assert_eq!(out.inner, 7);
            near(out.response.rect.height(), 48.);
            assert!(out.drawing.is_none());
            let out = Empty::new(&theme, "empty-labels")
                .title("")
                .description("")
                .show(ui, |_| {});
            near(out.response.rect.height(), 56.);
            assert!(out.drawing.is_none());
            let out = Empty::new(&theme, "content").show(ui, |body| {
                body.ui()
                    .allocate_exact_size(egui::vec2(20., 15.), egui::Sense::hover())
                    .0
            });
            near(out.inner.top(), out.response.rect.top() + 24.);
            near(out.response.rect.height(), 63.);
            let out = Empty::new(&theme, "only-title")
                .title("Title")
                .show(ui, |_| {});
            assert_eq!(
                out.drawing.unwrap().items().len(),
                1,
                "no default border or background in pinned Empty"
            );
        })
        .drop_without_applying_deltas();
}

#[test]
fn long_copy_balances_without_elision_and_preserves_words_nbsp_and_all_content() {
    context()
        .run_ui(input(900.), |ui| {
            let theme = Theme::light();
            theme.apply(ui.style_mut());
            ui.set_max_width(360.);
            let copy = "Your saved projects will appear here after you create or import a project";
            let out = Empty::new(&theme, "long")
                .description(copy)
                .show(ui, |_| {});
            let text = texts(out.drawing.as_ref().unwrap())[0];
            assert!(text.galley.rows.len() > 1);
            assert_eq!(text.galley.text(), copy);
            assert!(!text.galley.elided);
            assert!(
                text.galley.job.wrap.max_width < 264.,
                "short copy is balanced, not ordinary full-width wrapping"
            );
            for row in &text.galley.rows {
                assert!(row.rect().center().x.abs() <= 0.5);
            }
            for copy in [
                "Supercalifragilisticexpialidocious",
                "This\u{a0}entire\u{a0}run\u{a0}must\u{a0}stay\u{a0}together",
            ] {
                ui.scope(|ui| {
                    ui.set_width(140.);
                    let out = Empty::new(&theme, copy).description(copy).show(ui, |_| {});
                    let text = texts(out.drawing.as_ref().unwrap())[0];
                    assert_eq!(text.galley.rows.len(), 1);
                    assert_eq!(text.galley.text(), copy);
                    near(out.response.rect.width(), 140.);
                });
            }
        })
        .drop_without_applying_deltas();
}

#[test]
fn whitespace_collapses_but_accessibility_keeps_complete_copy_and_stable_ids_without_live_semantics()
 {
    let ctx = context();
    ctx.enable_accesskit();
    let mut ids = Vec::new();
    for copy in ["  No\nprojects\tyet  ", "A changed explanation"] {
        let mut output = ctx.run_ui(input(900.), |ui| {
            let theme = Theme::light();
            let out = Empty::new(&theme, "stable")
                .title("Title")
                .description(copy)
                .show(ui, |_| {});
            let b = Empty::new(&theme, "other").title("Title").show(ui, |_| {});
            assert_ne!(out.response.id, b.response.id);
            ids.push(out.response.id);
            if copy.starts_with(' ') {
                assert_eq!(
                    texts(out.drawing.as_ref().unwrap())[1].galley.text(),
                    "No projects yet"
                );
            }
        });
        output.textures_delta.clear();
        let tree = output.platform_output.accesskit_update.as_ref().unwrap();
        let label = tree
            .nodes
            .iter()
            .find(|(_, node)| node.value() == Some(copy))
            .expect("complete accessible description");
        assert_eq!(label.1.role(), egui::accesskit::Role::Label);
        assert_eq!(
            label.1.live().unwrap_or(egui::accesskit::Live::Off),
            egui::accesskit::Live::Off
        );
        assert!(!tree.nodes.iter().any(|(_, node)| matches!(
            node.role(),
            egui::accesskit::Role::Alert
                | egui::accesskit::Role::Status
                | egui::accesskit::Role::ProgressIndicator
        )));
        output.drop_without_applying_deltas();
    }
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn narrow_clipped_and_accented_copy_keeps_finite_bounds_and_its_glyphs() {
    for width in [0., 20., 96., 130., 320.] {
        context()
            .run_ui(input(900.), |ui| {
                let theme = Theme::light();
                theme.apply(ui.style_mut());
                ui.set_max_width(width);
                let clip = egui::Rect::from_min_size(
                    ui.available_rect_before_wrap().min,
                    egui::vec2(320., 900.),
                );
                ui.set_clip_rect(clip);
                let out = Empty::new(&theme, "clipped")
                    .title("Éléments déjà créés")
                    .description("Überprüfen Sie Ihre gespeicherten Projekte.")
                    .show(ui, |body| {
                        let child =
                            Button::new(&theme, "An intentionally overflowing caller action")
                                .register(body.ui());
                        let rect = child.response.rect;
                        if let Some(group) = child.drawing {
                            body.draw(group);
                        }
                        rect
                    });
                near(out.response.rect.width(), width);
                let group = out.drawing.unwrap();
                assert!(clip.contains_rect(group.bounds()));
                assert!(group.bounds().is_finite());
                for item in group.items() {
                    if let Item::Shape(s) = item {
                        assert!(
                            s.clip_rect
                                .contains_rect(s.shape.visual_bounding_rect().intersect(clip))
                        );
                    }
                }
            })
            .drop_without_applying_deltas();
    }
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
fn explicit_child_groups_get_opacity_once_and_direct_paint_stays_above_the_group() {
    let ctx = context();
    let host = Arc::new(Host::default());
    let _install = compositing::install(&ctx, host.clone()).unwrap();
    let output = ctx.run_ui(input(900.), |ui| {
        ui.multiply_opacity(0.6);
        let theme = Theme::light();
        Empty::new(&theme, "opacity")
            .title("Nothing yet")
            .show(ui, |body| {
                let child = Button::new(&theme, "Create").register(body.ui());
                if let Some(group) = child.drawing {
                    body.draw(group);
                }
                body.ui().label("Direct caller text");
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
            .any(|i| matches!(i,Item::Group(g) if g.opacity()==1.))
    );
    let callback = output
        .shapes
        .iter()
        .position(|s| matches!(s.shape, egui::Shape::Callback(_)))
        .unwrap();
    let direct = output
        .shapes
        .iter()
        .position(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Direct caller text"),
        )
        .unwrap();
    assert!(callback < direct);
    output.drop_without_applying_deltas();
}
#[test]
fn missing_host_is_explicit_and_unbounded_parent_clip_is_supported() {
    context()
        .run_ui(input(900.), |ui| {
            ui.set_clip_rect(egui::Rect::EVERYTHING);
            ui.set_max_width(320.);
            assert!(matches!(
                Empty::new(&Theme::light(), "no-host")
                    .title("Title")
                    .show(ui, |_| {})
                    .paint(ui.painter()),
                Err(compositing::Error::MissingHost)
            ));
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
fn child_actions_remain_caller_owned_stable_and_disabled_safe() {
    let ctx = context();
    ctx.options_mut(|o| o.ordered_input = true);
    let mut count = 0;
    let mut ids = Vec::new();
    let mut frame = |events: Vec<egui::Event>, disabled: bool, title: &str| {
        let mut rect = egui::Rect::NOTHING;
        ctx.run_ui(
            egui::RawInput {
                events,
                ..input(900.)
            },
            |ui| {
                ctx.begin_input();
                let out = Empty::new(&Theme::light(), "stable")
                    .title(title)
                    .show(ui, |body| {
                        if disabled {
                            body.ui().disable();
                        }
                        Button::new(&Theme::light(), "Create").register(body.ui())
                    });
                let button = out.inner;
                rect = button.response.rect;
                ids.push(button.response.id);
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
    let rect = frame(vec![], false, "Nothing yet");
    frame(
        vec![
            egui::Event::PointerMoved(rect.center()),
            pointer(rect.center(), true),
            pointer(rect.center(), false),
        ],
        false,
        "Nothing yet",
    );
    frame(
        vec![key(egui::Key::Tab), key(egui::Key::Enter)],
        false,
        "Changed copy",
    );
    frame(
        vec![
            pointer(rect.center(), true),
            pointer(rect.center(), false),
            key(egui::Key::Enter),
        ],
        true,
        "Changed copy",
    );
    frame(vec![], false, "Nothing yet");
    assert_eq!(count, 2);
    assert!(ids.iter().all(|id| *id == ids[0]));
}

#[test]
fn repeated_words_in_min_content_columns_never_create_whitespace_only_rows() {
    for scale in [1., 1.5] {
        let ctx = context();
        ctx.set_pixels_per_point(scale);
        ctx.run_ui(input(900.), |_| {})
            .drop_without_applying_deltas();
        ctx.run_ui(input(900.), |ui| {
            let theme = Theme::light();
            theme.apply(ui.style_mut());
            for copy in [
                "office office office",
                "WWWWW WWWWW WWWWW",
                "one\u{a0}two one\u{a0}two one\u{a0}two",
            ] {
                for title in [true, false] {
                    ui.scope(|ui| {
                        ui.set_width(140.);
                        let empty = Empty::new(&theme, (copy, title));
                        let empty = if title {
                            empty.title(copy)
                        } else {
                            empty.description(copy)
                        };
                        let out = empty.show(ui, |_| {});
                        let text = texts(out.drawing.as_ref().unwrap())[0];
                        assert_eq!(text.galley.rows.len(), 3, "{scale}, {copy}, title={title}");
                        assert!(
                            text.galley
                                .rows
                                .iter()
                                .all(|row| !row.text().trim().is_empty())
                        );
                        assert_eq!(text.galley.text(), copy);
                        near(out.response.rect.height(), 96. + text.galley.size().y);
                    });
                }
            }
        })
        .drop_without_applying_deltas();
    }
}
