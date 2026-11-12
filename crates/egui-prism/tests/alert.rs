//! Public Alert layout, semantic, clipping and compositing contracts.
use egui_prism::{
    alert::{Alert, AlertOutput, AlertVariant},
    compositing::{self, Backend, Error, Group, Item},
    egui,
    theme::{self, Theme},
};
use std::sync::{Arc, Mutex};
fn context() -> egui::Context {
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx, egui::FontDefinitions::default());
    ctx
}
fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}
fn background(output: &AlertOutput) -> &egui::epaint::RectShape {
    let Item::Shape(shape) = &output.drawing.as_ref().unwrap().items()[0] else {
        panic!("shape")
    };
    let egui::Shape::Rect(rect) = &shape.shape else {
        panic!("rect")
    };
    rect
}
fn texts(output: &AlertOutput) -> Vec<&egui::epaint::TextShape> {
    output
        .drawing
        .as_ref()
        .unwrap()
        .items()
        .iter()
        .filter_map(|item| match item {
            Item::Shape(shape) => match &shape.shape {
                egui::Shape::Text(text) => Some(text),
                _ => None,
            },
            _ => None,
        })
        .collect()
}
#[test]
fn reference_geometry_and_light_dark_variants() {
    for theme in [Theme::light(), Theme::dark()] {
        context()
            .run_ui(Default::default(), |ui| {
                theme.apply(ui.style_mut());
                ui.set_max_width(448.0);
                for (i, variant) in [AlertVariant::Default, AlertVariant::Destructive]
                    .into_iter()
                    .enumerate()
                {
                    let out = Alert::new(&theme, i)
                        .title("Heads up!")
                        .description("You can add components.")
                        .variant(variant)
                        .show(ui);
                    close(out.response.rect.width(), 448.0);
                    close(out.response.rect.height(), 68.0);
                    let bg = background(&out);
                    assert_eq!(bg.fill, theme.colors.surface.background);
                    assert_eq!(bg.stroke, egui::Stroke::new(1.0, theme.colors.border));
                    assert_eq!(bg.corner_radius, egui::CornerRadius::same(10));
                    let text = texts(&out);
                    assert_eq!(text.len(), 2);
                    close(text[0].pos.x - bg.rect.left(), 17.0);
                    close(text[0].pos.y - bg.rect.top(), 13.0);
                    close(text[1].pos.y - text[0].pos.y, 22.0);
                    assert_eq!(
                        text[0].galley.job.sections[0].format.font_id,
                        egui::FontId::new(
                            14.0,
                            egui::FontFamily::Name("prism-geist-medium".into())
                        )
                    );
                    close(
                        text[0].galley.job.sections[0].format.extra_letter_spacing,
                        -0.35,
                    );
                    assert_eq!(
                        text[1].galley.job.sections[0].format.font_id,
                        egui::FontId::proportional(14.0)
                    );
                    for t in &text {
                        assert_eq!(t.galley.job.sections[0].format.line_height, Some(20.0));
                    }
                    let fg = if variant == AlertVariant::Default {
                        theme.colors.surface.foreground
                    } else {
                        theme.colors.destructive.background
                    };
                    assert_eq!(text[0].galley.job.sections[0].format.color, fg);
                    let desc = if variant == AlertVariant::Default {
                        theme.colors.muted.foreground
                    } else {
                        let [r, g, b, a] = fg.to_srgba_unmultiplied();
                        egui::Color32::from_rgba_unmultiplied(
                            r,
                            g,
                            b,
                            (f32::from(a) * 0.9).round() as u8,
                        )
                    };
                    assert_eq!(text[1].galley.job.sections[0].format.color, desc);
                }
            })
            .drop_without_applying_deltas();
    }
}
#[test]
fn icon_title_description_combinations_use_css_grid_rows() {
    context()
        .run_ui(Default::default(), |ui| {
            ui.set_max_width(448.0);
            let theme = Theme::light();
            let texture = ui.ctx().load_texture(
                "icon",
                egui::ColorImage::filled([16, 16], egui::Color32::WHITE),
                egui::TextureOptions::LINEAR,
            );
            for title in [false, true] {
                for description in [false, true] {
                    for icon in [false, true] {
                        let mut alert = Alert::new(&theme, (title, description, icon));
                        if title {
                            alert = alert.title("Title");
                        }
                        if description {
                            alert = alert.description("Description");
                        }
                        if icon {
                            alert = alert.icon((texture.id(), egui::vec2(16.0, 16.0)));
                        }
                        let out = alert.show(ui);
                        close(
                            out.response.rect.height(),
                            26.0 + if title && description {
                                42.0
                            } else if title || description {
                                20.0
                            } else if icon {
                                16.0
                            } else {
                                0.0
                            },
                        );
                        for t in texts(&out) {
                            close(
                                t.pos.x - out.response.rect.left(),
                                if icon { 45.0 } else { 17.0 },
                            );
                        }
                        if icon {
                            let items = out.drawing.as_ref().unwrap().items();
                            let Item::Shape(s) = &items[1] else {
                                panic!("icon")
                            };
                            let egui::Shape::Rect(icon) = &s.shape else {
                                panic!("image")
                            };
                            close(icon.rect.width(), 16.0);
                            close(icon.rect.height(), 16.0);
                            close(icon.rect.top() - out.response.rect.top(), 15.0);
                        }
                    }
                }
            }
        })
        .drop_without_applying_deltas();
}
#[test]
fn title_elides_description_wraps_and_complete_semantics_keep_stable_identity() {
    let ctx = context();
    ctx.enable_accesskit();
    let title =
        "A deliberately long title that must elide visually without losing its accessible name";
    let desc = "A long description wraps onto several lines inside the available width and remains accessible.";
    let mut ids = Vec::new();
    for reverse in [false, true] {
        let out = ctx.run_ui(Default::default(), |ui| {
            ui.set_max_width(160.0);
            let mut order = [0, 1];
            if reverse {
                order.reverse();
            }
            let mut frame_ids = [egui::Id::NULL; 2];
            for i in order {
                let a = Alert::new(&Theme::light(), i)
                    .title(title)
                    .description(desc)
                    .show(ui);
                frame_ids[i] = a.response.id;
                let t = texts(&a);
                assert_eq!(t[0].galley.rows.len(), 1);
                assert!(t[0].galley.elided);
                assert!(t[1].galley.rows.len() > 1);
                close(a.response.rect.width(), 160.0);
                assert_eq!(a.response.sense, egui::Sense::hover());
                assert!(!a.response.clicked());
                assert!(!a.response.has_focus());
            }
            ids.push(frame_ids);
        });
        let update = out.platform_output.accesskit_update.as_ref().unwrap();
        let nodes: Vec<_> = update
            .nodes
            .iter()
            .map(|(_, n)| n)
            .filter(|n| n.role() == egui::Role::Alert)
            .collect();
        assert_eq!(nodes.len(), 2);
        for n in nodes {
            assert_eq!(n.label(), Some(format!("{title}\n{desc}").as_str()));
            assert_eq!(n.description(), None);
            assert_eq!(n.live(), Some(egui::accesskit::Live::Assertive));
            assert!(n.is_live_atomic());
            assert!(!n.supports_action(egui::accesskit::Action::Focus));
            assert!(!n.supports_action(egui::accesskit::Action::Click));
        }
        out.drop_without_applying_deltas();
    }
    assert_eq!(ids[0], ids[1]);
    assert_ne!(ids[0][0], ids[0][1]);
}
#[test]
fn narrow_zero_and_clipped_layouts_bound_the_group_and_preserve_exact_response() {
    for width in [0.0, 20.0, 34.0, 40.0, 80.0, 320.0] {
        for scale in [1.0, 1.5] {
            let ctx = context();
            ctx.set_pixels_per_point(scale);
            ctx.run_ui(Default::default(), |ui| {
                ui.set_max_width(width);
                let clip = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(width, 50.0));
                ui.set_clip_rect(clip);
                let out = Alert::new(&Theme::light(), "narrow")
                    .title("Long title")
                    .description("Words that can wrap with a long description")
                    .show(ui);
                close(out.response.rect.width(), width);
                if let Some(group) = out.drawing {
                    assert!(group.bounds().is_finite());
                    assert!(clip.contains_rect(group.bounds()));
                    for item in group.items() {
                        let Item::Shape(s) = item else {
                            panic!("shape")
                        };
                        assert!(clip.contains_rect(s.clip_rect));
                    }
                }
            })
            .drop_without_applying_deltas();
        }
    }
    context()
        .run_ui(Default::default(), |ui| {
            ui.set_clip_rect(egui::Rect::NOTHING);
            assert!(
                Alert::new(&Theme::light(), "hidden")
                    .title("Title")
                    .show(ui)
                    .drawing
                    .is_none()
            );
        })
        .drop_without_applying_deltas();
}
#[test]
fn whitespace_empty_content_and_custom_theme_metrics() {
    context()
        .run_ui(Default::default(), |ui| {
            ui.set_max_width(448.0);
            ui.set_clip_rect(egui::Rect::EVERYTHING);
            let mut theme = Theme::light();
            theme.metrics.control_height = 99.0;
            theme.metrics.corner_radius = egui::CornerRadius::same(30);
            theme.typography.body = 50.0;
            let out = Alert::new(&theme, "space")
                .title("  Hello\n\t world\u{00a0}now  ")
                .description("  More\n words  ")
                .show(ui);
            let t = texts(&out);
            assert_eq!(t[0].galley.job.text, "Hello world\u{00a0}now");
            assert_eq!(t[1].galley.job.text, "More words");
            close(out.response.rect.height(), 68.0);
            assert_eq!(background(&out).corner_radius, egui::CornerRadius::same(10));
            let empty = Alert::new(&theme, "empty").show(ui);
            close(empty.response.rect.height(), 26.0);
            let title = Alert::new(&theme, "empty-title").title("").show(ui);
            close(title.response.rect.height(), 42.0);
            let desc = Alert::new(&theme, "empty-desc").description("").show(ui);
            close(desc.response.rect.height(), 26.0);
        })
        .drop_without_applying_deltas();
}
#[derive(Default)]
struct Recorder(Mutex<Vec<Group>>);
impl Backend for Recorder {
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
fn explicit_submission_disabled_scope_and_parent_opacity() {
    let ctx = context();
    let recorder = Arc::new(Recorder::default());
    let _host = compositing::install(&ctx, recorder.clone()).unwrap();
    ctx.run_ui(Default::default(), |ui| {
        ui.multiply_opacity(0.6);
        let painter = ui.painter().clone();
        ui.add_enabled_ui(false, |ui| {
            let out = Alert::new(&Theme::dark(), "error")
                .title("Error")
                .variant(AlertVariant::Destructive)
                .show(ui);
            assert!(!out.response.enabled());
            assert_eq!(out.drawing.as_ref().unwrap().opacity(), 1.0);
            assert!(recorder.0.lock().unwrap().is_empty());
            out.paint(&painter).unwrap();
        });
    })
    .drop_without_applying_deltas();
    let groups = recorder.0.lock().unwrap();
    assert_eq!(groups.len(), 1);
    close(groups[0].opacity(), 0.6);
    context()
        .run_ui(Default::default(), |ui| {
            assert!(matches!(
                Alert::new(&Theme::light(), "no-host")
                    .show(ui)
                    .paint(ui.painter()),
                Err(Error::MissingHost)
            ));
        })
        .drop_without_applying_deltas();
}
#[test]
fn exact_bounds_survive_justified_parent_layout() {
    context()
        .run_ui(Default::default(), |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(400.0, 160.0),
                egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    let out = Alert::new(&Theme::light(), "justified")
                        .title("Title")
                        .show(ui);
                    close(out.response.rect.height(), 46.0);
                    assert_eq!(
                        ui.ctx().read_response(out.response.id).unwrap().rect,
                        out.response.rect
                    );
                    assert_eq!(background(&out).rect, out.response.rect);
                },
            );
        })
        .drop_without_applying_deltas();
}

