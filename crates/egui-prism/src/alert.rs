//! Inline notices with CSS minimum sizing, caller-owned content and explicit drawing.
//!
//! ```
//! use egui_prism::{alert::{Alert, AlertVariant}, egui, theme::Theme};
//! egui::__run_test_ui(|ui| {
//!     let theme = Theme::light();
//!     let notice = Alert::new(&theme, "save-error")
//!         .title("Unable to save")
//!         .description("Your changes are still available in this window.")
//!         .variant(AlertVariant::Destructive)
//!         .show(ui);
//!     // Submit notice.drawing through the host compositor in this frame.
//! });
//! ```

mod text;

use egui::{
    Color32, FontId, IdSalt, Image, Rect, Response, Sense, Shape, Stroke, TextFormat, TextStyle,
    vec2,
};

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};

/// The caller-selected visual presentation, independent of application policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    /// Card surface, ordinary title and muted description.
    #[default]
    Default,
    /// Card surface with destructive-colored title, icon and description.
    Destructive,
}

/// An inline alert borrowing its theme, title, description and optional image.
///
/// The caller owns content, stable identity, image resources and any actions.
/// This component retains no state, starts no timers and never dismisses itself.
/// Width fills the finite UI width with a 34-point minimum; the title elides to one line,
/// and a plain-text description wraps. Install fonts with [`crate::theme::install_fonts`].
#[must_use = "call show and submit the returned drawing"]
pub struct Alert<'theme, 'a> {
    theme: &'theme Theme,
    id_salt: IdSalt,
    title: Option<&'theme str>,
    description: Option<&'theme str>,
    icon: Option<Image<'a>>,
    variant: AlertVariant,
}

impl<'theme, 'a> Alert<'theme, 'a> {
    /// Create an empty notice with an identity stable across content changes.
    pub fn new(theme: &'theme Theme, id_salt: impl egui::AsIdSalt) -> Self {
        Self {
            theme,
            id_salt: IdSalt::new(id_salt),
            title: None,
            description: None,
            icon: None,
            variant: AlertVariant::Default,
        }
    }

