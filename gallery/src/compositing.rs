use egui_prism::{
    compositing::{self, Group, Item},
    egui::{self, epaint::ClippedShape},
    theme::Theme,
};
use std::sync::{Arc, Mutex};

pub(super) struct CompositingCase {
    image: egui::TextureHandle,
    opacity: f32,
    parent: f32,
    clip: bool,
    result: Arc<Mutex<Option<Result<usize, String>>>>,
    pending: bool,
    automatic: bool,
    stage: usize,
    passed: usize,
    restarted: bool,
    reference_frames: Arc<Mutex<ReferenceFrames>>,
    error: Option<String>,
    announced: bool,
}

const OPACITIES: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
const BACKGROUNDS: [egui::Color32; 4] = [
    egui::Color32::WHITE,
    egui::Color32::from_rgb(24, 24, 27),
    egui::Color32::from_rgb(74, 143, 198),
    egui::Color32::from_rgb(230, 180, 90),
];

impl CompositingCase {
    pub(super) fn new(context: &egui::Context, automatic: bool) -> Self {
        let mut image = egui::ColorImage::filled([16, 16], egui::Color32::WHITE);
        for y in 0..16 {
            for x in 8..16 {
                image[(x, y)] = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 128);
            }
        }
        Self {
            image: context.load_texture("compositing-source", image, egui::TextureOptions::NEAREST),
            opacity: 0.5,
            parent: 1.0,
            clip: false,
            result: Arc::default(),
            pending: false,
            automatic,
            stage: 0,
            passed: 0,
            restarted: false,
            reference_frames: Arc::default(),
            error: None,
            announced: false,
        }
    }

    pub(super) fn take_restart_request(&mut self) -> bool {
        if self.automatic && self.stage == 1 && !self.restarted {
            self.restarted = true;
            true
        } else {
            false
        }
    }
    pub(super) fn automatic(&self) -> bool {
        self.automatic
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        if let Some(result) = self.result.lock().expect("capture result").take() {
            self.pending = false;
            match result {
                Ok(count) => {
                    self.passed += count;
                    self.stage += 1;
                }
                Err(error) => {
                    if self.automatic {
                        panic!("framebuffer regression: {error}");
                    }
                    self.error = Some(error);
                }
            }
        }
        let requested = if self.automatic {
            ui.label("Final framebuffer conformance: images, layers, clips, rings, shadows and resource restart");
            false
        } else {
            ui.heading("Host compositing");
            ui.label("Independent group fixture. Component adoption is tracked separately.");
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::Slider::new(&mut self.opacity, 0.0..=1.0).text("Group opacity"));
                ui.add(egui::Slider::new(&mut self.parent, 0.0..=1.0).text("Parent opacity"));
                ui.checkbox(&mut self.clip, "Clip right edge");
            });
            ui.button("Check visible framebuffer matrix").clicked()
        };
        ui.label(format!("Framebuffer samples passed: {}", self.passed));
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
        }
        let stage_zoom = if self.stage < 3 { 1.0 } else { 1.5 };
        if self.automatic && self.stage < 6 && ui.ctx().zoom_factor() != stage_zoom {
            ui.ctx().set_zoom_factor(stage_zoom);
            ui.ctx().request_repaint();
            return;
        }
        let transformed = self.automatic && self.stage % 3 == 2;
        let transform = if transformed {
            egui::emath::TSTransform::new(egui::vec2(7.0, 5.0), 0.75)
        } else {
            egui::emath::TSTransform::IDENTITY
        };
        let layer = egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::unique("compositing-conformance"),
        );
        ui.ctx().set_transform_layer(layer, transform);
        let painter = egui::Painter::new(ui.ctx().clone(), layer, ui.clip_rect());
        let mut probes = Vec::new();
        if self.restarted {
            probes.push((
                egui::pos2(5.0, 5.0),
                [255, 255, 255],
                "revoked callback after host restart".into(),
            ));
        }
        for (row, background) in BACKGROUNDS.into_iter().enumerate() {
            ui.horizontal(|ui| {
                for opacity in OPACITIES {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(80.0, 56.0), egui::Sense::hover());
                    painter.rect_filled(rect, 0, background);
                    let inside = rect.shrink(8.0);
                    let black = egui::Color32::from_rgb(24, 24, 27);
                    let mut mesh = egui::Mesh::with_texture(self.image.id());
                    let image_rect = egui::Rect::from_min_size(
                        inside.min + egui::vec2(8.0, 8.0),
                        egui::vec2(32.0, 16.0),
                    );
                    mesh.add_rect_with_uv(
                        image_rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    let items = vec![
                        item(rect, egui::Shape::rect_filled(inside, 0, black)),
                        item(rect, mesh.into()),
                    ];
                    let group = Group::new(rect, opacity, items).expect("fixture group");
                    let red = egui::Color32::from_rgba_unmultiplied(200, 30, 80, 102);
                    let group = if transformed {
                        Group::new(
                            rect,
                            0.5,
                            vec![
                                item(rect, egui::Shape::rect_filled(inside, 0, red)),
                                Item::Group(group),
                            ],
                        )
                        .expect("nested fixture")
                    } else {
                        group
                    };
                    let clipped = self.automatic && self.stage == 5;
                    let mut group_painter = if clipped {
                        painter.with_clip_rect(egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.min.x + 32.0, rect.max.y),
                        ))
                    } else {
                        painter.clone()
                    };
                    if transformed {
                        group_painter.set_opacity(0.8);
                    }
                    compositing::paint(&group_painter, group).expect("installed compositor");
                    for (point, alpha) in [
                        (image_rect.min + egui::vec2(4.0, 8.0), 1.0),
                        (image_rect.min + egui::vec2(24.0, 8.0), 128.0 / 255.0),
                    ] {
                        let expected = std::array::from_fn::<_, 3, _>(|i| {
                            if clipped && point.x > rect.min.x + 32.0 {
                                return background[i];
                            }
                            let assembled =
                                (255.0 * alpha + f32::from(black[i]) * (1.0 - alpha)).round();
                            let (color, alpha) = if transformed {
                                (
                                    (assembled * opacity + f32::from(red[i]) * (1.0 - opacity))
                                        * 0.4,
                                    (opacity + 0.4 * (1.0 - opacity)) * 0.4,
                                )
                            } else {
                                (assembled * opacity, opacity)
                            };
                            (color + f32::from(background[i]) * (1.0 - alpha)).round() as u8
                        });
                        probes.push((
                            transform * point,
                            expected,
                            format!("row {row}, opacity {opacity}, image alpha {alpha}"),
                        ));
                    }
                    probes.push((
                        transform * (rect.min + egui::vec2(2.0, 2.0)),
                        [background.r(), background.g(), background.b()],
                        "outside group".into(),
                    ));
                }
            });
        }
        ui.label("Isolated group / per-shape alpha. Each fixture includes all six role palettes, rings, shadows and images.");
        let opacity = if self.automatic {
            [0.0, 1.0, 0.5][self.stage.min(5) % 3]
        } else {
            self.opacity
        };
        let decoration =
            draw_comparison(ui, theme, self.image.id(), opacity, self.parent, self.clip);
        let done = self.automatic && self.stage >= 6;
        if done {
            ui.colored_label(
                egui::Color32::GREEN,
                format!(
                    "COMPOSITING PASS: {} final framebuffer samples",
                    self.passed
                ),
            );
            {
                if !self.announced {
                    println!(
                        "COMPOSITING PASS: {} final framebuffer samples",
                        self.passed
                    );
                }
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            self.announced = true;
        } else if !self.pending
            && (requested || (self.automatic && (ui.ctx().zoom_factor() - stage_zoom).abs() < 0.01))
        {
            self.pending = true;
            let reference_frames = self.reference_frames.clone();
            let automatic = self.automatic;
            let stage = self.stage;
            let result = self.result.clone();
            let ctx = ui.ctx().clone();
            let ppp = ctx.pixels_per_point();
            ui.ctx().request_screenshot(move |image| {
                let outcome = probes
                    .iter()
                    .try_for_each(|(point, expected, label)| {
                        let x = (point.x * ppp).round() as usize;
                        let y = (point.y * ppp).round() as usize;
                        if x >= image.size[0] || y >= image.size[1] {
                            return Err(format!("{label}: probe outside capture"));
                        }
                        let actual = image[(x, y)];
                        if (0..3).any(|i| actual[i].abs_diff(expected[i]) > 2) {
                            Err(format!(
                                "{label}: expected {expected:?}, got {actual:?} at {x},{y}"
                            ))
                        } else {
                            Ok(())
                        }
                    })
                    .and_then(|()| {
                        let extra = if automatic {
                            verify_decoration(
                                &image,
                                decoration,
                                ppp,
                                stage,
                                &mut reference_frames.lock().expect("reference frames"),
                            )?
                        } else {
                            0
                        };
                        Ok(probes.len() + extra)
                    });
                *result.lock().expect("capture result") = Some(outcome);
                ctx.request_repaint();
            });
        }
    }
}
fn item(clip_rect: egui::Rect, shape: egui::Shape) -> Item {
    Item::Shape(ClippedShape::new(clip_rect, shape))
}

