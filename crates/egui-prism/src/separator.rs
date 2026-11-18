//! Noninteractive separators with caller-owned layout and explicit drawing.
//!
//! ```
//! use egui_prism::{compositing, egui, separator::{Orientation, Separator}, theme::Theme};
//!
//! fn content(ui: &mut egui::Ui, theme: &Theme) -> Result<(), compositing::Error> {
//!     Separator::new(theme).show(ui).paint(ui.painter())?;
//!     ui.allocate_ui_with_layout(
//!         egui::vec2(240.0, 48.0), egui::Layout::left_to_right(egui::Align::Center),
//!         |ui| Separator::new(theme).orientation(Orientation::Vertical)
//!             .decorative(false).show(ui).paint(ui.painter()),
//!     ).inner?;
//!     Ok(())
//! }
//! ```

use crate::{
    compositing::{self, Error, Group, Item},
    theme::Theme,
};
use egui::{Response, Sense, Shape, vec2};

/// The direction of the one-logical-point separator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Fill the parent's available width.
    #[default]
    Horizontal,
    /// Fill the parent's available height.
    Vertical,
}

/// A separator borrowing its theme, without retained state or interaction.
pub struct Separator<'a> {
    theme: &'a Theme,
    orientation: Orientation,
    decorative: bool,
}

impl<'a> Separator<'a> {
    /// Create a horizontal, decorative separator.
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            orientation: Orientation::Horizontal,
            decorative: true,
        }
    }

    /// Choose which available parent extent the separator fills.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Omit separator semantics when decorative, or expose its role and direction.
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    /// Allocate within a bounded parent and prepare unfaded, clipped drawing.
    ///
    /// Adds no padding or margin; normal parent item spacing still applies.
    /// Sizing passes report zero intrinsic length. The caller must constrain
    /// the chosen axis in actual layout, especially inside scroll areas.
    ///
    /// # Panics
    /// Panics when the parent's chosen extent is unbounded outside a sizing pass.
    pub fn show(self, ui: &mut egui::Ui) -> SeparatorOutput {
        let extent = if ui.is_sizing_pass() {
            0.0
        } else {
            let available = ui.available_size_before_wrap();
            match self.orientation {
                Orientation::Horizontal => available.x,
                Orientation::Vertical => available.y,
            }
        };
        assert!(
            extent.is_finite(),
            "separator requires a bounded parent extent"
        );
        let size = match self.orientation {
            Orientation::Horizontal => vec2(extent.max(0.0), 1.0),
            Orientation::Vertical => vec2(1.0, extent.max(0.0)),
        };
        // Justified parents may reserve more space than the line needs. Register
        // only the aligned line so hover, accessibility and drawing bounds agree.
        let (id, allocation) = ui.allocate_space(size);
        let rect = ui.layout().align_size_within_rect(size, allocation);
        let mut response = ui.interact(rect, id, Sense::hover());
        response.set_intrinsic_size(size);
        if !self.decorative {
            response.widget_info(|| egui::WidgetInfo::new(egui::Role::Splitter));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_orientation(match self.orientation {
                    Orientation::Horizontal => egui::accesskit::Orientation::Horizontal,
                    Orientation::Vertical => egui::accesskit::Orientation::Vertical,
                });
            });
        }

        let pixels_per_point = ui.pixels_per_point();
        let antialias = ui.ctx().tessellation_options(|options| {
            if options.feathering {
                0.5 * options.feathering_size_in_pixels / pixels_per_point
            } else {
                0.0
            }
        });
        let bounds = rect.expand(antialias).intersect(ui.clip_rect());
        let drawing = if rect.is_positive() && ui.is_rect_visible(bounds) && bounds.is_positive() {
            // Thin RectShapes become pixel-snapped line segments in epaint.
            // A filled path preserves this rectangle at fractional scale, and
            // expanded bounds keep its antialiasing inside the compositing target.
            let shape = Shape::convex_polygon(
                vec![
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                ],
                self.theme.colors.border,
                egui::Stroke::NONE,
            );
            Some(
                Group::new(
                    bounds,
                    1.0,
                    vec![Item::Shape(egui::epaint::ClippedShape::new(bounds, shape))],
                )
                .expect("separator geometry and intersected clips are finite and positive"),
            )
        } else {
            None
        };
        SeparatorOutput { response, drawing }
    }
}

/// The noninteractive response and frame-local drawing for a separator.
#[must_use = "submit the drawing with paint or include it in a parent Group"]
pub struct SeparatorOutput {
    /// A hover-only response with no click, drag, or keyboard focus behavior.
    pub response: Response,
    /// The unfaded drawing, absent for clipped or zero-length separators.
    pub drawing: Option<Group>,
}

impl SeparatorOutput {
    /// Submit with a same-context, same-layer painter, applying parent opacity once.
    ///
    /// Capture the painter before a disabled scope to avoid fading this static
    /// separator. For shared sibling opacity, include the drawing in a parent group.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "separator painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "separator painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}