    /// Set a single-line title, preserving its complete accessible text.
    pub fn title(mut self, title: &'theme str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set wrapping plain text with the reference's 20-point description line height.
    pub fn description(mut self, description: &'theme str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set a decorative 16-point image, inheriting the title color unless tinted.
    pub fn icon(mut self, icon: impl Into<Image<'a>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Choose the visual presentation without inferring success or error policy.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Allocate a noninteractive assertive alert and prepare its unfaded drawing.
    ///
    /// The caller must provide finite available width, even in scrolling layouts.
    /// The border box is at least 34 points wide. Description min-content may
    /// overflow the box; the parent UI clip controls visibility, not the Alert.
    /// Clipping never removes accessible content.
    pub fn show(self, ui: &mut egui::Ui) -> AlertOutput {
        let width = ui.available_width();
        assert!(width.is_finite(), "alert requires a finite available width");
        let width = width.max(34.0);
        let colors = self.theme.colors;
        let foreground = match self.variant {
            AlertVariant::Default => colors.surface.foreground,
            AlertVariant::Destructive => colors.destructive.background,
        };
        let description_color = match self.variant {
            AlertVariant::Default => colors.muted.foreground,
            AlertVariant::Destructive => {
                let [r, g, b, a] = foreground.to_srgba_unmultiplied();
                Color32::from_rgba_unmultiplied(r, g, b, (f32::from(a) * 0.9).round() as u8)
            }
        };
        let inset = vec2(17.0, 13.0); // Border plus px-4 / py-3.
        let icon_column = if self.icon.is_some() { 28.0 } else { 0.0 };
        let text_width = (width - 2.0 * inset.x - icon_column).max(0.0);
        // Shape the full normalized input before selecting legal CSS line breaks.
        // egui's ordinary wrapper can emergency-split words and prioritizes dash
        // candidates differently from the reference browser.
        let natural_description = self.description.map(|text| {
            layout_text(
                ui,
                text,
                f32::INFINITY,
                TextStyle::Body,
                description_color,
                false,
            )
        });
        let (description, column_width) = match natural_description {
            Some(natural) => {
                let (galley, width) = text::description(ui, natural, text_width);
                (Some(galley), width)
            }
            None => (None, text_width),
        };
        let title = self
            .title
            .map(|text| layout_text(ui, text, column_width, TextStyle::Button, foreground, true));
        let title_height = title.as_ref().map_or(0.0, |text| {
            if text.job.text.is_empty() {
                16.0
            } else {
                text.size().y.max(16.0)
            }
        });
        let description_height = description.as_ref().map_or(0.0, |text| {
            if text.job.text.is_empty() {
                0.0
            } else {
                text.size().y
            }
        });
        let first_row_height = if self.title.is_some() {
            title_height
        } else {
            description_height
        }
        .max(if self.icon.is_some() { 16.0 } else { 0.0 });
        let description_offset = if self.title.is_some() && self.description.is_some() {
            first_row_height + 2.0
        } else {
            0.0
        };
        let content_height = if self.title.is_some() && self.description.is_some() {
            description_offset + description_height
        } else {
            first_row_height
        };
        let size = vec2(width, 2.0 * inset.y + content_height);
        let (_, allocated) = ui.allocate_space(size);
        let rect = ui.layout().align_size_within_rect(size, allocated);
        let id = ui.make_persistent_id(self.id_salt);
        let mut response = ui.interact(rect, id, Sense::hover());
        response.set_intrinsic_size(size);
        // AccessKit live-region adapters announce label changes. A supplementary
        // description alone is not announcement content, especially without a title.
        let label = [self.title, self.description]
            .into_iter()
            .flatten()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Alert, response.enabled(), &label)
        });
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_live(egui::accesskit::Live::Assertive);
            node.set_live_atomic();
        });

        let parent_clip = ui.clip_rect();
        let drawing = if parent_clip.is_positive() {
            let background = Shape::Rect(egui::epaint::RectShape::new(
                rect,
                egui::CornerRadius::same(10),
                colors.surface.background,
                Stroke::new(1.0, colors.border),
                egui::StrokeKind::Inside,
            ));
            let mut items = vec![Item::Shape(egui::epaint::ClippedShape::new(
                parent_clip,
                background,
            ))];
            let content_origin = rect.min + inset;
            // Keep fixed padding and the icon slot even below their intrinsic footprint.
            let content_clip = parent_clip;
            if let Some(mut icon) = self.icon {
                if icon.image_options().tint == Color32::WHITE {
                    icon = icon.tint(foreground);
                }
                let icon_rect =
                    Rect::from_min_size(content_origin + vec2(0.0, 2.0), vec2(16.0, 16.0));
                if content_clip.is_positive() {
                    items.extend(
                        crate::button::image_shapes(ui, &icon, icon_rect)
                            .into_iter()
                            .map(|shape| {
                                Item::Shape(egui::epaint::ClippedShape::new(
                                    content_clip.intersect(icon_rect),
                                    shape,
                                ))
                            }),
                    );
                }
            }
            let text_origin = content_origin + vec2(icon_column, 0.0);
            let text_clip = parent_clip;
            if text_clip.is_positive() {
                let title_clip = Rect::from_min_size(text_origin, vec2(column_width, title_height))
                    .intersect(parent_clip);
                for (galley, offset, color, clip) in [
                    (title, 0.0, foreground, title_clip),
                    (
                        description,
                        description_offset,
                        description_color,
                        text_clip,
                    ),
                ] {
                    if let Some(galley) = galley.filter(|galley| galley.num_vertices > 0) {
                        items.push(Item::Shape(egui::epaint::ClippedShape::new(
                            clip,
                            Shape::galley(text_origin + vec2(0.0, offset), galley, color),
                        )));
                    }
                }
            }
            let bounds = items.iter().fold(Rect::NOTHING, |bounds, item| {
                let Item::Shape(shape) = item else {
                    unreachable!("Alert emits shapes")
                };
                bounds.union(
                    shape
                        .shape
                        .visual_bounding_rect()
                        .intersect(shape.clip_rect),
                )
            });
            if bounds.is_positive() {
                for item in &mut items {
                    let Item::Shape(shape) = item else {
                        unreachable!("Alert emits shapes")
                    };
                    shape.clip_rect = shape.clip_rect.intersect(bounds);
                }
                Some(Group::new(bounds, 1.0, items).expect("finite alert geometry and clips"))
            } else {
                None
            }
        } else {
            None
        };
        AlertOutput { response, drawing }
    }
}

fn layout_text(
    ui: &egui::Ui,
    text: &str,
    width: f32,
    style: TextStyle,
    color: Color32,
    title: bool,
) -> std::sync::Arc<egui::Galley> {
    // CSS normal whitespace collapses ASCII spaces, but not nonbreaking spaces.
    let text = text
        .split([' ', '\t', '\n', '\r', '\u{000c}'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let length = text.len();
    let job = egui::text::LayoutJob {
        text,
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: egui::text::ByteIndex(0)..egui::text::ByteIndex(length),
            format: TextFormat {
                font_id: FontId::new(14.0, style.resolve(ui.style()).family),
                line_height: Some(20.0),
                extra_letter_spacing: if title { -0.35 } else { 0.0 },
                color,
                ..Default::default()
            },
        }],
        wrap: egui::text::TextWrapping {
            max_width: width,
            max_rows: if title { 1 } else { usize::MAX },
            break_anywhere: false,
            ..Default::default()
        },
        break_on_newline: false,
        ..Default::default()
    };
    ui.painter().layout_job(job)
}

/// A hover-only response and frame-local drawing for a caller-owned inline notice.
#[must_use = "submit the drawing with paint or include it in a parent Group"]
pub struct AlertOutput {
    /// The exact notice bounds, with no click, drag or focus behavior.
    pub response: Response,
    /// The unfaded drawing, absent when outside the parent clip.
    pub drawing: Option<Group>,
}

impl AlertOutput {
    /// Submit through a same-context, same-layer painter captured before disabling.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "alert painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "alert painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}
