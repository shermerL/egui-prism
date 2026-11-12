//! Public Item layout, independent input, semantics and composition contracts.
use egui_prism::{
    button::{Button, ButtonRole, IconButton},
    compositing::{self, Group, Item as Drawing},
    egui,
    item::{Item, ItemAction, ItemSize, ItemVariant},
    theme::Theme,
};
use std::sync::{Arc, Mutex};
fn context() -> egui::Context {
    let ctx = egui::Context::default();
    egui_prism::theme::install_fonts(&ctx, egui::FontDefinitions::default());
    ctx.all_styles_mut(|style| Theme::light().apply(style));
    ctx
}
fn input(width: f32, events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 900.0),
        )),
        events,
        ..Default::default()
    }
}
fn texts(group: &Group) -> Vec<&egui::epaint::TextShape> {
    group
        .items()
        .iter()
        .flat_map(|item| match item {
            Drawing::Shape(shape) => match &shape.shape {
                egui::Shape::Text(text) => vec![text],
                _ => vec![],
            },
            Drawing::Group(_) => vec![],
        })
        .collect()
}
#[test]
fn finite_slots_keep_passive_root_and_independent_final_hit_rectangles() {
    for width in [600.0, 160.0, 80.0] {
        let ctx = context();
        ctx.run_ui(input(width, vec![]), |ui| {
            let theme = Theme::light();
            let output = Item::new(&theme, "row")
                .title("Project")
                .description("A useful description of the current project")
                .icon((egui::TextureId::User(1), egui::vec2(16.0, 16.0)))
                .action(ItemAction::new(
                    "open",
                    Button::new(&theme, "Open").role(ButtonRole::Outline),
                ))
                .action(ItemAction::new(
                    "download",
                    Button::new(&theme, "Download original archive"),
                ))
                .register(ui);
            assert!(!output.response.sense.is_focusable());
            assert!(!output.response.sense.senses_click());
            assert!(
                output
                    .actions
                    .iter()
                    .all(|action| output.ids.contains(&action.response.id))
            );
            let [a, b] = output.actions.as_slice() else {
                panic!("actions");
            };
            assert!(!a.response.rect.intersects(b.response.rect));
            assert!(ui.min_rect().contains_rect(b.response.rect));
            assert_eq!(b.response.rect.left() - a.response.rect.right(), 8.0);
            assert_eq!(
                a.response.rect,
                ctx.read_response(a.response.id).unwrap().rect
            );
            assert_eq!(
                b.response.rect,
                ctx.read_response(b.response.id).unwrap().rect
            );
            assert!(a.drawing.is_none() && b.drawing.is_none());
            let drawing = output.drawing.unwrap();
            let text = texts(&drawing);
            for label in text {
                assert!(!label.visual_bounding_rect().intersects(a.response.rect));
            }
            if width < 200.0 {
                assert!(a.response.rect.top() > output.response.rect.top() + 50.0);
            }
            assert_eq!(
                drawing
                    .items()
                    .iter()
                    .filter(|item| matches!(item, Drawing::Group(_)))
                    .count(),
                output
                    .actions
                    .iter()
                    .filter(|action| ui.is_rect_visible(action.response.rect.expand(4.0)))
                    .count()
            );
        })
        .drop_without_applying_deltas();
    }
}
#[test]
fn action_identity_survives_reorder_and_nested_item_instances() {
    let ctx = context();
    let mut first = None;
    for reversed in [false, true] {
        ctx.run_ui(input(500.0, vec![]), |ui| {
            let theme = Theme::light();
            let mut item = Item::new(&theme, "first").title("Changed copy");
            for key in if reversed { ["b", "a"] } else { ["a", "b"] } {
                item = item.action(ItemAction::new(key, Button::new(&theme, key)));
            }
            let out = item.register(ui);
            let mut ids = out
                .actions
                .iter()
                .map(|a| a.response.id)
                .collect::<Vec<_>>();
            if reversed {
                ids.reverse();
            }
            if let Some(previous) = &first {
                assert_eq!(&ids, previous);
            } else {
                first = Some(ids.clone());
            }
            let second = Item::new(&theme, "second")
                .action(ItemAction::new("a", Button::new(&theme, "a")))
                .register(ui);
            assert_ne!(ids[0], second.actions[0].response.id);
        })
        .drop_without_applying_deltas();
    }
}
#[test]
fn description_clamps_visibly_but_preserves_full_accessible_copy() {
    let ctx = context();
    ctx.enable_accesskit();
    let title = "项目 Project 🔎";
    let description = "This complete description contains enough words for many lines, and must remain available to accessibility even when its visible drawing is limited to two lines.";
    let mut inspected = false;
    let mut output = ctx.run_ui(input(180.0, vec![]), |ui| {
        let theme = Theme::light();
        let out = Item::new(&theme, "copy")
            .title(title)
            .description(description)
            .register(ui);
        let group = out.drawing.unwrap();
        let text = texts(&group);
        let desc = text.last().unwrap();
        assert!(desc.galley.elided);
        assert_eq!(desc.galley.rows.len(), 2);
        assert_eq!(desc.galley.job.text, description);
        inspected = true;
    });
    assert!(inspected);
    output.textures_delta.clear();
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    for label in [title, description] {
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.value() == Some(label)
                    && node.role() == egui::accesskit::Role::Label)
        );
    }
    output.drop_without_applying_deltas();
}
#[test]
fn source_sizes_theme_border_and_clipping_are_explicit() {
    for theme in [Theme::light(), Theme::dark()] {
        for scale in [1.0, 1.5] {
            for size in [ItemSize::Default, ItemSize::Small] {
                let ctx = context();
                ctx.set_pixels_per_point(scale);
                ctx.run_ui(input(500.0,vec![]), |ui| {
            let clip=egui::Rect::from_min_size(egui::pos2(0.0,0.0),egui::vec2(200.0,200.0));ui.set_clip_rect(clip);
            let out=Item::new(&theme,"item").variant(ItemVariant::Outline).size(size).title("Title").register(ui);
            let g=out.drawing.unwrap();assert!(clip.contains_rect(g.bounds()));
            let t=texts(&g)[0];assert_eq!(t.galley.job.sections[0].format.font_id.size,14.0);
            assert_eq!(t.galley.job.sections[0].format.line_height,Some(19.25));
            assert_eq!(t.pos.x-out.response.rect.left(),17.0);
            assert_eq!(t.pos.y-out.response.rect.top(),if size==ItemSize::Small {13.0} else {17.0});
            assert!(g.items().iter().any(|item| matches!(item,Drawing::Shape(s) if matches!(&s.shape,egui::Shape::Rect(r) if r.stroke.color==theme.colors.border && r.stroke.width==1.0))));
        }).drop_without_applying_deltas();
            }
        }
    }
}
struct Collector(Mutex<Vec<Group>>);
impl compositing::Backend for Collector {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        self.0.lock().unwrap().push(group.clone());
        Ok(egui::PaintCallback {
            rect: group.bounds(),
            callback: Arc::new(group),
        })
    }
}
#[test]
fn parent_opacity_is_applied_once_to_complete_group_and_disabled_action_once_inside() {
    let ctx = context();
    let recorder = Arc::new(Collector(Mutex::new(vec![])));
    let _installation = compositing::install(&ctx, recorder.clone()).unwrap();
    ctx.run_ui(input(500.0, vec![]), |ui| {
        ui.set_opacity(0.6);
        let theme = Theme::light();
        let out = Item::new(&theme, "opacity")
            .title("Title")
            .action(ItemAction::new("disabled", Button::new(&theme, "Disabled")).enabled(false))
            .register(ui);
        assert!(!out.actions[0].response.enabled());
        out.paint(ui.painter()).unwrap();
    })
    .drop_without_applying_deltas();
    let groups = recorder.0.lock().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].opacity(), 0.6);
    assert!(
        groups[0]
            .items()
            .iter()
            .any(|item| matches!(item,Drawing::Group(g) if g.opacity()==0.5))
    );
}
#[test]
fn visible_item_requires_compositor_but_hidden_item_does_not() {
    let ctx = context();
    ctx.run_ui(input(500.0, vec![]), |ui| {
        let theme = Theme::light();
        assert!(matches!(
            Item::new(&theme, "visible")
                .title("Title")
                .register(ui)
                .paint(ui.painter()),
            Err(compositing::Error::MissingHost)
        ));
        ui.set_invisible();
        assert!(
            Item::new(&theme, "hidden")
                .title("Title")
                .register(ui)
                .paint(ui.painter())
                .is_ok()
        );
    })
    .drop_without_applying_deltas();
}
struct InputHarness {
    ctx: egui::Context,
    counts: [usize; 2],
    disabled: bool,
    hidden: bool,
    ids: Vec<egui::Id>,
    rects: Vec<egui::Rect>,
}
impl InputHarness {
    fn new() -> Self {
        let ctx = context();
        ctx.options_mut(|o| o.ordered_input = true);
        Self {
            ctx,
            counts: [0; 2],
            disabled: false,
            hidden: false,
            ids: vec![],
            rects: vec![],
        }
    }
    fn frame(&mut self, events: Vec<egui::Event>) {
        let ctx = self.ctx.clone();
        ctx.run_ui(input(500.0, events), |ui| {
            ctx.begin_input();
            let theme = Theme::light();
            let mut actions = if self.hidden {
                vec![]
            } else {
                Item::new(&theme, "input")
                    .title("Passive row")
                    .action(ItemAction::new("first", Button::new(&theme, "Open")))
                    .action(
                        ItemAction::icon(
                            "second",
                            IconButton::new(
                                &theme,
                                (egui::TextureId::User(1), egui::vec2(16., 16.)),
                                "Other action",
                            ),
                        )
                        .enabled(!self.disabled),
                    )
                    .register(ui)
                    .actions
            };
            for old in &self.ids {
                if !actions.iter().any(|a| a.response.id == *old) {
                    ctx.remove_input_widget(*old);
                }
            }
            self.ids = actions.iter().map(|a| a.response.id).collect();
            self.rects = actions.iter().map(|a| a.response.rect).collect();
            while let Some(event) = ctx.next_input_event() {
                for (index, button) in actions.iter_mut().enumerate() {
                    button.cancel_unfocused();
                    if event.target == Some(button.response.id) && button.handle(&event).clicked() {
                        self.counts[index] += 1;
                    }
                }
            }
        })
        .drop_without_applying_deltas();
    }
}
fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn key(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}
#[test]
fn ordered_actions_activate_once_and_disabled_removed_or_blurred_actions_do_not() {
    let mut h = InputHarness::new();
    h.frame(vec![]);
    let [a, b] = [h.rects[0].center(), h.rects[1].center()];
    h.frame(vec![
        egui::Event::PointerMoved(a),
        pointer(a, true),
        pointer(a, false),
        egui::Event::PointerMoved(b),
        pointer(b, true),
        pointer(b, false),
    ]);
    assert_eq!(h.counts, [1, 1]);
    h.frame(vec![]);
    assert_eq!(h.counts, [1, 1]);
    h.disabled = true;
    h.frame(vec![]);
    h.frame(vec![
        egui::Event::PointerMoved(b),
        pointer(b, true),
        pointer(b, false),
    ]);
    assert_eq!(h.counts, [1, 1]);
    h.ctx.memory_mut(|m| m.request_focus(h.ids[0]));
    h.frame(vec![key(egui::Key::Space, true)]);
    h.frame(vec![key(egui::Key::Space, false)]);
    assert_eq!(h.counts, [2, 1]);
    h.frame(vec![key(egui::Key::Space, true)]);
    h.frame(vec![
        egui::Event::WindowFocused(false),
        key(egui::Key::Space, false),
    ]);
    assert_eq!(h.counts, [2, 1]);
    h.frame(vec![
        egui::Event::WindowFocused(true),
        egui::Event::PointerMoved(a),
        pointer(a, true),
    ]);
    h.hidden = true;
    h.frame(vec![pointer(a, false)]);
    h.hidden = false;
    h.frame(vec![]);
    assert_eq!(h.counts, [2, 1]);
}

