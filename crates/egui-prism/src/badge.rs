//! Static, caller-owned text badges with explicit drawing submission.
//!
//! Install the host's fonts with [`crate::theme::install_fonts`] to use Geist
//! Medium. Badge preserves the installed Button family but fixes its size at
//! 12 points.
//!
//! The host installs a compositor before submitting drawings:
//!
//! ```
//! use egui_prism::{
//!     badge::{Badge, BadgeVariant},
//!     compositing,
//!     egui,
//!     theme::Theme,
//! };
//!
//! fn status(ui: &mut egui::Ui, theme: &Theme) -> Result<egui::Response, compositing::Error> {
//!     // Capture before entering any disabled scope.
//!     let painter = ui.painter().clone();
//!     Badge::new(theme, "Ready")
//!         .variant(BadgeVariant::Secondary)
//!         .show(ui)
//!         .paint(&painter)
//! }
//! ```

use egui::{
    Color32, CornerRadius, FontId, Response, Sense, Shape, Stroke, StrokeKind, TextFormat,
    TextStyle, pos2, vec2,
};

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};

const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 16.0;

/// The pinned static-span appearances. None adds interaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Primary background and foreground; the upstream `default` variant.
    #[default]
    Primary,
    /// Secondary background and foreground.
    Secondary,
    /// Destructive background and white text; the dark fill has 60% opacity.
    Destructive,
    /// Transparent fill, page foreground, and semantic border.
    Outline,
    /// Transparent fill and page foreground.
    Ghost,
    /// Transparent fill and primary-colored text, without an underline or action.
    Link,
}

/// A borrowed text badge with no retained widget state.
///
/// Text never wraps or elides. Clipping changes only its visible drawing, not
/// its natural width or accessible name. Theme control dimensions and text sizes
/// do not change the badge's pinned geometry.
pub struct Badge<'a> {
    theme: &'a Theme,
    text: &'a str,
    variant: BadgeVariant,
}

impl<'a> Badge<'a> {
    /// Create a primary badge borrowing its theme and complete text.
    pub fn new(theme: &'a Theme, text: &'a str) -> Self {
        Self {
            theme,
            text,
            variant: BadgeVariant::Primary,
        }
    }

    /// Select a visual variant without adding interaction.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Allocate a noninteractive label and prepare its unfaded drawing.
    ///
    /// This does not submit paint or retain component state.
    pub fn show(self, ui: &mut egui::Ui) -> BadgeOutput {
        let colors = self.theme.colors;
        let transparent = Color32::TRANSPARENT;
        let (fill, foreground, border) = match self.variant {
            BadgeVariant::Primary => (
                colors.primary.background,
                colors.primary.foreground,
                transparent,
            ),
            BadgeVariant::Secondary => (
                colors.secondary.background,
                colors.secondary.foreground,
                transparent,
            ),
            BadgeVariant::Destructive => {
                let mut fill = colors.destructive.background;
                if self.theme.dark {
                    let [r, g, b, a] = fill.to_srgba_unmultiplied();
                    fill = Color32::from_rgba_unmultiplied(
                        r,
                        g,
                        b,
                        (f32::from(a) * 0.6).round() as u8,
                    );
                }
                (fill, Color32::WHITE, transparent)
            }
            BadgeVariant::Outline => (transparent, colors.background.foreground, colors.border),
            BadgeVariant::Ghost => (transparent, colors.background.foreground, transparent),
            BadgeVariant::Link => (transparent, colors.primary.background, transparent),
        };

        // CSS whitespace-nowrap collapses ASCII whitespace but preserves nonbreaking spaces.
        let text = self
            .text
            .split([' ', '\t', '\n', '\r', '\u{000c}'])
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let line_height = if text.is_empty() { 0.0 } else { LINE_HEIGHT };
        let text_len = text.len();
        let family = TextStyle::Button.resolve(ui.style()).family;
        let job = egui::text::LayoutJob {
            text,
            sections: vec![egui::text::LayoutSection {
                leading_space: 0.0,
                byte_range: egui::text::ByteIndex(0)..egui::text::ByteIndex(text_len),
                format: TextFormat {
                    font_id: FontId::new(FONT_SIZE, family),
                    line_height: Some(LINE_HEIGHT),
                    color: foreground,
                    ..Default::default()
                },
            }],
            wrap: egui::text::TextWrapping {
                max_width: f32::INFINITY,
                ..Default::default()
            },
            break_on_newline: false,
            ..Default::default()
        };
        let galley = ui.painter().layout_job(job);

        // Padding plus the reserved one-point border, including transparent borders.
        let inset = vec2(8.0 + 1.0, 2.0 + 1.0);
        let size = vec2(galley.size().x + 2.0 * inset.x, line_height + 2.0 * inset.y);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Label, response.enabled(), self.text)
        });

        // Intersect before constructing the group: unbounded parent clips remain
        // valid, and a long clipped badge does not request a huge offscreen surface.
        let bounds = rect.expand(1.0).intersect(ui.clip_rect());
        let drawing = if ui.is_rect_visible(rect) && bounds.is_positive() {
            let line_box = rect.shrink2(inset);
            let text_pos = pos2(line_box.left(), line_box.center().y - galley.size().y * 0.5);
            let background = Shape::Rect(egui::epaint::RectShape::new(
                rect,
                CornerRadius::same(11),
                fill,
                Stroke::new(1.0, border),
                StrokeKind::Inside,
            ));
            Some(
                Group::new(
                    bounds,
                    1.0,
                    vec![
                        Item::Shape(egui::epaint::ClippedShape::new(bounds, background)),
                        Item::Shape(egui::epaint::ClippedShape::new(
                            rect.intersect(bounds),
                            Shape::galley(text_pos, galley, foreground),
                        )),
                    ],
                )
                .expect("badge geometry and intersected clips are finite and positive"),
            )
        } else {
            None
        };

        BadgeOutput { response, drawing }
    }
}

/// A static label response and its frame-local, unfaded drawing.
#[must_use = "submit the drawing with paint or include it in a parent Group"]
pub struct BadgeOutput {
    /// A hover-only label response, not a click, drag, or focus target.
    pub response: Response,
    /// The unfaded drawing, absent when the badge is not visible.
    pub drawing: Option<Group>,
}

impl BadgeOutput {
    /// Submit using a same-context, same-layer painter.
    ///
    /// Capture the painter before entering a disabled scope: a static span does
    /// not receive egui's disabled fade. Actual parent opacity is applied once
    /// by the compositor. For overlapping siblings, assemble a parent `Group`
    /// instead of submitting each child independently.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "badge painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "badge painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}
