//! Caller-owned Card sections, with explicit drawing and ordinary egui child layout.
//!
//! ```
//! use egui_prism::{card::Card, compositing, egui, theme::Theme};
//! fn panel(ui: &mut egui::Ui, theme: &Theme) -> Result<(), compositing::Error> {
//!     let painter = ui.painter().clone();
//!     Card::new(theme).show(ui, |card| {
//!         card.header("Project", Some("Caller-owned content and actions."));
//!         card.content(|body| { body.ui().label("No application state is stored in Card."); });
//!         card.footer(20.0, |footer| { footer.ui().label("Ready"); });
//!     }).paint(&painter)?;
//!     Ok(())
//! }
//! ```

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};
use egui::emath::GuiRounding as _;
use egui::{
    Align, FontId, Layout, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextStyle, Ui,
    UiBuilder, pos2, vec2,
};

/// A borrowed theme for a noninteractive, full-available-width Card.
pub struct Card<'a> {
    theme: &'a Theme,
}

impl<'a> Card<'a> {
    /// Create a card without retaining application or layout state.
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    /// Lay out sections once and prepare their drawing, without submitting it.
    ///
    /// Width must be finite. The minimum border box is two points wide; callers
    /// constrain wider cards with an ordinary egui parent UI. Children may
    /// overflow that box, as in CSS; the parent's clip remains authoritative.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut CardUi<'_>) -> R) -> CardOutput<R> {
        let width = ui.available_width();
        assert!(width.is_finite(), "Card requires a finite available width");
        let width = width.max(2.0);
        let origin = ui.available_rect_before_wrap().min;
        let slot = ui.painter().add(Shape::Noop);
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    origin + vec2(25.0, 25.0),
                    vec2((width - 50.0).max(0.0), f32::INFINITY),
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        child.spacing_mut().item_spacing.y = 24.0;
        let mut card = CardUi {
            ui: &mut child,
            theme: self.theme,
            spacing: ui.spacing().item_spacing,
            width: (width - 50.0).max(0.0),
            items: Vec::new(),
        };
        let inner = add(&mut card);
        let mut items = card.items;
        let rect = Rect::from_min_size(origin, vec2(width, child.min_rect().height() + 50.0));
        let id = ui.advance_cursor_after_rect(rect);
        let response = ui.interact(rect, id, Sense::hover());
        let clip = ui.clip_rect();
        let mut bounds = rect.expand(5.0);
        for item in &items {
            let extent = match item {
                Item::Group(group) => group.bounds(),
                Item::Shape(shape) => shape
                    .shape
                    .visual_bounding_rect()
                    .intersect(shape.clip_rect),
            };
            bounds = bounds.union(extent);
        }
        let bounds = bounds.intersect(clip);
        let drawing = if bounds.is_positive() {
            let background = Shape::rect_filled(rect, 14, self.theme.colors.surface.background);
            let border = Shape::rect_stroke(
                rect,
                14,
                Stroke::new(1.0, self.theme.colors.border),
                StrokeKind::Inside,
            );
            let mut shell = vec![
                Item::Shape(egui::epaint::ClippedShape::new(
                    bounds,
                    crate::shadow::small(ui.ctx(), rect, 14.into()),
                )),
                Item::Shape(egui::epaint::ClippedShape::new(bounds, background)),
                Item::Shape(egui::epaint::ClippedShape::new(bounds, border)),
            ];
            shell.append(&mut items);
            Some(Group::new(bounds, 1.0, shell).expect("finite card drawing without callbacks"))
        } else {
            None
        };
        CardOutput {
            inner,
            response,
            drawing,
            slot,
        }
    }
}

/// Frame-local Card sections in caller-selected order, with a 24-point gap.
pub struct CardUi<'a> {
    ui: &'a mut Ui,
    theme: &'a Theme,
    items: Vec<Item>,
    spacing: egui::Vec2,
    width: f32,
}

