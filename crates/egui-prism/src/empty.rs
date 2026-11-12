//! Caller-owned empty-state copy and actions, without loading or error policy.
//!
//! ```
//! use egui_prism::{empty::Empty, egui, theme::Theme};
//! egui::__run_test_ui(|ui| {
//!     let theme = Theme::light();
//!     let output = Empty::new(&theme, "projects-empty")
//!         .title("No projects yet")
//!         .description("Create a project to get started.")
//!         .show(ui, |_| {});
//!     // Submit output.drawing through the host compositor in this frame.
//! });
//! ```

use std::sync::Arc;

use egui::{
    Align, Color32, FontId, IdSalt, Image, Layout, Rect, Response, Sense, Shape, TextStyle, Ui,
    UiBuilder, pos2, vec2,
};

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};

/// Borrowed copy and an optional decorative icon for a passive empty state.
///
/// The caller owns meaning, actions, image resources and stable identity. This
/// component retains no application state and never starts loading or retries.
#[must_use = "call show and submit the returned drawing"]
pub struct Empty<'theme, 'image> {
    theme: &'theme Theme,
    id_salt: IdSalt,
    title: Option<&'theme str>,
    description: Option<&'theme str>,
    icon: Option<Image<'image>>,
}

impl<'theme, 'image> Empty<'theme, 'image> {
    /// Create an empty composition with an identity stable across copy changes.
    pub fn new(theme: &'theme Theme, id_salt: impl egui::AsIdSalt) -> Self {
        Self {
            theme,
            id_salt: IdSalt::new(id_salt),
            title: None,
            description: None,
            icon: None,
        }
    }

    /// Set centered, wrapping 18-point medium text with tight letter spacing.
    pub fn title(mut self, title: &'theme str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set centered, wrapping 14-point plain text with relaxed line spacing.
    pub fn description(mut self, description: &'theme str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set a decorative 24-point image in the reference's 40-point muted icon tile.
    pub fn icon(mut self, icon: impl Into<Image<'image>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Lay out the header and caller content once, without submitting drawing.
    ///
    /// The root uses finite available width and natural height. Padding is 24
    /// points below a 768-point viewport, 48 above it, independent of container
    /// width. Header and content are capped at 384 points. An unused content UI
    /// is omitted, including its gap. Overflow keeps the parent's clip.
    ///
    /// Plain text collapses CSS whitespace and balances space-delimited lines.
    /// Browser punctuation/CJK line breaking and exact CSS balancing are not
    /// guaranteed; see the component's parity evidence before claiming a match.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut EmptyUi<'_>) -> R) -> EmptyOutput<R> {
        let width = ui.available_width();
        assert!(width.is_finite(), "Empty requires a finite available width");
        let width = width.max(0.0);
        let padding = if ui.ctx().viewport_rect().width() >= 768.0 {
            48.0
        } else {
            24.0
        };
        let column_width = (width - 2.0 * padding).clamp(0.0, 384.0);
        let origin = ui.available_rect_before_wrap().min;
        let center_x = origin.x + width * 0.5;
        let id = ui.make_persistent_id(self.id_salt);
        let slot = ui.painter().add(Shape::Noop);
        let mut items = Vec::new();
        let mut y = origin.y + padding;
        let mut header = false;

        if let Some(mut icon) = self.icon {
            let rect = Rect::from_center_size(pos2(center_x, y + 20.0), vec2(40.0, 40.0));
            push_shape(
                &mut items,
                Shape::rect_filled(rect, 10, self.theme.colors.muted.background),
                ui.clip_rect(),
            );
            if icon.image_options().tint == Color32::WHITE {
                icon = icon.tint(self.theme.colors.background.foreground);
            }
            let icon_rect = rect.shrink(8.0);
            for shape in crate::button::image_shapes(ui, &icon, icon_rect) {
                push_shape(&mut items, shape, ui.clip_rect().intersect(icon_rect));
            }
            // EmptyMedia's mb-2 is independent of EmptyHeader's gap-2.
            y += 48.0;
            header = true;
        }
        for (key, text, title) in [
            ("title", self.title, true),
            ("description", self.description, false),
        ] {
            let Some(text) = text else { continue };
            if header {
                y += 8.0;
            }
            let color = if title {
                self.theme.colors.background.foreground
            } else {
                self.theme.colors.muted.foreground
            };
            let galley = layout_text(ui, text, column_width, title, color);
            let height = if galley.job.text.is_empty() {
                0.0
            } else {
                galley.size().y
            };
            let rect = Rect::from_min_size(
                pos2(center_x - galley.size().x * 0.5, y),
                vec2(galley.size().x, height),
            );
            ui.interact(rect, id.with(key), Sense::hover())
                .widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), text)
                });
            if height > 0.0 {
                let text_pos = pos2(center_x - galley.rect.center().x, y);
                push_shape(
                    &mut items,
                    Shape::galley(text_pos, galley, color),
                    ui.clip_rect(),
                );
            }
            y += height;
            header = true;
        }

        let content_top = y + if header { 24.0 } else { 0.0 };
        let mut child = ui.new_child(
            UiBuilder::new()
                .scope_id(id.with("content"))
                .max_rect(Rect::from_min_size(
                    pos2(center_x - column_width * 0.5, content_top),
                    vec2(column_width, f32::INFINITY),
                ))
                .layout(Layout::top_down(Align::Center)),
        );
        child.spacing_mut().item_spacing.y = 16.0;
        child.visuals_mut().override_text_color = Some(self.theme.colors.background.foreground);
        child.style_mut().text_styles.insert(
            TextStyle::Body,
            FontId::new(14.0, TextStyle::Body.resolve(ui.style()).family),
        );
        let mut content = EmptyUi {
            ui: &mut child,
            items: &mut items,
        };
        let inner = add(&mut content);
        if child.min_size() != egui::Vec2::ZERO {
            y = content_top + child.min_rect().height();
        }
        let rect = Rect::from_min_size(origin, vec2(width, y - origin.y + padding));
        ui.advance_cursor_after_rect(rect);
        let response = ui.interact(rect, id, Sense::hover());
        let mut bounds = Rect::NOTHING;
        for item in &items {
            bounds = bounds.union(match item {
                Item::Group(group) => group.bounds(),
                Item::Shape(shape) => shape
                    .shape
                    .visual_bounding_rect()
                    .expand(1.0)
                    .intersect(shape.clip_rect),
            });
        }
        let bounds = bounds.intersect(ui.clip_rect());
        let drawing = bounds
            .is_positive()
            .then(|| Group::new(bounds, 1.0, items).expect("finite Empty drawing"));
        EmptyOutput {
            inner,
            response,
            drawing,
            slot,
        }
    }
}