#[test]
fn balancing_preserves_unbreakable_words_and_empty_copy_has_no_line_box() {
    let ctx = context();
    ctx.run_ui(input(180.0, vec![]), |ui| {
        let theme = Theme::light();
        let out = Item::new(&theme, "word")
            .description("UnbreakableProjectIdentifier a")
            .register(ui);
        let group = out.drawing.unwrap();
        let description = texts(&group)[0];
        assert_eq!(description.galley.rows.len(), 2);
        assert_eq!(
            description.galley.rows[0].text().trim(),
            "UnbreakableProjectIdentifier"
        );
        assert_eq!(description.galley.rows[1].text(), "a");
        let repeated = Item::new(&theme, "repeated")
            .description("office office office")
            .register(ui);
        let group = repeated.drawing.unwrap();
        let repeated = texts(&group)[0];
        assert!(
            repeated
                .galley
                .rows
                .iter()
                .all(|row| !row.text().trim().is_empty())
        );
        assert!(!repeated.galley.elided);
        assert_eq!(
            repeated
                .galley
                .rows
                .iter()
                .map(|row| row.text())
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>(),
            vec!["office", "office", "office"]
        );
        let empty = Item::new(&theme, "empty").title("").register(ui);
        assert_eq!(empty.response.rect.height(), 34.0);
        let group = empty.drawing.unwrap();
        assert!(
            texts(&group)
                .iter()
                .all(|shape| shape.galley.rows.is_empty())
        );
    })
    .drop_without_applying_deltas();
}