impl CardUi<'_> {
    /// Add the pinned two-row header, with 16-point title and optional 14-point description.
    ///
    /// The explicit second grid row leaves an eight-point gap even without a
    /// description. Empty strings have no text height. Plain text uses egui's
    /// Unicode wrapping, not browser CSS line breaking.
    pub fn header(&mut self, title: &str, description: Option<&str>) {
        let start = self.ui.available_rect_before_wrap().min;
        let width = self.width;
        let mut y = start.y;
        for (text, style, size, line_height, color) in [
            (
                title,
                TextStyle::Heading,
                16.0,
                16.0,
                self.theme.colors.surface.foreground,
            ),
            (
                description.unwrap_or(""),
                TextStyle::Body,
                14.0,
                20.0,
                self.theme.colors.muted.foreground,
            ),
        ] {
            let text = text
                .split([' ', '\t', '\n', '\r', '\u{000c}'])
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            let galley = self.ui.painter().layout_job(egui::text::LayoutJob {
                text: text.clone(),
                sections: vec![egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: egui::text::ByteIndex(0)..egui::text::ByteIndex(text.len()),
                    format: egui::TextFormat {
                        font_id: FontId::new(size, style.resolve(self.ui.style()).family),
                        line_height: Some(line_height),
                        color,
                        ..Default::default()
                    },
                }],
                wrap: egui::text::TextWrapping {
                    max_width: width,
                    ..Default::default()
                },
                break_on_newline: false,
                ..Default::default()
            });
            let height = if text.is_empty() {
                0.0
            } else {
                galley.size().y
            };
            let rect = Rect::from_min_size(pos2(start.x, y), vec2(galley.size().x, height));
            let response = self
                .ui
                .interact(rect, self.ui.next_auto_id(), Sense::hover());
            self.ui.skip_ahead_auto_ids(1);
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Label, response.enabled(), &text)
            });
            if height > 0.0 {
                let shape = Shape::galley(rect.min, galley, color);
                let clip = shape
                    .visual_bounding_rect()
                    .expand(1.0)
                    .intersect(self.ui.clip_rect());
                if clip.is_positive() {
                    self.items
                        .push(Item::Shape(egui::epaint::ClippedShape::new(clip, shape)));
                }
            }
            y += height;
            if style == TextStyle::Heading {
                y += 8.0;
            }
        }
        self.ui
            .allocate_exact_size(vec2(width, y - start.y), Sense::hover());
    }

    /// Add a content section with no extra vertical padding.
    pub fn content<R>(&mut self, add: impl FnOnce(&mut CardSection<'_>) -> R) -> R {
        self.section(Layout::top_down(Align::Min), None, add)
    }

    /// Add a vertically centered footer with the caller's maximum child height.
    ///
    /// Pass zero for an empty row, or the tallest child height in logical points.
    /// Explicit height lets egui position input and drawing in one pass without
    /// replaying the caller's closure. Nonfinite, negative, or insufficient
    /// height is rejected instead of clipping or silently misaligning controls.
    pub fn footer<R>(&mut self, height: f32, add: impl FnOnce(&mut CardSection<'_>) -> R) -> R {
        assert!(
            height.is_finite() && height >= 0.0,
            "Card footer height must be finite and nonnegative"
        );
        self.section(Layout::left_to_right(Align::Center), Some(height), add)
    }

    fn section<R>(
        &mut self,
        layout: Layout,
        height: Option<f32>,
        add: impl FnOnce(&mut CardSection<'_>) -> R,
    ) -> R {
        let mut max_rect = self.ui.available_rect_before_wrap();
        max_rect.max.x = max_rect.min.x + self.width;
        if let Some(height) = height {
            max_rect.max.y = max_rect.min.y + height;
        }
        let mut ui = self
            .ui
            .new_child(UiBuilder::new().max_rect(max_rect).layout(layout));
        // Section content inherits caller spacing, not the inter-section gap.
        ui.spacing_mut().item_spacing = self.spacing;
        if layout.is_horizontal() {
            ui.spacing_mut().item_spacing.x = 0.0;
        }
        ui.visuals_mut().override_text_color = Some(self.theme.colors.surface.foreground);
        let mut section = CardSection {
            ui: &mut ui,
            items: &mut self.items,
        };
        let inner = add(&mut section);
        let used = if ui.min_size() == egui::Vec2::ZERO {
            Rect::from_min_size(max_rect.min, egui::Vec2::ZERO)
        } else {
            ui.min_rect()
        };
        let used = if let Some(height) = height {
            assert!(
                used.bottom().round_ui() <= (max_rect.top() + height).round_ui(),
                "Card footer height must fit its tallest child"
            );
            Rect::from_min_size(max_rect.min, vec2(used.width(), height))
        } else {
            used
        };
        self.ui.advance_cursor_after_rect(used);
        inner
    }
}

/// An ordinary child UI and explicit unfaded child drawings for one section.
pub struct CardSection<'a> {
    ui: &'a mut Ui,
    items: &'a mut Vec<Item>,
}

impl CardSection<'_> {
    /// Borrow the child UI for caller-owned layout, input and direct painting.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// Include an unfaded child group in the card's opacity isolation.
    ///
    /// Keep referenced textures alive through rendering. Ordinary egui painting
    /// and arbitrary GPU callbacks stay outside this group, above the card.
    pub fn draw(&mut self, group: Group) {
        self.items.push(Item::Group(group));
    }
}

/// Caller results, passive card bounds and the frame-local composited drawing.
#[must_use = "paint the card or include its drawing in a parent Group"]
pub struct CardOutput<R> {
    /// Values returned by the caller's section construction.
    pub inner: R,
    /// A hover-only response that does not absorb child activation or focus.
    pub response: Response,
    /// Unfaded shell, header and explicitly submitted child groups.
    pub drawing: Option<Group>,
    slot: egui::layers::ShapeIdx,
}

impl<R> CardOutput<R> {
    /// Submit behind ordinary child painting using a same-context, same-layer painter.
    ///
    /// Capture that painter before a disabled scope to avoid fading the passive
    /// shell. Explicit child groups retain their own disabled/opacity policy.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::InnerResponse<R>, Error> {
        assert_eq!(&self.response.ctx, painter.ctx(), "card painter context");
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "card painter layer"
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
                    .expect("submitted card callback");
                list.reset_shape(egui::layers::ShapeIdx(marker.0 + 1));
                list.mutate_shape(self.slot, |shape| *shape = callback);
            });
        }
        Ok(egui::InnerResponse::new(self.inner, self.response))
    }
}