#[derive(Default)]
struct ReferenceFrames {
    background: Vec<egui::Color32>,
    assembled: Vec<egui::Color32>,
    size: [usize; 2],
}

fn verify_decoration(
    image: &egui::ColorImage,
    rects: [egui::Rect; 2],
    ppp: f32,
    stage: usize,
    frames: &mut ReferenceFrames,
) -> Result<usize, String> {
    let starts = rects.map(|rect| {
        [
            (rect.min.x * ppp).round() as usize,
            (rect.min.y * ppp).round() as usize,
        ]
    });
    let size = [
        (rects[0].width() * ppp).round() as usize,
        (rects[0].height() * ppp).round() as usize,
    ];
    if starts
        .iter()
        .any(|start| start[0] + size[0] > image.size[0] || start[1] + size[1] > image.size[1])
    {
        return Err(
            "decorated fixture must fit in the framebuffer; enlarge the test viewport".into(),
        );
    }
    let read = |start: [usize; 2]| {
        (0..size[1])
            .flat_map(|y| (0..size[0]).map(move |x| image[(start[0] + x, start[1] + y)]))
            .collect::<Vec<_>>()
    };
    let actual = read(starts[0]);
    match stage % 3 {
        0 => {
            frames.background = actual;
            frames.size = size;
            Ok(0)
        }
        1 => {
            frames.assembled = actual;
            Ok(0)
        }
        _ => {
            if frames.size != size
                || frames.background.len() != actual.len()
                || frames.assembled.len() != actual.len()
            {
                return Err("reference framebuffer dimensions changed within a comparison".into());
            }
            let legacy = read(starts[1]);
            let mut differences = 0;
            for (index, actual) in actual.iter().enumerate() {
                for channel in 0..3 {
                    let expected = (f32::from(frames.background[index][channel]) * 0.5
                        + f32::from(frames.assembled[index][channel]) * 0.5)
                        .round() as u8;
                    if actual[channel].abs_diff(expected) > 2 {
                        return Err(format!(
                            "decorated pixel {index}, channel {channel}: isolated {} versus flattened reference {expected}",
                            actual[channel]
                        ));
                    }
                    if legacy[index][channel].abs_diff(expected) > 2 {
                        differences += 1;
                    }
                }
            }
            if differences == 0 {
                return Err("negative control did not distinguish per-shape opacity".into());
            }
            Ok(actual.len())
        }
    }
}