#[test]
fn unbreakable_words_and_nbsp_keep_min_content_width_without_expanding_the_notice() {
    for description in [
        "Supercalifragilisticexpialidocious",
        "A\u{a0}long\u{a0}nonbreaking\u{a0}phrase",
    ] {
        context()
            .run_ui(Default::default(), |ui| {
                ui.set_max_width(160.0);
                let output = Alert::new(&Theme::light(), "long-word")
                    .description(description)
                    .show(ui);
                close(output.response.rect.width(), 160.0);
                close(output.response.rect.height(), 46.0);
                let text = texts(&output);
                assert_eq!(text[0].galley.rows.len(), 1);
                assert!(text[0].galley.size().x > 126.0);
                let group = output.drawing.as_ref().unwrap();
                assert!(group.bounds().width() <= 162.0);
                for item in group.items().iter().skip(1) {
                    let Item::Shape(shape) = item else {
                        panic!("text")
                    };
                    assert!(shape.clip_rect.right() <= output.response.rect.right() - 17.0);
                }
            })
            .drop_without_applying_deltas();
    }
}

#[test]
fn description_only_and_body_updates_change_the_complete_live_region_label() {
    let ctx = context();
    ctx.enable_accesskit();
    let mut ids = Vec::new();
    for (title, body) in [
        (None, "First body"),
        (None, "Changed body"),
        (Some("Title"), "Changed body"),
        (Some("Title"), "New body"),
    ] {
        let output = ctx.run_ui(Default::default(), |ui| {
            let theme = Theme::light();
            let mut notice = Alert::new(&theme, "stable").description(body);
            if let Some(title) = title {
                notice = notice.title(title);
            }
            ids.push(notice.show(ui).response.id);
        });
        let node = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == egui::Role::Alert)
            .unwrap();
        let expected = title.map_or_else(|| body.to_owned(), |title| format!("{title}\n{body}"));
        assert_eq!(node.label(), Some(expected.as_str()));
        assert_eq!(node.description(), None);
        assert_eq!(node.live(), Some(egui::accesskit::Live::Assertive));
        assert!(node.is_live_atomic());
        output.drop_without_applying_deltas();
    }
    assert!(ids.windows(2).all(|ids| ids[0] == ids[1]));
}