fn push_shape(items: &mut Vec<Item>, shape: Shape, clip: Rect) {
    let clip = shape.visual_bounding_rect().expand(1.0).intersect(clip);
    if clip.is_positive() {
        items.push(Item::Shape(egui::epaint::ClippedShape::new(clip, shape)));
    }
}

fn layout_text(ui: &Ui, text: &str, width: f32, title: bool, color: Color32) -> Arc<egui::Galley> {
    let text = text
        .split([' ', '\t', '\n', '\r', '\u{000c}'])
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let style = if title {
        TextStyle::Button
    } else {
        TextStyle::Body
    };
    let mut job = egui::text::LayoutJob {
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: egui::text::ByteIndex(0)..egui::text::ByteIndex(text.len()),
            format: egui::TextFormat {
                font_id: FontId::new(
                    if title { 18.0 } else { 14.0 },
                    style.resolve(ui.style()).family,
                ),
                line_height: Some(if title { 28.0 } else { 22.75 }),
                extra_letter_spacing: if title { -0.45 } else { 0.0 },
                color,
                ..Default::default()
            },
        }],
        text,
        wrap: egui::text::TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        },
        break_on_newline: false,
        halign: Align::Center,
        ..Default::default()
    };
    let natural = ui.painter().layout_job(job.clone());
    // Keep words and NBSP runs intact, including when the parent is narrower.
    // egui considers overflow before accepting the current space as a break.
    // Include each run's closing space so it cannot spill onto an empty row;
    // centered layout excludes that trailing whitespace from visible bounds.
    let min_content = natural
        .rows
        .iter()
        .flat_map(|row| {
            row.glyphs
                .split_inclusive(|glyph| glyph.chr == ' ')
                .filter_map(|run| Some(run.last()?.max_x() - run.first()?.pos.x))
        })
        .fold(0.0, f32::max);
    job.wrap.max_width = width.max(min_content);
    let mut galley = ui.painter().layout_job(job.clone());
    let lines = galley.rows.len();
    if lines > 1 {
        // Find the narrowest wrap width retaining the initial line count. This
        // balances short copy without quadratic partitioning or truncating long copy.
        let mut low = min_content;
        let mut high = job.wrap.max_width;
        while high - low > 0.25 {
            let mid = (low + high) * 0.5;
            job.wrap.max_width = mid;
            let candidate = ui.painter().layout_job(job.clone());
            if candidate.rows.len() <= lines {
                high = mid;
                galley = candidate;
            } else {
                low = mid;
            }
        }
    }
    galley
}

/// Caller content using ordinary egui layout and explicit child drawing groups.
pub struct EmptyUi<'a> {
    ui: &'a mut Ui,
    items: &'a mut Vec<Item>,
}

impl EmptyUi<'_> {
    /// Borrow the centered content UI for caller-owned layout and input.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// Include an unfaded child group in this composition's opacity isolation.
    ///
    /// Keep referenced textures alive through rendering. Ordinary egui painting
    /// and arbitrary GPU callbacks remain outside the group, above its drawing.
    pub fn draw(&mut self, group: Group) {
        self.items.push(Item::Group(group));
    }
}

/// Caller results, passive bounds and a frame-local drawing with no state semantics.
#[must_use = "paint the output or include its drawing in a parent Group"]
pub struct EmptyOutput<R> {
    /// The value returned by the caller's content closure.
    pub inner: R,
    /// A hover-only response, never a click, drag or keyboard-focus target.
    pub response: Response,
    /// Unfaded copy, icon and explicit child groups; absent without visible drawing.
    pub drawing: Option<Group>,
    slot: egui::layers::ShapeIdx,
}

impl<R> EmptyOutput<R> {
    /// Submit behind direct child painting using a same-context, same-layer painter.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::InnerResponse<R>, Error> {
        assert_eq!(&self.response.ctx, painter.ctx(), "Empty painter context");
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Empty painter layer"
        );
        if let Some(group) = self.drawing {
            let marker = painter.add(Shape::Noop);
            compositing::paint(painter, group)?;
            painter.ctx().graphics_mut(|graphics| {
                let list = graphics.entry(painter.layer_id());
                let callback = list
                    .all_entries()
                    .nth(marker.0 + 1)
                    .cloned()
                    .expect("submitted Empty callback");
                list.reset_shape(egui::layers::ShapeIdx(marker.0 + 1));
                list.mutate_shape(self.slot, |shape| *shape = callback);
            });
        }
        Ok(egui::InnerResponse::new(self.inner, self.response))
    }
}