#[test]
fn surface_colors_transition_for_100ms_without_idle_or_clipped_repaint() {
    let ctx = context();
    let sample = |time, variant, clipped| {
        let mut colors = None;
        let mut raw = input(500.0, vec![]);
        raw.time = Some(time);
        let full = ctx.run_ui(raw, |ui| {
            if clipped {
                ui.set_clip_rect(egui::Rect::NOTHING);
            }
            let theme = Theme::light();
            let out = Item::new(&theme, "transition")
                .title("Title")
                .variant(variant)
                .register(ui);
            if let Some(group) = out.drawing {
                let rects = group
                    .items()
                    .iter()
                    .filter_map(|item| {
                        if let Drawing::Shape(shape) = item {
                            if let egui::Shape::Rect(rect) = &shape.shape {
                                return Some(rect);
                            }
                            None
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                colors = Some((rects[0].fill, rects[1].stroke.color));
            }
        });
        let delay = full.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        full.drop_without_applying_deltas();
        (colors, delay)
    };
    for time in [0.0, 0.1, 0.2] {
        sample(time, ItemVariant::Default, false);
    }
    let start = sample(1.0, ItemVariant::Muted, false);
    assert_eq!(start.0.unwrap().0, egui::Color32::TRANSPARENT);
    assert_eq!(start.1, std::time::Duration::ZERO);
    let middle = sample(1.05, ItemVariant::Muted, false).0.unwrap().0;
    let target = Theme::light().colors.muted.background.gamma_multiply(0.5);
    assert!(middle.a() > 0 && middle.a() < target.a());
    // Retarget from the current visual color rather than restarting at an old endpoint.
    assert_eq!(
        sample(1.05, ItemVariant::Outline, false).0.unwrap().0,
        middle
    );
    let done = sample(1.16, ItemVariant::Outline, false).0.unwrap();
    assert_eq!(
        done,
        (egui::Color32::TRANSPARENT, Theme::light().colors.border)
    );
    sample(1.2, ItemVariant::Outline, false);
    assert_ne!(
        sample(1.3, ItemVariant::Outline, false).1,
        std::time::Duration::ZERO
    );
    sample(2.0, ItemVariant::Muted, true);
    assert_ne!(
        sample(2.01, ItemVariant::Muted, true).1,
        std::time::Duration::ZERO
    );
    assert_eq!(sample(2.2, ItemVariant::Muted, false).0.unwrap().0, target);
    for (time, variant) in [
        (3.0, ItemVariant::Default),
        (3.01, ItemVariant::Muted),
        (3.02, ItemVariant::Muted),
    ] {
        let mut raw = input(500.0, vec![]);
        raw.time = Some(time);
        let output = ctx.run_ui(raw, |ui| {
            ui.set_opacity(0.0);
            let theme = Theme::light();
            let _ = Item::new(&theme, "zero-opacity")
                .title("Title")
                .variant(variant)
                .register(ui);
        });
        assert_ne!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::ZERO
        );
        output.drop_without_applying_deltas();
    }
    assert_eq!(
        sample(3.2, ItemVariant::Default, false).0.unwrap().0,
        egui::Color32::TRANSPARENT
    );
}

#[test]
fn returned_ids_allow_complete_same_batch_retirement() {
    let ctx = context();
    ctx.options_mut(|options| options.ordered_input = true);
    ctx.run_ui(input(500.0, vec![key(egui::Key::Tab, true)]), |ui| {
        ctx.begin_input();
        let theme = Theme::light();
        let out = Item::new(&theme, "retire")
            .title("Title")
            .description("Description")
            .action(ItemAction::new("first", Button::new(&theme, "Open")))
            .action(ItemAction::new("second", Button::new(&theme, "Archive")))
            .register(ui);
        assert_eq!(out.ids.len(), 7);
        assert!(out.ids.iter().all(|id| ctx.read_response(*id).is_some()));
        for id in &out.ids {
            ctx.remove_input_widget(*id);
        }
        while let Some(event) = ctx.next_input_event() {
            assert!(!event.target.is_some_and(|id| out.ids.contains(&id)));
        }
        assert!(ctx.memory(|memory| memory.focused()).is_none());
    })
    .drop_without_applying_deltas();
}