fn draw_comparison(
    ui: &mut egui::Ui,
    theme: &Theme,
    texture: egui::TextureId,
    opacity: f32,
    parent: f32,
    clipped: bool,
) -> [egui::Rect; 2] {
    let mut rects = [egui::Rect::NOTHING; 2];
    ui.horizontal(|ui| {
        for (index, isolated) in [true, false].into_iter().enumerate() {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(280.0, 144.0), egui::Sense::hover());
            rects[index] = rect;
            for y in 0..9 {
                for x in 0..18 {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            rect.min + egui::vec2(x as f32 * 16.0, y as f32 * 16.0),
                            egui::vec2(16.0, 16.0),
                        )
                        .intersect(rect),
                        0,
                        if (x + y) % 2 == 0 {
                            egui::Color32::from_rgb(210, 220, 240)
                        } else {
                            egui::Color32::from_rgb(150, 170, 200)
                        },
                    );
                }
            }
            let clip = if clipped {
                egui::Rect::from_min_max(rect.min, rect.max - egui::vec2(60.0, 0.0))
            } else {
                rect
            };
            let roles = [
                theme.colors.primary,
                theme.colors.secondary,
                theme.colors.surface,
                egui_prism::theme::ColorPair {
                    background: egui::Color32::TRANSPARENT,
                    foreground: theme.colors.background.foreground,
                },
                theme.colors.destructive,
                egui_prism::theme::ColorPair {
                    background: egui::Color32::TRANSPARENT,
                    foreground: theme.colors.primary.background,
                },
            ];
            let mut children = Vec::new();
            for (role, pair) in roles.into_iter().enumerate() {
                let card = egui::Rect::from_min_size(
                    rect.min
                        + egui::vec2(
                            16.0 + (role % 3) as f32 * 84.0,
                            20.0 + (role / 3) as f32 * 68.0,
                        ),
                    egui::vec2(72.0, 36.0),
                );
                let mut mesh = egui::Mesh::with_texture(texture);
                mesh.add_rect_with_uv(
                    egui::Rect::from_min_size(
                        card.min + egui::vec2(6.0, 10.0),
                        egui::vec2(32.0, 16.0),
                    ),
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    pair.foreground,
                );
                let text = ui.painter().layout_no_wrap(
                    "Aa".into(),
                    egui::FontId::new(14.0, egui::FontFamily::Name("prism-geist-medium".into())),
                    pair.foreground,
                );
                let shapes = vec![
                    egui::Shadow {
                        offset: [0, 2],
                        blur: 8,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(90),
                    }
                    .as_shape(card, 6)
                    .into(),
                    egui::Shape::rect_stroke(
                        card.expand(3.0),
                        9,
                        egui::Stroke::new(3.0, theme.colors.ring),
                        egui::StrokeKind::Outside,
                    ),
                    egui::Shape::rect_filled(card, 6, pair.background),
                    egui::Shape::rect_stroke(
                        card,
                        6,
                        egui::Stroke::new(1.0, theme.colors.border),
                        egui::StrokeKind::Inside,
                    ),
                    mesh.into(),
                    egui::Shape::galley(card.min + egui::vec2(42.0, 9.0), text, pair.foreground),
                ];
                if isolated {
                    children.push(Item::Group(
                        Group::new(
                            card.expand(12.0),
                            1.0,
                            shapes.into_iter().map(|shape| item(clip, shape)).collect(),
                        )
                        .expect("role group"),
                    ));
                } else {
                    let mut painter = ui.painter().with_clip_rect(clip);
                    painter.set_opacity(opacity * parent);
                    painter.extend(shapes);
                }
            }
            if isolated {
                let child = Group::new(rect, opacity, children).expect("roles");
                let group =
                    Group::new(rect, parent, vec![Item::Group(child)]).expect("parent group");
                compositing::paint(ui.painter(), group).expect("installed compositor");
            }
        }
    });
    rects
}
